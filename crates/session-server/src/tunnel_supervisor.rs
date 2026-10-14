//! Background retry owner. Process liveness is distinct from WAN readiness.
use super::{ControlConfig,OwnedTunnel,ProcessState};
use std::{path::PathBuf,sync::mpsc,thread,time::{Duration,Instant}};
use tokio::sync::watch;

#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize)]
#[serde(rename_all="snake_case")]
pub enum TunnelFailure { Start,Status,Stop }
#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize)]
#[serde(tag="state",rename_all="snake_case")]
pub enum TunnelStatus {
    Starting,Running,Backoff {attempt:u32,delay_ms:u64,failure:TunnelFailure},
    Stopping,Stopped,Failed {failure:TunnelFailure},
}

/// No child output, hostname, credential path or secret enters status updates.
/// Drop requests cancellation without waiting on the UI/executor thread.
pub struct TunnelSupervisor {
    stop:Option<mpsc::Sender<()>>,worker:Option<thread::JoinHandle<()>>,
    status:watch::Receiver<TunnelStatus>,
}
impl TunnelSupervisor {
    /// Does not provision anything. The caller must supply dedicated verified
    /// executable/configuration and a private runtime directory.
    pub fn start(executable:PathBuf,runtime_dir:PathBuf,config:ControlConfig)->Result<Self,&'static str> {
        Self::spawn(move ||OwnedTunnel::start(&executable,&runtime_dir,&config),Policy::default())
    }
    /// Submits every lifecycle transition directly into the existing bounded
    /// Host journal. A weak reference cannot extend the server lifetime.
    pub fn start_observed(executable:PathBuf,runtime_dir:PathBuf,config:ControlConfig,host:&std::sync::Arc<crate::Host>)->Result<Self,&'static str> {
        Self::spawn_observed(move ||OwnedTunnel::start(&executable,&runtime_dir,&config),Policy::default(),Some(std::sync::Arc::downgrade(host)))
    }
    pub(super) fn from_deployment(factory:impl FnMut()->Result<OwnedTunnel,&'static str>+Send+'static,host:&std::sync::Arc<crate::Host>)->Result<Self,&'static str> {
        Self::spawn_observed(factory,Policy::default(),Some(std::sync::Arc::downgrade(host)))
    }
    fn spawn<P:ManagedProcess+'static>(factory:impl FnMut()->Result<P,&'static str>+Send+'static,policy:Policy)->Result<Self,&'static str> {
        Self::spawn_observed(factory,policy,None)
    }
    fn spawn_observed<P:ManagedProcess+'static>(factory:impl FnMut()->Result<P,&'static str>+Send+'static,policy:Policy,host:Option<std::sync::Weak<crate::Host>>)->Result<Self,&'static str> {
        let (stop_tx,stop_rx)=mpsc::channel();let (status_tx,status)=watch::channel(TunnelStatus::Starting);
        let worker=thread::Builder::new().name("together-control-tunnel".into())
            .spawn(move ||run(factory,stop_rx,Reporter {status:status_tx,host},policy)).map_err(|_|"tunnel_worker_start_failed")?;
        Ok(Self {stop:Some(stop_tx),worker:Some(worker),status})
    }
    pub fn subscribe(&self)->watch::Receiver<TunnelStatus> {self.status.clone()}
    /// Waits for actual worker/child cleanup off the executor, with a five-second
    /// caller deadline. On timeout the worker still owns cleanup; it is not killed.
    pub async fn shutdown(mut self)->Result<(),&'static str> {
        self.request_stop();
        let worker=self.worker.take().ok_or("tunnel_worker_missing")?;
        let joined=tokio::task::spawn_blocking(move ||worker.join());
        tokio::time::timeout(Duration::from_secs(5),joined).await
            .map_err(|_|"tunnel_shutdown_timeout")?
            .map_err(|_|"tunnel_shutdown_join_failed")?
            .map_err(|_|"tunnel_worker_failed")?;
        if *self.status.borrow()!=TunnelStatus::Stopped {return Err("tunnel_shutdown_failed");}
        Ok(())
    }
    fn request_stop(&mut self) {if let Some(stop)=self.stop.take() {let _=stop.send(());}}
}
impl Drop for TunnelSupervisor {fn drop(&mut self) {self.request_stop();}}

trait ManagedProcess:Send {fn state(&mut self)->Result<ProcessState,&'static str>;fn stop(&mut self)->Result<(),&'static str>;}
impl ManagedProcess for OwnedTunnel {
    fn state(&mut self)->Result<ProcessState,&'static str> {OwnedTunnel::state(self)}
    fn stop(&mut self)->Result<(),&'static str> {OwnedTunnel::stop(self)}
}
#[derive(Clone,Copy)] struct Policy {poll:Duration,base:Duration,cap:Duration,healthy:Duration}
impl Default for Policy {fn default()->Self {Self {poll:Duration::from_millis(250),base:Duration::from_secs(1),cap:Duration::from_secs(30),healthy:Duration::from_secs(60)}}}
impl Policy {
    fn delay(self,attempt:u32)->Duration {self.base.saturating_mul(1u32.checked_shl(attempt.saturating_sub(1).min(31)).unwrap_or(u32::MAX)).min(self.cap)}
}
fn cancelled(stop:&mpsc::Receiver<()>,wait:Duration,status:&Reporter)->bool {
    let started=Instant::now();
    loop {
        if status.host_stopped() {return true;}
        let remaining=wait.saturating_sub(started.elapsed());
        if !matches!(stop.recv_timeout(remaining.min(Duration::from_millis(250))),Err(mpsc::RecvTimeoutError::Timeout)) {return true;}
        if started.elapsed()>=wait {return status.host_stopped();}
    }
}
struct Reporter {status:watch::Sender<TunnelStatus>,host:Option<std::sync::Weak<crate::Host>>}
impl Reporter {
    fn host_stopped(&self)->bool {
        self.host.as_ref().is_some_and(|weak|weak.upgrade().map_or(true,|host|*host.stopping.borrow()))
    }
    fn send(&self,status:TunnelStatus) {
        // Record transitions at their origin; a watch subscriber may coalesce
        // rapid updates. Journal producers are bounded/nonblocking.
        if let Some(host)=self.host.as_ref().and_then(|host|host.upgrade()) {host.record_tunnel(status);}
        self.status.send_replace(status);
    }
}
fn run<P:ManagedProcess>(mut factory:impl FnMut()->Result<P,&'static str>,stop:mpsc::Receiver<()>,status:Reporter,policy:Policy) {
    let mut attempt=0u32;
    loop {
        if cancelled(&stop,Duration::ZERO,&status) {break;}
        status.send(TunnelStatus::Starting);
        let failure=match factory() {
            Err(_)=>TunnelFailure::Start,
            Ok(mut process)=>{
                let started=Instant::now();status.send(TunnelStatus::Running);
                let failure=loop {
                    if cancelled(&stop,policy.poll,&status) {
                        status.send(TunnelStatus::Stopping);
                        let final_status=if process.stop().is_ok() {TunnelStatus::Stopped}else{TunnelStatus::Failed {failure:TunnelFailure::Stop}};
                        drop(process);status.send(final_status);return;
                    }
                    match process.state() {
                        Ok(ProcessState::Running)=>{},
                        Ok(_)=>break TunnelFailure::Status,
                        Err(_)=>break TunnelFailure::Status,
                    }
                };
                // Reap old child before another spawn. A stop failure prevents
                // retries, so the supervisor never intentionally owns two children.
                if process.stop().is_err() {drop(process);status.send(TunnelStatus::Failed {failure:TunnelFailure::Stop});return;}
                drop(process);
                if started.elapsed()>=policy.healthy {attempt=0;}
                failure
            }
        };
        attempt=attempt.saturating_add(1);
        let delay=policy.delay(attempt);
        status.send(TunnelStatus::Backoff {attempt,delay_ms:delay.as_millis() as u64,failure});
        if cancelled(&stop,delay,&status) {break;}
    }
    status.send(TunnelStatus::Stopping);status.send(TunnelStatus::Stopped);
}

#[cfg(test)]
pub(super) fn supervise_test_child(factory:impl FnMut()->Result<OwnedTunnel,&'static str>+Send+'static)->TunnelSupervisor {
    TunnelSupervisor::spawn(factory,Policy::default()).unwrap()
}

#[cfg(test)] mod tests {
    use super::*;
    use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
    struct Fake {active:Arc<AtomicUsize>,stops:Arc<AtomicUsize>,exit:bool,stop_fails:bool}
    impl ManagedProcess for Fake {
        fn state(&mut self)->Result<ProcessState,&'static str> {Ok(if self.exit {ProcessState::Exited {success:false}}else{ProcessState::Running})}
        fn stop(&mut self)->Result<(),&'static str> {self.stops.fetch_add(1,Ordering::SeqCst);if self.stop_fails {Err("PRIVATE_ERROR")}else{Ok(())}}
    }
    impl Drop for Fake {fn drop(&mut self) {self.active.fetch_sub(1,Ordering::SeqCst);}}
    fn policy()->Policy {Policy {poll:Duration::from_millis(5),base:Duration::from_millis(10),cap:Duration::from_millis(40),healthy:Duration::from_secs(60)}}
    async fn wait(status:&mut watch::Receiver<TunnelStatus>,predicate:impl Fn(TunnelStatus)->bool) {
        tokio::time::timeout(Duration::from_secs(3),async {
            loop {if predicate(*status.borrow_and_update()) {return;} status.changed().await.unwrap();}
        }).await.unwrap();
    }
    #[tokio::test] async fn observed_start_failure_exports_ordered_private_lifecycle() {
        let directory=std::env::temp_dir().join(format!("together-tunnel-journal-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let host=crate::Host::new("PC_SECRET_MARKER".repeat(4),"PHONE_SECRET_MARKER".repeat(4)).unwrap();
        host.enable_diagnostics(directory.join("journal")).unwrap();
        let config=ControlConfig::new("private-host.example.com","09fdab9b-262a-481f-b5f9-3d8bb705ce58",directory.join("CREDENTIAL_SECRET_MARKER.json")).unwrap();
        let supervisor=TunnelSupervisor::start_observed(directory.join("BINARY_SECRET_MARKER.exe"),directory.clone(),config,&host).unwrap();
        let mut status=supervisor.subscribe();
        wait(&mut status,|s|matches!(s,TunnelStatus::Backoff {failure:TunnelFailure::Start,..})).await;
        supervisor.shutdown().await.unwrap();
        let text=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
        for forbidden in ["SECRET_MARKER","private-host","09fdab9b","invalid_tunnel_executable",directory.to_str().unwrap()] {
            assert!(!text.contains(forbidden));
        }
        let rows:Vec<serde_json::Value>=text.lines().map(|line|serde_json::from_str(line).unwrap()).collect();
        assert_eq!(rows.len(),4);
        let states:Vec<_>=rows.iter().map(|row|row["record"]["sample"]["status"]["state"].as_str().unwrap()).collect();
        assert_eq!(states,vec!["starting","backoff","stopping","stopped"]);
        assert!(rows.iter().all(|row|row["record"]["event"]=="tunnel"));
        assert_eq!(rows[1]["record"]["sample"]["status"]["failure"],"start");
        assert_eq!(rows[1]["record"]["sample"]["status"]["attempt"],1);
        assert_eq!(rows[1]["record"]["sample"]["status"]["delay_ms"],1000);
        assert!(rows.windows(2).all(|r|r[0]["sequence"].as_u64().unwrap()<r[1]["sequence"].as_u64().unwrap()
            && r[0]["record"]["sample"]["server_ms"].as_u64().unwrap()<=r[1]["record"]["sample"]["server_ms"].as_u64().unwrap()));
        let health=host.diagnostic_health().unwrap();assert_eq!(health.dropped_records,0);assert_eq!(health.io_errors,0);
        drop(host);
        for _ in 0..100 {if std::fs::remove_dir_all(&directory).is_ok() {break;}tokio::time::sleep(Duration::from_millis(10)).await;}
        assert!(!directory.exists());
    }
    #[tokio::test] async fn observation_does_not_keep_host_alive_or_prevent_shutdown() {
        let host=crate::Host::new("PC_SYNTHETIC_KEY".repeat(4),"PHONE_SYNTHETIC_KEY".repeat(4)).unwrap();
        let weak=Arc::downgrade(&host);
        let active=Arc::new(AtomicUsize::new(0));let stops=Arc::new(AtomicUsize::new(0));let (a,s)=(active.clone(),stops.clone());
        let supervisor=TunnelSupervisor::spawn_observed(move ||{a.fetch_add(1,Ordering::SeqCst);Ok(Fake {active:a.clone(),stops:s.clone(),exit:false,stop_fails:false})},policy(),Some(weak.clone())).unwrap();
        let mut status=supervisor.subscribe();wait(&mut status,|s|s==TunnelStatus::Running).await;
        drop(host);assert!(weak.upgrade().is_none());
        supervisor.shutdown().await.unwrap();assert_eq!(active.load(Ordering::SeqCst),0);assert_eq!(stops.load(Ordering::SeqCst),1);
    }
    #[tokio::test] async fn host_shutdown_stops_running_child_and_interrupts_long_backoff() {
        for fail_start in [false,true] {
            let host=crate::Host::new("PC_SYNTHETIC_KEY".repeat(4),"PHONE_SYNTHETIC_KEY".repeat(4)).unwrap();
            let active=Arc::new(AtomicUsize::new(0));let stops=Arc::new(AtomicUsize::new(0));let starts=Arc::new(AtomicUsize::new(0));
            let (a,s,n)=(active.clone(),stops.clone(),starts.clone());
            let supervisor=TunnelSupervisor::spawn_observed(move ||{
                n.fetch_add(1,Ordering::SeqCst);
                if fail_start {return Err("PRIVATE_FAILURE");}
                a.fetch_add(1,Ordering::SeqCst);Ok(Fake {active:a.clone(),stops:s.clone(),exit:false,stop_fails:false})
            },Policy {base:Duration::from_secs(30),cap:Duration::from_secs(30),..policy()},Some(Arc::downgrade(&host))).unwrap();
            let mut status=supervisor.subscribe();
            wait(&mut status,|s|if fail_start {matches!(s,TunnelStatus::Backoff {..})}else{s==TunnelStatus::Running}).await;
            host.request_shutdown();
            // No explicit supervisor stop yet: server lifetime must own this.
            wait(&mut status,|s|s==TunnelStatus::Stopped).await;
            supervisor.shutdown().await.unwrap();
            assert_eq!(starts.load(Ordering::SeqCst),1);assert_eq!(active.load(Ordering::SeqCst),0);
            assert_eq!(stops.load(Ordering::SeqCst),if fail_start {0}else{1});
        }
    }
    #[test] fn restart_delays_saturate_without_overflow() {
        let p=Policy::default();
        assert_eq!((1..=7).map(|n|p.delay(n).as_secs()).collect::<Vec<_>>(),vec![1,2,4,8,16,30,30]);
        assert_eq!(p.delay(u32::MAX),Duration::from_secs(30));
    }
    #[tokio::test] async fn failed_start_backs_off_and_cancellation_prevents_retry() {
        let starts=Arc::new(AtomicUsize::new(0));let seen=starts.clone();
        let supervisor=TunnelSupervisor::spawn::<Fake>(move ||{seen.fetch_add(1,Ordering::SeqCst);Err("SECRET_PATH_OR_ERROR")},Policy {base:Duration::from_secs(30),..policy()}).unwrap();
        let mut status=supervisor.subscribe();
        wait(&mut status,|s|matches!(s,TunnelStatus::Backoff {failure:TunnelFailure::Start,..})).await;
        supervisor.shutdown().await.unwrap();assert_eq!(starts.load(Ordering::SeqCst),1);
        assert_eq!(*status.borrow(),TunnelStatus::Stopped);
    }
    #[tokio::test] async fn retry_reaps_previous_child_before_new_child_and_stop_is_acknowledged() {
        let active=Arc::new(AtomicUsize::new(0));let stops=Arc::new(AtomicUsize::new(0));let starts=Arc::new(AtomicUsize::new(0));
        let (a,s,n)=(active.clone(),stops.clone(),starts.clone());
        let supervisor=TunnelSupervisor::spawn(move ||{
            assert_eq!(a.fetch_add(1,Ordering::SeqCst),0);
            Ok(Fake {active:a.clone(),stops:s.clone(),exit:n.fetch_add(1,Ordering::SeqCst)==0,stop_fails:false})
        },policy()).unwrap();
        let mut status=supervisor.subscribe();
        wait(&mut status,|s|s==TunnelStatus::Running && starts.load(Ordering::SeqCst)==2).await;
        supervisor.shutdown().await.unwrap();assert_eq!(active.load(Ordering::SeqCst),0);assert_eq!(stops.load(Ordering::SeqCst),2);
    }
    #[tokio::test] async fn drop_cancels_running_worker_and_cleans_child() {
        let active=Arc::new(AtomicUsize::new(0));let stops=Arc::new(AtomicUsize::new(0));let (a,s)=(active.clone(),stops.clone());
        let supervisor=TunnelSupervisor::spawn(move ||{a.fetch_add(1,Ordering::SeqCst);Ok(Fake {active:a.clone(),stops:s.clone(),exit:false,stop_fails:false})},policy()).unwrap();
        let mut status=supervisor.subscribe();wait(&mut status,|s|s==TunnelStatus::Running).await;
        drop(supervisor);wait(&mut status,|s|s==TunnelStatus::Stopped).await;
        assert_eq!(active.load(Ordering::SeqCst),0);assert_eq!(stops.load(Ordering::SeqCst),1);
    }
    #[tokio::test] async fn stop_failure_disables_restart_and_reports_only_fixed_status() {
        let active=Arc::new(AtomicUsize::new(0));let stops=Arc::new(AtomicUsize::new(0));let (a,s)=(active.clone(),stops.clone());
        let supervisor=TunnelSupervisor::spawn(move ||{assert_eq!(a.fetch_add(1,Ordering::SeqCst),0);Ok(Fake {active:a.clone(),stops:s.clone(),exit:true,stop_fails:true})},policy()).unwrap();
        let mut status=supervisor.subscribe();wait(&mut status,|s|matches!(s,TunnelStatus::Failed {failure:TunnelFailure::Stop})).await;
        assert_eq!(supervisor.shutdown().await,Err("tunnel_shutdown_failed"));
        assert_eq!(active.load(Ordering::SeqCst),0);assert_eq!(stops.load(Ordering::SeqCst),1);
    }
}

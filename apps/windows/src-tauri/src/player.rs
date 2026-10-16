use serde::Serialize;
use std::{path::PathBuf, sync::{Arc, Mutex, mpsc}, thread, time::Duration};
use together_player_mpv::Mpv;
use together_session_server::{Host, Action};
use together_sync_core::{Snapshot, MediaIdentity, MediaProvider, PlayerFault, READY_POSITION_TOLERANCE_MS, drift::{DriftController, Correction}};

#[derive(Clone, Default, Serialize)]
pub struct Status {
    pub initialized: bool, pub state: String, pub position_ms: u64,
    pub duration_ms: u64, pub buffering: bool, pub error: Option<String>,
    pub generation: u64, pub drift_ms: i64, pub rate: f64,
    pub sampled_server_ms: u64,
}
pub struct PlayerController {
    pub status: Arc<Mutex<Status>>, pub hwnd: usize,
    stop:tokio::sync::watch::Sender<bool>,
    native_stop:Arc<std::sync::atomic::AtomicBool>,
    lifecycle:tokio::sync::Mutex<PlayerLifecycle>,
}
struct PlayerTasks {native:thread::JoinHandle<()>,feed:tauri::async_runtime::JoinHandle<()>}
struct PlayerLifecycle {tasks:Option<PlayerTasks>,joining:Option<tokio::task::JoinHandle<Result<(),&'static str>>>,result:Option<Result<(),&'static str>>}
impl PlayerController {
    /// Admission only: final Host/source/grant fences still own later publication.
    pub fn ensure_available(&self)->Result<(),&'static str> {
        const UNAVAILABLE:&str="Native player is unavailable. Restart 2gether and try again.";
        if self.native_stop.load(std::sync::atomic::Ordering::Acquire) {return Err(UNAVAILABLE);}
        let status=self.status.lock().map_err(|_|UNAVAILABLE)?;
        if !status.initialized {
            return Err(if status.error.is_some() || status.state=="Failed" || status.state=="Stopped" {UNAVAILABLE}
                else {"Native player is still starting. Try again shortly."});
        }
        Ok(())
    }
    /// Nonblocking fallback. Native teardown remains on its owning thread.
    pub fn request_stop(&self) {
        self.native_stop.store(true,std::sync::atomic::Ordering::Release);
        self.stop.send_replace(true);
    }
    /// Join controller and native teardown off the UI thread. The five-second
    /// caller budget includes lifecycle lock contention. A deadline failure leaves
    /// the worker owning its resources; no thread is forcibly terminated.
    pub async fn shutdown(&self)->Result<(),&'static str> {
        let deadline=tokio::time::Instant::now()+Duration::from_secs(5);
        self.request_stop();
        let mut guard=tokio::time::timeout_at(deadline,self.lifecycle.lock()).await
            .map_err(|_|"player_shutdown_timeout")?;
        if let Some(result)=guard.result {return result;}
        if guard.joining.is_none() {
            let Some(tasks)=guard.tasks.take() else {return Err("player_shutdown_incomplete");};
            // Lifecycle owns this completion task. Cancelling a caller or its
            // deadline drops only the wait, not the native/feed join ownership.
            let completion_status=self.status.clone();
            guard.joining=Some(tokio::spawn(async move {
                let joined=tokio::task::spawn_blocking(move ||tasks.native.join());
                let feed=tasks.feed.await;
                let native=joined.await;
                // A std thread join error confirms that owner has ended. Do not
                // mark a timed-out or cancelled join worker as released.
                if matches!(&native,Ok(Err(_))) {
                    if let Ok(mut status)=completion_status.lock() {
                        status.initialized=false;status.state="Failed".into();
                        status.error=Some("Native player worker failed".into());
                    }
                }
                feed.map_err(|_|"player_feed_join_failed")?;
                native.map_err(|_|"player_native_join_failed")?.map_err(|_|"player_native_worker_failed")?;
                Ok(())
            }));
        }
        match tokio::time::timeout_at(deadline,guard.joining.as_mut().unwrap()).await {
            Ok(completed)=>{
                let result=completed.unwrap_or(Err("player_shutdown_completion_failed"));
                guard.joining=None;guard.result=Some(result);result
            },
            Err(_)=>Err("player_shutdown_timeout"),
        }
    }
}
impl Drop for PlayerController {fn drop(&mut self) {self.request_stop();}}
fn capture_command_failure(player:&mut Mpv,failure:&mut Option<PlayerFault>,status:&Arc<Mutex<Status>>) {
    if player.take_command_failure() && failure.is_none() {
        *failure=Some(PlayerFault::CommandFailed);
        status.lock().unwrap().error=Some("Native playback command failed".into());
    }
}

fn capture_event_failure(event:together_player_mpv::Event,generation:u64,failure:&mut Option<PlayerFault>,status:&Arc<Mutex<Status>>) {
    // Secondary events cannot rewrite the pending fault awaiting explicit recovery.
    if failure.is_some() {return;}
    if event.source_failure {
        *failure=Some(PlayerFault::DecoderFailed);
        // END_FILE source failures may have a zero event-level error. Keep a
        // fixed notice without exposing the native message or media address.
        status.lock().unwrap().error=Some("Native media playback failed".into());
    }
    else if event.error<0 && (event.generation==0 || event.generation==generation) {
        status.lock().unwrap().error=Some(format!("Native player event error {}",event.error));
        *failure=Some(PlayerFault::CommandFailed);
    }
}

#[cfg(all(test,windows))]
mod command_failure_tests {
    use super::*;
    #[test] fn current_native_source_failure_publishes_fixed_notice() {
        let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap();
        let directory=root.join("work/source-validation").join(format!("native-source-notice-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let invalid=directory.join("invalid-format.mp4");std::fs::write(&invalid,b"NOT_A_PLAYABLE_MEDIA_FILE\n").unwrap();
        let mut player=Mpv::open_with_options(&root.join("native/vendor/windows/libmpv-2.dll"),
            &[("vo","null"),("ao","null"),("mute","yes")]).unwrap();
        let status=Arc::new(Mutex::new(Status::default()));let mut failure=None;
        player.prepare_local(&invalid).unwrap();
        let deadline=std::time::Instant::now()+Duration::from_secs(3);let mut source_events=0;
        while std::time::Instant::now()<deadline {
            while let Some(event)=player.poll() {
                if event.source_failure {source_events+=1;}
                capture_event_failure(event,7,&mut failure,&status);
            }
            if source_events>0 {break;}
            thread::sleep(Duration::from_millis(5));
        }
        let notice=status.lock().unwrap().error.clone();
        capture_event_failure(together_player_mpv::Event {id:5,error:-99,generation:7,source_failure:false},7,&mut failure,&status);
        let secondary_preserved=status.lock().unwrap().error==notice;
        drop(player);
        std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "scope":"actual muted native format failure; controller event policy, not decoder crash",
            "source_failure_events":source_events,"decoder_failed_recorded":failure==Some(PlayerFault::DecoderFailed),
            "notice":notice,"secondary_event_preserves_notice":secondary_preserved,"core_released":true,
            "physical_gate_passed":false
        })).unwrap()).unwrap();
        assert!(source_events>0,"no actual native source failure observed");
        assert_eq!(failure,Some(PlayerFault::DecoderFailed));
        assert_eq!(notice.as_deref(),Some("Native media playback failed"));
        assert!(secondary_preserved);
    }
    #[test] fn later_native_format_event_preserves_pending_command_failure() {
        let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap();
        let directory=root.join("work/source-validation").join(format!("native-event-fault-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let invalid=directory.join("invalid-local-format.mp4");
        std::fs::write(&invalid,b"PRIVATE_NOT_AN_MP4_SOURCE\n").unwrap();
        let mut player=Mpv::open_with_options(&root.join("native/vendor/windows/libmpv-2.dll"),
            &[("vo","null"),("ao","null"),("mute","yes")]).unwrap();
        let status=Arc::new(Mutex::new(Status::default()));let mut failure=None;
        assert!(player.queue(7,&["PRIVATE_INVALID_COMMAND"]).is_err());
        capture_command_failure(&mut player,&mut failure,&status);
        assert_eq!(failure,Some(PlayerFault::CommandFailed));
        let original_error=status.lock().unwrap().error.clone();
        player.prepare_local(&invalid).unwrap();
        let deadline=std::time::Instant::now()+Duration::from_secs(3);let mut format_events=0;
        while std::time::Instant::now()<deadline {
            while let Some(event)=player.poll() {
                let source_failure=event.source_failure;
                capture_event_failure(event,7,&mut failure,&status);
                if source_failure {format_events+=1;}
            }
            if format_events>0 {break;}
            thread::sleep(Duration::from_millis(5));
        }
        let code=match failure {Some(PlayerFault::CommandFailed)=>"command_failed",Some(PlayerFault::DecoderFailed)=>"decoder_failed",_=>"other"};
        std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "scope":"real muted native command rejection then current-entry format event; controller event policy only",
            "format_events":format_events,"retained_fault":code,"original_error_retained":status.lock().unwrap().error==original_error,
            "decoder_crash_verified":false,"physical_gate_passed":false
        })).unwrap()).unwrap();
        assert!(format_events>0,"no actual native format event observed");
        assert_eq!(failure,Some(PlayerFault::CommandFailed),"secondary format event replaced pending native fault");
        assert_eq!(status.lock().unwrap().error,original_error);
        // Without a pending fault, the same real format failure must still be admitted.
        let mut first=None;
        capture_event_failure(together_player_mpv::Event {id:7,error:0,generation:0,source_failure:true},7,&mut first,&status);
        assert_eq!(first,Some(PlayerFault::DecoderFailed));
    }
    #[tokio::test] async fn discarded_native_result_becomes_a_persistent_authoritative_fault() {
        let dll=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../native/vendor/windows/libmpv-2.dll");
        let mut player=Mpv::open_with_options(&dll,&[("vo","null"),("ao","null")]).unwrap();
        let status=Arc::new(Mutex::new(Status::default()));let mut failure=None;
        // Exercise the very pattern the controller previously ignored.
        let _=player.queue(1,&["PRIVATE_INVALID_COMMAND"]);
        player.pause(true).unwrap();
        capture_command_failure(&mut player,&mut failure,&status);
        assert_eq!(failure,Some(PlayerFault::CommandFailed));
        assert_eq!(status.lock().unwrap().error.as_deref(),Some("Native playback command failed"));
        let host=Host::new("pc-command-failure-test-000000000000".into(),"phone-command-failure-test-00000000".into()).unwrap();
        host.attach_local_pc().await.unwrap();
        host.local_pc(None,Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
        let before=host.snapshot().await;
        host.local_pc(Some(before.generation),Action::PlayerFailed {code:failure.unwrap()}).await.unwrap();
        let faulted=host.snapshot().await;
        assert_eq!(faulted.participants["pc"].player_fault,Some(PlayerFault::CommandFailed));
        assert!(!faulted.timeline.playing);assert!(faulted.generation>before.generation);
        player.speed(1.0).unwrap();capture_command_failure(&mut player,&mut failure,&status);
        assert_eq!(failure,Some(PlayerFault::CommandFailed),"successful calls must not recover a terminal fault");
    }
}
// Explicit barriers for owned headless regressions; absent from application builds.
#[cfg(test)]
struct FeedProbe {
    hold_after_generation:std::sync::atomic::AtomicU64,
    held:std::sync::atomic::AtomicBool,
    held_generation:tokio::sync::watch::Sender<Option<u64>>,
    native_iterations:tokio::sync::watch::Sender<u64>,
    feed_iterations:tokio::sync::watch::Sender<u64>,
    hold_native:Mutex<bool>,
    native_wake:std::sync::Condvar,
    native_held:tokio::sync::watch::Sender<bool>,
    queued_generation:tokio::sync::watch::Sender<Option<u64>>,
    full_attempts:tokio::sync::watch::Sender<u64>,
    applied_seeks:Mutex<Vec<(u64,u64)>>,
    forced_rate:Mutex<Option<f64>>,
    admitted_rate:tokio::sync::watch::Sender<Option<f64>>,
    reset_rates:Mutex<Vec<(u64,f64)>>,
    hold_before_controls:std::sync::atomic::AtomicU64,
    hold_at_eof:std::sync::atomic::AtomicBool,
    sampled_eof:tokio::sync::watch::Sender<Option<(u64,u64,u64)>>,
    end_submissions:Mutex<Vec<u64>>,
    reposition_native:Mutex<Option<u64>>,
    reposition_submitted:tokio::sync::watch::Sender<bool>,
    reject_at_eof:std::sync::atomic::AtomicBool,
    scheduled_start:tokio::sync::watch::Sender<Option<(u64,u64)>>,
    applied_starts:Mutex<Vec<(u64,u64)>>,
    pending_fault:Mutex<Option<PlayerFault>>,
    reject_native_command:std::sync::atomic::AtomicBool,
    command_rejected:tokio::sync::watch::Sender<bool>,
    hold_teardown:std::sync::atomic::AtomicBool,
    teardown_held:tokio::sync::watch::Sender<bool>,
    fail_after_native_release:std::sync::atomic::AtomicBool,
    native_released:tokio::sync::watch::Sender<bool>,
    fixture_loads:tokio::sync::watch::Sender<u64>,
}
#[cfg(test)]
impl FeedProbe {
    fn new()->Arc<Self> {
        Arc::new(Self {hold_after_generation:std::sync::atomic::AtomicU64::new(u64::MAX),
            held:std::sync::atomic::AtomicBool::new(false),
            held_generation:tokio::sync::watch::channel(None).0,
            native_iterations:tokio::sync::watch::channel(0).0,
            feed_iterations:tokio::sync::watch::channel(0).0,
            hold_native:Mutex::new(false),native_wake:std::sync::Condvar::new(),
            native_held:tokio::sync::watch::channel(false).0,
            queued_generation:tokio::sync::watch::channel(None).0,
            full_attempts:tokio::sync::watch::channel(0).0,applied_seeks:Mutex::new(Vec::new()),
            forced_rate:Mutex::new(None),admitted_rate:tokio::sync::watch::channel(None).0,reset_rates:Mutex::new(Vec::new()),
            hold_before_controls:std::sync::atomic::AtomicU64::new(u64::MAX),
            hold_at_eof:std::sync::atomic::AtomicBool::new(false),sampled_eof:tokio::sync::watch::channel(None).0,
            end_submissions:Mutex::new(Vec::new()),reposition_native:Mutex::new(None),
            reposition_submitted:tokio::sync::watch::channel(false).0,reject_at_eof:std::sync::atomic::AtomicBool::new(false),
            scheduled_start:tokio::sync::watch::channel(None).0,applied_starts:Mutex::new(Vec::new()),
            pending_fault:Mutex::new(None),fixture_loads:tokio::sync::watch::channel(0).0,
            reject_native_command:std::sync::atomic::AtomicBool::new(false),command_rejected:tokio::sync::watch::channel(false).0,
            hold_teardown:std::sync::atomic::AtomicBool::new(false),teardown_held:tokio::sync::watch::channel(false).0,
            fail_after_native_release:std::sync::atomic::AtomicBool::new(false),native_released:tokio::sync::watch::channel(false).0})
    }
    fn hold_native(&self,held:bool) {*self.hold_native.lock().unwrap()=held;self.native_wake.notify_all();}
}
impl PlayerController {
    pub fn start(root: PathBuf, hwnd: usize, host: Arc<Host>) -> Self {
        Self::start_owned(root,hwnd,host,#[cfg(test)] None)
    }
    fn start_owned(root:PathBuf,hwnd:usize,host:Arc<Host>,#[cfg(test)] probe:Option<Arc<FeedProbe>>)->Self {
        #[cfg(test)] let native_probe=probe.clone();
        let (sender, receiver) = mpsc::sync_channel::<(Snapshot,Option<together_session_server::media::Grant>)>(1);
        let (feedback, mut reports) = tokio::sync::mpsc::unbounded_channel::<(u64,Action)>();
        let status = Arc::new(Mutex::new(Status { state: "Starting player".into(), rate:1.0, ..Default::default() }));
        let observed = status.clone();
        let clock = host.clone();
        let (stop,mut stop_feed)=tokio::sync::watch::channel(false);
        let native_stop=Arc::new(std::sync::atomic::AtomicBool::new(false));
        let worker_stop=native_stop.clone();
        let native=thread::Builder::new().name("together-native-player".into()).spawn(move || {
            let mut proof_log = if cfg!(debug_assertions) && std::env::var("TOGETHER_SESSION_PROOF").as_deref()==Ok("1") {
                std::fs::File::create(root.join("work/native-session-proof.jsonl")).ok()
            } else {None};
            let proof_run_id=if proof_log.is_some() {
                match std::env::var("TOGETHER_PROOF_RUN_ID").ok().and_then(|id|uuid::Uuid::parse_str(&id).ok()).filter(|id|!id.is_nil()) {
                    Some(id)=>Some(id.to_string()),
                    None=>{observed.lock().unwrap().error=Some("Proof run ID required".into());return;},
                }
            }else{None};
            let mut logged_at=0;
            let mut logged_native_state=None;
            let parent = hwnd.to_string();
            let mut options = if hwnd == 0 { vec![("vo","null"),("ao","null")] }
                else { vec![("wid",parent.as_str()),("vo","gpu"),("gpu-context","d3d11"),("hwdec","auto-safe"),("force-window","yes")] };
            if hwnd==0 || proof_log.is_some() || (cfg!(debug_assertions) && std::env::var("TOGETHER_HIDDEN_ALPHA_PROOF").as_deref()==Ok("1")) {options.push(("mute","yes"));}
            let mut player = match Mpv::open_with_options(&root.join("native/vendor/windows/libmpv-2.dll"), &options) {
                Ok(p) => p, Err(_) => {
                    let mut status=observed.lock().unwrap();
                    status.initialized=false;status.state="Failed".into();
                    status.error=Some("Native libmpv initialization failed".into());return;
                }
            };
            let mut current: Option<Snapshot> = None;
            let mut generation = u64::MAX;
            let mut media: Option<MediaIdentity> = None;
            let mut source_revision=0;
            let mut seek_pending = false;
            let mut steady_since = None;
            let mut ready_sent_at = 0;
            let mut started = false;
            let mut end_sent_at=0;
            let mut execution_sent_at=0;
            let mut native_failure:Option<PlayerFault>=None;
            let mut failure_sent_at=0;
            let mut buffering_report=None;
            let mut rate = 1.0;
            let mut drift = DriftController::default();
            loop {
                if worker_stop.load(std::sync::atomic::Ordering::Acquire) {break;}
                #[cfg(test)] if let Some(probe)=&native_probe {
                    let mut held=probe.hold_native.lock().unwrap();
                    while *held && !worker_stop.load(std::sync::atomic::Ordering::Acquire) {
                        probe.native_held.send_replace(true);
                        held=probe.native_wake.wait_timeout(held,Duration::from_millis(100)).unwrap().0;
                    }
                    probe.native_held.send_replace(false);
                }
                #[cfg(test)] if worker_stop.load(std::sync::atomic::Ordering::Acquire) {break;}
                // State/source deliveries wake the channel immediately. With no
                // selected media, a 100 Hz decoder poll has no playback deadline.
                let poll_ms=if proof_log.is_some() || current.as_ref().is_some_and(|s|s.media.is_some()) {10} else {100};
                match receiver.recv_timeout(Duration::from_millis(poll_ms)) {
                    Ok((next,source)) => {
                        // A capacity-one queue can retain an earlier pause/seek
                        // while authority advances. Reject it before native work;
                        // Full deliveries remain retryable in the feeder below.
                        if !clock.is_current_media_generation(&next) {continue;}
                        if generation != next.generation {
                            generation = next.generation; started = false; ready_sent_at = 0; steady_since = None;
                            buffering_report=None;
                            end_sent_at=0;
                            execution_sent_at=0;
                            let _ = player.pause(true); let _ = player.speed(1.0); rate=1.0; drift = DriftController::default();
                            let next_source_revision=source.as_ref().map(|grant|grant.source_revision()).unwrap_or(0);
                            // Transport preparation must not erase a failure that
                            // authority has not recorded. Recover the same source
                            // only after that exact fault is recorded and a fresh
                            // preparation authorizes recovery (matching Android).
                            let recover_failed_source=native_failure.is_some_and(|code|
                                next.prepare.is_some() && next.fault.is_none() && next.participants["pc"].player_fault==Some(code));
                            if media != next.media || source_revision!=next_source_revision || recover_failed_source {
                                media = next.media.clone();
                                if let Some(id) = &media {
                                    // Isolated fixture milestone. No fake TorBox resolver or media proxy.
                                    if id.provider == MediaProvider::Fixture && id.torrent_id == 0 && id.file_id == 0 {
                                        let path = root.join("work/media/clock-test.mp4");
                                        if player.prepare_local(&path).is_err() {
                                            observed.lock().unwrap().error=Some("Local proof media failed to load".into());
                                            native_failure=Some(PlayerFault::LoadFailed);
                                        } else {
                                            source_revision=0;native_failure=None;observed.lock().unwrap().error=None;
                                            #[cfg(test)] if let Some(probe)=&native_probe {probe.fixture_loads.send_modify(|count|*count+=1);}
                                        }
                                    } else if id.provider==MediaProvider::Torbox && source.is_some() {
                                        match source.unwrap().deliver(|url|player.prepare_url(url,generation)) {
                                            Ok(Ok(()))=>{source_revision=next_source_revision;native_failure=None;observed.lock().unwrap().error=None;},
                                            Ok(Err(_))=>{native_failure=Some(PlayerFault::LoadFailed);observed.lock().unwrap().error=Some("TorBox media failed to load".into());},
                                            Err(_)=>{generation=u64::MAX;continue;},
                                        }
                                    } else {
                                        // An old native file remains loaded until replaced.
                                        // Never let it report READY for an unresolved identity.
                                        native_failure=Some(PlayerFault::LoadFailed);
                                        observed.lock().unwrap().error=Some("Source resolver is not connected yet".into());
                                    }
                                }
                            }
                            seek_pending = next.media.is_some();
                        }
                        current = Some(next);
                    },
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {},
                }
                if worker_stop.load(std::sync::atomic::Ordering::Acquire) {break;}
                while let Some(event) = player.poll() {
                    capture_event_failure(event,generation,&mut native_failure,&observed);
                }
                #[cfg(test)] if let Some(probe)=&native_probe {
                    if let Some(position)=probe.reposition_native.lock().unwrap().take() {
                        // Test-only native position discontinuity, independent of
                        // the unchanged PC-authoritative timeline. No EOF injection.
                        let seconds=(position as f64/1000.0).to_string();
                        probe.reposition_submitted.send_replace(player.queue(generation,&["seek",&seconds,"absolute+exact"]).is_ok());
                    }
                    if probe.reject_native_command.swap(false,std::sync::atomic::Ordering::AcqRel) {
                        // The real native command API rejects this unknown name;
                        // production capture below must consume its sticky error.
                        probe.command_rejected.send_replace(player.queue(generation,&["PRIVATE_INVALID_COMMAND"]).is_err());
                    }
                }
                capture_command_failure(&mut player,&mut native_failure,&observed);
                #[cfg(test)] if let Some(probe)=&native_probe {
                    if let Some(code)=probe.pending_fault.lock().unwrap().take() {
                        native_failure=Some(code);observed.lock().unwrap().error=Some("Injected native test failure".into());
                    }
                }
                let Some(s) = &current else { continue; };
                let source_ready=native_failure.is_none() && player.source_ready();
                let duration_read = player.duration_seconds();
                let position_read_started_ms = clock.now();
                let position_read = player.position_ms();
                // A display placeholder after a failed read must never be READY.
                let file_loaded=source_ready && position_read.is_ok() && duration_read.as_ref().is_ok_and(|duration|*duration>0.0);
                let duration=duration_read.unwrap_or(0.0);
                let position=position_read.unwrap_or(0);
                // Associate time with the position read, before other property reads
                // and controller corrections. Match Android's post-read timestamp.
                let now = clock.now();
                let position_read_ms = now.saturating_sub(position_read_started_ms);
                let sampled_rate = rate;
                let cache_buffering = player.property("paused-for-cache").is_ok_and(|v| v == "yes");
                let buffering=cache_buffering || (s.media.is_some() && !file_loaded);
                let seeking = player.property("seeking").is_ok_and(|v| v == "yes");
                let paused = player.property("pause").is_ok_and(|v| v == "yes");
                let eof=player.property("eof-reached").is_ok_and(|v|v=="yes");
                #[cfg(test)] if let Some(probe)=&native_probe {
                    if eof && started && s.timeline.playing && s.prepare.is_none() &&
                        probe.hold_at_eof.swap(false,std::sync::atomic::Ordering::AcqRel) {
                        if probe.reject_at_eof.swap(false,std::sync::atomic::Ordering::AcqRel) {
                            probe.command_rejected.send_replace(player.queue(generation,&["PRIVATE_INVALID_COMMAND"]).is_err());
                            capture_command_failure(&mut player,&mut native_failure,&observed);
                        }
                        probe.sampled_eof.send_replace(Some((generation,position,(duration*1000.0) as u64)));
                        probe.hold_before_controls.store(generation,std::sync::atomic::Ordering::Release);
                    }
                    if s.prepare.is_none() && s.timeline.playing && !started && paused && now<s.timeline.anchor_ms {
                        probe.scheduled_start.send_if_modified(|value| {
                            let next=Some((generation,s.timeline.anchor_ms));
                            if *value==next {false} else {*value=next;true}
                        });
                    }
                    if probe.hold_before_controls.compare_exchange(generation,u64::MAX,
                        std::sync::atomic::Ordering::AcqRel,std::sync::atomic::Ordering::Acquire).is_ok() {
                        if let Some(value)=probe.forced_rate.lock().unwrap().take() {
                            // Controlled correction state, not an estimator failure.
                            player.speed(value).unwrap();rate=value;
                            probe.admitted_rate.send_replace(Some(value));
                        }
                        probe.hold_native(true);
                        let mut held=probe.hold_native.lock().unwrap();
                        while *held && !worker_stop.load(std::sync::atomic::Ordering::Acquire) {
                            probe.native_held.send_replace(true);
                            held=probe.native_wake.wait_timeout(held,Duration::from_millis(100)).unwrap().0;
                        }
                        probe.native_held.send_replace(false);
                    }
                }
                // Authority can advance while native properties are sampled,
                // independently of channel delivery. Keep observations alive,
                // but freeze a superseded state instead of seeking, starting or
                // correcting from it. No Host lock spans native API calls.
                if !clock.is_current_media_generation(s) {
                    started=false;steady_since=None;
                    if !paused {let _=player.pause(true);}
                    if rate!=1.0 {
                        let reset=player.speed(1.0);
                        #[cfg(test)] if reset.is_ok() {if let Some(probe)=&native_probe {
                            probe.reset_rates.lock().unwrap().push((generation,1.0));
                        }}
                        let _=reset;rate=1.0;
                    }
                    capture_command_failure(&mut player,&mut native_failure,&observed);
                } else {
                    // EOF classification must not replace an already pending
                    // terminal native result before authority records that code.
                    if native_failure.is_none() && eof && started && s.timeline.playing && s.prepare.is_none() && !seeking && duration>0.0 {
                        let end=(duration*1000.0) as u64;
                        if position.abs_diff(end)>1000 || s.timeline.position(now).saturating_add(2500)<end {
                            native_failure=Some(PlayerFault::UnexpectedEnd);
                            observed.lock().unwrap().error=Some("Native media ended unexpectedly".into());
                        }
                    }
                    if let Some(code)=native_failure {
                        let _=player.pause(true);
                        if (s.participants["pc"].player_fault!=Some(code) || s.prepare.is_some()) && now.saturating_sub(failure_sent_at)>=300 {
                            let _=feedback.send((generation,Action::PlayerFailed {code}));failure_sent_at=now;
                        }
                    } else if s.participants["pc"].player_fault.is_some() && file_loaded && now.saturating_sub(failure_sent_at)>=300 {
                        let _=feedback.send((generation,Action::PlayerRecovered));failure_sent_at=now;
                    }
                    if native_failure.is_none() && eof && started && s.timeline.playing && s.prepare.is_none() && !seeking && duration>0.0 && now.saturating_sub(end_sent_at)>=300 {
                        let delivered=feedback.send((generation,Action::Ended {position_ms:position,duration_ms:(duration*1000.0) as u64})).is_ok();
                        #[cfg(test)] if delivered {if let Some(probe)=&native_probe {probe.end_submissions.lock().unwrap().push(generation);}}
                        #[cfg(not(test))] let _=delivered;
                        end_sent_at=now;
                    }
                    let target = s.prepare.as_ref().map_or(s.timeline.position_ms, |p|p.position_ms);
                    if seek_pending && file_loaded && duration > 0.0 {
                        if player.seek(target,generation).is_ok() {
                            seek_pending=false;
                            #[cfg(test)] if let Some(probe)=&native_probe {probe.applied_seeks.lock().unwrap().push((generation,target));}
                        }
                        steady_since=None;
                        continue;
                    }
                    if native_failure.is_none() && buffering != s.participants["pc"].buffering && s.media.is_some()
                        && buffering_report!=Some((generation,buffering)) {
                        if feedback.send((generation,Action::Buffering {active:buffering})).is_ok() {
                            buffering_report=Some((generation,buffering));
                        }
                    }
                    if let Some(prep) = &s.prepare {
                        started=false;
                        if !paused { let _ = player.pause(true); }
                        if file_loaded && paused && !seek_pending && !seeking && !buffering && position.abs_diff(target)<=READY_POSITION_TOLERANCE_MS {
                            let since=*steady_since.get_or_insert(now);
                            if now.saturating_sub(since)>=150 && !prep.ready.contains("pc") && now.saturating_sub(ready_sent_at)>=300 {
                                let _ = feedback.send((generation,Action::Ready {position_ms:position})); ready_sent_at=now;
                            }
                        } else { steady_since=None; }
                    } else if native_failure.is_none() && s.timeline.playing && s.fault.is_none() && !buffering && file_loaded && !seek_pending {
                        if !started && now>=s.timeline.anchor_ms {
                            if now.saturating_sub(s.timeline.anchor_ms)>200 {
                                let _ = feedback.send((generation,Action::Seek {position_ms:s.timeline.position(now)}));
                            } else if player.pause(false).is_ok() {
                                started=true;
                                #[cfg(test)] if let Some(probe)=&native_probe {probe.applied_starts.lock().unwrap().push((generation,now));}
                            }
                        }
                        if started {
                            match drift.update(s.timeline.position(now),position,now,generation,!seeking && !paused,true) {
                                Correction::Rate(value) => { if player.speed(value).is_ok() {rate=value;} },
                                Correction::Seek(pos) => {let _ = feedback.send((generation,Action::Seek {position_ms:pos}));},
                                Correction::Hold => {},
                            }
                        }
                    } else {
                        started=false;
                        if !paused { let _ = player.pause(true); }
                        if rate!=1.0 { let _ = player.speed(1.0);rate=1.0; }
                    }
                    capture_command_failure(&mut player,&mut native_failure,&observed);
                    if native_failure.is_none() && s.prepare.is_none() && s.fault.is_none() && s.media.is_some() && file_loaded && !buffering && !seeking && !seek_pending
                        && paused!=s.timeline.playing && (!s.timeline.playing || (started && now>=s.timeline.anchor_ms))
                        && (!s.timeline.playing || position>=s.timeline.position_ms.saturating_add(20))
                        && position.abs_diff(s.timeline.position(now))<=500
                        && !s.participants["pc"].execution.as_ref().is_some_and(|e|e.generation==generation && e.playing==s.timeline.playing)
                        && now.saturating_sub(execution_sent_at)>=300 {
                        let _=feedback.send((generation,Action::Executed {playing:!paused,position_ms:position,observed_server_ms:now}));execution_sent_at=now;
                    }
                }
                let mut status=observed.lock().unwrap();
                status.initialized=true; status.position_ms=position;status.duration_ms=(duration*1000.0) as u64;
                status.buffering=buffering;status.generation=generation;status.rate=rate;
                status.sampled_server_ms=now;
                #[cfg(test)] if let Some(probe)=&native_probe {probe.native_iterations.send_modify(|count|*count+=1);}
                status.drift_ms=position as i64-s.timeline.position(now) as i64;
                status.state=if native_failure.is_some() {"Failed"} else if s.ended {"Ended"} else if duration==0.0 {"Idle"} else if buffering {"Buffering"} else if seeking {"Seeking"} else if paused {"Paused"} else {"Playing"}.into();
                let native_state=(generation,paused,seeking,buffering,file_loaded,s.media.is_some(),s.prepare.is_some(),s.ended,native_failure.is_some());
                let journal_ms=if proof_log.is_some() || s.media.is_some() {100} else {2000};
                // Keep transitions immediately, but avoid rotating useful history
                // away with ten identical idle observations every second.
                if logged_native_state!=Some(native_state) || now.saturating_sub(logged_at)>=journal_ms {
                    clock.record_native(together_session_server::diagnostics::NativeSample {
                        server_ms:now,generation,position_ms:position,target_ms:s.timeline.position(now),
                        paused,seeking,buffering,source_ready:file_loaded,rate:sampled_rate,drift_ms:status.drift_ms,
                    });
                    if let Some(log)=&mut proof_log {
                        use std::io::Write;
                        let entry=serde_json::json!({"run_id":proof_run_id,"server_ms":now,"device":"pc","generation":generation,
                            "position_ms":position,"target_ms":s.timeline.position(now),"paused":paused,
                            "buffering":buffering,"seeking":seeking,"preparing":s.prepare.is_some(),
                            "source_ready":file_loaded,"native_cache_buffering":cache_buffering,
                            "anchor_ms":s.timeline.anchor_ms,"desired_playing":s.desired_playing,
                            "rate":sampled_rate,"position_read_ms":position_read_ms,
                            "drift_ms":status.drift_ms,"error":status.error});
                        let _=writeln!(log,"{}",entry);let _=log.flush();
                    }
                    logged_at=now;logged_native_state=Some(native_state);
                }
            }
            #[cfg(test)] if let Some(probe)=&native_probe {
                let mut held=probe.hold_native.lock().unwrap();
                while probe.hold_teardown.load(std::sync::atomic::Ordering::Acquire) {
                    probe.teardown_held.send_replace(true);
                    held=probe.native_wake.wait_timeout(held,Duration::from_millis(100)).unwrap().0;
                }
                probe.teardown_held.send_replace(false);
            }
            let _=player.pause(true);drop(player);
            #[cfg(test)] if let Some(probe)=&native_probe {
                probe.native_released.send_replace(true);
                if probe.fail_after_native_release.load(std::sync::atomic::Ordering::Acquire) {
                    panic!("Controlled worker failure after native release");
                }
            }
            let mut status=observed.lock().unwrap();status.initialized=false;status.state="Stopped".into();
        }).expect("player worker");
        let health=status.clone();
        let feed_native_stop=native_stop.clone();
        let feed=tauri::async_runtime::spawn(async move {
            if *stop_feed.borrow() {return;}
            if host.attach_local_pc().await.is_err() {
                health.lock().unwrap().error=Some("PC session attachment failed".into());
                feed_native_stop.store(true,std::sync::atomic::Ordering::Release);
                return;
            }
            let mut tick=tokio::time::interval(Duration::from_millis(20));
            let mut delivered=None;
            loop {
                if *stop_feed.borrow() {break;}
                tokio::select! {
                    _=stop_feed.changed()=>break,
                    _ = tick.tick() => {
                        #[cfg(test)] if let Some(probe)=&probe {probe.feed_iterations.send_modify(|count|*count+=1);}
                        #[cfg(test)] if probe.as_ref().is_some_and(|p|p.held.load(std::sync::atomic::Ordering::Acquire)) {continue;}
                        let stale={let p=health.lock().unwrap();p.initialized && host.now().saturating_sub(p.sampled_server_ms)>1000};
                        let mut state=host.snapshot().await;
                        // An unresponsive native owner freezes playback once.
                        // Do not journal identical commands at 50 Hz or let this
                        // availability signal compete with terminal authority.
                        if stale && state.media.is_some() && !state.ended && state.fault.is_none()
                            && !state.participants["pc"].buffering && state.participants["pc"].player_fault.is_none() {
                            if host.local_pc(Some(state.generation),Action::Buffering {active:true}).await.is_ok() {
                                state=host.snapshot().await;
                            }
                        }
                        let source=host.media_grant(&state,together_session_server::media::Device::Pc);
                        let identity=(state.epoch.clone(),state.revision,state.generation,source.as_ref().map(|grant|grant.source_revision()));
                        if delivered.as_ref()==Some(&identity) {continue;}
                        match sender.try_send((state,source)) {
                            Ok(()) => {
                                #[cfg(test)] if let Some(probe)=&probe {probe.queued_generation.send_replace(Some(identity.2));}
                                #[cfg(test)] if let Some(probe)=&probe {
                                    if identity.2==probe.hold_after_generation.load(std::sync::atomic::Ordering::Acquire) {
                                        probe.held.store(true,std::sync::atomic::Ordering::Release);
                                        probe.held_generation.send_replace(Some(identity.2));
                                    }
                                }
                                delivered=Some(identity);
                            },
                            Err(mpsc::TrySendError::Disconnected(_)) => break,
                            Err(mpsc::TrySendError::Full(_)) => {
                                #[cfg(test)] if let Some(probe)=&probe {probe.full_attempts.send_modify(|count|*count+=1);}
                            },
                        }
                    },
                    report=reports.recv() => match report {
                        Some((generation,action)) => { let _=host.local_pc(Some(generation),action).await; },
                        None => break,
                    }
                }
            }
            host.detach_local_pc().await;
        });
        Self {status,hwnd,stop,native_stop,lifecycle:tokio::sync::Mutex::new(PlayerLifecycle {tasks:Some(PlayerTasks {native,feed}),joining:None,result:None})}
    }
}

#[cfg(all(test,windows))]
mod lifecycle_tests {
    use super::*;
    fn root()->PathBuf {PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap()}
    fn host()->Arc<Host> {Host::new("pc-player-lifecycle-test-0000000000".into(),"phone-player-lifecycle-test-0000000".into()).unwrap()}
    async fn loaded(player:&PlayerController,host:&Host) {
        host.local_pc(None,Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
        tokio::time::timeout(Duration::from_secs(10),async {
            loop {
                {let status=player.status.lock().unwrap();assert!(status.error.is_none());if status.initialized && status.duration_ms>0 {break;}}
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }).await.unwrap();
    }
    #[tokio::test] async fn ended_native_owners_reject_new_playback_admission() {
        let directory=root().join("work/source-validation").join(format!("native-admission-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let mut receipts=Vec::new();
        for case in ["startup_failed","joined_stop"] {
            let host=host();host.enable_diagnostics(directory.join(case).join("journal")).unwrap();
            let player=PlayerController::start(if case=="startup_failed" {directory.join("missing-runtime")} else {root()},0,host.clone());
            tokio::time::timeout(Duration::from_secs(5),async {
                loop {
                    let done={let status=player.status.lock().unwrap();if case=="startup_failed" {status.error.is_some()} else {status.initialized}};
                    if done {break;}tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }).await.unwrap();
            if case=="joined_stop" {assert_eq!(player.ensure_available(),Ok(()));}
            let shutdown=player.shutdown().await;
            let admission=player.ensure_available();
            let snapshot=host.snapshot().await;
            receipts.push(serde_json::json!({"case":case,"shutdown":shutdown,"admission":admission,
                "status":player.status.lock().unwrap().clone(),"pc_connected":snapshot.participants["pc"].connected}));
            std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&receipts).unwrap()).unwrap();
            host.request_shutdown();host.flush_diagnostics().await.unwrap();
            assert_eq!(shutdown,Ok(()));assert!(!snapshot.participants["pc"].connected);
            assert_eq!(admission,Err("Native player is unavailable. Restart 2gether and try again."));
        }
    }
    #[tokio::test] async fn failed_native_startup_is_terminal_and_releases_session_owner() {
        let directory=root().join("work/source-validation").join(format!("pc-startup-failure-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let mut receipts=Vec::new();
        for case in ["missing_library","invalid_library"] {
            let isolated=directory.join(case);
            std::fs::create_dir_all(isolated.join("native/vendor/windows")).unwrap();
            if case=="invalid_library" {
                std::fs::write(isolated.join("native/vendor/windows/libmpv-2.dll"),b"isolated invalid native DLL").unwrap();
            }
            let host=host();host.enable_diagnostics(isolated.join("journal")).unwrap();
            let player=PlayerController::start(isolated,0,host.clone());
            let before=tokio::time::timeout(Duration::from_secs(5),async {
                loop {
                    let status=player.status.lock().unwrap().clone();
                    if status.error.is_some() {break status;}
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }).await;
            // Always join before assertions, including the controlled old-status comparison.
            let shutdown=player.shutdown().await;
            let pc_connected=host.snapshot().await.participants["pc"].connected;
            let after=player.status.lock().unwrap().clone();
            receipts.push(serde_json::json!({"case":case,"before":before.as_ref().ok(),
                "after":after,"shutdown":shutdown,"pc_connected":pc_connected}));
            std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&receipts).unwrap()).unwrap();
            host.request_shutdown();host.flush_diagnostics().await.unwrap();
            let before=before.unwrap();
            assert_eq!(shutdown,Ok(()));assert!(!pc_connected);
            assert!(!before.initialized);assert_eq!(before.state,"Failed");
            assert_eq!(before.error.as_deref(),Some("Native libmpv initialization failed"));
            assert_eq!(after.state,"Failed");assert_eq!(after.error,before.error);
            assert_eq!(host.diagnostic_health().unwrap().io_errors,0);
        }
        assert_eq!(receipts.len(),2);
    }
    #[tokio::test] async fn buffering_feedback_is_not_requeued_before_authority_delivery() {
        let run_id=uuid::Uuid::new_v4();
        let directory=root().join("work/source-validation").join(format!("buffer-feedback-{run_id}"));
        let host=host();host.enable_diagnostics(directory.clone()).unwrap();
        let player=PlayerController::start(root(),0,host.clone());loaded(&player,&host).await;
        let first=host.snapshot().await.generation;
        for _ in 0..12 {
            host.local_pc(None,Action::Buffering {active:true}).await.unwrap();
            tokio::time::timeout(Duration::from_secs(3),async {
                while host.snapshot().await.participants["pc"].buffering {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            }).await.unwrap();
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        player.shutdown().await.unwrap();host.flush_diagnostics().await.unwrap();
        let rows=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
        let mut generations=std::collections::BTreeSet::new();let mut duplicates=0;
        for line in rows.lines() {
            let row:serde_json::Value=serde_json::from_str(line).unwrap();
            let event=&row["record"];let sample=&event["sample"];
            if event["event"]=="command" && sample["kind"]=="buffering" {
                let generation=sample["generation"].as_u64().unwrap();
                if generation>first && !generations.insert(generation) {duplicates+=1;}
            }
        }
        std::fs::write(directory.join("review.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "run_id":run_id,"buffering_cycles":12,"buffer_command_generations":generations.len(),
            "duplicate_generation_reports":duplicates,"scope":"headless_pc_native_buffer_feedback",
            "physical_gate_passed":false
        })).unwrap()).unwrap();
        assert!(generations.len()>=12);assert_eq!(duplicates,0);
    }
    #[tokio::test] async fn delayed_authority_delivery_does_not_repeat_buffering_reports() {
        use std::sync::atomic::Ordering;
        let run_id=uuid::Uuid::new_v4();
        let directory=std::env::var_os("TOGETHER_FEED_PRESSURE_DIR").map(PathBuf::from)
            .unwrap_or_else(||root().join("work/source-validation").join(format!("feed-pressure-{run_id}")));
        let host=host();host.enable_diagnostics(directory.join("journal")).unwrap();
        let probe=FeedProbe::new();
        let player=PlayerController::start_owned(root(),0,host.clone(),Some(probe.clone()));
        loaded(&player,&host).await;
        let mut held=probe.held_generation.subscribe();let mut observations=probe.native_iterations.subscribe();
        let mut held_generations=Vec::new();
        for _ in 0..6 {
            let generation=host.snapshot().await.generation+1;
            probe.hold_after_generation.store(generation,Ordering::Release);
            host.local_pc(None,Action::Buffering {active:true}).await.unwrap();
            tokio::time::timeout(Duration::from_secs(3),async {
                while *held.borrow_and_update()!=Some(generation) {held.changed().await.unwrap();}
            }).await.unwrap();
            let first=*observations.borrow_and_update();
            // Count completed native observations while delivery is explicitly
            // held. Sleeping alone would not prove the lag condition occurred.
            tokio::time::timeout(Duration::from_secs(3),async {
                while *observations.borrow_and_update()<first+30 {observations.changed().await.unwrap();}
            }).await.unwrap();
            assert_eq!(player.status.lock().unwrap().generation,generation);
            let current=host.snapshot().await;
            assert!(current.generation>generation && !current.participants["pc"].buffering);
            held_generations.push(generation);
            probe.hold_after_generation.store(u64::MAX,Ordering::Release);probe.held.store(false,Ordering::Release);
            tokio::time::timeout(Duration::from_secs(3),async {
                loop {
                    let observed=player.status.lock().unwrap().generation;
                    if observed==host.snapshot().await.generation {break;}
                    observations.changed().await.unwrap();
                }
            }).await.unwrap();
        }
        let error=player.status.lock().unwrap().error.clone();
        player.shutdown().await.unwrap();host.flush_diagnostics().await.unwrap();
        let text=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
        let mut counts=std::collections::BTreeMap::<u64,usize>::new();
        for line in text.lines() {
            let row:serde_json::Value=serde_json::from_str(line).unwrap();let sample=&row["record"]["sample"];
            if row["record"]["event"]=="command" && sample["kind"]=="buffering" {
                let generation=sample["generation"].as_u64().unwrap();
                if held_generations.contains(&generation) {*counts.entry(generation).or_default()+=1;}
            }
        }
        let duplicates:usize=counts.values().map(|count|count.saturating_sub(1)).sum();
        let receipt=serde_json::json!({"run_id":run_id,"scope":"muted headless PC delayed authority delivery",
            "held_generations":held_generations,"native_observations_per_hold":30,"buffer_command_counts":counts,
            "duplicate_buffer_commands":duplicates,"native_error":error.is_some(),"physical_gate_passed":false});
        std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
        assert!(error.is_none());assert_eq!(counts.len(),6);assert_eq!(duplicates,0);
    }
    #[tokio::test] async fn stalled_native_watchdog_reports_once_and_preserves_authoritative_fault() {
        let directory=std::env::var_os("TOGETHER_WATCHDOG_PROOF_DIR").map(PathBuf::from)
            .unwrap_or_else(||root().join("work/source-validation").join(format!("watchdog-{}",uuid::Uuid::new_v4())));
        let host=host();host.enable_diagnostics(directory.join("journal")).unwrap();
        let probe=FeedProbe::new();let player=PlayerController::start_owned(root(),0,host.clone(),Some(probe.clone()));
        loaded(&player,&host).await;
        let mut held=probe.native_held.subscribe();let mut ticks=probe.feed_iterations.subscribe();
        probe.hold_native(true);
        tokio::time::timeout(Duration::from_secs(3),async {
            while !*held.borrow_and_update() {held.changed().await.unwrap();}
        }).await.unwrap();
        let began=host.now();
        tokio::time::timeout(Duration::from_secs(5),async {
            while !host.snapshot().await.participants["pc"].buffering {ticks.changed().await.unwrap();}
        }).await.unwrap();
        async fn ticks_after(ticks:&mut tokio::sync::watch::Receiver<u64>,count:u64) {
            let target=*ticks.borrow_and_update()+count;
            tokio::time::timeout(Duration::from_secs(3),async {
                while *ticks.borrow_and_update()<target {ticks.changed().await.unwrap();}
            }).await.unwrap();
        }
        fn buffer_count(bytes:Vec<u8>,since:u64)->usize {
            String::from_utf8(bytes).unwrap().lines().filter(|line| {
                let row:serde_json::Value=serde_json::from_str(line).unwrap();let sample=&row["record"]["sample"];
                row["record"]["event"]=="command" && sample["kind"]=="buffering" && sample["server_ms"].as_u64().is_some_and(|at|at>=since)
            }).count()
        }
        ticks_after(&mut ticks,30).await;
        let stale_reports=buffer_count(host.export_diagnostics().await.unwrap(),began);
        // The native worker stays held. A terminal PC-authoritative fault takes
        // precedence over the watchdog, even with buffering explicitly cleared.
        host.local_pc(None,Action::PlayerFailed {code:PlayerFault::DecoderFailed}).await.unwrap();
        host.local_pc(None,Action::Buffering {active:false}).await.unwrap();
        let before_fault=buffer_count(host.export_diagnostics().await.unwrap(),began);
        ticks_after(&mut ticks,30).await;
        let after_fault=buffer_count(host.export_diagnostics().await.unwrap(),began);
        let faulted=host.snapshot().await;
        probe.hold_native(false);player.shutdown().await.unwrap();host.flush_diagnostics().await.unwrap();
        let receipt=serde_json::json!({"scope":"muted headless PC native worker hold and watchdog",
            "feed_ticks_per_phase":30,"stale_buffer_reports":stale_reports,
            "additional_reports_after_fault":after_fault-before_fault,
            "fault_buffering":faulted.participants["pc"].buffering,"physical_gate_passed":false});
        std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
        assert_eq!(stale_reports,1);assert_eq!(after_fault,before_fault);
        assert_eq!(faulted.participants["pc"].player_fault,Some(PlayerFault::DecoderFailed));
        assert!(!faulted.participants["pc"].buffering);
    }
    #[tokio::test] async fn full_snapshot_queue_retries_latest_authority_without_executing_obsolete_seek() {
        let directory=std::env::var_os("TOGETHER_FULL_QUEUE_DIR").map(PathBuf::from)
            .unwrap_or_else(||root().join("work/source-validation").join(format!("full-queue-{}",uuid::Uuid::new_v4())));
        let host=host();host.enable_diagnostics(directory.join("journal")).unwrap();
        let probe=FeedProbe::new();let player=PlayerController::start_owned(root(),0,host.clone(),Some(probe.clone()));
        loaded(&player,&host).await;
        let mut held=probe.native_held.subscribe();let mut queued=probe.queued_generation.subscribe();
        let mut full=probe.full_attempts.subscribe();let mut observations=probe.native_iterations.subscribe();
        probe.hold_native(true);
        tokio::time::timeout(Duration::from_secs(3),async {
            while !*held.borrow_and_update() {held.changed().await.unwrap();}
        }).await.unwrap();
        probe.applied_seeks.lock().unwrap().clear();
        host.local_pc(None,Action::Seek {position_ms:1000}).await.unwrap();
        let obsolete=host.snapshot().await.generation;
        tokio::time::timeout(Duration::from_secs(3),async {
            while *queued.borrow_and_update()!=Some(obsolete) {queued.changed().await.unwrap();}
        }).await.unwrap();
        host.local_pc(None,Action::Pause).await.unwrap();
        host.local_pc(None,Action::Seek {position_ms:2000}).await.unwrap();
        let latest=host.snapshot().await.generation;
        let first=*full.borrow_and_update();
        tokio::time::timeout(Duration::from_secs(3),async {
            while *full.borrow_and_update()<first+5 {full.changed().await.unwrap();}
        }).await.unwrap();
        let full_attempts=*full.borrow();probe.hold_native(false);
        tokio::time::timeout(Duration::from_secs(5),async {
            loop {
                let status=player.status.lock().unwrap().clone();
                if status.generation==latest && status.position_ms.abs_diff(2000)<=100 && status.state=="Paused" {break;}
                observations.changed().await.unwrap();
            }
        }).await.unwrap();
        let applied=probe.applied_seeks.lock().unwrap().clone();
        player.shutdown().await.unwrap();host.flush_diagnostics().await.unwrap();
        let obsolete_seeks=applied.iter().filter(|(generation,_)|*generation==obsolete).count();
        let latest_seeks=applied.iter().filter(|(generation,position)|*generation==latest && *position==2000).count();
        let receipt=serde_json::json!({"scope":"muted headless PC actual capacity-one snapshot queue",
            "full_attempts":full_attempts,"obsolete_generation":obsolete,"latest_generation":latest,
            "obsolete_native_seeks":obsolete_seeks,"latest_native_seeks":latest_seeks,"applied_seeks":applied,
            "physical_gate_passed":false});
        std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
        assert!(full_attempts>=5);assert!(latest_seeks>=1);assert_eq!(obsolete_seeks,0);
    }
    #[tokio::test] async fn authority_changed_during_native_observation_cannot_submit_old_seek() {
        use std::sync::atomic::Ordering;
        let directory=std::env::var_os("TOGETHER_CURRENT_AUTHORITY_DIR").map(PathBuf::from)
            .unwrap_or_else(||root().join("work/source-validation").join(format!("current-authority-{}",uuid::Uuid::new_v4())));
        let host=host();host.enable_diagnostics(directory.join("journal")).unwrap();
        let probe=FeedProbe::new();let player=PlayerController::start_owned(root(),0,host.clone(),Some(probe.clone()));
        loaded(&player,&host).await;
        let mut observations=probe.native_iterations.subscribe();let mut held=probe.native_held.subscribe();
        let obsolete=host.snapshot().await.generation+1;
        probe.applied_seeks.lock().unwrap().clear();
        probe.hold_before_controls.store(obsolete,Ordering::Release);
        host.local_pc(None,Action::Seek {position_ms:1000}).await.unwrap();
        tokio::time::timeout(Duration::from_secs(3),async {
            while !*held.borrow_and_update() {held.changed().await.unwrap();}
        }).await.unwrap();
        // Dequeue checkpoint has already passed. Hold new delivery while the
        // worker continues sampling the old snapshot after authority changes.
        probe.held.store(true,Ordering::Release);
        host.local_pc(None,Action::Pause).await.unwrap();
        let pause=host.snapshot().await;
        let first=*observations.borrow_and_update();probe.hold_native(false);
        tokio::time::timeout(Duration::from_secs(3),async {
            while *observations.borrow_and_update()<first+30 {observations.changed().await.unwrap();}
        }).await.unwrap();
        let applied=probe.applied_seeks.lock().unwrap().clone();
        let frozen=player.status.lock().unwrap().clone();
        probe.held.store(false,Ordering::Release);
        host.local_pc(None,Action::Seek {position_ms:2000}).await.unwrap();
        let latest=host.snapshot().await.generation;
        tokio::time::timeout(Duration::from_secs(5),async {
            loop {
                let status=player.status.lock().unwrap().clone();
                if status.generation==latest && status.state=="Paused" && status.position_ms.abs_diff(2000)<=100 {break;}
                observations.changed().await.unwrap();
            }
        }).await.unwrap();
        let final_seeks=probe.applied_seeks.lock().unwrap().clone();
        player.shutdown().await.unwrap();host.flush_diagnostics().await.unwrap();
        let obsolete_submissions=applied.iter().filter(|(generation,_)|*generation==obsolete).count();
        std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "scope":"muted headless PC authority change between native observation and controls",
            "native_observations_while_delivery_held":30,"obsolete_generation":obsolete,
            "pause_generation":pause.generation,"retained_native_generation":frozen.generation,
            "native_state_while_delivery_held":frozen.state,"obsolete_seek_submissions":obsolete_submissions,
            "final_seek_submissions":final_seeks,"latest_generation":latest,"latest_paused_position_ms":2000,
            "physical_gate_passed":false
        })).unwrap()).unwrap();
        assert_eq!(frozen.generation,obsolete);assert_eq!(obsolete_submissions,0);assert_eq!(frozen.state,"Paused");
        assert!(final_seeks.iter().any(|(generation,position)|*generation==latest && *position==2000));
    }
    #[tokio::test] async fn transport_prepare_cannot_clear_pending_native_failure_before_authority_records_it() {
        pending_fault_cases(false).await;
    }
    #[tokio::test] async fn rejected_native_command_survives_transport_prepare_until_authority_and_explicit_recovery() {
        pending_fault_cases(true).await;
    }
    async fn pending_fault_cases(actual_command:bool) {
        use std::sync::atomic::Ordering;
        let expected=if actual_command {PlayerFault::CommandFailed} else {PlayerFault::DecoderFailed};
        let environment=if actual_command {"TOGETHER_COMMAND_FAULT_DIR"} else {"TOGETHER_PENDING_FAULT_DIR"};
        let directory=std::env::var_os(environment).map(PathBuf::from)
            .unwrap_or_else(||root().join("work/source-validation").join(format!("pending-fault-{}",uuid::Uuid::new_v4())));
        let mut receipts=Vec::new();let mut invalid_cases=Vec::new();
        for case in ["seek","buffering"] {
            let host=host();host.enable_diagnostics(directory.join(case).join("journal")).unwrap();
            let probe=FeedProbe::new();let player=PlayerController::start_owned(root(),0,host.clone(),Some(probe.clone()));
            loaded(&player,&host).await;
            let mut observations=probe.native_iterations.subscribe();let mut held=probe.native_held.subscribe();
            tokio::time::timeout(Duration::from_secs(3),async {
                loop {
                    let state=host.snapshot().await;
                    if player.status.lock().unwrap().generation==state.generation && state.prepare.as_ref().is_some_and(|p|p.ready.contains("pc")) {break;}
                    observations.changed().await.unwrap();
                }
            }).await.unwrap();
            let before=host.snapshot().await;let initial_loads=*probe.fixture_loads.borrow();
            probe.hold_before_controls.store(before.generation,Ordering::Release);
            if actual_command {probe.reject_native_command.store(true,Ordering::Release);}
            else {*probe.pending_fault.lock().unwrap()=Some(expected);}
            tokio::time::timeout(Duration::from_secs(3),async {
                while !*held.borrow_and_update() {held.changed().await.unwrap();}
            }).await.unwrap();
            if actual_command {
                assert!(*probe.command_rejected.borrow(),"native command API must reject the test command");
                assert_eq!(player.status.lock().unwrap().error.as_deref(),Some("Native playback command failed"));
            }
            assert!(host.snapshot().await.participants["pc"].player_fault.is_none());
            // Failure is native-owned but feedback has not yet run. A new seek
            // creates preparation; buffering follows the same rule. Neither
            // may silently reload away that failure.
            let action=if case=="seek" {Action::Seek {position_ms:1000}} else {Action::Buffering {active:true}};
            host.local_pc(None,action).await.unwrap();
            let first=*observations.borrow_and_update();probe.hold_native(false);
            tokio::time::timeout(Duration::from_secs(4),async {
                while *observations.borrow_and_update()<first+60 {observations.changed().await.unwrap();}
            }).await.unwrap();
            let after=host.snapshot().await;let pending_loads=*probe.fixture_loads.borrow()-initial_loads;
            let recorded=after.participants["pc"].player_fault==Some(expected);
            let pending_error=player.status.lock().unwrap().error.is_some();
            let recovery=if recorded {
                host.local_pc(None,Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
                tokio::time::timeout(Duration::from_secs(5),async {
                    loop {
                        let state=host.snapshot().await;
                        if state.participants["pc"].player_fault.is_none() && state.prepare.as_ref().is_some_and(|p|p.ready.contains("pc"))
                            && player.status.lock().unwrap().error.is_none() {break;}
                        observations.changed().await.unwrap();
                    }
                }).await.unwrap();true
            } else {false};
            let recovery_loads=*probe.fixture_loads.borrow()-initial_loads-pending_loads;
            player.shutdown().await.unwrap();host.flush_diagnostics().await.unwrap();
            receipts.push(serde_json::json!({"case":case,
                "scope":if actual_command {"real muted native fixture with rejected native command and held feedback"}
                    else {"real muted native fixture with injected DecoderFailed and held feedback"},
                "fault_is_injected":!actual_command,"expected_fault":expected,"native_command_rejected":*probe.command_rejected.borrow(),"native_observations_after_transport_prepare":60,"unrecorded_fault_fixture_reloads":pending_loads,
                "authority_recorded_expected_fault":recorded,"native_error_preserved":pending_error,
                "explicit_load_recovery":recovery,"explicit_recovery_fixture_reloads":recovery_loads,
                "physical_gate_passed":false
            }));
            std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&serde_json::json!({"cases":receipts})).unwrap()).unwrap();
            if pending_loads!=0 || !recorded || !pending_error || !recovery || recovery_loads!=1 {invalid_cases.push(case);}
        }
        assert!(invalid_cases.is_empty(),"pending failure cleared in {invalid_cases:?}");
    }
    #[tokio::test] async fn idle_native_work_is_bounded_and_selected_media_still_loads() {
        let run_id=uuid::Uuid::new_v4();
        let directory=root().join("work/source-validation").join(format!("idle-native-{run_id}"));
        let host=host();host.enable_diagnostics(directory.clone()).unwrap();
        let player=PlayerController::start(root(),0,host.clone());
        let until=tokio::time::Instant::now()+Duration::from_secs(5);
        let mut observations=std::collections::BTreeSet::new();
        while tokio::time::Instant::now()<until {
            {let status=player.status.lock().unwrap();assert!(status.error.is_none());
                if status.initialized {observations.insert(status.sampled_server_ms);}}
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        // A real native worker, not a mirrored interval helper: the old 100 Hz
        // idle loop would produce hundreds of observations and ~50 journal rows.
        assert!((5..=100).contains(&observations.len()));
        host.flush_diagnostics().await.unwrap();
        let bytes=host.export_diagnostics().await.unwrap();
        let rows=String::from_utf8(bytes).unwrap();
        let native_rows=rows.lines().filter(|line| {
            let row:serde_json::Value=serde_json::from_str(line).unwrap();
            row["record"]["event"]=="native_pc" || row["event"]=="native_pc"
        }).count();
        assert!((1..=8).contains(&native_rows));
        loaded(&player,&host).await;
        let health=host.diagnostic_health().unwrap();
        assert_eq!(health.io_errors,0);assert_eq!(health.dropped_records,0);
        player.shutdown().await.unwrap();host.flush_diagnostics().await.unwrap();
        std::fs::write(directory.join("review.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "run_id":run_id,"idle_observation_seconds":5,"idle_native_observations":observations.len(),
            "idle_native_journal_rows":native_rows,"selected_fixture_loaded":true,
            "scope":"headless_pc_native_idle_and_fixture_load","physical_gate_passed":false
        })).unwrap()).unwrap();
    }
    #[tokio::test] async fn shutdown_joins_actual_native_player_and_feedback_owner() {
        let host=host();let player=PlayerController::start(root(),0,host.clone());loaded(&player,&host).await;
        let (first,second)=tokio::join!(player.shutdown(),player.shutdown());first.unwrap();second.unwrap();player.shutdown().await.unwrap();
        {let status=player.status.lock().unwrap();assert!(!status.initialized);assert_eq!(status.state,"Stopped");}
        assert!(!host.snapshot().await.participants["pc"].connected);
        let weak=Arc::downgrade(&host);drop(host);assert!(weak.upgrade().is_none());
    }
    #[tokio::test] async fn cancelled_shutdown_wait_preserves_join_completion_for_next_caller() {
        shutdown_wait_case(true).await;
    }
    #[tokio::test] async fn timed_out_shutdown_wait_can_observe_late_native_teardown() {
        shutdown_wait_case(false).await;
    }
    async fn shutdown_wait_case(cancelled:bool) {
        use std::sync::atomic::Ordering;
        let environment=if cancelled {"TOGETHER_CANCELLED_SHUTDOWN_DIR"} else {"TOGETHER_SHUTDOWN_TIMEOUT_DIR"};
        let directory=std::env::var_os(environment).map(PathBuf::from)
            .unwrap_or_else(||root().join("work/source-validation").join(format!("cancelled-shutdown-{}",uuid::Uuid::new_v4())));
        let host=host();host.enable_diagnostics(directory.join("journal")).unwrap();
        let probe=FeedProbe::new();let player=Arc::new(PlayerController::start_owned(root(),0,host.clone(),Some(probe.clone())));
        loaded(&player,&host).await;
        probe.hold_teardown.store(true,Ordering::Release);let mut held=probe.teardown_held.subscribe();
        let started=tokio::time::Instant::now();
        let closing=player.clone();let first=tokio::spawn(async move {closing.shutdown().await});
        tokio::time::timeout(Duration::from_secs(3),async {
            while !*held.borrow_and_update() {held.changed().await.unwrap();}
        }).await.unwrap();
        let first_expected=if cancelled {
            first.abort();first.await.unwrap_err().is_cancelled()
        } else {first.await.unwrap()==Err("player_shutdown_timeout")};
        let first_wait_ms=started.elapsed().as_millis();
        let state_while_owned=player.status.lock().unwrap().state.clone();
        probe.hold_teardown.store(false,Ordering::Release);probe.native_wake.notify_all();
        tokio::time::timeout(Duration::from_secs(3),async {
            loop {if player.status.lock().unwrap().state=="Stopped" {break;}tokio::time::sleep(Duration::from_millis(10)).await;}
        }).await.unwrap();
        let retried=player.shutdown().await;let final_state=player.status.lock().unwrap().clone();
        host.flush_diagnostics().await.unwrap();
        std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "scope":"real muted native PC controlled teardown hold and bounded/cancelled shutdown wait",
            "teardown_hold_is_test_barrier":true,"first_caller_cancelled":cancelled,
            "first_timeout":!cancelled,"first_outcome_as_expected":first_expected,"first_wait_ms":first_wait_ms,"state_while_native_owned":state_while_owned,
            "second_shutdown_result":retried,"final_native_state":final_state.state,
            "pc_attached_after_shutdown":host.snapshot().await.participants["pc"].connected,"physical_gate_passed":false
        })).unwrap()).unwrap();
        retried.unwrap();assert!(first_expected);
        if !cancelled {assert!((5000..8000).contains(&first_wait_ms));}
        assert!(!final_state.initialized);assert_eq!(final_state.state,"Stopped");
        assert!(!host.snapshot().await.participants["pc"].connected);
    }
    #[tokio::test] async fn concurrent_shutdown_waiter_cancellation_and_deadline_preserve_owner() {
        use std::sync::atomic::Ordering;
        let directory=std::env::var_os("TOGETHER_CONCURRENT_SHUTDOWN_DIR").map(PathBuf::from)
            .unwrap_or_else(||root().join("work/source-validation").join(format!("concurrent-shutdown-{}",uuid::Uuid::new_v4())));
        let mut receipts=Vec::new();let mut invalid=Vec::new();
        for cancelled in [true,false] {
            let host=host();let probe=FeedProbe::new();
            let player=Arc::new(PlayerController::start_owned(root(),0,host.clone(),Some(probe.clone())));
            loaded(&player,&host).await;
            probe.hold_teardown.store(true,Ordering::Release);let mut held=probe.teardown_held.subscribe();
            let closing=player.clone();let owner=tokio::spawn(async move {closing.shutdown().await});
            tokio::time::timeout(Duration::from_secs(3),async {
                while !*held.borrow_and_update() {held.changed().await.unwrap();}
            }).await.unwrap();
            assert!(player.lifecycle.try_lock().is_err(),"first caller owns lifecycle while teardown is held");
            let started=tokio::time::Instant::now();let mut waiter=Box::pin(player.shutdown());
            // Poll once to prove this caller actually queued behind the owner.
            std::future::poll_fn(|cx| {
                assert!(std::future::Future::poll(waiter.as_mut(),cx).is_pending());
                std::task::Poll::Ready(())
            }).await;
            let second=if cancelled {drop(waiter);None} else {Some(waiter.await)};
            let wait_ms=started.elapsed().as_millis();
            let owner_still_running=!owner.is_finished();
            let state_while_owned=player.status.lock().unwrap().state.clone();
            probe.hold_teardown.store(false,Ordering::Release);probe.native_wake.notify_all();
            let first=tokio::time::timeout(Duration::from_secs(3),owner).await.unwrap().unwrap();
            let late=player.shutdown().await;
            receipts.push(serde_json::json!({"cancelled_queued_waiter":cancelled,"second_result":second,
                "second_wait_ms":wait_ms,"owner_still_running_at_waiter_cancellation":owner_still_running,
                "state_while_native_owned":state_while_owned,"first_result":first,"late_result":late,
                "final_native_state":player.status.lock().unwrap().state,"pc_attached":host.snapshot().await.participants["pc"].connected}));
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&serde_json::json!({
                "scope":"real muted native PC queued shutdown waiters with explicit teardown barrier",
                "cases":receipts,"physical_gate_passed":false
            })).unwrap()).unwrap();
            if late.is_err() || (cancelled && (!owner_still_running || first.is_err())) ||
                (!cancelled && (second!=Some(Err("player_shutdown_timeout")) || !(5000..6500).contains(&wait_ms))) {invalid.push(cancelled);}
        }
        assert!(invalid.is_empty(),"queued shutdown wait behavior failed for cancelled={invalid:?}");
    }
    #[tokio::test] async fn native_join_failure_is_retained_and_status_does_not_claim_live_player() {
        use std::sync::atomic::Ordering;
        let directory=std::env::var_os("TOGETHER_NATIVE_JOIN_FAILURE_DIR").map(PathBuf::from)
            .unwrap_or_else(||root().join("work/source-validation").join(format!("native-join-failure-{}",uuid::Uuid::new_v4())));
        let host=host();host.enable_diagnostics(directory.join("journal")).unwrap();
        let probe=FeedProbe::new();let player=PlayerController::start_owned(root(),0,host.clone(),Some(probe.clone()));
        loaded(&player,&host).await;
        probe.fail_after_native_release.store(true,Ordering::Release);
        let first=player.shutdown().await;let second=player.shutdown().await;
        let status=player.status.lock().unwrap().clone();
        let native_released=*probe.native_released.borrow();let pc_connected=host.snapshot().await.participants["pc"].connected;
        host.flush_diagnostics().await.unwrap();
        std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "scope":"real muted native PC safely released before controlled Rust worker panic",
            "controlled_rust_panic":true,"native_released":native_released,"first_result":first,"cached_result":second,
            "status_initialized":status.initialized,"status_state":status.state,"status_error":status.error,
            "pc_connected":pc_connected,"physical_gate_passed":false
        })).unwrap()).unwrap();
        assert!(native_released);assert_eq!(first,Err("player_native_worker_failed"));assert_eq!(second,first);
        assert!(!pc_connected);assert!(!status.initialized);assert!(status.error.is_some());
    }
    #[tokio::test] async fn rejected_attachment_stops_new_native_worker_and_preserves_existing_pc_owner() {
        let host=host();host.attach_local_pc().await.unwrap();
        let player=PlayerController::start(root(),0,host.clone());
        tokio::time::timeout(Duration::from_secs(5),async {
            loop {
                {let status=player.status.lock().unwrap();
                    if status.state=="Stopped" {
                        assert!(!status.initialized);
                        assert_eq!(status.error.as_deref(),Some("PC session attachment failed"));break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }).await.unwrap();
        player.shutdown().await.unwrap();
        assert!(host.snapshot().await.participants["pc"].connected,"rejected owner must not detach the existing owner");
        host.detach_local_pc().await;
    }
    #[tokio::test] async fn unresolved_source_change_cannot_reuse_loaded_native_fixture_as_ready() {
        let host=host();let player=PlayerController::start(root(),0,host.clone());loaded(&player,&host).await;
        host.local_pc(None,Action::Load {media:MediaIdentity::torbox(1,2)}).await.unwrap();
        tokio::time::timeout(Duration::from_secs(5),async {
            loop {
                let state=host.snapshot().await;
                if state.participants["pc"].player_fault==Some(PlayerFault::LoadFailed) {
                    assert!(!state.timeline.playing);assert!(state.prepare.is_none());
                    assert_eq!(state.fault.as_deref(),Some("player_failed_pc"));break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }).await.unwrap();
        {let status=player.status.lock().unwrap();assert!(status.error.is_some());}
        // Returning to the supported identity is an explicit new load, not a
        // buffering report clearing a terminal source fault.
        host.local_pc(None,Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
        tokio::time::timeout(Duration::from_secs(5),async {
            loop {let state=host.snapshot().await;
                if state.participants["pc"].player_fault.is_none() && state.fault.is_none()
                    && state.prepare.as_ref().is_some_and(|p|p.ready.contains("pc")) {break;}
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }).await.unwrap();
        player.shutdown().await.unwrap();
    }
    #[tokio::test] async fn drop_requests_actual_native_cleanup_without_blocking_caller() {
        let host=host();let player=PlayerController::start(root(),0,host.clone());loaded(&player,&host).await;
        let status=player.status.clone();let weak=Arc::downgrade(&host);drop(host);drop(player);
        tokio::time::timeout(Duration::from_secs(5),async {
            loop {if weak.upgrade().is_none() && status.lock().unwrap().state=="Stopped" {break;}
                tokio::time::sleep(Duration::from_millis(10)).await;}
        }).await.unwrap();
    }
}

#[cfg(all(test,windows))]
#[path="player_authority_tests.rs"]
mod authority_tests;

#[cfg(test)]
mod admission_tests {
    use super::*;
    fn controller(status:Status)->PlayerController {
        PlayerController {status:Arc::new(Mutex::new(status)),hwnd:0,
            stop:tokio::sync::watch::channel(false).0,
            native_stop:Arc::new(std::sync::atomic::AtomicBool::new(false)),
            lifecycle:tokio::sync::Mutex::new(PlayerLifecycle {tasks:None,joining:None,result:None})}
    }
    #[test] fn unavailable_and_starting_have_fixed_feedback() {
        for state in ["Failed","Stopped"] {
            let player=controller(Status {state:state.into(),..Default::default()});
            assert_eq!(player.ensure_available(),Err("Native player is unavailable. Restart 2gether and try again."));
        }
        let player=controller(Status {state:"Starting player".into(),..Default::default()});
        assert_eq!(player.ensure_available(),Err("Native player is still starting. Try again shortly."));
    }
    #[test] fn initialized_faulted_player_remains_admitted_for_explicit_recovery() {
        for state in ["Idle","Paused","Playing","Buffering","Failed"] {
            let player=controller(Status {initialized:true,state:state.into(),error:Some("fixed pending fault".into()),..Default::default()});
            assert_eq!(player.ensure_available(),Ok(()));
        }
    }
    #[test] fn stop_request_precedes_stale_initialized_status() {
        let player=controller(Status {initialized:true,state:"Playing".into(),..Default::default()});
        assert_eq!(player.ensure_available(),Ok(()));player.request_stop();
        assert_eq!(player.ensure_available(),Err("Native player is unavailable. Restart 2gether and try again."));
    }
}

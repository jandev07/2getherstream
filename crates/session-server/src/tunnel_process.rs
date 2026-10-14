//! Own exactly one tunnel child; never discover or terminate processes by name.
use super::ControlConfig;
use std::{fs::OpenOptions,io::Write,path::{Path,PathBuf},process::{Command,Stdio}};

#[cfg(windows)]
use super::native_child::NativeChild as Child;
#[cfg(not(windows))]
use std::process::Child;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum ProcessState { Running, Exited { success:bool }, Stopped }

/// This is a process-liveness primitive, not proof of public tunnel readiness.
/// The caller must move blocking start/stop operations off the UI/async thread.
pub struct OwnedTunnel { process:OwnedProcess, config_path:PathBuf }
impl OwnedTunnel {
    /// Requires an explicitly chosen executable and existing private runtime
    /// directory. Writes only a new random config, never an existing file.
    pub fn start(executable:&Path,runtime_dir:&Path,config:&ControlConfig)->Result<Self,&'static str> {
        if !executable.is_absolute() || !executable.is_file() {return Err("invalid_tunnel_executable");}
        if !runtime_dir.is_absolute() || !runtime_dir.is_dir() {return Err("invalid_tunnel_runtime_directory");}
        let yaml=config.yaml()?;
        let config_path=runtime_dir.join(format!("together-control-{}.yml",uuid::Uuid::new_v4()));
        let mut file=OpenOptions::new().write(true).create_new(true).open(&config_path)
            .map_err(|_|"tunnel_config_create_failed")?;
        if file.write_all(yaml.as_bytes()).and_then(|_|file.sync_all()).is_err() {
            drop(file);let _=std::fs::remove_file(&config_path);return Err("tunnel_config_write_failed");
        }
        drop(file);
        let mut command=launch_command(executable,&config_path,runtime_dir);
        match OwnedProcess::spawn(&mut command) {
            Ok(process)=>Ok(Self {process,config_path}),
            Err(error)=>{let _=std::fs::remove_file(&config_path);Err(error)}
        }
    }
    pub fn state(&mut self)->Result<ProcessState,&'static str> {self.process.state()}
    /// Stops/reaps only the child handle created by start. Idempotent.
    pub fn stop(&mut self)->Result<(),&'static str> {
        self.process.stop()?;
        std::fs::remove_file(&self.config_path).or_else(|e|if e.kind()==std::io::ErrorKind::NotFound {Ok(())}else{Err(e)})
            .map_err(|_|"tunnel_config_cleanup_failed")
    }
}
impl Drop for OwnedTunnel {fn drop(&mut self) {let _=self.stop();}}

fn launch_command(executable:&Path,config_path:&Path,runtime_dir:&Path)->Command {
    let mut command=Command::new(executable);
    // No shell, token, inherited TUNNEL_* overrides, proxy variables or log path.
    command.env_clear();
    // Never inherit a caller-selected current directory during connector startup.
    command.current_dir(runtime_dir);
    for key in ["SystemRoot","WINDIR","TEMP","TMP"] {
        if let Some(value)=std::env::var_os(key) {command.env(key,value);}
    }
    command.arg("tunnel").arg("--no-autoupdate").arg("--config").arg(config_path).arg("run");
    command
}
struct OwnedProcess {child:Child,stopped:bool}
impl OwnedProcess {
    fn spawn(command:&mut Command)->Result<Self,&'static str> {
        command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        #[cfg(windows)] let child=Child::spawn_explicit(command)?;
        #[cfg(not(windows))] let child=command.spawn().map_err(|_|"tunnel_process_start_failed")?;
        Ok(Self {child,stopped:false})
    }
    fn state(&mut self)->Result<ProcessState,&'static str> {
        if self.stopped {return Ok(ProcessState::Stopped);}
        self.child.try_wait().map(|status|status.map_or(ProcessState::Running,|s|ProcessState::Exited {success:s.success()}))
            .map_err(|_|"tunnel_process_status_failed")
    }
    fn stop(&mut self)->Result<(),&'static str> {
        if self.stopped {return Ok(());}
        if self.child.try_wait().map_err(|_|"tunnel_process_status_failed")?.is_none() {
            // If exit races kill, wait still reaps it. Keep the handle on failure.
            if self.child.kill().is_err() && self.child.try_wait().ok().flatten().is_none() {
                return Err("tunnel_process_stop_failed");
            }
        }
        self.child.wait().map_err(|_|"tunnel_process_reap_failed")?;
        self.stopped=true;Ok(())
    }
}
impl Drop for OwnedProcess {fn drop(&mut self) {let _=self.stop();}}

#[cfg(test)] mod tests {
    use super::*;
    use std::fs::File;
    #[test] #[ignore] fn child_fixture() {
        if let Some(marker)=std::env::var_os("TOGETHER_TUNNEL_CHILD_MARKER") {
            File::create(marker).unwrap();
            if std::env::var_os("TOGETHER_TUNNEL_CHILD_EXIT").is_some() {return;}
            loop {std::thread::sleep(std::time::Duration::from_millis(100));}
        }
    }
    fn child(exit:bool)->(OwnedProcess,PathBuf) {
        let marker=std::env::temp_dir().join(format!("together-tunnel-child-{}",uuid::Uuid::new_v4()));
        let mut command=Command::new(std::env::current_exe().unwrap());
        command.args(["tunnel::process::tests::child_fixture","--exact","--ignored"])
            .env("TOGETHER_TUNNEL_CHILD_MARKER",&marker);
        if exit {command.env("TOGETHER_TUNNEL_CHILD_EXIT","1");}
        let process=OwnedProcess::spawn(&mut command).unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        while !marker.exists() {
            assert!(std::time::Instant::now()<deadline,"child did not become ready");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        (process,marker)
    }
    #[test] fn stop_reaps_only_owned_child_and_is_idempotent() {
        let (mut first,a)=child(false);let (mut second,b)=child(false);
        assert_eq!(first.state().unwrap(),ProcessState::Running);
        first.stop().unwrap();first.stop().unwrap();
        assert_eq!(first.state().unwrap(),ProcessState::Stopped);
        assert!(first.child.try_wait().unwrap().is_some());
        assert_eq!(second.state().unwrap(),ProcessState::Running);
        second.stop().unwrap();std::fs::remove_file(a).unwrap();std::fs::remove_file(b).unwrap();
    }
    #[test] fn natural_exit_is_observed_and_reaped() {
        let (mut process,marker)=child(true);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        while process.state().unwrap()==ProcessState::Running {
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(process.state().unwrap(),ProcessState::Exited {success:true});
        process.stop().unwrap();std::fs::remove_file(marker).unwrap();
    }
    #[test] fn drop_stops_child_and_removes_only_its_config() {
        let (process,marker)=child(false);
        let owned=std::env::temp_dir().join(format!("together-config-{}",uuid::Uuid::new_v4()));
        let unrelated=std::env::temp_dir().join(format!("together-unrelated-{}",uuid::Uuid::new_v4()));
        File::create(&owned).unwrap();File::create(&unrelated).unwrap();
        // Retain a native handle to the original child for a PID-reuse-safe check.
        #[cfg(windows)] let handle={
            use std::os::windows::io::{AsHandle,OwnedHandle};
            let handle:OwnedHandle=process.child.as_handle().try_clone_to_owned().unwrap();handle
        };
        drop(OwnedTunnel {process,config_path:owned.clone()});
        assert!(!owned.exists());assert!(unrelated.exists());
        #[cfg(windows)] {
            use std::os::windows::io::AsRawHandle;
            assert_eq!(unsafe {windows_sys::Win32::System::Threading::WaitForSingleObject(handle.as_raw_handle() as _,5000)},0);
            drop(handle);
        }
        std::fs::remove_file(marker).unwrap();std::fs::remove_file(unrelated).unwrap();
    }
    #[tokio::test] async fn supervisor_shutdown_reaps_real_child_and_owned_config() {
        let (process,marker)=child(false);
        let config_path=std::env::temp_dir().join(format!("together-supervised-{}",uuid::Uuid::new_v4()));
        File::create(&config_path).unwrap();
        #[cfg(windows)] let handle={
            use std::os::windows::io::AsHandle;
            process.child.as_handle().try_clone_to_owned().unwrap()
        };
        let mut owned=Some(OwnedTunnel {process,config_path:config_path.clone()});
        let supervisor=super::super::supervisor::supervise_test_child(move ||Ok(owned.take().expect("unexpected retry")));
        let mut status=supervisor.subscribe();
        tokio::time::timeout(std::time::Duration::from_secs(3),async {
            loop {
                if *status.borrow_and_update()==super::super::TunnelStatus::Running {break;}
                status.changed().await.unwrap();
            }
        }).await.unwrap();
        supervisor.shutdown().await.unwrap();
        assert_eq!(*status.borrow(),super::super::TunnelStatus::Stopped);
        assert!(!config_path.exists());
        #[cfg(windows)] {
            use std::os::windows::io::AsRawHandle;
            assert_eq!(unsafe {windows_sys::Win32::System::Threading::WaitForSingleObject(handle.as_raw_handle() as _,0)},0);
        }
        std::fs::remove_file(marker).unwrap();
    }
    #[test] fn failed_start_cleans_new_config_and_preserves_existing_files() {
        let dir=std::env::temp_dir().join(format!("together-start-failure-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let bad_exe=dir.join("invalid.exe");std::fs::write(&bad_exe,b"not executable").unwrap();
        let keep=dir.join("existing.yml");std::fs::write(&keep,b"keep unchanged").unwrap();
        let config=ControlConfig::new("together2.example.com","09fdab9b-262a-481f-b5f9-3d8bb705ce58",dir.join("absent-credentials.json")).unwrap();
        assert!(matches!(OwnedTunnel::start(&bad_exe,&dir,&config),Err("tunnel_process_start_failed")));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(),2);
        assert_eq!(std::fs::read(&keep).unwrap(),b"keep unchanged");
        assert!(matches!(OwnedTunnel::start(Path::new("cloudflared.exe"),&dir,&config),Err("invalid_tunnel_executable")));
        std::fs::remove_file(keep).unwrap();std::fs::remove_file(bad_exe).unwrap();std::fs::remove_dir(dir).unwrap();
    }
    #[test] fn command_is_control_config_only_and_does_not_inherit_overrides() {
        let exe=std::env::current_exe().unwrap();let path=std::env::temp_dir().join("control config.yml");
        let command=launch_command(&exe,&path,std::env::temp_dir().as_path());
        assert_eq!(command.get_current_dir(),Some(std::env::temp_dir().as_path()));
        let args:Vec<_>=command.get_args().collect();
        assert_eq!(args,vec![std::ffi::OsStr::new("tunnel"),std::ffi::OsStr::new("--no-autoupdate"),
            std::ffi::OsStr::new("--config"),path.as_os_str(),std::ffi::OsStr::new("run")]);
        assert!(command.get_envs().all(|(key,_)|["SystemRoot","WINDIR","TEMP","TMP"].iter().any(|allowed|key==std::ffi::OsStr::new(allowed))));
    }
}

//! Stable Windows atomic job launch. Only explicit Command env/args are used.
use super::WindowsJob;
use std::{ffi::OsStr,io,os::windows::{ffi::OsStrExt,io::{AsHandle,AsRawHandle,BorrowedHandle,FromRawHandle,OwnedHandle},process::ExitStatusExt},process::{Command,ExitStatus}};
use windows_sys::Win32::{Foundation::{DuplicateHandle,DUPLICATE_SAME_ACCESS},System::Threading::*};
pub(super) struct NativeChild {process:OwnedHandle,job:Option<WindowsJob>,exit:Option<ExitStatus>}
impl AsHandle for NativeChild {fn as_handle(&self)->BorrowedHandle<'_> {self.process.as_handle()}}
impl NativeChild {
    pub(super) fn spawn_explicit(command:&Command)->Result<Self,&'static str> {Self::spawn(command,None)}
    pub(super) fn spawn_with_stderr(command:&Command,stderr:&std::fs::File)->Result<Self,&'static str> {Self::spawn(command,Some(stderr))}
    fn spawn(command:&Command,stderr:Option<&std::fs::File>)->Result<Self,&'static str> {
        let job=WindowsJob::new()?;let application=wide(command.get_program())?;
        let mut line=quote(command.get_program())?;
        for arg in command.get_args() {line.push(32);line.extend(quote(arg)?);}
        line.push(0);if line.len()>32767 {return Err("tunnel_command_too_long");}
        let directory=command.get_current_dir().map(|path|wide(path.as_os_str())).transpose()?;
        let mut entries=Vec::new();
        for (key,value) in command.get_envs() {if let Some(value)=value {
            let key:Vec<u16>=key.encode_wide().collect();let value:Vec<u16>=value.encode_wide().collect();
            if key.is_empty() || key.contains(&0) || key.contains(&61) || value.contains(&0) {return Err("invalid_tunnel_environment");}
            entries.push((key,value));
        }}
        entries.sort_by_key(|(key,_)|String::from_utf16_lossy(key).to_uppercase());
        let mut environment=Vec::new();
        for (key,value) in entries {environment.extend(key);environment.push(61);environment.extend(value);environment.push(0);}
        environment.push(0);if environment.len()==1 {environment.push(0);}
        let null=std::fs::OpenOptions::new().read(true).write(true).open("NUL").map_err(|_|"tunnel_null_stream_failed")?;
        let mut raw_null=std::ptr::null_mut();
        // Inherit only a duplicate of NUL, never arbitrary parent handles.
        if unsafe {DuplicateHandle(GetCurrentProcess(),null.as_raw_handle() as _,GetCurrentProcess(),&mut raw_null,0,1,DUPLICATE_SAME_ACCESS)}==0 {return Err("tunnel_null_stream_failed");}
        let null=unsafe {OwnedHandle::from_raw_handle(raw_null as _)};
        let stderr=stderr.map(|file| {
            let mut raw=std::ptr::null_mut();
            if unsafe {DuplicateHandle(GetCurrentProcess(),file.as_raw_handle() as _,GetCurrentProcess(),&mut raw,0,1,DUPLICATE_SAME_ACCESS)}==0 {return Err("tunnel_capture_stream_failed");}
            Ok(unsafe {OwnedHandle::from_raw_handle(raw as _)})
        }).transpose()?;
        let raw_stderr=stderr.as_ref().map_or(raw_null,|handle|handle.as_raw_handle() as _);
        let jobs=[job.as_handle().as_raw_handle()];let mut handles=vec![null.as_raw_handle()];
        if let Some(handle)=stderr.as_ref() {handles.push(handle.as_raw_handle());}
        let mut attributes=Attributes::new(2)?;
        attributes.set(PROC_THREAD_ATTRIBUTE_JOB_LIST as usize,&jobs)?;
        attributes.set(PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,handles.as_slice())?;
        let mut startup:STARTUPINFOEXW=unsafe {std::mem::zeroed()};
        startup.StartupInfo.cb=std::mem::size_of_val(&startup) as u32;
        startup.StartupInfo.dwFlags=STARTF_USESTDHANDLES;
        startup.StartupInfo.hStdInput=raw_null;startup.StartupInfo.hStdOutput=raw_null;startup.StartupInfo.hStdError=raw_stderr;
        startup.lpAttributeList=attributes.pointer();
        let mut info:PROCESS_INFORMATION=unsafe {std::mem::zeroed()};
        // All pointed-to storage is retained through creation. The job is part
        // of creation itself; failure never falls back to an uncontained child.
        if unsafe {CreateProcessW(application.as_ptr(),line.as_mut_ptr(),std::ptr::null(),std::ptr::null(),1,
            CREATE_NO_WINDOW|CREATE_UNICODE_ENVIRONMENT|EXTENDED_STARTUPINFO_PRESENT,
            environment.as_ptr() as _,directory.as_ref().map_or(std::ptr::null(),|path|path.as_ptr()),
            &startup.StartupInfo,&mut info)}==0 {return Err("tunnel_process_start_failed");}
        // Successful creation transfers exactly these two handles to the caller.
        let process=unsafe {OwnedHandle::from_raw_handle(info.hProcess as _)};
        let thread=unsafe {OwnedHandle::from_raw_handle(info.hThread as _)};drop(thread);
        Ok(Self {process,job:Some(job),exit:None})
    }
    pub(super) fn try_wait(&mut self)->io::Result<Option<ExitStatus>> {
        if self.exit.is_some() {return Ok(self.exit);}
        match unsafe {WaitForSingleObject(self.process.as_raw_handle() as _,0)} {
            258=>Ok(None),0=>{
                let mut code=0;if unsafe {GetExitCodeProcess(self.process.as_raw_handle() as _,&mut code)}==0 {return Err(io::Error::other("tunnel_process_status_failed"));}
                self.exit=Some(ExitStatus::from_raw(code));self.job.take();Ok(self.exit)
            },_=>Err(io::Error::other("tunnel_process_wait_failed")),
        }
    }
    pub(super) fn kill(&mut self)->io::Result<()> {self.job.take();Ok(())}
    pub(super) fn wait(&mut self)->io::Result<ExitStatus> {
        if unsafe {WaitForSingleObject(self.process.as_raw_handle() as _,INFINITE)}!=0 {return Err(io::Error::other("tunnel_process_wait_failed"));}
        self.try_wait()?.ok_or_else(||io::Error::other("tunnel_process_status_failed"))
    }
}
impl Drop for NativeChild {fn drop(&mut self) {self.job.take();}}
#[repr(C,align(16))]
#[derive(Clone)]
struct AttributeWord([u8;16]);
struct Attributes<'a> {storage:Vec<AttributeWord>,initialized:bool,values:std::marker::PhantomData<&'a ()>}
impl<'a> Attributes<'a> {
    fn pointer(&mut self)->LPPROC_THREAD_ATTRIBUTE_LIST {self.storage.as_mut_ptr() as _}
    fn new(count:u32)->Result<Self,&'static str> {
        let mut bytes=0;unsafe {InitializeProcThreadAttributeList(std::ptr::null_mut(),count,0,&mut bytes)};
        if bytes==0 {return Err("tunnel_attributes_size_failed");}
        let mut list=Self {storage:vec![AttributeWord([0;16]);bytes.div_ceil(std::mem::size_of::<AttributeWord>())],initialized:false,values:std::marker::PhantomData};
        if unsafe {InitializeProcThreadAttributeList(list.pointer(),count,0,&mut bytes)}==0 {return Err("tunnel_attributes_init_failed");}
        list.initialized=true;Ok(list)
    }
    fn set<T:?Sized>(&mut self,key:usize,value:&'a T)->Result<(),&'static str> {
        // Borrow lifetime retains attribute payloads until the list is deleted.
        if unsafe {UpdateProcThreadAttribute(self.pointer(),0,key,value as *const _ as _,std::mem::size_of_val(value),std::ptr::null_mut(),std::ptr::null_mut())}==0 {return Err("tunnel_attributes_set_failed");}Ok(())
    }
}
impl Drop for Attributes<'_> {fn drop(&mut self) {if self.initialized {unsafe {DeleteProcThreadAttributeList(self.pointer())};}}}
fn wide(value:&OsStr)->Result<Vec<u16>,&'static str> {
    let mut value:Vec<u16>=value.encode_wide().collect();if value.contains(&0) {return Err("invalid_tunnel_process_text");}value.push(0);Ok(value)
}
/// Microsoft CRT quoting; the application path is also supplied separately.
fn quote(value:&OsStr)->Result<Vec<u16>,&'static str> {
    let value=wide(value)?;let mut result=vec![34];let mut slashes=0;
    for &unit in &value[..value.len()-1] {
        if unit==92 {slashes+=1;continue;}
        if unit==34 {result.extend(std::iter::repeat_n(92,slashes*2+1));}else{result.extend(std::iter::repeat_n(92,slashes));}
        slashes=0;result.push(unit);
    }
    result.extend(std::iter::repeat_n(92,slashes*2));result.push(34);Ok(result)
}
#[cfg(test)] mod tests {
    use super::*;
    use std::{path::{Path,PathBuf},process::{Child,Stdio},time::{Duration,Instant}};
    use std::os::windows::process::CommandExt;
    fn dir()->PathBuf {let root=std::env::temp_dir().join(format!("together-atomic-job-{}",uuid::Uuid::new_v4()));std::fs::create_dir(&root).unwrap();root}
    fn ready(path:&Path) {let until=Instant::now()+Duration::from_secs(10);while !std::fs::metadata(path).is_ok_and(|m|m.len()>0) {assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(10));}}
    fn command(fixture:&str,root:&Path)->Command {
        let mut command=Command::new(std::env::current_exe().unwrap());
        command.args([fixture,"--exact","--ignored"]).env_clear().env("TOGETHER_ATOMIC_ROOT",root);command
    }
    #[test] #[ignore] fn capture_fixture() {
        if std::env::var_os("TOGETHER_ATOMIC_ROOT").is_none(){return;}
        eprintln!("FIXED_CAPTURE_MARKER");
        loop {std::thread::sleep(Duration::from_millis(100));}
    }
    #[test] fn explicit_stderr_capture_remains_inside_the_owned_job() {
        let root=dir();let path=root.join("capture.txt");
        let capture=std::fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap();
        let mut launch=command("tunnel::native_child::tests::capture_fixture",&root);launch.arg("--nocapture");
        let mut child=NativeChild::spawn_with_stderr(&launch,&capture).unwrap();
        let until=Instant::now()+Duration::from_secs(10);
        while !std::fs::read_to_string(&path).unwrap_or_default().contains("FIXED_CAPTURE_MARKER") {
            assert!(Instant::now()<until,"explicit capture was not inherited");std::thread::sleep(Duration::from_millis(10));
        }
        assert!(child.try_wait().unwrap().is_none());child.kill().unwrap();child.wait().unwrap();
        drop(child);drop(capture);std::fs::remove_file(path).unwrap();std::fs::remove_dir(root).unwrap();
    }
    #[test] #[ignore] fn echo_fixture() {
        let Some(root)=std::env::var_os("TOGETHER_ATOMIC_ROOT") else{return;};
        let report=serde_json::json!({"args":std::env::args().skip(1).collect::<Vec<_>>(),
            "cwd":std::env::current_dir().unwrap(),
            "value":std::env::var("EXPLICIT_VALUE").unwrap(),"inherited_path_present":std::env::var_os("PATH").is_some()});
        std::fs::write(PathBuf::from(root).join("echo.json"),report.to_string()).unwrap();
    }
    #[test] #[ignore] fn leaf_fixture() {
        let Some(root)=std::env::var_os("TOGETHER_ATOMIC_ROOT") else{return;};let root=PathBuf::from(root);
        std::fs::write(root.join("leaf.pid"),std::process::id().to_string()).unwrap();let mut _grandchild=None;
        if std::env::var_os("TOGETHER_ATOMIC_DESCENDANT").is_some() {
            let grand=root.join("grandchild");std::fs::create_dir(&grand).unwrap();
            let mut command=command("tunnel::native_child::tests::leaf_fixture",&grand);
            command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).creation_flags(CREATE_NO_WINDOW);
            // Descendant is immediate; there is no assignment gate.
            _grandchild=Some(TestChild(command.spawn().unwrap()));ready(&grand.join("leaf.pid"));
            // Exit without fixture Drop so cleanup must come from the owned job.
            if std::env::var_os("TOGETHER_ATOMIC_EXIT_AFTER_DESCENDANT").is_some() {std::process::exit(37);}
        }
        loop {std::thread::sleep(Duration::from_millis(100));}
    }
    #[test] #[ignore] fn owner_fixture() {
        let Some(root)=std::env::var_os("TOGETHER_ATOMIC_ROOT") else{return;};let root=PathBuf::from(root);
        let mut command=command("tunnel::native_child::tests::leaf_fixture",&root);command.env("TOGETHER_ATOMIC_DESCENDANT","1");
        let _child=NativeChild::spawn_explicit(&command).unwrap();
        ready(&root.join("grandchild/leaf.pid"));std::fs::write(root.join("owner.ready"),b"ready").unwrap();
        loop {std::thread::sleep(Duration::from_millis(100));}
    }
    struct TestChild(Child);
    impl Drop for TestChild {fn drop(&mut self) {let _=self.0.kill();let _=self.0.wait();}}
    fn open(path:&Path)->OwnedHandle {
        let pid:u32=std::fs::read_to_string(path).unwrap().parse().unwrap();let raw=unsafe {OpenProcess(PROCESS_SYNCHRONIZE,0,pid)};assert!(!raw.is_null());unsafe {OwnedHandle::from_raw_handle(raw as _)}
    }
    #[test] fn arguments_and_explicit_environment_round_trip_without_parent_environment() {
        let root=dir();let mut command=command("tunnel::native_child::tests::echo_fixture",&root);command.current_dir(&root);
        // Extra exact filters allow difficult arguments to traverse the real CRT
        // while only the full fixture name matches a test.
        command.args(["", "spaces and unicode 🦀", "literal\"quote", "trailing\\", "before\\\"quote"]).env("EXPLICIT_VALUE","space 🦀 \" value\\");
        let expected:Vec<_>=command.get_args().map(|x|x.to_str().unwrap().to_owned()).collect();
        let mut child=NativeChild::spawn_explicit(&command).unwrap();assert!(child.wait().unwrap().success());
        let report:serde_json::Value=serde_json::from_slice(&std::fs::read(root.join("echo.json")).unwrap()).unwrap();
        assert_eq!(report["args"],serde_json::json!(expected));assert_eq!(report["value"],"space 🦀 \" value\\");
        assert_eq!(PathBuf::from(report["cwd"].as_str().unwrap()).canonicalize().unwrap(),root.canonicalize().unwrap());
        assert_eq!(report["inherited_path_present"],false);std::fs::remove_dir_all(root).unwrap();
    }
    #[test] fn process_is_created_in_its_exact_job_and_drop_terminates_it() {
        let root=dir();let child=NativeChild::spawn_explicit(&command("tunnel::native_child::tests::leaf_fixture",&root)).unwrap();let mut in_job=0;
        assert_ne!(unsafe {windows_sys::Win32::System::JobObjects::IsProcessInJob(child.process.as_raw_handle() as _,child.job.as_ref().unwrap().as_handle().as_raw_handle() as _,&mut in_job)},0);
        assert_ne!(in_job,0);let handle=child.as_handle().try_clone_to_owned().unwrap();
        drop(child);assert_eq!(unsafe {WaitForSingleObject(handle.as_raw_handle() as _,5000)},0);std::fs::remove_dir_all(root).unwrap();
    }
    #[test] fn atomic_launcher_owner_crash_kills_immediate_descendant_tree() {
        let root=dir();let mut command=command("tunnel::native_child::tests::owner_fixture",&root);
        command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).creation_flags(CREATE_NO_WINDOW);
        let mut owner=TestChild(command.spawn().unwrap());ready(&root.join("owner.ready"));
        let child=open(&root.join("leaf.pid"));let descendant=open(&root.join("grandchild/leaf.pid"));
        assert_eq!(unsafe {WaitForSingleObject(child.as_raw_handle() as _,0)},258);assert_eq!(unsafe {WaitForSingleObject(descendant.as_raw_handle() as _,0)},258);
        owner.0.kill().unwrap();owner.0.wait().unwrap();
        assert_eq!(unsafe {WaitForSingleObject(child.as_raw_handle() as _,5000)},0);assert_eq!(unsafe {WaitForSingleObject(descendant.as_raw_handle() as _,5000)},0);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test] fn natural_root_exit_reaps_descendants_and_caches_nonzero_status() {
        let root=dir();let mut command=command("tunnel::native_child::tests::leaf_fixture",&root);
        command.env("TOGETHER_ATOMIC_DESCENDANT","1").env("TOGETHER_ATOMIC_EXIT_AFTER_DESCENDANT","1");
        let mut child=NativeChild::spawn_explicit(&command).unwrap();ready(&root.join("grandchild/leaf.pid"));
        let descendant=open(&root.join("grandchild/leaf.pid"));
        // Root is already terminated, but its job is still held by NativeChild.
        assert_eq!(unsafe {WaitForSingleObject(child.process.as_raw_handle() as _,5000)},0);
        assert_eq!(unsafe {WaitForSingleObject(descendant.as_raw_handle() as _,0)},258);
        let status=child.try_wait().unwrap().unwrap();assert_eq!(status.code(),Some(37));
        assert_eq!(unsafe {WaitForSingleObject(descendant.as_raw_handle() as _,5000)},0);
        assert_eq!(child.try_wait().unwrap(),Some(status));assert_eq!(child.wait().unwrap(),status);
        drop(child);std::fs::remove_dir_all(root).unwrap();
    }
    #[test] #[ignore] fn handle_isolation_fixture() {
        use std::os::windows::fs::OpenOptionsExt;
        let Some(root)=std::env::var_os("TOGETHER_ATOMIC_ROOT") else{return;};
        let root=PathBuf::from(root);let sentinel=root.join("exclusive-sentinel");
        let file=std::fs::OpenOptions::new().write(true).create_new(true).share_mode(0).open(&sentinel).unwrap();
        let mut raw=std::ptr::null_mut();
        assert_ne!(unsafe {DuplicateHandle(GetCurrentProcess(),file.as_raw_handle() as _,GetCurrentProcess(),&mut raw,0,1,DUPLICATE_SAME_ACCESS)},0);
        let inheritable=unsafe {OwnedHandle::from_raw_handle(raw as _)};
        let mut flags=0;assert_ne!(unsafe {windows_sys::Win32::Foundation::GetHandleInformation(raw,&mut flags)},0);
        assert_ne!(flags & windows_sys::Win32::Foundation::HANDLE_FLAG_INHERIT,0);
        let child=NativeChild::spawn_explicit(&command("tunnel::native_child::tests::leaf_fixture",&root)).unwrap();
        ready(&root.join("leaf.pid"));drop(file);drop(inheritable);
        // If the child inherited the exclusive handle, reopening would fail
        // until that child exits. Confirm it is alive throughout this check.
        assert_eq!(unsafe {WaitForSingleObject(child.process.as_raw_handle() as _,0)},258);
        let reopened=std::fs::OpenOptions::new().write(true).share_mode(0).open(&sentinel).unwrap();
        assert_eq!(unsafe {WaitForSingleObject(child.process.as_raw_handle() as _,0)},258);
        drop(reopened);drop(child);
    }
    #[test] fn handle_list_excludes_unrelated_inheritable_file_handle() {
        // Isolate the inheritable sentinel from concurrent test fixtures that
        // use ordinary std::process::Command and could inherit it themselves.
        let root=dir();let mut fixture=NativeChild::spawn_explicit(&command("tunnel::native_child::tests::handle_isolation_fixture",&root)).unwrap();
        let status=fixture.wait().unwrap();drop(fixture);std::fs::remove_dir_all(root).unwrap();assert!(status.success());
    }
    #[test] #[ignore] fn copied_cloudflared_offline_ingress_validation() {
        let executable=PathBuf::from(std::env::var_os("TOGETHER_CLOUDFLARED_OFFLINE_EXE").expect("explicit copied executable required"));assert!(executable.is_absolute() && executable.is_file());
        let root=dir();let config_path=root.join("space # unicode 🦀.yml");
        let config=super::super::ControlConfig::new("together2.example.com","09fdab9b-262a-481f-b5f9-3d8bb705ce58",root.join("nonexistent-credentials.json")).unwrap();std::fs::write(&config_path,config.yaml().unwrap()).unwrap();
        let mut command=Command::new(executable);command.args(["tunnel","--no-autoupdate","--config"]).arg(&config_path).args(["ingress","validate"]).env_clear();
        for key in ["SystemRoot","WINDIR","TEMP","TMP"] {if let Some(value)=std::env::var_os(key) {command.env(key,value);}}
        let mut child=NativeChild::spawn_explicit(&command).unwrap();assert!(child.wait().unwrap().success());std::fs::remove_dir_all(root).unwrap();println!("copied_cloudflared_offline_atomic_validation_passed");
    }
}

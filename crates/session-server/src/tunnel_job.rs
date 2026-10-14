//! Windows job ownership used by the atomic native tunnel launcher.
use std::os::windows::io::{AsHandle,BorrowedHandle,FromRawHandle,OwnedHandle};
use windows_sys::Win32::System::JobObjects::{CreateJobObjectW,SetInformationJobObject,
    JobObjectExtendedLimitInformation,JOBOBJECT_EXTENDED_LIMIT_INFORMATION,JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE};

/// Unnamed, non-inheritable job. Keep its handle alive while the native launcher
/// owns the tunnel. Closing the last handle terminates associated processes.
/// This alone does not make an ordinary Command::spawn atomic or crash-safe.
pub struct WindowsJob {handle:OwnedHandle}
impl WindowsJob {
    pub fn new()->Result<Self,&'static str> {
        // Null security attributes make the job handle non-inheritable; null
        // name prevents another process opening it through a public name.
        let raw=unsafe {CreateJobObjectW(std::ptr::null(),std::ptr::null())};
        if raw.is_null() {return Err("tunnel_job_create_failed");}
        // SAFETY: successful CreateJobObjectW gives exactly one owned handle.
        let handle=unsafe {OwnedHandle::from_raw_handle(raw as _)};
        let mut limits:JOBOBJECT_EXTENDED_LIMIT_INFORMATION=unsafe {std::mem::zeroed()};
        limits.BasicLimitInformation.LimitFlags=JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: limits is the documented structure and remains alive through
        // this call. No breakaway flags or unrelated process assignment occurs.
        if unsafe {SetInformationJobObject(raw,JobObjectExtendedLimitInformation,
            &limits as *const _ as _,std::mem::size_of_val(&limits) as u32)}==0 {
            return Err("tunnel_job_limits_failed");
        }
        Ok(Self {handle})
    }
    /// Borrow for PROC_THREAD_ATTRIBUTE_JOB_LIST during atomic creation.
    pub fn as_handle(&self)->BorrowedHandle<'_> {self.handle.as_handle()}
}

#[cfg(test)] mod tests {
    use super::*;
    use std::os::windows::io::AsRawHandle;
    use std::{path::{Path,PathBuf},process::{Command,Child,Stdio},time::{Duration,Instant}};
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::{Foundation::{GetHandleInformation,HANDLE_FLAG_INHERIT},System::{
        JobObjects::AssignProcessToJobObject,Threading::{OpenProcess,WaitForSingleObject,PROCESS_SYNCHRONIZE}}};
    fn assign(job:&WindowsJob,child:&Child) {
        assert_ne!(unsafe {AssignProcessToJobObject(job.as_handle().as_raw_handle() as _,child.as_raw_handle() as _)},0);
    }
    struct ChildGuard(Child);
    impl std::ops::Deref for ChildGuard {type Target=Child;fn deref(&self)->&Child {&self.0}}
    impl std::ops::DerefMut for ChildGuard {fn deref_mut(&mut self)->&mut Child {&mut self.0}}
    impl Drop for ChildGuard {fn drop(&mut self) {let _=self.0.kill();let _=self.0.wait();}}
    fn spawn(name:&str,root:&Path,grandchild:bool)->ChildGuard {
        let mut command=Command::new(std::env::current_exe().unwrap());
        command.args([name,"--exact","--ignored"]).env("TOGETHER_JOB_PROOF_ROOT",root)
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).creation_flags(0x08000000);
        command.env_remove("TOGETHER_JOB_PROOF_GRANDCHILD");
        if grandchild {command.env("TOGETHER_JOB_PROOF_GRANDCHILD","1");}
        ChildGuard(command.spawn().unwrap())
    }
    fn wait_file(path:&Path) {
        let until=Instant::now()+Duration::from_secs(10);
        while !std::fs::metadata(path).is_ok_and(|metadata|metadata.len()>0) {assert!(Instant::now()<until,"fixture readiness timeout");std::thread::sleep(Duration::from_millis(10));}
    }
    #[test] #[ignore] fn leaf_fixture() {
        let Some(root)=std::env::var_os("TOGETHER_JOB_PROOF_ROOT") else {return;};let root=PathBuf::from(root);
        std::fs::write(root.join("leaf.pid"),std::process::id().to_string()).unwrap();
        let mut _grandchild=None;
        if std::env::var_os("TOGETHER_JOB_PROOF_GRANDCHILD").is_some() {
            wait_file(&root.join("assigned"));
            let child_root=root.join("grandchild");std::fs::create_dir(&child_root).unwrap();
            _grandchild=Some(spawn("tunnel::windows_job::tests::leaf_fixture",&child_root,false));
            // Remove the inherited flag on the grandchild's command to prevent
            // recursively spawning test descendants.
            wait_file(&child_root.join("leaf.pid"));
            std::fs::write(root.join("descendant-ready"),b"ready").unwrap();
        }
        loop {std::thread::sleep(Duration::from_millis(100));}
    }
    #[test] #[ignore] fn owner_fixture() {
        let Some(root)=std::env::var_os("TOGETHER_JOB_PROOF_ROOT") else {return;};let root=PathBuf::from(root);
        let job=WindowsJob::new().unwrap();
        let _child=spawn("tunnel::windows_job::tests::leaf_fixture",&root,true);
        wait_file(&root.join("leaf.pid"));assign(&job,&_child);
        std::fs::write(root.join("assigned"),b"ready").unwrap();
        wait_file(&root.join("descendant-ready"));std::fs::write(root.join("owner-ready"),b"ready").unwrap();
        loop {std::thread::sleep(Duration::from_millis(100));}
    }
    fn directory()->PathBuf {
        let path=std::env::temp_dir().join(format!("together-job-proof-{}",uuid::Uuid::new_v4()));std::fs::create_dir(&path).unwrap();path
    }
    fn open_process(path:&Path)->OwnedHandle {
        let pid: u32=std::fs::read_to_string(path).unwrap().parse().unwrap();
        let raw=unsafe {OpenProcess(PROCESS_SYNCHRONIZE,0,pid)};assert!(!raw.is_null());
        unsafe {OwnedHandle::from_raw_handle(raw as _)}
    }
    fn wait_handle(handle:BorrowedHandle<'_>,ms:u32)->u32 {unsafe {WaitForSingleObject(handle.as_raw_handle() as _,ms)}}
    #[test] fn closing_job_kills_assigned_child_but_preserves_unrelated_child() {
        let first=directory();let second=directory();let job=WindowsJob::new().unwrap();
        let mut flags=0;assert_ne!(unsafe {GetHandleInformation(job.as_handle().as_raw_handle() as _,&mut flags)},0);
        assert_eq!(flags&HANDLE_FLAG_INHERIT,0);
        let mut child=spawn("tunnel::windows_job::tests::leaf_fixture",&first,false);
        let mut unrelated=spawn("tunnel::windows_job::tests::leaf_fixture",&second,false);
        wait_file(&first.join("leaf.pid"));wait_file(&second.join("leaf.pid"));assign(&job,&child);
        drop(job);assert_eq!(wait_handle(child.as_handle(),5000),0);child.wait().unwrap();
        assert_eq!(wait_handle(unrelated.as_handle(),0),258);unrelated.kill().unwrap();unrelated.wait().unwrap();
        std::fs::remove_dir_all(first).unwrap();std::fs::remove_dir_all(second).unwrap();
    }
    #[test] fn forced_owner_exit_kills_child_and_inherited_descendant_without_rust_cleanup() {
        let root=directory();let mut owner=spawn("tunnel::windows_job::tests::owner_fixture",&root,false);
        wait_file(&root.join("owner-ready"));
        // Retain handles before terminating the isolated owner, preventing PID
        // reuse from changing which processes are checked.
        let child=open_process(&root.join("leaf.pid"));let grandchild=open_process(&root.join("grandchild/leaf.pid"));
        assert_eq!(wait_handle(child.as_handle(),0),258);assert_eq!(wait_handle(grandchild.as_handle(),0),258);
        owner.kill().unwrap();owner.wait().unwrap();
        assert_eq!(wait_handle(child.as_handle(),5000),0);assert_eq!(wait_handle(grandchild.as_handle(),5000),0);
        std::fs::remove_dir_all(root).unwrap();
    }
}

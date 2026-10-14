//! Dedicated local provisioning input; never read legacy paths or emit its contents.
use super::{ControlConfig,OwnedTunnel,TunnelSupervisor};
use serde::Deserialize;
use sha2::{Digest,Sha256};
use std::{fs::File,io::Read,path::{Path,PathBuf}};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {version:u32,hostname:String,tunnel_id:String,executable_sha256:String}
/// No Debug/Serialize: hostname, identity and paths stay out of diagnostics.
pub struct TunnelDeployment {directory:PathBuf,config:ControlConfig,digest:[u8;32]}
// Read-only namespace guard. This rejects existing redirects, but does not
// lock all directory handles against a concurrent/privileged namespace swap.
pub(super) fn reject_redirected(path:&Path)->Result<(),&'static str> {
    for ancestor in path.ancestors() {
        let metadata=match std::fs::symlink_metadata(ancestor) {
            Ok(metadata)=>metadata,
            Err(error) if error.kind()==std::io::ErrorKind::NotFound=>continue,
            Err(_)=>return Err("tunnel_deployment_metadata_failed"),
        };
        #[cfg(windows)] let redirected={
            use std::os::windows::fs::MetadataExt;
            metadata.file_attributes() & 0x400 != 0 // FILE_ATTRIBUTE_REPARSE_POINT
        };
        #[cfg(not(windows))] let redirected=metadata.file_type().is_symlink();
        if redirected {return Err("tunnel_deployment_redirected");}
    }
    Ok(())
}
impl TunnelDeployment {
    pub fn public_origin(&self)->String {self.config.public_origin()}
    /// Missing manifest disables startup. Malformed/unreadable provisioning fails
    /// closed. Installation/ACL/DNS provisioning are deliberately separate.
    pub fn load(directory:&Path)->Result<Option<Self>,&'static str> {
        if !directory.is_absolute() {return Err("tunnel_deployment_path_invalid");}
        reject_redirected(&directory.join("deployment.json"))?;
        let file=match File::open(directory.join("deployment.json")) {
            Ok(file)=>file,
            Err(e) if e.kind()==std::io::ErrorKind::NotFound=>return Ok(None),
            Err(_)=>return Err("tunnel_deployment_read_failed"),
        };
        let mut bytes=Vec::new();file.take(16385).read_to_end(&mut bytes).map_err(|_|"tunnel_deployment_read_failed")?;
        if bytes.is_empty() || bytes.len()>16384 {return Err("tunnel_deployment_invalid");}
        let manifest:Manifest=serde_json::from_slice(&bytes).map_err(|_|"tunnel_deployment_invalid")?;
        if manifest.version!=1 || (manifest.executable_sha256.len()!=64 || !manifest.executable_sha256.bytes().all(|b|b.is_ascii_hexdigit())) {return Err("tunnel_deployment_invalid");}
        let mut digest=[0u8;32];
        for (i,byte) in digest.iter_mut().enumerate() {
            *byte=u8::from_str_radix(&manifest.executable_sha256[i*2..i*2+2],16).map_err(|_|"tunnel_deployment_invalid")?;
        }
        let config=ControlConfig::new(&manifest.hostname,&manifest.tunnel_id,directory.join("credentials.json"))
            .map_err(|_|"tunnel_deployment_invalid")?;
        Ok(Some(Self {directory:directory.to_path_buf(),config,digest}))
    }
    fn verified_executable(&self)->Result<File,&'static str> {
        let path=self.directory.join("cloudflared.exe");
        reject_redirected(&path)?;
        let mut options=std::fs::OpenOptions::new();options.read(true);
        #[cfg(windows)] {
            use std::os::windows::fs::OpenOptionsExt;
            // FILE_SHARE_READ only: retain the handle across process creation so
            // ordinary file writes/deletion cannot replace the verified image.
            options.share_mode(1);
        }
        let mut file=options.open(path).map_err(|_|"tunnel_binary_open_failed")?;
        if !file.metadata().map_err(|_|"tunnel_binary_read_failed")?.is_file() {return Err("tunnel_binary_invalid");}
        let mut hash=Sha256::new();let mut buffer=[0u8;65536];let mut total=0usize;
        loop {
            let count=file.read(&mut buffer).map_err(|_|"tunnel_binary_read_failed")?;
            if count==0 {break;}
            total+=count;if total>128*1024*1024 {return Err("tunnel_binary_invalid");}
            hash.update(&buffer[..count]);
        }
        if total==0 || hash.finalize().as_slice()!=self.digest {return Err("tunnel_binary_mismatch");}
        Ok(file)
    }
    fn start(&self)->Result<OwnedTunnel,&'static str> {
        // Verify on every retry, not just when the manifest is first loaded.
        let _verified=self.verified_executable()?;
        reject_redirected(&self.directory.join("credentials.json"))?;
        if !self.directory.join("credentials.json").is_file() {return Err("tunnel_credentials_missing");}
        let runtime=self.directory.join("runtime");
        reject_redirected(&runtime)?;
        std::fs::create_dir_all(&runtime).map_err(|_|"tunnel_runtime_create_failed")?;
        reject_redirected(&runtime)?;
        OwnedTunnel::start(&self.directory.join("cloudflared.exe"),&runtime,&self.config)
    }
    pub fn supervise(self,host:&std::sync::Arc<crate::Host>)->Result<TunnelSupervisor,&'static str> {
        TunnelSupervisor::from_deployment(move ||self.start(),host)
    }
}

#[cfg(test)] mod tests {
    use super::*;
    fn directory()->PathBuf {
        let p=std::env::temp_dir().join(format!("together-deployment-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir(&p).unwrap();p
    }
    fn manifest(digest:String)->serde_json::Value {serde_json::json!({"version":1,"hostname":"together2.example.com",
        "tunnel_id":"09fdab9b-262a-481f-b5f9-3d8bb705ce58","executable_sha256":digest})}
    fn write(p:&Path,value:&serde_json::Value) {std::fs::write(p.join("deployment.json"),serde_json::to_vec(value).unwrap()).unwrap();}
    #[test] fn absent_is_disabled_but_invalid_or_unknown_inputs_fail_closed() {
        let p=directory();assert!(TunnelDeployment::load(&p).unwrap().is_none());
        for value in [serde_json::json!({}), {let mut m=manifest("0".repeat(64));m["token"]=serde_json::json!("PRIVATE_MARKER");m},
            manifest("é".repeat(32)),manifest("€".repeat(21)+"a"),manifest("z".repeat(64)),{let mut m=manifest("0".repeat(64));m["version"]=serde_json::json!(2);m}] {
            write(&p,&value);assert!(matches!(TunnelDeployment::load(&p),Err("tunnel_deployment_invalid")));
        }
        std::fs::write(p.join("deployment.json"),vec![b' ';16385]).unwrap();
        assert!(matches!(TunnelDeployment::load(&p),Err("tunnel_deployment_invalid")));
        std::fs::remove_file(p.join("deployment.json")).unwrap();std::fs::remove_dir(p).unwrap();
    }
    #[tokio::test] async fn deployment_failure_is_private_and_host_shutdown_owns_supervisor() {
        let p=directory();write(&p,&manifest("0".repeat(64)));
        std::fs::write(p.join("cloudflared.exe"),b"never executed").unwrap();
        let host=crate::Host::new("PC_PRIVATE_MARKER".repeat(4),"PHONE_PRIVATE_MARKER".repeat(4)).unwrap();
        host.enable_diagnostics(p.join("journal")).unwrap();
        let supervisor=TunnelDeployment::load(&p).unwrap().unwrap().supervise(&host).unwrap();
        let mut status=supervisor.subscribe();
        tokio::time::timeout(std::time::Duration::from_secs(3),async {
            loop {
                if matches!(*status.borrow_and_update(),super::super::TunnelStatus::Backoff {failure:super::super::TunnelFailure::Start,..}) {break;}
                status.changed().await.unwrap();
            }
        }).await.unwrap();
        host.request_shutdown();
        tokio::time::timeout(std::time::Duration::from_secs(3),async {
            loop {
                if *status.borrow_and_update()==super::super::TunnelStatus::Stopped {break;}
                status.changed().await.unwrap();
            }
        }).await.unwrap();
        supervisor.shutdown().await.unwrap();
        let journal=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
        for forbidden in ["PRIVATE_MARKER","together2.example.com","tunnel_binary_mismatch",p.to_str().unwrap()] {
            assert!(!journal.contains(forbidden));
        }
        assert!(!p.join("runtime").exists());drop(host);
        for _ in 0..100 {
            if std::fs::remove_dir_all(&p).is_ok() {break;}
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert!(!p.exists());
    }
    #[cfg(windows)]
    #[test] fn junction_deployment_and_runtime_are_rejected_without_touching_target() {
        use std::os::windows::process::CommandExt;
        let p=directory();let target=p.join("target");std::fs::create_dir(&target).unwrap();
        let sentinel=target.join("sentinel.txt");std::fs::write(&sentinel,b"preserve target").unwrap();
        fn junction(link:&Path,target:&Path) {
            // cmd builtin only creates a new test junction. Never delete/move
            // through cmd. Reject metacharacters before passing test paths.
            for path in [link,target] {assert!(!path.to_str().unwrap().chars().any(|c|"&|<>^%!\"\r\n".contains(c)));}
            let status=std::process::Command::new("cmd.exe").args(["/D","/C","mklink","/J"])
                .arg(link).arg(target).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null()).creation_flags(0x08000000).status().unwrap();
            assert!(status.success(),"temporary junction creation failed");
        }
        let link=p.join("redirect");junction(&link,&target);
        assert!(matches!(TunnelDeployment::load(&link),Err("tunnel_deployment_redirected")));
        // remove_dir removes the junction itself, never recurse through it.
        std::fs::remove_dir(&link).unwrap();
        let bytes=b"synthetic image, never launched";
        std::fs::write(p.join("cloudflared.exe"),bytes).unwrap();
        std::fs::write(p.join("credentials.json"),b"synthetic placeholder").unwrap();
        write(&p,&manifest(format!("{:x}",Sha256::digest(bytes))));
        let deployment=TunnelDeployment::load(&p).unwrap().unwrap();
        let runtime=p.join("runtime");junction(&runtime,&target);
        assert!(matches!(deployment.start(),Err("tunnel_deployment_redirected")));
        assert_eq!(std::fs::read_dir(&target).unwrap().count(),1);
        assert_eq!(std::fs::read(&sentinel).unwrap(),b"preserve target");
        std::fs::remove_dir(&runtime).unwrap();drop(deployment);std::fs::remove_dir_all(p).unwrap();
    }
    #[cfg(windows)]
    #[test] fn verified_real_image_launches_and_owned_config_is_cleaned_after_exit() {
        let p=directory();let executable=p.join("cloudflared.exe");
        // This is the Rust test harness, NOT cloudflared. Its fixed tunnel CLI
        // arguments are rejected, exercising successful native image creation
        // and a real nonzero exit without opening a tunnel or network connection.
        std::fs::copy(std::env::current_exe().unwrap(),&executable).unwrap();
        let digest=format!("{:x}",Sha256::digest(std::fs::read(&executable).unwrap()));
        write(&p,&manifest(digest));
        let credentials=p.join("credentials.json");let fake=b"synthetic untouched credential placeholder";
        std::fs::write(&credentials,fake).unwrap();
        let deployment=TunnelDeployment::load(&p).unwrap().unwrap();
        let mut child=deployment.start().unwrap();
        let configs:Vec<_>=std::fs::read_dir(p.join("runtime")).unwrap().map(|e|e.unwrap().path()).collect();
        assert_eq!(configs.len(),1);
        let yaml=std::fs::read_to_string(&configs[0]).unwrap();
        assert!(yaml.contains("path: ^/(session|pair|health|media|updates/android|diagnostics/phone)$"));
        assert!(yaml.contains("service: http://127.0.0.1:47820"));
        let until=std::time::Instant::now()+std::time::Duration::from_secs(10);
        loop {
            match child.state().unwrap() {
                super::super::ProcessState::Exited {success}=>{assert!(!success);break;},
                super::super::ProcessState::Running=>{},
                other=>panic!("Unexpected fixture process state: {other:?}"),
            }
            assert!(std::time::Instant::now()<until,"copied harness did not exit");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        child.stop().unwrap();child.stop().unwrap();drop(child);
        assert_eq!(std::fs::read_dir(p.join("runtime")).unwrap().count(),0);
        assert_eq!(std::fs::read(&credentials).unwrap(),fake);
        drop(deployment);std::fs::remove_dir_all(p).unwrap();
    }
    #[test] fn binary_hash_is_checked_again_before_launch_and_no_config_written_on_failure() {
        let p=directory();let bytes=b"synthetic binary, never executed";
        write(&p,&manifest(format!("{:x}",Sha256::digest(bytes))));
        let deployment=TunnelDeployment::load(&p).unwrap().unwrap();
        assert!(matches!(deployment.verified_executable(),Err("tunnel_binary_open_failed")));
        std::fs::write(p.join("cloudflared.exe"),bytes).unwrap();let verified=deployment.verified_executable().unwrap();
        #[cfg(windows)] {
            assert!(std::fs::OpenOptions::new().write(true).open(p.join("cloudflared.exe")).is_err());
            assert!(std::fs::remove_file(p.join("cloudflared.exe")).is_err());
        }
        drop(verified);
        assert!(matches!(deployment.start(),Err("tunnel_credentials_missing")));
        std::fs::write(p.join("cloudflared.exe"),b"modified image").unwrap();
        assert!(matches!(deployment.start(),Err("tunnel_binary_mismatch")));
        assert!(!p.join("runtime").exists());
        std::fs::remove_file(p.join("cloudflared.exe")).unwrap();std::fs::remove_file(p.join("deployment.json")).unwrap();std::fs::remove_dir(p).unwrap();
    }
}

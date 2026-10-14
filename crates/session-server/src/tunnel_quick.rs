//! Private Alpha temporary HTTPS control address. No account/domain or media proxy.
use std::{fs::File,io::Read,path::{Path,PathBuf},process::Command};
use sha2::{Digest,Sha256};
use super::native_child::NativeChild;
const PIN:&str="f096265ec2fcbe9bb6e2d64268db167ced3fcbb83d894bdb9e2fcdb26f2ea7e2";
/// Exactly one child/job and new capture file. Raw connector text never enters diagnostics.
pub struct QuickTunnel {child:NativeChild,log:PathBuf,origin:Option<String>}
impl QuickTunnel {
    pub fn start(executable:&Path,runtime:&Path)->Result<Self,&'static str> {
        if !executable.is_absolute() || !runtime.is_absolute() {return Err("temporary_tunnel_path_invalid");}
        super::deployment::reject_redirected(executable)?;
        super::deployment::reject_redirected(runtime)?;
        let mut options=std::fs::OpenOptions::new();options.read(true);
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(1);
        let mut image=options.open(executable).map_err(|_|"temporary_tunnel_binary_missing")?;
        if image.metadata().map_err(|_|"temporary_tunnel_binary_invalid")?.len()>128*1024*1024 {return Err("temporary_tunnel_binary_invalid");}
        let mut hash=Sha256::new();let mut buffer=[0u8;65536];
        loop {let count=image.read(&mut buffer).map_err(|_|"temporary_tunnel_binary_invalid")?;if count==0 {break}hash.update(&buffer[..count]);}
        let digest:String=hash.finalize().iter().map(|byte|format!("{byte:02x}")).collect();
        if digest!=PIN {return Err("temporary_tunnel_binary_mismatch");}
        std::fs::create_dir_all(runtime).map_err(|_|"temporary_tunnel_runtime_failed")?;
        let log=runtime.join(format!("temporary-control-{}.txt",uuid::Uuid::new_v4()));
        let output=std::fs::OpenOptions::new().write(true).create_new(true).open(&log).map_err(|_|"temporary_tunnel_capture_failed")?;
        let mut command=Command::new(executable);command.env_clear().current_dir(runtime);
        for key in ["SystemRoot","WINDIR","TEMP","TMP"] {if let Some(value)=std::env::var_os(key) {command.env(key,value);}}
        command.args(["tunnel","--no-autoupdate","--protocol","http2","--url","http://127.0.0.1:47820"]);
        match NativeChild::spawn_with_stderr(&command,&output) {
            Ok(child)=>Ok(Self {child,log,origin:None}),
            Err(code)=>{drop(output);let _=std::fs::remove_file(log);Err(code)},
        }
    }
    /// A discovered URL is distinct from a verified end-to-end WAN connection.
    pub fn origin(&mut self)->Result<Option<String>,&'static str> {
        if self.child.try_wait().map_err(|_|"temporary_tunnel_status_failed")?.is_some() {self.origin=None;return Err("temporary_tunnel_exited");}
        let file=File::open(&self.log).map_err(|_|"temporary_tunnel_capture_failed")?;
        if file.metadata().map_err(|_|"temporary_tunnel_capture_failed")?.len()>262144 {
            self.origin=None;let _=self.child.kill();return Err("temporary_tunnel_capture_limit");
        }
        if self.origin.is_none() {
            let mut bytes=Vec::new();file.take(32768).read_to_end(&mut bytes).map_err(|_|"temporary_tunnel_capture_failed")?;
            self.origin=extract_origin(&String::from_utf8_lossy(&bytes));
        }
        Ok(self.origin.clone())
    }
    pub fn stop(&mut self)->Result<(),&'static str> {
        self.child.kill().map_err(|_|"temporary_tunnel_stop_failed")?;
        self.child.wait().map_err(|_|"temporary_tunnel_stop_failed")?;
        self.origin=None;std::fs::remove_file(&self.log).or_else(|e|if e.kind()==std::io::ErrorKind::NotFound {Ok(())}else{Err(e)})
            .map_err(|_|"temporary_tunnel_capture_cleanup_failed")
    }
}
impl Drop for QuickTunnel {fn drop(&mut self){let _=self.stop();}}
fn extract_origin(text:&str)->Option<String> {
    for tail in text.split("https://").skip(1) {
        let host=tail.split(|c:char|!(c.is_ascii_alphanumeric() || c=='-' || c=='.')).next()?;
        if let Some(label)=host.strip_suffix(".trycloudflare.com") {
            if (1..=63).contains(&label.len()) && !label.starts_with('-') && !label.ends_with('-') && label.bytes().all(|b|b.is_ascii_lowercase() || b.is_ascii_digit() || b==b'-') {
                return Some(format!("https://{host}"));
            }
        }
    }
    None
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn only_complete_single_label_temporary_https_origins_are_exposed() {
        assert_eq!(extract_origin("| https://safe-room.trycloudflare.com |"),Some("https://safe-room.trycloudflare.com".into()));
        for invalid in ["http://safe.trycloudflare.com","https://safe.trycloudflare.com.evil","https://-bad.trycloudflare.com","https://a.b.trycloudflare.com","https://PRIVATE.trycloudflare.com","https://evil.example/private"] {assert!(extract_origin(invalid).is_none());}
    }
}

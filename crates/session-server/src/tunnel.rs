//! Control-only named-tunnel configuration and explicitly owned process lifecycle.
use std::path::PathBuf;
#[path = "tunnel_process.rs"]
mod process;
pub use process::{OwnedTunnel,ProcessState};
#[path = "tunnel_supervisor.rs"]
mod supervisor;
#[path = "tunnel_deployment.rs"]
mod deployment;
pub use deployment::TunnelDeployment;
pub use supervisor::{TunnelSupervisor,TunnelStatus,TunnelFailure};

#[cfg(windows)]
#[path = "tunnel_job.rs"]
mod windows_job;
#[cfg(windows)]
pub use windows_job::WindowsJob;

#[cfg(windows)]
#[path = "tunnel_native_child.rs"]
mod native_child;

pub struct ControlConfig {hostname:String,tunnel_id:uuid::Uuid,credentials_file:PathBuf}
impl ControlConfig {
    /// Public address for the local pairing UI; contains no credentials.
    pub fn public_origin(&self)->String {format!("https://{}",self.hostname)}
    pub fn new(hostname:&str,tunnel_id:&str,credentials_file:PathBuf)->Result<Self,&'static str> {
        let hostname=hostname.to_ascii_lowercase();
        if hostname.len()>253 || !hostname.contains('.') || hostname.parse::<std::net::IpAddr>().is_ok()
            || !hostname.split('.').all(|label| !label.is_empty() && label.len()<=63
                && !label.starts_with('-') && !label.ends_with('-')
                && label.bytes().all(|c|c.is_ascii_lowercase() || c.is_ascii_digit() || c==b'-')) {
            return Err("invalid_tunnel_hostname");
        }
        let tunnel_id=uuid::Uuid::parse_str(tunnel_id).map_err(|_|"invalid_tunnel_id")?;
        if tunnel_id.is_nil() {return Err("invalid_tunnel_id");}
        let path=credentials_file.to_str().ok_or("invalid_tunnel_credentials_path")?;
        if !credentials_file.is_absolute() || credentials_file.file_name().is_none()
            || path.chars().any(|c|c.is_control() || c=='\u{2028}' || c=='\u{2029}') {
            return Err("invalid_tunnel_credentials_path");
        }
        Ok(Self {hostname,tunnel_id,credentials_file})
    }
    /// Local configuration contents, never a diagnostic payload. JSON string
    /// quoting is also valid YAML double-quoted scalar syntax and protects
    /// Windows backslashes, spaces, quotes and '#' in the credential file path.
    pub fn yaml(&self)->Result<String,&'static str> {
        let path=serde_json::to_string(self.credentials_file.to_str().ok_or("invalid_tunnel_credentials_path")?)
            .map_err(|_|"invalid_tunnel_credentials_path")?;
        Ok(format!("tunnel: {}\ncredentials-file: {}\n\ningress:\n  - hostname: {}\n    path: ^/(session|pair|health|media|updates/android|diagnostics/phone)$\n    service: http://127.0.0.1:47820\n  - service: http_status:404\n",
            self.tunnel_id,path,self.hostname))
    }
}

#[cfg(test)] mod tests {
    use super::*;
    const ID:&str="09fdab9b-262a-481f-b5f9-3d8bb705ce58";
    fn path()->PathBuf {std::env::temp_dir().join("Together 2 # tunnel").join("credentials.json")}
    #[test] fn control_origin_paths_and_default_deny_are_fixed() {
        let yaml=ControlConfig::new("Together2.Example.com",ID,path()).unwrap().yaml().unwrap();
        assert!(yaml.contains("hostname: together2.example.com\n"));
        assert!(yaml.contains("path: ^/(session|pair|health|media|updates/android|diagnostics/phone)$\n"));
        assert_eq!(yaml.matches("service:").count(),2);
        assert!(yaml.contains("service: http://127.0.0.1:47820\n"));
        assert!(yaml.ends_with("  - service: http_status:404\n"));
        assert!(!yaml.contains("warp-routing"));assert!(!yaml.contains("tcp://"));
    }
    #[test] fn wildcard_scheme_ip_and_yaml_hostname_injection_are_rejected() {
        for bad in ["*.example.com","https://example.com","example.com:443","127.0.0.1","localhost",
            "example.com\nservice: tcp://127.0.0.1:22","example.com/path","user@example.com",
            "-bad.example.com","bad-.example.com","a..example.com","example.com.","exämple.com"] {
            assert!(matches!(ControlConfig::new(bad,ID,path()),Err("invalid_tunnel_hostname")));
        }
    }
    #[test] fn ids_and_credential_paths_fail_without_echoing_input() {
        for bad in ["PRIVATE_TUNNEL_TOKEN","00000000-0000-0000-0000-000000000000"] {
            assert!(matches!(ControlConfig::new("together2.example.com",bad,path()),Err("invalid_tunnel_id")));
        }
        for bad in [PathBuf::from("credentials.json"),std::env::temp_dir().join("private\ncredentials.json")] {
            assert!(matches!(ControlConfig::new("together2.example.com",ID,bad),Err("invalid_tunnel_credentials_path")));
        }
    }
    #[test] fn credential_path_is_quoted_without_reading_or_modifying_it() {
        let path=std::env::temp_dir().join("Together # \"private\"").join("credentials.json");
        let yaml=ControlConfig::new("together2.example.com",ID,path.clone()).unwrap().yaml().unwrap();
        let quoted=yaml.lines().find_map(|line|line.strip_prefix("credentials-file: ")).unwrap();
        assert_eq!(serde_json::from_str::<String>(quoted).unwrap(),path.to_str().unwrap());
    }
}

#[cfg(windows)]
#[path="tunnel_quick.rs"]
mod quick;
#[cfg(windows)]
pub use quick::QuickTunnel;

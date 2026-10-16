use serde::{Deserialize, Serialize};
use std::{path::Path, io::Read};
/// Public release metadata only; never account credentials or media capabilities.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    pub protocol:u32, pub package:String, pub version_code:u64,
    pub version_name:String, pub apk_url:String, pub sha256:String, pub size_bytes:u64,
}
impl Release {
    pub fn validate(&self)->Result<(), &'static str> {
        let url=url::Url::parse(&self.apk_url).map_err(|_|"invalid release metadata")?;
        if self.protocol!=1 || self.package!="app.together.v2" || self.version_code==0 || self.version_code>i32::MAX as u64
            || self.version_name.is_empty() || self.version_name.len()>64 || self.version_name.chars().any(char::is_control)
            || !(1..=536_870_912).contains(&self.size_bytes) || self.sha256.len()!=64
            || !self.sha256.bytes().all(|c|c.is_ascii_hexdigit())
            || url.scheme()!="https" || url.port_or_known_default()!=Some(443) || url.host_str().is_none() || !url.username().is_empty()
            || url.password().is_some() || url.query().is_some() || url.fragment().is_some() || self.apk_url.len()>2048 {
            return Err("invalid release metadata");
        } Ok(())
    }
    pub fn read(path:&Path)->Result<Self, &'static str> {
        let file=std::fs::File::open(path).map_err(|_|"update unavailable")?;
        let mut bytes=Vec::new();file.take(8193).read_to_end(&mut bytes).map_err(|_|"update unavailable")?;
        if bytes.len()>8192 {return Err("invalid release metadata");}
        let release:Self=serde_json::from_slice(&bytes).map_err(|_|"invalid release metadata")?;
        release.validate()?;Ok(release)
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn unsupported_download_port_is_not_advertised() {
        let mut candidate=release();candidate.apk_url="https://example.invalid:444/update.apk".into();
        assert_eq!(candidate.validate(),Err("invalid release metadata"));
        candidate.apk_url="https://example.invalid:443/update.apk".into();assert!(candidate.validate().is_ok());
    }
    #[tokio::test] async fn private_release_endpoint_rejects_pc_and_distinguishes_missing_feed() {
        use axum::{extract::State,http::{HeaderMap,HeaderValue,StatusCode}};
        let host=crate::Host::new("pc-update-synthetic-000000000000000".into(),"phone-update-synthetic-00000000000".into()).unwrap();
        let directory=std::env::temp_dir().join(format!("update-feed-{}",uuid::Uuid::new_v4()));std::fs::create_dir(&directory).unwrap();
        let path=directory.join("release.json");host.configure_android_updates(path.clone()).unwrap();
        let headers=|token:&str| {let mut h=HeaderMap::new();h.insert("authorization",HeaderValue::from_str(&format!("Bearer {token}")).unwrap());h};
        assert_eq!(crate::phone_update(State(host.clone()),headers("pc-update-synthetic-000000000000000")).await.status(),StatusCode::UNAUTHORIZED);
        assert_eq!(crate::phone_update(State(host.clone()),headers("phone-update-synthetic-00000000000")).await.status(),StatusCode::SERVICE_UNAVAILABLE);
        std::fs::write(&path,serde_json::to_vec(&release()).unwrap()).unwrap();
        let response=crate::phone_update(State(host.clone()),headers("phone-update-synthetic-00000000000")).await;
        assert_eq!(response.status(),StatusCode::OK);assert_eq!(response.headers()["cache-control"],"no-store");
        std::fs::write(&path,vec![b'x';8193]).unwrap();
        assert_eq!(crate::phone_update(State(host.clone()),headers("phone-update-synthetic-00000000000")).await.status(),StatusCode::SERVICE_UNAVAILABLE);
        std::fs::remove_file(path).unwrap();std::fs::remove_dir(directory).unwrap();
    }
    fn release()->Release {Release {protocol:1,package:"app.together.v2".into(),version_code:3,version_name:"Alpha 1".into(),apk_url:"https://releases.example.org/2gether.apk".into(),sha256:"a".repeat(64),size_bytes:224_000_000}}
    #[test] fn rejects_credentials_and_unbounded_or_wrong_release() {
        for url in ["http://example.org/a.apk","https://user:secret@example.org/a.apk","https://example.org/a.apk?token=secret","https://example.org/a.apk#fragment"] {
            let mut r=release();r.apk_url=url.into();assert!(r.validate().is_err());
        }
        let mut r=release();r.package="legacy.app".into();assert!(r.validate().is_err());
        let mut r=release();r.size_bytes=536_870_913;assert!(r.validate().is_err());
        let mut r=release();r.sha256="z".repeat(64);assert!(r.validate().is_err());
        assert!(release().validate().is_ok());
    }
}

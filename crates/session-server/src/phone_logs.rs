use serde::{Deserialize,Serialize};
use sha2::{Digest,Sha256};
use std::collections::VecDeque;

// Remember only bounded numeric/boolean sample fingerprints, never credentials
// or raw request bodies. A retry receipt refers to the original completed write.
#[derive(Default)]
pub(crate) struct RetryCache {entries:VecDeque<(uuid::Uuid,[u8;32])>}
pub(crate) enum RetryCheck {New,Same,Conflict}
impl RetryCache {
    pub(crate) fn check(&self,id:uuid::Uuid,digest:[u8;32])->RetryCheck {
        match self.entries.iter().find(|(saved,_)|*saved==id) {
            None=>RetryCheck::New,
            Some((_,saved)) if *saved==digest=>RetryCheck::Same,
            Some(_)=>RetryCheck::Conflict,
        }
    }
    pub(crate) fn committed(&mut self,id:uuid::Uuid,digest:[u8;32]) {
        if self.entries.len()==64 {self.entries.pop_front();}
        self.entries.push_back((id,digest));
    }
}
#[derive(Deserialize,Serialize)]
#[serde(rename_all="snake_case")]
pub enum Playback {Starting,Loading,Playing,Paused,Ended,Failed}
#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub elapsed_ms:u64,pub utc_ms:u64,pub generation:u64,pub position_ms:u64,
    pub duration_ms:u64,pub connected:bool,pub buffering:bool,pub seeking:bool,
    pub preparing:bool,pub source_ready:bool,pub fault:bool,pub playback:Playback,
    pub drift_ms:i64,pub rtt_ms:Option<u64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Batch {pub protocol:u32,pub batch_id:uuid::Uuid,pub samples:Vec<Sample>}
impl Batch {
    pub(crate) fn fingerprint(&self)->Result<[u8;32],serde_json::Error> {
        Ok(Sha256::digest(serde_json::to_vec(&self.samples)?).into())
    }
    pub fn valid(&self)->bool {
        self.protocol==1 && (1..=20).contains(&self.samples.len()) && self.samples.iter().all(|s|
            s.elapsed_ms<=i64::MAX as u64 && s.utc_ms<=i64::MAX as u64 && s.generation<=i64::MAX as u64
            && s.position_ms<=604_800_000 && s.duration_ms<=604_800_000
            && s.drift_ms.abs_diff(0)<=604_800_000 && s.rtt_ms.is_none_or(|r|r<=60_000))
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn retry_window_is_bounded_without_forgetting_intervening_batches() {
        let mut cache=RetryCache::default();let first=uuid::Uuid::new_v4();
        cache.committed(first,[1;32]);
        for n in 0..63 {cache.committed(uuid::Uuid::new_v4(),[n;32]);}
        assert!(matches!(cache.check(first,[1;32]),RetryCheck::Same));
        assert!(matches!(cache.check(first,[2;32]),RetryCheck::Conflict));
        cache.committed(uuid::Uuid::new_v4(),[3;32]);
        assert!(matches!(cache.check(first,[1;32]),RetryCheck::New));
        assert_eq!(cache.entries.len(),64);
    }
    #[tokio::test] async fn interleaved_retry_is_deduplicated_and_changed_payload_is_not_acknowledged() {
        use axum::{extract::State,http::{HeaderMap,HeaderValue,StatusCode},Json};
        let host=crate::Host::new("pc-diagnostic-retry-test-00000000000".into(),"phone-diagnostic-retry-test-0000000".into()).unwrap();
        let directory=std::env::temp_dir().join(format!("phone-log-retry-{}",uuid::Uuid::new_v4()));
        host.enable_diagnostics(directory.clone()).unwrap();
        let first=serde_json::json!({"protocol":1,"batch_id":uuid::Uuid::new_v4().to_string(),"samples":[{
            "elapsed_ms":10,"utc_ms":100,"generation":2,"position_ms":30,"duration_ms":500,
            "connected":true,"buffering":false,"seeking":false,"preparing":false,"source_ready":true,
            "fault":false,"playback":"paused","drift_ms":0,"rtt_ms":null}]});
        let mut second=first.clone();second["batch_id"]=serde_json::json!(uuid::Uuid::new_v4());
        let mut changed=first.clone();changed["samples"][0]["position_ms"]=serde_json::json!(31);
        for (body,expected) in [(&first,StatusCode::NO_CONTENT),(&second,StatusCode::NO_CONTENT),
            (&first,StatusCode::NO_CONTENT),(&changed,StatusCode::CONFLICT)] {
            let mut headers=HeaderMap::new();headers.insert("authorization",HeaderValue::from_static("Bearer phone-diagnostic-retry-test-0000000"));
            let response=crate::phone_logs(State(host.clone()),headers,Json(serde_json::from_value(body.clone()).unwrap())).await;
            assert_eq!(response.status(),expected);
        }
        let text=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
        assert_eq!(text.matches("\"event\":\"phone\"").count(),2);
        assert!(!text.contains("diagnostic-retry-test"));
        drop(host);
        for _ in 0..100 {if std::fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
        assert!(!directory.exists());
    }
    #[tokio::test] async fn phone_only_upload_is_flushed_and_retry_is_deduplicated() {
        use axum::{extract::State,http::{HeaderMap,HeaderValue,StatusCode},Json};
        let host=crate::Host::new("pc-diagnostics-synthetic-0000000000".into(),"phone-diagnostics-synthetic-000000".into()).unwrap();
        let directory=std::env::temp_dir().join(format!("phone-log-test-{}",uuid::Uuid::new_v4()));
        host.enable_diagnostics(directory.clone()).unwrap();
        let value=serde_json::json!({"protocol":1,"batch_id":uuid::Uuid::new_v4().to_string(),"samples":[{
            "elapsed_ms":10,"utc_ms":100,"generation":2,"position_ms":30,"duration_ms":500,"connected":true,
            "buffering":false,"seeking":false,"preparing":false,"source_ready":true,"fault":false,
            "playback":"playing","drift_ms":-15,"rtt_ms":100}]});
        for token in ["wrong","pc-diagnostics-synthetic-0000000000","phone-diagnostics-synthetic-000000","phone-diagnostics-synthetic-000000"] {
            let mut headers=HeaderMap::new();headers.insert("authorization",HeaderValue::from_str(&format!("Bearer {token}")).unwrap());
            let response=crate::phone_logs(State(host.clone()),headers,Json(serde_json::from_value(value.clone()).unwrap())).await;
            assert_eq!(response.status(),if token.starts_with("phone") {StatusCode::NO_CONTENT} else {StatusCode::UNAUTHORIZED});
        }
        let logs=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
        assert_eq!(logs.matches("\"event\":\"phone\"").count(),1);assert!(!logs.contains("synthetic"));
        host.request_shutdown();drop(host);
        for _ in 0..100 {if std::fs::remove_dir_all(&directory).is_ok() {break;}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
        assert!(!directory.exists());
    }
    #[test] fn arbitrary_error_url_and_unknown_fields_cannot_enter_phone_log() {
        for input in [r#"{"protocol":1,"batch_id":"00000000-0000-0000-0000-000000000001","samples":[],"url":"secret"}"#,
            r#"{"elapsed_ms":1,"utc_ms":1,"generation":1,"position_ms":0,"duration_ms":0,"connected":true,"buffering":false,"seeking":false,"preparing":false,"source_ready":false,"fault":false,"playback":"https://secret","drift_ms":0,"rtt_ms":null}"#] {
            assert!(serde_json::from_str::<Batch>(input).is_err());
        }
        let batch=Batch {protocol:1,batch_id:uuid::Uuid::nil(),samples:vec![]};assert!(!batch.valid());
    }
}

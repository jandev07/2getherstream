//! PC-only provider contract investigation. No returned URL is followed or delivered.
use super::*;
use zeroize::Zeroize;

#[derive(Debug,Serialize)]
pub struct Report {
    pub http_status:u16,pub success:bool,pub file_token_present:bool,pub presigned_token_present:bool,
    pub credential_free_https_urls:u32,pub account_bearing_urls:u32,pub rejected_urls:u32,
}
use serde::Serialize;
impl TorBox {
    pub async fn inspect_stream_contract(&self,media:&MediaIdentity)->Result<Report> {
        media.validate().map_err(anyhow::Error::msg)?;
        ensure!(media.provider==MediaProvider::Torbox,"unsupported media provider");
        let mut response=self.client.get("https://api.torbox.app/v1/api/stream/createstream")
            .bearer_auth(self.key.as_str()).query(&[("id",media.torrent_id.to_string()),("file_id",media.file_id.to_string()),
                ("type","torrent".into()),("scrobbling_enabled","false".into())]).send().await
            .map_err(|_|anyhow::anyhow!("TorBox stream inspection failed"))?;
        let status=response.status().as_u16();
        let mut report=Report {http_status:status,success:false,file_token_present:false,presigned_token_present:false,
            credential_free_https_urls:0,account_bearing_urls:0,rejected_urls:0};
        if !response.status().is_success() {return Ok(report);}
        let mut body=Zeroizing::new(Vec::new());
        while let Some(chunk)=response.chunk().await.map_err(|_|anyhow::anyhow!("TorBox stream response interrupted"))? {
            ensure!(body.len()+chunk.len()<=1024*1024,"TorBox stream response too large");body.extend_from_slice(&chunk);
        }
        let mut value:serde_json::Value=serde_json::from_slice(&body).map_err(|_|anyhow::anyhow!("TorBox stream response invalid"))?;
        report.success=value["success"]==true;
        inspect(&value, &self.key, &mut report,0);
        clear(&mut value);Ok(report)
    }
}
fn inspect(value:&serde_json::Value,key:&str,report:&mut Report,depth:u8) {
    if depth>12 {return;}
    match value {
        serde_json::Value::Object(fields)=>for (name,value) in fields {
            if name=="file_token" || name=="token" {report.file_token_present|=value.as_str().is_some_and(|s|!s.is_empty());}
            if name=="presigned_token" {report.presigned_token_present|=value.as_str().is_some_and(|s|!s.is_empty());}
            inspect(value,key,report,depth+1);
        },
        serde_json::Value::Array(values)=>for value in values.iter().take(128) {inspect(value,key,report,depth+1);},
        serde_json::Value::String(raw) if raw.starts_with("https://")=>match validate_stream_url(raw,key) {
            Ok(_)=>report.credential_free_https_urls+=1,
            Err(error) if error.to_string()=="TorBox API credential leaked into media URL"=>report.account_bearing_urls+=1,
            Err(_)=>report.rejected_urls+=1,
        },_=>{},
    }
}
fn clear(value:&mut serde_json::Value) {
    match value {
        serde_json::Value::String(s)=>s.zeroize(),
        serde_json::Value::Array(values)=>for value in values {clear(value);},
        serde_json::Value::Object(values)=>for (_,value) in values {clear(value);},_=>{},
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn reports_only_fixed_counts_never_provider_text_urls_or_tokens() {
        let mut value=serde_json::json!({"data":{"file_token":"PRIVATE_FILE","presigned_token":"PRIVATE_SIGNED",
            "url":"https://cdn.invalid/file?token=PRIVATE_ACCOUNT","other":"https://cdn.invalid/file?cap=PRIVATE_FILE","detail":"private error"}});
        let mut report=Report {http_status:200,success:true,file_token_present:false,presigned_token_present:false,credential_free_https_urls:0,account_bearing_urls:0,rejected_urls:0};
        inspect(&value,"PRIVATE_ACCOUNT",&mut report,0);
        assert_eq!(report.account_bearing_urls,1);assert_eq!(report.credential_free_https_urls,1);
        assert!(report.file_token_present && report.presigned_token_present);
        let text=serde_json::to_string(&report).unwrap();assert!(!text.contains("PRIVATE") && !text.contains("https://") && !text.contains("private error"));
        clear(&mut value);assert_eq!(value["data"]["file_token"],"");
    }
}

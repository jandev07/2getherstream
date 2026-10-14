//! PC-only TorBox resolver. Never expose API permalinks or raw HTTP errors.
pub mod secret;
pub mod selection;
pub mod stremio;
pub mod preparation;
#[cfg(feature = "experimental-stream-audit")]
pub mod stream_audit;
#[cfg(feature = "experimental-t3")]
pub mod t3;
#[cfg(feature = "experimental-download-redirect")]
pub mod download_redirect;
use anyhow::{ensure, Result};
use reqwest::{Client, Url};
use serde::Deserialize;
pub use together_sync_core::{MediaIdentity,MediaProvider};
use std::{net::IpAddr, time::Duration};
use zeroize::Zeroizing;

#[derive(Deserialize)]
struct Envelope<T> { success: bool, data: Option<T> }
#[derive(Deserialize)]
struct Torrent { id: u64, hash: String, download_present: bool, files: Option<Vec<File>> }
#[derive(Deserialize)]
struct File { id: u64, name: String, size: u64, #[serde(default)] infected: Option<bool> }

/// Deliberately has no Debug/Serialize implementation: URL bearer credentials are sensitive.
pub struct StreamLease { identity: MediaIdentity, url: Url }
impl StreamLease {
    pub fn identity(&self) -> &MediaIdentity { &self.identity }
    pub fn player_url(&self) -> &str { self.url.as_str() }
}
pub struct DeviceStreams { pub pc: StreamLease, pub phone: StreamLease }
// Resolve the two independent capabilities together. A failed role cancels the
// other request; callers never receive half a pair or wait two timeout periods.
async fn collect_device_streams(
    pc: impl std::future::Future<Output=Result<StreamLease>>,
    phone: impl std::future::Future<Output=Result<StreamLease>>,
) -> Result<DeviceStreams> {
    let (pc, phone) = tokio::try_join!(pc, phone)?;
    Ok(DeviceStreams { pc, phone })
}
enum DeliveryPolicy {FileScopedOnly,#[cfg(feature="private-alpha-account-links")] PrivateAlphaAccountLinks}
pub struct TorBox { client: Client, key: Zeroizing<String>,delivery:DeliveryPolicy,
    preparation_attempts:tokio::sync::Mutex<std::collections::HashSet<String>> }
impl TorBox {
    /// Explicit private Alpha exception authorized after secure paths failed.
    /// Account-bearing CDN links expose account access to the recipient device.
    /// Never use this constructor for a general release or an unpaired device.
    #[cfg(feature="private-alpha-account-links")]
    pub fn new_private_alpha(key:Zeroizing<String>)->Result<Self> {
        let mut client=Self::new(key)?;client.delivery=DeliveryPolicy::PrivateAlphaAccountLinks;Ok(client)
    }
    /// Hostname only for provider-origin verification; never returns a path,
    /// query or account credential. Does not follow the supplied media link.
    #[cfg(feature="private-alpha-account-links")]
    pub async fn inspect_download_host(&self,media:&MediaIdentity)->Result<String> {
        media.validate().map_err(anyhow::Error::msg)?;
        let raw=Zeroizing::new(self.response::<String>(self.client.get("https://api.torbox.app/v1/api/torrents/requestdl")
            .query(&[("token",self.key.to_string()),("torrent_id",media.torrent_id.to_string()),("file_id",media.file_id.to_string()),
                ("zip_link","false".into()),("redirect","false".into())])).await?);
        let url=validate_stream_syntax(&raw)?;
        let host=url.host_str().ok_or_else(||anyhow::anyhow!("Media CDN host missing"))?;
        validate_account_key_absent(host,&self.key)?;Ok(host.to_owned())
    }
    pub fn new(key: Zeroizing<String>) -> Result<Self> {
        ensure!(!key.is_empty(), "TorBox credential is empty");
        let client = Client::builder().timeout(Duration::from_secs(25)).connect_timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none()).user_agent("Together2/0.1")
            .build().map_err(|_| anyhow::anyhow!("TorBox HTTP initialization failed"))?;
        Ok(Self { client, key,delivery:DeliveryPolicy::FileScopedOnly,preparation_attempts:tokio::sync::Mutex::new(std::collections::HashSet::new()) })
    }
    async fn response<T: serde::de::DeserializeOwned>(&self, request: reqwest::RequestBuilder) -> Result<T> {
        let mut response = request.send().await.map_err(|_| anyhow::anyhow!("TorBox network request failed"))?;
        ensure!(response.status().is_success(), "TorBox HTTP request rejected ({})", response.status().as_u16());
        // Bound untrusted metadata; don't collect an arbitrarily large response.
        let mut body = Zeroizing::new(Vec::new());
        while let Some(chunk) = response.chunk().await.map_err(|_| anyhow::anyhow!("TorBox response interrupted"))? {
            ensure!(body.len() + chunk.len() <= 8 * 1024 * 1024, "TorBox response too large"); body.extend_from_slice(&chunk);
        }
        let envelope: Envelope<T> = serde_json::from_slice(&body)
            .map_err(|_| anyhow::anyhow!("TorBox response format invalid"))?;
        ensure!(envelope.success, "TorBox API request rejected");
        envelope.data.ok_or_else(|| anyhow::anyhow!("TorBox response contained no data"))
    }
    /// Only existing, available video files; never creates/deletes torrents.
    pub async fn available_videos(&self) -> Result<Vec<MediaIdentity>> {
        self.video_inventory(false).await
    }
    /// Explicit user source import can follow Stremio's just-created account item.
    /// One fresh inventory read avoids treating the provider's cached list as current.
    pub async fn available_videos_fresh(&self)->Result<Vec<MediaIdentity>> {self.video_inventory(true).await}
    async fn video_inventory(&self,fresh:bool)->Result<Vec<MediaIdentity>> {
        let mut result = Vec::new();
        for page in 0..100u64 {
            let torrents: Vec<Torrent> = self.response(self.client.get("https://api.torbox.app/v1/api/torrents/mylist")
                .bearer_auth(self.key.as_str()).query(&[("limit",100.to_string()),("offset",(page*100).to_string()),("bypass_cache",fresh.to_string())])).await?;
            let end = torrents.len() < 100;
            for torrent in torrents.into_iter().filter(|t| t.download_present) {
                for file in torrent.files.unwrap_or_default().into_iter().filter(|f| f.infected == Some(false) && video_filename(&f.name)) {
                    if file.size==0 {continue;}
                    let media=public_file_identity(torrent.id,&torrent.hash,&file,&self.key)?;
                    result.push(media);
                }
            }
            if end { return Ok(result); }
        }
        anyhow::bail!("TorBox library exceeded pagination limit")
    }
    async fn lease(&self, media: &MediaIdentity, ip: Option<IpAddr>) -> Result<StreamLease> {
        ensure!(media.provider == MediaProvider::Torbox, "unsupported media provider");
        media.validate().map_err(anyhow::Error::msg)?;
        let mut query = vec![("token", self.key.to_string()), ("torrent_id", media.torrent_id.to_string()),
            ("file_id", media.file_id.to_string()), ("zip_link", "false".into()), ("redirect", "false".into())];
        if let Some(ip) = ip { query.push(("user_ip", ip.to_string())); }
        // TorBox's documented requestdl requires token in the query. Raw request/error URLs never escape.
        let request = self.client.get("https://api.torbox.app/v1/api/torrents/requestdl").query(&query);
        for (_, value) in &mut query { use zeroize::Zeroize; value.zeroize(); }
        let raw = Zeroizing::new(self.response::<String>(request).await?);
        let url = match self.delivery {
            DeliveryPolicy::FileScopedOnly=>validate_stream_url(&raw,self.key.as_str())?,
            #[cfg(feature="private-alpha-account-links")]
            DeliveryPolicy::PrivateAlphaAccountLinks=>validate_private_alpha_url(&raw,self.key.as_str())?,
        };
        Ok(StreamLease { identity: media.clone(), url })
    }
    pub async fn device_streams(&self, media: &MediaIdentity, pc_ip: Option<IpAddr>, phone_ip: Option<IpAddr>) -> Result<DeviceStreams> {
        self.verify_media_current(media).await?;
        // Separate requests for the same canonical file. CDN may legitimately return equal URL strings.
        collect_device_streams(self.lease(media, pc_ip), self.lease(media, phone_ip)).await
    }
    /// A browser row can outlive a provider account change. Re-read the selected
    /// torrent before issuing either role's link; IDs and a range size alone do
    /// not establish that the selected hash/filename still refer to that file.
    pub async fn verify_media_current(&self,media:&MediaIdentity)->Result<()> {
        media.validate().map_err(anyhow::Error::msg)?;
        ensure!(media.provider==MediaProvider::Torbox,"Unsupported media provider");
        let current:Torrent=self.response(self.client.get("https://api.torbox.app/v1/api/torrents/mylist")
            .bearer_auth(self.key.as_str()).query(&[("id",media.torrent_id.to_string()),("bypass_cache","true".into())])).await?;
        verify_current_file(&current,media,&self.key)
    }
    /// Small direct range read validates the CDN, without downloading a movie or proxying it.
    pub async fn probe(&self, lease: &StreamLease) -> Result<usize> {
        let mut response = self.client.get(lease.url.clone()).header("Range", "bytes=0-1023").header("Accept-Encoding","identity").send().await
            .map_err(|_| anyhow::anyhow!("Media CDN request failed"))?;
        ensure!(response.status() == reqwest::StatusCode::PARTIAL_CONTENT, "Media CDN did not honor range request");
        validate_probe_headers(response.headers(),lease.identity.filesize)?;
        let mut length = 0;
        while let Some(chunk) = response.chunk().await.map_err(|_| anyhow::anyhow!("Media CDN response interrupted"))? {
            length += chunk.len(); ensure!(length <= 1024, "Media CDN range exceeded requested size");
        }
        ensure!(length == 1024, "Media CDN returned an incomplete range"); Ok(length)
    }
}
/// A bounded capability check, not a content hash or proof of media identity.
fn validate_probe_headers(headers:&reqwest::header::HeaderMap,expected_size:Option<u64>)->Result<()> {
    use reqwest::header::{CONTENT_RANGE,CONTENT_ENCODING,CONTENT_LENGTH};
    ensure!(headers.get_all(CONTENT_RANGE).iter().count()==1,"Media CDN range metadata invalid");
    let range=headers.get(CONTENT_RANGE).and_then(|h|h.to_str().ok())
        .ok_or_else(||anyhow::anyhow!("Media CDN range metadata invalid"))?;
    let total=range.split_once(' ').filter(|(unit,_)|unit.eq_ignore_ascii_case("bytes"))
        .and_then(|(_,spec)|spec.strip_prefix("0-1023/"))
        .filter(|s|!s.is_empty() && s.len()<=20 && s.bytes().all(|b|b.is_ascii_digit()))
        .and_then(|s|s.parse::<u64>().ok()).ok_or_else(||anyhow::anyhow!("Media CDN range metadata invalid"))?;
    ensure!(total>=1024 && expected_size.map_or(true,|size|size==total),"Media CDN file size mismatch");
    ensure!(headers.get_all(CONTENT_ENCODING).iter().count()<=1 && headers.get(CONTENT_ENCODING)
        .map_or(true,|h|h.to_str().is_ok_and(|s|s.eq_ignore_ascii_case("identity"))),"Media CDN content encoding invalid");
    ensure!(headers.get_all(CONTENT_LENGTH).iter().count()<=1 && headers.get(CONTENT_LENGTH)
        .map_or(true,|h|h.to_str().ok().filter(|s|!s.is_empty() && s.bytes().all(|b|b.is_ascii_digit()))
            .and_then(|s|s.parse::<u64>().ok())==Some(1024)),"Media CDN range length invalid");
    Ok(())
}
/// Provider filenames enter IPC, snapshots and diagnostics. Validate them before
/// they can cross that public boundary, even when a provider says a file is ready.
fn public_file_identity(torrent_id:u64,hash:&str,file:&File,key:&str)->Result<MediaIdentity> {
    validate_account_key_absent(&file.name,key)?;
    validate_account_key_absent(hash,key)?;
    ensure!(!file.name.is_empty() && file.name.len()<=512 && !file.name.chars().any(char::is_control) && !file.name.contains("://"),"TorBox public filename invalid");
    let media=MediaIdentity {torrent_hash:Some(hash.into()),filename:Some(file.name.clone()),filesize:Some(file.size),
        ..MediaIdentity::torbox(torrent_id,file.id)};
    media.validate().map_err(anyhow::Error::msg)?;Ok(media)
}
fn verify_current_file(torrent:&Torrent,expected:&MediaIdentity,key:&str)->Result<()> {
    ensure!(torrent.id==expected.torrent_id && torrent.download_present,"Selected TorBox item is no longer ready");
    let files:Vec<_>=torrent.files.as_deref().unwrap_or_default().iter().filter(|file|file.id==expected.file_id).collect();
    ensure!(files.len()==1 && files[0].infected==Some(false) && video_filename(&files[0].name),"Selected TorBox file is no longer one clean video");
    let current=public_file_identity(torrent.id,&torrent.hash,files[0],key)?;
    ensure!(expected.torrent_hash.as_ref().is_some_and(|hash|hash.eq_ignore_ascii_case(&torrent.hash))
        && expected.filename==current.filename && expected.filesize==current.filesize,"Selected TorBox file identity changed");
    Ok(())
}
fn video_filename(name: &str) -> bool {
    name.rsplit('.').next().is_some_and(|ext| ["mkv", "mp4", "webm", "avi", "mov", "m4v", "ts"].contains(&ext.to_ascii_lowercase().as_str()))
}
fn validate_stream_url(raw: &str, key: &str) -> Result<Url> {
    let url=validate_stream_syntax(raw)?;
    validate_account_key_absent(raw,key)?;Ok(url)
}
fn validate_stream_syntax(raw:&str)->Result<Url> {
    ensure!(raw.len()<=8192,"Invalid media CDN URL");
    let url = Url::parse(raw).map_err(|_| anyhow::anyhow!("Invalid media CDN URL"))?;
    // UTF-8 paths can expand during percent encoding. Bound the exact canonical
    // URL that crosses the native/phone boundary, not only the provider input.
    ensure!(url.as_str().len()<=8192,"Invalid media CDN URL");
    // Match native boundary syntax checks on the original input as well:
    // parsing can normalize empty userinfo, slashes, whitespace or backslashes.
    ensure!(raw.starts_with("https://") && !raw.contains('\\') &&
        !raw.chars().any(|ch|ch.is_control() || ch.is_whitespace()),"Unsafe media CDN URL");
    let authority=raw[8..].split(['/', '?', '#']).next().unwrap_or("");
    ensure!(!authority.is_empty() && !authority.contains('@'),"Unsafe media CDN URL");
    ensure!(url.scheme() == "https" && url.host_str().is_some() && url.username().is_empty() && url.password().is_none() && url.fragment().is_none(), "Unsafe media CDN URL");
    ensure!(!is_torbox_api_origin(&url), "TorBox API credential leaked into media URL");
    Ok(url)
}
fn validate_account_key_absent(raw:&str,key:&str)->Result<()> {
    // A URL-encoded account token is still an account token. Check bounded decode
    // passes and UUID case variants without exposing intermediate bearer strings.
    let normalized_key=Zeroizing::new(key.to_ascii_lowercase());
    let mut decoded=Zeroizing::new(raw.to_owned());
    for _ in 0..4 {
        let lower=Zeroizing::new(decoded.to_ascii_lowercase());
        ensure!(!lower.contains(normalized_key.as_str()),"TorBox API credential leaked into media URL");
        let next=Zeroizing::new(percent_encoding::percent_decode_str(&decoded).decode_utf8_lossy().into_owned());
        if next.as_str()==decoded.as_str() {break;}
        decoded=next;
    }
    let final_lower=Zeroizing::new(decoded.to_ascii_lowercase());
    ensure!(!final_lower.contains(normalized_key.as_str()),"TorBox API credential leaked into media URL");
    let final_decode=Zeroizing::new(percent_encoding::percent_decode_str(&decoded).decode_utf8_lossy().into_owned());
    ensure!(final_decode.as_str()==decoded.as_str(),"Unsafe nested media CDN URL encoding");
    Ok(())
}
#[cfg(feature="private-alpha-account-links")]
fn validate_private_alpha_url(raw:&str,key:&str)->Result<Url> {
    // Prefer the secure form even when this private exception is enabled.
    if let Ok(url)=validate_stream_url(raw,key) {return Ok(url);}
    let url=validate_stream_syntax(raw)?;
    let host=url.host_str().unwrap_or("");
    // Provider's published CDN domains, verified against its support allowlist.
    // Never use a broad shared-hosting suffix or a name inferred from a link.
    let known=["torbox.app","tb-cdn.cx","tb-cdn.io","tb-cdn.pw","tb-cdn.sh","tb-cdn.st","tb-cdn.to","tb-cdn.earth"];
    ensure!(known.iter().any(|domain|host.ends_with(&format!(".{domain}"))) && !host.ends_with('.') && url.port_or_known_default()==Some(443),"Private Alpha CDN origin not approved");
    let pairs:Vec<_>=url.query_pairs().collect();
    ensure!(pairs.iter().filter(|(name,_)|name=="token").count()==1 && pairs.iter().any(|(name,value)|name=="token" && value.as_ref()==key),"Private Alpha CDN token invalid");
    // The exception covers only the known token parameter, never credential
    // copies in paths, other parameters, nested URLs or alternative encodings.
    let mut sanitized=url.clone();
    sanitized.set_query(None);
    {let mut query=sanitized.query_pairs_mut();for (name,value) in &pairs {if name!="token" {query.append_pair(name,value);}}}
    validate_account_key_absent(sanitized.as_str(),key)?;
    Ok(url)
}
fn is_torbox_api_origin(url:&Url)->bool {
    // A DNS root dot does not turn the API origin into a media CDN.
    url.host_str().is_some_and(|host|host.trim_end_matches('.').eq_ignore_ascii_case("api.torbox.app"))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature="private-alpha-account-links")]
    #[test] fn private_alpha_exception_is_exact_token_known_cdn_only_and_default_remains_strict() {
        let key="SYNTHETIC_ACCOUNT_KEY";
        let permitted="https://dl.torbox.app/file.mp4?token=SYNTHETIC_ACCOUNT_KEY";
        assert!(validate_stream_url(permitted,key).is_err());
        assert!(validate_private_alpha_url(permitted,key).is_ok());
        assert!(validate_private_alpha_url("https://nexus-000.indi.tb-cdn.pw/file?token=SYNTHETIC_ACCOUNT_KEY",key).is_ok());
        for raw in ["https://evil.invalid/file?token=SYNTHETIC_ACCOUNT_KEY",
            "https://dl.torbox.app.evil.invalid/file?token=SYNTHETIC_ACCOUNT_KEY",
            "https://nexus.tb-cdn.pw.evil.invalid/file?token=SYNTHETIC_ACCOUNT_KEY",
            "https://api.torbox.app/file?token=SYNTHETIC_ACCOUNT_KEY",
            "https://dl.torbox.app./file?token=SYNTHETIC_ACCOUNT_KEY",
            "https://dl.torbox.app:8443/file?token=SYNTHETIC_ACCOUNT_KEY",
            "https://dl.torbox.app/file?token=SYNTHETIC_ACCOUNT_KEY&token=SYNTHETIC_ACCOUNT_KEY",
            "https://dl.torbox.app/SYNTHETIC_ACCOUNT_KEY?token=SYNTHETIC_ACCOUNT_KEY",
            "https://dl.torbox.app/file?token=SYNTHETIC_ACCOUNT_KEY&other=SYNTHETIC_ACCOUNT_KEY"] {
            let error=validate_private_alpha_url(raw,key).unwrap_err().to_string();assert!(!error.contains(key));
        }
        let strict=TorBox::new(Zeroizing::new(key.into())).unwrap();assert!(matches!(strict.delivery,DeliveryPolicy::FileScopedOnly));
        assert!(validate_private_alpha_url("https://cdn.invalid/file?cap=opaque",key).is_ok());
    }
    #[test] fn api_root_dot_and_encoded_hosts_cannot_be_media_capabilities() {
        for raw in ["https://api.torbox.app./file?cap=opaque", "https://API.TORBOX.APP.:8443/file",
            "https://%61pi.torbox.app./file", "https://api.torbox.app../file"] {
            assert_eq!(validate_stream_url(raw,"different-key").unwrap_err().to_string(),"TorBox API credential leaked into media URL");
        }
        assert!(validate_stream_url("https://cdn.invalid./file?cap=opaque","different-key").is_ok());
        assert!(validate_stream_url("https://api.torbox.app.cdn.invalid/file?cap=opaque","different-key").is_ok());
    }
    #[test] fn normalized_away_raw_syntax_cannot_publish_a_lease() {
        for raw in ["https://@cdn.invalid/file", "https:///cdn.invalid/file", "https://cdn.invalid/\nprivate-input",
            "https://cdn.invalid/\0private-input", "https://cdn.invalid/ private-input", "https://cdn.invalid\\private-input"] {
            let error=validate_stream_url(raw,"different-key").unwrap_err().to_string();
            assert_eq!(error,"Unsafe media CDN URL");
            assert!(!error.contains("private-input"));
        }
        assert!(validate_stream_url("https://cdn.invalid/a%20b.mp4?cap=opaque","different-key").is_ok());
    }
    #[test] fn canonical_media_url_size_is_bounded_after_unicode_encoding() {
        let prefix="https://cdn.invalid/";
        let ascii=format!("{prefix}{}","a".repeat(8192-prefix.len()));
        assert_eq!(validate_stream_url(&ascii,"private-account-key").unwrap().as_str().len(),8192);
        assert!(validate_stream_url(&(ascii+"a"),"private-account-key").is_err());
        let small=format!("{prefix}{}","é".repeat(1000));
        assert!(validate_stream_url(&small,"private-account-key").unwrap().as_str().len()<=8192);
        let expanded=format!("{prefix}{}","é".repeat(2000));assert!(expanded.len()<8192);
        let error=validate_stream_url(&expanded,"private-account-key").unwrap_err().to_string();
        assert_eq!(error,"Invalid media CDN URL");assert!(!error.contains("é"));
    }
    #[test] fn probe_metadata_rejects_wrong_unknown_duplicate_or_compressed_ranges() {
        use reqwest::header::*;
        let mut headers=HeaderMap::new();headers.insert(CONTENT_RANGE,HeaderValue::from_static("bytes 0-1023/4096"));
        assert!(validate_probe_headers(&headers,Some(4096)).is_ok());
        assert!(validate_probe_headers(&headers,None).is_ok());
        headers.insert(CONTENT_RANGE,HeaderValue::from_static("Bytes 0-1023/4096"));
        assert!(validate_probe_headers(&headers,Some(4096)).is_ok());
        assert!(validate_probe_headers(&headers,Some(8192)).is_err());
        for raw in ["bytes 1-1024/4096","bytes 0-1022/4096","bytes 0-1023/*","bytes 0-1023/1023",
            "bytes 0-1023/+4096","bytes 0-1023/18446744073709551616","private-secret https://cdn.invalid"] {
            headers.insert(CONTENT_RANGE,HeaderValue::from_str(raw).unwrap());
            let error=validate_probe_headers(&headers,None).unwrap_err().to_string();
            assert!(!error.contains("private-secret") && !error.contains("https://"));
        }
        headers.insert(CONTENT_RANGE,HeaderValue::from_static("bytes 0-1023/4096"));
        headers.append(CONTENT_RANGE,HeaderValue::from_static("bytes 0-1023/4096"));
        assert!(validate_probe_headers(&headers,None).is_err());
        headers.insert(CONTENT_RANGE,HeaderValue::from_static("bytes 0-1023/4096"));
        headers.insert(CONTENT_ENCODING,HeaderValue::from_static("gzip"));assert!(validate_probe_headers(&headers,None).is_err());
        headers.insert(CONTENT_ENCODING,HeaderValue::from_static("identity"));
        headers.insert(CONTENT_LENGTH,HeaderValue::from_static("1024"));assert!(validate_probe_headers(&headers,Some(4096)).is_ok());
        headers.insert(CONTENT_LENGTH,HeaderValue::from_static("1025"));assert!(validate_probe_headers(&headers,None).is_err());
        headers.remove(CONTENT_RANGE);assert!(validate_probe_headers(&headers,None).is_err());
    }
    #[tokio::test] async fn real_probe_requests_identity_range_and_rejects_wrong_file_size() {
        use tokio::io::{AsyncReadExt,AsyncWriteExt};
        for size in [4096,8192] {
            let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
            let server=tokio::spawn(async move {
                let (mut stream,_)=listener.accept().await.unwrap();let mut request=Vec::new();let mut buffer=[0u8;1024];
                while !request.windows(4).any(|s|s==b"\r\n\r\n") {
                    let count=stream.read(&mut buffer).await.unwrap();assert!(count>0);request.extend_from_slice(&buffer[..count]);assert!(request.len()<=4096);
                }
                let request=String::from_utf8(request).unwrap().to_ascii_lowercase();
                assert!(request.contains("range: bytes=0-1023\r\n"));assert!(request.contains("accept-encoding: identity\r\n"));
                stream.write_all(b"HTTP/1.1 206 Partial Content\r\nContent-Range: bytes 0-1023/4096\r\nContent-Length: 1024\r\nConnection: close\r\n\r\n").await.unwrap();
                // A rejecting client may close before consuming the body.
                let _=stream.write_all(&[42u8;1024]).await;
            });
            let api=TorBox::new(Zeroizing::new("synthetic-secret".into())).unwrap();
            let lease=StreamLease {identity:MediaIdentity {filesize:Some(size),..MediaIdentity::torbox(1,0)},
                url:Url::parse(&format!("http://{address}/fixture?token=synthetic-lease")).unwrap()};
            let result=api.probe(&lease).await;
            if size==4096 {assert_eq!(result.unwrap(),1024);}else {assert_eq!(result.unwrap_err().to_string(),"Media CDN file size mismatch");}
            server.await.unwrap();
        }
    }
    #[test] fn credentials_and_permalinks_cannot_be_delivered() {
        for url in ["http://cdn.test/file", "https://user:pass@cdn.test/file", "https://api.torbox.app/v1/api/torrents/requestdl?token=secret", "https://cdn.test/file?token=secret"] {
            let error = validate_stream_url(url, "secret").unwrap_err().to_string();
            assert!(!error.contains("secret")); assert!(!error.contains("https://"));
        }
        assert!(validate_stream_url("https://cdn.test/file?token=lease-token", "secret").is_ok());
    }
    #[test] fn only_video_files_selected() { assert!(video_filename("movie.MKV")); assert!(!video_filename("movie.mkv.exe")); assert!(!video_filename("README.txt")); }
    #[test] fn fresh_selected_file_rejects_changed_metadata_readiness_scan_and_duplicate_ids() {
        let hash="0123456789012345678901234567890123456789";
        let fixture=||Torrent {id:2,hash:hash.into(),download_present:true,files:Some(vec![File {id:7,name:"movie.mp4".into(),size:4096,infected:Some(false)}])};
        let selected=public_file_identity(2,hash,&fixture().files.unwrap()[0],"synthetic-private-key").unwrap();
        verify_current_file(&fixture(),&selected,"synthetic-private-key").unwrap();
        for mutation in 0..9 {
            let mut torrent=fixture();
            match mutation {
                0=>torrent.id=3,1=>torrent.download_present=false,2=>torrent.hash="ffffffffffffffffffffffffffffffffffffffff".into(),
                3=>torrent.files.as_mut().unwrap()[0].name="other.mp4".into(),4=>torrent.files.as_mut().unwrap()[0].size=8192,
                5=>torrent.files.as_mut().unwrap()[0].infected=None,6=>torrent.files.as_mut().unwrap()[0].infected=Some(true),
                7=>torrent.files.as_mut().unwrap().push(File {id:7,name:"other.mp4".into(),size:4096,infected:Some(false)}),
                _=>torrent.files=None,
            }
            assert!(verify_current_file(&torrent,&selected,"synthetic-private-key").is_err(),"mutation {mutation}");
        }
        assert!(verify_current_file(&fixture(),&MediaIdentity::torbox(2,7),"synthetic-private-key").is_err());
    }
    #[test] fn public_inventory_metadata_rejects_credentials_urls_and_control_text_before_serialization() {
        let hash="0123456789012345678901234567890123456789";
        let file=|name:&str|File {id:7,name:name.into(),size:4096,infected:Some(false)};
        let normal=public_file_identity(2,hash,&file("Series/Season 1/Episode 1.mkv"),"private-account-key").unwrap();
        assert_eq!(normal.file_id,7);assert_eq!(normal.filename.as_deref(),Some("Series/Season 1/Episode 1.mkv"));
        for name in ["private-account-key.mp4","PRIVATE-ACCOUNT-KEY.mkv","private%2Daccount%2Dkey.mp4",
            "private%252Daccount%252Dkey.mp4","https://cdn.invalid/private.mp4","movie\nname.mp4"] {
            let error=public_file_identity(2,hash,&file(name),"private-account-key").unwrap_err().to_string();
            assert!(!error.contains(name));assert!(!error.contains("private-account-key"));
        }
        assert!(public_file_identity(2,hash,&file("movie.mp4"),hash).is_err());
    }
    #[test] fn encoded_or_case_changed_account_tokens_cannot_escape_guard() {
        for raw in ["https://cdn.test/file?token=TEST-SECRET","https://cdn.test/file?token=test%2Dsecret","https://cdn.test/file?token=test%252Dsecret","https://cdn.test/test%2Dsecret/file","https://cdn.test/file?token=test%252525252Dsecret","https://cdn.test/file?token=test%25252525252Dsecret"] {
            assert!(validate_stream_url(raw,"test-secret").is_err());
        }
        assert!(validate_stream_url("https://cdn.test/file#token=test-secret","test-secret").is_err());
    }
    #[test] fn unavailable_files_and_unknown_scan_status_parse_safely() {
        let torrents: Envelope<Vec<Torrent>> = serde_json::from_str(r#"{"success":true,"data":[{"id":1,"hash":"abc","download_present":false,"files":null},{"id":2,"hash":"def","download_present":true,"files":[{"id":0,"name":"movie.mp4","size":100,"infected":null}]}]}"#).unwrap();
        let items=torrents.data.unwrap();
        assert!(items[0].files.is_none());
        assert_ne!(items[1].files.as_ref().unwrap()[0].infected,Some(false));
    }
    #[tokio::test] async fn transport_error_does_not_contain_request_credentials() {
        let listener=std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address=listener.local_addr().unwrap();drop(listener);
        let api=TorBox::new(Zeroizing::new("test-secret".into())).unwrap();
        let request=api.client.get(format!("http://{address}/?token=test-secret"));
        let error=api.response::<serde_json::Value>(request).await.unwrap_err().to_string();
        assert_eq!(error,"TorBox network request failed");
    }
    #[tokio::test] async fn api_error_detail_is_never_exposed() {
        use tokio::io::{AsyncReadExt,AsyncWriteExt};
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address=listener.local_addr().unwrap();
        let server=tokio::spawn(async move {
            let (mut stream,_)=listener.accept().await.unwrap();
            let mut request=[0;4096];let _=stream.read(&mut request).await.unwrap();
            let body=r#"{"success":false,"detail":"test-secret https://private-cdn.test/private","data":null}"#;
            let response=format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body);
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        let api=TorBox::new(Zeroizing::new("test-secret".into())).unwrap();
        let error=api.response::<serde_json::Value>(api.client.get(format!("http://{address}/"))).await.unwrap_err().to_string();
        assert_eq!(error,"TorBox API request rejected");server.await.unwrap();
    }
}

//! Explicit PC-side cached-item preparation. Never follow addon wrappers, delete
//! account items, alter seeding settings, guess file indices, or retry a create.
use super::*;
use crate::stremio::{StreamChoice,MAX_STREAM_CHOICES};
const PREPARATION_READ_BUDGET:Duration=Duration::from_secs(60);
const PREPARATION_READ_TIMEOUT:&str="TorBox preparation checks took too long";
#[derive(Debug)] struct PreflightTimeout;
impl std::fmt::Display for PreflightTimeout {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {f.write_str(PREPARATION_READ_TIMEOUT)}
}
impl std::error::Error for PreflightTimeout {}
/// A fixed local category; callers must not expose arbitrary provider errors.
pub fn is_preflight_timeout(error:&anyhow::Error)->bool {error.is::<PreflightTimeout>()}
fn preflight_timeout()->anyhow::Error {anyhow::Error::new(PreflightTimeout)}
const API:&str="https://api.torbox.app/v1/api/torrents";
#[derive(Deserialize)] struct CachedTorrent {hash:String,files:Vec<CachedFile>}
#[derive(Deserialize)] struct CachedFile {name:String,size:u64}
/// Not serializable/debuggable; only validated filename/size can enter UI.
pub struct CachedSelection {hash:String,filename:String,filesize:u64}
impl CachedSelection {
    pub fn filename(&self)->&str {&self.filename}
    pub fn filesize(&self)->u64 {self.filesize}
    pub fn same_file(&self,other:&Self)->bool {self.hash==other.hash && self.filename==other.filename && self.filesize==other.filesize}
    pub fn matches_media(&self,media:&MediaIdentity)->bool {media.provider==MediaProvider::Torbox
        && media.torrent_hash.as_ref().is_some_and(|hash|hash.eq_ignore_ascii_case(&self.hash))
        && media.filename.as_deref()==Some(self.filename.as_str()) && media.filesize==Some(self.filesize)}
}
pub enum Prepared {Ready(MediaIdentity),Pending}
impl TorBox {
    pub async fn cached_selections(&self,choices:&[StreamChoice])->Result<Vec<(usize,CachedSelection)>> {
        self.cached_selections_at(choices,API).await
    }
    async fn cached_selections_at(&self,choices:&[StreamChoice],api:&str)->Result<Vec<(usize,CachedSelection)>> {
        ensure!(choices.len()<=MAX_STREAM_CHOICES,"Stremio source count exceeded");
        let mut hashes:Vec<String>=choices.iter().filter_map(|choice|choice.cache_reference()).filter(|(hash,_)|hash.len()==40).map(|(hash,_)|hash.to_ascii_lowercase()).collect();
        hashes.sort();hashes.dedup();
        let mut cached=Vec::new();
        for batch in hashes.chunks(100) {
            let entries:Vec<CachedTorrent>=self.response(self.client.get(format!("{api}/checkcached"))
                .bearer_auth(self.key.as_str()).query(&[("hash",batch.join(",")),("format","list".into()),("list_files","true".into())])).await?;
            ensure!(entries.len()<=100,"TorBox cached result count exceeded");
            cached.extend(entries.into_iter().filter(|entry|batch.iter().any(|hash|hash.eq_ignore_ascii_case(&entry.hash))));
        }
        let mut result=Vec::new();
        for (index,choice) in choices.iter().enumerate() {
            let Some((hash,name))=choice.cache_reference() else {continue;};
            if hash.len()!=40 {continue;}
            if let Some(selection)=match_cached(&cached,hash,name,&self.key)? {result.push((index,selection));}
        }
        Ok(result)
    }
    pub async fn prepare_cached(&self,selection:&CachedSelection)->Result<Prepared> {self.prepare_cached_at(selection,API).await}
    async fn prepare_cached_at(&self,selection:&CachedSelection,api:&str)->Result<Prepared> {
        self.prepare_cached_before(selection,api,tokio::time::Instant::now()+PREPARATION_READ_BUDGET).await
    }
    async fn prepare_cached_before(&self,selection:&CachedSelection,api:&str,deadline:tokio::time::Instant)->Result<Prepared> {
        // This budget covers only preflight reads and waiting for create ownership.
        // Once transmitting begins, retain the existing uncertain-create marker.
        if tokio::time::Instant::now()>=deadline {return Err(preflight_timeout());}
        let mut attempts=tokio::time::timeout_at(deadline,self.preparation_attempts.lock()).await
            .map_err(|_|preflight_timeout())?;
        if tokio::time::Instant::now()>=deadline {return Err(preflight_timeout());}
        let existing=tokio::time::timeout_at(deadline,async {
            if let Some(torrent)=self.torrent_by_hash_at(&selection.hash,api).await? {
                return ready_file(&torrent,selection,&self.key).map(Some);
            }
            ensure!(!attempts.contains(&selection.hash),"TorBox preparation was already attempted; check the account before retrying");
            let entries:Vec<CachedTorrent>=self.response(self.client.get(format!("{api}/checkcached"))
                .bearer_auth(self.key.as_str()).query(&[("hash",selection.hash.as_str()),("format","list"),("list_files","true")])).await?;
            ensure!(entries.len()<=100,"TorBox cached result count exceeded");
            let current=match_cached(&entries,&selection.hash,&selection.filename,&self.key)?
                .ok_or_else(||anyhow::anyhow!("Selected TorBox cached video is no longer available"))?;
            ensure!(current.filesize==selection.filesize,"TorBox cached file changed before preparation");
            Ok(None)
        }).await.map_err(|_|preflight_timeout())??;
        if let Some(existing)=existing {return Ok(existing);}
        if tokio::time::Instant::now()>=deadline {return Err(preflight_timeout());}
        // Do not apply the preflight timeout to this write or its follow-up read.
        // A request timeout could have created an item remotely; never retry it.
        attempts.insert(selection.hash.clone());
        #[derive(Deserialize)] struct Created {torrent_id:Option<u64>,queued_id:Option<u64>}
        // Form encoding follows the documented create API. No external trackers,
        // custom seeding policy or provider name are inferred from an addon URL.
        let created:Created=self.response(self.client.post(format!("{api}/createtorrent"))
            .bearer_auth(self.key.as_str()).form(&[("magnet",format!("magnet:?xt=urn:btih:{}",selection.hash)),("allow_zip","false".into())])).await?;
        ensure!(created.torrent_id.is_some() || created.queued_id.is_some(),"TorBox preparation response missing item identity");
        if let Some(id)=created.torrent_id {
            let torrent:Torrent=self.response(self.client.get(format!("{api}/mylist"))
                .bearer_auth(self.key.as_str()).query(&[("id",id.to_string()),("bypass_cache","true".into())])).await?;
            ensure!(torrent.id==id,"TorBox preparation returned a different item");
            return ready_file(&torrent,selection,&self.key);
        }
        Ok(Prepared::Pending)
    }
    async fn torrent_by_hash_at(&self,hash:&str,api:&str)->Result<Option<Torrent>> {
        let mut found=None;
        for page in 0..100u64 {
            let entries:Vec<Torrent>=self.response(self.client.get(format!("{api}/mylist"))
                .bearer_auth(self.key.as_str()).query(&[("offset",(page*100).to_string()),("limit","100".into()),("bypass_cache","true".into())])).await?;
            let end=entries.len()<100;
            for entry in entries.into_iter().filter(|entry|entry.hash.eq_ignore_ascii_case(hash)) {
                ensure!(found.is_none(),"TorBox account has ambiguous copies of the selected torrent");found=Some(entry);
            }
            if end {return Ok(found);}
        }
        anyhow::bail!("TorBox library exceeded pagination limit")
    }
}
fn same_filename(actual:&str,expected:&str)->bool {actual==expected || actual.rsplit('/').next()==Some(expected)}
fn match_cached(entries:&[CachedTorrent],hash:&str,name:&str,key:&str)->Result<Option<CachedSelection>> {
    ensure!(hash.len()==40 && hash.bytes().all(|b|b.is_ascii_hexdigit()),"Cached preparation requires a supported torrent hash");
    validate_account_key_absent(name,key)?;
    ensure!(name.len()<=512 && !name.is_empty() && !name.chars().any(char::is_control) && !name.contains("://"),"Cached filename invalid");
    let matching:Vec<_>=entries.iter().filter(|entry|entry.hash.eq_ignore_ascii_case(hash)).collect();
    ensure!(matching.len()<=1,"TorBox cache returned ambiguous torrent identities");
    let Some(entry)=matching.first() else {return Ok(None);};
    ensure!(entry.files.len()<=10000,"TorBox cache file count exceeded");
    let files:Vec<_>=entry.files.iter().filter(|file|same_filename(&file.name,name) && file.size>=1024 && video_filename(&file.name)).collect();
    if files.len()!=1 {return Ok(None);}
    validate_account_key_absent(&files[0].name,key)?;
    ensure!(files[0].name.len()<=512 && !files[0].name.chars().any(char::is_control) && !files[0].name.contains("://"),"Cached filename invalid");
    Ok(Some(CachedSelection {hash:hash.to_ascii_lowercase(),filename:files[0].name.clone(),filesize:files[0].size}))
}
fn ready_file(torrent:&Torrent,selection:&CachedSelection,key:&str)->Result<Prepared> {
    ensure!(torrent.hash.eq_ignore_ascii_case(&selection.hash),"TorBox preparation returned a different torrent hash");
    if !torrent.download_present {return Ok(Prepared::Pending);}
    let Some(inventory)=torrent.files.as_deref() else {return Ok(Prepared::Pending);};
    let files:Vec<_>=inventory.iter().filter(|file|
        file.size==selection.filesize && same_filename(&file.name,&selection.filename) && video_filename(&file.name)).collect();
    ensure!(files.len()==1,"Prepared TorBox torrent does not identify one exact clean video");
    match files[0].infected {None=>return Ok(Prepared::Pending),Some(true)=>anyhow::bail!("Prepared TorBox video failed its clean scan"),Some(false)=>{}}
    Ok(Prepared::Ready(public_file_identity(torrent.id,&torrent.hash,files[0],key)?))
}
#[cfg(test)] mod tests {
    use super::*;
    const HASH:&str="0123456789012345678901234567890123456789";
    fn selection()->CachedSelection {CachedSelection {hash:HASH.into(),filename:"folder/movie.mp4".into(),filesize:4096}}
    fn torrent(ready:bool)->serde_json::Value {serde_json::json!({"id":27,"hash":HASH,"download_present":ready,
        "files":[{"id":8,"name":"folder/movie.mp4","size":4096,"infected":false}]})}
    fn cache()->serde_json::Value {serde_json::json!([{"hash":HASH,"files":[{"name":"folder/movie.mp4","size":4096}]}])}
    #[test] fn cached_and_prepared_files_require_exact_unique_clean_identity_and_size() {
        let mut entries:Vec<CachedTorrent>=serde_json::from_value(cache()).unwrap();
        assert_eq!(match_cached(&entries,HASH,"movie.mp4","private-account").unwrap().unwrap().filesize,4096);
        entries[0].files.push(CachedFile {name:"another/movie.mp4".into(),size:4096});
        assert!(match_cached(&entries,HASH,"movie.mp4","private-account").unwrap().is_none());
        assert!(match_cached(&entries,HASH,"private-account.mp4","private-account").is_err());
        let mut item:Torrent=serde_json::from_value(torrent(true)).unwrap();
        let Prepared::Ready(media)=ready_file(&item,&selection(),"private-account").unwrap() else {panic!("not ready")};assert_eq!(media.file_id,8);
        item.files.as_mut().unwrap()[0].infected=None;assert!(matches!(ready_file(&item,&selection(),"private-account").unwrap(),Prepared::Pending));
        item.files.as_mut().unwrap()[0].infected=Some(true);assert!(ready_file(&item,&selection(),"private-account").is_err());
        item.download_present=false;assert!(matches!(ready_file(&item,&selection(),"private-account").unwrap(),Prepared::Pending));
        item.hash="ffffffffffffffffffffffffffffffffffffffff".into();assert!(ready_file(&item,&selection(),"private-account").is_err());
    }
    async fn api(responses:Vec<(&'static str,serde_json::Value)>)->(String,tokio::task::JoinHandle<()>) {
        api_delayed(responses,None).await
    }
    async fn api_delayed(responses:Vec<(&'static str,serde_json::Value)>,delay_create:Option<Duration>)->(String,tokio::task::JoinHandle<()>) {
        use tokio::io::{AsyncReadExt,AsyncWriteExt};
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base=format!("http://{}/v1/api/torrents",listener.local_addr().unwrap());
        let task=tokio::spawn(async move {for (expected,data) in responses {
            let (mut socket,_)=listener.accept().await.unwrap();let mut request=Vec::new();let mut buffer=[0u8;2048];
            loop {
                let count=socket.read(&mut buffer).await.unwrap();assert!(count>0);request.extend_from_slice(&buffer[..count]);assert!(request.len()<=8192);
                if let Some(end)=request.windows(4).position(|bytes|bytes==b"\r\n\r\n") {
                    let headers=String::from_utf8_lossy(&request[..end]).to_ascii_lowercase();
                    let length=headers.lines().find_map(|line|line.strip_prefix("content-length: ")).map(|n|n.parse::<usize>().unwrap()).unwrap_or(0);
                    if request.len()>=end+4+length {break;}
                }
            }
            let text=String::from_utf8(request).unwrap();assert!(text.starts_with(expected));
            assert!(!text.contains("controltorrent") && !text.contains("token="));
            if expected.starts_with("POST") {
                assert!(text.contains("magnet=magnet%3A%3Fxt%3Durn%3Abtih%3A"));assert!(text.contains("allow_zip=false"));assert!(!text.contains("seed="));
            }
            if expected.starts_with("POST") {if let Some(delay)=delay_create {tokio::time::sleep(delay).await;}}
            let body=serde_json::json!({"success":true,"data":data}).to_string();
            let response=format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
            socket.write_all(response.as_bytes()).await.unwrap();
        }});(base,task)
    }
    #[tokio::test] async fn existing_item_is_reused_and_cached_creation_loads_only_returned_canonical_file() {
        let client=TorBox::new(Zeroizing::new("synthetic-key".into())).unwrap();
        let (base,server)=api(vec![("GET /v1/api/torrents/mylist?",serde_json::json!([torrent(true)]))]).await;
        assert!(matches!(client.prepare_cached_at(&selection(),&base).await.unwrap(),Prepared::Ready(_)));server.await.unwrap();
        let (base,server)=api(vec![("GET /v1/api/torrents/mylist?",serde_json::json!([])),
            ("GET /v1/api/torrents/checkcached?",cache()),("POST /v1/api/torrents/createtorrent ",serde_json::json!({"torrent_id":27})),
            ("GET /v1/api/torrents/mylist?id=27&bypass_cache=true ",torrent(true))]).await;
        let Prepared::Ready(media)=client.prepare_cached_at(&selection(),&base).await.unwrap() else {panic!("not ready")};
        assert_eq!((media.torrent_id,media.file_id),(27,8));server.await.unwrap();
    }
    #[tokio::test] async fn uncertain_create_is_not_repeated_and_pending_items_do_not_trigger_create() {
        let client=TorBox::new(Zeroizing::new("synthetic-key".into())).unwrap();
        let (base,server)=api(vec![("GET /v1/api/torrents/mylist?",serde_json::json!([])),
            ("GET /v1/api/torrents/checkcached?",cache()),("POST /v1/api/torrents/createtorrent ",serde_json::json!({"unexpected":"private-detail"})),
            ("GET /v1/api/torrents/mylist?",serde_json::json!([])),("GET /v1/api/torrents/mylist?",serde_json::json!([torrent(false)]))]).await;
        assert!(client.prepare_cached_at(&selection(),&base).await.is_err());
        let error=client.prepare_cached_at(&selection(),&base).await.err().unwrap();assert!(!is_preflight_timeout(&error));
        let text=error.to_string();assert!(text.contains("already attempted") && !text.contains("private-detail"));
        assert!(matches!(client.prepare_cached_at(&selection(),&base).await.unwrap(),Prepared::Pending));server.await.unwrap();
    }
    #[tokio::test] async fn transmitted_create_is_not_cancelled_by_the_preflight_deadline() {
        let client=TorBox::new(Zeroizing::new("synthetic-key".into())).unwrap();
        let (base,server)=api_delayed(vec![
            ("GET /v1/api/torrents/mylist?",serde_json::json!([])),
            ("GET /v1/api/torrents/checkcached?",cache()),
            ("POST /v1/api/torrents/createtorrent ",serde_json::json!({"queued_id":27}))],
            Some(Duration::from_millis(500))).await;
        let deadline=tokio::time::Instant::now()+Duration::from_millis(300);
        assert!(matches!(client.prepare_cached_before(&selection(),&base,deadline).await.unwrap(),Prepared::Pending));
        assert!(tokio::time::Instant::now()>deadline);
        assert!(client.preparation_attempts.lock().await.contains(HASH));server.await.unwrap();
    }
    #[tokio::test] async fn expired_preflight_or_owner_wait_never_transmits_or_marks_a_create() {
        let client=TorBox::new(Zeroizing::new("synthetic-key".into())).unwrap();
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base=format!("http://{}/v1/api/torrents",listener.local_addr().unwrap());
        let error=client.prepare_cached_before(&selection(),&base,tokio::time::Instant::now()).await.err().unwrap();
        assert_eq!(error.to_string(),PREPARATION_READ_TIMEOUT);assert!(is_preflight_timeout(&error));
        let held=client.preparation_attempts.lock().await;
        let error=client.prepare_cached_before(&selection(),&base,
            tokio::time::Instant::now()+Duration::from_millis(20)).await.err().unwrap();
        assert_eq!(error.to_string(),PREPARATION_READ_TIMEOUT);assert!(is_preflight_timeout(&error));assert!(held.is_empty());drop(held);
        assert!(tokio::time::timeout(Duration::from_millis(20),listener.accept()).await.is_err());
    }
    #[tokio::test] async fn stalled_preflight_body_is_cancelled_and_does_not_poison_a_later_explicit_check() {
        use tokio::io::{AsyncReadExt,AsyncWriteExt};
        let client=TorBox::new(Zeroizing::new("synthetic-key".into())).unwrap();
        for stall_cache in [false,true] {
            let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let base=format!("http://{}/v1/api/torrents",listener.local_addr().unwrap());
            let server=tokio::spawn(async move {
                let mut buffer=[0u8;2048];
                if stall_cache {
                    let (mut inventory,_)=listener.accept().await.unwrap();let mut request=Vec::new();
                    while !request.windows(4).any(|bytes|bytes==b"\r\n\r\n") {
                        let n=inventory.read(&mut buffer).await.unwrap();assert!(n>0);request.extend_from_slice(&buffer[..n]);assert!(request.len()<=8192);
                    }
                    assert!(request.starts_with(b"GET /v1/api/torrents/mylist?"));
                    let body=serde_json::json!({"success":true,"data":[]}).to_string();
                    inventory.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
                }
                let (mut socket,_)=listener.accept().await.unwrap();let mut request=Vec::new();
                while !request.windows(4).any(|bytes|bytes==b"\r\n\r\n") {
                    let n=socket.read(&mut buffer).await.unwrap();assert!(n>0);request.extend_from_slice(&buffer[..n]);assert!(request.len()<=8192);
                }
                assert!(request.starts_with(if stall_cache {b"GET /v1/api/torrents/checkcached?"}else{b"GET /v1/api/torrents/mylist?"}));
                // Valid metadata headers with a body that never finishes. Dropping
                // the timed-out response must release this actual HTTP connection.
                socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 4096\r\nConnection: close\r\n\r\n{").await.unwrap();
                assert_eq!(tokio::time::timeout(Duration::from_secs(2),socket.read(&mut buffer)).await.unwrap().unwrap(),0);
                assert!(tokio::time::timeout(Duration::from_millis(20),listener.accept()).await.is_err(),"preflight timeout sent another request");
            });
            let error=client.prepare_cached_before(&selection(),&base,
                tokio::time::Instant::now()+Duration::from_millis(200)).await.err().unwrap();
            assert_eq!(error.to_string(),PREPARATION_READ_TIMEOUT);assert!(is_preflight_timeout(&error));
            assert!(client.preparation_attempts.lock().await.is_empty());server.await.unwrap();
            let (base,server)=api(vec![("GET /v1/api/torrents/mylist?",serde_json::json!([torrent(true)]))]).await;
            assert!(matches!(client.prepare_cached_at(&selection(),&base).await.unwrap(),Prepared::Ready(_)));
            server.await.unwrap();
        }
    }
}

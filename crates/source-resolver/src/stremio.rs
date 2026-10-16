//! Read-only PC Stremio account access. Account/addon credentials never enter UI models.
use anyhow::{ensure,Result};
use reqwest::Client;
use serde::{Deserialize,Serialize};
use zeroize::{Zeroize,Zeroizing};
use std::time::Duration;
/// Shared bound for one discovery result, including combined addon responses.
pub const MAX_STREAM_CHOICES:usize=1000;

#[derive(Clone,Serialize)]
pub struct LibraryTitle {pub id:String,pub kind:String,pub name:String}
#[derive(Clone,Serialize)]
pub struct Catalog {pub addon_index:usize,pub catalog_index:usize,pub kind:String,pub name:String,pub searchable:bool,pub search_required:bool}
pub struct Stremio {client:Client,key:Zeroizing<String>}
/// Configured addon transports can embed credentials. Keep them PC-private.
pub struct Addon {transport:reqwest::Url,manifest:serde_json::Value}
/// A stream descriptor is private PC data. It cannot be serialized into UI,
/// room snapshots or logs, and never becomes a native URL directly.
pub struct StreamChoice {name:String,hash:Option<String>,filename:Option<String>,provider_ids:Option<(u64,u64)>}
impl StreamChoice {
    /// A user may copy a Stremio/Torrentio link after starting it on the PC.
    /// Decode only an independently recognized identity; never follow or return it.
    pub fn from_copied_link(raw:Zeroizing<String>)->Result<Self> {
        let (hash,filename)=torrentio_reference(raw.trim()).ok_or_else(||anyhow::anyhow!("Copied link is not a recognized direct TorBox source reference"))?;
        Ok(Self {name:"Copied Stremio source".into(),hash:Some(hash),filename:Some(filename),provider_ids:None})
    }
    pub fn name(&self)->&str {&self.name}
    pub(crate) fn cache_reference(&self)->Option<(&str,&str)> {Some((self.hash.as_deref()?,self.filename.as_deref()?))}
    /// Resolve only against the PC's canonical available TorBox files. An addon
    /// fileIdx is not assumed to be a TorBox file ID. Ambiguity fails closed.
    pub fn canonical_media(&self,available:&[super::MediaIdentity])->Result<super::MediaIdentity> {
        let matches:Vec<_>=available.iter().filter(|media| {
            if media.provider!=super::MediaProvider::Torbox {return false;}
            if let Some((torrent,file))=self.provider_ids {return media.torrent_id==torrent && media.file_id==file;}
            self.hash.as_ref().is_some_and(|hash|media.torrent_hash.as_ref().is_some_and(|value|value.eq_ignore_ascii_case(hash)))
                && self.filename.as_ref().is_some_and(|name|media.filename.as_ref().is_some_and(|filename|
                    filename==name || filename.rsplit('/').next()==Some(name.as_str())))
        }).collect();
        ensure!(matches.len()==1,"Stremio stream does not identify one available TorBox video");
        matches[0].validate().map_err(anyhow::Error::msg)?;Ok(matches[0].clone())
    }
}
impl Addon {
    pub fn supports_stream(&self,kind:&str,id:&str)->bool {
        self.supports("stream",kind,id)
    }
    pub fn supports_meta(&self,kind:&str,id:&str)->bool {self.supports("meta",kind,id)}
    fn supports(&self,name:&str,kind:&str,id:&str)->bool {
        let allows=|definition:&serde_json::Value| {
            let types=definition["types"].as_array().or_else(||self.manifest["types"].as_array());
            let prefixes=definition["idPrefixes"].as_array().or_else(||self.manifest["idPrefixes"].as_array());
            types.map_or(true,|types|types.iter().any(|value|value.as_str()==Some(kind)))
                && prefixes.map_or(true,|prefixes|prefixes.iter().any(|value|value.as_str().is_some_and(|prefix|id.starts_with(prefix))))
        };
        self.manifest["resources"].as_array().is_some_and(|resources|resources.iter().any(|resource|
            (resource.as_str()==Some(name) || resource["name"].as_str()==Some(name)) && allows(resource)))
    }
    pub fn catalog_count(&self)->usize {self.manifest["catalogs"].as_array().map_or(0,Vec::len)}
    pub fn has_streams(&self)->bool {
        self.manifest["resources"].as_array().is_some_and(|resources|resources.iter().any(|resource|
            resource.as_str()==Some("stream") || resource["name"].as_str()==Some("stream")))
    }
}
fn http_client()->Result<Client> {
    Client::builder().timeout(Duration::from_secs(20)).connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none()).user_agent("Together2/0.1")
        .build().map_err(|_|anyhow::anyhow!("Stremio HTTP initialization failed"))
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct LibraryRequest<'a> {auth_key:&'a str,collection:&'static str,all:bool,ids:Vec<String>}
#[derive(Deserialize)]
struct Response {result:Option<Vec<Item>>,error:Option<serde_json::Value>}
#[derive(Deserialize)]
struct Item {#[serde(rename="_id")] id:String,#[serde(rename="type")] kind:String,name:String,#[serde(default)] removed:bool}
impl Stremio {
    pub async fn episodes(&self,addon:&Addon,id:&str)->Result<Vec<LibraryTitle>> {
        ensure!(public_text(id,256,&self.key) && id!="." && id!=".." && addon.supports_meta("series",id),"Stremio series metadata unavailable");
        let prefix=addon.transport.path().strip_suffix("manifest.json").ok_or_else(||anyhow::anyhow!("Stremio addon transport invalid"))?;
        let encoded=percent_encoding::utf8_percent_encode(id,percent_encoding::NON_ALPHANUMERIC);
        let mut url=addon.transport.clone();url.set_path(&format!("{prefix}meta/series/{encoded}.json"));
        let response=self.client.get(url).send().await.map_err(|_|anyhow::anyhow!("Stremio episode request failed"))?;
        let body=bounded_response(response,4*1024*1024).await?;parse_episodes(&body,id,&self.key)
    }
    pub fn catalogs(&self,addons:&[Addon])->Vec<Catalog> {
        addons.iter().enumerate().flat_map(|(addon_index,addon)| {
            addon.manifest["catalogs"].as_array().into_iter().flatten().enumerate().filter_map(move |(catalog_index,catalog)| {
                let kind=catalog["type"].as_str()?;
                let id=catalog["id"].as_str()?;
                let name=catalog["name"].as_str().unwrap_or(id);
                if !["movie","series"].contains(&kind) || !public_text(name,256,&self.key) || !public_text(id,256,&self.key) {return None;}
                let extras=catalog["extra"].as_array();
                if extras.is_some_and(|items|items.iter().any(|extra|extra["isRequired"]==true && extra["name"]!="search")) {return None;}
                let searchable=extras.is_some_and(|items|items.iter().any(|extra|extra["name"]=="search"));
                let search_required=extras.is_some_and(|items|items.iter().any(|extra|extra["name"]=="search" && extra["isRequired"]==true));
                Some(Catalog {addon_index,catalog_index,kind:kind.into(),name:name.into(),searchable,search_required})
            })
        }).take(512).collect()
    }
    /// Query only the chosen configured catalog. Credentials are not attached as
    /// headers; an addon uses its own private configured transport path.
    pub async fn catalog(&self,addon:&Addon,index:usize,search:Option<&str>)->Result<Vec<LibraryTitle>> {
        let definition=addon.manifest["catalogs"].as_array().and_then(|catalogs|catalogs.get(index))
            .ok_or_else(||anyhow::anyhow!("Choose a Stremio catalog"))?;
        let url=catalog_url(&addon.transport,definition,search)?;
        let response=self.client.get(url).send().await.map_err(|_|anyhow::anyhow!("Stremio catalog request failed"))?;
        let body=bounded_response(response,4*1024*1024).await?;
        parse_catalog(&body,&self.key)
    }
    pub async fn streams(&self,addon:&Addon,kind:&str,id:&str)->Result<Vec<StreamChoice>> {
        ensure!(["movie","series"].contains(&kind) && !id.is_empty() && id.len()<=256 && id!="." && id!=".." && !id.chars().any(char::is_control),"Stremio video identity invalid");
        ensure!(addon.supports_stream(kind,id),"Stremio addon does not support this video identity");
        let prefix=addon.transport.path().strip_suffix("manifest.json").ok_or_else(||anyhow::anyhow!("Stremio addon transport invalid"))?;
        let encode=|s:&str|percent_encoding::utf8_percent_encode(s,percent_encoding::NON_ALPHANUMERIC).to_string();
        let mut url=addon.transport.clone();url.set_path(&format!("{prefix}stream/{}/{}.json",encode(kind),encode(id)));
        let response=self.client.get(url).send().await.map_err(|_|anyhow::anyhow!("Stremio stream request failed"))?;
        let body=bounded_response(response,4*1024*1024).await?;
        parse_streams(&body,&self.key)
    }
    /// Existing DPAPI intake stores email/password; only this fixed official
    /// endpoint receives them. The resulting session stays in PC memory.
    pub async fn from_pc_secret(secret:Zeroizing<String>)->Result<Self> {
        if let Ok(client)=Self::new(Zeroizing::new(secret.to_string())) {return Ok(client);}
        #[derive(Deserialize,Serialize)] struct Login {
            email:String,password:String,#[serde(rename="type",default="login_type")] kind:String,
            #[serde(default)] facebook:bool,
        }
        fn login_type()->String {"Login".into()}
        impl Drop for Login {fn drop(&mut self) {self.email.zeroize();self.password.zeroize();}}
        let mut login:Login=serde_json::from_str(&secret).map_err(|_|anyhow::anyhow!("Stremio credential format invalid"))?;
        ensure!(!login.email.is_empty() && login.email.len()<=320 && !login.password.is_empty() && login.password.len()<=4096,"Stremio credential format invalid");
        login.kind="Login".into();login.facebook=false;
        let client=http_client()?;
        let response=client.post("https://api.strem.io/api/login").json(&login).send().await
            .map_err(|_|anyhow::anyhow!("Stremio sign-in request failed"))?;
        let body=bounded_response(response,64*1024).await?;
        #[derive(Deserialize)] struct LoginResponse {result:Option<Auth>,error:Option<serde_json::Value>}
        #[derive(Deserialize)] struct Auth {#[serde(rename="authKey")] auth_key:String}
        let response:LoginResponse=serde_json::from_slice(&body).map_err(|_|anyhow::anyhow!("Stremio sign-in response invalid"))?;
        ensure!(response.error.is_none(),"Stremio sign-in rejected");
        let auth=response.result.ok_or_else(||anyhow::anyhow!("Stremio sign-in session missing"))?;
        Self::new(Zeroizing::new(auth.auth_key))
    }
    /// Accept an auth key or the existing encrypted PC session object's authKey.
    /// This does not sign in, change the account, or persist new credentials.
    pub fn new(secret:Zeroizing<String>)->Result<Self> {
        let key=if secret.trim_start().starts_with('{') {
            #[derive(Deserialize)] struct SessionSecret {#[serde(rename="authKey")] auth_key:String}
            let session:SessionSecret=serde_json::from_str(&secret).map_err(|_|anyhow::anyhow!("Stremio credential format invalid"))?;
            Zeroizing::new(session.auth_key)
        }else {secret};
        ensure!((16..=4096).contains(&key.len()) && key.bytes().all(|b|(33..=126).contains(&b)),"Stremio credential format invalid");
        let client=http_client()?;
        Ok(Self {client,key})
    }
    pub async fn addons(&self)->Result<Vec<Addon>> {
        #[derive(Serialize)] #[serde(rename_all="camelCase")] struct Request<'a> {auth_key:&'a str,update:bool}
        let response=self.client.post("https://api.strem.io/api/addonCollectionGet")
            .json(&Request {auth_key:&self.key,update:false}).send().await
            .map_err(|_|anyhow::anyhow!("Stremio addon request failed"))?;
        let body=bounded_response(response,4*1024*1024).await?;
        parse_addons(&body)
    }
    pub async fn library(&self)->Result<Vec<LibraryTitle>> {
        let request=LibraryRequest {auth_key:&self.key,collection:"libraryItem",all:true,ids:Vec::new()};
        let response=self.client.post("https://api.strem.io/api/datastoreGet").json(&request).send().await
            .map_err(|_|anyhow::anyhow!("Stremio account request failed"))?;
        let body=bounded_response(response,4*1024*1024).await?;
        parse_library(&body,&self.key)
    }
}
fn parse_streams(body:&[u8],secret:&str)->Result<Vec<StreamChoice>> {
    #[derive(Deserialize)] struct Envelope {streams:Vec<Stream>}
    #[derive(Deserialize)] #[serde(rename_all="camelCase")] struct Stream {
        name:Option<String>,info_hash:Option<String>,url:Option<String>,behavior_hints:Option<Hints>,
    }
    #[derive(Deserialize)] struct Hints {filename:Option<String>}
    let envelope:Envelope=serde_json::from_slice(body).map_err(|_|anyhow::anyhow!("Stremio stream response invalid"))?;
    ensure!(envelope.streams.len()<=MAX_STREAM_CHOICES,"Stremio stream count exceeded");
    Ok(envelope.streams.into_iter().map(|mut stream| {
        // Provider text can accidentally contain credentials unrelated to the
        // Stremio account key. Never expose raw stream names/descriptions.
        let name="Stream option".into();
        if let Some(name)=stream.name.as_mut() {name.zeroize();}
        let mut hash=stream.info_hash.filter(|hash|[40,64].contains(&hash.len()) && hash.bytes().all(|b|b.is_ascii_hexdigit()));
        let mut filename=stream.behavior_hints.and_then(|hints|hints.filename).filter(|name|public_text(name,512,secret));
        if hash.is_none() {
            if let Some((reference_hash,reference_name))=stream.url.as_deref().and_then(torrentio_reference) {
                hash=Some(reference_hash);filename=Some(reference_name);
            }
        }
        // API IDs are a canonical reference, not a link to forward. Reject
        // duplicate parameters, aliases and unrelated service paths.
        let provider_ids=stream.url.as_ref().and_then(|raw|reqwest::Url::parse(raw).ok()).and_then(|url| {
            if url.scheme()!="https" || url.host_str()!=Some("api.torbox.app") || url.path()!="/v1/api/torrents/requestdl"
                || !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {return None;}
            let pairs:Vec<_>=url.query_pairs().collect();
            let parameter=|name:&str| {let values:Vec<_>=pairs.iter().filter(|(key,_)|key==name).collect();
                if values.len()!=1 {None}else{values[0].1.parse::<u64>().ok()}};
            Some((parameter("torrent_id")?,parameter("file_id")?))
        });
        if let Some(url)=stream.url.as_mut() {url.zeroize();}
        StreamChoice {name,hash,filename,provider_ids}
    }).collect())
}
/// Independently implemented from Torrentio's public TorBox route contract.
/// Inspect only; do not follow the wrapper, use its key or run its create/delete
/// fallback. The PC resolves its own canonical file and mints its own leases.
fn torrentio_reference(raw:&str)->Option<(String,String)> {
    if raw.len()>8192 || !raw.starts_with("https://") || raw.contains('\\') || raw.chars().any(|c|c.is_control() || c.is_whitespace()) {return None;}
    let url=reqwest::Url::parse(raw).ok()?;
    if url.host_str()!=Some("torrentio.strem.fun") || url.port_or_known_default()!=Some(443) || !url.username().is_empty()
        || url.password().is_some() || url.fragment().is_some() || url.query().is_some() {return None;}
    let segments:Vec<_>=url.path_segments()?.collect();
    if segments.len()!=7 || segments[0]!="resolve" || segments[1]!="torbox" {return None;}
    let hash=segments[3];
    if ![40,64].contains(&hash.len()) || !hash.bytes().all(|b|b.is_ascii_hexdigit()) {return None;}
    let filename=percent_encoding::percent_decode_str(segments[4]).decode_utf8().ok()?.into_owned();
    if filename.is_empty() || filename.len()>512 || filename.chars().any(char::is_control) || filename.contains("://") {return None;}
    Some((hash.into(),filename))
}
fn parse_addons(body:&[u8])->Result<Vec<Addon>> {
    #[derive(Deserialize)] struct Envelope {result:Option<Collection>,error:Option<serde_json::Value>}
    #[derive(Deserialize)] struct Collection {addons:Vec<Descriptor>}
    #[derive(Deserialize)] struct Descriptor {#[serde(rename="transportUrl")] transport: String,manifest:serde_json::Value}
    let response:Envelope=serde_json::from_slice(body).map_err(|_|anyhow::anyhow!("Stremio addon response invalid"))?;
    ensure!(response.error.is_none(),"Stremio addon request rejected");
    let descriptors=response.result.ok_or_else(||anyhow::anyhow!("Stremio addons missing"))?.addons;
    ensure!(descriptors.len()<=128,"Stremio addon count exceeded");
    Ok(descriptors.into_iter().filter_map(|mut descriptor| {
        let raw=&descriptor.transport;
        let syntax=raw.len()<=8192 && raw.starts_with("https://") && !raw.contains('\\') && !raw.chars().any(|c|c.is_control() || c.is_whitespace())
            && !raw[8..].split(['/', '?', '#']).next().unwrap_or("").contains('@');
        let url=if syntax {reqwest::Url::parse(raw).ok()}else{None};descriptor.transport.zeroize();
        let transport=url.filter(|url|url.scheme()=="https" && url.username().is_empty() && url.password().is_none()
            && url.fragment().is_none() && url.query().is_none() && url.host_str().is_some()
            && url.path().ends_with("/manifest.json"))?;
        Some(Addon {transport,manifest:descriptor.manifest})
    }).collect())
}
fn catalog_url(transport:&reqwest::Url,definition:&serde_json::Value,search:Option<&str>)->Result<reqwest::Url> {
    let kind=definition["type"].as_str().filter(|kind|["movie","series"].contains(kind))
        .ok_or_else(||anyhow::anyhow!("Stremio catalog type unsupported"))?;
    let id=definition["id"].as_str().filter(|id|!id.is_empty() && id.len()<=256 && *id!="." && *id!=".." && !id.chars().any(char::is_control))
        .ok_or_else(||anyhow::anyhow!("Stremio catalog ID invalid"))?;
    let extras=definition["extra"].as_array();
    let query=search.map(str::trim).filter(|s|!s.is_empty());
    if let Some(query)=query {
        // Count Unicode scalar values, not UTF-8 bytes; stop once over budget.
        ensure!((2..=160).contains(&query.chars().take(161).count()) && !query.chars().any(char::is_control),"Search needs 2 to 160 characters");
        ensure!(extras.is_some_and(|items|items.iter().any(|extra|extra["name"]=="search")),"This catalog does not support search");
    }
    if let Some(extras)=extras {
        ensure!(!extras.iter().any(|extra|extra["isRequired"]==true && (extra["name"]!="search" || query.is_none())),"This catalog requires a search or filter");
    }
    let encode=|s:&str|percent_encoding::utf8_percent_encode(s,percent_encoding::NON_ALPHANUMERIC).to_string();
    let prefix=transport.path().strip_suffix("manifest.json").ok_or_else(||anyhow::anyhow!("Stremio addon transport invalid"))?;
    let mut path=format!("{prefix}catalog/{}/{}",encode(kind),encode(id));
    if let Some(query)=query {path.push_str(&format!("/search={}",encode(query)));}
    path.push_str(".json");
    let mut url=transport.clone();url.set_path(&path);Ok(url)
}
fn public_text(s:&str,limit:usize,secret:&str)->bool {
    !s.is_empty() && s.len()<=limit && !s.chars().any(char::is_control)
        && !s.to_ascii_lowercase().contains(&secret.to_ascii_lowercase()) && !s.contains("://")
}
fn parse_catalog(body:&[u8],secret:&str)->Result<Vec<LibraryTitle>> {
    #[derive(Deserialize)] struct Envelope {metas:Vec<Meta>}
    #[derive(Deserialize)] struct Meta {id:String,#[serde(rename="type")] kind:String,name:String}
    let envelope:Envelope=serde_json::from_slice(body).map_err(|_|anyhow::anyhow!("Stremio catalog format invalid"))?;
    ensure!(envelope.metas.len()<=2000,"Stremio catalog too large");
    let mut seen=std::collections::HashSet::new();
    Ok(envelope.metas.into_iter().filter(|item|["movie","series"].contains(&item.kind.as_str())
        && public_text(&item.id,256,secret) && public_text(&item.name,512,secret)
        && seen.insert((item.kind.clone(),item.id.clone())))
        .map(|item|LibraryTitle {id:item.id,kind:item.kind,name:item.name}).collect())
}
fn parse_episodes(body:&[u8],series_id:&str,secret:&str)->Result<Vec<LibraryTitle>> {
    #[derive(Deserialize)] struct Envelope {meta:Meta}
    #[derive(Deserialize)] struct Meta {id:String,#[serde(rename="type")] kind:String,videos:Vec<Episode>}
    #[derive(Deserialize)] struct Episode {id:String,title:Option<String>,season:Option<u32>,episode:Option<u32>}
    let envelope:Envelope=serde_json::from_slice(body).map_err(|_|anyhow::anyhow!("Stremio episode response invalid"))?;
    ensure!(envelope.meta.id==series_id && envelope.meta.kind=="series","Stremio returned a different series");
    ensure!(envelope.meta.videos.len()<=10000,"Stremio episode count exceeded");
    let mut seen=std::collections::HashSet::new();
    let mut episodes:Vec<_>=envelope.meta.videos.into_iter().filter(|episode|public_text(&episode.id,256,secret)
        && episode.season.is_some_and(|n|n<=10000) && episode.episode.is_some_and(|n|n<=10000) && seen.insert(episode.id.clone())).collect();
    episodes.sort_by_key(|episode|(episode.season,episode.episode));
    Ok(episodes.into_iter().map(|episode| {
        let mut name=format!("S{} E{}",episode.season.unwrap(),episode.episode.unwrap());
        if let Some(title)=episode.title.filter(|title|public_text(title,400,secret)) {name.push_str(" · ");name.push_str(&title);}
        LibraryTitle {id:episode.id,kind:"series".into(),name}
    }).collect())
}
async fn bounded_response(mut response:reqwest::Response,limit:usize)->Result<Zeroizing<Vec<u8>>> {
    ensure!(response.status().is_success(),"Stremio account request rejected");
    let mut body=Zeroizing::new(Vec::new());
    while let Some(chunk)=response.chunk().await.map_err(|_|anyhow::anyhow!("Stremio response interrupted"))? {
        ensure!(body.len()+chunk.len()<=limit,"Stremio response too large");body.extend_from_slice(&chunk);
    }
    Ok(body)
}
fn parse_library(body:&[u8],secret:&str)->Result<Vec<LibraryTitle>> {
    let response:Response=serde_json::from_slice(body).map_err(|_|anyhow::anyhow!("Stremio library format invalid"))?;
    ensure!(response.error.is_none(),"Stremio account request rejected");
    let items=response.result.ok_or_else(||anyhow::anyhow!("Stremio library missing"))?;
    ensure!(items.len()<=10_000,"Stremio library too large");
    let mut seen=std::collections::HashSet::new();
    Ok(items.into_iter().filter(|item|!item.removed && ["movie","series"].contains(&item.kind.as_str())
        && public_text(&item.id,256,secret) && public_text(&item.name,512,secret) && seen.insert((item.kind.clone(),item.id.clone())))
        .map(|item|LibraryTitle {id:item.id,kind:item.kind,name:item.name}).collect())
}
#[cfg(test)] mod tests {
    #[test] fn stream_count_boundary_rejects_oversized_provider_response() {
        let body=|count|serde_json::to_vec(&serde_json::json!({"streams":vec![serde_json::json!({});count]})).unwrap();
        assert_eq!(parse_streams(&body(MAX_STREAM_CHOICES),"synthetic-private-key").unwrap().len(),MAX_STREAM_CHOICES);
        assert!(parse_streams(&body(MAX_STREAM_CHOICES+1),"synthetic-private-key").is_err());
    }
    use super::*;
    #[test] fn library_is_read_only_public_metadata_without_urls_credentials_or_removed_items() {
        let raw=br#"{"result":[{"_id":"tt1254207","type":"movie","name":"Big Buck Bunny","state":{"timeOffset":30000},"transportUrl":"https://addon.invalid/private"},{"_id":"tt2","type":"movie","name":"removed","removed":true},{"_id":"tt3","type":"series","name":"https://private.invalid"},{"_id":"tt4","type":"movie","name":"PRIVATE_ACCOUNT_KEY"},{"_id":"tt1254207","type":"movie","name":"duplicate"}]}"#;
        let result=parse_library(raw,"PRIVATE_ACCOUNT_KEY").unwrap();assert_eq!(result.len(),1);
        let encoded=serde_json::to_string(&result).unwrap();
        assert!(!encoded.contains("private") && !encoded.contains("timeOffset") && !encoded.contains("transportUrl"));
    }
    #[test] fn private_response_errors_are_never_exposed() {
        for raw in [br#"{"error":{"message":"https://private.invalid/key"}}"#.as_slice(),br#"{"result":null}"#,b"not json private-key"] {
            let error=parse_library(raw,"PRIVATE_ACCOUNT_KEY").err().unwrap().to_string();assert!(!error.contains("private"));
        }
        assert!(Stremio::new(Zeroizing::new("bad\nprivate-key".into())).is_err());
        assert!(Stremio::new(Zeroizing::new(r#"{"authKey":"synthetic-account-key-12345","user":{"email":"ignored@example.invalid"}}"#.into())).is_ok());
    }
    #[test] fn configured_addon_transports_stay_private_and_invalid_schemes_are_filtered() {
        let raw=br#"{"result":{"addons":[{"transportUrl":"https://addon.invalid/private-token/manifest.json","manifest":{"resources":[{"name":"stream","types":["movie"]}],"catalogs":[]}},{"transportUrl":"http://insecure.invalid/manifest.json","manifest":{}},{"transportUrl":"https://secret@addon.invalid/manifest.json","manifest":{}}]}}"#;
        let addons=parse_addons(raw).unwrap();assert_eq!(addons.len(),1);assert!(addons[0].has_streams());assert_eq!(addons[0].catalog_count(),0);
        assert!(addons[0].transport.path().contains("private-token"));
        assert!(parse_addons(br#"{"error":{"message":"private"}}"#).err().unwrap().to_string().contains("rejected"));
    }
    #[test] fn streams_obey_configured_types_prefixes_and_require_exact_file_hints() {
        let addon=Addon {transport:reqwest::Url::parse("https://addon.invalid/manifest.json").unwrap(),
            manifest:serde_json::json!({"types":["movie","series"],"resources":[{"name":"stream","types":["movie"],"idPrefixes":["tt"]}]})};
        assert!(addon.supports_stream("movie","tt123"));assert!(!addon.supports_stream("series","tt123:1:1"));
        assert!(!addon.supports_stream("movie","other:123"));
        let hash="0123456789012345678901234567890123456789";
        let choice=StreamChoice {name:"Stream option".into(),hash:Some(hash.into()),filename:None,provider_ids:None};
        let media=super::super::MediaIdentity {torrent_hash:Some(hash.into()),filename:Some("movie.mp4".into()),filesize:Some(4096),..super::super::MediaIdentity::torbox(1,7)};
        assert!(choice.canonical_media(&[media]).is_err());
    }
    #[test] fn catalog_requests_preserve_configured_origin_and_encode_search_as_one_resource() {
        let transport=reqwest::Url::parse("https://addon.invalid/private-token/manifest.json").unwrap();
        let definition=serde_json::json!({"type":"movie","id":"top/new","extra":[{"name":"search","isRequired":true}]});
        assert!(catalog_url(&transport,&definition,None).is_err());
        let url=catalog_url(&transport,&definition,Some(" movie / & sequel ")).unwrap();
        assert_eq!(url.origin(),transport.origin());
        assert_eq!(url.path(),"/private-token/catalog/movie/top%2Fnew/search=movie%20%2F%20%26%20sequel.json");
        assert!(url.query().is_none());
        assert!(catalog_url(&transport,&serde_json::json!({"type":"movie","id":".."}),None).is_err());
        assert!(catalog_url(&transport,&serde_json::json!({"type":"movie","id":"top"}),Some("movie")).is_err());
    }
    #[test] fn catalog_search_counts_unicode_characters_and_bounds_the_encoded_resource() {
        let transport=reqwest::Url::parse("https://addon.invalid/config/manifest.json").unwrap();
        let definition=serde_json::json!({"type":"movie","id":"search","extra":[{"name":"search","isRequired":true}]});
        let query="映".repeat(160);
        let url=catalog_url(&transport,&definition,Some(&query)).expect("160 search characters must be accepted");
        assert_eq!(url.origin(),transport.origin());assert!(url.query().is_none());
        assert!(url.as_str().len()<2048);
        assert_eq!(percent_encoding::percent_decode_str(url.path()).decode_utf8().unwrap(),format!("/config/catalog/movie/search/search={query}.json"));
        assert!(catalog_url(&transport,&definition,Some("映")).is_err());
        assert!(catalog_url(&transport,&definition,Some(&"映".repeat(161))).is_err());
        assert!(catalog_url(&transport,&definition,Some("🎬")).is_err());
        assert!(catalog_url(&transport,&definition,Some("🎬映")).is_ok());
    }
    #[test] fn catalogs_expose_only_title_metadata_and_deduplicate_per_kind() {
        let body=br#"{"metas":[{"id":"tt1","type":"movie","name":"Movie","poster":"https://private.invalid"},{"id":"tt1","type":"movie","name":"duplicate"},{"id":"tt1","type":"series","name":"Series"},{"id":"private-account-key","type":"movie","name":"bad"}]}"#;
        let result=parse_catalog(body,"private-account-key").unwrap();assert_eq!(result.len(),2);
        let serialized=serde_json::to_string(&result).unwrap();assert!(!serialized.contains("poster") && !serialized.contains("private"));
    }
    #[test] fn episodes_use_explicit_ids_and_do_not_guess_indices_or_export_provider_fields() {
        let body=br#"{"meta":{"id":"tt-series","type":"series","videos":[{"id":"explicit-second","title":"Second","season":1,"episode":2,"stream":"https://private.invalid"},{"id":"explicit-first","title":"First","season":1,"episode":1},{"id":"explicit-first","title":"Duplicate","season":1,"episode":1},{"id":"bad","title":"Missing numbers"},{"id":"private-key","title":"Bad","season":1,"episode":3}]}}"#;
        let result=parse_episodes(body,"tt-series","private-key").unwrap();assert_eq!(result.len(),2);
        assert_eq!(result[0].id,"explicit-first");assert_eq!(result[1].id,"explicit-second");
        assert!(result[0].name.starts_with("S1 E1"));
        let serialized=serde_json::to_string(&result).unwrap();assert!(!serialized.contains("private") && !serialized.contains("stream"));
        assert!(parse_episodes(body,"different-series","private-key").is_err());
    }
    #[test] fn stream_descriptors_cannot_guess_file_indices_or_publish_addon_urls() {
        let hash="0123456789012345678901234567890123456789";
        let raw=serde_json::json!({"streams":[{"name":"Option","infoHash":hash,"fileIdx":99,"behaviorHints":{"filename":"movie.mp4"},"url":"https://addon.invalid/private-token"}]});
        let streams=parse_streams(&serde_json::to_vec(&raw).unwrap(),"private-account-key").unwrap();
        let media=super::super::MediaIdentity {torrent_hash:Some(hash.into()),filename:Some("movie.mp4".into()),filesize:Some(4096),..super::super::MediaIdentity::torbox(1,7)};
        assert_eq!(streams[0].canonical_media(&[media.clone()]).unwrap().file_id,7);
        assert!(streams[0].canonical_media(&[media.clone(),media.clone()]).is_err());
        let mut other=media.clone();other.filename=Some("other.mp4".into());assert!(streams[0].canonical_media(&[other]).is_err());
        let raw=br#"{"streams":[{"name":"https://private.invalid","url":"https://api.torbox.app/v1/api/torrents/requestdl?token=PRIVATE_ACCOUNT&torrent_id=1&file_id=7"},{"url":"https://api.torbox.app/v1/api/torrents/requestdl?torrent_id=1&torrent_id=2&file_id=7"}]}"#;
        let streams=parse_streams(raw,"private-account-key").unwrap();assert_eq!(streams[0].name(),"Stream option");
        assert_eq!(streams[0].canonical_media(&[media.clone()]).unwrap(),media);
        assert!(streams[1].canonical_media(&[media]).is_err());
    }
    #[test] fn torrentio_route_uses_hash_and_filename_without_following_or_using_its_account_key() {
        let hash="0123456789012345678901234567890123456789";
        let raw=format!("https://torrentio.strem.fun/resolve/torbox/PRIVATE_TOKEN/{hash}/movie%20file.mp4/99/display.mp4");
        let reference=torrentio_reference(&raw).unwrap();assert_eq!(reference,(hash.into(),"movie file.mp4".into()));
        for raw in [raw.replace("torrentio.strem.fun","torrentio.strem.fun.evil.invalid"),raw.replace("/torbox/","/realdebrid/"),
            format!("{raw}?token=PRIVATE_TOKEN"),raw.replace(hash,"not-a-hash")] {assert!(torrentio_reference(&raw).is_none());}
        let body=serde_json::to_vec(&serde_json::json!({"streams":[{"url":raw}]})).unwrap();
        let choices=parse_streams(&body,"different-stremio-key").unwrap();
        let media=super::super::MediaIdentity {torrent_hash:Some(hash.into()),filename:Some("folder/movie file.mp4".into()),filesize:Some(4096),..super::super::MediaIdentity::torbox(2,7)};
        assert_eq!(choices[0].canonical_media(&[media.clone()]).unwrap().file_id,7);
        let mut duplicate=media.clone();duplicate.file_id=8;duplicate.filename=Some("other/movie file.mp4".into());
        assert!(choices[0].canonical_media(&[media,duplicate]).is_err());
        let raw=format!("https://torrentio.strem.fun/resolve/torbox/PRIVATE_TOKEN/{hash}/episode.mp4/99/display.mp4");
        assert_eq!(StreamChoice::from_copied_link(Zeroizing::new(raw)).unwrap().filename.as_deref(),Some("episode.mp4"));
        for raw in ["http://127.0.0.1:11470/private-url","https://unrelated.invalid/private-url","https://torrentio.strem.fun/resolve/torbox/PRIVATE_TOKEN/bad/episode.mp4/99/display.mp4"] {
            let error=StreamChoice::from_copied_link(Zeroizing::new(raw.into())).err().unwrap().to_string();
            assert!(!error.contains("PRIVATE_TOKEN") && !error.contains("private-url"));
        }
    }
}

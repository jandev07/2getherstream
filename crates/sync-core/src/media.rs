//! File identity is independent of CDN locations and temporary credentials.
use serde::{Deserialize,Serialize};

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize,Default)]
#[serde(rename_all="snake_case")]
pub enum MediaProvider { #[default] Fixture, Torbox }

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaIdentity {
    #[serde(default)] pub provider:MediaProvider,
    pub torrent_id:u64,
    pub file_id:u64,
    #[serde(default,skip_serializing_if="Option::is_none")] pub torrent_hash:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub filename:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub filesize:Option<u64>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub duration_ms:Option<u64>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub imdb_id:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub season:Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub episode:Option<u32>,
}
impl MediaIdentity {
    pub fn fixture() -> Self {
        Self {provider:MediaProvider::Fixture,torrent_id:0,file_id:0,torrent_hash:None,filename:None,
            filesize:None,duration_ms:None,imdb_id:None,season:None,episode:None}
    }
    pub fn torbox(torrent_id:u64,file_id:u64) -> Self {
        Self {provider:MediaProvider::Torbox,torrent_id,file_id,..Self::fixture()}
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        match self.provider {
            MediaProvider::Fixture if self.torrent_id!=0 || self.file_id!=0 => return Err("invalid fixture identity"),
            MediaProvider::Torbox if self.torrent_id==0 => return Err("invalid torrent identity"),
            _=>{}
        }
        if self.filename.as_ref().is_some_and(|v|v.len()>4096 || v.contains('\0')) {return Err("invalid media filename");}
        if self.torrent_hash.as_ref().is_some_and(|v| ![40,64].contains(&v.len()) || !v.bytes().all(|c|c.is_ascii_hexdigit())) {return Err("invalid torrent hash");}
        if self.imdb_id.as_ref().is_some_and(|v|!v.starts_with("tt") || v.len()>32 || v.len()<3 || !v[2..].bytes().all(|c|c.is_ascii_digit())) {return Err("invalid IMDb identity");}
        if self.duration_ms==Some(0) || self.filesize==Some(0) {return Err("invalid media size or duration");}
        Ok(())
    }
}

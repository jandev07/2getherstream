use together_sync_core::{MediaIdentity,MediaProvider,Session};

#[test] fn metadata_round_trips_without_cdn_identity() {
    let mut identity=MediaIdentity::torbox(42,0);
    identity.torrent_hash=Some("a".repeat(40));identity.filename=Some("episode.mkv".into());
    identity.filesize=Some(12345);identity.duration_ms=Some(7200000);
    identity.imdb_id=Some("tt1234567".into());identity.season=Some(1);identity.episode=Some(2);
    identity.validate().unwrap();
    let encoded=serde_json::to_string(&identity).unwrap();
    assert_eq!(serde_json::from_str::<MediaIdentity>(&encoded).unwrap(),identity);
    assert!(!encoded.contains("url"));assert!(!encoded.contains("https://"));
    assert!(serde_json::from_str::<MediaIdentity>(r#"{"provider":"torbox","torrent_id":42,"file_id":0,"url":"https://cdn.test/private"}"#).is_err());
}
#[test] fn invalid_media_cannot_mutate_authoritative_state() {
    let mut room=Session::new("test".into());
    let invalid=MediaIdentity {provider:MediaProvider::Fixture,..MediaIdentity::torbox(42,0)};
    assert!(room.load(invalid,0).is_err());
    assert!(room.state.media.is_none());assert_eq!(room.state.generation,0);assert_eq!(room.state.revision,0);
    let mut invalid=MediaIdentity::torbox(42,0);invalid.torrent_hash=Some("https://cdn.test/private".into());
    assert!(room.load(invalid,0).is_err());
    let legacy:MediaIdentity=serde_json::from_str(r#"{"torrent_id":0,"file_id":0}"#).unwrap();
    assert_eq!(legacy,MediaIdentity::fixture());
}

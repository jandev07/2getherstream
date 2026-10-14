//! Private per-device media handoff. Never part of a snapshot or diagnostic event.
use std::sync::{Arc, Mutex, Weak};
use together_source_resolver::DeviceStreams;
use together_sync_core::{MediaIdentity, Snapshot};
use zeroize::Zeroizing;

#[derive(Clone, PartialEq, Eq)]
struct Binding { epoch: String, generation: u64, media: Option<MediaIdentity> }
impl From<&Snapshot> for Binding {
    fn from(state: &Snapshot) -> Self {
        Self { epoch: state.epoch.clone(), generation: state.generation, media: state.media.clone() }
    }
}
pub(crate) struct Sources { identity: MediaIdentity, pc: Zeroizing<String>, phone: Zeroizing<String> }
impl Sources {
    pub(crate) fn validated(streams: DeviceStreams) -> Result<Self, &'static str> {
        if streams.pc.identity() != streams.phone.identity() { return Err("media identity mismatch"); }
        Ok(Self { identity: streams.pc.identity().clone(),
            pc: Zeroizing::new(streams.pc.player_url().to_owned()),
            phone: Zeroizing::new(streams.phone.player_url().to_owned()) })
    }
    pub(crate) fn identity(&self) -> &MediaIdentity { &self.identity }
}
pub(crate) struct Slot { binding: Option<Binding>, sources: Option<Sources>, closed: bool, source_revision:u64 }
impl Slot {
    pub(crate) fn new() -> Self { Self { binding: None, sources: None, closed: false, source_revision:0 } }
    pub(crate) fn clear(&mut self) { self.sources = None; }
    pub(crate) fn identity(&self)->Option<&MediaIdentity> {self.sources.as_ref().map(Sources::identity)}
    pub(crate) fn close(&mut self) { self.clear(); self.closed = true; }
    pub(crate) fn install(&mut self, sources: Sources) -> Result<(), &'static str> {
        if self.closed {return Err("media unavailable");}
        self.source_revision=self.source_revision.checked_add(1).ok_or("media revision exhausted")?;
        self.sources = Some(sources);Ok(())
    }
    pub(crate) fn observe(&mut self, state: &Snapshot) {
        if self.sources.as_ref().is_some_and(|sources| state.media.as_ref() != Some(&sources.identity)) { self.clear(); }
        self.binding = Some(state.into());
    }
}
#[derive(Clone, Copy)]
pub enum Device { Pc, Phone }
// No Debug, Serialize or URL getter. Queuing a grant does not authorize a load.
pub struct Grant { slot: Weak<Mutex<Slot>>, binding: Binding, device: Device, source_revision:u64 }
impl Grant {
    pub fn source_revision(&self)->u64 {self.source_revision}
    /// Check again on the native owner thread, at the final synchronous handoff.
    /// Keep the callback short. Do not retain its URL, await or reenter the Host.
    pub fn deliver<T>(self, handoff: impl FnOnce(&str) -> T) -> Result<T, &'static str> {
        let slot = self.slot.upgrade().ok_or("media unavailable")?;
        let guard = slot.lock().map_err(|_| "media unavailable")?;
        if guard.closed || guard.binding.as_ref() != Some(&self.binding) || guard.source_revision!=self.source_revision { return Err("stale media handoff"); }
        let sources = guard.sources.as_ref().filter(|sources| self.binding.media.as_ref() == Some(&sources.identity))
            .ok_or("media unavailable")?;
        let url = match self.device { Device::Pc => &sources.pc, Device::Phone => &sources.phone };
        Ok(handoff(url.as_str()))
    }
}
pub(crate) fn grant(slot: &Arc<Mutex<Slot>>, state: &Snapshot, device: Device) -> Option<Grant> {
    let guard = slot.lock().ok()?;
    let binding = Binding::from(state);
    if guard.closed || guard.binding.as_ref() != Some(&binding) ||
        !guard.sources.as_ref().is_some_and(|sources| state.media.as_ref() == Some(&sources.identity)) { return None; }
    Some(Grant { slot: Arc::downgrade(slot), binding, device, source_revision:guard.source_revision })
}
/// Generation checkpoint for an in-process native snapshot queue. This is a
/// short observation, not a lock held across blocking native property calls.
pub(crate) fn current_generation(slot:&Arc<Mutex<Slot>>,state:&Snapshot)->bool {
    let Ok(guard)=slot.lock() else {return false};
    !guard.closed && guard.binding.as_ref().is_some_and(|binding|
        binding.epoch==state.epoch && binding.generation==state.generation && binding.media==state.media)
}

#[cfg(test)]
mod tests {
    use super::*;
    use together_sync_core::Session;
    fn replacement(identity:MediaIdentity)->Sources {
        Sources {identity,pc:Zeroizing::new("https://cdn.invalid/NEW_PC".into()),
            phone:Zeroizing::new("https://cdn.invalid/NEW_PHONE".into())}
    }
    async fn refresh_host()->Arc<crate::Host> {
        let host=crate::Host::new("pc-refresh-synthetic-00000000000000".into(),"phone-refresh-synthetic-00000000000".into()).unwrap();
        host.attach_local_pc().await.unwrap();
        host.local_pc(None,crate::Action::Load {media:MediaIdentity::torbox(1,2)}).await.unwrap();
        let state=host.snapshot().await;
        host.media.lock().unwrap().install(replacement(state.media.unwrap())).unwrap();host
    }
    #[tokio::test] async fn native_generation_checkpoint_rejects_replacement_foreign_identity_and_shutdown() {
        let host=refresh_host().await;let current=host.snapshot().await;
        assert!(host.is_current_media_generation(&current));
        let mut foreign=current.clone();foreign.epoch="foreign-authority".into();
        assert!(!host.is_current_media_generation(&foreign));
        foreign=current.clone();foreign.media=Some(MediaIdentity::torbox(3,4));
        assert!(!host.is_current_media_generation(&foreign));
        host.local_pc(None,crate::Action::Pause).await.unwrap();
        assert!(!host.is_current_media_generation(&current));
        let latest=host.snapshot().await;assert!(host.is_current_media_generation(&latest));
        host.request_shutdown();assert!(!host.is_current_media_generation(&latest));
    }
    #[tokio::test] async fn full_snapshot_queue_cannot_authorize_a_superseded_device_capability() {
        for device in [Device::Pc,Device::Phone] {
            for change in ["pause","seek","reload","shutdown"] {
                let host=refresh_host().await;let before=host.snapshot().await;
                let old=host.media_grant(&before,device).unwrap();
                let (sender,receiver)=std::sync::mpsc::sync_channel(1);
                assert!(sender.try_send((before,Some(old))).is_ok());
                match change {
                    "pause"=>host.local_pc(None,crate::Action::Pause).await.unwrap(),
                    "seek"=>host.local_pc(None,crate::Action::Seek {position_ms:2000}).await.unwrap(),
                    "reload"=>host.local_pc(None,crate::Action::Load {media:MediaIdentity::torbox(3,4)}).await.unwrap(),
                    "shutdown"=>host.request_shutdown(),_=>unreachable!(),
                }
                let latest=host.snapshot().await;
                assert!(matches!(sender.try_send((latest.clone(),host.media_grant(&latest,device))),Err(std::sync::mpsc::TrySendError::Full(_))));
                let (queued,old)=receiver.try_recv().unwrap();
                assert!(!host.is_current_media_generation(&queued));
                let called=std::sync::atomic::AtomicBool::new(false);
                assert!(old.unwrap().deliver(|_|called.store(true,std::sync::atomic::Ordering::SeqCst)).is_err());
                assert!(!called.load(std::sync::atomic::Ordering::SeqCst));
                assert!(sender.try_send((latest.clone(),host.media_grant(&latest,device))).is_ok());
                let (queued,current)=receiver.try_recv().unwrap();
                assert_eq!(host.is_current_media_generation(&queued),change!="shutdown");
                if change=="pause" || change=="seek" {
                    assert!(current.unwrap().deliver(|url|url.ends_with(match device {Device::Pc=>"NEW_PC",Device::Phone=>"NEW_PHONE"})).unwrap());
                } else {assert!(current.is_none(),"reload or shutdown retained an obsolete capability");}
            }
        }
    }
    #[tokio::test] async fn refreshing_links_preserves_position_intent_and_revokes_both_old_grants() {
        for playing in [false,true] {
            let host=refresh_host().await;
            host.local_pc(None,if playing {crate::Action::Play}else{crate::Action::Pause}).await.unwrap();
            host.local_pc(None,crate::Action::Seek {position_ms:42_000}).await.unwrap();
            let before=host.snapshot().await;
            let pc=host.media_grant(&before,Device::Pc).unwrap();
            let phone=host.media_grant(&before,Device::Phone).unwrap();
            let revision=pc.source_revision();
            let mut updates=host.updates.subscribe();
            host.publish_sources(&before.epoch,before.generation,replacement(before.media.clone().unwrap()),true).await.unwrap();
            let after=host.snapshot().await;
            assert_eq!(after.generation,before.generation+1);
            assert_eq!(after.timeline.position_ms,42_000);
            assert_eq!(after.desired_playing,playing);
            assert_eq!(after.prepare.as_ref().unwrap().position_ms,42_000);
            assert!(pc.deliver(|_|panic!("old PC capability")).is_err());
            assert!(phone.deliver(|_|panic!("old phone capability")).is_err());
            let fresh=host.media_grant(&updates.recv().await.unwrap(),Device::Pc).unwrap();
            assert_eq!(fresh.source_revision(),revision+1);
            assert!(fresh.deliver(|url|url.ends_with("NEW_PC")).unwrap());
        }
    }
    #[tokio::test] async fn rejected_refresh_preserves_sources_and_authoritative_state() {
        for change in ["pause","seek","replace","epoch","identity","ended","shutdown"] {
            let host=refresh_host().await;
            let before=host.snapshot().await;
            let mut epoch=before.epoch.clone();
            let mut sources=replacement(before.media.clone().unwrap());
            match change {
                "pause"=>host.local_pc(None,crate::Action::Pause).await.unwrap(),
                "seek"=>host.local_pc(None,crate::Action::Seek {position_ms:13_000}).await.unwrap(),
                "replace"=>host.local_pc(None,crate::Action::Load {media:MediaIdentity::torbox(3,4)}).await.unwrap(),
                "epoch"=>epoch="foreign-epoch".into(),
                "identity"=>sources=replacement(MediaIdentity::torbox(1,3)),
                "ended"=>host.session.lock().await.state.ended=true,
                "shutdown"=>host.request_shutdown(),_=>unreachable!(),
            }
            let current=host.snapshot().await;
            let grant=host.media_grant(&current,Device::Phone);
            let serialized=serde_json::to_value(&current).unwrap();
            assert!(host.publish_sources(&epoch,before.generation,sources,true).await.is_err(),"{change}");
            assert_eq!(serde_json::to_value(host.snapshot().await).unwrap(),serialized,"{change}");
            if let Some(grant)=grant {assert!(grant.deliver(|url|url.ends_with("NEW_PHONE")).unwrap());}
        }
    }
    #[tokio::test]
    async fn completed_replacement_load_preserves_newer_commands_and_existing_capabilities() {
        for change in ["pause","seek","shutdown"] {
            let host=refresh_host().await;
            let before=host.snapshot().await;
            let old_pc=host.media_grant(&before,Device::Pc).unwrap();
            let old_phone=host.media_grant(&before,Device::Phone).unwrap();
            let source_revision=old_pc.source_revision();
            // A fully prepared different movie waits at the final publication
            // boundary; no provider work remains when authority changes.
            let completed=replacement(MediaIdentity::torbox(3,4));
            match change {
                "pause"=>host.local_pc(None,crate::Action::Pause).await.unwrap(),
                "seek"=>host.local_pc(None,crate::Action::Seek {position_ms:13_000}).await.unwrap(),
                "shutdown"=>host.request_shutdown(),
                _=>unreachable!(),
            }
            let authoritative=host.snapshot().await;
            assert!(old_pc.deliver(|_|panic!("obsolete PC handoff")).is_err());
            assert!(old_phone.deliver(|_|panic!("obsolete phone handoff")).is_err());
            let result=host.publish_sources(&before.epoch,before.generation,completed,false).await;
            assert_eq!(result,Err(if change=="shutdown" {"server stopping"}else {"stale media resolution"}));
            assert_eq!(serde_json::to_value(host.snapshot().await).unwrap(),serde_json::to_value(&authoritative).unwrap());
            for device in [Device::Pc,Device::Phone] {
                let grant=host.media_grant(&authoritative,device);
                if change=="shutdown" {assert!(grant.is_none());}
                else {
                    let grant=grant.expect("rejected replacement lost existing capability");
                    assert_eq!(grant.source_revision(),source_revision);
                    assert!(grant.deliver(|url|url.ends_with(match device {
                        Device::Pc=>"NEW_PC",Device::Phone=>"NEW_PHONE"
                    })).unwrap());
                }
            }
        }
    }
    #[tokio::test] async fn link_refresh_does_not_claim_a_faulted_player_has_recovered() {
        let host=refresh_host().await;
        host.local_pc(None,crate::Action::PlayerFailed {code:together_sync_core::PlayerFault::LoadFailed}).await.unwrap();
        let before=host.snapshot().await;
        host.publish_sources(&before.epoch,before.generation,replacement(before.media.unwrap()),true).await.unwrap();
        let after=host.snapshot().await;
        assert!(after.participants["pc"].player_fault.is_some());
        assert!(!after.timeline.playing);
    }
    fn setup() -> (Arc<Mutex<Slot>>, Session) {
        let mut session = Session::new("synthetic-epoch".into());
        session.load(MediaIdentity::torbox(1, 2), 0).unwrap();
        let slot = Arc::new(Mutex::new(Slot::new()));
        let mut guard = slot.lock().unwrap();
        guard.install(Sources { identity: session.state.media.clone().unwrap(),
            pc: Zeroizing::new("https://cdn.invalid/pc?cap=PRIVATE_PC".into()),
            phone: Zeroizing::new("https://cdn.invalid/phone?cap=PRIVATE_PHONE".into()) }).unwrap();
        guard.observe(&session.state); drop(guard); (slot, session)
    }
    #[test] fn roles_receive_only_their_own_capability_and_snapshots_have_none() {
        let (slot, session) = setup();
        assert!(grant(&slot, &session.state, Device::Pc).unwrap().deliver(|url| url.contains("PRIVATE_PC") && !url.contains("PRIVATE_PHONE")).unwrap());
        assert!(grant(&slot, &session.state, Device::Phone).unwrap().deliver(|url| url.contains("PRIVATE_PHONE") && !url.contains("PRIVATE_PC")).unwrap());
        let public = serde_json::to_string(&session.state).unwrap();
        assert!(!public.contains("PRIVATE_") && !public.contains("cdn.invalid"));
    }
    #[test] fn queued_grants_are_revoked_on_seek_same_file_reload_and_shutdown() {
        for event in [0, 1, 2] {
            let (slot, mut session) = setup();
            let old = grant(&slot, &session.state, Device::Phone).unwrap();
            let mut guard = slot.lock().unwrap();
            match event {
                0 => { session.seek(500, session.state.generation, 10).unwrap(); guard.observe(&session.state); },
                1 => { guard.clear(); session.load(MediaIdentity::torbox(1, 2), 10).unwrap(); guard.observe(&session.state); },
                _ => guard.close(),
            }
            drop(guard);
            let called = std::sync::atomic::AtomicBool::new(false);
            assert!(old.deliver(|_| called.store(true, std::sync::atomic::Ordering::SeqCst)).is_err());
            assert!(!called.load(std::sync::atomic::Ordering::SeqCst));
        }
    }
    #[test] fn replacement_and_wrong_epoch_cannot_authorize_an_old_source() {
        let (slot, mut session) = setup();
        let old = grant(&slot, &session.state, Device::Pc).unwrap();
        let mut foreign = session.state.clone(); foreign.epoch = "other".into();
        assert!(grant(&slot, &foreign, Device::Pc).is_none());
        session.load(MediaIdentity::torbox(3, 4), 10).unwrap();
        slot.lock().unwrap().observe(&session.state);
        assert!(old.deliver(|_| panic!("must not deliver")).is_err());
        assert!(grant(&slot, &session.state, Device::Pc).is_none());
    }
    #[tokio::test] async fn private_http_metadata_rejects_pc_and_stale_requests_without_leaking_into_journal() {
        use axum::{extract::{State,Query},http::{HeaderMap,HeaderValue,StatusCode}};
        let host=crate::Host::new("pc-synthetic-private-media-000000000".into(),"phone-synthetic-private-media-000000".into()).unwrap();
        let directory=std::env::temp_dir().join(format!("together-private-media-{}",uuid::Uuid::new_v4()));
        host.enable_diagnostics(directory.clone()).unwrap();
        host.attach_local_pc().await.unwrap();
        host.local_pc(None,crate::Action::Load {media:MediaIdentity::torbox(1,2)}).await.unwrap();
        let state=host.snapshot().await;
        host.media.lock().unwrap().install(Sources {identity:state.media.clone().unwrap(),
            pc:Zeroizing::new("https://cdn.invalid/PRIVATE_PC".into()),phone:Zeroizing::new("https://cdn.invalid/PRIVATE_PHONE".into())}).unwrap();
        let queued=host.media_grant(&state,Device::Pc).unwrap();
        assert!(host.local_pc(None,crate::Action::Load {media:MediaIdentity::torbox(0,2)}).await.is_err());
        assert!(queued.deliver(|url|url.contains("PRIVATE_PC")).unwrap(),"invalid load must preserve the active handoff");
        let headers=|token:&str| {let mut headers=HeaderMap::new();headers.insert("authorization",HeaderValue::from_str(&format!("Bearer {token}")).unwrap());headers};
        let request=|epoch:String,generation|Query(crate::MediaRequest {epoch,generation});
        for token in ["wrong","pc-synthetic-private-media-000000000"] {
            let response=crate::phone_media(State(host.clone()),headers(token),request(state.epoch.clone(),state.generation)).await;
            assert_eq!(response.status(),StatusCode::UNAUTHORIZED);
        }
        let stale=crate::phone_media(State(host.clone()),headers("phone-synthetic-private-media-000000"),request(state.epoch.clone(),state.generation+1)).await;
        assert_eq!(stale.status(),StatusCode::CONFLICT);
        let accepted=crate::phone_media(State(host.clone()),headers("phone-synthetic-private-media-000000"),request(state.epoch.clone(),state.generation)).await;
        assert_eq!(accepted.status(),StatusCode::OK);assert_eq!(accepted.headers()["cache-control"],"no-store");
        let bytes=axum::body::to_bytes(accepted.into_body(),16384).await.unwrap();
        let text=std::str::from_utf8(&bytes).unwrap();assert!(text.contains("PRIVATE_PHONE"));assert!(!text.contains("PRIVATE_PC"));
        let journal=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
        assert!(!journal.contains("PRIVATE_") && !journal.contains("cdn.invalid"));
        host.request_shutdown();
        let stopped=crate::phone_media(State(host.clone()),headers("phone-synthetic-private-media-000000"),request(state.epoch,state.generation)).await;
        assert_eq!(stopped.status(),StatusCode::SERVICE_UNAVAILABLE);
        drop(host);for _ in 0..100 {if std::fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
        assert!(!directory.exists());
    }
}

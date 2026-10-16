//! PC-only credential and source-selection owner. UI receives identities, never URLs.
use std::{path::PathBuf, sync::Arc};
use together_source_resolver::{MediaIdentity, TorBox};
use together_session_server::{Host,diagnostics::{SourceStage,SourceOutcome}};
// One budget spans verification, both role links and direct ranges. Individual
// HTTP limits otherwise accumulate; publication and provider creates stay outside.
const READ_DEADLINE:std::time::Duration=std::time::Duration::from_secs(60);
const READ_TIMEOUT:&str="TorBox lookup took too long. Try again.";
async fn read_before<T>(deadline:tokio::time::Instant,work:impl std::future::Future<Output=Result<T,&'static str>>)->Result<T,&'static str> {
    // timeout_at can poll an immediately-ready future at an expired deadline.
    // Do not start a later provider stage after the shared budget is exhausted.
    if tokio::time::Instant::now()>=deadline {return Err(READ_TIMEOUT);}
    tokio::time::timeout_at(deadline,work).await.map_err(|_|READ_TIMEOUT)?
}
// Reuse Host authority/shutdown notifications; no new owner or polling loop.
async fn read_for_authority<T>(host:&Host,before:&together_sync_core::Snapshot,work:impl std::future::Future<Output=Result<T,&'static str>>)->Result<T,&'static str> {
    tokio::select! {biased;
        reason=host.media_authority_changed(&before.epoch,before.generation)=>Err(reason),
        result=work=>result,
    }
}
async fn read_stage<T>(deadline:tokio::time::Instant,stage:SourceStage,
    record:impl Fn(SourceStage,SourceOutcome),work:impl std::future::Future<Output=Result<T,&'static str>>)->Result<T,&'static str> {
    record(stage,SourceOutcome::Started);
    let result=read_before(deadline,work).await;
    record(stage,match &result {
        Ok(_)=>SourceOutcome::Passed,
        Err(error) if *error=="stale media resolution" || *error=="TorBox selection replaced"=>SourceOutcome::Replaced,
        Err(_)=>SourceOutcome::Failed,
    });
    result
}
struct State { client:Option<Arc<TorBox>>, videos:Vec<MediaIdentity>, selection:u64 }
pub struct Controller { vault:PathBuf, state:tokio::sync::Mutex<State> }
// Private transaction: direct URLs never enter IPC or diagnostics.
pub(super) struct ResolvedSelection {before:together_sync_core::Snapshot,streams:together_source_resolver::DeviceStreams,
    load:bool,selection:u64,began:std::time::Instant}
pub fn private_alpha_account_links()->bool {cfg!(feature="private-alpha-account-links")}
impl Controller {
    pub fn new(vault:PathBuf)->Self {Self {vault,state:tokio::sync::Mutex::new(State {client:None,videos:Vec::new(),selection:0})}}
    async fn client(&self)->Result<Arc<TorBox>, &'static str> {
        let mut state=self.state.lock().await;
        if let Some(client)=&state.client {return Ok(client.clone());}
        let vault=self.vault.clone();
        let client=tokio::task::spawn_blocking(move || {
            let key=together_source_resolver::secret::read_key(&vault)?;
            #[cfg(feature="private-alpha-account-links")]
            {TorBox::new_private_alpha(key)}
            #[cfg(not(feature="private-alpha-account-links"))]
            {TorBox::new(key)}
        })
            .await.map_err(|_|"TorBox credential unavailable")?.map_err(|_|"TorBox credential unavailable")?;
        let client=Arc::new(client);state.client=Some(client.clone());Ok(client)
    }
    pub async fn videos(&self)->Result<Vec<MediaIdentity>, &'static str> {
        let deadline=tokio::time::Instant::now()+READ_DEADLINE;
        let videos=read_before(deadline,async {
            self.client().await?.available_videos().await.map_err(|_|"Could not retrieve available TorBox videos")
        }).await?;
        self.state.lock().await.videos=videos.clone();Ok(videos)
    }
    pub async fn cached_selections(&self,choices:&[together_source_resolver::stremio::StreamChoice])->Result<Vec<(usize,together_source_resolver::preparation::CachedSelection)>,&'static str> {
        self.client().await?.cached_selections(choices).await.map_err(|_|"Could not verify cached TorBox source files")
    }
    pub async fn prepare_cached(&self,selection:&together_source_resolver::preparation::CachedSelection)->Result<together_source_resolver::preparation::Prepared,&'static str> {
        self.client().await?.prepare_cached(selection).await.map_err(|error| {
            if together_source_resolver::preparation::is_preflight_timeout(&error) {
                "TorBox preparation checks took too long. Try again."
            } else {
                "TorBox could not prepare this file. Check your account limits or pending items; this app will not delete items or automatically retry a creation."
            }
        })
    }
    pub async fn select(&self,host:&Host,torrent_id:u64,file_id:u64)->Result<(), &'static str> {
        self.resolve(host,Some((torrent_id,file_id))).await
    }
    pub async fn copied_source(&self,host:&Host,mut raw:String)->Result<(), &'static str> {
        let deadline=tokio::time::Instant::now()+READ_DEADLINE;
        let before=host.snapshot().await;
        let choice=together_source_resolver::stremio::StreamChoice::from_copied_link(zeroize::Zeroizing::new(std::mem::take(&mut raw)))
            .map_err(|_|"This link is not a supported TorBox source reference. Use the Stremio catalog or your ready TorBox videos instead.")?;
        read_before(deadline,read_for_authority(host,&before,self.client())).await?;
        let (client,selection)=read_before(deadline,read_for_authority(host,&before,self.reserve())).await?;
        let available=read_before(deadline,read_for_authority(host,&before,async {
            client.available_videos_fresh().await.map_err(|_|"Could not refresh ready TorBox videos")
        })).await?;
        let media=choice.canonical_media(&available).map_err(|_|"The copied source is not ready as one exact TorBox file. Let Stremio finish preparing it, then try again.")?;
        self.fetch_publish(host,before,media,true,client,selection,deadline).await
    }
    pub async fn refresh(&self,host:&Host)->Result<(), &'static str> {
        self.resolve(host,None).await
    }
    async fn resolve(&self,host:&Host,selection_ids:Option<(u64,u64)>)->Result<(), &'static str> {
        let deadline=tokio::time::Instant::now()+READ_DEADLINE;
        let before=host.snapshot().await;
        let media={
            let state=self.state.lock().await;
            let media=if let Some((torrent_id,file_id))=selection_ids {
                state.videos.iter().find(|media|media.torrent_id==torrent_id && media.file_id==file_id)
                    .cloned().ok_or("Select an available TorBox video first")?
            } else {
                let media=before.media.clone().filter(|media|media.provider==together_sync_core::MediaProvider::Torbox)
                    .ok_or("Select a TorBox movie before refreshing playback links")?;
                if before.ended {return Err("Movie ended; use Replay");}media
            };
            media
        };
        self.resolve_media(host,before,media,selection_ids.is_some(),deadline).await
    }
    /// Stremio supplies a canonical file from our own TorBox inventory, never an addon URL.
    /// Keep the snapshot from the beginning of the user's selection across all network work.
    pub(super) async fn resolve_canonical(&self,host:&Host,media:MediaIdentity,before:together_sync_core::Snapshot)->Result<ResolvedSelection, &'static str> {
        let deadline=tokio::time::Instant::now()+READ_DEADLINE;
        let (client,selection)=read_before(deadline,read_for_authority(host,&before,self.reserve())).await?;
        self.fetch_resolved(host,before,media,true,client,selection,deadline).await
    }
    async fn resolve_media(&self,host:&Host,before:together_sync_core::Snapshot,media:MediaIdentity,load:bool,deadline:tokio::time::Instant)->Result<(), &'static str> {
        let (client,selection)=read_before(deadline,read_for_authority(host,&before,self.reserve())).await?;
        self.fetch_publish(host,before,media,load,client,selection,deadline).await
    }
    async fn reserve(&self)->Result<(Arc<TorBox>,u64),&'static str> {
            let mut state=self.state.lock().await;
            let client=state.client.clone().ok_or("TorBox credential unavailable")?;
            state.selection=state.selection.checked_add(1).ok_or("TorBox selection exhausted")?;
            Ok((client,state.selection))
    }
    async fn still_current(&self,host:&Host,before:&together_sync_core::Snapshot,selection:u64)->Result<(),&'static str> {
        if self.state.lock().await.selection!=selection {return Err("TorBox selection replaced");}
        let current=host.snapshot().await;
        if current.epoch!=before.epoch || current.generation!=before.generation {return Err("stale media resolution");}
        Ok(())
    }
    async fn fetch_publish(&self,host:&Host,before:together_sync_core::Snapshot,media:MediaIdentity,load:bool,client:Arc<TorBox>,selection:u64,deadline:tokio::time::Instant)->Result<(),&'static str> {
        let resolved=self.fetch_resolved(host,before,media,load,client,selection,deadline).await?;
        self.publish_resolved(host,resolved).await
    }
    async fn fetch_resolved(&self,host:&Host,before:together_sync_core::Snapshot,media:MediaIdentity,load:bool,client:Arc<TorBox>,selection:u64,deadline:tokio::time::Instant)->Result<ResolvedSelection,&'static str> {
        let began=std::time::Instant::now();
        let record=|stage,outcome|host.record_source(selection,before.generation,!load,stage,outcome,began.elapsed().as_millis().min(u64::MAX as u128) as u64);
        let streams=read_stage(deadline,SourceStage::CurrentFileAndLinks,record,read_for_authority(host,&before,async {
            self.still_current(host,&before,selection).await?;
            client.device_streams(&media,None,None).await
                .map_err(|_|"TorBox could not provide playback links accepted by this build's delivery policy")
        })).await?;
        // Both probes share the remaining budget. Never return half a pair or
        // publish links after either role fails validation or the deadline expires.
        read_stage(deadline,SourceStage::DirectRanges,record,read_for_authority(host,&before,async {
            self.still_current(host,&before,selection).await?;
            tokio::try_join!(client.probe(&streams.pc),client.probe(&streams.phone))
                .map_err(|_|"TorBox device playback links failed direct media validation")
        })).await?;
        Ok(ResolvedSelection {before,streams,load,selection,began})
    }
    pub(super) fn discard_resolved(&self,host:&Host,resolved:ResolvedSelection) {
        use together_session_server::diagnostics::{SourceStage,SourceOutcome};
        // Fixed categories only. Dropping the transaction never exposes its URLs.
        host.record_source(resolved.selection,resolved.before.generation,!resolved.load,
            SourceStage::Publication,SourceOutcome::Replaced,
            resolved.began.elapsed().as_millis().min(u64::MAX as u128) as u64);
    }
    pub(super) async fn publish_resolved(&self,host:&Host,resolved:ResolvedSelection)->Result<(),&'static str> {
        use together_session_server::diagnostics::{SourceStage as Stage,SourceOutcome as Outcome};
        let ResolvedSelection {before,streams,load,selection,began}=resolved;
        let record=|stage,outcome|host.record_source(selection,before.generation,!load,stage,outcome,began.elapsed().as_millis().min(u64::MAX as u128) as u64);
        // Publication is short and atomic with replacement; provider IO is complete.
        let state=self.state.lock().await;
        record(Stage::Publication,Outcome::Started);
        if state.selection!=selection {record(Stage::Publication,Outcome::Replaced);return Err("TorBox selection replaced");}
        let result=if load {host.load_resolved_media(&before.epoch,before.generation,streams).await}
        else {host.refresh_resolved_media(&before.epoch,before.generation,streams).await};
        record(Stage::Publication,match result {Ok(())=>Outcome::Passed,Err("stale media resolution")=>Outcome::Replaced,Err(_)=>Outcome::Failed});
        result
    }
}
#[cfg(test)] mod tests {
    use super::*;
    use std::future::Future;
    #[tokio::test]
    async fn authority_replacement_drops_pending_read_without_waiting_for_deadline() {
        struct Dropped(Arc<std::sync::atomic::AtomicBool>);
        impl Drop for Dropped {fn drop(&mut self) {self.0.store(true,std::sync::atomic::Ordering::SeqCst);}}
        let directory=std::env::var_os("TOGETHER_SOURCE_AUTHORITY_PROOF_DIR").map(PathBuf::from)
            .unwrap_or_else(||std::env::temp_dir().join(format!("source-authority-{}",uuid::Uuid::new_v4())));
        for (stage,label) in [(SourceStage::CurrentFileAndLinks,"links"),(SourceStage::DirectRanges,"ranges")] {
            for change in ["pause","seek","load"] {
                let host=Host::new("synthetic-authority-read-pc-00000000".into(),"synthetic-authority-read-phone-00000".into()).unwrap();
                let case=directory.join(format!("{label}-{change}"));
                host.enable_diagnostics(case.join("journal")).unwrap();
                host.attach_local_pc().await.unwrap();
                host.local_pc(None,together_session_server::Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
                let before=host.snapshot().await;
                let dropped=Arc::new(std::sync::atomic::AtomicBool::new(false));let guard=Dropped(dropped.clone());
                let record=|stage,outcome|host.record_source(1,before.generation,false,stage,outcome,0);
                let mut read=Box::pin(read_stage(tokio::time::Instant::now()+READ_DEADLINE,stage,record,
                    read_for_authority(&host,&before,async move {let _guard=guard;std::future::pending::<Result<(),&'static str>>().await})));
                assert!(std::future::poll_fn(|cx|std::task::Poll::Ready(read.as_mut().poll(cx))).await.is_pending());
                // READY changes revision without replacing this preparation generation.
                host.local_pc(None,together_session_server::Action::Ready {position_ms:0}).await.unwrap();
                assert_eq!(host.snapshot().await.generation,before.generation);
                assert!(std::future::poll_fn(|cx|std::task::Poll::Ready(read.as_mut().poll(cx))).await.is_pending());
                assert!(!dropped.load(std::sync::atomic::Ordering::SeqCst));
                let action=match change {
                    "pause"=>together_session_server::Action::Pause,
                    "seek"=>together_session_server::Action::Seek {position_ms:2000},
                    _=>together_session_server::Action::Load {media:MediaIdentity::fixture()},
                };
                host.local_pc(None,action).await.unwrap();
                let current=host.snapshot().await;
                let result=std::future::poll_fn(|cx|std::task::Poll::Ready(read.as_mut().poll(cx))).await;
                let cancelled=matches!(result,std::task::Poll::Ready(Err("stale media resolution")));
                let promptly_dropped=dropped.load(std::sync::atomic::Ordering::SeqCst);
                drop(read);
                let unchanged=serde_json::to_value(host.snapshot().await).unwrap()==serde_json::to_value(&current).unwrap();
                let journal=host.export_diagnostics().await.unwrap();
                std::fs::write(case.join("journal.jsonl"),&journal).unwrap();
                std::fs::write(case.join("receipt.json"),serde_json::to_vec_pretty(&serde_json::json!({
                    "stage":label,"change":change,"before_generation":before.generation,
                    "current_generation":current.generation,"cancelled_on_next_poll":cancelled,
                    "pending_read_dropped":promptly_dropped,"authority_preserved":unchanged,
                    "provider_network_executed":false,"physical_gate_passed":false
                })).unwrap()).unwrap();
                host.request_shutdown();
                assert!(cancelled,"superseded provider read remained pending until its deadline");
                assert!(promptly_dropped);assert!(unchanged);
                let text=String::from_utf8(journal).unwrap();
                let rows:Vec<serde_json::Value>=text.lines().map(|line|serde_json::from_str(line).unwrap()).collect();
                let source:Vec<_>=rows.iter().filter(|row|row["record"]["event"]=="source").collect();
                assert_eq!(source.len(),2);assert_eq!(source[1]["record"]["sample"]["outcome"],"replaced");
                assert!(!text.contains("https://") && !text.contains("Bearer "));
            }
        }
    }
    #[tokio::test]
    async fn already_replaced_source_does_not_poll_work_or_reserve_a_selection() {
        let host=Host::new("synthetic-old-source-pc-000000000000".into(),"synthetic-old-source-phone-000000000".into()).unwrap();
        host.attach_local_pc().await.unwrap();
        host.local_pc(None,together_session_server::Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
        let before=host.snapshot().await;host.local_pc(None,together_session_server::Action::Pause).await.unwrap();
        let work=std::future::poll_fn(|_|->std::task::Poll<Result<(),&'static str>> {
            panic!("already superseded source work was polled")
        });
        assert_eq!(read_for_authority(&host,&before,work).await,Err("stale media resolution"));
        let controller=Controller::new(PathBuf::from("unused-superseded-source-vault"));
        assert!(matches!(controller.resolve_canonical(&host,MediaIdentity::torbox(1,2),before).await,Err("stale media resolution")));
        assert_eq!(controller.state.lock().await.selection,0);
    }
    #[tokio::test]
    async fn stopped_host_does_not_poll_source_work_or_reserve_another_selection() {
        let host=Host::new("synthetic-private-stop-pc-00000000000".into(),"synthetic-private-stop-phone-0000000".into()).unwrap();
        host.request_shutdown();
        let before=host.snapshot().await;
        let work=std::future::poll_fn(|_|->std::task::Poll<Result<(),&'static str>> {
            panic!("stopped Host polled source work")
        });
        assert_eq!(read_for_authority(&host,&before,work).await,Err("server stopping"));
        let controller=Controller::new(PathBuf::from("unused-synthetic-vault"));
        let result=controller.resolve_canonical(&host,MediaIdentity::torbox(1,2),host.snapshot().await).await;
        assert!(matches!(result,Err("server stopping")));
        assert_eq!(controller.state.lock().await.selection,0);
    }
    #[tokio::test]
    async fn host_shutdown_drops_pending_source_stage_and_records_no_publication() {
        struct Dropped(Arc<std::sync::atomic::AtomicBool>);
        impl Drop for Dropped {fn drop(&mut self) {self.0.store(true,std::sync::atomic::Ordering::SeqCst);}}
        let directory=std::env::var_os("TOGETHER_SOURCE_SHUTDOWN_PROOF_DIR").map(PathBuf::from)
            .unwrap_or_else(||std::env::temp_dir().join(format!("source-shutdown-{}",uuid::Uuid::new_v4())));
        for (index,stage) in [SourceStage::CurrentFileAndLinks,SourceStage::DirectRanges].into_iter().enumerate() {
            let host=Host::new("synthetic-private-cancel-pc-00000000".into(),"synthetic-private-cancel-phone-0000".into()).unwrap();
            host.enable_diagnostics(directory.join(format!("stage-{index}"))).unwrap();
            host.attach_local_pc().await.unwrap();
            host.local_pc(None,together_session_server::Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
            let before=host.snapshot().await;
            let dropped=Arc::new(std::sync::atomic::AtomicBool::new(false));let guard=Dropped(dropped.clone());
            let record=|stage,outcome|host.record_source(1,before.generation,false,stage,outcome,0);
            let mut read=Box::pin(read_stage(tokio::time::Instant::now()+READ_DEADLINE,stage,record,
                read_for_authority(&host,&before,async move {let _guard=guard;std::future::pending::<Result<(),&'static str>>().await})));
            assert!(std::future::poll_fn(|cx|std::task::Poll::Ready(read.as_mut().poll(cx))).await.is_pending());
            host.request_shutdown();
            assert_eq!(tokio::time::timeout(std::time::Duration::from_secs(1),read).await.unwrap(),Err("server stopping"));
            assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
            assert!(serde_json::to_value(host.snapshot().await).unwrap()==serde_json::to_value(&before).unwrap());
            let text=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
            assert!(!text.contains("https://") && !text.contains("Bearer "));
            let rows:Vec<serde_json::Value>=text.lines().map(|row|serde_json::from_str(row).unwrap()).collect();
            let source:Vec<_>=rows.iter().filter(|row|row["record"]["event"]=="source").collect();
            assert_eq!(source.len(),2);
            assert_eq!(source[0]["record"]["sample"]["outcome"],"started");
            assert_eq!(source[1]["record"]["sample"]["outcome"],"failed");
            assert!(source.iter().all(|row|row["record"]["sample"]["stage"]!="publication"));
        }
    }
    #[tokio::test]
    async fn cancelling_a_source_wait_does_not_shutdown_host_or_prevent_another_read() {
        let host=Host::new("synthetic-private-wait-pc-00000000000".into(),"synthetic-private-wait-phone-0000000".into()).unwrap();
        let before=host.snapshot().await;
        let mut read=Box::pin(read_for_authority(&host,&before,std::future::pending::<Result<(),&'static str>>()));
        assert!(std::future::poll_fn(|cx|std::task::Poll::Ready(read.as_mut().poll(cx))).await.is_pending());
        drop(read);
        assert_eq!(read_for_authority(&host,&before,async {Ok(7)}).await,Ok(7));
        host.attach_local_pc().await.unwrap();
    }
    #[tokio::test]
    async fn read_budget_preserves_results_and_never_polls_an_expired_stage() {
        let deadline=tokio::time::Instant::now()+READ_DEADLINE;
        assert_eq!(read_before(deadline,async {Ok(7)}).await,Ok(7));
        assert_eq!(read_before(deadline,async {Err::<(),_>("synthetic provider unavailable")}).await,
            Err("synthetic provider unavailable"));
        let polls=std::sync::atomic::AtomicUsize::new(0);
        let work=std::future::poll_fn(|_| {
            polls.fetch_add(1,std::sync::atomic::Ordering::SeqCst);
            std::task::Poll::Ready(Ok(9))
        });
        assert_eq!(read_before(tokio::time::Instant::now(),work).await,Err(READ_TIMEOUT));
        assert_eq!(polls.load(std::sync::atomic::Ordering::SeqCst),0);
    }
    #[tokio::test]
    async fn role_ranges_use_the_remaining_budget_instead_of_starting_a_new_one() {
        let deadline=tokio::time::Instant::now()+std::time::Duration::from_millis(100);
        let events=std::sync::Mutex::new(Vec::new());
        let record=|stage,outcome|events.lock().unwrap().push((stage,outcome));
        read_stage(deadline,SourceStage::CurrentFileAndLinks,record,async {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;Ok(())
        }).await.unwrap();
        // This would complete if the second stage received a fresh budget.
        let result=read_stage(deadline,SourceStage::DirectRanges,record,async {
            tokio::time::sleep_until(deadline+std::time::Duration::from_millis(30)).await;Ok(())
        }).await;
        assert_eq!(result,Err(READ_TIMEOUT));
        let events=events.into_inner().unwrap();
        assert!(matches!(events.as_slice(),[
            (SourceStage::CurrentFileAndLinks,SourceOutcome::Started),
            (SourceStage::CurrentFileAndLinks,SourceOutcome::Passed),
            (SourceStage::DirectRanges,SourceOutcome::Started),
            (SourceStage::DirectRanges,SourceOutcome::Failed)
        ]));
    }
    #[tokio::test]
    async fn range_timeout_drops_pending_role_without_returning_a_partial_pair() {
        struct Dropped(Arc<std::sync::atomic::AtomicBool>);
        impl Drop for Dropped {fn drop(&mut self) {self.0.store(true,std::sync::atomic::Ordering::SeqCst);}}
        let pc_finished=Arc::new(std::sync::atomic::AtomicBool::new(false));
        let phone_dropped=Arc::new(std::sync::atomic::AtomicBool::new(false));
        let phone_guard=Dropped(phone_dropped.clone());
        let pc=async {
            pc_finished.store(true,std::sync::atomic::Ordering::SeqCst);Ok::<_,&'static str>(1024)
        };
        let phone=async move {
            let _guard=phone_guard;
            std::future::pending::<Result<usize,&'static str>>().await
        };
        let events=std::sync::Mutex::new(Vec::new());
        let result=read_stage(tokio::time::Instant::now()+std::time::Duration::from_millis(20),
            SourceStage::DirectRanges,|_,outcome|events.lock().unwrap().push(outcome),
            async {tokio::try_join!(pc,phone)}).await;
        assert_eq!(result,Err(READ_TIMEOUT));
        assert!(pc_finished.load(std::sync::atomic::Ordering::SeqCst));
        assert!(phone_dropped.load(std::sync::atomic::Ordering::SeqCst));
        assert!(matches!(events.into_inner().unwrap().as_slice(),[SourceOutcome::Started,SourceOutcome::Failed]));
    }
    #[tokio::test]
    async fn timed_out_read_drops_work_preserves_authority_and_records_fixed_stage_failure() {
        struct Dropped(Arc<std::sync::atomic::AtomicBool>);
        impl Drop for Dropped {fn drop(&mut self) {self.0.store(true,std::sync::atomic::Ordering::SeqCst);}}
        let dropped=Arc::new(std::sync::atomic::AtomicBool::new(false));
        let guard=Dropped(dropped.clone());
        let host=Host::new("synthetic-private-deadline-pc-0000000".into(),"synthetic-private-deadline-phone-0000".into()).unwrap();
        let directory=std::env::var_os("TOGETHER_SOURCE_DEADLINE_PROOF_DIR").map(PathBuf::from)
            .unwrap_or_else(||std::env::temp_dir().join(format!("source-deadline-{}",uuid::Uuid::new_v4())));
        host.enable_diagnostics(directory.join("journal")).unwrap();
        host.attach_local_pc().await.unwrap();
        host.local_pc(None,together_session_server::Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
        let before=host.snapshot().await;
        let record=|stage,outcome|host.record_source(1,before.generation,false,stage,outcome,0);
        let result=read_stage(tokio::time::Instant::now()+std::time::Duration::from_millis(20),
            SourceStage::CurrentFileAndLinks,record,async move {
                let _guard=guard;
                std::future::pending::<Result<(),&'static str>>().await
            }).await;
        assert_eq!(result,Err(READ_TIMEOUT));
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst),"timeout retained pending work");
        assert!(serde_json::to_value(host.snapshot().await).unwrap()==serde_json::to_value(before).unwrap());
        let text=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
        assert!(!text.contains("https://") && !text.contains("Bearer "));
        let rows:Vec<serde_json::Value>=text.lines().map(|row|serde_json::from_str(row).unwrap()).collect();
        let source:Vec<_>=rows.iter().filter(|row|row["record"]["event"]=="source").collect();
        assert_eq!(source.len(),2);
        assert_eq!(source[0]["record"]["sample"]["stage"],"current_file_and_links");
        assert_eq!(source[0]["record"]["sample"]["outcome"],"started");
        assert_eq!(source[1]["record"]["sample"]["outcome"],"failed");
        assert!(host.media_grant(&host.snapshot().await,together_session_server::media::Device::Phone).is_none());
    }
    #[tokio::test]
    async fn read_stage_keeps_replacement_distinct_from_provider_failure() {
        for (error,replaced) in [("stale media resolution",true),("TorBox selection replaced",true),("synthetic provider unavailable",false)] {
            let events=std::sync::Mutex::new(Vec::new());
            let result=read_stage(tokio::time::Instant::now()+READ_DEADLINE,SourceStage::DirectRanges,
                |_,outcome|events.lock().unwrap().push(outcome),async {Err::<(),_>(error)}).await;
            assert_eq!(result,Err(error));
            let events=events.into_inner().unwrap();
            assert!(matches!(events[0],SourceOutcome::Started));
            assert_eq!(matches!(events[1],SourceOutcome::Replaced),replaced);
            assert_eq!(matches!(events[1],SourceOutcome::Failed),!replaced);
        }
    }
    #[tokio::test]
    #[ignore="Explicit PC-only completed source handoff proof; no native or phone acceptance"]
    async fn real_provider_completed_source_cannot_overwrite_pause_seek_or_shutdown() {
        use together_session_server::{Action,media::Device};
        assert!(private_alpha_account_links(),"private Alpha source policy required");
        let directory=PathBuf::from(std::env::var_os("TOGETHER_SOURCE_TRANSACTION_PROOF_DIR")
            .expect("isolated proof directory required"));
        assert!(directory.is_dir(),"proof directory must already exist");
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let controller=Controller::new(root.join(".secrets/torbox.dpapi"));
        let media=controller.videos().await.expect("provider inventory unavailable")
            .into_iter().next().expect("no ready provider video available");
        let host=Host::new("synthetic-private-handoff-pc-00000000".into(),
            "synthetic-private-handoff-phone-00000".into()).unwrap();
        host.enable_diagnostics(directory.join("journal")).unwrap();
        host.attach_local_pc().await.unwrap();
        let resolved=controller.resolve_canonical(&host,media.clone(),host.snapshot().await)
            .await.expect("initial provider transaction failed");
        controller.publish_resolved(&host,resolved).await.expect("initial publication failed");
        let initial=host.snapshot().await;
        let initial_revision=host.media_grant(&initial,Device::Pc).expect("initial PC grant missing").source_revision();
        let mut preserved_commands=0;
        for change in ["pause","seek","shutdown"] {
            let before=host.snapshot().await;
            // The actual provider work and both bounded ranges finish before the
            // authoritative command. This exercises the final publication fence.
            let resolved=controller.resolve_canonical(&host,media.clone(),before.clone())
                .await.expect("pending provider transaction failed");
            let old_pc=host.media_grant(&before,Device::Pc).expect("PC grant missing before change");
            let old_phone=host.media_grant(&before,Device::Phone).expect("phone grant missing before change");
            match change {
                "pause"=>host.local_pc(None,Action::Pause).await.unwrap(),
                "seek"=>host.local_pc(None,Action::Seek {position_ms:13_000}).await.unwrap(),
                "shutdown"=>host.request_shutdown(),
                _=>unreachable!(),
            }
            let authoritative=host.snapshot().await;
            assert!(old_pc.deliver(|_|()).is_err(),"old PC grant survived authority change");
            assert!(old_phone.deliver(|_|()).is_err(),"old phone grant survived authority change");
            let result=controller.publish_resolved(&host,resolved).await;
            assert_eq!(result,Err(if change=="shutdown" {"server stopping"}else {"stale media resolution"}));
            // Compare privately; assertion output must never print media metadata.
            assert!(serde_json::to_value(host.snapshot().await).unwrap()==serde_json::to_value(&authoritative).unwrap(),
                "rejected publication changed authoritative state");
            if change=="shutdown" {
                assert!(host.media_grant(&authoritative,Device::Pc).is_none());
                assert!(host.media_grant(&authoritative,Device::Phone).is_none());
            } else {
                for device in [Device::Pc,Device::Phone] {
                    let grant=host.media_grant(&authoritative,device).expect("existing source capability lost");
                    assert_eq!(grant.source_revision(),initial_revision,"rejected source replaced existing capabilities");
                    assert!(grant.deliver(|_|()).is_ok(),"current capability could not be delivered");
                }
                if change=="pause" {assert!(!authoritative.desired_playing);}
                else {assert_eq!(authoritative.timeline.position_ms,13_000);}
                preserved_commands+=1;
            }
        }
        let text=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
        assert!(!text.contains("https://") && !text.contains("Bearer ") && !text.contains("filename"),
            "private data entered source journal");
        let rows:Vec<serde_json::Value>=text.lines().map(|row|serde_json::from_str(row).unwrap()).collect();
        let publication_count=|wanted:&str|rows.iter().filter(|row|row["record"]["event"]=="source"
            && row["record"]["sample"]["stage"]=="publication"
            && row["record"]["sample"]["outcome"]==wanted).count();
        assert_eq!(publication_count("passed"),1);
        assert_eq!(publication_count("replaced"),2);
        assert_eq!(publication_count("failed"),1);
        let receipt=serde_json::json!({"status":"passed","scope":"PC completed source handoff and authority changes; no players",
            "provider_transactions":4,"preserved_commands":preserved_commands,"rejected_late_publications":3,
            "existing_capabilities_preserved":true,"shutdown_capabilities_revoked":true,
            "publication_passed_rows":1,"publication_replaced_rows":2,"publication_failed_rows":1,
            "journal_privacy_check_passed":true,"native_playback_verified":false,"phone_verified":false,"wan_control_verified":false});
        std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
    }
    #[tokio::test] async fn obsolete_selection_or_authoritative_command_stops_before_more_provider_work() {
        let controller=Controller::new(PathBuf::from("unused-synthetic-vault"));
        let host=Host::new("synthetic-private-pc-token-0000000".into(),"synthetic-private-phone-token-0000".into()).unwrap();
        host.attach_local_pc().await.unwrap();
        host.local_pc(None,together_session_server::Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
        let before=host.snapshot().await;
        assert!(controller.still_current(&host,&before,0).await.is_ok());
        controller.state.lock().await.selection=1;
        assert_eq!(controller.still_current(&host,&before,0).await,Err("TorBox selection replaced"));
        assert!(controller.still_current(&host,&before,1).await.is_ok());
        host.local_pc(None,together_session_server::Action::Pause).await.unwrap();
        assert_eq!(controller.still_current(&host,&before,1).await,Err("stale media resolution"));
    }
}

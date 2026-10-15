//! PC selection owner. Configured addon URLs and provider credentials never enter IPC.
use std::{path::PathBuf,sync::Arc};
use serde::Serialize;
use together_source_resolver::{stremio::{Addon,Catalog,LibraryTitle,Stremio},MediaIdentity};
use together_session_server::Host;
use together_source_resolver::preparation::{CachedSelection,Prepared};
#[derive(Serialize)]
pub struct Browser {pub connected:bool,pub revision:u64,pub catalogs:Vec<Catalog>,pub titles:Vec<LibraryTitle>,pub episode_mode:bool}
#[derive(Clone,Serialize)]
pub struct OptionRow {pub index:usize,pub filename:String,pub filesize:Option<u64>}
#[derive(Serialize)]
pub struct Options {pub revision:u64,pub candidates:usize,pub unavailable_providers:usize,pub cache_available:bool,pub ready:Vec<OptionRow>,pub cached:Vec<OptionRow>}
#[derive(Serialize)] pub struct PreparationStatus {pub ready:bool,pub filename:String,pub filesize:u64}
// A per-request timeout is insufficient when many configured addons fail in sequence.
const LOOKUP_DEADLINE:std::time::Duration=std::time::Duration::from_secs(60);
async fn bounded_lookup<T>(deadline:std::time::Duration,work:impl std::future::Future<Output=Result<T,&'static str>>)->Result<T,&'static str> {
    tokio::time::timeout(deadline,work).await.map_err(|_|"Stremio lookup took too long. Try another catalog or a ready TorBox video.")?
}
// Future cancellation/abort must clean up ownership as well as normal return.
struct LookupOwner<'a> {registry:&'a std::sync::Mutex<Option<tokio::sync::watch::Sender<bool>>>,sender:tokio::sync::watch::Sender<bool>}
impl Drop for LookupOwner<'_> {
    fn drop(&mut self) {
        if let Ok(mut active)=self.registry.lock() {
            if active.as_ref().is_some_and(|current|current.same_channel(&self.sender)) {*active=None;}
        }
    }
}
enum Choice {Ready(MediaIdentity),Cached(Arc<CachedSelection>)}
struct State {client:Option<Arc<Stremio>>,addons:Arc<Vec<Addon>>,catalogs:Vec<Catalog>,titles:Vec<LibraryTitle>,parent_titles:Option<Vec<LibraryTitle>>,choices:Vec<Choice>,revision:u64}
pub struct Controller {vault:PathBuf,state:tokio::sync::Mutex<State>,lookup:std::sync::Mutex<Option<tokio::sync::watch::Sender<bool>>>}
impl Controller {
    pub fn new(vault:PathBuf)->Self {Self {vault,lookup:std::sync::Mutex::new(None),state:tokio::sync::Mutex::new(State {client:None,addons:Arc::new(Vec::new()),catalogs:Vec::new(),titles:Vec::new(),parent_titles:None,choices:Vec::new(),revision:0})}}
    pub async fn browser(&self)->Browser {let state=self.state.lock().await;public_browser(&state)}
    async fn read_only_lookup<T>(&self,work:impl std::future::Future<Output=Result<T,&'static str>>)->Result<T,&'static str> {
        let (cancel,mut cancelled)=tokio::sync::watch::channel(false);
        {
            let mut active=self.lookup.lock().map_err(|_|"Stremio lookup unavailable")?;
            if let Some(previous)=active.replace(cancel.clone()) {previous.send_replace(true);}
        }
        let _owner=LookupOwner {registry:&self.lookup,sender:cancel};
        tokio::select! {biased;
            _=cancelled.changed()=>Err("Stremio lookup cancelled"),
            result=bounded_lookup(LOOKUP_DEADLINE,work)=>result,
        }
    }
    pub async fn cancel_lookup(&self)->Result<Browser,&'static str> {
        if let Some(active)=self.lookup.lock().map_err(|_|"Stremio lookup unavailable")?.take() {active.send_replace(true);}
        let mut state=self.state.lock().await;
        Self::begin_locked(&mut state,None)?;
        Ok(public_browser(&state))
    }

    async fn current(&self,revision:u64)->Result<(),&'static str> {
        if self.state.lock().await.revision!=revision {return Err("Stremio selection replaced");}
        Ok(())
    }
    fn begin_locked(state:&mut State,expected:Option<u64>)->Result<u64,&'static str> {
        if expected.is_some_and(|expected|expected!=state.revision) {return Err("Selection changed; choose the movie again");}
        state.revision=state.revision.checked_add(1).ok_or("Stremio selection exhausted")?;
        state.choices.clear();Ok(state.revision)
    }
    #[cfg(test)]
    async fn begin(&self,expected:Option<u64>)->Result<u64,&'static str> {
        let mut state=self.state.lock().await;Self::begin_locked(&mut state,expected)
    }
    pub async fn connect(&self)->Result<Browser,&'static str> {
        self.read_only_lookup(self.connect_inner()).await
    }
    async fn connect_inner(&self)->Result<Browser,&'static str> {
        let (revision,existing)={let mut state=self.state.lock().await;
            let revision=Self::begin_locked(&mut state,None)?;(revision,state.client.clone())};
        let client:Result<Arc<Stremio>,&'static str>=async {
            if let Some(client)=existing {return Ok(client);}
            let vault=self.vault.clone();
            let secret=tokio::task::spawn_blocking(move ||together_source_resolver::secret::read_key(&vault)).await
                .map_err(|_|"Stremio PC credentials unavailable")?.map_err(|_|"Stremio PC credentials unavailable")?;
            self.current(revision).await?;
            Ok(Arc::new(Stremio::from_pc_secret(secret).await.map_err(|_|"Could not sign in with the saved Stremio account")?))
        }.await;
        let client=match client {Ok(client)=>client,Err(error)=>{self.connection_failed(revision).await;return Err(error);}};
        self.current(revision).await?;
        let (titles,addons)=match tokio::try_join!(client.library(),client.addons()) {
            Ok(result)=>result,
            Err(_)=>{self.connection_failed(revision).await;return Err("Could not load your Stremio library and addons; reconnect to sign in again");},
        };
        let catalogs=client.catalogs(&addons);
        let mut state=self.state.lock().await;
        if state.revision!=revision {return Err("Stremio connection replaced");}
        state.client=Some(client);state.addons=Arc::new(addons);state.catalogs=catalogs;state.titles=titles;state.parent_titles=None;
        Ok(public_browser(&state))
    }
    async fn connection_failed(&self,revision:u64) {
        let mut state=self.state.lock().await;
        // Late failures must not erase a newer successful account/catalog.
        if state.revision!=revision {return;}
        state.client=None;state.addons=Arc::new(Vec::new());state.catalogs.clear();
        state.titles.clear();state.parent_titles=None;state.choices.clear();
    }
    pub async fn browse(&self,expected:u64,index:usize,search:Option<String>)->Result<Browser,&'static str> {
        self.read_only_lookup(self.browse_inner(expected,index,search)).await
    }
    async fn browse_inner(&self,expected:u64,index:usize,search:Option<String>)->Result<Browser,&'static str> {
        let (revision,client,addons,catalog)={let mut state=self.state.lock().await;
            let revision=Self::begin_locked(&mut state,Some(expected))?;
            (revision,state.client.clone().ok_or("Connect Stremio first")?,state.addons.clone(),state.catalogs.get(index).cloned().ok_or("Choose a catalog")?)};
        let titles=client.catalog(&addons[catalog.addon_index],catalog.catalog_index,search.as_deref()).await
            .map_err(|_|"Catalog unavailable; check the search requirement and try again")?;
        let mut state=self.state.lock().await;
        if state.revision!=revision {return Err("Catalog selection replaced");}
        state.titles=titles;state.parent_titles=None;Ok(public_browser(&state))
    }
    pub async fn episodes(&self,expected:u64,index:usize)->Result<Browser,&'static str> {
        self.read_only_lookup(self.episodes_inner(expected,index)).await
    }
    async fn episodes_inner(&self,expected:u64,index:usize)->Result<Browser,&'static str> {
        let (revision,client,addons,title)={let mut state=self.state.lock().await;
            let revision=Self::begin_locked(&mut state,Some(expected))?;
            if state.parent_titles.is_some() {return Err("Choose an episode or return to series titles");}
            (revision,state.client.clone().ok_or("Connect Stremio first")?,state.addons.clone(),state.titles.get(index).cloned().ok_or("Choose a series")?)};
        if title.kind!="series" {return Err("Choose a series");}
        let mut episodes=Vec::new();
        for addon in addons.iter().filter(|addon|addon.supports_meta("series",&title.id)) {
            self.current(revision).await?;
            let result=client.episodes(addon,&title.id).await;
            self.current(revision).await?;
            if let Ok(result)=result {if !result.is_empty() {episodes=result;break;}}
        }
        if episodes.is_empty() {return Err("No episode list was available from your configured addons");}
        let mut state=self.state.lock().await;
        if state.revision!=revision {return Err("Series selection replaced");}
        state.parent_titles=Some(std::mem::take(&mut state.titles));state.titles=episodes;Ok(public_browser(&state))
    }
    pub async fn back(&self,expected:u64)->Result<Browser,&'static str> {
        let mut state=self.state.lock().await;
        Self::begin_locked(&mut state,Some(expected))?;
        if let Some(titles)=state.parent_titles.take() {state.titles=titles;}
        Ok(public_browser(&state))
    }
    pub async fn options(&self,torbox:&super::torbox::Controller,expected:u64,index:usize)->Result<Options,&'static str> {
        self.read_only_lookup(self.options_inner(torbox,expected,index)).await
    }
    async fn options_inner(&self,torbox:&super::torbox::Controller,expected:u64,index:usize)->Result<Options,&'static str> {
        let (revision,client,addons,title,episode_mode)={let mut state=self.state.lock().await;
            let revision=Self::begin_locked(&mut state,Some(expected))?;
            (revision,state.client.clone().ok_or("Connect Stremio first")?,state.addons.clone(),state.titles.get(index).cloned().ok_or("Choose a movie or episode")?,state.parent_titles.is_some())};
        if title.kind!="movie" && !(title.kind=="series" && episode_mode) {return Err("Choose an episode before selecting a series stream");}
        self.current(revision).await?;
        let available=torbox.videos().await;
        self.current(revision).await?;
        let available=available?;
        let mut candidates=0;let mut unavailable_providers=0;let mut choices=Vec::new();let mut descriptors=Vec::new();
        for addon in addons.iter().filter(|addon|addon.supports_stream(&title.kind,&title.id)) {
            self.current(revision).await?;
            let result=client.streams(addon,&title.kind,&title.id).await;
            self.current(revision).await?;
            match result {
                Ok(streams)=>{
                    if streams.len()>together_source_resolver::stremio::MAX_STREAM_CHOICES.saturating_sub(descriptors.len()) {
                        return Err("Stremio returned too many source choices; use a ready TorBox video instead");
                    }
                    candidates+=streams.len();for stream in streams {
                    if let Ok(media)=stream.canonical_media(&available) {
                        if !choices.iter().any(|entry:&Choice|matches!(entry,Choice::Ready(entry) if entry.torrent_id==media.torrent_id && entry.file_id==media.file_id)) {choices.push(Choice::Ready(media));}
                    }
                    descriptors.push(stream);
                }},Err(_)=>unavailable_providers+=1,
            }
        }
        let ready=choices.iter().enumerate().filter_map(|(index,choice)|match choice {Choice::Ready(media)=>Some(OptionRow {index,filename:media.filename.clone().unwrap_or("Video".into()),filesize:media.filesize}),_=>None}).collect();
        let mut cached=Vec::new();let mut cache_available=true;
        self.current(revision).await?;
        let plans=torbox.cached_selections(&descriptors).await;
        self.current(revision).await?;
        match plans {
            Ok(plans)=>for (_,plan) in plans {
                if choices.iter().any(|choice|match choice {Choice::Ready(media)=>plan.matches_media(media),Choice::Cached(existing)=>plan.same_file(existing)}) {continue;}
                cached.push(OptionRow {index:choices.len(),filename:plan.filename().into(),filesize:Some(plan.filesize())});choices.push(Choice::Cached(Arc::new(plan)));
            },Err(_)=>cache_available=false,
        }
        let mut state=self.state.lock().await;
        if state.revision!=revision {return Err("Movie selection replaced");}
        state.choices=choices;Ok(Options {revision,candidates,unavailable_providers,cache_available,ready,cached})
    }
    pub async fn select(&self,torbox:&super::torbox::Controller,host:&Host,revision:u64,index:usize)->Result<(),&'static str> {
        let before=host.snapshot().await;
        self.select_with(revision,index,|media|torbox.resolve_canonical(host,media,before),
            |resolved|torbox.publish_resolved(host,resolved),|resolved|torbox.discard_resolved(host,resolved)).await
    }
    async fn select_with<T,R,P>(&self,revision:u64,index:usize,
        resolve:impl FnOnce(MediaIdentity)->R,publish:impl FnOnce(T)->P,discard:impl FnOnce(T))->Result<(),&'static str>
        where R:std::future::Future<Output=Result<T,&'static str>>,P:std::future::Future<Output=Result<(),&'static str>> {
        let media={
            let state=self.state.lock().await;
            if state.revision!=revision {return Err("Selection changed; choose the movie again");}
            match state.choices.get(index) {Some(Choice::Ready(media))=>media.clone(),_=>return Err("Prepare this video in TorBox before watching")}
        };
        let resolved=resolve(media).await?;
        // Fence the original choice at the final short publication boundary,
        // rather than holding browser ownership throughout provider requests.
        let state=self.state.lock().await;
        if state.revision!=revision {discard(resolved);return Err("Selection changed; choose the movie again");}
        publish(resolved).await
    }
    pub async fn prepare(&self,torbox:&super::torbox::Controller,revision:u64,index:usize)->Result<PreparationStatus,&'static str> {
        let plan={
            let state=self.state.lock().await;
            if state.revision!=revision {return Err("Selection changed; choose the movie again");}
            match state.choices.get(index) {
                Some(Choice::Cached(plan))=>plan.clone(),
                Some(Choice::Ready(media))=>return Ok(PreparationStatus {ready:true,filename:media.filename.clone().unwrap_or("Video".into()),filesize:media.filesize.unwrap_or(0)}),
                None=>return Err("Choose a cached video"),
            }
        };
        let status=PreparationStatus {ready:false,filename:plan.filename().into(),filesize:plan.filesize()};
        // Provider IO must not reserve the browser state. The provider retains
        // its create-attempt guard; a replaced result cannot update this choice.
        self.finish_preparation(revision,index,status,torbox.prepare_cached(&plan)).await
    }
    async fn finish_preparation(&self,revision:u64,index:usize,status:PreparationStatus,
        pending:impl std::future::Future<Output=Result<Prepared,&'static str>>)->Result<PreparationStatus,&'static str> {
        let result=pending.await?;
        let mut state=self.state.lock().await;
        if state.revision!=revision {return Err("Selection changed; choose the movie again");}
        match result {
            Prepared::Ready(media)=>{state.choices[index]=Choice::Ready(media);Ok(PreparationStatus {ready:true,..status})},
            Prepared::Pending=>Ok(status),
        }
    }

}
fn public_browser(state:&State)->Browser {Browser {connected:state.client.is_some(),revision:state.revision,catalogs:state.catalogs.clone(),titles:state.titles.clone(),episode_mode:state.parent_titles.is_some()}}
#[cfg(test)] mod tests {
    use super::*;
    #[tokio::test] async fn pending_preparation_allows_browser_reads_and_obsolete_result_cannot_replace_choice() {
        let controller=Arc::new(Controller::new(PathBuf::from("unused-private-preparation-vault")));
        {let mut state=controller.state.lock().await;state.revision=1;state.choices.push(Choice::Ready(MediaIdentity::torbox(1,2)));}
        let (entered_tx,entered_rx)=tokio::sync::oneshot::channel();
        let (release_tx,release_rx)=tokio::sync::oneshot::channel();
        let pending_controller=controller.clone();
        let task=tokio::spawn(async move {
            pending_controller.finish_preparation(1,0,PreparationStatus {ready:false,filename:"movie.mp4".into(),filesize:4096},async {
                entered_tx.send(()).unwrap();release_rx.await.unwrap();Ok(Prepared::Ready(MediaIdentity::torbox(9,8)))
            }).await
        });
        entered_rx.await.unwrap();
        assert_eq!(tokio::time::timeout(std::time::Duration::from_secs(1),controller.browser()).await.unwrap().revision,1);
        let revision=tokio::time::timeout(std::time::Duration::from_secs(1),controller.begin(Some(1))).await.unwrap().unwrap();
        controller.state.lock().await.choices.push(Choice::Ready(MediaIdentity::torbox(5,6)));
        release_tx.send(()).unwrap();
        assert!(matches!(task.await.unwrap(),Err("Selection changed; choose the movie again")));
        let state=controller.state.lock().await;
        assert_eq!(state.revision,revision);
        assert!(matches!(&state.choices[0],Choice::Ready(media) if media.torrent_id==5 && media.file_id==6));
    }
    #[tokio::test] async fn current_preparation_commits_ready_file_and_pending_or_failure_preserves_choice() {
        let controller=Controller::new(PathBuf::from("unused-private-preparation-vault"));
        {let mut state=controller.state.lock().await;state.revision=1;state.choices.push(Choice::Ready(MediaIdentity::torbox(1,2)));}
        let status=||PreparationStatus {ready:false,filename:"movie.mp4".into(),filesize:4096};
        assert!(!controller.finish_preparation(1,0,status(),async {Ok(Prepared::Pending)}).await.unwrap().ready);
        assert!(matches!(controller.finish_preparation(1,0,status(),async {Err("provider unavailable")}).await,Err("provider unavailable")));
        assert!(matches!(&controller.state.lock().await.choices[0],Choice::Ready(media) if media.torrent_id==1));
        let ready=controller.finish_preparation(1,0,status(),async {Ok(Prepared::Ready(MediaIdentity::torbox(7,8)))}).await.unwrap();
        assert!(ready.ready);assert_eq!(ready.filename,"movie.mp4");
        assert!(matches!(&controller.state.lock().await.choices[0],Choice::Ready(media) if media.torrent_id==7 && media.file_id==8));
    }
    #[tokio::test] async fn pending_watch_resolution_does_not_lock_browser_and_replaced_choice_never_publishes() {
        let controller=Arc::new(Controller::new(PathBuf::from("unused-private-watch-vault")));
        {let mut state=controller.state.lock().await;state.revision=1;state.choices.push(Choice::Ready(MediaIdentity::torbox(1,2)));}
        let (entered_tx,entered_rx)=tokio::sync::oneshot::channel();let (release_tx,release_rx)=tokio::sync::oneshot::channel();
        let published=Arc::new(std::sync::atomic::AtomicUsize::new(0));let commits=published.clone();let discarded=Arc::new(std::sync::atomic::AtomicUsize::new(0));let drops=discarded.clone();let pending_controller=controller.clone();
        let task=tokio::spawn(async move {pending_controller.select_with(1,0,|media|async move {
            assert_eq!(media.torrent_id,1);entered_tx.send(()).unwrap();release_rx.await.unwrap();Ok(())
        },|_|async move {commits.fetch_add(1,std::sync::atomic::Ordering::SeqCst);Ok(())},|_| {drops.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}).await});
        entered_rx.await.unwrap();
        assert_eq!(tokio::time::timeout(std::time::Duration::from_secs(1),controller.browser()).await.unwrap().revision,1);
        controller.begin(Some(1)).await.unwrap();release_tx.send(()).unwrap();
        assert!(matches!(task.await.unwrap(),Err("Selection changed; choose the movie again")));
        assert_eq!(published.load(std::sync::atomic::Ordering::SeqCst),0);assert_eq!(discarded.load(std::sync::atomic::Ordering::SeqCst),1);
    }
    #[tokio::test] async fn current_watch_publishes_once_and_resolution_error_never_publishes() {
        let controller=Controller::new(PathBuf::from("unused-private-watch-vault"));
        {let mut state=controller.state.lock().await;state.revision=1;state.choices.push(Choice::Ready(MediaIdentity::torbox(1,2)));}
        let published=std::sync::atomic::AtomicUsize::new(0);
        controller.select_with(1,0,|_|async {Ok(())},|_|async {published.fetch_add(1,std::sync::atomic::Ordering::SeqCst);Ok(())},|_| {}).await.unwrap();
        let failed=controller.select_with(1,0,|_|async {Err::<(),_>("resolution failed")},|_|async {published.fetch_add(1,std::sync::atomic::Ordering::SeqCst);Ok(())},|_| {}).await;
        assert_eq!(failed,Err("resolution failed"));assert_eq!(published.load(std::sync::atomic::Ordering::SeqCst),1);
    }
    #[tokio::test] async fn lookup_deadline_drops_pending_provider_work_and_preserves_completed_results() {
        struct Dropped(Arc<std::sync::atomic::AtomicUsize>);
        impl Drop for Dropped {fn drop(&mut self) {self.0.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}}
        let drops=Arc::new(std::sync::atomic::AtomicUsize::new(0));let guard=Dropped(drops.clone());
        let result=bounded_lookup(std::time::Duration::from_millis(20),async move {
            let _guard=guard;std::future::pending::<Result<(),&'static str>>().await
        }).await;
        assert_eq!(result,Err("Stremio lookup took too long. Try another catalog or a ready TorBox video."));
        assert_eq!(drops.load(std::sync::atomic::Ordering::SeqCst),1);
        assert_eq!(bounded_lookup(LOOKUP_DEADLINE,async {Ok(7)}).await,Ok(7));
        assert_eq!(bounded_lookup(LOOKUP_DEADLINE,async {Err::<(),_>("catalog unavailable")}).await,Err("catalog unavailable"));
    }
    #[tokio::test] async fn cancellation_drops_lookup_and_invalidates_choices_without_clearing_titles() {
        let controller=Arc::new(Controller::new(PathBuf::from("unused-private-cancel-vault")));
        {let mut state=controller.state.lock().await;state.revision=3;state.titles.push(LibraryTitle {id:"movie".into(),kind:"movie".into(),name:"Movie".into()});state.choices.push(Choice::Ready(MediaIdentity::torbox(1,2)));}
        let (entered_tx,entered_rx)=tokio::sync::oneshot::channel();let worker=controller.clone();
        let task=tokio::spawn(async move {worker.read_only_lookup(async {entered_tx.send(()).unwrap();std::future::pending::<Result<(),&'static str>>().await}).await});
        entered_rx.await.unwrap();let browser=controller.cancel_lookup().await.unwrap();
        assert_eq!(browser.revision,4);assert_eq!(browser.titles.len(),1);
        assert_eq!(tokio::time::timeout(std::time::Duration::from_secs(1),task).await.unwrap().unwrap(),Err("Stremio lookup cancelled"));
        assert!(controller.state.lock().await.choices.is_empty());assert!(controller.lookup.lock().unwrap().is_none());
    }
    #[tokio::test] async fn replaced_lookup_cleanup_cannot_disarm_cancellation_of_new_lookup() {
        let controller=Arc::new(Controller::new(PathBuf::from("unused-private-cancel-vault")));
        let start=|controller:Arc<Controller>| {
            let (entered_tx,entered_rx)=tokio::sync::oneshot::channel();
            let task=tokio::spawn(async move {controller.read_only_lookup(async {entered_tx.send(()).unwrap();std::future::pending::<Result<(),&'static str>>().await}).await});
            (entered_rx,task)
        };
        let (first_entered,first)=start(controller.clone());first_entered.await.unwrap();
        let (second_entered,second)=start(controller.clone());second_entered.await.unwrap();
        assert_eq!(first.await.unwrap(),Err("Stremio lookup cancelled"));
        assert!(controller.lookup.lock().unwrap().is_some());controller.cancel_lookup().await.unwrap();
        assert_eq!(tokio::time::timeout(std::time::Duration::from_secs(1),second).await.unwrap().unwrap(),Err("Stremio lookup cancelled"));
    }
    #[tokio::test] async fn aborted_lookup_releases_only_its_cancellation_owner() {
        let controller=Arc::new(Controller::new(PathBuf::from("unused-private-cancel-vault")));
        let (entered_tx,entered_rx)=tokio::sync::oneshot::channel();let worker=controller.clone();
        let task=tokio::spawn(async move {worker.read_only_lookup(async {entered_tx.send(()).unwrap();std::future::pending::<Result<(),&'static str>>().await}).await});
        entered_rx.await.unwrap();task.abort();assert!(task.await.unwrap_err().is_cancelled());
        assert!(controller.lookup.lock().unwrap().is_none(),"aborted lookup left a stale cancellation owner");
        assert_eq!(controller.read_only_lookup(async {Ok(7)}).await,Ok(7));
    }
    #[tokio::test]
    #[ignore="Explicit PC-only real-provider source transaction/range proof; no native or phone acceptance"]
    async fn real_provider_source_transaction_discards_obsolete_and_publishes_current_with_safe_journal() {
        assert!(super::super::torbox::private_alpha_account_links(),"private Alpha source policy required");
        let directory=PathBuf::from(std::env::var_os("TOGETHER_SOURCE_TRANSACTION_PROOF_DIR").expect("isolated proof directory required"));
        assert!(directory.is_dir(),"proof directory must already exist");
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let torbox=super::super::torbox::Controller::new(root.join(".secrets/torbox.dpapi"));
        let inventory=torbox.videos().await.expect("provider inventory unavailable");
        let media=inventory.into_iter().next().expect("no ready provider video available");
        let host=Host::new("synthetic-private-source-pc-000000000".into(),"synthetic-private-source-phone-000000".into()).unwrap();
        host.enable_diagnostics(directory.join("journal")).unwrap();host.attach_local_pc().await.unwrap();
        let controller=Arc::new(Controller::new(PathBuf::from("unused-private-source-proof")));
        {let mut state=controller.state.lock().await;state.revision=1;state.choices.push(Choice::Ready(media.clone()));}
        let before=host.snapshot().await;
        let resolved_controller=controller.clone();
        let result=controller.select_with(1,0,|selected|async {
            let resolved=torbox.resolve_canonical(&host,selected,before).await?;
            resolved_controller.begin(Some(1)).await?;
            Ok(resolved)
        },|resolved|torbox.publish_resolved(&host,resolved),|resolved|torbox.discard_resolved(&host,resolved)).await;
        assert!(matches!(result,Err("Selection changed; choose the movie again")),"obsolete transaction was not rejected");
        assert!(host.snapshot().await.media.is_none(),"obsolete transaction published media");
        controller.state.lock().await.choices.push(Choice::Ready(media.clone()));
        controller.select(&torbox,&host,2,0).await.expect("current provider transaction failed");
        let current=host.snapshot().await;
        assert!(current.media.as_ref().is_some_and(|selected|selected.torrent_id==media.torrent_id && selected.file_id==media.file_id),"published canonical file mismatch");
        let text=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
        assert!(!text.contains("https://") && !text.contains("Bearer ") && !text.contains("filename"),"private data entered source journal");
        let rows:Vec<serde_json::Value>=text.lines().map(|row|serde_json::from_str(row).unwrap()).collect();
        let source_rows:Vec<_>=rows.iter().filter(|row|row["record"]["event"]=="source").collect();
        let outcome=|wanted:&str|source_rows.iter().filter(|row|row["record"]["sample"]["stage"]=="publication" && row["record"]["sample"]["outcome"]==wanted).count();
        assert_eq!(outcome("replaced"),1);assert_eq!(outcome("passed"),1);
        let receipt=serde_json::json!({"status":"passed","scope":"PC source metadata and direct ranges; no native player or Android",
            "obsolete_publications":0,"replaced_publication_rows":1,"current_publication_rows":1,
            "journal_privacy_check_passed":true,"provider_transactions":2,"native_playback_verified":false,"phone_verified":false,"wan_control_verified":false});
        std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
    }
    #[tokio::test] async fn browser_reports_disconnected_without_exposing_private_configuration() {
        let controller=Controller::new(PathBuf::from("unused-private-test-vault"));
        let value=serde_json::to_value(controller.browser().await).unwrap();
        assert_eq!(value["connected"],false);
        assert!(!value.to_string().contains("vault"));
    }
    #[tokio::test] async fn connection_wait_has_cancellation_owner_before_state_admission() {
        use std::{future::Future,task::Poll};
        let vault=std::env::temp_dir().join(format!("missing-connection-cancel-{}",uuid::Uuid::new_v4()));
        assert!(!vault.exists());
        let controller=Controller::new(vault);
        let state=controller.state.lock().await;
        let mut connection=Box::pin(controller.connect());
        // Explicitly poll the actual connect method into its held state lock.
        std::future::poll_fn(|cx|{assert!(connection.as_mut().poll(cx).is_pending());Poll::Ready(())}).await;
        assert!(controller.lookup.lock().unwrap().is_some(),"pending connection has no cancellation owner");
        let mut cancel=Box::pin(controller.cancel_lookup());
        std::future::poll_fn(|cx|{assert!(cancel.as_mut().poll(cx).is_pending());Poll::Ready(())}).await;
        assert!(matches!(connection.await,Err("Stremio lookup cancelled")));
        drop(state);
        let browser=cancel.await.unwrap();
        assert!(!browser.connected);assert_eq!(browser.revision,1);
        assert!(controller.lookup.lock().unwrap().is_none());
        // Cancellation stopped before credential/provider work; next explicit retry
        // still follows the ordinary missing-vault error and clears its owner.
        assert!(matches!(controller.connect().await,Err("Stremio PC credentials unavailable")));
        assert!(controller.lookup.lock().unwrap().is_none());
    }
    #[tokio::test] async fn missing_credential_reconnect_clears_old_titles_and_explains_failure() {
        let path=std::env::temp_dir().join(format!("missing-stremio-vault-{}",uuid::Uuid::new_v4()));
        assert!(!path.exists());
        let controller=Controller::new(path);
        {let mut state=controller.state.lock().await;
            state.titles.push(LibraryTitle {id:"cached-title".into(),kind:"movie".into(),name:"Old cached movie".into()});
            state.choices.push(Choice::Ready(MediaIdentity::torbox(1,7)));
        }
        let error=match controller.connect().await {Ok(_)=>panic!("missing vault connected"),Err(error)=>error};
        assert_eq!(error,"Stremio PC credentials unavailable");
        let browser=controller.browser().await;
        assert!(!browser.connected);assert!(browser.titles.is_empty());assert_eq!(browser.revision,1);
        assert!(controller.state.lock().await.choices.is_empty());
    }
    #[tokio::test] async fn current_connection_failure_clears_choices_but_stale_failure_preserves_newer_account() {
        let controller=Controller::new(PathBuf::from("unused-private-test-vault"));
        {let mut state=controller.state.lock().await;
            state.revision=2;
            state.client=Some(Arc::new(Stremio::new(zeroize::Zeroizing::new("synthetic-private-stremio-key-0000".into())).unwrap()));
            state.catalogs.push(Catalog {addon_index:0,catalog_index:0,kind:"movie".into(),name:"Movies".into(),searchable:false,search_required:false});
            state.titles.push(LibraryTitle {id:"movie-original".into(),kind:"movie".into(),name:"Movie".into()});
            state.parent_titles=Some(state.titles.clone());
            state.choices.push(Choice::Ready(MediaIdentity::torbox(1,7)));
        }
        controller.connection_failed(1).await;
        let current=controller.browser().await;
        assert!(current.connected);assert_eq!(current.titles.len(),1);assert_eq!(current.catalogs.len(),1);
        assert_eq!(controller.state.lock().await.choices.len(),1);
        controller.connection_failed(2).await;
        let disconnected=controller.browser().await;
        assert!(!disconnected.connected);assert!(disconnected.titles.is_empty());assert!(disconnected.catalogs.is_empty());
        assert!(!disconnected.episode_mode);assert!(controller.state.lock().await.choices.is_empty());
        assert!(!serde_json::to_string(&disconnected).unwrap().contains("private"));
    }
    #[tokio::test] async fn stale_browser_revisions_cannot_reuse_or_replace_private_choices() {
        let controller=Controller::new(PathBuf::from("unused-test-vault"));
        let revision=controller.begin(None).await.unwrap();
        controller.state.lock().await.choices.push(Choice::Ready(MediaIdentity::torbox(1,7)));
        assert!(controller.begin(Some(revision-1)).await.is_err());
        assert_eq!(controller.state.lock().await.choices.len(),1);
        let next=controller.begin(Some(revision)).await.unwrap();assert_eq!(next,revision+1);
        assert!(controller.state.lock().await.choices.is_empty());
        let browser=controller.browser().await;assert_eq!(browser.revision,next);
        assert!(!serde_json::to_string(&browser).unwrap().contains("vault"));
    }
    #[tokio::test] async fn returning_from_episodes_invalidates_choices_and_restores_the_original_series() {
        let controller=Controller::new(PathBuf::from("unused-test-vault"));
        {let mut state=controller.state.lock().await;
            state.parent_titles=Some(vec![LibraryTitle {id:"series-original".into(),kind:"series".into(),name:"Series".into()}]);
            state.titles=vec![LibraryTitle {id:"explicit-episode".into(),kind:"series".into(),name:"S1 E1".into()}];
            state.choices.push(Choice::Ready(MediaIdentity::torbox(1,7)));}
        let browser=controller.back(0).await.unwrap();assert!(!browser.episode_mode);assert_eq!(browser.titles[0].id,"series-original");
        assert!(controller.state.lock().await.choices.is_empty());assert!(controller.back(0).await.is_err());
    }
}

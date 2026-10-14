use axum::{extract::{State, WebSocketUpgrade, DefaultBodyLimit, ws::{WebSocket, Message}}, http::{HeaderMap, StatusCode}, response::IntoResponse, routing::{get,post}, Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::{collections::BTreeSet, sync::Arc, time::Instant};
use tokio::sync::{broadcast, Mutex};
use together_sync_core::{Session, Snapshot, MediaIdentity, PlayerFault, PROTOCOL_VERSION};
use sha2::{Digest,Sha256};
use subtle::ConstantTimeEq;
mod limits;
pub mod credentials;
pub mod diagnostics;
pub mod tunnel;
pub mod media;
pub mod android_update;
pub mod phone_logs;
struct PairingWindow {digest:[u8;32],expires_ms:u64,attempts:u8}
// Cancellation of serve() must not leave its ticker owning the Host forever.
struct ServerLifetime {ticker:Option<tokio::task::JoinHandle<()>>,stopping:tokio::sync::watch::Sender<bool>}
impl Drop for ServerLifetime {fn drop(&mut self) {self.stopping.send_replace(true);if let Some(ticker)=self.ticker.take() {ticker.abort();}}}

pub struct Host {
    session: Mutex<Session>,
    connections: Mutex<BTreeSet<String>>,
    updates: broadcast::Sender<Snapshot>,
    boot: Instant,
    pc_token: [u8;32],
    phone_token: [u8;32],
    upgrades:Mutex<limits::Bucket>,
    pairing:Mutex<Option<PairingWindow>>,
    phone_credential:zeroize::Zeroizing<String>,
    diagnostics:std::sync::OnceLock<diagnostics::Journal>,
    diagnostics_setup:std::sync::Mutex<()>,
    stopping:tokio::sync::watch::Sender<bool>,
    serve_started:std::sync::atomic::AtomicBool,
    media:Arc<std::sync::Mutex<media::Slot>>,
    android_release:std::sync::OnceLock<std::path::PathBuf>,
    phone_log_batch:Mutex<phone_logs::RetryCache>,
}
impl Host {
    pub fn new(pc_token: String, phone_token: String) -> anyhow::Result<Arc<Self>> {
        anyhow::ensure!(credentials::valid_pairing_token(&pc_token) && credentials::valid_pairing_token(&phone_token) && pc_token != phone_token,
            "two distinct pairing tokens of 32 to 256 visible ASCII characters are required");
        let (updates, _) = broadcast::channel(64);
        let (stopping,_)=tokio::sync::watch::channel(false);
        Ok(Arc::new(Self { session: Mutex::new(Session::new(uuid::Uuid::new_v4().to_string())),
            connections: Mutex::new(BTreeSet::new()), updates, boot: Instant::now(),
            pc_token:Sha256::digest(pc_token.as_bytes()).into(),phone_token:Sha256::digest(phone_token.as_bytes()).into(),
            upgrades:Mutex::new(limits::Bucket::new(60,1,0)),pairing:Mutex::new(None),phone_credential:zeroize::Zeroizing::new(phone_token),diagnostics:std::sync::OnceLock::new(),diagnostics_setup:std::sync::Mutex::new(()),stopping,serve_started:std::sync::atomic::AtomicBool::new(false),media:Arc::new(std::sync::Mutex::new(media::Slot::new())),android_release:std::sync::OnceLock::new(),phone_log_batch:Mutex::new(phone_logs::RetryCache::default()) }))
    }
    /// Local setup path only. Requests never supply filesystem paths.
    pub fn configure_android_updates(&self,path:std::path::PathBuf)->Result<(), &'static str> {
        self.android_release.set(path).map_err(|_|"updates already configured")
    }
    pub fn now(&self) -> u64 { self.boot.elapsed().as_millis() as u64 }
    /// Local application lifecycle only, never a remote route. Idempotently
    /// stops accepting control work and wakes the listener and all sockets.
    /// Await the serve task separately (with an application shutdown deadline).
    /// A stopped Host cannot be restarted; create a fresh authority epoch.
    pub fn request_shutdown(&self) {self.stopping.send_replace(true);if let Ok(mut slot)=self.media.lock() {slot.close();}}
    /// In-process read-only work may stop with the Host. Subscribe before checking
    /// so shutdown racing waiter setup is never missed. Dropping a waiter is safe.
    pub async fn shutdown_requested(&self) {
        let mut stopping=self.stopping.subscribe();
        if *stopping.borrow_and_update() {return;}
        let _=stopping.changed().await;
    }
    /// Validated PC resolver output only. No network route accepts source URLs.
    /// An asynchronous resolution cannot overwrite a newer authoritative action.
    pub async fn load_resolved_media(&self, epoch:&str, generation:u64, streams:together_source_resolver::DeviceStreams)->Result<(), &'static str> {
        let sources=media::Sources::validated(streams)?;
        self.publish_sources(epoch,generation,sources,false).await
    }
    /// Replace both device capabilities without restarting the movie. A newer
    /// pause, seek, replacement or shutdown invalidates the pending resolution.
    pub async fn refresh_resolved_media(&self, epoch:&str, generation:u64, streams:together_source_resolver::DeviceStreams)->Result<(), &'static str> {
        let sources=media::Sources::validated(streams)?;
        self.publish_sources(epoch,generation,sources,true).await
    }
    async fn publish_sources(&self,epoch:&str,generation:u64,sources:media::Sources,refresh:bool)->Result<(), &'static str> {
        let mut session=self.session.lock().await;
        if *self.stopping.borrow() {return Err("server stopping");}
        if session.state.epoch!=epoch || session.state.generation!=generation {return Err("stale media resolution");}
        sources.identity().validate()?;
        if refresh {
            if session.state.media.as_ref()!=Some(sources.identity()) {return Err("media identity mismatch");}
            if session.state.ended {return Err("movie ended; use Replay");}
        }
        let mut slot=self.media.lock().map_err(|_|"media unavailable")?;
        slot.install(sources)?;
        let identity=slot.identity().ok_or("media unavailable")?.clone();
        let now=self.now();
        if refresh {
            let position=session.state.timeline.position(now);
            session.seek(position,generation,now)?;
        } else {session.load(identity,now)?;}
        slot.observe(&session.state);drop(slot);
        self.publish(&session);Ok(())
    }
    pub fn media_grant(&self,state:&Snapshot,device:media::Device)->Option<media::Grant> {media::grant(&self.media,state,device)}
    /// Local native owner only; queued snapshots may outlive PC pause/seek/load.
    /// Source capabilities additionally validate atomically at their own handoff.
    pub fn is_current_media_generation(&self,state:&Snapshot)->bool {media::current_generation(&self.media,state)}
    pub async fn snapshot(&self) -> Snapshot { self.session.lock().await.state.clone() }
    /// Local Windows UI only; there is deliberately no public route to open this window.
    pub async fn open_pairing(&self)->Result<String,&'static str> {
        let mut pairing=self.pairing.lock().await;
        if *self.stopping.borrow() {return Err("server stopping");}
        let random=uuid::Uuid::new_v4().simple().to_string();let code=&random[..12];
        *pairing=Some(PairingWindow {digest:Sha256::digest(code.as_bytes()).into(),expires_ms:self.now()+300_000,attempts:0});
        Ok(format!("{}-{}-{}",&code[..4],&code[4..8],&code[8..]))
    }
    pub async fn cancel_pairing(&self) {*self.pairing.lock().await=None;}
    async fn claim_pairing(&self,code:&str)->Result<String,StatusCode> {
        let mut pairing=self.pairing.lock().await;
        if *self.stopping.borrow() {return Err(StatusCode::SERVICE_UNAVAILABLE);}
        let Some(window)=pairing.as_mut() else {return Err(StatusCode::FORBIDDEN)};
        if self.now()>=window.expires_ms || window.attempts>=8 {*pairing=None;return Err(StatusCode::FORBIDDEN);}
        window.attempts+=1;
        let normalized=code.replace('-',"").to_ascii_lowercase();
        let digest:[u8;32]=Sha256::digest(normalized.as_bytes()).into();
        if normalized.len()!=12 || !bool::from(digest.ct_eq(&window.digest)) {return Err(StatusCode::FORBIDDEN);}
        *pairing=None;Ok(self.phone_credential.to_string())
    }
    /// In-process Windows controller boundary. Never exposed as an HTTP route.
    pub async fn attach_local_pc(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!*self.stopping.borrow(),"server stopping");
        // Reserve only after both waits succeed. disconnect releases its session
        // lock before acquiring connections, so this ordering has no lock cycle.
        let mut slots=self.connections.lock().await;
        anyhow::ensure!(!*self.stopping.borrow(),"server stopping");
        anyhow::ensure!(!slots.contains("pc"),"PC already attached");
        let mut s = self.session.lock().await;
        anyhow::ensure!(!*self.stopping.borrow(),"server stopping");
        s.connection("pc", true, self.now()).map_err(anyhow::Error::msg)?;
        slots.insert("pc".into());self.publish(&s); Ok(())
    }
    pub async fn detach_local_pc(&self) { disconnect(self,"pc").await; }
    pub async fn local_pc(&self, generation: Option<u64>, action: Action) -> Result<(), &'static str> {
        let mut s = self.session.lock().await;
        let (kind,position_ms)=action.diagnostic_fields();
        let base = s.state.generation;
        let result = if *self.stopping.borrow() {Err("server stopping")}
            else if generation.is_some_and(|g| g != base) {Err("stale generation")} else {match action {
            Action::Load { media } => match media.validate() {
                Ok(())=>{self.media.lock().map_err(|_|"media unavailable")?.clear();s.load(media,self.now())},
                Err(code)=>Err(code),
            },
            Action::Play => s.intent(true,base,self.now()),
            Action::Pause => s.intent(false,base,self.now()),
            Action::Seek { position_ms } if position_ms <= 604_800_000 => s.seek(position_ms,base,self.now()),
            Action::Seek { .. } => Err("position out of range"),
            Action::Ready { position_ms } => s.ready("pc",base,position_ms,self.now()),
            Action::Buffering { active } => s.buffering("pc",base,active,self.now()),
            Action::Ended {position_ms,duration_ms} => s.end_of_media("pc",base,position_ms,duration_ms,self.now()),
            Action::Executed {playing,position_ms,observed_server_ms} => s.executed("pc",base,playing,position_ms,observed_server_ms,self.now()),
            Action::PlayerFailed {code} => s.player_failed("pc",base,code,self.now()),
            Action::PlayerRecovered => s.player_recovered("pc",base,self.now()),
        }};
        self.record_command(diagnostics::CommandSample {server_ms:self.now(),actor:diagnostics::Actor::Pc,
            kind,position_ms,sequence:None,generation:generation.unwrap_or(base),
            resulting_generation:s.state.generation,revision:s.state.revision,
            outcome:diagnostics::CommandOutcome::from_error(result.err())});
        if result.is_ok() { self.publish(&s); }
        result
    }
    /// Local application setup only. Never accepts a remote path or creates an HTTP route.
    pub fn enable_diagnostics(&self,directory:std::path::PathBuf)->Result<(), &'static str> {
        let _guard=self.diagnostics_setup.lock().map_err(|_|"diagnostics_setup_unavailable")?;
        if self.diagnostics.get().is_some() {return Err("diagnostics_already_enabled");}
        let journal=diagnostics::Journal::start(directory).map_err(|_|"diagnostics_worker_unavailable")?;
        self.diagnostics.set(journal).map_err(|_|"diagnostics_already_enabled")
    }
    pub fn diagnostic_health(&self)->Option<diagnostics::Health> {self.diagnostics.get().map(|j|j.health())}
    pub fn record_native(&self,sample:diagnostics::NativeSample) {
        if let Some(journal)=self.diagnostics.get() {journal.record(diagnostics::Event::NativePc(sample));}
    }
    pub fn record_source(&self,selection:u64,generation:u64,refresh:bool,stage:diagnostics::SourceStage,outcome:diagnostics::SourceOutcome,elapsed_ms:u64) {
        if let Some(journal)=self.diagnostics.get() {journal.record(diagnostics::Event::Source(diagnostics::SourceSample {
            server_ms:self.now(),selection,generation,refresh,stage,outcome,elapsed_ms}));}
    }
    pub(crate) fn record_tunnel(&self,status:tunnel::TunnelStatus) {
        if let Some(journal)=self.diagnostics.get() {
            journal.record(diagnostics::Event::Tunnel(diagnostics::TunnelSample {server_ms:self.now(),status}));
        }
    }
    /// Local app lifecycle only; fixed enums cannot carry secret/error text.
    pub fn record_app_lifecycle(&self,kind:diagnostics::AppLifecycleKind) {
        if let Some(journal)=self.diagnostics.get() {journal.record(diagnostics::Event::AppLifecycle(diagnostics::AppLifecycleSample {server_ms:self.now(),kind}));}
    }
    pub async fn flush_diagnostics(&self)->Result<(), &'static str> {
        self.diagnostics.get().ok_or("diagnostics_unavailable")?.flush().await
    }
    fn record_command(&self,sample:diagnostics::CommandSample) {
        if let Some(journal)=self.diagnostics.get() {journal.record(diagnostics::Event::Command(sample));}
    }
    fn record_access(&self,endpoint:diagnostics::AccessEndpoint,kind:diagnostics::AccessKind) {
        if let Some(journal)=self.diagnostics.get() {
            journal.record(diagnostics::Event::Access(diagnostics::AccessSample {server_ms:self.now(),endpoint,kind}));
        }
    }
    fn record_control(&self,role:&str,kind:diagnostics::ControlKind) {
        if let Some(journal)=self.diagnostics.get() {
            journal.record(diagnostics::Event::Control(diagnostics::ControlSample {server_ms:self.now(),
                actor:if role=="pc" {diagnostics::Actor::Pc} else {diagnostics::Actor::Phone},kind}));
        }
    }
    pub async fn export_diagnostics(&self)->Result<Vec<u8>, &'static str> {
        self.diagnostics.get().ok_or("diagnostics_unavailable")?.export().await
    }
    fn publish(&self, s: &Session) {
        if let Ok(mut slot)=self.media.lock() {slot.observe(&s.state);}
        if let Some(journal)=self.diagnostics.get() {
            journal.record(diagnostics::Event::Session(Box::new(diagnostics::SessionSample::from_snapshot(&s.state,self.now()))));
        }
        let _ = self.updates.send(s.state.clone());
    }
    pub fn router(self: Arc<Self>) -> Router {
        Router::new().route("/health", get(health))
            .route("/session", get(upgrade)).route("/pair",post(pair)).route("/media",get(phone_media)).route("/updates/android",get(phone_update)).route("/diagnostics/phone",post(phone_logs).layer(DefaultBodyLimit::max(16384)))
            .layer(DefaultBodyLimit::max(4096)).with_state(self)
    }
    pub async fn serve(self: Arc<Self>, listener: tokio::net::TcpListener) -> anyhow::Result<()> {
        anyhow::ensure!(!*self.stopping.borrow(),"Host stopped; create a fresh Host");
        anyhow::ensure!(self.serve_started.compare_exchange(false,true,std::sync::atomic::Ordering::AcqRel,std::sync::atomic::Ordering::Acquire).is_ok(),"Host listener already started; create a fresh Host");
        let mut stop_listener=self.stopping.subscribe();
        let host = self.clone();
        let mut stop_ticker=self.stopping.subscribe();
        let ticker = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
            loop {
                if *stop_ticker.borrow() {break;}
                tokio::select! {biased; _=stop_ticker.changed()=>break, _=interval.tick()=>{}}
                let mut s = host.session.lock().await;
                if *stop_ticker.borrow() {break;}
                let rev = s.state.revision; s.tick(host.now()); if s.state.revision != rev { host.publish(&s); }
            }
        });
        let mut lifetime=ServerLifetime {ticker:Some(ticker),stopping:self.stopping.clone()};
        let result = axum::serve(listener, self.router()).with_graceful_shutdown(async move {
            // Subscribe-before-check also covers shutdown racing listener setup.
            if !*stop_listener.borrow_and_update() {let _=stop_listener.changed().await;}
        }).await;
        // Normal return joins the ticker; cancellation still uses the Drop
        // guard so a dropped serve future cannot retain Host indefinitely.
        lifetime.stopping.send_replace(true);
        if let Some(ticker)=lifetime.ticker.take() {ticker.abort();let _=ticker.await;}
        result?; Ok(())
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PairRequest {code:String}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MediaRequest {epoch:String,generation:u64}
async fn phone_media(State(host):State<Arc<Host>>,headers:HeaderMap,axum::extract::Query(request):axum::extract::Query<MediaRequest>)->axum::response::Response {
    if *host.stopping.borrow() {return StatusCode::SERVICE_UNAVAILABLE.into_response();}
    if !host.upgrades.lock().await.take(host.now()) {return StatusCode::TOO_MANY_REQUESTS.into_response();}
    let Some(token)=headers.get("authorization").and_then(|h|h.to_str().ok()).and_then(|s|s.strip_prefix("Bearer ")) else {return StatusCode::UNAUTHORIZED.into_response();};
    let digest:[u8;32]=Sha256::digest(token.as_bytes()).into();
    if !bool::from(digest.ct_eq(&host.phone_token)) {return StatusCode::UNAUTHORIZED.into_response();}
    let state=host.snapshot().await;
    if request.epoch!=state.epoch || request.generation!=state.generation {return StatusCode::CONFLICT.into_response();}
    let Some(grant)=host.media_grant(&state,media::Device::Phone) else {return StatusCode::SERVICE_UNAVAILABLE.into_response();};
    let source_revision=grant.source_revision();
    match grant.deliver(|url|json!({"protocol":PROTOCOL_VERSION,"epoch":state.epoch,"generation":state.generation,
        "source_revision":source_revision,"media":state.media,"url":url})) {
        Ok(body)=>([("cache-control","no-store"),("pragma","no-cache")],Json(body)).into_response(),
        Err(_)=>StatusCode::CONFLICT.into_response(),
    }
}
async fn phone_logs(State(host):State<Arc<Host>>,headers:HeaderMap,Json(batch):Json<phone_logs::Batch>)->axum::response::Response {
    if *host.stopping.borrow() {return StatusCode::SERVICE_UNAVAILABLE.into_response();}
    if !host.upgrades.lock().await.take(host.now()) {return StatusCode::TOO_MANY_REQUESTS.into_response();}
    let Some(token)=headers.get("authorization").and_then(|h|h.to_str().ok()).and_then(|s|s.strip_prefix("Bearer ")) else {return StatusCode::UNAUTHORIZED.into_response();};
    let digest:[u8;32]=Sha256::digest(token.as_bytes()).into();
    if !bool::from(digest.ct_eq(&host.phone_token)) {return StatusCode::UNAUTHORIZED.into_response();}
    if !batch.valid() {return StatusCode::BAD_REQUEST.into_response();}
    let Some(journal)=host.diagnostics.get() else {return StatusCode::SERVICE_UNAVAILABLE.into_response();};
    let Ok(fingerprint)=batch.fingerprint() else {return StatusCode::INTERNAL_SERVER_ERROR.into_response();};
    let mut previous=host.phone_log_batch.lock().await;
    match previous.check(batch.batch_id,fingerprint) {
        phone_logs::RetryCheck::Same=>return StatusCode::NO_CONTENT.into_response(),
        phone_logs::RetryCheck::Conflict=>return StatusCode::CONFLICT.into_response(),
        phone_logs::RetryCheck::New=>{},
    }
    let batch_id=batch.batch_id;
    if journal.phone_batch(batch).await.is_err() {return StatusCode::SERVICE_UNAVAILABLE.into_response();}
    previous.committed(batch_id,fingerprint);StatusCode::NO_CONTENT.into_response()
}
async fn phone_update(State(host):State<Arc<Host>>,headers:HeaderMap)->axum::response::Response {
    if *host.stopping.borrow() {return StatusCode::SERVICE_UNAVAILABLE.into_response();}
    if !host.upgrades.lock().await.take(host.now()) {return StatusCode::TOO_MANY_REQUESTS.into_response();}
    let Some(token)=headers.get("authorization").and_then(|h|h.to_str().ok()).and_then(|s|s.strip_prefix("Bearer ")) else {return StatusCode::UNAUTHORIZED.into_response();};
    let digest:[u8;32]=Sha256::digest(token.as_bytes()).into();
    if !bool::from(digest.ct_eq(&host.phone_token)) {return StatusCode::UNAUTHORIZED.into_response();}
    let Some(path)=host.android_release.get().cloned() else {return StatusCode::SERVICE_UNAVAILABLE.into_response();};
    match tokio::task::spawn_blocking(move ||android_update::Release::read(&path)).await {
        Ok(Ok(release))=>([("cache-control","no-store")],Json(release)).into_response(),
        _=>StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}
async fn health(State(host):State<Arc<Host>>)->axum::response::Response {
    if *host.stopping.borrow() {StatusCode::SERVICE_UNAVAILABLE.into_response()}
    else {"together-control-v1".into_response()}
}
async fn pair(State(host):State<Arc<Host>>,Json(request):Json<PairRequest>)->axum::response::Response {
    if *host.stopping.borrow() {return StatusCode::SERVICE_UNAVAILABLE.into_response();}
    if !host.upgrades.lock().await.take(host.now()) {return StatusCode::TOO_MANY_REQUESTS.into_response();}
    match host.claim_pairing(&request.code).await {
        Ok(token)=>([("cache-control","no-store")],Json(json!({"protocol":PROTOCOL_VERSION,"token":token}))).into_response(),
        Err(status)=>{
            host.record_access(diagnostics::AccessEndpoint::Pair,diagnostics::AccessKind::PairingRejected);
            status.into_response()
        },
    }
}
fn reject_authentication(host:&Host)->axum::response::Response {
    host.record_access(diagnostics::AccessEndpoint::Session,diagnostics::AccessKind::AuthenticationRejected);
    StatusCode::UNAUTHORIZED.into_response()
}
async fn upgrade(State(host): State<Arc<Host>>, headers: HeaderMap, ws: WebSocketUpgrade) -> axum::response::Response {
    if *host.stopping.borrow() {return StatusCode::SERVICE_UNAVAILABLE.into_response();}
    if !host.upgrades.lock().await.take(host.now()) {return StatusCode::TOO_MANY_REQUESTS.into_response();}
    let Some(token) = headers.get("authorization").and_then(|h| h.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")) else { return reject_authentication(&host); };
    if token.len()>256 {return reject_authentication(&host);}
    let digest:[u8;32]=Sha256::digest(token.as_bytes()).into();
    let pc=bool::from(digest.ct_eq(&host.pc_token));let phone=bool::from(digest.ct_eq(&host.phone_token));
    let role = if pc { "pc" } else if phone { "phone" } else { return reject_authentication(&host); };
    let mut slots = host.connections.lock().await;
    if *host.stopping.borrow() {return StatusCode::SERVICE_UNAVAILABLE.into_response();}
    if !slots.insert(role.into()) {
        host.record_access(diagnostics::AccessEndpoint::Session,diagnostics::AccessKind::RoleInUse);
        return StatusCode::CONFLICT.into_response();
    }
    drop(slots);
    let role = role.to_string();
    let failure_host = host.clone(); let failure_role = role.clone();
    ws.max_message_size(16 * 1024).max_frame_size(16 * 1024)
        .on_failed_upgrade(move |_| { tokio::spawn(async move { failure_host.connections.lock().await.remove(&failure_role); }); })
        .on_upgrade(move |socket| client(host, role, socket)).into_response()
}
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Clock { protocol: u32, t0_ms: u64 },
    Command { protocol: u32, epoch: String, seq: u64, generation: u64, action: Action },
}
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Load { media: MediaIdentity },
    Play,
    Pause,
    Seek { position_ms: u64 },
    Ready { position_ms: u64 },
    Buffering { active: bool },
    Ended { position_ms:u64, duration_ms:u64 },
    Executed { playing:bool, position_ms:u64, observed_server_ms:u64 },
    PlayerFailed {code:PlayerFault},
    PlayerRecovered,
}
impl Action {
    fn diagnostic_fields(&self)->(diagnostics::CommandKind,Option<u64>) {
        use diagnostics::CommandKind as K;
        match self {
            Self::Load {..}=>(K::Load,None),Self::Play=>(K::Play,None),Self::Pause=>(K::Pause,None),
            Self::Seek {position_ms}=>(K::Seek,Some(*position_ms)),Self::Ready {position_ms}=>(K::Ready,Some(*position_ms)),
            Self::Buffering {..}=>(K::Buffering,None),Self::Ended {position_ms,..}=>(K::Ended,Some(*position_ms)),
            Self::Executed {position_ms,..}=>(K::Executed,Some(*position_ms)),
            Self::PlayerFailed {..}=>(K::PlayerFailed,None),Self::PlayerRecovered=>(K::PlayerRecovered,None),
        }
    }
}
const CONTROL_WRITE_TIMEOUT:std::time::Duration=std::time::Duration::from_secs(5);
#[derive(Debug,PartialEq,Eq)]
enum SendOutcome {Sent,Stopped,Failed,TimedOut}
async fn send_frame(host:&Host,role:&str,socket:&mut WebSocket,frame:Message,stopping:&mut tokio::sync::watch::Receiver<bool>)->bool {
    let outcome=send_until_stopped(stopping,socket.send(frame)).await;
    if outcome==SendOutcome::TimedOut {host.record_control(role,diagnostics::ControlKind::WriteTimeout);}
    outcome==SendOutcome::Sent
}
async fn send_until_stopped<F,E>(stopping:&mut tokio::sync::watch::Receiver<bool>,send:F)->SendOutcome
where F:std::future::Future<Output=Result<(),E>> {
    if *stopping.borrow() {return SendOutcome::Stopped;}
    // A socket write can stall before recv/heartbeat is polled again. Bound it
    // independently so cleanup freezes authority and releases the reserved role.
    // Shutdown retains priority and never waits for this write deadline.
    tokio::select! {biased;
        _=stopping.changed()=>SendOutcome::Stopped,
        result=tokio::time::timeout(CONTROL_WRITE_TIMEOUT,send)=>match result {
            Ok(Ok(()))=>SendOutcome::Sent,Ok(Err(_))=>SendOutcome::Failed,Err(_)=>SendOutcome::TimedOut,
        },
    }
}
async fn send(host:&Host,role:&str,socket: &mut WebSocket, value: serde_json::Value,stopping:&mut tokio::sync::watch::Receiver<bool>) -> bool {
    send_frame(host,role,socket,Message::Text(value.to_string().into()),stopping).await
}
async fn finish_client(host:&Host,role:&str,socket:&mut WebSocket,stopping:bool) {
    // Freeze authority/release the role before waiting on a possibly blocked socket.
    host.record_control(role,if stopping {diagnostics::ControlKind::ServerStopping} else {diagnostics::ControlKind::SocketDisconnected});
    disconnect(host,role).await;
    if stopping {
        let _=tokio::time::timeout(std::time::Duration::from_millis(250),socket.send(Message::Close(Some(axum::extract::ws::CloseFrame {code:1012,reason:"server_restart".into()})))).await;
    }
}
async fn client(host: Arc<Host>, role: String, mut socket: WebSocket) {
    let mut stopping=host.stopping.subscribe();
    if *stopping.borrow() {finish_client(&host,&role,&mut socket,true).await;return;}
    let mut updates = host.updates.subscribe();
    let joined={let mut s=host.session.lock().await;
        if *stopping.borrow() {false} else {
            let _=s.connection(&role,true,host.now());
            host.record_control(&role,diagnostics::ControlKind::SocketConnected);host.publish(&s);true
        }
    };
    if !joined {finish_client(&host,&role,&mut socket,true).await;return;}
    if !send(&host,&role,&mut socket, json!({"type":"snapshot", "server_ms":host.now(), "state":host.snapshot().await}),&mut stopping).await {
        let stopped=*stopping.borrow();finish_client(&host, &role,&mut socket,stopped).await; return;
    }
    let mut seq = 0;
    let mut last_seen = Instant::now();
    let mut outstanding_ping: Option<u64> = None;
    let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(2));
    let mut frames=limits::Bucket::new(80,40,host.now());
    loop {
        tokio::select! {
            _=stopping.changed()=>break,
            frame = socket.recv() => {
                let Some(Ok(frame)) = frame else { break; };
                if *stopping.borrow() {break;}
                if !frames.take(host.now()) {
                    host.record_control(&role,diagnostics::ControlKind::RateLimited);
                    let _=send_frame(&host,&role,&mut socket,Message::Close(Some(axum::extract::ws::CloseFrame {code:1013,reason:"control_rate_limit".into()})),&mut stopping).await;
                    break;
                }
                last_seen = Instant::now();
                match frame {
                    Message::Close(_) => break,
                    Message::Ping(v) => { if !send_frame(&host,&role,&mut socket,Message::Pong(v),&mut stopping).await { break; } },
                    Message::Pong(v) => {
                        if let Ok(bytes)=<[u8;8]>::try_from(v.as_ref()) {
                            let sent=u64::from_be_bytes(bytes);
                            if outstanding_ping==Some(sent) {
                                outstanding_ping=None;
                                let mut s=host.session.lock().await;
                                if s.observe_rtt(&role,host.now().saturating_sub(sent)).is_ok() {host.publish(&s);}
                            }
                        }
                    },
                    Message::Text(raw) => {
                        let t1 = host.now();
                        let request: Request = match serde_json::from_str(&raw) {
                            Ok(r) => r, Err(_) => { host.record_control(&role,diagnostics::ControlKind::InvalidRequest);
                                if !send(&host,&role,&mut socket,json!({"type":"error","code":"invalid_request"}),&mut stopping).await { break; } continue; }
                        };
                        match request {
                            Request::Clock { protocol, t0_ms } => {
                                if protocol != PROTOCOL_VERSION {host.record_control(&role,diagnostics::ControlKind::UnsupportedProtocol);break;}
                                if !send(&host,&role,&mut socket, json!({"type":"clock","protocol":PROTOCOL_VERSION,"epoch":host.snapshot().await.epoch,"t0_ms":t0_ms,"t1_ms":t1,"t2_ms":host.now()}),&mut stopping).await { break; }
                            },
                            Request::Command { protocol, epoch, seq: n, generation, action } => {
                                let mut s = host.session.lock().await;
                                let (kind,position_ms)=action.diagnostic_fields();
                                let result = if *host.stopping.borrow() {Err("server stopping")}
                                    else if protocol != PROTOCOL_VERSION { Err("unsupported protocol") }
                                    else if epoch != s.state.epoch { Err("stale epoch") }
                                    else if n <= seq { Err("duplicate sequence") }
                                    else if generation != s.state.generation { Err("stale generation") }
                                    else {
                                        seq = n;
                                        match action {
                                            Action::Load { media } if role == "pc" => match media.validate() {
                                                Ok(())=>{if let Ok(mut slot)=host.media.lock() {slot.clear();}s.load(media, host.now())},
                                                Err(code)=>Err(code),
                                            },
                                            Action::Load { .. } => Err("host only"),
                                            Action::Play => s.intent(true, generation, host.now()),
                                            Action::Pause => s.intent(false, generation, host.now()),
                                            Action::Seek { position_ms } if position_ms <= 604_800_000 => s.seek(position_ms, generation, host.now()),
                                            Action::Seek { .. } => Err("position out of range"),
                                            Action::Ready { position_ms } => s.ready(&role, generation, position_ms, host.now()),
                                            Action::Buffering { active } => s.buffering(&role, generation, active, host.now()),
                                            Action::Ended {position_ms,duration_ms} => s.end_of_media(&role,generation,position_ms,duration_ms,host.now()),
                                            Action::Executed {playing,position_ms,observed_server_ms} => s.executed(&role,generation,playing,position_ms,observed_server_ms,host.now()),
                                            Action::PlayerFailed {code} => s.player_failed(&role,generation,code,host.now()),
                                            Action::PlayerRecovered => s.player_recovered(&role,generation,host.now()),
                                        }
                                    };
                                let code = result.err();
                                host.record_command(diagnostics::CommandSample {server_ms:host.now(),
                                    actor:if role=="pc" {diagnostics::Actor::Pc} else {diagnostics::Actor::Phone},
                                    kind,position_ms,sequence:Some(n),generation,resulting_generation:s.state.generation,
                                    revision:s.state.revision,outcome:diagnostics::CommandOutcome::from_error(code)});
                                if code.is_none() { host.publish(&s); }
                                let reply = json!({"type":"ack", "seq":n,"error":code,"server_ms":host.now(),"state":s.state});
                                tracing::info!(event="command", participant=%role, generation=s.state.generation, revision=s.state.revision, accepted=code.is_none());
                                drop(s);
                                if !send(&host,&role,&mut socket, reply,&mut stopping).await { break; }
                            }
                        }
                    },
                    _ => {}
                }
            },
            update = updates.recv() => {
                let state = match update { Ok(s) => s, Err(broadcast::error::RecvError::Lagged(_)) => host.snapshot().await, Err(_) => break };
                if !send(&host,&role,&mut socket,json!({"type":"snapshot","server_ms":host.now(),"state":state}),&mut stopping).await { break; }
            },
            _ = heartbeat.tick() => {
                if last_seen.elapsed().as_secs() >= 6 {host.record_control(&role,diagnostics::ControlKind::HeartbeatTimeout);break;}
                let stamp=host.now();
                // Preserve a valid delayed pong across heartbeat ticks. Replacing
                // the nonce every two seconds made supported 2-5s RTTs invisible.
                if !outstanding_ping.is_some_and(|sent|stamp.saturating_sub(sent)<=5000) {
                    outstanding_ping=Some(stamp);
                    if !send_frame(&host,&role,&mut socket,Message::Ping(stamp.to_be_bytes().to_vec().into()),&mut stopping).await { break; }
                }
            }
        }
    }
    let stopped=*stopping.borrow();finish_client(&host,&role,&mut socket,stopped).await;
}
async fn disconnect(host: &Host, role: &str) {
    // Keep the role reserved until the old connection has frozen the timeline.
    { let mut s = host.session.lock().await; let _ = s.connection(role, false, host.now()); host.publish(&s); }
    host.connections.lock().await.remove(role);
    tracing::info!(event="disconnect", participant=%role);
}
#[cfg(test)] mod pairing_tests {
    #[test] fn host_rejects_header_unsafe_credentials_without_echoing_them() {
        for invalid in ["x".repeat(31), "x".repeat(257), " ".repeat(32),
            format!("{}\r\nPRIVATE_TOKEN", "x".repeat(32)), format!("{}\t", "x".repeat(32)),
            format!("{}\0", "x".repeat(32)), format!("{}\u{7f}", "x".repeat(32)), "é".repeat(32)] {
            for phone in [false,true] {
                let (pc,phone)=if phone {("p".repeat(32),invalid.clone())} else {(invalid.clone(),"f".repeat(32))};
                let error=match super::Host::new(pc,phone) {Ok(_)=>panic!("unsafe credential accepted"),Err(error)=>error};
                assert_eq!(error.to_string(),"two distinct pairing tokens of 32 to 256 visible ASCII characters are required");
            }
        }
        for length in [32,256] {
            assert!(super::Host::new("!".repeat(length),"~".repeat(length)).is_ok());
        }
        assert!(super::Host::new("p".repeat(32),"p".repeat(32)).is_err());
    }
    use super::*;
    fn host()->Arc<Host> {Host::new("pc-test-credential-00000000000000000".into(),"phone-test-credential-00000000000000000".into()).unwrap()}
    #[tokio::test] async fn stopped_host_does_not_advertise_health_or_issue_pairing_credentials() {
        let h=host();let code=h.open_pairing().await.unwrap();
        assert_eq!(health(State(h.clone())).await.status(),StatusCode::OK);
        h.stopping.send_replace(true);
        assert_eq!(health(State(h.clone())).await.status(),StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(pair(State(h.clone()),Json(PairRequest {code})).await.status(),StatusCode::SERVICE_UNAVAILABLE);
        assert!(h.pairing.lock().await.is_some(),"stopped endpoint must not consume pairing code");
        assert!(h.attach_local_pc().await.is_err());
    }
    async fn assert_pending<F:std::future::Future>(future:std::pin::Pin<&mut F>) {
        let mut future=future;
        assert!(std::future::poll_fn(|cx|std::task::Poll::Ready(future.as_mut().poll(cx))).await.is_pending());
    }
    #[tokio::test] async fn shutdown_waiter_handles_pending_already_stopped_and_cancelled_waiters() {
        let h=host();
        let mut cancelled=Box::pin(h.shutdown_requested());assert_pending(cancelled.as_mut()).await;
        drop(cancelled);
        let mut first=Box::pin(h.shutdown_requested());let mut second=Box::pin(h.shutdown_requested());
        assert_pending(first.as_mut()).await;assert_pending(second.as_mut()).await;
        h.request_shutdown();
        tokio::time::timeout(std::time::Duration::from_secs(1),first).await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(1),second).await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(1),h.shutdown_requested()).await.unwrap();
    }
    #[tokio::test] async fn shutdown_during_pairing_lock_wait_does_not_issue_or_consume_credentials() {
        let h=host();let code=h.open_pairing().await.unwrap();
        let pairing=h.pairing.lock().await;
        let mut claim=Box::pin(h.claim_pairing(&code));let mut open=Box::pin(h.open_pairing());
        assert_pending(claim.as_mut()).await;assert_pending(open.as_mut()).await;
        h.request_shutdown();drop(pairing);
        assert_eq!(claim.await,Err(StatusCode::SERVICE_UNAVAILABLE));assert_eq!(open.await,Err("server stopping"));
        let pairing=h.pairing.lock().await;assert_eq!(pairing.as_ref().unwrap().attempts,0);
        let expected:[u8;32]=Sha256::digest(code.replace('-',"").as_bytes()).into();
        assert!(bool::from(pairing.as_ref().unwrap().digest.ct_eq(&expected)));
    }
    #[tokio::test] async fn shutdown_during_local_attach_lock_wait_never_reserves_or_connects_pc() {
        // Exercise both awaited locks, with a polled pending operation before stop.
        for session_lock in [false,true] {
            let h=host();let before=h.snapshot().await;
            let slots=if session_lock {None}else{Some(h.connections.lock().await)};
            let session=if session_lock {Some(h.session.lock().await)}else{None};
            let mut attach=Box::pin(h.attach_local_pc());assert_pending(attach.as_mut()).await;
            h.request_shutdown();drop(slots);drop(session);assert!(attach.await.is_err());
            assert!(h.connections.lock().await.is_empty());let after=h.snapshot().await;
            assert!(!after.participants["pc"].connected);assert_eq!(after.revision,before.revision);
            assert_eq!(after.generation,before.generation);
        }
    }
    #[tokio::test] async fn shutdown_before_serve_is_idempotent_and_rejects_listener_start() {
        let h=host();h.request_shutdown();h.request_shutdown();
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address=listener.local_addr().unwrap();assert!(h.clone().serve(listener).await.is_err());
        // Rejected startup releases its listener without starting a ticker.
        let replacement=tokio::net::TcpListener::bind(address).await.unwrap();drop(replacement);
        assert!(!h.serve_started.load(std::sync::atomic::Ordering::Acquire));
        assert!(h.attach_local_pc().await.is_err());assert_eq!(h.local_pc(None,Action::Play).await,Err("server stopping"));
    }
    #[tokio::test] async fn shutdown_cancels_pending_write_and_does_not_start_new_write() {
        let (sender,mut receiver)=tokio::sync::watch::channel(false);
        let pending=tokio::spawn(async move {send_until_stopped(&mut receiver,std::future::pending::<Result<(),()>>()).await});
        tokio::task::yield_now().await;sender.send_replace(true);
        assert_eq!(tokio::time::timeout(std::time::Duration::from_secs(1),pending).await.unwrap().unwrap(),SendOutcome::Stopped);
        let mut stopped=sender.subscribe();
        let unexpected=std::future::poll_fn(|_|->std::task::Poll<Result<(),()>> {panic!("stopped connection attempted write")});
        assert_eq!(send_until_stopped(&mut stopped,unexpected).await,SendOutcome::Stopped);
    }
    #[tokio::test(start_paused=true)] async fn stalled_control_write_expires_and_drops_the_pending_send() {
        struct Dropped(Arc<std::sync::atomic::AtomicBool>);
        impl Drop for Dropped {fn drop(&mut self) {self.0.store(true,std::sync::atomic::Ordering::SeqCst);}}
        let (_sender,mut receiver)=tokio::sync::watch::channel(false);
        let dropped=Arc::new(std::sync::atomic::AtomicBool::new(false));
        let guard=Dropped(dropped.clone());
        let never=async move {let _guard=guard;std::future::pending::<Result<(),()>>().await};
        let mut pending=Box::pin(send_until_stopped(&mut receiver,never));assert_pending(pending.as_mut()).await;
        tokio::time::advance(CONTROL_WRITE_TIMEOUT-std::time::Duration::from_millis(1)).await;
        assert_pending(pending.as_mut()).await;assert!(!dropped.load(std::sync::atomic::Ordering::SeqCst));
        tokio::time::advance(std::time::Duration::from_millis(1)).await;
        assert_eq!(pending.await,SendOutcome::TimedOut);assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(serde_json::to_string(&diagnostics::ControlKind::WriteTimeout).unwrap(),"\"write_timeout\"");
    }
    #[tokio::test(start_paused=true)] async fn completed_and_failed_writes_finish_immediately_and_stop_beats_expiry() {
        let (sender,mut receiver)=tokio::sync::watch::channel(false);
        assert_eq!(send_until_stopped(&mut receiver,std::future::ready(Ok::<(),()>(()))).await,SendOutcome::Sent);
        assert_eq!(send_until_stopped(&mut receiver,std::future::ready(Err::<(),()>(()))).await,SendOutcome::Failed);
        let mut pending=Box::pin(send_until_stopped(&mut receiver,std::future::pending::<Result<(),()>>()));
        assert_pending(pending.as_mut()).await;tokio::time::advance(CONTROL_WRITE_TIMEOUT).await;
        sender.send_replace(true);assert_eq!(pending.await,SendOutcome::Stopped);
    }
    #[tokio::test] async fn real_socket_backpressure_times_out_and_releases_phone_for_reconnect() {
        use futures_util::{SinkExt,StreamExt};
        use tokio_tungstenite::{client_async,tungstenite::{client::IntoClientRequest,Message as ClientMessage}};
        let directory=std::env::temp_dir().join(format!("together-backpressure-{}",uuid::Uuid::new_v4()));
        let h=host();h.enable_diagnostics(directory.clone()).unwrap();h.attach_local_pc().await.unwrap();
        h.local_pc(None,Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
        // Small real TCP buffers make backpressure reachable without large frames
        // or unbounded allocations. No test-only deadline/configuration override.
        let listener_socket=tokio::net::TcpSocket::new_v4().unwrap();
        listener_socket.set_send_buffer_size(4096).unwrap();
        listener_socket.bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let listener=listener_socket.listen(16).unwrap();let address=listener.local_addr().unwrap();
        let server_host=h.clone();let server=tokio::spawn(async move {server_host.serve(listener).await.unwrap();});
        let peer=tokio::net::TcpSocket::new_v4().unwrap();peer.set_recv_buffer_size(1024).unwrap();
        let tcp=peer.connect(address).await.unwrap();tcp.set_nodelay(true).unwrap();
        let mut request=format!("ws://{address}/session").into_client_request().unwrap();
        request.headers_mut().insert("authorization", "Bearer phone-test-credential-00000000000000000".parse().unwrap());
        let (mut phone,_)=client_async(request,tcp).await.unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(3),async {
            loop {if let Some(Ok(ClientMessage::Text(raw)))=phone.next().await {
                if serde_json::from_str::<serde_json::Value>(&raw).unwrap()["type"]=="snapshot" {break;}
            }}
        }).await.unwrap();
        let joined=h.snapshot().await;assert!(joined.participants["phone"].connected);
        // Synthetic native readiness/execution starts authority before the
        // transport stalls. This does not instantiate either native player.
        h.local_pc(None,Action::Play).await.unwrap();
        h.local_pc(None,Action::Ready {position_ms:0}).await.unwrap();
        {let mut s=h.session.lock().await;let generation=s.state.generation;
            s.ready("phone",generation,0,h.now()).unwrap();h.publish(&s);
        }
        let commit=h.snapshot().await;
        tokio::time::sleep(std::time::Duration::from_millis(commit.timeline.anchor_ms.saturating_sub(h.now())+40)).await;
        {let mut s=h.session.lock().await;let generation=s.state.generation;let now=h.now();
            let position=s.state.timeline.position(now);
            for role in ["pc","phone"] {s.executed(role,generation,true,position,now,now).unwrap();}
            h.publish(&s);
        }
        let active=h.snapshot().await;assert!(active.timeline.playing);assert!(active.prepare.is_none());
        let publish_host=h.clone();
        let flood=tokio::spawn(async move {
            // Simulate frequent server measurements, retaining bounded broadcast
            // storage. Native user actions/seek generations are not spammed.
            for index in 0..30_000 {
                {let mut s=publish_host.session.lock().await;
                    if !s.state.participants["phone"].connected {break;}
                    s.observe_rtt("pc",index%2).unwrap();publish_host.publish(&s);
                }
                tokio::task::yield_now().await;
            }
        });
        tokio::time::timeout(std::time::Duration::from_secs(15),async {
            loop {
                if !h.snapshot().await.participants["phone"].connected {break;}
                // Keep upstream traffic alive while deliberately never reading
                // downstream. An idle-peer timeout must not stand in for the write deadline.
                phone.send(ClientMessage::Pong(vec![0;8].into())).await.unwrap();
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }).await.unwrap();
        flood.await.unwrap();drop(phone);
        let stopped=h.snapshot().await;
        assert!(!stopped.timeline.playing);assert!(stopped.generation>active.generation);
        assert!(stopped.timeline.position_ms>active.timeline.position_ms);assert!(stopped.fault.is_none());
        let text=String::from_utf8(h.export_diagnostics().await.unwrap()).unwrap();
        assert!(text.lines().map(|s|serde_json::from_str::<serde_json::Value>(s).unwrap()).any(|r|
            r["record"]["event"]=="control" && r["record"]["sample"]["kind"]=="write_timeout"));
        let mut request=format!("ws://{address}/session").into_client_request().unwrap();
        request.headers_mut().insert("authorization", "Bearer phone-test-credential-00000000000000000".parse().unwrap());
        let (mut replacement,_)=tokio_tungstenite::connect_async(request).await.unwrap();
        let state=tokio::time::timeout(std::time::Duration::from_secs(3),async {
            loop {if let Some(Ok(ClientMessage::Text(raw)))=replacement.next().await {
                let frame:serde_json::Value=serde_json::from_str(&raw).unwrap();
                if frame["type"]=="snapshot" {break frame["state"].clone();}
            }}
        }).await.unwrap();
        assert_eq!(state["epoch"],joined.epoch);assert!(state["generation"].as_u64().unwrap()>stopped.generation);
        assert_eq!(state["participants"]["phone"]["connected"],true);
        replacement.close(None).await.unwrap();drop(replacement);h.request_shutdown();server.await.unwrap();drop(h);
        for _ in 0..100 {if std::fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
        assert!(!directory.exists());
    }
    #[tokio::test] async fn pairing_is_local_opened_one_use_expiring_and_attempt_limited() {
        let h=host();assert_eq!(h.claim_pairing("0000-0000-0000").await.unwrap_err(),StatusCode::FORBIDDEN);
        let code=h.open_pairing().await.unwrap();assert!(h.claim_pairing(&code.to_uppercase()).await.is_ok());
        assert_eq!(h.claim_pairing(&code).await.unwrap_err(),StatusCode::FORBIDDEN);
        let code=h.open_pairing().await.unwrap();
        for _ in 0..8 {assert_eq!(h.claim_pairing("invalid").await.unwrap_err(),StatusCode::FORBIDDEN);}
        assert_eq!(h.claim_pairing(&code).await.unwrap_err(),StatusCode::FORBIDDEN);
        let code=h.open_pairing().await.unwrap();h.pairing.lock().await.as_mut().unwrap().expires_ms=0;
        assert_eq!(h.claim_pairing(&code).await.unwrap_err(),StatusCode::FORBIDDEN);
        let code=h.open_pairing().await.unwrap();h.cancel_pairing().await;assert!(h.claim_pairing(&code).await.is_err());
    }
}

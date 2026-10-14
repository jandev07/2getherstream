//! Bounded, typed local diagnostics. No arbitrary strings, snapshots or URLs enter
//! the journal. Producers never wait for filesystem I/O or for queue capacity.
use serde::Serialize;
use std::{fs::{self, File}, io::{self, BufWriter, Write, Read}, path::PathBuf,
    sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}, mpsc}, time::Duration};
use together_sync_core::{Snapshot, MediaProvider, PlayerFault, Execution};

const QUEUE_CAPACITY: usize = 256;
const FILE_BYTES: u64 = 8 * 1024 * 1024;
const FILE_COUNT: usize = 8;

#[derive(Clone,Copy,Serialize)]
#[serde(rename_all="snake_case")]
pub enum Actor {Pc,Phone}
#[derive(Clone,Copy,Serialize)]
#[serde(rename_all="snake_case")]
pub enum ControlKind {SocketConnected,SocketDisconnected,ServerStopping,InvalidRequest,UnsupportedProtocol,RateLimited,HeartbeatTimeout,WriteTimeout}
#[derive(Serialize)]
pub struct ControlSample {pub server_ms:u64,pub actor:Actor,pub kind:ControlKind}
#[derive(Clone,Copy,Serialize)]
#[serde(rename_all="snake_case")]
pub enum AccessEndpoint {Session,Pair}
#[derive(Clone,Copy,Serialize)]
#[serde(rename_all="snake_case")]
pub enum AccessKind {AuthenticationRejected,PairingRejected,RoleInUse}
#[derive(Serialize)]
pub struct AccessSample {pub server_ms:u64,pub endpoint:AccessEndpoint,pub kind:AccessKind}
#[derive(Clone,Copy,Serialize)]
#[serde(rename_all="snake_case")]
pub enum CommandKind {Load,Play,Pause,Seek,Ready,Buffering,Ended,Executed,PlayerFailed,PlayerRecovered}
#[derive(Serialize)]
#[serde(rename_all="snake_case")]
pub enum CommandOutcome {
    Accepted,StaleGeneration,StaleEpoch,DuplicateSequence,UnsupportedProtocol,HostOnly,
    PositionOutOfRange,ParticipantUnavailable,NoMedia,NoPrepare,PrepareExpired,
    PositionNotReady,NoCommittedPlayback,ExecutionExpired,ExecutionTooEarly,
    ExecutionPositionMismatch,ExecutionStateMismatch,PrematureEnd,DurationMismatch,
    NotPlaying,ServerStopping,OtherRejected,
}
impl CommandOutcome {
    pub fn from_error(code:Option<&str>)->Self {match code {
        None=>Self::Accepted,Some("stale generation")=>Self::StaleGeneration,
        Some("server stopping")=>Self::ServerStopping,
        Some("stale epoch")=>Self::StaleEpoch,Some("duplicate sequence")=>Self::DuplicateSequence,
        Some("unsupported protocol")=>Self::UnsupportedProtocol,Some("host only")=>Self::HostOnly,
        Some("position out of range")=>Self::PositionOutOfRange,
        Some("participant not available"|"participant offline")=>Self::ParticipantUnavailable,
        Some("no media")=>Self::NoMedia,Some("no prepare")=>Self::NoPrepare,
        Some("prepare expired")=>Self::PrepareExpired,Some("player has not reached target")=>Self::PositionNotReady,
        Some("no committed playback")=>Self::NoCommittedPlayback,
        Some("execution observation expired")=>Self::ExecutionExpired,
        Some("execution before scheduled start")=>Self::ExecutionTooEarly,
        Some("execution position mismatch")=>Self::ExecutionPositionMismatch,
        Some("execution state mismatch")=>Self::ExecutionStateMismatch,
        Some("premature end report")=>Self::PrematureEnd,Some("media duration mismatch")=>Self::DurationMismatch,
        Some("not in active playback")=>Self::NotPlaying,Some(_)=>Self::OtherRejected,
    }}
}
#[derive(Serialize)]
pub struct CommandSample {
    pub server_ms:u64,pub actor:Actor,pub kind:CommandKind,pub sequence:Option<u64>,
    pub generation:u64,pub resulting_generation:u64,pub revision:u64,
    pub position_ms:Option<u64>,pub outcome:CommandOutcome,
}

#[derive(Clone,Copy,Serialize)]
#[serde(rename_all="snake_case")]
pub enum AppLifecycleKind {ShutdownRequested,TunnelStopped,TunnelStopFailed,PlayerStopped,PlayerStopFailed,ServerStopped,ServerStopFailed,ExitPermitted}
#[derive(Serialize)]
pub struct AppLifecycleSample {pub server_ms:u64,pub kind:AppLifecycleKind}

#[derive(Clone,Copy,Serialize)]
#[serde(rename_all="snake_case")]
pub enum SourceStage {CurrentFileAndLinks,DirectRanges,Publication}
#[derive(Clone,Copy,Serialize)]
#[serde(rename_all="snake_case")]
pub enum SourceOutcome {Started,Passed,Failed,Replaced}
#[derive(Serialize)]
pub struct SourceSample {pub server_ms:u64,pub selection:u64,pub generation:u64,pub refresh:bool,
    pub stage:SourceStage,pub outcome:SourceOutcome,pub elapsed_ms:u64}

#[derive(Serialize)]
pub struct TunnelSample {pub server_ms:u64,pub status:crate::tunnel::TunnelStatus}

#[derive(Serialize)]
pub struct NativeSample {
    pub server_ms:u64, pub generation:u64, pub position_ms:u64, pub target_ms:u64,
    pub paused:bool, pub seeking:bool, pub buffering:bool, pub source_ready:bool,
    pub rate:f64, pub drift_ms:i64,
}
#[derive(Serialize)]
struct ParticipantSample {
    connected:bool, buffering:bool, rtt_ms:u64, player_fault:Option<PlayerFault>,
    ready:bool, execution:Option<Execution>,
}
#[derive(Serialize)]
pub struct SessionSample {
    server_ms:u64, generation:u64, revision:u64, media_present:bool,
    provider:Option<MediaProvider>, torrent_id:Option<u64>, file_id:Option<u64>,
    desired_playing:bool, playing:bool, ended:bool, position_ms:u64, anchor_ms:u64,
    preparing:bool, prepare_position_ms:Option<u64>, prepare_deadline_ms:Option<u64>,
    fault:Option<&'static str>, pc:Option<ParticipantSample>, phone:Option<ParticipantSample>,
}
impl SessionSample {
    pub fn from_snapshot(s:&Snapshot, server_ms:u64)->Self {
        let participant=|id:&str| s.participants.get(id).map(|p|ParticipantSample {
            connected:p.connected, buffering:p.buffering, rtt_ms:p.rtt_ms,
            player_fault:p.player_fault, execution:p.execution.clone(),
            ready:s.prepare.as_ref().is_some_and(|p|p.ready.contains(id)),
        });
        // Even a future reducer fault containing unexpected text cannot leak it.
        let fault=s.fault.as_deref().map(|code|match code {
            "prepare_timeout"=>"prepare_timeout", "execution_timeout_pc"=>"execution_timeout_pc",
            "execution_timeout_phone"=>"execution_timeout_phone", "player_failed_pc"=>"player_failed_pc",
            "player_failed_phone"=>"player_failed_phone", _=>"unknown_fault",
        });
        Self {server_ms, generation:s.generation, revision:s.revision, media_present:s.media.is_some(),
            provider:s.media.as_ref().map(|m|m.provider.clone()), torrent_id:s.media.as_ref().map(|m|m.torrent_id),
            file_id:s.media.as_ref().map(|m|m.file_id), desired_playing:s.desired_playing,
            playing:s.timeline.playing, ended:s.ended, position_ms:s.timeline.position(server_ms),
            anchor_ms:s.timeline.anchor_ms, preparing:s.prepare.is_some(),
            prepare_position_ms:s.prepare.as_ref().map(|p|p.position_ms),
            prepare_deadline_ms:s.prepare.as_ref().map(|p|p.deadline_ms), fault,
            pc:participant("pc"),phone:participant("phone")}
    }
}
#[derive(Serialize)]
#[serde(tag="event",content="sample",rename_all="snake_case")]
pub enum Event { Session(Box<SessionSample>), NativePc(NativeSample), Phone(crate::phone_logs::Sample), Command(CommandSample), Control(ControlSample), Tunnel(TunnelSample), Access(AccessSample), AppLifecycle(AppLifecycleSample), Source(SourceSample) }
enum Command { Record(Event), PhoneBatch(Vec<crate::phone_logs::Sample>,tokio::sync::oneshot::Sender<Result<(), &'static str>>), Export(tokio::sync::oneshot::Sender<Result<Vec<u8>, &'static str>>), Flush(tokio::sync::oneshot::Sender<Result<(), &'static str>>) }
#[derive(Default)]
struct Counters { dropped:AtomicU64, io_errors:AtomicU64, active:AtomicBool }
#[derive(Serialize)]
pub struct Health {pub active:bool,pub dropped_records:u64,pub io_errors:u64}
pub struct Journal { sender:mpsc::SyncSender<Command>, counters:Arc<Counters> }
impl Journal {
    pub fn start(directory:PathBuf)->io::Result<Self> {
        Self::start_with_limits(directory,FILE_BYTES,FILE_COUNT)
    }
    fn start_with_limits(directory:PathBuf,file_bytes:u64,file_count:usize)->io::Result<Self> {
        let (sender,receiver)=mpsc::sync_channel(QUEUE_CAPACITY);
        let counters=Arc::new(Counters::default());let worker=counters.clone();
        worker.active.store(true,Ordering::Relaxed);
        std::thread::Builder::new().name("together-diagnostics".into()).spawn(move || {
            let Ok(mut storage)=Storage::open(directory,file_bytes,file_count) else {
                worker.io_errors.fetch_add(1,Ordering::Relaxed);worker.active.store(false,Ordering::Relaxed);return;
            };
            let run_id=uuid::Uuid::new_v4().to_string();let mut sequence=0u64;
            let mut flushed_at=std::time::Instant::now();
            loop {
                let result=match receiver.recv_timeout(Duration::from_millis(250)) {
                    Ok(Command::Record(event))=>{
                        storage.record(event,&run_id,&mut sequence,&worker)
                    },
                    Ok(Command::PhoneBatch(samples,reply))=>{
                        // One queue slot admits the whole bounded upload. Unrelated
                        // producer drops cannot invalidate its own write receipt.
                        let result=samples.into_iter().try_for_each(|sample|
                            storage.record(Event::Phone(sample),&run_id,&mut sequence,&worker))
                            .and_then(|_|storage.flush());
                        if result.is_err() {worker.io_errors.fetch_add(1,Ordering::Relaxed);}
                        // This receipt covers this batch only. Earlier unrelated
                        // errors stay in health/record metadata, but do not turn
                        // a successful current write+flush into a failed upload.
                        let acknowledged=if result.is_ok() {
                            Ok(())
                        } else {Err("diagnostic_batch_failed")};
                        let _=reply.send(acknowledged);Ok(())
                    },
                    Ok(Command::Export(reply))=>{
                        let result=storage.export();let failed=result.is_err();
                        let _=reply.send(result.map_err(|_|"diagnostic_export_failed"));
                        if failed {Err(io::Error::other("export failed"))} else {Ok(())}
                    },
                    Ok(Command::Flush(reply))=>{
                        let result=storage.flush();
                        if result.is_err() {worker.io_errors.fetch_add(1,Ordering::Relaxed);}
                        let acknowledged=if worker.io_errors.load(Ordering::Relaxed)==0 {Ok(())}else{Err("diagnostic_flush_failed")};
                        let _=reply.send(acknowledged);Ok(())
                    },
                    Err(mpsc::RecvTimeoutError::Timeout)=>storage.flush(),
                    Err(mpsc::RecvTimeoutError::Disconnected)=>{let _=storage.flush();break;},
                };
                if result.is_err() {worker.io_errors.fetch_add(1,Ordering::Relaxed);}
                // Continuous samples must not postpone flushing indefinitely.
                if flushed_at.elapsed()>=Duration::from_millis(250) {
                    if storage.flush().is_err() {worker.io_errors.fetch_add(1,Ordering::Relaxed);}
                    flushed_at=std::time::Instant::now();
                }
            }
            worker.active.store(false,Ordering::Relaxed);
        })?;
        Ok(Self {sender,counters})
    }
    pub fn record(&self,event:Event) {
        if self.sender.try_send(Command::Record(event)).is_err() {
            self.counters.dropped.fetch_add(1,Ordering::Relaxed);
        }
    }
    pub fn health(&self)->Health {Health {active:self.counters.active.load(Ordering::Relaxed),
        dropped_records:self.counters.dropped.load(Ordering::Relaxed),io_errors:self.counters.io_errors.load(Ordering::Relaxed)}}
    /// Admit all phone samples in one nonblocking queue operation, then acknowledge
    /// writes and userspace flush. Filesystem failure can still leave partial rows;
    /// a failure/timeout is never a success receipt or a power-loss guarantee.
    pub async fn phone_batch(&self,batch:crate::phone_logs::Batch)->Result<(), &'static str> {
        if !batch.valid() {return Err("diagnostic_batch_invalid");}
        let health=self.health();
        if !health.active {return Err("diagnostic_writer_unavailable");}
        let (reply,wait)=tokio::sync::oneshot::channel();
        self.sender.try_send(Command::PhoneBatch(batch.samples,reply)).map_err(|_|"diagnostic_queue_unavailable")?;
        tokio::time::timeout(Duration::from_secs(2),wait).await
            .map_err(|_|"diagnostic_batch_timeout")?.map_err(|_|"diagnostic_writer_unavailable")?
    }
    /// FIFO acknowledgement for accepted records preceding this request. Flush
    /// userspace buffers only; this is not fsync or a guarantee against power loss.
    /// Producers remain nonblocking, and a full queue fails without waiting.
    pub async fn flush(&self)->Result<(), &'static str> {
        let (reply,wait)=tokio::sync::oneshot::channel();
        self.sender.try_send(Command::Flush(reply)).map_err(|_|"diagnostic_queue_unavailable")?;
        tokio::time::timeout(Duration::from_secs(2),wait).await
            .map_err(|_|"diagnostic_flush_timeout")?.map_err(|_|"diagnostic_writer_unavailable")?
    }
    pub async fn export(&self)->Result<Vec<u8>, &'static str> {
        let (reply,wait)=tokio::sync::oneshot::channel();
        self.sender.try_send(Command::Export(reply)).map_err(|_|"diagnostic_queue_unavailable")?;
        tokio::time::timeout(Duration::from_secs(10),wait).await
            .map_err(|_|"diagnostic_export_timeout")?.map_err(|_|"diagnostic_writer_unavailable")?
    }
}
struct Storage {directory:PathBuf,writer:Option<BufWriter<File>>,bytes:u64,limit:u64,count:usize}
impl Storage {
    fn record(&mut self,event:Event,run_id:&str,sequence:&mut u64,counters:&Counters)->io::Result<()> {
        *sequence+=1;
        let recorded_unix_ms=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
        self.write(&serde_json::json!({"schema":1,"run_id":run_id,"sequence":sequence,
            "recorded_unix_ms":recorded_unix_ms,"dropped_records":counters.dropped.load(Ordering::Relaxed),
            "io_errors":counters.io_errors.load(Ordering::Relaxed),"record":event}))
    }
    fn open(directory:PathBuf,limit:u64,count:usize)->io::Result<Self> {
        fs::create_dir_all(&directory)?;
        let mut s=Self {directory,writer:None,bytes:0,limit,count};s.rotate()?;Ok(s)
    }
    fn path(&self,index:usize)->PathBuf {self.directory.join(format!("session-{index}.jsonl"))}
    fn flush(&mut self)->io::Result<()> {if let Some(writer)=self.writer.as_mut(){writer.flush()?;}Ok(())}
    fn rotate(&mut self)->io::Result<()> {
        self.flush()?;self.writer.take();
        for index in (0..self.count).rev() {
            let path=self.path(index);
            if path.exists() {
                if index==self.count-1 {fs::remove_file(path)?;}
                else {fs::rename(path,self.path(index+1))?;}
            }
        }
        self.writer=Some(BufWriter::new(File::options().write(true).create_new(true).open(self.path(0))?));
        self.bytes=0;Ok(())
    }
    fn write(&mut self,record:&serde_json::Value)->io::Result<()> {
        let mut bytes=serde_json::to_vec(record)?;bytes.push(b'\n');
        if bytes.len() as u64>self.limit {return Err(io::Error::other("record too large"));}
        if self.bytes+bytes.len() as u64>self.limit {self.rotate()?;}
        self.writer.as_mut().ok_or_else(||io::Error::other("writer unavailable"))?.write_all(&bytes)?;
        self.bytes+=bytes.len() as u64;Ok(())
    }
    fn export(&mut self)->io::Result<Vec<u8>> {
        self.flush()?;let mut result=Vec::new();
        for index in (0..self.count).rev() {
            let path=self.path(index);
            if path.exists() {
                let mut bytes=Vec::new();File::open(path)?.take(self.limit+1).read_to_end(&mut bytes)?;
                if bytes.len() as u64>self.limit {return Err(io::Error::other("journal file oversized"));}
                result.extend_from_slice(&bytes);
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event()->Event {Event::Session(Box::new(SessionSample::from_snapshot(&together_sync_core::Session::new("epoch".into()).state,10)))}
    fn phone_batch(count:usize)->crate::phone_logs::Batch {
        crate::phone_logs::Batch {protocol:1,batch_id:uuid::Uuid::new_v4(),samples:(0..count).map(|n|
            crate::phone_logs::Sample {elapsed_ms:n as u64,utc_ms:100,generation:1,position_ms:0,duration_ms:1000,
                connected:true,buffering:false,seeking:false,preparing:false,source_ready:true,fault:false,
                playback:crate::phone_logs::Playback::Paused,drift_ms:0,rtt_ms:None}).collect()}
    }
    #[tokio::test] async fn successful_phone_batch_is_not_poisoned_by_previous_unrelated_io_error() {
        let directory=std::env::temp_dir().join(format!("together-phone-recovery-{}",uuid::Uuid::new_v4()));
        let encoded=|event:Event|serde_json::to_vec(&serde_json::json!({
            "schema":1,"run_id":uuid::Uuid::nil(),"sequence":1,
            "recorded_unix_ms":u64::MAX,"dropped_records":0,"io_errors":1,"record":event
        })).unwrap().len() as u64+1;
        let limit=encoded(Event::Phone(phone_batch(1).samples.remove(0)))+32;
        assert!(encoded(event())>limit,"fixture must reject only the larger unrelated event");
        let journal=Journal::start_with_limits(directory.clone(),limit,2).unwrap();
        journal.record(event());
        assert_eq!(journal.flush().await,Err("diagnostic_flush_failed"));
        assert_eq!(journal.health().io_errors,1);
        let result=journal.phone_batch(phone_batch(1)).await;
        let rows=String::from_utf8(journal.export().await.unwrap()).unwrap();
        let values:Vec<serde_json::Value>=rows.lines().map(|line|serde_json::from_str(line).unwrap()).collect();

        assert_eq!(journal.health().io_errors,1);
        drop(journal);
        for _ in 0..100 {if fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(Duration::from_millis(10)).await;}
        assert!(!directory.exists());
        assert_eq!(result,Ok(()),"the new phone batch was written and flushed successfully");
        assert_eq!(values.len(),1);assert_eq!(values[0]["record"]["event"],"phone");
        assert_eq!(values[0]["io_errors"],1,"historical loss remains visible");
    }
    #[tokio::test] async fn phone_batch_uses_one_slot_and_ignores_unrelated_producer_drops() {
        let (sender,receiver)=mpsc::sync_channel(1);
        let counters=Arc::new(Counters::default());counters.active.store(true,Ordering::Relaxed);
        let journal=Arc::new(Journal {sender,counters});
        let writer=journal.clone();
        let worker=std::thread::spawn(move ||{
            let Command::PhoneBatch(samples,reply)=receiver.recv().unwrap() else {panic!("expected batch");};
            assert_eq!(samples.len(),20);
            // Fill the only slot, then drop an unrelated playback diagnostic.
            writer.record(event());writer.record(event());
            assert_eq!(writer.health().dropped_records,1);
            reply.send(Ok(())).unwrap();
        });
        journal.phone_batch(phone_batch(20)).await.unwrap();worker.join().unwrap();
    }
    #[tokio::test] async fn phone_batch_rejection_has_no_partial_queue_admission_or_false_receipt() {
        let (sender,receiver)=mpsc::sync_channel(1);
        let counters=Arc::new(Counters::default());counters.active.store(true,Ordering::Relaxed);
        let journal=Journal {sender,counters};journal.record(event());
        assert_eq!(journal.phone_batch(phone_batch(20)).await,Err("diagnostic_queue_unavailable"));
        assert!(matches!(receiver.try_recv().unwrap(),Command::Record(_)));
        assert!(matches!(receiver.try_recv(),Err(mpsc::TryRecvError::Empty)));
        for count in [0,21] {assert_eq!(journal.phone_batch(phone_batch(count)).await,Err("diagnostic_batch_invalid"));}
        assert!(matches!(receiver.try_recv(),Err(mpsc::TryRecvError::Empty)));
        drop(receiver);
        assert_eq!(journal.phone_batch(phone_batch(1)).await,Err("diagnostic_queue_unavailable"));
        let (sender,receiver)=mpsc::sync_channel(1);
        let journal=Journal {sender,counters:journal.counters.clone()};
        let worker=std::thread::spawn(move ||{assert!(matches!(receiver.recv().unwrap(),Command::PhoneBatch(_, _)));});
        assert_eq!(journal.phone_batch(phone_batch(1)).await,Err("diagnostic_writer_unavailable"));worker.join().unwrap();
    }
    #[tokio::test] async fn phone_batch_receipt_flushes_every_row_in_order() {
        let directory=std::env::temp_dir().join(format!("together-phone-batch-{}",uuid::Uuid::new_v4()));
        let journal=Journal::start(directory.clone()).unwrap();
        journal.record(event());journal.phone_batch(phone_batch(20)).await.unwrap();
        let text=fs::read_to_string(directory.join("session-0.jsonl")).unwrap();
        let rows:Vec<serde_json::Value>=text.lines().map(|line|serde_json::from_str(line).unwrap()).collect();
        assert_eq!(rows.len(),21);assert_eq!(rows[0]["record"]["event"],"session");
        for (index,row) in rows.iter().enumerate().skip(1) {
            assert_eq!(row["sequence"],index+1);assert_eq!(row["record"]["event"],"phone");
            assert_eq!(row["record"]["sample"]["elapsed_ms"],index-1);
        }
        drop(journal);
        for _ in 0..100 {if fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(Duration::from_millis(10)).await;}
        assert!(!directory.exists());
    }
    #[tokio::test(start_paused=true)] async fn phone_batch_missing_acknowledgement_is_bounded() {
        let (sender,_receiver)=mpsc::sync_channel(1);
        let counters=Arc::new(Counters::default());counters.active.store(true,Ordering::Relaxed);
        let journal=Journal {sender,counters};
        assert_eq!(journal.phone_batch(phone_batch(1)).await,Err("diagnostic_batch_timeout"));
    }
    #[tokio::test] async fn phone_batch_disk_failure_never_acknowledges_success() {
        let directory=std::env::temp_dir().join(format!("together-phone-batch-failure-{}",uuid::Uuid::new_v4()));
        // An intentionally tiny journal cannot store even one typed phone row.
        let journal=Journal::start_with_limits(directory.clone(),64,2).unwrap();
        assert_eq!(journal.phone_batch(phone_batch(2)).await,Err("diagnostic_batch_failed"));
        assert_eq!(journal.health().io_errors,1);
        assert_eq!(journal.phone_batch(phone_batch(1)).await,Err("diagnostic_batch_failed"));
        assert!(fs::read(directory.join("session-0.jsonl")).unwrap().is_empty());
        drop(journal);
        for _ in 0..100 {if fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(Duration::from_millis(10)).await;}
        assert!(!directory.exists());
    }
    #[tokio::test] async fn failed_http_upload_never_enters_successful_retry_cache() {
        use axum::{extract::State,http::{HeaderMap,HeaderValue,StatusCode},Json};
        let directory=std::env::temp_dir().join(format!("phone-http-write-failure-{}",uuid::Uuid::new_v4()));
        let host=crate::Host::new("pc-diagnostics-synthetic-0000000000".into(),"phone-diagnostics-synthetic-000000".into()).unwrap();
        assert!(host.diagnostics.set(Journal::start_with_limits(directory.clone(),64,2).unwrap()).is_ok());
        let id=uuid::Uuid::new_v4();
        for _ in 0..2 {
            let mut batch=phone_batch(1);batch.batch_id=id;
            let mut headers=HeaderMap::new();headers.insert("authorization",HeaderValue::from_static("Bearer phone-diagnostics-synthetic-000000"));
            let response=crate::phone_logs(State(host.clone()),headers,Json(batch)).await;
            assert_eq!(response.status(),StatusCode::SERVICE_UNAVAILABLE);
        }
        assert_eq!(host.diagnostic_health().unwrap().io_errors,2);
        assert!(fs::read(directory.join("session-0.jsonl")).unwrap().is_empty());
        drop(host);
        for _ in 0..100 {if fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(Duration::from_millis(10)).await;}
        assert!(!directory.exists());
    }
    #[test] fn unknown_command_error_text_cannot_enter_journal() {
        assert_eq!(serde_json::to_string(&CommandOutcome::from_error(Some("SECRET_TOKEN_URL"))).unwrap(),"\"other_rejected\"");
        assert_eq!(serde_json::to_string(&CommandOutcome::from_error(Some("stale epoch"))).unwrap(),"\"stale_epoch\"");
        assert_eq!(serde_json::to_string(&CommandOutcome::from_error(None)).unwrap(),"\"accepted\"");
    }
    #[tokio::test] async fn source_failures_export_only_fixed_phases_and_numeric_context_in_order() {
        let directory=std::env::temp_dir().join(format!("together-source-journal-{}",uuid::Uuid::new_v4()));
        let host=crate::Host::new("synthetic-private-pc-token-0000000".into(),"synthetic-private-phone-token-0000".into()).unwrap();
        host.enable_diagnostics(directory.clone()).unwrap();
        host.record_source(7,11,true,SourceStage::CurrentFileAndLinks,SourceOutcome::Passed,150);
        host.record_source(7,11,true,SourceStage::DirectRanges,SourceOutcome::Failed,350);
        host.record_source(8,12,false,SourceStage::Publication,SourceOutcome::Replaced,600);
        host.flush_diagnostics().await.unwrap();
        let bytes=host.export_diagnostics().await.unwrap();let text=String::from_utf8(bytes).unwrap();
        assert!(!text.contains("synthetic-private") && !text.contains("https://") && !text.contains("filename"));
        let rows:Vec<serde_json::Value>=text.lines().map(|line|serde_json::from_str(line).unwrap()).collect();
        let samples:Vec<_>=rows.iter().map(|row|{assert_eq!(row["record"]["event"],"source");&row["record"]["sample"]}).collect();
        assert_eq!(samples.len(),3);assert_eq!(samples[0]["stage"],"current_file_and_links");
        assert_eq!(samples[1]["stage"],"direct_ranges");assert_eq!(samples[1]["outcome"],"failed");
        assert_eq!(samples[1]["selection"],7);assert_eq!(samples[1]["generation"],11);assert_eq!(samples[1]["refresh"],true);
        assert_eq!(samples[2]["outcome"],"replaced");assert_eq!(samples[2]["elapsed_ms"],600);
        assert!(samples.iter().all(|sample|sample.as_object().unwrap().len()==7));
        drop(host);
        for _ in 0..100 {if fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(Duration::from_millis(10)).await;}
        assert!(!directory.exists());
    }
    #[tokio::test] async fn journal_preserves_order_and_excludes_untrusted_fields() {
        let directory=std::env::temp_dir().join(format!("together-journal-{}",uuid::Uuid::new_v4()));
        let journal=Journal::start(directory.clone()).unwrap();
        let mut s=together_sync_core::Session::new("SECRET_EPOCH".into()).state;
        let mut media=together_sync_core::MediaIdentity::fixture();media.filename=Some("SECRET_URL_AND_TOKEN".into());
        s.media=Some(media);s.fault=Some("SECRET_ERROR".into());
        journal.record(Event::Session(Box::new(SessionSample::from_snapshot(&s,10))));journal.record(event());
        let bytes=journal.export().await.unwrap();let text=String::from_utf8(bytes).unwrap();
        assert!(!text.contains("SECRET"));assert!(text.contains("unknown_fault"));
        let rows:Vec<serde_json::Value>=text.lines().map(|s|serde_json::from_str(s).unwrap()).collect();
        assert_eq!(rows.len(),2);assert_eq!(rows[0]["sequence"],1);assert_eq!(rows[1]["sequence"],2);
        assert_eq!(rows[0]["run_id"],rows[1]["run_id"]);assert_eq!(journal.health().io_errors,0);
        drop(journal); // Export already flushed; worker owns the open handle until channel close.
        // Windows cannot delete the active writer; cleanup after it observes disconnect.
        for _ in 0..100 {if fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(Duration::from_millis(10)).await;}
        assert!(!directory.exists());
    }
    #[test] fn rotation_is_bounded_and_exports_oldest_first() {
        let directory=std::env::temp_dir().join(format!("together-rotation-{}",uuid::Uuid::new_v4()));
        let mut storage=Storage::open(directory.clone(),64,3).unwrap();
        for n in 0..30 {storage.write(&serde_json::json!({"sequence":n,"sample":123456789})).unwrap();}
        let bytes=storage.export().unwrap();assert!(bytes.len()<=192);
        let rows:Vec<serde_json::Value>=String::from_utf8(bytes).unwrap().lines().map(|s|serde_json::from_str(s).unwrap()).collect();
        assert_eq!(rows.last().unwrap()["sequence"],29);
        assert!(rows.windows(2).all(|r|r[0]["sequence"].as_u64()<r[1]["sequence"].as_u64()));
        assert_eq!(fs::read_dir(&directory).unwrap().count(),3);
        drop(storage);
        let mut restarted=Storage::open(directory.clone(),64,3).unwrap();
        restarted.write(&serde_json::json!({"sequence":30,"sample":123456789})).unwrap();
        let text=String::from_utf8(restarted.export().unwrap()).unwrap();
        assert!(text.contains("\"sequence\":29"));assert!(text.contains("\"sequence\":30"));
        assert_eq!(fs::read_dir(&directory).unwrap().count(),3);
        drop(restarted);fs::remove_dir_all(directory).unwrap();
    }
    #[tokio::test] async fn flush_acknowledges_typed_shutdown_records_without_export() {
        let directory=std::env::temp_dir().join(format!("together-flush-{}",uuid::Uuid::new_v4()));
        let journal=Journal::start(directory.clone()).unwrap();
        for kind in [AppLifecycleKind::ShutdownRequested,AppLifecycleKind::TunnelStopped,AppLifecycleKind::TunnelStopFailed,AppLifecycleKind::PlayerStopped,AppLifecycleKind::ServerStopped,AppLifecycleKind::ExitPermitted] {
            journal.record(Event::AppLifecycle(AppLifecycleSample {server_ms:42,kind}));
        }
        journal.flush().await.unwrap();
        let text=fs::read_to_string(directory.join("session-0.jsonl")).unwrap();
        let rows:Vec<serde_json::Value>=text.lines().map(|line|serde_json::from_str(line).unwrap()).collect();
        let kinds:Vec<_>=rows.iter().map(|row|row["record"]["sample"]["kind"].as_str().unwrap()).collect();
        assert_eq!(kinds,["shutdown_requested","tunnel_stopped","tunnel_stop_failed","player_stopped","server_stopped","exit_permitted"]);
        assert!(rows.iter().all(|row|row["record"]["event"]=="app_lifecycle"));
        assert_eq!(journal.health().io_errors,0);drop(journal);
        for _ in 0..100 {if fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(Duration::from_millis(10)).await;}
        assert!(!directory.exists());
    }
    #[tokio::test] async fn flush_queue_full_and_writer_disconnect_fail_without_false_acknowledgement() {
        let (sender,receiver)=mpsc::sync_channel(1);
        let journal=Journal {sender,counters:Arc::new(Counters::default())};journal.record(event());
        assert_eq!(journal.flush().await,Err("diagnostic_queue_unavailable"));
        drop(receiver);assert_eq!(journal.flush().await,Err("diagnostic_queue_unavailable"));
        // A writer accepting a request but disappearing before ack also fails.
        let (sender,receiver)=mpsc::sync_channel(1);let journal=Journal {sender,counters:Arc::new(Counters::default())};
        let worker=std::thread::spawn(move ||{assert!(matches!(receiver.recv().unwrap(),Command::Flush(_)));});
        assert_eq!(journal.flush().await,Err("diagnostic_writer_unavailable"));worker.join().unwrap();
        // A writer that never consumes/acks must not keep app exit waiting.
        let (sender,_receiver)=mpsc::sync_channel(1);let journal=Journal {sender,counters:Arc::new(Counters::default())};
        assert_eq!(tokio::time::timeout(Duration::from_secs(3),journal.flush()).await.unwrap(),Err("diagnostic_flush_timeout"));
    }
    #[test] fn full_queue_drops_records_without_waiting() {
        let (sender,_receiver)=mpsc::sync_channel(1);
        let journal=Journal {sender,counters:Arc::new(Counters::default())};
        journal.record(event());journal.record(event());journal.record(event());
        assert_eq!(journal.health().dropped_records,2);
    }
    #[tokio::test] async fn filesystem_failure_is_observable_without_panicking_producer() {
        let path=std::env::temp_dir().join(format!("together-invalid-{}",uuid::Uuid::new_v4()));
        fs::write(&path,b"not a directory").unwrap();let journal=Journal::start(path.clone()).unwrap();
        for _ in 0..100 {if !journal.health().active{break;}tokio::time::sleep(Duration::from_millis(10)).await;}
        journal.record(event());assert_eq!(journal.health().io_errors,1);
        assert!(!journal.health().active);assert!(journal.export().await.is_err());assert!(journal.flush().await.is_err());
        fs::remove_file(path).unwrap();
    }
}

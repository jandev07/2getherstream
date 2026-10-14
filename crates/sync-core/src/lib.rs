//! Deterministic session reducer. All timestamps are milliseconds since server boot.
//! Callers serialize mutations; player events never become user intentions.
pub mod clock;
pub mod drift;
pub mod media;
pub use media::{MediaIdentity,MediaProvider};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const PROTOCOL_VERSION: u32 = 1;
pub const PREPARE_TIMEOUT_MS: u64 = 15_000;
pub const COMMIT_LEAD_MS: u64 = 500;
/// Allows one frame at 10 fps after an exact seek; this is not a drift threshold.
pub const READY_POSITION_TOLERANCE_MS: u64 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub position_ms: u64,
    pub anchor_ms: u64,
    pub playing: bool,
}
impl Timeline {
    pub fn position(&self, now: u64) -> u64 {
        self.position_ms.saturating_add(if self.playing { now.saturating_sub(self.anchor_ms) } else { 0 })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prepare {
    pub generation: u64,
    pub position_ms: u64,
    pub deadline_ms: u64,
    pub ready: BTreeSet<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    pub generation: u64, pub playing: bool, pub position_ms: u64,
    pub observed_server_ms: u64, pub received_server_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub connected: bool, pub buffering: bool, pub rtt_ms: u64,
    #[serde(default)] pub execution: Option<Execution>,
    #[serde(default)] pub player_fault: Option<PlayerFault>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum PlayerFault {LoadFailed,DecoderFailed,CommandFailed,UnexpectedEnd}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub protocol: u32,
    pub epoch: String,
    pub revision: u64,
    pub generation: u64,
    pub media: Option<MediaIdentity>,
    pub desired_playing: bool,
    #[serde(default)] pub ended: bool,
    pub timeline: Timeline,
    pub prepare: Option<Prepare>,
    pub participants: BTreeMap<String, Participant>,
    pub fault: Option<String>,
}

pub struct Session { pub state: Snapshot }
impl Session {
    pub fn new(epoch: String) -> Self {
        let participants = ["pc", "phone"].into_iter().map(|id| (id.to_string(), Participant { connected: false, buffering: false, rtt_ms:0, execution:None, player_fault:None })).collect();
        Self { state: Snapshot { protocol: PROTOCOL_VERSION, epoch, revision: 0, generation: 0, media: None,
            desired_playing: false, ended: false, timeline: Timeline { position_ms: 0, anchor_ms: 0, playing: false },
            prepare: None, participants, fault: None } }
    }
    fn changed(&mut self) { self.state.revision += 1; }
    fn freeze(&mut self, now: u64) {
        self.state.timeline.position_ms = self.state.timeline.position(now);
        self.state.timeline.anchor_ms = now;
        self.state.timeline.playing = false;
    }
    fn prepare(&mut self, position_ms: u64, now: u64) {
        self.state.ended=false;
        self.freeze(now);
        self.state.generation += 1;
        self.state.timeline.position_ms = position_ms;
        self.state.prepare = Some(Prepare { generation: self.state.generation, position_ms,
            deadline_ms: now + PREPARE_TIMEOUT_MS, ready: BTreeSet::new() });
        self.state.fault = None;
        self.changed();
    }
    pub fn connection(&mut self, id: &str, connected: bool, now: u64) -> Result<(), &'static str> {
        let p = self.state.participants.get_mut(id).ok_or("unknown participant")?;
        if p.connected == connected { return Ok(()); }
        p.connected = connected;
        p.buffering = false;
        p.rtt_ms = 0;
        p.player_fault=None;
        if self.state.media.is_some() && !self.state.ended && !self.state.participants.values().any(|p|p.player_fault.is_some()) {
            let pos = self.state.timeline.position(now);
            self.prepare(pos, now);
        } else { self.changed(); }
        Ok(())
    }
    /// Terminal native failures stop the room. Buffering reports cannot clear them.
    pub fn player_failed(&mut self,id:&str,generation:u64,code:PlayerFault,now:u64) -> Result<(), &'static str> {
        self.valid_base(generation)?;
        if self.state.media.is_none() {return Err("no media");}
        let p=self.state.participants.get_mut(id).ok_or("unknown participant")?;
        if !p.connected {return Err("participant offline");}
        if p.player_fault==Some(code) && self.state.prepare.is_none() {return Ok(());}
        p.player_fault=Some(code);self.freeze(now);self.state.prepare=None;self.state.generation+=1;
        self.state.fault=Some(format!("player_failed_{id}"));self.changed();Ok(())
    }
    pub fn player_recovered(&mut self,id:&str,generation:u64,now:u64) -> Result<(), &'static str> {
        self.valid_base(generation)?;
        let p=self.state.participants.get_mut(id).ok_or("unknown participant")?;
        if !p.connected {return Err("participant offline");}
        if p.player_fault.take().is_none() {return Ok(());}
        if !self.state.participants.values().any(|p|p.player_fault.is_some()) && self.state.media.is_some() && !self.state.ended {
            self.prepare(self.state.timeline.position(now),now);
        } else {self.changed();}
        Ok(())
    }
    pub fn load(&mut self, media: MediaIdentity, now: u64) -> Result<(), &'static str> {
        media.validate()?;
        self.state.media = Some(media);
        self.state.desired_playing = false;
        self.prepare(0, now);
        Ok(())
    }
    /// RTT measured by server-issued WebSocket ping/pong, never client wall time.
    pub fn observe_rtt(&mut self, id: &str, rtt_ms: u64) -> Result<(), &'static str> {
        if rtt_ms > 5000 { return Err("RTT out of range"); }
        let p=self.state.participants.get_mut(id).ok_or("unknown participant")?;
        if !p.connected {return Err("participant offline");}
        p.rtt_ms=rtt_ms; self.changed(); Ok(())
    }
    pub fn seek(&mut self, position_ms: u64, base: u64, now: u64) -> Result<(), &'static str> {
        self.valid_base(base)?;
        if self.state.media.is_none() { return Err("no media"); }
        self.prepare(position_ms, now);
        Ok(())
    }
    pub fn intent(&mut self, playing: bool, base: u64, now: u64) -> Result<(), &'static str> {
        self.valid_base(base)?;
        if self.state.media.is_none() { return Err("no media"); }
        self.state.desired_playing = playing;
        let pos = if playing && self.state.ended {0} else {self.state.timeline.position(now)};
        if playing { self.prepare(pos, now); }
        else { self.freeze(now); self.state.generation += 1; self.state.prepare = None; self.changed(); }
        Ok(())
    }
    fn valid_base(&self, base: u64) -> Result<(), &'static str> {
        if base != self.state.generation { Err("stale generation") } else { Ok(()) }
    }
    /// Native EOF feedback is distinct from pause/buffering. Reject stale/premature reports.
    pub fn end_of_media(&mut self,id:&str,generation:u64,position_ms:u64,duration_ms:u64,now:u64) -> Result<(), &'static str> {
        self.valid_base(generation)?;
        let participant=self.state.participants.get(id).ok_or("unknown participant")?;
        if !participant.connected {return Err("participant offline");}
        let media=self.state.media.as_ref().ok_or("no media")?;
        if !self.state.timeline.playing || self.state.prepare.is_some() || self.state.fault.is_some() || now<self.state.timeline.anchor_ms {return Err("not in active playback");}
        if duration_ms==0 || duration_ms>604_800_000 || position_ms.abs_diff(duration_ms)>1000 || self.state.timeline.position(now).abs_diff(duration_ms)>2500 {return Err("premature end report");}
        if media.duration_ms.is_some_and(|d|d.abs_diff(duration_ms)>1000) {return Err("media duration mismatch");}
        self.state.desired_playing=false;self.state.ended=true;self.state.prepare=None;
        self.state.timeline=Timeline {position_ms:duration_ms,anchor_ms:now,playing:false};
        self.state.generation+=1;self.changed();Ok(())
    }
    pub fn buffering(&mut self, id: &str, generation: u64, buffering: bool, now: u64) -> Result<(), &'static str> {
        self.valid_base(generation)?;
        let p = self.state.participants.get_mut(id).ok_or("unknown participant")?;
        if !p.connected { return Err("participant offline"); }
        if p.buffering == buffering { return Ok(()); }
        p.buffering = buffering;
        // Any buffer state change invalidates readiness and scheduled starts.
        // Desired user intent survives the barrier; it is never inferred from mpv pause.
        if self.state.media.is_some() && !self.state.ended && !self.state.participants.values().any(|p|p.player_fault.is_some()) {
            let pos = self.state.timeline.position(now);
            self.prepare(pos, now);
        } else { self.changed(); }
        Ok(())
    }
    pub fn ready(&mut self, id: &str, generation: u64, position_ms: u64, now: u64) -> Result<(), &'static str> {
        self.valid_base(generation)?;
        let p = self.state.participants.get(id).ok_or("unknown participant")?;
        if !p.connected || p.buffering || p.player_fault.is_some() { return Err("participant not available"); }
        let prep = self.state.prepare.as_mut().ok_or("no prepare")?;
        if now >= prep.deadline_ms { return Err("prepare expired"); }
        if position_ms.abs_diff(prep.position_ms) > READY_POSITION_TOLERANCE_MS { return Err("player has not reached target"); }
        prep.ready.insert(id.into());
        let available = self.state.participants.iter().all(|(id, p)| p.connected && !p.buffering && p.player_fault.is_none() && prep.ready.contains(id));
        if available {
            let max_rtt=self.state.participants.values().map(|p|p.rtt_ms).max().unwrap_or(0);
            let lead=COMMIT_LEAD_MS.max(max_rtt.saturating_mul(2).saturating_add(250)).min(5000);
            self.state.timeline = Timeline { position_ms: prep.position_ms, anchor_ms: now + lead,
                playing: self.state.desired_playing };
            self.state.prepare = None;
        }
        self.changed();
        Ok(())
    }
    /// An observed native state, not acceptance of a command by the player queue.
    pub fn executed(&mut self,id:&str,generation:u64,playing:bool,position_ms:u64,observed_server_ms:u64,now:u64) -> Result<(), &'static str> {
        self.valid_base(generation)?;
        if self.state.media.is_none() || self.state.prepare.is_some() || self.state.fault.is_some() {return Err("no committed playback");}
        if playing!=self.state.timeline.playing {return Err("execution state mismatch");}
        let participant=self.state.participants.get_mut(id).ok_or("unknown participant")?;
        if !participant.connected || participant.buffering || participant.player_fault.is_some() {return Err("participant not available");}
        if observed_server_ms>now.saturating_add(250) || now.saturating_sub(observed_server_ms)>5000 {return Err("execution observation expired");}
        if playing && observed_server_ms<self.state.timeline.anchor_ms {return Err("execution before scheduled start");}
        if position_ms.abs_diff(self.state.timeline.position(observed_server_ms))>500 || (playing && position_ms<self.state.timeline.position_ms.saturating_add(20)) {return Err("execution position mismatch");}
        if participant.execution.as_ref().is_some_and(|e|e.generation==generation && e.playing==playing) {return Ok(());}
        participant.execution=Some(Execution {generation,playing,position_ms,observed_server_ms,received_server_ms:now});
        self.changed();Ok(())
    }
    pub fn tick(&mut self, now: u64) {
        if self.state.prepare.as_ref().is_some_and(|p| now >= p.deadline_ms) {
            self.freeze(now);
            self.state.prepare = None;
            self.state.generation += 1;
            self.state.fault = Some("prepare_timeout".into());
            self.changed();
        }
        // A future commit alone is not evidence that either native player ran it.
        if self.state.media.is_some() && !self.state.ended && self.state.prepare.is_none() && self.state.fault.is_none() {
            let grace=self.state.participants.values().map(|p|p.rtt_ms.saturating_mul(2).saturating_add(2000)).max().unwrap_or(2000).min(12000);
            if now>=self.state.timeline.anchor_ms.saturating_add(grace) {
                let missing=self.state.participants.iter().find(|(_,p)|p.connected && !p.buffering && !p.execution.as_ref().is_some_and(|e|e.generation==self.state.generation && e.playing==self.state.timeline.playing)).map(|(id,_)|id.clone());
                if let Some(id)=missing {
                    self.freeze(now);self.state.generation+=1;
                    self.state.fault=Some(format!("execution_timeout_{id}"));self.changed();
                }
            }
        }
    }
}

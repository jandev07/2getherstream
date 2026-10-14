//! PC-side source transaction. Fetching links never holds the selection lock.
//! URLs stay private until both device leases belong to the current selection.
//! The application must still fence native loads against its session epoch/media.
//!
//! ```no_run
//! use together_source_resolver::{MediaIdentity, TorBox};
//! use together_source_resolver::selection::{Device, MediaSelection, SelectionError, SelectedLease};
//! async fn resolve_current(selection: &MediaSelection, torbox: &TorBox,
//!     media: MediaIdentity) -> Result<(SelectedLease, SelectedLease), SelectionError> {
//!     let ticket = selection.select(media)?;
//!     let id = ticket.id();
//!     ticket.resolve(torbox, None, None).await?;
//!     Ok((selection.take(&id, Device::Pc)?, selection.take(&id, Device::Phone)?))
//! }
//! ```
use crate::{DeviceStreams, MediaIdentity, MediaProvider, StreamLease, TorBox};
use std::{future::Future, net::IpAddr, sync::{Arc, Mutex, Weak}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionError { InvalidMedia, Superseded, ResolverFailed, IdentityMismatch, Cancelled, Closed, Unavailable, NotReady, AlreadyTaken }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionStatus { Empty, Resolving, Ready, Failed(SelectionError), Closed }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device { Pc, Phone }

// Not serializable: callers cannot manufacture receipts from a numeric sequence.
#[derive(Clone)]
pub struct SelectionId { owner: Weak<Mutex<State>>, sequence: u64 }
/// A lease queued for delivery, with no unguarded URL getter or serialization.
pub struct SelectedLease { id: SelectionId, lease: StreamLease }
impl SelectedLease {
    /// Consume at the final local handoff. Replacement/close cannot interleave
    /// with this callback. Keep it short and synchronous (e.g. enqueue a command).
    /// Do not reenter this MediaSelection, await network/native work, or retain a
    /// copied URL for a later unfenced load. Controllers must additionally check
    /// the session epoch/media when their queued native command executes.
    pub fn deliver<T>(self, handoff: impl FnOnce(&StreamLease) -> T) -> Result<T, SelectionError> {
        let owner = self.id.owner.upgrade().ok_or(SelectionError::Closed)?;
        let state = owner.lock().map_err(|_| SelectionError::Unavailable)?;
        if state.status == SelectionStatus::Closed { return Err(SelectionError::Closed); }
        if state.sequence != self.id.sequence { return Err(SelectionError::Superseded); }
        if state.status != SelectionStatus::Ready { return Err(SelectionError::NotReady); }
        // Keep the guard alive through the callback, preventing a check/use race.
        let result = handoff(&self.lease);
        drop(state);
        Ok(result)
    }
}
pub struct MediaSelection { state: Arc<Mutex<State>> }
struct State {
    sequence: u64,
    status: SelectionStatus,
    pc: Option<StreamLease>,
    phone: Option<StreamLease>,
    changed: tokio::sync::watch::Sender<()>,
}
impl State {
    fn discard(&mut self) { self.pc = None; self.phone = None; }
    fn invalidate(&mut self) -> Result<(), SelectionError> {
        self.discard();
        self.changed.send_replace(());
        match self.sequence.checked_add(1) {
            Some(next) => { self.sequence = next; Ok(()) },
            None => { self.status = SelectionStatus::Closed; Err(SelectionError::Unavailable) },
        }
    }
}
pub struct ResolutionTicket {
    id: SelectionId,
    media: MediaIdentity,
    finished: bool,
    changed: tokio::sync::watch::Receiver<()>,
}

impl Default for MediaSelection { fn default() -> Self { Self::new() } }
impl MediaSelection {
    pub fn new() -> Self {
        let (changed, _) = tokio::sync::watch::channel(());
        Self { state: Arc::new(Mutex::new(State { sequence: 0, status: SelectionStatus::Empty, pc: None, phone: None, changed })) }
    }
    /// Invoke at the serialized source-selection boundary, including a refresh.
    /// Selecting the same file again supersedes the preceding fetch and leases.
    pub fn select(&self, media: MediaIdentity) -> Result<ResolutionTicket, SelectionError> {
        let mut state = self.state.lock().map_err(|_| SelectionError::Unavailable)?;
        if state.status == SelectionStatus::Closed { return Err(SelectionError::Closed); }
        state.invalidate()?;
        // An invalid new choice also revokes old delivery instead of falling back.
        if media.provider != MediaProvider::Torbox || media.validate().is_err() {
            state.status = SelectionStatus::Failed(SelectionError::InvalidMedia);
            return Err(SelectionError::InvalidMedia);
        }
        state.status = SelectionStatus::Resolving;
        Ok(ResolutionTicket { id: SelectionId { owner: Arc::downgrade(&self.state), sequence: state.sequence }, media, finished: false, changed: state.changed.subscribe() })
    }
    pub fn status(&self) -> Result<SelectionStatus, SelectionError> {
        Ok(self.state.lock().map_err(|_| SelectionError::Unavailable)?.status)
    }
    /// Unload/leave the source without permanently closing this authority.
    /// Pending fetches and already-taken undelivered wrappers are revoked.
    pub fn clear(&self) -> Result<(), SelectionError> {
        let mut state = self.state.lock().map_err(|_| SelectionError::Unavailable)?;
        if state.status == SelectionStatus::Closed { return Err(SelectionError::Closed); }
        state.invalidate()?;
        state.status = SelectionStatus::Empty;
        Ok(())
    }
    /// Each role receives only its separately requested lease, at most once.
    /// The queued wrapper must pass its final deliver() check before URL access.
    /// Ready means resolution succeeded, not that native playback is ready.
    pub fn take(&self, id: &SelectionId, device: Device) -> Result<SelectedLease, SelectionError> {
        if !Weak::ptr_eq(&id.owner, &Arc::downgrade(&self.state)) { return Err(SelectionError::Superseded); }
        let mut state = self.state.lock().map_err(|_| SelectionError::Unavailable)?;
        if state.status == SelectionStatus::Closed { return Err(SelectionError::Closed); }
        if state.sequence != id.sequence { return Err(SelectionError::Superseded); }
        if state.status != SelectionStatus::Ready { return Err(SelectionError::NotReady); }
        let lease = match device { Device::Pc => &mut state.pc, Device::Phone => &mut state.phone }
            .take().ok_or(SelectionError::AlreadyTaken)?;
        Ok(SelectedLease { id: id.clone(), lease })
    }
    /// Irreversible authority shutdown; no late task can republish its URLs.
    pub fn close(&self) {
        if let Ok(mut state) = self.state.lock() { state.discard(); state.status = SelectionStatus::Closed; state.changed.send_replace(()); }
    }
}
impl Drop for MediaSelection { fn drop(&mut self) { self.close(); } }

impl ResolutionTicket {
    pub fn id(&self) -> SelectionId { self.id.clone() }
    pub async fn resolve(self, torbox: &TorBox, pc_ip: Option<IpAddr>, phone_ip: Option<IpAddr>) -> Result<(), SelectionError> {
        let media = self.media.clone();
        self.resolve_with(torbox.device_streams(&media, pc_ip, phone_ip)).await
    }
    // Only the production resolver can construct leases. No public fake URL seam.
    async fn resolve_with(mut self, fetch: impl Future<Output = anyhow::Result<DeviceStreams>>) -> Result<(), SelectionError> {
        // Reject tasks queued after replacement before doing provider work.
        self.check_current()?;
        // Dropping the obsolete future cancels its outstanding metadata request.
        // Replacement/shutdown wins when response and revocation are both ready.
        let resolved = tokio::select! { biased;
            _ = self.changed.changed() => {
                self.check_current()?;
                return Err(SelectionError::Unavailable);
            }
            result = fetch => result,
        };
        let owner = self.id.owner.upgrade().ok_or(SelectionError::Closed)?;
        let mut state = owner.lock().map_err(|_| SelectionError::Unavailable)?;
        self.check_state(&state)?;
        self.finished = true;
        let streams = match resolved {
            Ok(streams) => streams,
            Err(_) => { state.status = SelectionStatus::Failed(SelectionError::ResolverFailed); return Err(SelectionError::ResolverFailed); }
        };
        if streams.pc.identity() != &self.media || streams.phone.identity() != &self.media {
            state.status = SelectionStatus::Failed(SelectionError::IdentityMismatch);
            return Err(SelectionError::IdentityMismatch);
        }
        state.pc = Some(streams.pc); state.phone = Some(streams.phone);
        state.status = SelectionStatus::Ready;
        Ok(())
    }
    fn check_state(&self, state: &State) -> Result<(), SelectionError> {
        if state.status == SelectionStatus::Closed { return Err(SelectionError::Closed); }
        if state.sequence != self.id.sequence { return Err(SelectionError::Superseded); }
        if state.status != SelectionStatus::Resolving { return Err(SelectionError::NotReady); }
        Ok(())
    }
    fn check_current(&self) -> Result<(), SelectionError> {
        let owner = self.id.owner.upgrade().ok_or(SelectionError::Closed)?;
        let state = owner.lock().map_err(|_| SelectionError::Unavailable)?;
        self.check_state(&state)
    }
}
impl Drop for ResolutionTicket {
    fn drop(&mut self) {
        if self.finished { return; }
        if let Some(owner) = self.id.owner.upgrade() {
            if let Ok(mut state) = owner.lock() {
                if self.check_state(&state).is_ok() { state.discard(); state.status = SelectionStatus::Failed(SelectionError::Cancelled); }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn streams(media: &MediaIdentity) -> DeviceStreams {
        let lease = |role| StreamLease { identity: media.clone(), url: crate::validate_stream_url(&format!("https://cdn.invalid/{role}?cap=synthetic"), "account-secret").unwrap() };
        DeviceStreams { pc: lease("pc"), phone: lease("phone") }
    }
    #[tokio::test]
    async fn device_requests_run_concurrently_and_do_not_publish_before_both_finish() {
        let media = MediaIdentity::torbox(1, 2); let pair = streams(&media);
        let barrier = Arc::new(tokio::sync::Barrier::new(3));
        let pc_barrier = barrier.clone(); let phone_barrier = barrier.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(crate::collect_device_streams(
            async move { pc_barrier.wait().await; Ok(pair.pc) },
            async move { phone_barrier.wait().await; rx.await.unwrap(); Ok(pair.phone) },
        ));
        tokio::time::timeout(std::time::Duration::from_secs(2), barrier.wait()).await.unwrap();
        assert!(!task.is_finished()); tx.send(()).unwrap();
        let pair = task.await.unwrap().unwrap();
        assert!(pair.pc.player_url().contains("/pc?")); assert!(pair.phone.player_url().contains("/phone?"));
    }
    #[tokio::test]
    async fn either_device_failure_cancels_other_inflight_request() {
        for pc_fails in [true, false] {
            let barrier = Arc::new(tokio::sync::Barrier::new(3));
            let pc_barrier = barrier.clone(); let phone_barrier = barrier.clone();
            let dropped = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            struct CountDrop(Arc<std::sync::atomic::AtomicUsize>);
            impl Drop for CountDrop { fn drop(&mut self) { self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst); } }
            let pc_drop = CountDrop(dropped.clone()); let phone_drop = CountDrop(dropped.clone());
            let task = tokio::spawn(crate::collect_device_streams(
                async move { let _guard = pc_drop; pc_barrier.wait().await; if pc_fails { anyhow::bail!("rejected"); } std::future::pending().await },
                async move { let _guard = phone_drop; phone_barrier.wait().await; if !pc_fails { anyhow::bail!("rejected"); } std::future::pending().await },
            ));
            tokio::time::timeout(std::time::Duration::from_secs(2), barrier.wait()).await.unwrap();
            assert!(tokio::time::timeout(std::time::Duration::from_secs(2), task).await.unwrap().unwrap().is_err());
            assert_eq!(dropped.load(std::sync::atomic::Ordering::SeqCst), 2);
        }
    }
    #[tokio::test]
    async fn separate_roles_are_taken_once_and_receipts_cannot_cross_owners() {
        let first = MediaSelection::new(); let second = MediaSelection::new();
        let media = MediaIdentity::torbox(1, 2);
        let ticket = first.select(media.clone()).unwrap(); let id = ticket.id();
        assert!(matches!(first.take(&id, Device::Pc), Err(SelectionError::NotReady)));
        ticket.resolve_with(async { Ok(streams(&media)) }).await.unwrap();
        assert!(matches!(second.take(&id, Device::Pc), Err(SelectionError::Superseded)));
        let pc = first.take(&id, Device::Pc).unwrap(); let phone = first.take(&id, Device::Phone).unwrap();
        let pc_media = pc.deliver(|lease| { assert!(lease.player_url().contains("/pc?")); lease.identity().clone() }).unwrap();
        let phone_media = phone.deliver(|lease| { assert!(lease.player_url().contains("/phone?")); lease.identity().clone() }).unwrap();
        assert_eq!(pc_media, phone_media);
        assert!(matches!(first.take(&id, Device::Pc), Err(SelectionError::AlreadyTaken)));
        assert!(matches!(first.take(&id, Device::Phone), Err(SelectionError::AlreadyTaken)));
    }
    #[tokio::test]
    async fn delayed_old_result_cannot_replace_new_ready_pair_even_for_same_file() {
        let selection = MediaSelection::new(); let media = MediaIdentity::torbox(1, 2);
        let old = selection.select(media.clone()).unwrap(); let old_id = old.id();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (finish_tx, finish_rx) = tokio::sync::oneshot::channel();
        let old_media = media.clone();
        let task = tokio::spawn(old.resolve_with(async move { started_tx.send(()).unwrap(); finish_rx.await.unwrap(); Ok(streams(&old_media)) }));
        started_rx.await.unwrap();
        let new = selection.select(media.clone()).unwrap(); let new_id = new.id();
        new.resolve_with(async { Ok(streams(&media)) }).await.unwrap();
        // Supersession cancels the provider future even if it never replies.
        assert_eq!(task.await.unwrap(), Err(SelectionError::Superseded));
        assert!(finish_tx.send(()).is_err());
        assert_eq!(selection.status().unwrap(), SelectionStatus::Ready);
        assert!(matches!(selection.take(&old_id, Device::Phone), Err(SelectionError::Superseded)));
        assert!(selection.take(&new_id, Device::Phone).is_ok());
    }
    #[tokio::test]
    async fn stale_queued_fetch_never_runs_and_invalid_selection_revokes_ready_pair() {
        let selection = MediaSelection::new(); let media = MediaIdentity::torbox(1, 2);
        let old = selection.select(media.clone()).unwrap();
        let new = selection.select(media.clone()).unwrap(); let id = new.id();
        assert_eq!(old.resolve_with(async { panic!("superseded fetch polled") }).await, Err(SelectionError::Superseded));
        new.resolve_with(async { Ok(streams(&media)) }).await.unwrap();
        assert!(matches!(selection.select(MediaIdentity::fixture()), Err(SelectionError::InvalidMedia)));
        assert!(matches!(selection.take(&id, Device::Pc), Err(SelectionError::Superseded)));
        assert_eq!(selection.status().unwrap(), SelectionStatus::Failed(SelectionError::InvalidMedia));
    }
    #[tokio::test]
    async fn failure_and_wrong_file_never_publish_partial_leases_or_private_errors() {
        let selection = MediaSelection::new(); let media = MediaIdentity::torbox(1, 2);
        let ticket = selection.select(media.clone()).unwrap(); let id = ticket.id();
        let error = ticket.resolve_with(async { anyhow::bail!("https://private.invalid/?account-secret") }).await.unwrap_err();
        assert_eq!(format!("{error:?}"), "ResolverFailed");
        assert!(matches!(selection.take(&id, Device::Pc), Err(SelectionError::NotReady)));
        let ticket = selection.select(media.clone()).unwrap(); let id = ticket.id();
        let mut pair = streams(&media); pair.phone.identity = MediaIdentity::torbox(1, 3);
        assert_eq!(ticket.resolve_with(async { Ok(pair) }).await, Err(SelectionError::IdentityMismatch));
        assert!(matches!(selection.take(&id, Device::Pc), Err(SelectionError::NotReady)));
        assert!(matches!(selection.take(&id, Device::Phone), Err(SelectionError::NotReady)));
    }
    #[tokio::test]
    async fn cancellation_releases_pending_selection_and_refresh_revokes_unclaimed_lease() {
        let selection = MediaSelection::new(); let media = MediaIdentity::torbox(1, 2);
        let ticket = selection.select(media.clone()).unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(ticket.resolve_with(async move { tx.send(()).unwrap(); std::future::pending().await }));
        rx.await.unwrap(); task.abort(); let _ = task.await;
        assert_eq!(selection.status().unwrap(), SelectionStatus::Failed(SelectionError::Cancelled));
        let ticket = selection.select(media.clone()).unwrap(); let id = ticket.id();
        ticket.resolve_with(async { Ok(streams(&media)) }).await.unwrap();
        let _ = selection.take(&id, Device::Pc).unwrap();
        let refresh = selection.select(media).unwrap();
        assert!(matches!(selection.take(&id, Device::Phone), Err(SelectionError::Superseded)));
        drop(refresh);
        assert_eq!(selection.status().unwrap(), SelectionStatus::Failed(SelectionError::Cancelled));
    }
    #[tokio::test]
    async fn already_taken_links_are_rejected_at_delayed_handoff_after_replacement_or_refresh() {
        for same_file in [false, true] {
            let selection = MediaSelection::new(); let media = MediaIdentity::torbox(1, 2);
            let ticket = selection.select(media.clone()).unwrap(); let id = ticket.id();
            ticket.resolve_with(async { Ok(streams(&media)) }).await.unwrap();
            let pc = selection.take(&id, Device::Pc).unwrap();
            let phone = selection.take(&id, Device::Phone).unwrap();
            let (tx, rx) = tokio::sync::oneshot::channel();
            let queued = tokio::spawn(async move {
                rx.await.unwrap();
                pc.deliver(|_| panic!("stale native handoff executed"))
            });
            let next_media = if same_file { media } else { MediaIdentity::torbox(2, 3) };
            let next = selection.select(next_media.clone()).unwrap(); let next_id = next.id();
            next.resolve_with(async { Ok(streams(&next_media)) }).await.unwrap();
            tx.send(()).unwrap();
            assert_eq!(queued.await.unwrap(), Err(SelectionError::Superseded));
            assert_eq!(phone.deliver(|_| panic!("stale phone handoff executed")), Err(SelectionError::Superseded));
            assert_eq!(selection.take(&next_id, Device::Pc).unwrap().deliver(|lease| lease.identity().clone()).unwrap(), next_media);
        }
    }
    #[tokio::test]
    async fn taken_links_cannot_escape_close_owner_drop_or_invalid_choice() {
        for boundary in ["close", "drop", "invalid"] {
            let selection = MediaSelection::new(); let media = MediaIdentity::torbox(1, 2);
            let ticket = selection.select(media.clone()).unwrap(); let id = ticket.id();
            ticket.resolve_with(async { Ok(streams(&media)) }).await.unwrap();
            let lease = selection.take(&id, Device::Phone).unwrap();
            let expected = if boundary == "invalid" {
                assert!(matches!(selection.select(MediaIdentity::fixture()), Err(SelectionError::InvalidMedia)));
                SelectionError::Superseded
            } else { selection.close(); SelectionError::Closed };
            if boundary == "drop" { drop(selection); }
            assert_eq!(lease.deliver(|_| panic!("revoked link escaped")), Err(expected));
        }
    }
    #[tokio::test]
    async fn clearing_unloads_and_cancels_fetches_without_disabling_next_selection() {
        let selection = MediaSelection::new(); let media = MediaIdentity::torbox(1, 2);
        let first = selection.select(media.clone()).unwrap(); let id = first.id();
        first.resolve_with(async { Ok(streams(&media)) }).await.unwrap();
        let queued = selection.take(&id, Device::Pc).unwrap();
        selection.clear().unwrap();
        assert_eq!(selection.status().unwrap(), SelectionStatus::Empty);
        assert_eq!(queued.deliver(|_| panic!("unloaded source delivered")), Err(SelectionError::Superseded));
        let pending = selection.select(media.clone()).unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(pending.resolve_with(async move { tx.send(()).unwrap(); std::future::pending().await }));
        rx.await.unwrap(); selection.clear().unwrap();
        assert_eq!(task.await.unwrap(), Err(SelectionError::Superseded));
        assert_eq!(selection.status().unwrap(), SelectionStatus::Empty);
        let next = selection.select(media.clone()).unwrap(); let id = next.id();
        next.resolve_with(async { Ok(streams(&media)) }).await.unwrap();
        assert!(selection.take(&id, Device::Phone).unwrap().deliver(|_| ()).is_ok());
        selection.close(); assert_eq!(selection.clear(), Err(SelectionError::Closed));
    }
    #[tokio::test]
    async fn exhausted_sequence_closes_instead_of_preserving_old_delivery() {
        let selection = MediaSelection::new(); let media = MediaIdentity::torbox(1, 2);
        let ticket = selection.select(media.clone()).unwrap(); let id = ticket.id();
        ticket.resolve_with(async { Ok(streams(&media)) }).await.unwrap();
        let queued = selection.take(&id, Device::Pc).unwrap();
        selection.state.lock().unwrap().sequence = u64::MAX;
        assert_eq!(selection.clear(), Err(SelectionError::Unavailable));
        assert_eq!(selection.status().unwrap(), SelectionStatus::Closed);
        assert_eq!(queued.deliver(|_| panic!("sequence exhaustion leaked source")), Err(SelectionError::Closed));
    }
    #[tokio::test]
    async fn shutdown_and_owner_drop_fence_late_results() {
        for drop_owner in [false, true] {
            let selection = MediaSelection::new(); let media = MediaIdentity::torbox(1, 2);
            let ticket = selection.select(media.clone()).unwrap();
            let (started_tx, started_rx) = tokio::sync::oneshot::channel();
            let (finish_tx, finish_rx) = tokio::sync::oneshot::channel();
            let task = tokio::spawn(ticket.resolve_with(async move { started_tx.send(()).unwrap(); finish_rx.await.unwrap(); Ok(streams(&media)) }));
            started_rx.await.unwrap();
            selection.close();
            assert!(matches!(selection.select(MediaIdentity::torbox(2, 1)), Err(SelectionError::Closed)));
            if drop_owner { drop(selection); }
            assert_eq!(task.await.unwrap(), Err(SelectionError::Closed));
            assert!(finish_tx.send(()).is_err());
        }
    }
}

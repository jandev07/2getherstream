use tokio::sync::OwnedMutexGuard;

/// Blocking lifecycle work keeps its state lease even if the awaiting caller leaves.
/// The existing blocking task owns completion; no second operation may enter early.
pub(crate) async fn blocking<T,R,F>(mut state:OwnedMutexGuard<T>,change:F)->Result<R,tokio::task::JoinError>
where T:Send+'static,R:Send+'static,F:FnOnce(&mut T)->R+Send+'static {
    tokio::task::spawn_blocking(move ||change(&mut state)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::{Mutex,oneshot};
    use std::time::Duration;

    #[tokio::test] async fn cancelled_waiter_keeps_state_reserved_until_blocking_publication() {
        let state=Arc::new(Mutex::new(0));
        let (entered_tx,entered_rx)=oneshot::channel();let (release_tx,release_rx)=std::sync::mpsc::channel();
        let guard=state.clone().lock_owned().await;
        let waiter=tokio::spawn(blocking(guard,move |value| {
            entered_tx.send(()).unwrap();release_rx.recv_timeout(Duration::from_secs(3)).unwrap();*value=1;
        }));
        entered_rx.await.unwrap();waiter.abort();assert!(waiter.await.unwrap_err().is_cancelled());
        let reserved=state.try_lock().is_err();release_tx.send(()).unwrap();
        let value=tokio::time::timeout(Duration::from_secs(3),state.lock()).await.unwrap();
        assert!(reserved,"cancelled waiter released state before background completion");assert_eq!(*value,1);
    }
    // Isolated old caller-held lease pattern; no actual process or Tauri command.
    #[tokio::test] async fn borrowed_caller_guard_comparison_admits_before_mock_completion() {
        let state=Arc::new(Mutex::new(0));let target=state.clone();
        let (entered_tx,entered_rx)=oneshot::channel();let (release_tx,release_rx)=std::sync::mpsc::channel();
        let (finished_tx,finished_rx)=oneshot::channel();
        let waiter=tokio::spawn(async move {
            let mut guard=target.lock().await;
            let result=tokio::task::spawn_blocking(move || {
                entered_tx.send(()).unwrap();release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                let _=finished_tx.send(());1
            }).await.unwrap();
            *guard=result;
        });
        entered_rx.await.unwrap();waiter.abort();assert!(waiter.await.unwrap_err().is_cancelled());
        let admitted=state.try_lock().is_ok();release_tx.send(()).unwrap();finished_rx.await.unwrap();
        assert!(admitted,"old caller pattern did not reproduce premature admission");
        assert_eq!(*state.lock().await,0,"cancelled old caller must not publish late result");
    }
    #[tokio::test] async fn queued_waiter_cancellation_never_enters_blocking_work() {
        let state=Arc::new(Mutex::new(0));let held=state.lock().await;
        let invoked=Arc::new(std::sync::atomic::AtomicBool::new(false));
        let target=state.clone();let called=invoked.clone();
        let (attempted_tx,attempted_rx)=oneshot::channel();
        let waiter=tokio::spawn(async move {
            attempted_tx.send(()).unwrap();
            let guard=target.lock_owned().await;
            blocking(guard,move |_|{called.store(true,std::sync::atomic::Ordering::Release)}).await.unwrap();
        });
        attempted_rx.await.unwrap();waiter.abort();assert!(waiter.await.unwrap_err().is_cancelled());drop(held);
        assert!(!invoked.load(std::sync::atomic::Ordering::Acquire));
    }
    #[tokio::test] async fn next_transition_observes_completion_after_first_waiter_cancelled() {
        let state=Arc::new(Mutex::new(Vec::<u8>::new()));
        let (entered_tx,entered_rx)=oneshot::channel();let (release_tx,release_rx)=std::sync::mpsc::channel();
        let waiter=tokio::spawn(blocking(state.clone().lock_owned().await,move |value| {
            entered_tx.send(()).unwrap();release_rx.recv_timeout(Duration::from_secs(3)).unwrap();value.push(1);
        }));
        entered_rx.await.unwrap();waiter.abort();let _=waiter.await;
        let next_state=state.clone();let next=tokio::spawn(async move {
            blocking(next_state.lock_owned().await,|value|{assert_eq!(value.as_slice(),[1]);value.push(2)}).await.unwrap();
        });
        let reserved=state.try_lock().is_err();release_tx.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(3),next).await.unwrap().unwrap();
        assert!(reserved);assert_eq!(state.lock().await.as_slice(),[1,2]);
    }
    #[tokio::test] async fn failed_or_panicking_operation_releases_state_without_inventing_success() {
        let state=Arc::new(Mutex::new(0));
        let result=blocking(state.clone().lock_owned().await,|_|Err::<(),_>("fixed stop failure")).await.unwrap();
        assert_eq!(result,Err("fixed stop failure"));assert!(state.try_lock().is_ok());
        let result=blocking(state.clone().lock_owned().await,|_|panic!("controlled mock transition panic")).await;
        assert!(result.unwrap_err().is_panic());assert!(state.try_lock().is_ok());
    }
}

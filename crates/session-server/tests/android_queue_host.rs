//! Explicit owned loopback bridge for Android's real socket/ACK controller.
//! The PC reports synthetic readiness/execution; it never decodes media.
use together_session_server::{Host,Action};
use together_sync_core::MediaIdentity;
use std::{path::PathBuf,time::Duration};

#[tokio::test]
#[ignore="Requires owned Android queued-control instrumentation and lifecycle runner"]
async fn android_queue_loopback_host() {
    let directory=PathBuf::from(std::env::var_os("TOGETHER_ANDROID_QUEUE_DIR").expect("Owned proof directory required"));
    let run=uuid::Uuid::parse_str(&std::env::var("TOGETHER_ANDROID_QUEUE_RUN").expect("Owned run required")).unwrap();
    std::fs::create_dir_all(&directory).unwrap();
    const PHONE:&str="synthetic-android-queue-phone-0000000000";
    let host=Host::new("synthetic-android-queue-pc-000000000000".into(),PHONE.into()).unwrap();
    host.enable_diagnostics(directory.join("journal")).unwrap();
    host.attach_local_pc().await.unwrap();
    host.local_pc(None,Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
    let serving=host.clone();let server=tokio::spawn(async move {serving.serve(listener).await});
    let (stop_pc,mut stopped)=tokio::sync::watch::channel(false);let pc_host=host.clone();
    let pc=tokio::spawn(async move {
        // Only a test participant; no application scheduling loop is added.
        let mut tick=tokio::time::interval(Duration::from_millis(20));
        loop {
            tokio::select! {biased; _=stopped.changed()=>break, _=tick.tick()=>{}}
            let state=pc_host.snapshot().await;let now=pc_host.now();
            let action=if let Some(prepare)=&state.prepare {
                if prepare.ready.contains("pc") {continue;}
                Action::Ready {position_ms:prepare.position_ms}
            } else {
                if state.fault.is_some() || state.ended || (state.timeline.playing && now<state.timeline.anchor_ms.saturating_add(25)) ||
                    state.participants["pc"].execution.as_ref().is_some_and(|e|e.generation==state.generation && e.playing==state.timeline.playing) {continue;}
                Action::Executed {playing:state.timeline.playing,position_ms:state.timeline.position(now),observed_server_ms:now}
            };
            let _=pc_host.local_pc(Some(state.generation),action).await;
        }
    });
    std::fs::write(directory.join("host-ready.json"),serde_json::to_vec(&serde_json::json!({
        "run_id":run.to_string(),"port":address.port(),"phone_token":PHONE
    })).unwrap()).unwrap();
    let completed=tokio::time::timeout(Duration::from_secs(120),async {
        while !directory.join("client-done").is_file() {tokio::time::sleep(Duration::from_millis(50)).await;}
    }).await.is_ok();
    let detached=tokio::time::timeout(Duration::from_secs(3),async {
        while host.snapshot().await.participants["phone"].connected {tokio::time::sleep(Duration::from_millis(10)).await;}
    }).await.is_ok();
    stop_pc.send_replace(true);pc.await.unwrap();
    host.request_shutdown();server.await.unwrap().unwrap();host.detach_local_pc().await;
    let journal=host.export_diagnostics().await.unwrap();std::fs::write(directory.join("host-journal.jsonl"),&journal).unwrap();
    let rows:Vec<serde_json::Value>=String::from_utf8(journal).unwrap().lines().map(|line|serde_json::from_str(line).unwrap()).collect();
    let commands:Vec<_>=rows.iter().filter_map(|row| {
        let sample=&row["record"]["sample"];
        (row["record"]["event"]=="command" && sample["actor"]=="phone" &&
            (sample["kind"]=="play" || sample["kind"]=="pause")).then_some(sample.clone())
    }).collect();
    let health=host.diagnostic_health().unwrap();
    let rebound=tokio::net::TcpListener::bind(address).await.is_ok();
    let client=std::fs::read(directory.join("client.json")).ok().and_then(|bytes|serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    let matched=client.as_ref().is_some_and(|client|commands.len()==2 &&
        client["run_id"]==run.to_string() && commands[0]["kind"]=="play" && commands[1]["kind"]=="pause" &&
        commands.iter().all(|sample|sample["outcome"]=="accepted") &&
        commands[0]["generation"]==client["base_generation"] && commands[0]["sequence"]==client["play_sequence"] &&
        commands[0]["resulting_generation"].as_u64()==client["base_generation"].as_u64().map(|g|g+1) &&
        commands[1]["generation"]==commands[0]["resulting_generation"] &&
        commands[1]["resulting_generation"].as_u64()==client["base_generation"].as_u64().map(|g|g+2));
    std::fs::write(directory.join("host-receipt.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "run_id":run.to_string(),"client_completed":completed,"phone_detached":detached,"listener_rebound":rebound,
        "phone_user_commands":commands,"client_journal_match":matched,"journal_io_errors":health.io_errors,
        "journal_dropped":health.dropped_records,"pc_native_decode":false,"physical_gate_passed":false
    })).unwrap()).unwrap();
    assert!(completed && detached && rebound && matched);assert_eq!(health.io_errors,0);assert_eq!(health.dropped_records,0);
}

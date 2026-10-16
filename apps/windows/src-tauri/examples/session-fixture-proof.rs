//! Owned, headless and muted integration host for the local Android session test.
//! Uses the production PC follower, but no provider credentials or media delivery.
#[path="../src/player.rs"]
mod player;
use std::{path::PathBuf,sync::Arc,time::Duration};
use together_session_server::{Action,Host};
use together_sync_core::MediaIdentity;

async fn run()->Result<(),&'static str> {
    let root=PathBuf::from(std::env::args().nth(1).ok_or("runtime_required")?);
    let run=uuid::Uuid::parse_str(&std::env::var("TOGETHER_PROOF_RUN_ID").map_err(|_|"run_required")?)
        .map_err(|_|"run_invalid")?;
    if run.is_nil() {return Err("run_invalid");}
    let token=std::env::var("TOGETHER_PHONE_TOKEN").map_err(|_|"proof_credential_required")?;
    let listener=tokio::net::TcpListener::bind("127.0.0.1:47822").await.map_err(|_|"isolated_listener_unavailable")?;
    let host=Host::new(uuid::Uuid::new_v4().to_string()+&uuid::Uuid::new_v4().to_string(),token)
        .map_err(|_|"host_init_failed")?;
    host.enable_diagnostics(root.join("work/native-journal-proof")).map_err(|_|"journal_failed")?;
    let serving=host.clone();let server=tokio::spawn(async move {serving.serve(listener).await});
    let native=player::PlayerController::start(root.clone(),0,Arc::clone(&host));
    let result=async {
        tokio::time::timeout(Duration::from_secs(15),async {
            loop {
                if native.ensure_available().is_ok() && host.snapshot().await.participants["pc"].connected {break;}
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }).await.map_err(|_|"native_pc_startup_failed")?;
        host.local_pc(None,Action::Load {media:MediaIdentity::fixture()}).await.map_err(|_|"fixture_load_failed")?;
        std::fs::write(root.join("work/fixture-host-ready.json"),
            serde_json::to_vec(&serde_json::json!({"run_id":run.to_string(),"port":47822})).unwrap())
            .map_err(|_|"ready_write_failed")?;
        let deadline=tokio::time::Instant::now()+Duration::from_secs(3900);
        loop {
            if root.join("work/fixture-host-stop").is_file() {break;}
            if tokio::time::Instant::now()>=deadline {return Err("fixture_host_deadline");}
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Ok(())
    }.await;
    host.request_shutdown();
    let native_result=native.shutdown().await;
    let server_result=tokio::time::timeout(Duration::from_secs(15),server).await;
    let journal=host.export_diagnostics().await.map_err(|_|"journal_export_failed")?;
    std::fs::write(root.join("work/fixture-host-journal.jsonl"),journal).map_err(|_|"journal_write_failed")?;
    let health=host.diagnostic_health().ok_or("journal_health_missing")?;
    let native_stopped=native.status.lock().is_ok_and(|status|!status.initialized && status.state=="Stopped");
    let server_stopped=matches!(server_result,Ok(Ok(Ok(()))));
    std::fs::write(root.join("work/fixture-host-receipt.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "run_id":run.to_string(),"requested_flow_completed":result.is_ok(),"native_shutdown":native_result.is_ok(),
        "native_stopped":native_stopped,"server_stopped":server_stopped,
        "journal_io_errors":health.io_errors,"journal_dropped":health.dropped_records,
        "provider_used":false,"physical_gate_passed":false
    })).unwrap()).map_err(|_|"receipt_write_failed")?;
    if native_result.is_err() || !native_stopped || !server_stopped || health.io_errors!=0 || health.dropped_records!=0 {
        return Err("fixture_owner_cleanup_failed");
    }
    result
}
#[tokio::main]
async fn main() {
    if let Err(code)=run().await {eprintln!("FIXTURE_SESSION_FAILED code={code}");std::process::exit(1);}
}

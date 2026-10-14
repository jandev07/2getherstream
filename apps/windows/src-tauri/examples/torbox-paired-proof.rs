//! Headless, muted real-provider harness. Reuses the production native controller.
//! No credentials, URLs, filenames or account responses are printed.
#[path="../src/player.rs"]
mod player;
use std::{path::PathBuf,sync::Arc,time::Duration};
use together_session_server::Host;
use together_source_resolver::{secret,TorBox};

async fn run()->Result<(),&'static str> {
    let root=PathBuf::from(std::env::args().nth(1).ok_or("runtime_required")?);
    let token=std::env::var("TOGETHER_PHONE_TOKEN").map_err(|_|"proof_credential_required")?;
    // Reserve the isolated loopback endpoint before any player/provider work.
    let listener=tokio::net::TcpListener::bind("127.0.0.1:47822").await.map_err(|_|"isolated_listener_unavailable")?;
    let host=Host::new(uuid::Uuid::new_v4().to_string()+&uuid::Uuid::new_v4().to_string(),token).map_err(|_|"host_init_failed")?;
    host.enable_diagnostics(root.join("work/native-journal-proof")).map_err(|_|"diagnostic_owner_failed")?;
    let server_host=host.clone();
    let server=tokio::spawn(async move {server_host.serve(listener).await});
    let native=player::PlayerController::start(root.clone(),0,Arc::clone(&host));
    let result=async {
        // The production controller owns PC attachment and feedback. Do not
        // attach independently: a duplicate owner makes that task exit early.
        tokio::time::timeout(Duration::from_secs(10),async {
            loop {
                let initialized=native.status.lock().is_ok_and(|status|status.initialized && status.error.is_none());
                if initialized && host.snapshot().await.participants["pc"].connected {break;}
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }).await.map_err(|_|"native_pc_owner_startup_failed")?;
        let vault=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../.secrets/torbox.dpapi");
        let client=TorBox::new_private_alpha(secret::read_key(&vault).map_err(|_|"provider_credential_unavailable")?).map_err(|_|"provider_init_failed")?;
        let videos=client.available_videos().await.map_err(|_|"provider_inventory_failed")?;
        let media=videos.first().ok_or("provider_video_unavailable")?;
        let streams=client.device_streams(media,None,None).await.map_err(|_|"provider_capabilities_failed")?;
        tokio::try_join!(client.probe(&streams.pc),client.probe(&streams.phone)).map_err(|_|"provider_ranges_failed")?;
        let before=host.snapshot().await;
        host.load_resolved_media(&before.epoch,before.generation,streams).await.map_err(|_|"provider_publication_failed")?;
        let code=host.open_pairing().await.map_err(|_|"pairing_window_failed")?;
        tokio::fs::write(root.join("work/pairing-code-proof.txt"),code).await.map_err(|_|"pairing_nonce_write_failed")?;
        println!("REAL_PROVIDER_HOST_READY muted=true headless=true physical_gate=false");
        let deadline=tokio::time::Instant::now()+Duration::from_secs(600);
        loop {
            if root.join("work/startup-proof-stop.txt").is_file(){break;}
            if tokio::time::Instant::now()>=deadline{return Err("provider_harness_deadline");}
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Ok(())
    }.await;
    host.request_shutdown();
    let native_result=native.shutdown().await;
    let server_result=tokio::time::timeout(Duration::from_secs(15),server).await;
    if native_result.is_err() || !matches!(server_result,Ok(Ok(Ok(())))) {return Err("provider_owner_cleanup_failed");}
    result
}
#[tokio::main]
async fn main() {
    if let Err(code)=run().await {
        eprintln!("REAL_PROVIDER_PROOF_FAILED code={code}");std::process::exit(1);
    }
}

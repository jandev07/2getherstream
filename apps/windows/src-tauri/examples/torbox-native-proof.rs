//! Muted/headless PC-only provider decode proof. No Android, tunnel or account writes.
use std::{path::Path,time::{Duration,Instant}};
use together_player_mpv::Mpv;
use together_source_resolver::{secret,TorBox};
#[path="support/provider_media.rs"]
mod provider_media;
fn check_events(player:&mut Mpv)->Result<(),&'static str> {
    while let Some(event)=player.poll() {if event.source_failure || event.error<0 {return Err("native_provider_event_failed");}}
    if player.take_command_failure() {return Err("native_provider_command_failed");}Ok(())
}
fn wait(player:&mut Mpv,seconds:u64,condition:impl Fn(&mut Mpv)->bool)->Result<(),&'static str> {
    let deadline=Instant::now()+Duration::from_secs(seconds);
    while Instant::now()<deadline {check_events(player)?;if condition(player) {return Ok(());}std::thread::sleep(Duration::from_millis(50));}
    Err("native_provider_phase_timeout")
}
fn play_progress(player:&mut Mpv,milliseconds:u64)->Result<(u64,u64),&'static str> {
    let before=player.position_ms().map_err(|_|"native_provider_position_missing")?;
    player.pause(false).map_err(|_|"native_provider_play_failed")?;
    wait(player,30,|player|player.position_ms().is_ok_and(|position|position>=before+milliseconds))?;
    let after=player.position_ms().map_err(|_|"native_provider_position_missing")?;
    Ok((before,after))
}
async fn run()->Result<serde_json::Value,&'static str> {
    let args:Vec<_>=std::env::args().collect();
    let dll=args.get(1).ok_or("native_provider_dll_required")?;
    let selection=provider_media::Selection::from_environment()?;
    let vault=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.secrets/torbox.dpapi");
    let client=TorBox::new_private_alpha(secret::read_key(&vault).map_err(|_|"native_provider_credential_unavailable")?).map_err(|_|"native_provider_client_failed")?;
    let videos=client.available_videos().await.map_err(|_|"native_provider_inventory_failed")?;
    let media=selection.select(&videos)?;
    let pair=client.device_streams(media,None,None).await.map_err(|_|"native_provider_links_failed")?;
    if pair.pc.identity()!=pair.phone.identity() {return Err("native_provider_identity_mismatch");}
    let (pc_bytes,phone_candidate_bytes)=tokio::try_join!(client.probe(&pair.pc),client.probe(&pair.phone)).map_err(|_|"native_provider_ranges_failed")?;
    let mut player=Mpv::open_with_options(Path::new(dll),&[("vo","null"),("ao","null"),("mute","yes"),("msg-level","all=no"),
        ("demuxer-max-bytes","33554432"),("demuxer-max-back-bytes","8388608")]).map_err(|_|"native_provider_initialization_failed")?;
    if player.property("tls-verify").ok().as_deref()!=Some("yes") {return Err("native_provider_tls_verification_disabled");}
    player.prepare_url(pair.pc.player_url(),1).map_err(|_|"native_provider_load_failed")?;
    wait(&mut player,45,|player|player.source_ready())?;
    let duration=player.property("duration").map_err(|_|"native_provider_duration_missing")?.parse::<f64>().map_err(|_|"native_provider_duration_invalid")?;
    if !duration.is_finite() || duration<40.0 {return Err("native_provider_fixture_too_short");}
    if player.property("pause").ok().as_deref()!=Some("yes") {return Err("native_provider_prepare_was_not_paused");}
    let (first_start,first_end)=play_progress(&mut player,12000)?;
    player.pause(true).map_err(|_|"native_provider_pause_failed")?;
    std::thread::sleep(Duration::from_millis(150));
    let paused=player.position_ms().map_err(|_|"native_provider_position_missing")?;
    let deadline=Instant::now()+Duration::from_secs(2);let mut pause_max_drift=0;
    while Instant::now()<deadline {
        check_events(&mut player)?;let position=player.position_ms().map_err(|_|"native_provider_position_missing")?;
        pause_max_drift=pause_max_drift.max(position.abs_diff(paused));std::thread::sleep(Duration::from_millis(50));
    }
    if pause_max_drift>250 {return Err("native_provider_pause_drift");}
    let seek_target=((duration*500.0) as u64).min(45000);
    player.seek(seek_target,2).map_err(|_|"native_provider_seek_failed")?;
    wait(&mut player,45,|player|player.property("seeking").ok().as_deref()==Some("no") && player.position_ms().is_ok_and(|position|position.abs_diff(seek_target)<=1500))?;
    let seek_landed=player.position_ms().map_err(|_|"native_provider_position_missing")?;
    let (second_start,second_end)=play_progress(&mut player,12000)?;
    player.pause(true).map_err(|_|"native_provider_final_pause_failed")?;check_events(&mut player)?;
    Ok(serde_json::json!({"status":"passed","kind":"pc_only_torbox_native","account_bearing_private_exception":true,
        "muted":true,"headless":true,"pc_range_bytes":pc_bytes,"phone_candidate_range_bytes":phone_candidate_bytes,
        "torrent_id":media.torrent_id,"file_id":media.file_id,"selection_mode":selection.mode(),"duration_ms":(duration*1000.0) as u64,
        "first_start_ms":first_start,"first_end_ms":first_end,"pause_max_drift_ms":pause_max_drift,
        "seek_target_ms":seek_target,"seek_landed_ms":seek_landed,"second_start_ms":second_start,"second_end_ms":second_end,
        "android_handoff":false,"physical_gate_passed":false}))
}
#[tokio::main]
async fn main() {
    match run().await {
        Ok(report)=>println!("{report}"),
        Err(code)=>{println!("{}",serde_json::json!({"status":"failed","kind":"pc_only_torbox_native","failure":code,"android_handoff":false,"physical_gate_passed":false}));std::process::exit(1);}
    }
}

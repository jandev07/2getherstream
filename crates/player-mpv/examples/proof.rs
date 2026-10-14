//! Actual native decoder smoke test. No simulation; uses a generated local fixture.
use anyhow::{ensure, Result};
use std::{path::PathBuf, thread, time::{Duration, Instant}};
use together_player_mpv::Mpv;
fn wait(player: &mut Mpv, condition: impl Fn(&mut Mpv) -> bool) -> Result<()> {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        while let Some(event) = player.poll() { ensure!(event.error >= 0, "native command error {}",event.error); }
        if condition(player) { return Ok(()); }
        thread::sleep(Duration::from_millis(10));
    }
    anyhow::bail!("native player condition timed out")
}
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let dll = PathBuf::from(args.next().expect("absolute DLL path"));
    let file = args.next().expect("absolute local test video path");
    let mut player = Mpv::open_with_options(&dll, &[("vo","null"),("ao","null")])?;
    println!("Native version: {}",player.property("mpv-version")?);
    player.prepare_local(std::path::Path::new(&file))?;
    wait(&mut player,|p| p.property("duration").ok().and_then(|s| s.parse::<f64>().ok()).is_some_and(|n| n > 59.0))?;
    ensure!(player.property("pause")? == "yes", "load did not remain paused");
    ensure!(player.property("track-list/count")?.parse::<u64>()? >= 2,"missing video/audio tracks");
    player.pause(false)?;
    wait(&mut player,|p| p.position_ms().is_ok_and(|n| n > 300))?;
    player.pause(true)?;
    let before = player.position_ms()?;
    thread::sleep(Duration::from_millis(200));
    ensure!(before.abs_diff(player.position_ms()?) <= 30,"pause did not stop position");
    player.seek(20_000,2)?;
    wait(&mut player,|p| p.position_ms().is_ok_and(|n| n.abs_diff(20_000) <= 80) && p.property("seeking").is_ok_and(|s| s == "no"))?;
    ensure!(player.property("paused-for-cache")? == "no", "unexpected fixture buffering");
    player.pause(false)?;
    wait(&mut player,|p| p.position_ms().is_ok_and(|n| n > 20_300))?;
    player.pause(true)?;
    let missing=std::path::Path::new(&file).with_file_name(format!("missing-native-proof-{}.mp4",std::process::id()));
    ensure!(!missing.exists(),"failure proof requires a nonexistent source");
    player.prepare_local(&missing)?;
    let until=Instant::now()+Duration::from_secs(10);
    let mut failed=false;
    while Instant::now()<until && !failed {
        while let Some(event)=player.poll() {failed|=event.source_failure;}
        thread::sleep(Duration::from_millis(10));
    }
    ensure!(failed,"native terminal load error was not source-fenced");
    player.prepare_local(std::path::Path::new(&file))?;
    wait(&mut player,|p|p.source_ready())?;
    ensure!(player.property("pause")?=="yes","recovery resumed without authority");
    println!("PASS native load, duration, audio/video tracks, play, pause, exact seek, seeking and cache-state observation.");
    println!("PASS source-fenced terminal failure and explicit reload recovery.");
    Ok(())
}

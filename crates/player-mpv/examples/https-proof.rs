//! Real native HTTPS load/play/pause/seek against a pinned AndroidX public test asset.
use anyhow::{ensure, Result};
use std::{path::PathBuf, thread, time::{Duration, Instant}};
use together_player_mpv::Mpv;
const SOURCE: &str = "https://raw.githubusercontent.com/androidx/media/8c6678b657ede1e7883fc164ef73ed483c7796c3/libraries/test_data/src/test/assets/media/mp4/sample.mp4";
fn wait(player: &mut Mpv, condition: impl Fn(&mut Mpv) -> bool) -> Result<()> {
    let start=Instant::now();
    while start.elapsed()<Duration::from_secs(30) {
        while let Some(event)=player.poll() {ensure!(event.error>=0,"native HTTPS command failed");}
        if condition(player) {return Ok(());}
        thread::sleep(Duration::from_millis(10));
    }
    anyhow::bail!("native HTTPS condition timed out")
}
fn main() -> Result<()> {
    let dll=PathBuf::from(std::env::args().nth(1).expect("absolute native DLL path"));
    let mut player=Mpv::open_with_options(&dll,&[("vo","null"),("ao","null")])?;
    ensure!(player.property("tls-verify")?=="yes","native TLS verification default disabled");
    player.prepare_url(SOURCE,1)?;
    wait(&mut player,|p|p.source_ready() && p.property("duration").ok().and_then(|s|s.parse::<f64>().ok()).is_some_and(|d|d>0.5))?;
    ensure!(player.property("pause")?=="yes","HTTPS load did not stay paused");
    // Rejected API aliases must never replace the already-ready native source.
    // These are synthetic rejection inputs, never network requests or keys.
    for rejected in ["https://api.torbox.app./private-input", "https://%61pi.torbox.app./private-input"] {
        let error=player.prepare_url(rejected,2).unwrap_err();
        ensure!(error.to_string()=="Invalid media HTTPS URL","URL guard error exposed input");
        ensure!(player.source_ready(),"rejected source invalidated active media");
        ensure!(player.property("pause")?=="yes","rejected source changed native pause");
    }
    println!("PASS native Windows rejected API aliases preserve active HTTPS media");
    player.pause(false)?;
    wait(&mut player,|p|p.position_ms().is_ok_and(|pos|pos>200))?;
    player.pause(true)?;
    let before=player.position_ms()?;thread::sleep(Duration::from_millis(300));
    ensure!(before.abs_diff(player.position_ms()?)<=80,"HTTPS pause position moved");
    let duration=player.property("duration")?.parse::<f64>()?;
    let target=(duration*500.0) as u64;
    player.seek(target,2)?;
    wait(&mut player,|p|p.position_ms().is_ok_and(|pos|pos.abs_diff(target)<=80) && p.property("seeking").is_ok_and(|s|s=="no"))?;
    println!("PASS native Windows HTTPS load/play/pause/exact seek with TLS verification");
    if let Some(file)=std::env::args().nth(2) {
        player.prepare_url(SOURCE,3)?;
        player.prepare_local(&PathBuf::from(file))?;
        wait(&mut player,|p|p.source_ready() && p.property("duration").ok().and_then(|s|s.parse::<f64>().ok()).is_some_and(|d|d>=59.0))?;
        thread::sleep(Duration::from_millis(500));
        ensure!(player.source_ready(),"latest source lost readiness");
        ensure!(player.property("duration")?.parse::<f64>()?>=59.0,"old source replaced new selection");
        println!("PASS native Windows rapid source replacement with playlist entry readiness");
    }
    Ok(())
}

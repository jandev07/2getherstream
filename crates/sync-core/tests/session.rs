use together_sync_core::{Session, MediaIdentity, clock::ClockEstimator, drift::{DriftController, Correction}};
fn room() -> Session {
    let mut s = Session::new("test-epoch".into());
    s.connection("pc", true, 0).unwrap(); s.connection("phone", true, 0).unwrap();
    s.load(MediaIdentity::torbox(1,2), 0).unwrap(); s
}
#[test] fn native_end_is_fenced_and_replay_prepares_from_zero() {
    let mut s=room();s.intent(true,s.state.generation,0).unwrap();ready(&mut s,1);
    let generation=s.state.generation;
    assert!(s.end_of_media("phone",generation,59999,60000,500).is_err());
    assert!(s.end_of_media("phone",generation-1,59999,60000,60500).is_err());
    s.end_of_media("pc",generation,59967,60000,60501).unwrap();
    assert!(s.state.ended);assert!(!s.state.desired_playing);assert_eq!(s.state.timeline.position(90000),60000);
    s.connection("phone",false,61000).unwrap();s.connection("phone",true,62000).unwrap();
    assert!(s.state.ended);assert!(s.state.prepare.is_none());
    s.intent(true,s.state.generation,63000).unwrap();
    assert!(!s.state.ended);assert_eq!(s.state.prepare.as_ref().unwrap().position_ms,0);
    assert!(s.end_of_media("phone",generation,60000,60000,64000).is_err());
}
#[test] fn end_cannot_override_seek_or_user_pause() {
    let mut s=room();ready(&mut s,0);
    assert!(s.end_of_media("pc",s.state.generation,60000,60000,60500).is_err());
    s.intent(true,s.state.generation,61000).unwrap();
    assert!(s.end_of_media("pc",s.state.generation,60000,60000,61500).is_err());
    ready(&mut s,61001);s.seek(1000,s.state.generation,61600).unwrap();
    assert!(s.end_of_media("pc",s.state.generation,60000,60000,62000).is_err());
    assert_eq!(s.state.prepare.as_ref().unwrap().position_ms,1000);
}
fn ready(s: &mut Session, now: u64) {
    let g = s.state.generation; let pos = s.state.prepare.as_ref().unwrap().position_ms;
    s.ready("pc", g, pos, now).unwrap(); s.ready("phone", g, pos, now + 1).unwrap();
}
#[test] fn ready_accepts_frame_quantization_but_rejects_out_of_bounds_positions() {
    let mut s=room();s.seek(26513,s.state.generation,10).unwrap();
    let g=s.state.generation;
    assert_eq!(s.ready("pc",g,26614,20),Err("player has not reached target"));
    assert_eq!(s.ready("phone",g,26412,20),Err("player has not reached target"));
    assert!(s.state.prepare.as_ref().unwrap().ready.is_empty());
    s.ready("pc",g,26613,21).unwrap();
    s.ready("phone",g,26600,22).unwrap();
    assert!(s.state.prepare.is_none());
    assert!(!s.state.timeline.playing);
    assert_eq!(s.state.timeline.position_ms,26513);
}
#[test] fn execution_requires_current_commit_and_real_state() {
    let mut s=room();let g=s.state.generation;
    assert_eq!(s.executed("pc",g,false,0,0,0),Err("no committed playback"));
    s.intent(true,g,0).unwrap();ready(&mut s,0);let g=s.state.generation;
    assert_eq!(s.executed("phone",g,true,0,500,500),Err("execution before scheduled start"));
    assert_eq!(s.executed("phone",g,false,0,600,600),Err("execution state mismatch"));
    assert_eq!(s.executed("phone",g,true,10000,600,600),Err("execution position mismatch"));
    assert_eq!(s.executed("phone",g,true,100,600,9000),Err("execution observation expired"));
    s.executed("pc",g,true,99,600,600).unwrap();s.executed("phone",g,true,199,700,1000).unwrap();
    s.tick(10000);assert!(s.state.fault.is_none());assert!(s.state.timeline.playing);
    s.intent(false,g,10000).unwrap();
    assert_eq!(s.executed("phone",g,true,9999,10000,10000),Err("stale generation"));
}
#[test] fn decoder_failure_cannot_be_cleared_by_buffering_and_retry_preserves_pause() {
    use together_sync_core::PlayerFault;
    let mut s=room();s.intent(true,s.state.generation,0).unwrap();ready(&mut s,0);let old=s.state.generation;
    s.player_failed("phone",old,PlayerFault::DecoderFailed,2000).unwrap();let g=s.state.generation;
    assert!(!s.state.timeline.playing);assert!(s.state.desired_playing);
    s.buffering("phone",g,true,2001).unwrap();s.buffering("phone",g,false,2002).unwrap();
    assert_eq!(s.state.generation,g);assert_eq!(s.state.fault.as_deref(),Some("player_failed_phone"));assert!(s.state.prepare.is_none());
    assert!(s.player_failed("pc",old,PlayerFault::LoadFailed,2003).is_err());
    s.intent(false,g,2100).unwrap();let paused=s.state.generation;
    s.player_recovered("phone",paused,2200).unwrap();let new=s.state.generation;
    assert!(s.state.prepare.is_some());assert!(s.state.participants["phone"].player_fault.is_none());
    assert!(s.ready("phone",paused,0,2201).is_err());
    let target=s.state.prepare.as_ref().unwrap().position_ms;
    s.ready("pc",new,target,2300).unwrap();s.ready("phone",new,target,2301).unwrap();
    assert!(!s.state.timeline.playing);assert!(!s.state.desired_playing);
}
#[test] fn a_failed_retry_with_same_native_error_stops_the_new_prepare() {
    use together_sync_core::PlayerFault;
    let mut s=room();s.player_failed("phone",s.state.generation,PlayerFault::DecoderFailed,0).unwrap();
    let failed=s.state.generation;s.player_failed("phone",failed,PlayerFault::DecoderFailed,1).unwrap();assert_eq!(s.state.generation,failed);
    s.intent(true,failed,100).unwrap();let retry=s.state.generation;assert!(s.state.prepare.is_some());
    s.player_failed("phone",retry,PlayerFault::DecoderFailed,200).unwrap();
    assert!(s.state.generation>retry);assert!(s.state.prepare.is_none());assert_eq!(s.state.fault.as_deref(),Some("player_failed_phone"));assert!(!s.state.timeline.playing);
}
#[test] fn missed_native_execution_freezes_both_and_explicit_play_retries() {
    let mut s=room();s.observe_rtt("phone",800).unwrap();s.intent(true,s.state.generation,0).unwrap();ready(&mut s,0);
    let g=s.state.generation;let anchor=s.state.timeline.anchor_ms;
    s.executed("pc",g,true,100,anchor+100,anchor+100).unwrap();
    s.tick(anchor+3599);assert!(s.state.fault.is_none());
    s.tick(anchor+3600);assert_eq!(s.state.fault.as_deref(),Some("execution_timeout_phone"));
    assert!(!s.state.timeline.playing);assert!(s.state.desired_playing);assert!(s.state.generation>g);
    let pos=s.state.timeline.position(anchor+8000);assert_eq!(pos,3600);
    assert!(s.executed("phone",g,true,3600,anchor+3600,anchor+3600).is_err());
    s.intent(true,s.state.generation,anchor+8000).unwrap();assert!(s.state.fault.is_none());assert_eq!(s.state.prepare.as_ref().unwrap().position_ms,pos);
}
#[test] fn no_start_until_both_ready_then_future_commit() {
    let mut s = room(); ready(&mut s, 0); s.intent(true, s.state.generation, 20).unwrap();
    let g = s.state.generation; s.ready("pc", g, 0, 30).unwrap();
    assert!(!s.state.timeline.playing); s.ready("phone", g, 0, 40).unwrap();
    assert_eq!(s.state.timeline.position(539), 0); assert_eq!(s.state.timeline.position(640), 100);
}
#[test] fn stale_ready_cannot_undo_new_seek() {
    let mut s = room(); let old = s.state.generation;
    s.seek(90_000, old, 5).unwrap(); assert_eq!(s.ready("phone", old, 0, 6), Err("stale generation"));
    assert_eq!(s.state.prepare.as_ref().unwrap().position_ms, 90_000);
}
#[test] fn buffer_preserves_intent_and_cancels_scheduled_commit() {
    let mut s = room(); s.intent(true, s.state.generation, 0).unwrap(); ready(&mut s, 1);
    s.buffering("phone", s.state.generation, true, 100).unwrap();
    assert!(s.state.desired_playing); assert!(!s.state.timeline.playing);
    assert_eq!(s.ready("phone", s.state.generation, 0, 101), Err("participant not available"));
    s.buffering("phone", s.state.generation, false, 200).unwrap(); ready(&mut s, 201);
    assert!(s.state.timeline.playing);
}
#[test] fn pause_during_buffering_stays_paused() {
    let mut s = room(); s.intent(true, s.state.generation, 0).unwrap(); ready(&mut s, 1);
    s.buffering("phone", s.state.generation, true, 1000).unwrap();
    s.intent(false, s.state.generation, 1100).unwrap();
    s.buffering("phone", s.state.generation, false, 1200).unwrap(); ready(&mut s, 1201);
    assert!(!s.state.timeline.playing); assert!(!s.state.desired_playing);
}
#[test] fn disconnect_and_reconnect_require_new_readiness() {
    let mut s = room(); s.intent(true, s.state.generation, 0).unwrap(); ready(&mut s, 1);
    s.connection("phone", false, 2000).unwrap(); let g = s.state.generation;
    assert!(!s.state.timeline.playing); s.connection("phone", true, 4000).unwrap();
    assert!(s.state.generation > g); assert!(s.state.prepare.as_ref().unwrap().ready.is_empty());
    ready(&mut s, 4001); assert!(s.state.timeline.playing);
}
#[test] fn readiness_checks_target_and_timeout() {
    let mut s = room(); assert_eq!(s.ready("pc", s.state.generation, 1000, 10), Err("player has not reached target"));
    let g = s.state.generation; s.tick(15_000); assert!(s.state.generation > g);
    assert_eq!(s.state.fault.as_deref(), Some("prepare_timeout"));
    assert!(!s.state.timeline.playing);
}
#[test] fn clock_uses_low_rtt_and_expires_samples() {
    let mut c = ClockEstimator::default(); c.observe(100.0, 620.0, 622.0, 142.0).unwrap();
    c.observe(200.0, 790.0, 792.0, 322.0).unwrap();
    assert_eq!(c.estimate(322.0).unwrap().offset_ms, 500.0);
    assert!(c.estimate(31_000.0).is_none()); assert!(c.observe(1.0, 2.0, 1.0, 3.0).is_err());
    c.reset(); assert!(c.estimate(400.0).is_none());
}
#[test] fn correction_is_bounded_and_requires_fresh_clock() {
    let mut d = DriftController::default();
    assert_eq!(d.update(1000,1400,0,1,true,true), Correction::Rate(0.995));
    assert_eq!(d.update(1000,1400,1000,1,true,true), Correction::Hold);
    assert_eq!(d.update(1000,6000,2000,1,true,true), Correction::Seek(1000));
    assert_eq!(d.update(5000,1000,3000,1,true,false), Correction::Rate(1.0));
    assert_eq!(d.update(5000,1000,4000,1,false,true), Correction::Rate(1.0));
}
#[test] fn syncplay_slowdown_and_delayed_follower_seek() {
    let mut d = DriftController::default();
    assert_eq!(d.update(10000,12000,0,1,true,true), Correction::Rate(0.95));
    assert_eq!(d.update(10000,11000,1000,1,true,true), Correction::Hold);
    assert_eq!(d.update(10000,10050,2000,1,true,true), Correction::Rate(1.0));
    d.reset();
    assert_eq!(d.update(10000,4000,0,1,true,true), Correction::Rate(1.005));
    assert_eq!(d.update(10000,4000,3000,1,true,true), Correction::Hold);
    assert_eq!(d.update(10000,4000,4000,1,true,true), Correction::Seek(10000));
}
#[test] fn generation_and_buffer_barriers_restore_nominal_speed() {
    let mut d = DriftController::default();
    assert_eq!(d.update(1000,1400,0,1,true,true), Correction::Rate(0.995));
    assert_eq!(d.update(1000,1400,1000,2,true,true), Correction::Rate(1.0));
    assert_eq!(d.update(1000,1400,2000,2,true,true), Correction::Rate(0.995));
    assert_eq!(d.update(1000,1400,3000,2,false,true), Correction::Rate(1.0));
}
#[test] fn two_hour_virtual_run_with_rate_error_and_jitter() {
    // Algorithm simulation only: physical decoding/network acceptance is a separate gate.
    let mut actual: f64 = 0.0; let mut max_error: f64 = 0.0; let mut speed = 1.0;
    let mut d = DriftController::default();
    for second in 1..=7200u64 {
        actual += 1000.0 * 1.0005 * speed;
        let target = second * 1000;
        // Measurement noise is deterministic and alternates +/- 12 ms.
        let measured = (actual + if second % 2 == 0 { 12.0 } else { -12.0 }).max(0.0) as u64;
        match d.update(target, measured, target, 1, true, true) {
            Correction::Rate(r) => speed = r,
            Correction::Seek(_) => panic!("unexpected hard correction during steady playback"),
            Correction::Hold => {}
        }
        max_error = max_error.max((target as f64 - actual).abs());
    }
    assert!(max_error < 200.0, "maximum error: {max_error} ms");
}

#[test] fn measured_rtt_extends_commit_lead_without_moving_prepared_position() {
    let mut s=room();s.observe_rtt("phone",800).unwrap();
    s.intent(true,s.state.generation,100).unwrap();ready(&mut s,200);
    assert_eq!(s.state.timeline.anchor_ms,2051);
    assert_eq!(s.state.timeline.position(2050),0);
    assert_eq!(s.state.timeline.position(2151),100);
    assert_eq!(s.observe_rtt("phone",5001),Err("RTT out of range"));
}

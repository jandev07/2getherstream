use together_sync_core::clock::ClockEstimator;

#[test] fn delayed_samples_do_not_evict_still_fresh_better_calibration_after_four_seconds() {
    let mut c=ClockEstimator::default();
    c.observe(100.0,1019.0,1019.0,938.0).unwrap(); // offset 500, RTT 838
    for i in 0..20 {
        let t0=2000.0+i as f64*500.0;
        c.observe(t0,t0+781.0,t0+781.0,t0+1087.0).unwrap(); // offset 237.5
    }
    assert_eq!(c.estimate(12587.0).unwrap().offset_ms,500.0);
    assert_eq!(c.estimate(12587.0).unwrap().rtt_ms,838.0);
    // No indefinite retention: the better sample expires at the same 30 s bound.
    assert_eq!(c.estimate(30939.0).unwrap().offset_ms,237.5);
    assert!(c.estimate(50000.0).is_none());
}
#[test] fn new_lower_rtt_calibration_still_replaces_retained_history_and_reset_clears_it() {
    let mut c=ClockEstimator::default();
    c.observe(100.0,1019.0,1019.0,938.0).unwrap();
    c.observe(2000.0,2530.0,2530.0,2040.0).unwrap(); // offset 510, RTT 40
    assert_eq!(c.estimate(2040.0).unwrap().offset_ms,510.0);
    c.reset();assert!(c.estimate(2040.0).is_none());
}

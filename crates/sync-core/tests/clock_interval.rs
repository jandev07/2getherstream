use together_sync_core::clock::ClockEstimator;

#[test]
fn overlap_constrains_a_lowest_rtt_point_outside_other_fresh_exchange() {
    let mut clock = ClockEstimator::default();
    clock.observe(0.0, 100.0, 100.0, 100.0).unwrap(); // [0,100], midpoint50
    clock.observe(1000.0, 1190.0, 1190.0, 1110.0).unwrap(); // [80,190]
    let estimate = clock.estimate_interval(1110.0).unwrap();
    assert_eq!(clock.estimate(1110.0).unwrap().offset_ms, 50.0);
    assert_eq!(estimate.sample.offset_ms, 90.0);
    assert_eq!(estimate.lowest_rtt_offset_ms, 50.0);
    assert_eq!(estimate.lower_offset_ms, Some(80.0));
    assert_eq!(estimate.upper_offset_ms, Some(100.0));
    assert_eq!(estimate.sample.rtt_ms, 100.0);
    assert_eq!(estimate.sample.received_ms, 100.0);
    assert_eq!(estimate.support_samples, 2);
    assert_eq!(estimate.oldest_received_ms, 100.0);
}

#[test]
fn directional_jitter_does_not_replace_consensus_with_a_faster_biased_point() {
    let mut clock = ClockEstimator::default();
    // Known constant offset500; forward/return transit varies independently.
    clock.observe(0.0, 1000.0, 1000.0, 800.0).unwrap(); // 500/300 transit
    clock.observe(1000.0, 1800.0, 1800.0, 1800.0).unwrap(); // 300/500
    assert_eq!(clock.estimate_interval(1800.0).unwrap().sample.offset_ms, 500.0);
    clock.observe(2000.0, 2900.0, 2900.0, 2700.0).unwrap(); // 400/300, faster RTT
    assert_eq!(clock.estimate(2700.0).unwrap().offset_ms, 550.0);
    assert_eq!(clock.estimate_interval(2700.0).unwrap().sample.offset_ms, 500.0);
}

#[test]
fn conflicting_bounds_fall_back_without_claiming_consistency() {
    let mut clock = ClockEstimator::default();
    clock.observe(0.0, 100.0, 100.0, 100.0).unwrap();
    clock.observe(1000.0, 1320.0, 1320.0, 1120.0).unwrap(); // [200,320]
    let estimate = clock.estimate_interval(1120.0).unwrap();
    assert_eq!(estimate.sample.offset_ms, 50.0);
    assert_eq!(estimate.lower_offset_ms, None);
    assert_eq!(estimate.upper_offset_ms, None);
    assert_eq!(estimate.support_samples, 2);
}

#[test]
fn expiry_future_samples_and_reset_fence_interval_support() {
    let mut clock = ClockEstimator::default();
    clock.observe(0.0, 100.0, 100.0, 100.0).unwrap();
    clock.observe(1000.0, 1190.0, 1190.0, 1110.0).unwrap();
    assert_eq!(clock.estimate_interval(100.0).unwrap().support_samples, 1);
    let boundary = clock.estimate_interval(30100.0).unwrap();
    assert_eq!(boundary.support_samples, 2);
    assert_eq!(30100.0 - boundary.oldest_received_ms, 30000.0);
    let estimate = clock.estimate_interval(30101.0).unwrap();
    assert_eq!(estimate.support_samples, 1);
    assert_eq!(estimate.sample.offset_ms, 135.0);
    assert_eq!(estimate.oldest_received_ms, 1110.0);
    assert!(clock.estimate_interval(31111.0).is_none());
    assert!(clock.estimate_interval(f64::NAN).is_none());
    clock.reset();
    assert!(clock.estimate_interval(1110.0).is_none());
}

#[test]
fn stable_directional_asymmetry_remains_an_unknown_clock_error() {
    let mut clock = ClockEstimator::default();
    // Actual offset500, fixed forward600/return200. Neither point nor interval
    // midpoint can discover the true offset from these exchanges alone.
    for i in 0..64 {
        let t0 = i as f64 * 500.0;
        clock.observe(t0, t0 + 1100.0, t0 + 1100.0, t0 + 800.0).unwrap();
    }
    let estimate = clock.estimate_interval(32300.0).unwrap();
    assert_eq!(estimate.sample.offset_ms, 700.0);
    assert_eq!(estimate.lower_offset_ms, Some(300.0));
    assert_eq!(estimate.upper_offset_ms, Some(1100.0));
    assert!(estimate.support_samples <= 64);
}

#[test]
fn independently_generated_transit_profiles_keep_true_offset_inside_bounds() {
    // Build timestamps from a known physical delay model, not from the
    // estimator's offset/RTT equations. Cover unequal transit, server work,
    // zero transit, positive/negative offset, expiry and history eviction.
    for true_offset in [-5000.0, 0.0, 5000.0] {
        let mut clock = ClockEstimator::default();
        let mut send = 10000.0;
        for forward in [0.0, 1.0, 250.0, 500.0, 750.0] {
            for backward in [0.0, 1.0, 250.0, 500.0, 750.0] {
                for work in [0.0, 3.0, 50.0] {
                    let receive_at_server = send + forward + true_offset;
                    let send_at_server = receive_at_server + work;
                    let receive_at_client = send + forward + work + backward;
                    clock.observe(send, receive_at_server, send_at_server, receive_at_client).unwrap();
                    let result = clock.estimate_interval(receive_at_client).unwrap();
                    let lower = result.lower_offset_ms.unwrap();
                    let upper = result.upper_offset_ms.unwrap();
                    assert!(lower <= true_offset && true_offset <= upper);
                    assert!((result.sample.offset_ms - true_offset).abs() <= (upper - lower) / 2.0);
                    assert!((1..=64).contains(&result.support_samples));
                    send += 2000.0;
                }
            }
        }
    }
}

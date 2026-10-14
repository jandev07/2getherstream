use together_sync_core::drift::{Correction,DriftController};

#[test] fn physical_filter_response_matches_at_ten_and_hundred_ms_cadences() {
    fn rates(step:u64)->Vec<f64> {
        let mut controller=DriftController::default();let mut rate=1.0;let mut result=Vec::new();
        controller.update(10000,10000,0,1,true,true);
        for now in (step..=3000).step_by(step as usize) {
            let actual=if now<=1000 {10400}else{10000};
            match controller.update(10000,actual,now,1,true,true) {
                Correction::Rate(value)=>rate=value,Correction::Hold=>{},
                Correction::Seek(_)=>panic!("small drift must not seek"),
            }
            if now%100==0 {result.push(rate);}
        }
        result
    }
    let fast=rates(10);let slow=rates(100);
    assert_eq!(fast,slow);assert!(fast.contains(&0.995));assert_eq!(fast.last(),Some(&1.0));
}
#[test] fn duplicate_and_reordered_samples_do_not_change_the_filter_or_seek() {
    let mut d=DriftController::default();
    assert_eq!(d.update(10000,10000,1000,1,true,true),Correction::Hold);
    assert_eq!(d.update(10000,10200,1100,1,true,true),Correction::Hold);
    assert_eq!(d.update(10000,16000,1100,1,true,true),Correction::Hold);
    assert_eq!(d.update(10000,16000,1099,1,true,true),Correction::Hold);
    assert_eq!(d.update(10000,10200,1200,1,true,true),Correction::Rate(0.995));
    assert_eq!(d.update(10000,10200,1200,2,true,true),Correction::Rate(1.0));
    assert_eq!(d.update(10000,16000,1300,2,true,true),Correction::Seek(10000));
    assert_eq!(d.update(10000,16000,1300,2,true,true),Correction::Hold);
}

#[test] fn opposing_small_errors_converge_without_seeks_or_excess_rate_commands() {
    let mut controllers=[DriftController::default(),DriftController::default()];
    let mut positions=[100130.0_f64,99870.0_f64];let mut rates=[1.0_f64;2];let mut changes=[0;2];
    for now in (0..=30000u64).step_by(100) {
        for i in 0..2 {
            if now>0 {positions[i]+=100.0*rates[i];}
            match controllers[i].update(100000+now,positions[i].round() as u64,now,1,true,true) {
                Correction::Rate(rate)=>{assert!((0.995..=1.005).contains(&rate));rates[i]=rate;changes[i]+=1;},
                Correction::Hold=>{},Correction::Seek(_)=>panic!("small opposing errors must not seek"),
            }
        }
    }
    assert!((positions[0]-positions[1]).abs()<60.0,"opposing errors must not remain in the entry band");
    assert_eq!(rates,[1.0,1.0]);assert!(changes.iter().all(|n|*n<=3));
}
#[test] fn bounded_probe_noise_does_not_start_rate_chatter() {
    let mut controller=DriftController::default();
    for now in (0..=10000u64).step_by(100) {
        let noise=if now%200==0 {20i64}else{-20};
        assert_eq!(controller.update(100000+now,((100000+now) as i64+noise) as u64,now,1,true,true),Correction::Hold);
    }
}

// Synthetic reporting cadence informed by the measured fixture, not frame accuracy.
#[test] fn quantized_positions_with_fresh_reads_do_not_request_large_corrections() {
    for phase in [0,25,50,75,99u64] {
        let mut controller=DriftController::default();let mut changes=0;
        for now in (0..=10000u64).step_by(10) {
            let position=100000+((now+phase)/100)*100-phase;
            match controller.update(100000+now,position,now,1,true,true) {
                Correction::Rate(value)=>{assert!((0.995..=1.005).contains(&value));changes+=1;},
                Correction::Hold=>{},Correction::Seek(_)=>panic!("coarse position reporting must not request seek"),
            }
        }
        assert!(changes<=2,"coarse reporting caused rate chatter");
    }
}
#[test] fn frozen_position_with_fresh_reads_requests_authority_only_after_existing_lag_window() {
    let mut controller=DriftController::default();let mut first_seek=None;
    for now in (0..=5200u64).step_by(10) {
        if let Correction::Seek(target)=controller.update(100000+now,100000,now,1,true,true) {
            assert_eq!(target,100000+now);first_seek=Some(now);break;
        }
    }
    assert!(first_seek.is_some_and(|at|at>5000 && at<=5200));
}
#[test] fn paused_or_expired_clock_never_interprets_repeated_position_as_playback_lag() {
    for (steady,fresh) in [(false,true),(true,false)] {
        let mut controller=DriftController::default();
        for now in (0..=10000u64).step_by(100) {
            assert_eq!(controller.update(100000+now,100000,now,1,steady,fresh),Correction::Rate(1.0));
        }
    }
}

//! Real muted PC decoder plus a synthetic control participant. The phone never
//! loads or decodes media; these tests do not establish Android acceptance.
use super::*;
use futures_util::{SinkExt,StreamExt};
use tokio_tungstenite::{connect_async,tungstenite::{client::IntoClientRequest,http::HeaderValue,Message}};
use std::sync::atomic::Ordering;
const PHONE:&str="phone-playing-test-credential-0000000000";
fn root()->PathBuf {PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap()}

// Synthetic readiness and execution follow actual received snapshots over the
// authenticated server socket. No test-only mutation API is added to Host.
async fn phone(host:Arc<Host>,address:std::net::SocketAddr,mut stop:tokio::sync::watch::Receiver<bool>)->(u64,u64) {
    let mut request=format!("ws://{address}/session").into_client_request().unwrap();
    request.headers_mut().insert("Authorization",HeaderValue::from_str(&format!("Bearer {PHONE}")).unwrap());
    let (mut socket,_)=connect_async(request).await.unwrap();
    let mut state:Option<Snapshot>=None;let mut seq=0;let mut inflight=false;
    let mut ready=None;let mut execution=None;let mut ready_count=0;let mut execution_count=0;
    let mut tick=tokio::time::interval(Duration::from_millis(20));
    loop {
        tokio::select! {
            _=stop.changed()=>break,
            message=socket.next()=>{
                match message {
                    Some(Ok(Message::Text(text)))=>{
                        let frame:serde_json::Value=serde_json::from_str(&text).unwrap();
                        if frame["type"]=="ack" {inflight=false;}
                        if frame["state"].is_object() {
                            let incoming:Snapshot=serde_json::from_value(frame["state"].clone()).unwrap();
                            if state.as_ref().is_none_or(|old|incoming.epoch!=old.epoch || incoming.revision>=old.revision) {state=Some(incoming);}
                        }
                    },
                    Some(Ok(Message::Ping(payload)))=>{socket.send(Message::Pong(payload)).await.unwrap();},
                    Some(Ok(Message::Close(_)))|None=>break,
                    Some(Ok(_))=>{},Some(Err(_))=>break,
                }
            },
            _=tick.tick()=>{
                if inflight {continue;}
                let Some(s)=&state else {continue;};let now=host.now();
                let action=if let Some(prep)=&s.prepare {
                    if ready==Some(s.generation) {continue;}
                    ready=Some(s.generation);ready_count+=1;
                    serde_json::json!({"type":"ready","position_ms":prep.position_ms})
                } else {
                    if execution==Some(s.generation) || (s.timeline.playing && now<s.timeline.anchor_ms.saturating_add(25)) {continue;}
                    execution=Some(s.generation);execution_count+=1;
                    serde_json::json!({"type":"executed","playing":s.timeline.playing,
                        "position_ms":s.timeline.position(now),"observed_server_ms":now})
                };
                seq+=1;inflight=true;
                socket.send(Message::Text(serde_json::json!({"type":"command","protocol":1,"epoch":s.epoch,
                    "generation":s.generation,"seq":seq,"action":action}).to_string().into())).await.unwrap();
            }
        }
    }
    let _=socket.close(None).await;(ready_count,execution_count)
}
async fn until_native(player:&PlayerController,observations:&mut tokio::sync::watch::Receiver<u64>,predicate:impl Fn(&Status)->bool) {
    tokio::time::timeout(Duration::from_secs(8),async {
        loop {
            let status=player.status.lock().unwrap().clone();
            assert!(status.error.is_none(),"native fixture error");
            if predicate(&status) {break;}
            observations.changed().await.unwrap();
        }
    }).await.unwrap();
}

#[tokio::test]
async fn playing_native_freezes_for_pause_seek_and_shutdown_before_new_delivery() {
    playing_authority_cases(None).await;
}
#[tokio::test]
async fn nonunit_native_rate_resets_before_new_authority_delivery() {
    for rate in [0.995,1.005] {playing_authority_cases(Some(rate)).await;}
}
async fn playing_authority_cases(forced_rate:Option<f64>) {
    let directory=std::env::var_os("TOGETHER_PLAYING_AUTHORITY_DIR").map(PathBuf::from)
        .unwrap_or_else(||root().join("work/source-validation").join(format!("playing-authority-{}",uuid::Uuid::new_v4())));
    let directory=if let Some(rate)=forced_rate {directory.join(format!("rate-{rate}"))}else{directory};
    let mut receipts=Vec::new();let mut unfrozen=Vec::new();
    for case in ["pause","seek","shutdown"] {
        let host=Host::new("pc-playing-test-credential-000000000000".into(),PHONE.into()).unwrap();
        host.enable_diagnostics(directory.join(case).join("journal")).unwrap();
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
        let server_host=host.clone();let server=tokio::spawn(async move {server_host.serve(listener).await});
        let (stop_phone,phone_stop)=tokio::sync::watch::channel(false);
        let phone_task=tokio::spawn(phone(host.clone(),address,phone_stop));
        let probe=FeedProbe::new();let player=PlayerController::start_owned(root(),0,host.clone(),Some(probe.clone()));
        let mut observations=probe.native_iterations.subscribe();let mut held=probe.native_held.subscribe();
        tokio::time::timeout(Duration::from_secs(5),async {
            while !host.snapshot().await.participants.values().all(|p|p.connected) {tokio::time::sleep(Duration::from_millis(10)).await;}
        }).await.unwrap();
        host.local_pc(None,Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
        until_native(&player,&mut observations,|p|p.duration_ms>0).await;
        host.local_pc(None,Action::Play).await.unwrap();
        until_native(&player,&mut observations,|p|p.state=="Playing" && p.position_ms>=50).await;
        // Establish real native Playing and accepted execution before hold.
        tokio::time::timeout(Duration::from_secs(5),async {
            while !host.snapshot().await.participants.values().all(|p|p.execution.as_ref().is_some_and(|e|e.playing)) {
                observations.changed().await.unwrap();
            }
        }).await.unwrap();
        let playing=player.status.lock().unwrap().clone();
        probe.held.store(true,Ordering::Release);
        *probe.forced_rate.lock().unwrap()=forced_rate;
        probe.hold_before_controls.store(playing.generation,Ordering::Release);
        tokio::time::timeout(Duration::from_secs(3),async {
            while !*held.borrow_and_update() {held.changed().await.unwrap();}
        }).await.unwrap();
        let admitted=*probe.admitted_rate.borrow();
        if let Some(rate)=forced_rate {assert_eq!(admitted,Some(rate));}
        probe.applied_seeks.lock().unwrap().clear();
        match case {
            "pause"=>host.local_pc(None,Action::Pause).await.unwrap(),
            "seek"=>host.local_pc(None,Action::Seek {position_ms:2000}).await.unwrap(),
            "shutdown"=>host.request_shutdown(),_=>unreachable!(),
        };
        let first=*observations.borrow_and_update();probe.hold_native(false);
        tokio::time::timeout(Duration::from_secs(3),async {
            while *observations.borrow_and_update()<first+30 {observations.changed().await.unwrap();}
        }).await.unwrap();
        let frozen=player.status.lock().unwrap().clone();
        let stale_seek_count=probe.applied_seeks.lock().unwrap().len();
        let resets=probe.reset_rates.lock().unwrap().clone();
        probe.held.store(false,Ordering::Release);
        if case=="pause" {
            let generation=host.snapshot().await.generation;
            until_native(&player,&mut observations,|p|p.generation==generation && p.state=="Paused").await;
            host.local_pc(None,Action::Play).await.unwrap();
            until_native(&player,&mut observations,|p|p.state=="Playing").await;
        } else if case=="seek" {
            until_native(&player,&mut observations,|p|p.state=="Playing" && p.position_ms>=2000).await;
        }
        let recovered=player.status.lock().unwrap().clone();
        player.shutdown().await.unwrap();host.request_shutdown();stop_phone.send_replace(true);
        let (ready_count,execution_count)=tokio::time::timeout(Duration::from_secs(5),phone_task).await.unwrap().unwrap();
        tokio::time::timeout(Duration::from_secs(5),server).await.unwrap().unwrap().unwrap();
        host.flush_diagnostics().await.unwrap();let health=host.diagnostic_health().unwrap();
        let exported=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
        assert!(!exported.contains(PHONE) && !exported.contains("ws://") && !exported.contains("Bearer"));
        receipts.push(serde_json::json!({"case":case,"initial_native_state":playing.state,"initial_native_position_ms":playing.position_ms,
            "held_generation":playing.generation,"observed_generation":frozen.generation,"observations_held":30,
            "state_after_authority_change":frozen.state,"rate_after_authority_change":frozen.rate,
            "forced_correction_rate":forced_rate,"admitted_native_rate_submission":admitted,
            "successful_reset_submissions":resets,"stale_seek_submissions":stale_seek_count,"recovered_state":recovered.state,"recovered_position_ms":recovered.position_ms,
            "synthetic_phone_ready_commands":ready_count,"synthetic_phone_execution_commands":execution_count,
            "journal_errors":health.io_errors,"journal_drops":health.dropped_records}));
        std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "scope":"real muted native PC with synthetic authenticated loopback phone",
            "cases":receipts,"android_native_verified":false,"physical_gate_passed":false,"wan_verified":false
        })).unwrap()).unwrap();
        assert_eq!(frozen.generation,playing.generation);
        if frozen.state!="Paused" || frozen.rate!=1.0 || (forced_rate.is_some() && resets!=vec![(playing.generation,1.0)]) {unfrozen.push(case);}
        assert_eq!(stale_seek_count,0);assert_eq!(health.io_errors,0);assert_eq!(health.dropped_records,0);
    }
    assert!(unfrozen.is_empty(),"retained playing authority did not freeze: {unfrozen:?}");
}

#[tokio::test]
async fn scheduled_start_superseded_before_anchor_never_submits_obsolete_unpause() {
    let directory=std::env::var_os("TOGETHER_SCHEDULED_START_DIR").map(PathBuf::from)
        .unwrap_or_else(||root().join("work/source-validation").join(format!("scheduled-start-{}",uuid::Uuid::new_v4())));
    let mut receipts=Vec::new();let mut obsolete_submissions=0;let mut all_paused=true;
    for case in ["pause","seek"] {
        let host=Host::new("pc-start-test-credential-00000000000000".into(),PHONE.into()).unwrap();
        host.enable_diagnostics(directory.join(case).join("journal")).unwrap();
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
        let server_host=host.clone();let server=tokio::spawn(async move {server_host.serve(listener).await});
        let (stop_phone,phone_stop)=tokio::sync::watch::channel(false);
        let phone_task=tokio::spawn(phone(host.clone(),address,phone_stop));
        let probe=FeedProbe::new();let player=PlayerController::start_owned(root(),0,host.clone(),Some(probe.clone()));
        let mut observations=probe.native_iterations.subscribe();let mut held=probe.native_held.subscribe();
        let mut scheduled=probe.scheduled_start.subscribe();
        tokio::time::timeout(Duration::from_secs(5),async {
            while !host.snapshot().await.participants.values().all(|p|p.connected) {tokio::time::sleep(Duration::from_millis(10)).await;}
        }).await.unwrap();
        host.local_pc(None,Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
        until_native(&player,&mut observations,|p|p.duration_ms>0).await;
        host.local_pc(None,Action::Play).await.unwrap();
        let (generation,anchor)=tokio::time::timeout(Duration::from_secs(5),async {
            loop {if let Some(value)=*scheduled.borrow_and_update() {break value;}scheduled.changed().await.unwrap();}
        }).await.unwrap();
        // Native has consumed a committed future timeline while still paused.
        // Hold the following control phase, then replace authority before anchor.
        probe.held.store(true,Ordering::Release);probe.hold_before_controls.store(generation,Ordering::Release);
        tokio::time::timeout(Duration::from_secs(3),async {
            while !*held.borrow_and_update() {held.changed().await.unwrap();}
        }).await.unwrap();
        let replaced_at=host.now();assert!(replaced_at<anchor,"setup missed future commit window");
        assert!(probe.applied_starts.lock().unwrap().is_empty());
        match case {
            "pause"=>host.local_pc(None,Action::Pause).await.unwrap(),
            "seek"=>host.local_pc(None,Action::Seek {position_ms:2000}).await.unwrap(),_=>unreachable!(),
        }
        let replaced_applied_at=host.now();assert!(replaced_applied_at<anchor,"replacement completed after commit anchor");
        probe.hold_native(false);
        // Cross the actual anchor using completed native observations, then
        // retain thirty more observations with new delivery still held.
        tokio::time::timeout(Duration::from_secs(3),async {
            while host.now()<anchor {observations.changed().await.unwrap();}
            let first=*observations.borrow_and_update();
            while *observations.borrow_and_update()<first+30 {observations.changed().await.unwrap();}
        }).await.unwrap();
        let frozen=player.status.lock().unwrap().clone();
        all_paused &= frozen.state=="Paused";
        let old_starts=probe.applied_starts.lock().unwrap().clone();
        let old_count=old_starts.iter().filter(|(g,_)|*g==generation).count();obsolete_submissions+=old_count;
        probe.held.store(false,Ordering::Release);
        if case=="pause" {
            let current=host.snapshot().await.generation;
            until_native(&player,&mut observations,|p|p.generation==current && p.state=="Paused").await;
            host.local_pc(None,Action::Play).await.unwrap();
        }
        until_native(&player,&mut observations,|p|p.state=="Playing" && (case!="seek" || p.position_ms>=2000)).await;
        let recovered=player.status.lock().unwrap().clone();
        let final_starts=probe.applied_starts.lock().unwrap().clone();
        player.shutdown().await.unwrap();host.request_shutdown();stop_phone.send_replace(true);
        tokio::time::timeout(Duration::from_secs(5),phone_task).await.unwrap().unwrap();
        tokio::time::timeout(Duration::from_secs(5),server).await.unwrap().unwrap().unwrap();
        host.flush_diagnostics().await.unwrap();let health=host.diagnostic_health().unwrap();
        receipts.push(serde_json::json!({"case":case,"obsolete_generation":generation,"anchor_ms":anchor,
            "replacement_requested_ms":replaced_at,"replacement_applied_ms":replaced_applied_at,
            "before_anchor_ms":anchor-replaced_applied_at,"observations_after_anchor":30,
            "retained_native_generation":frozen.generation,"retained_native_state":frozen.state,
            "obsolete_unpause_submissions":old_count,"old_start_trace":old_starts,"final_start_trace":final_starts,
            "recovered_generation":recovered.generation,"recovered_state":recovered.state,"recovered_position_ms":recovered.position_ms,
            "journal_errors":health.io_errors,"journal_drops":health.dropped_records}));
        std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "scope":"real muted PC future commit superseded with synthetic loopback phone",
            "cases":receipts,"android_native_verified":false,"physical_gate_passed":false,"wan_verified":false
        })).unwrap()).unwrap();
        assert_eq!(frozen.generation,generation);assert_eq!(health.io_errors,0);assert_eq!(health.dropped_records,0);
        assert!(final_starts.iter().any(|(g,_)|*g==recovered.generation && *g!=generation));
    }
    assert_eq!(obsolete_submissions,0);assert!(all_paused);
}

#[tokio::test]
async fn observed_native_eof_superseded_by_pc_pause_or_seek_does_not_submit_stale_end() {
    let directory=std::env::var_os("TOGETHER_EOF_AUTHORITY_DIR").map(PathBuf::from)
        .unwrap_or_else(||root().join("work/source-validation").join(format!("eof-authority-{}",uuid::Uuid::new_v4())));
    let mut receipts=Vec::new();let mut stale_submissions=0;
    for case in ["pause","seek"] {
        let host=Host::new("pc-eof-test-credential-0000000000000000".into(),PHONE.into()).unwrap();
        host.enable_diagnostics(directory.join(case).join("journal")).unwrap();
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
        let server_host=host.clone();let server=tokio::spawn(async move {server_host.serve(listener).await});
        let (stop_phone,phone_stop)=tokio::sync::watch::channel(false);
        let phone_task=tokio::spawn(phone(host.clone(),address,phone_stop));
        let probe=FeedProbe::new();let player=PlayerController::start_owned(root(),0,host.clone(),Some(probe.clone()));
        let mut observations=probe.native_iterations.subscribe();let mut held=probe.native_held.subscribe();
        tokio::time::timeout(Duration::from_secs(5),async {
            while !host.snapshot().await.participants.values().all(|p|p.connected) {tokio::time::sleep(Duration::from_millis(10)).await;}
        }).await.unwrap();
        host.local_pc(None,Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
        until_native(&player,&mut observations,|p|p.duration_ms>2000).await;
        let duration=player.status.lock().unwrap().duration_ms;
        host.local_pc(None,Action::Seek {position_ms:duration-1000}).await.unwrap();
        host.local_pc(None,Action::Play).await.unwrap();
        let playing_generation=host.snapshot().await.generation;
        probe.hold_at_eof.store(true,Ordering::Release);
        // Natural native playback from a near-end authoritative seek, no EOF injection.
        tokio::time::timeout(Duration::from_secs(8),async {
            while !*held.borrow_and_update() {held.changed().await.unwrap();}
        }).await.unwrap();
        let sampled=(*probe.sampled_eof.borrow()).unwrap();
        probe.held.store(true,Ordering::Release);
        assert!(probe.end_submissions.lock().unwrap().is_empty());
        match case {
            "pause"=>host.local_pc(None,Action::Pause).await.unwrap(),
            "seek"=>host.local_pc(None,Action::Seek {position_ms:2000}).await.unwrap(),_=>unreachable!(),
        };
        let replacement=host.snapshot().await;
        let first=*observations.borrow_and_update();probe.hold_native(false);
        tokio::time::timeout(Duration::from_secs(3),async {
            while *observations.borrow_and_update()<first+30 {observations.changed().await.unwrap();}
        }).await.unwrap();
        let retained=player.status.lock().unwrap().clone();let authority=host.snapshot().await;
        let old_end=probe.end_submissions.lock().unwrap().iter().filter(|generation|**generation==sampled.0).count();
        stale_submissions+=old_end;
        probe.held.store(false,Ordering::Release);
        if case=="pause" {host.local_pc(None,Action::Seek {position_ms:2000}).await.unwrap();host.local_pc(None,Action::Play).await.unwrap();}
        until_native(&player,&mut observations,|p|p.generation>sampled.0 && p.state=="Playing" && p.position_ms>=2000 && p.position_ms<duration-1000).await;
        let resumed=player.status.lock().unwrap().clone();
        player.shutdown().await.unwrap();host.request_shutdown();stop_phone.send_replace(true);
        let (ready_count,execution_count)=tokio::time::timeout(Duration::from_secs(5),phone_task).await.unwrap().unwrap();
        tokio::time::timeout(Duration::from_secs(5),server).await.unwrap().unwrap().unwrap();
        host.flush_diagnostics().await.unwrap();let health=host.diagnostic_health().unwrap();
        receipts.push(serde_json::json!({"case":case,"native_eof_observed":true,"eof_generation":sampled.0,
            "eof_position_ms":sampled.1,"eof_duration_ms":sampled.2,"replacement_generation":replacement.generation,
            "retained_observation_generation":retained.generation,"retained_native_state":retained.state,
            "observations_after_release":30,"stale_end_submissions":old_end,"authority_ended":authority.ended,
            "authority_fault":authority.fault,"resumed_generation":resumed.generation,"resumed_position_ms":resumed.position_ms,
            "synthetic_phone_ready":ready_count,"synthetic_phone_executed":execution_count,
            "journal_errors":health.io_errors,"journal_drops":health.dropped_records}));
        std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "scope":"real muted native EOF plus synthetic phone control; no Android decoding",
            "cases":receipts,"physical_gate_passed":false,"wan_verified":false
        })).unwrap()).unwrap();
        assert!(sampled.0>=playing_generation);assert_eq!(retained.generation,sampled.0);
        assert!(sampled.1.abs_diff(sampled.2)<=1000);assert_eq!(authority.generation,replacement.generation);
        assert!(!authority.ended && authority.fault.is_none());assert_eq!(health.io_errors,0);assert_eq!(health.dropped_records,0);
    }
    assert_eq!(stale_submissions,0,"superseded native EOF submitted stale end feedback");
}

#[tokio::test]
async fn actual_native_premature_eof_records_unexpected_end_until_explicit_reload() {premature_eof_case(false).await;}
#[tokio::test]
async fn actual_eof_cannot_replace_pending_native_command_failure() {premature_eof_case(true).await;}
async fn premature_eof_case(actual_command:bool) {
    let expected=if actual_command {PlayerFault::CommandFailed} else {PlayerFault::UnexpectedEnd};
    let expected_error=if actual_command {"Native playback command failed"} else {"Native media ended unexpectedly"};
    let directory=std::env::var_os("TOGETHER_PREMATURE_EOF_DIR").map(PathBuf::from)
        .unwrap_or_else(||root().join("work/source-validation").join(format!("premature-eof-{}",uuid::Uuid::new_v4())));
    let directory=if actual_command {directory.join("command-fault")} else {directory};
    let host=Host::new("pc-premature-eof-credential-0000000000".into(),PHONE.into()).unwrap();
    host.enable_diagnostics(directory.join("journal")).unwrap();
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
    let server_host=host.clone();let server=tokio::spawn(async move {server_host.serve(listener).await});
    let (stop_phone,phone_stop)=tokio::sync::watch::channel(false);
    let phone_task=tokio::spawn(phone(host.clone(),address,phone_stop));
    let probe=FeedProbe::new();let player=PlayerController::start_owned(root(),0,host.clone(),Some(probe.clone()));
    let mut observations=probe.native_iterations.subscribe();let mut held=probe.native_held.subscribe();
    tokio::time::timeout(Duration::from_secs(5),async {
        while !host.snapshot().await.participants.values().all(|p|p.connected) {tokio::time::sleep(Duration::from_millis(10)).await;}
    }).await.unwrap();
    host.local_pc(None,Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
    until_native(&player,&mut observations,|p|p.duration_ms>0).await;
    host.local_pc(None,Action::Play).await.unwrap();
    until_native(&player,&mut observations,|p|p.state=="Playing" && p.position_ms>=50).await;
    let before=host.snapshot().await;let loads_before=*probe.fixture_loads.borrow();
    let duration=player.status.lock().unwrap().duration_ms;
    probe.hold_at_eof.store(true,Ordering::Release);
    probe.reject_at_eof.store(actual_command,Ordering::Release);
    *probe.reposition_native.lock().unwrap()=Some(duration);
    tokio::time::timeout(Duration::from_secs(8),async {
        while !*held.borrow_and_update() {held.changed().await.unwrap();}
    }).await.unwrap();
    let sampled=(*probe.sampled_eof.borrow()).unwrap();let authoritative_at_eof=host.snapshot().await;
    let eof_held_server_ms=host.now();let pc_position_when_eof_held=authoritative_at_eof.timeline.position(eof_held_server_ms);
    assert!(*probe.reposition_submitted.borrow());
    if actual_command {assert!(*probe.command_rejected.borrow());}
    assert!(pc_position_when_eof_held.saturating_add(2500)<sampled.2);
    probe.held.store(true,Ordering::Release);probe.hold_native(false);
    let recorded=tokio::time::timeout(Duration::from_secs(3),async {
        loop {
            let s=host.snapshot().await;
            if s.participants["pc"].player_fault==Some(expected) {break s;}
            observations.changed().await.unwrap();
        }
    }).await;
    if recorded.is_err() {
        std::fs::write(directory.join("failed-recording.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "native_eof_generation":sampled.0,"native_position_ms":sampled.1,"native_duration_ms":sampled.2,
            "pc_position_when_eof_held_ms":pc_position_when_eof_held,
            "expected_code":expected,"actual_recorded_code":host.snapshot().await.participants["pc"].player_fault,
            "native_error":player.status.lock().unwrap().error,
            "expected_code_recorded":false,"end_feedback_submissions":probe.end_submissions.lock().unwrap().len()
        })).unwrap()).unwrap();
    }
    let recorded=recorded.unwrap();
    host.local_pc(None,Action::Buffering {active:true}).await.unwrap();
    let first=*observations.borrow_and_update();
    tokio::time::timeout(Duration::from_secs(3),async {
        while *observations.borrow_and_update()<first+30 {observations.changed().await.unwrap();}
    }).await.unwrap();
    let pending=host.snapshot().await;let loads_pending=*probe.fixture_loads.borrow();
    let pending_native=player.status.lock().unwrap().clone();
    let eof_submissions=probe.end_submissions.lock().unwrap().len();
    probe.held.store(false,Ordering::Release);
    host.local_pc(None,Action::Load {media:MediaIdentity::fixture()}).await.unwrap();
    tokio::time::timeout(Duration::from_secs(5),async {
        loop {
            let p=player.status.lock().unwrap().clone();
            if p.state=="Paused" && p.position_ms==0 && p.error.is_none() && *probe.fixture_loads.borrow()>loads_before {break;}
            observations.changed().await.unwrap();
        }
    }).await.unwrap();
    tokio::time::timeout(Duration::from_secs(5),async {
        while host.snapshot().await.participants["pc"].player_fault.is_some() {observations.changed().await.unwrap();}
    }).await.unwrap();
    host.local_pc(None,Action::Play).await.unwrap();
    until_native(&player,&mut observations,|p|p.state=="Playing" && p.position_ms>=50 && p.position_ms<duration-1000).await;
    let recovered=host.snapshot().await;let resumed=player.status.lock().unwrap().clone();let loads_final=*probe.fixture_loads.borrow();
    player.shutdown().await.unwrap();host.request_shutdown();stop_phone.send_replace(true);
    let (ready_count,execution_count)=tokio::time::timeout(Duration::from_secs(5),phone_task).await.unwrap().unwrap();
    tokio::time::timeout(Duration::from_secs(5),server).await.unwrap().unwrap().unwrap();
    host.flush_diagnostics().await.unwrap();let health=host.diagnostic_health().unwrap();
    std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "scope":"test-induced native seek to EOF while PC timeline remains near start; actual muted libmpv, synthetic phone",
        "native_reposition_command_accepted":true,"actual_eof_observed":true,"actual_invalid_command_rejected":actual_command,"before_generation":before.generation,
        "native_eof_generation":sampled.0,"native_eof_position_ms":sampled.1,"native_duration_ms":sampled.2,
        "pc_position_when_eof_held_ms":pc_position_when_eof_held,"eof_held_server_ms":eof_held_server_ms,
        "recorded_generation":recorded.generation,"recorded_fault":recorded.fault,
        "recorded_native_code":recorded.participants["pc"].player_fault,"pending_native_state":pending_native.state,
        "pending_native_error":pending_native.error,
        "pending_code":pending.participants["pc"].player_fault,"pending_ended":pending.ended,
        "observations_after_fault":30,"load_count_before":loads_before,"load_count_pending":loads_pending,
        "load_count_final":loads_final,"end_feedback_submissions":eof_submissions,
        "recovered_fault":recovered.fault,"recovered_native_code":recovered.participants["pc"].player_fault,
        "resumed_position_ms":resumed.position_ms,"resumed_state":resumed.state,
        "synthetic_phone_ready":ready_count,"synthetic_phone_executed":execution_count,
        "journal_errors":health.io_errors,"journal_drops":health.dropped_records,
        "android_native_verified":false,"physical_gate_passed":false,"wan_verified":false
    })).unwrap()).unwrap();
    assert_eq!(loads_pending,loads_before);assert_eq!(loads_final,loads_before+1);assert_eq!(eof_submissions,0);
    assert_eq!(pending.participants["pc"].player_fault,Some(expected));assert!(!pending.ended);
    assert_eq!(pending_native.state,"Failed");
    assert_eq!(pending_native.error.as_deref(),Some(expected_error));assert!(recovered.fault.is_none());
    assert_eq!(health.io_errors,0);assert_eq!(health.dropped_records,0);
}

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::{connect_async, tungstenite::{client::IntoClientRequest, http::HeaderValue, Message}, WebSocketStream, MaybeTlsStream};
type Socket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
const PC: &str = "pc-test-token-at-least-32-characters-000";
const PHONE: &str = "phone-test-token-at-least-32-characters";
#[tokio::test]
#[ignore="Explicit bounded control-only reconnect soak; no native players or physical acceptance"]
async fn sustained_control_reconnects_preserve_authority_and_release_resources() {
    use together_session_server::{Host,Action};
    let seconds=std::env::var("TOGETHER_CONTROL_SOAK_SECONDS").ok().map(|value|value.parse::<u64>().unwrap()).unwrap_or(1200);
    assert!((2..=1800).contains(&seconds));
    let directory=std::env::temp_dir().join(format!("together-control-soak-{}",uuid::Uuid::new_v4()));
    let host=Host::new(PC.into(),PHONE.into()).unwrap();host.enable_diagnostics(directory.clone()).unwrap();
    host.attach_local_pc().await.unwrap();
    host.local_pc(None,Action::Load {media:together_sync_core::MediaIdentity::fixture()}).await.unwrap();
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
    let url=format!("ws://{address}/session");let server=host.clone();
    let task=tokio::spawn(async move {server.serve(listener).await});
    let epoch=host.snapshot().await.epoch;let started=std::time::Instant::now();let mut cycles=0u64;
    let mut previous_generation=0;
    while started.elapsed().as_secs()<seconds {
        let (mut phone,_)=connect_async(request(&url,PHONE)).await.unwrap();
        let initial=read_type(&mut phone,"snapshot").await["state"].clone();
        assert_eq!(initial["epoch"],epoch);assert!(initial["generation"].as_u64().unwrap()>previous_generation);
        assert_eq!(initial["participants"]["phone"]["connected"],true);
        let paused=command(&mut phone,&initial,1,json!({"type":"pause"})).await;
        assert!(paused["error"].is_null());assert_eq!(paused["state"]["timeline"]["playing"],false);
        let position=(cycles%100)*1000;
        let seek=command(&mut phone,&paused["state"],2,json!({"type":"seek","position_ms":position})).await;
        assert!(seek["error"].is_null());assert_eq!(seek["state"]["prepare"]["position_ms"],position);
        let stale=command(&mut phone,&initial,3,json!({"type":"seek","position_ms":999999})).await;
        assert_eq!(stale["error"],"stale generation");assert_eq!(stale["state"]["prepare"]["position_ms"],position);
        host.local_pc(Some(seek["state"]["generation"].as_u64().unwrap()),Action::Pause).await.unwrap();
        previous_generation=host.snapshot().await.generation;
        phone.close(None).await.unwrap();drop(phone);
        tokio::time::timeout(std::time::Duration::from_secs(3),async {
            while host.snapshot().await.participants["phone"].connected {tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
        }).await.unwrap();
        // The upgrade budget refills one token per second. Faster artificial
        // reconnects eventually consume the burst allowance even when healthy.
        cycles+=1;tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
    host.request_shutdown();tokio::time::timeout(std::time::Duration::from_secs(3),task).await.unwrap().unwrap().unwrap();
    let frozen=serde_json::to_value(host.snapshot().await).unwrap();tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    assert_eq!(serde_json::to_value(host.snapshot().await).unwrap(),frozen);
    host.flush_diagnostics().await.unwrap();let health=host.diagnostic_health().unwrap();assert_eq!(health.io_errors,0);assert_eq!(health.dropped_records,0);
    let text=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
    assert!(!text.contains(PC) && !text.contains(PHONE) && !text.contains("https://"));
    let replacement=tokio::net::TcpListener::bind(address).await.unwrap();drop(replacement);
    let weak=std::sync::Arc::downgrade(&host);drop(host);assert!(weak.upgrade().is_none());
    for _ in 0..100 {if std::fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
    assert!(!directory.exists());
    println!("CONTROL_SOAK_PASSED seconds={seconds} cycles={cycles} physical_gate=false native_players=false");
}
#[tokio::test] async fn local_diagnostics_observe_authority_without_copying_credentials_or_metadata() {
    use together_session_server::{Host,Action};
    let directory=std::env::temp_dir().join(format!("together-host-journal-{}",uuid::Uuid::new_v4()));
    let host=Host::new(PC.into(),PHONE.into()).unwrap();host.enable_diagnostics(directory.clone()).unwrap();
    assert_eq!(host.enable_diagnostics(directory.clone()),Err("diagnostics_already_enabled"));
    host.attach_local_pc().await.unwrap();
    let mut media=together_sync_core::MediaIdentity::fixture();media.filename=Some("PRIVATE_MEDIA_NAME".into());
    host.local_pc(None,Action::Load {media}).await.unwrap();
    let before=host.snapshot().await;
    assert_eq!(host.local_pc(Some(before.generation-1),Action::Seek {position_ms:777}).await,Err("stale generation"));
    assert_eq!(host.snapshot().await.revision,before.revision);
    let text=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
    assert!(!text.contains(PC));assert!(!text.contains(PHONE));assert!(!text.contains("PRIVATE_MEDIA_NAME"));
    let rows:Vec<Value>=text.lines().map(|s|serde_json::from_str(s).unwrap()).collect();
    assert_eq!(rows.len(),4);assert_eq!(rows[0]["record"]["sample"]["pc"]["connected"],true);
    assert_eq!(rows[1]["record"]["sample"]["kind"],"load");
    assert_eq!(rows[1]["record"]["sample"]["outcome"],"accepted");
    assert_eq!(rows[2]["record"]["sample"]["media_present"],true);
    assert_eq!(rows[2]["record"]["sample"]["preparing"],true);
    assert_eq!(rows[3]["record"]["sample"]["outcome"],"stale_generation");
    assert_eq!(rows[3]["record"]["sample"]["position_ms"],777);
    assert_eq!(host.diagnostic_health().unwrap().io_errors,0);
    drop(host);
    for _ in 0..100 {if std::fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
    assert!(!directory.exists());
}
#[tokio::test] async fn socket_command_diagnostics_and_server_cancellation_release_host() {
    let directory=std::env::temp_dir().join(format!("together-socket-journal-{}",uuid::Uuid::new_v4()));
    let host=together_session_server::Host::new(PC.into(),PHONE.into()).unwrap();
    host.enable_diagnostics(directory.clone()).unwrap();
    host.attach_local_pc().await.unwrap();
    host.local_pc(None,together_session_server::Action::Load {media:together_sync_core::MediaIdentity::fixture()}).await.unwrap();
    let weak=std::sync::Arc::downgrade(&host);
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url=format!("ws://{}/session",listener.local_addr().unwrap());
    let server=host.clone();let task=tokio::spawn(async move {server.serve(listener).await.unwrap();});
    let (mut phone,_)=connect_async(request(&url,PHONE)).await.unwrap();
    let state=read_type(&mut phone,"snapshot").await["state"].clone();
    let ack=command(&mut phone,&state,1,json!({"type":"load","media":{"provider":"fixture","torrent_id":0,"file_id":0,"filename":"PRIVATE_REMOTE_NAME"}})).await;
    assert_eq!(ack["error"],"host only");
    let ack=command(&mut phone,&state,1,json!({"type":"play"})).await;
    assert_eq!(ack["error"],"duplicate sequence");
    let ack=command(&mut phone,&state,2,json!({"type":"pause"})).await;
    assert!(ack["error"].is_null());
    let text=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
    assert!(!text.contains(PHONE));assert!(!text.contains("PRIVATE_REMOTE_NAME"));
    let rows:Vec<Value>=text.lines().map(|s|serde_json::from_str(s).unwrap()).filter(|r:&Value|r["record"]["event"]=="command" && r["record"]["sample"]["actor"]=="phone").collect();
    assert_eq!(rows.len(),3);
    assert_eq!(rows[0]["record"]["sample"]["actor"],"phone");
    assert_eq!(rows[0]["record"]["sample"]["sequence"],1);
    assert_eq!(rows[0]["record"]["sample"]["outcome"],"host_only");
    assert_eq!(rows[1]["record"]["sample"]["outcome"],"duplicate_sequence");
    assert_eq!(rows[2]["record"]["sample"]["kind"],"pause");
    assert_eq!(rows[2]["record"]["sample"]["outcome"],"accepted");
    phone.close(None).await.unwrap();drop(phone);task.abort();let _=task.await;drop(host);
    for _ in 0..100 {if weak.upgrade().is_none(){break;}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
    assert!(weak.upgrade().is_none(),"cancelled server retained Host/ticker");
    for _ in 0..100 {if std::fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
    assert!(!directory.exists());
}
async fn start() -> (String, tokio::task::JoinHandle<()>) {
    let host = together_session_server::Host::new(PC.into(),PHONE.into()).unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/session",listener.local_addr().unwrap());
    let task = tokio::spawn(async move { host.serve(listener).await.unwrap(); }); (url, task)
}
fn request(url: &str, token: &str) -> tokio_tungstenite::tungstenite::http::Request<()> {
    let mut r = url.into_client_request().unwrap();
    r.headers_mut().insert("authorization",HeaderValue::from_str(&format!("Bearer {token}")).unwrap()); r
}
async fn read_type(s: &mut Socket, kind: &str) -> Value {
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop { match s.next().await.unwrap().unwrap() {
            Message::Text(raw) => { let v:Value = serde_json::from_str(&raw).unwrap(); if v["type"] == kind { return v; } },
            Message::Ping(v) => s.send(Message::Pong(v)).await.unwrap(),
            _ => {}
        } }
    }).await.unwrap()
}
async fn command(s: &mut Socket, state: &Value, seq: u64, action: Value) -> Value {
    s.send(Message::Text(json!({"type":"command","protocol":1,"epoch":state["epoch"],"seq":seq,"generation":state["generation"],"action":action}).to_string().into())).await.unwrap();
    read_type(s,"ack").await
}
#[tokio::test] async fn authenticated_roles_and_no_media_endpoint() {
    let (url, task) = start().await;
    let err = connect_async(request(&url,"wrong-token")).await.unwrap_err();
    match err { tokio_tungstenite::tungstenite::Error::Http(r) => assert_eq!(r.status(),401), _ => panic!("expected auth rejection") }
    let (mut phone,_) = connect_async(request(&url,PHONE)).await.unwrap();
    let state = read_type(&mut phone,"snapshot").await["state"].clone();
    let ack = command(&mut phone,&state,1,json!({"type":"load","media":{"provider":"torbox","torrent_id":1,"file_id":2}})).await;
    assert_eq!(ack["error"],"host only"); assert!(ack["state"]["media"].is_null());
    phone.close(None).await.unwrap(); task.abort();
}
#[tokio::test] async fn stale_and_duplicate_commands_rejected_over_socket() {
    let (url,task) = start().await;
    let (mut pc,_) = connect_async(request(&url,PC)).await.unwrap();
    let state = read_type(&mut pc,"snapshot").await["state"].clone();
    let loaded = command(&mut pc,&state,1,json!({"type":"load","media":{"provider":"torbox","torrent_id":1,"file_id":2}})).await;
    assert!(loaded["error"].is_null());
    let stale = command(&mut pc,&state,2,json!({"type":"seek","position_ms":9000})).await;
    assert_eq!(stale["error"],"stale generation");
    let dup = command(&mut pc,&loaded["state"],1,json!({"type":"pause"})).await;
    assert_eq!(dup["error"],"duplicate sequence");
    pc.close(None).await.unwrap(); task.abort();
}
#[tokio::test] async fn clocks_and_reconnect_snapshot_use_same_epoch_new_generation() {
    let (url,task) = start().await;
    let (mut pc,_) = connect_async(request(&url,PC)).await.unwrap(); read_type(&mut pc,"snapshot").await;
    let (mut phone,_) = connect_async(request(&url,PHONE)).await.unwrap(); read_type(&mut phone,"snapshot").await;
    let latest = read_type(&mut pc,"snapshot").await["state"].clone();
    let loaded = command(&mut pc,&latest,1,json!({"type":"load","media":{"provider":"torbox","torrent_id":1,"file_id":2}})).await;
    pc.send(Message::Text(json!({"type":"clock","protocol":1,"t0_ms":42}).to_string().into())).await.unwrap();
    let clock = read_type(&mut pc,"clock").await; assert_eq!(clock["t0_ms"],42); assert!(clock["t2_ms"].as_u64().unwrap() >= clock["t1_ms"].as_u64().unwrap());
    phone.close(None).await.unwrap();
    let mut disconnected;
    loop { disconnected=read_type(&mut pc,"snapshot").await["state"].clone(); if disconnected["participants"]["phone"]["connected"] == false { break; } }
    assert_eq!(disconnected["timeline"]["playing"],false);
    let (mut phone2,_) = connect_async(request(&url,PHONE)).await.unwrap();
    let reconnect = read_type(&mut phone2,"snapshot").await["state"].clone();
    assert_eq!(reconnect["epoch"],loaded["state"]["epoch"]);
    assert!(reconnect["generation"].as_u64().unwrap() > loaded["state"]["generation"].as_u64().unwrap());
    assert_eq!(reconnect["prepare"]["ready"],json!([]));
    phone2.close(None).await.unwrap(); pc.close(None).await.unwrap(); task.abort();
}

#[tokio::test] async fn local_pc_reservation_and_stale_feedback_are_enforced() {
    let host=together_session_server::Host::new(PC.into(),PHONE.into()).unwrap();
    host.attach_local_pc().await.unwrap();
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url=format!("ws://{}/session",listener.local_addr().unwrap());
    let running=host.clone();let task=tokio::spawn(async move {running.serve(listener).await.unwrap()});
    match connect_async(request(&url,PC)).await.unwrap_err() {
        tokio_tungstenite::tungstenite::Error::Http(r)=>assert_eq!(r.status(),409),
        _=>panic!("expected reserved PC role"),
    }
    host.local_pc(None,together_session_server::Action::Load {media:together_sync_core::MediaIdentity::fixture()}).await.unwrap();
    let generation=host.snapshot().await.generation;
    assert_eq!(host.local_pc(Some(generation-1),together_session_server::Action::Ready {position_ms:0}).await,Err("stale generation"));
    assert!(host.snapshot().await.prepare.unwrap().ready.is_empty());task.abort();
}
#[tokio::test] async fn control_diagnostics_explain_disconnects_without_recording_peer_text() {
    let directory=std::env::temp_dir().join(format!("together-control-journal-{}",uuid::Uuid::new_v4()));
    let host=together_session_server::Host::new(PC.into(),PHONE.into()).unwrap();
    host.enable_diagnostics(directory.clone()).unwrap();
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url=format!("ws://{}/session",listener.local_addr().unwrap());
    let server=host.clone();let task=tokio::spawn(async move {server.serve(listener).await.unwrap();});
    async fn offline(host:&together_session_server::Host) {
        tokio::time::timeout(std::time::Duration::from_secs(9),async {
            while host.snapshot().await.participants["phone"].connected {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }).await.unwrap();
    }
    let (mut phone,_)=connect_async(request(&url,PHONE)).await.unwrap();read_type(&mut phone,"snapshot").await;
    let before=host.snapshot().await;
    phone.send(Message::Text("PRIVATE_PEER_TEXT https://example.invalid/?token=PRIVATE_TOKEN".into())).await.unwrap();
    assert_eq!(read_type(&mut phone,"error").await["code"],"invalid_request");
    // read_type answers heartbeat pings; an independent valid RTT observation
    // may advance revision while this malformed request is being rejected.
    let mut before=serde_json::to_value(before).unwrap();
    let mut after=serde_json::to_value(host.snapshot().await).unwrap();
    for state in [&mut before,&mut after] {
        state.as_object_mut().unwrap().remove("revision");
        for participant in state["participants"].as_object_mut().unwrap().values_mut() {
            participant.as_object_mut().unwrap().remove("rtt_ms");
        }
    }
    assert_eq!(after,before,"malformed request mutated playback authority");
    phone.send(Message::Text(json!({"type":"clock","protocol":99,"t0_ms":42}).to_string().into())).await.unwrap();
    offline(&host).await;drop(phone);

    let (mut silent,_)=connect_async(request(&url,PHONE)).await.unwrap();read_type(&mut silent,"snapshot").await;
    // No polling, no automatic pong flush: the authenticated peer goes silent.
    offline(&host).await;drop(silent);

    let (mut flood,_)=connect_async(request(&url,PHONE)).await.unwrap();read_type(&mut flood,"snapshot").await;
    for _ in 0..120 {if flood.send(Message::Text("PRIVATE_FLOOD_TEXT".into())).await.is_err(){break;}}
    offline(&host).await;drop(flood);

    let (mut active,_)=connect_async(request(&url,PHONE)).await.unwrap();read_type(&mut active,"snapshot").await;
    task.abort();let _=task.await;offline(&host).await;drop(active);
    let text=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
    for forbidden in [PC,PHONE,"PRIVATE_PEER_TEXT","PRIVATE_TOKEN","PRIVATE_FLOOD_TEXT","example.invalid"] {assert!(!text.contains(forbidden));}
    let rows:Vec<Value>=text.lines().map(|line|serde_json::from_str(line).unwrap()).collect();
    let controls:Vec<&Value>=rows.iter().filter(|r|r["record"]["event"]=="control").map(|r|&r["record"]["sample"]).collect();
    for kind in ["invalid_request","unsupported_protocol","heartbeat_timeout","rate_limited","server_stopping"] {
        assert!(controls.iter().any(|s|s["kind"]==kind),"missing {kind}");
    }
    assert_eq!(controls.iter().filter(|s|s["kind"]=="socket_connected").count(),4);
    assert_eq!(controls.iter().filter(|s|s["kind"]=="socket_disconnected").count(),3);
    assert!(controls.iter().all(|s|s["actor"]=="phone" && s["server_ms"].is_u64()));
    assert_eq!(host.diagnostic_health().unwrap().dropped_records,0);
    assert_eq!(host.diagnostic_health().unwrap().io_errors,0);
    drop(host);
    for _ in 0..100 {if std::fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
    assert!(!directory.exists());
}
#[tokio::test] async fn server_stop_closes_active_phone_and_restart_rejects_old_epoch() {
    use together_session_server::{Host,Action};
    let host=Host::new(PC.into(),PHONE.into()).unwrap();
    host.attach_local_pc().await.unwrap();
    host.local_pc(None,Action::Load {media:together_sync_core::MediaIdentity::fixture()}).await.unwrap();
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address=listener.local_addr().unwrap();let url=format!("ws://{address}/session");
    let server=host.clone();let task=tokio::spawn(async move {server.serve(listener).await.unwrap();});
    let (mut phone,_)=connect_async(request(&url,PHONE)).await.unwrap();
    let old_state=read_type(&mut phone,"snapshot").await["state"].clone();
    host.local_pc(Some(old_state["generation"].as_u64().unwrap()),Action::Ready {position_ms:0}).await.unwrap();
    let ack=command(&mut phone,&old_state,1,json!({"type":"ready","position_ms":0})).await;
    let play=command(&mut phone,&ack["state"],2,json!({"type":"play"})).await;
    host.local_pc(Some(play["state"]["generation"].as_u64().unwrap()),Action::Ready {position_ms:0}).await.unwrap();
    let commit=command(&mut phone,&play["state"],3,json!({"type":"ready","position_ms":0})).await;
    assert_eq!(commit["state"]["timeline"]["playing"],true);
    task.abort();let _=task.await;
    let close=tokio::time::timeout(std::time::Duration::from_secs(2),async {
        loop {match phone.next().await.unwrap().unwrap() {
            Message::Close(Some(frame))=>break frame,
            Message::Ping(payload)=>{let _=phone.send(Message::Pong(payload)).await;},_=>{}
        }}
    }).await.unwrap();
    assert_eq!(close.code,tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Restart);
    assert_eq!(close.reason,"server_restart");drop(phone);
    let frozen=host.snapshot().await;
    assert!(!frozen.participants["phone"].connected);assert!(!frozen.timeline.playing);
    assert!(frozen.desired_playing);
    assert_eq!(host.local_pc(None,Action::Play).await,Err("server stopping"));
    assert_eq!(host.snapshot().await.revision,frozen.revision);
    let duplicate=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    assert!(host.clone().serve(duplicate).await.is_err());
    let weak=std::sync::Arc::downgrade(&host);drop(host);
    for _ in 0..100 {if weak.upgrade().is_none(){break;}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
    assert!(weak.upgrade().is_none(),"stopped client/ticker retained old Host");

    // Same address and pairing credentials, fresh authority epoch; missed commands
    // are never replayed into the new session. This is a transport test, not mpv.
    let next=Host::new(PC.into(),PHONE.into()).unwrap();
    let listener=tokio::net::TcpListener::bind(address).await.unwrap();
    let task=tokio::spawn(async move {next.serve(listener).await.unwrap();});
    let (mut phone,_)=connect_async(request(&url,PHONE)).await.unwrap();
    let state=read_type(&mut phone,"snapshot").await["state"].clone();
    assert_ne!(state["epoch"],old_state["epoch"]);assert!(state["media"].is_null());
    let rejected=command(&mut phone,&old_state,1,json!({"type":"play"})).await;
    assert_eq!(rejected["error"],"stale epoch");
    phone.close(None).await.unwrap();drop(phone);task.abort();let _=task.await;
}
#[tokio::test] async fn native_execution_and_terminal_failure_are_role_scoped_over_websocket() {
    let (url,task)=start().await;
    let (mut pc,_)=connect_async(request(&url,PC)).await.unwrap();read_type(&mut pc,"snapshot").await;
    let (mut phone,_)=connect_async(request(&url,PHONE)).await.unwrap();read_type(&mut phone,"snapshot").await;
    let state=read_type(&mut pc,"snapshot").await["state"].clone();
    let loaded=command(&mut pc,&state,1,json!({"type":"load","media":{"provider":"fixture","torrent_id":0,"file_id":0}})).await;
    let pc_ready=command(&mut pc,&loaded["state"],2,json!({"type":"ready","position_ms":0})).await;
    let committed=command(&mut phone,&pc_ready["state"],1,json!({"type":"ready","position_ms":0})).await;
    assert!(committed["error"].is_null());assert!(committed["state"]["prepare"].is_null());
    let executed=command(&mut phone,&committed["state"],2,json!({"type":"executed","playing":false,"position_ms":0,"observed_server_ms":committed["server_ms"]})).await;
    assert!(executed["error"].is_null());assert_eq!(executed["state"]["participants"]["phone"]["execution"]["playing"],false);
    assert!(executed["state"]["participants"]["pc"]["execution"].is_null());
    let failed=command(&mut phone,&executed["state"],3,json!({"type":"player_failed","code":"decoder_failed"})).await;
    assert!(failed["error"].is_null());assert_eq!(failed["state"]["fault"],"player_failed_phone");
    let stale=command(&mut phone,&executed["state"],4,json!({"type":"executed","playing":false,"position_ms":0,"observed_server_ms":failed["server_ms"]})).await;
    assert_eq!(stale["error"],"stale generation");
    let recovered=command(&mut phone,&failed["state"],5,json!({"type":"player_recovered"})).await;
    assert!(recovered["error"].is_null());assert!(recovered["state"]["prepare"].is_object());
    phone.close(None).await.unwrap();pc.close(None).await.unwrap();task.abort();
}
#[tokio::test] async fn rtt_is_measured_from_server_ping_nonce() {
    let(url,task)=start().await;let(mut pc,_)=connect_async(request(&url,PC)).await.unwrap();
    read_type(&mut pc,"snapshot").await;
    let nonce=loop {if let Message::Ping(bytes)=pc.next().await.unwrap().unwrap() {break bytes;}};
    tokio::time::sleep(std::time::Duration::from_millis(120)).await;
    pc.send(Message::Pong(nonce)).await.unwrap();
    let state=read_type(&mut pc,"snapshot").await;
    assert!(state["state"]["participants"]["pc"]["rtt_ms"].as_u64().unwrap()>=100);
    pc.close(None).await.unwrap();task.abort();
}

#[tokio::test] async fn delayed_pong_survives_the_next_heartbeat_tick() {
    let(url,task)=start().await;let(mut pc,_)=connect_async(request(&url,PC)).await.unwrap();
    read_type(&mut pc,"snapshot").await;
    let nonce=loop {if let Message::Ping(bytes)=pc.next().await.unwrap().unwrap() {break bytes;}};
    // The two-second heartbeat must not overwrite an unexpired ping nonce.
    // RTTs up to five seconds are accepted by the authoritative reducer.
    tokio::time::sleep(std::time::Duration::from_millis(2200)).await;
    pc.send(Message::Pong(nonce)).await.unwrap();
    let state=read_type(&mut pc,"snapshot").await;
    let measured=state["state"]["participants"]["pc"]["rtt_ms"].as_u64().unwrap();
    pc.close(None).await.unwrap();task.abort();
    assert!((2000..=5000).contains(&measured),"Unexpired delayed pong did not preserve its RTT");
}

#[tokio::test] async fn access_rejections_are_observable_without_peer_identity_or_credentials() {
    use tokio::io::{AsyncReadExt,AsyncWriteExt};
    let directory=std::env::temp_dir().join(format!("together-access-journal-{}",uuid::Uuid::new_v4()));
    let host=together_session_server::Host::new(PC.into(),PHONE.into()).unwrap();
    host.enable_diagnostics(directory.clone()).unwrap();
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
    let url=format!("ws://{address}/session");let server=host.clone();
    let task=tokio::spawn(async move {server.serve(listener).await.unwrap();});
    let before=serde_json::to_value(host.snapshot().await).unwrap();
    for token in [None,Some("PRIVATE_WRONG_BEARER".to_string()),Some("PRIVATE_OVERSIZED".repeat(30))] {
        let request=match token {None=>url.clone().into_client_request().unwrap(),Some(token)=>request(&url,&token)};
        let error=connect_async(request).await.unwrap_err();
        match error {tokio_tungstenite::tungstenite::Error::Http(response)=>assert_eq!(response.status(),401),_=>panic!("expected HTTP auth rejection")}
    }
    // Actual HTTP pair route, with no open pairing window; no arbitrary client
    // metadata, input code or address may enter diagnostics.
    let body=r#"{"code":"PRIVATE_PAIR_CODE"}"#;
    let mut socket=tokio::net::TcpStream::connect(address).await.unwrap();
    socket.write_all(format!("POST /pair HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
    let mut response=String::new();tokio::time::timeout(std::time::Duration::from_secs(3),socket.read_to_string(&mut response)).await.unwrap().unwrap();
    assert!(response.starts_with("HTTP/1.1 403"));
    assert_eq!(serde_json::to_value(host.snapshot().await).unwrap(),before);
    // A valid connection still works after rejection; duplicate role remains
    // rejected rather than disrupting the authenticated client.
    let (mut phone,_)=connect_async(request(&url,PHONE)).await.unwrap();
    read_type(&mut phone,"snapshot").await;
    let error=connect_async(request(&url,PHONE)).await.unwrap_err();
    match error {tokio_tungstenite::tungstenite::Error::Http(response)=>assert_eq!(response.status(),409),_=>panic!("expected duplicate role rejection")}
    phone.send(Message::Text(json!({"type":"clock","protocol":1,"t0_ms":1}).to_string().into())).await.unwrap();
    read_type(&mut phone,"clock").await;
    let text=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
    for private in [PC,PHONE,"PRIVATE_WRONG_BEARER","PRIVATE_OVERSIZED","PRIVATE_PAIR_CODE",&address.to_string()] {assert!(!text.contains(private));}
    let rows:Vec<Value>=text.lines().map(|line|serde_json::from_str(line).unwrap()).filter(|r:&Value|r["record"]["event"]=="access").collect();
    assert_eq!(rows.len(),5);
    for row in &rows[..3] {assert_eq!(row["record"]["sample"]["endpoint"],"session");assert_eq!(row["record"]["sample"]["kind"],"authentication_rejected");}
    assert_eq!(rows[3]["record"]["sample"]["endpoint"],"pair");assert_eq!(rows[3]["record"]["sample"]["kind"],"pairing_rejected");
    assert_eq!(rows[4]["record"]["sample"]["kind"],"role_in_use");
    assert!(rows.iter().all(|r|r["record"]["sample"].as_object().unwrap().len()==3));
    assert_eq!(host.diagnostic_health().unwrap().dropped_records,0);assert_eq!(host.diagnostic_health().unwrap().io_errors,0);
    phone.close(None).await.unwrap();drop(phone);task.abort();let _=task.await;drop(host);
    for _ in 0..100 {if std::fs::remove_dir_all(&directory).is_ok(){break;}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
    assert!(!directory.exists());
}

#[tokio::test] async fn requested_shutdown_drains_phone_releases_listener_and_stops_ticker() {
    use together_session_server::{Host,Action};
    let host=Host::new(PC.into(),PHONE.into()).unwrap();host.attach_local_pc().await.unwrap();
    host.local_pc(None,Action::Load {media:together_sync_core::MediaIdentity::fixture()}).await.unwrap();
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
    let server=host.clone();let task=tokio::spawn(async move {server.serve(listener).await});
    let (mut phone,_)=connect_async(request(&format!("ws://{address}/session"),PHONE)).await.unwrap();
    let state=read_type(&mut phone,"snapshot").await["state"].clone();
    host.local_pc(Some(state["generation"].as_u64().unwrap()),Action::Ready {position_ms:0}).await.unwrap();
    let ready=command(&mut phone,&state,1,json!({"type":"ready","position_ms":0})).await;
    let play=command(&mut phone,&ready["state"],2,json!({"type":"play"})).await;
    host.local_pc(Some(play["state"]["generation"].as_u64().unwrap()),Action::Ready {position_ms:0}).await.unwrap();
    let commit=command(&mut phone,&play["state"],3,json!({"type":"ready","position_ms":0})).await;
    assert_eq!(commit["state"]["timeline"]["playing"],true);
    host.request_shutdown();host.request_shutdown();
    assert_eq!(host.local_pc(None,Action::Play).await,Err("server stopping"));
    let close=tokio::time::timeout(std::time::Duration::from_secs(2),async {
        loop {match phone.next().await.unwrap().unwrap() {
            Message::Close(Some(frame))=>break frame,
            Message::Ping(payload)=>{let _=phone.send(Message::Pong(payload)).await;},_=>{}
        }}
    }).await.unwrap();
    assert_eq!(close.code,tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Restart);
    assert_eq!(close.reason,"server_restart");drop(phone);
    tokio::time::timeout(std::time::Duration::from_secs(2),task).await.unwrap().unwrap().unwrap();
    let frozen=host.snapshot().await;assert!(!frozen.timeline.playing);assert!(!frozen.participants["phone"].connected);
    assert!(frozen.desired_playing);
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    assert_eq!(serde_json::to_value(host.snapshot().await).unwrap(),serde_json::to_value(frozen).unwrap(),"shutdown left a ticker mutating authority");
    let replacement=tokio::net::TcpListener::bind(address).await.unwrap();drop(replacement);
    let weak=std::sync::Arc::downgrade(&host);drop(host);
    assert!(weak.upgrade().is_none(),"graceful shutdown retained Host");
}

// ACKs serialize on the same socket as snapshots. A newer authoritative state
// may reject a queued phone command; its receipt must not carry older authority
// than snapshots already accepted by that connection.
#[tokio::test]
async fn acknowledgements_preserve_authority_order_and_reconnect_sequence_owner() {
    use together_session_server::{Host,Action};
    let directory=std::env::var_os("TOGETHER_ACK_ORDER_DIR").map(std::path::PathBuf::from)
        .unwrap_or_else(||std::env::temp_dir().join(format!("together-ack-order-{}",uuid::Uuid::new_v4())));
    let host=Host::new(PC.into(),PHONE.into()).unwrap();
    host.enable_diagnostics(directory.join("journal")).unwrap();
    host.attach_local_pc().await.unwrap();
    host.local_pc(None,Action::Load {media:together_sync_core::MediaIdentity::fixture()}).await.unwrap();
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url=format!("ws://{}/session",listener.local_addr().unwrap());
    let server_host=host.clone();let server=tokio::spawn(async move {server_host.serve(listener).await});
    let mut receipts=Vec::new();
    for connection in 0..2 {
        let (mut phone,_)=connect_async(request(&url,PHONE)).await.unwrap();
        let mut state=read_type(&mut phone,"snapshot").await["state"].clone();
        let epoch=state["epoch"].clone();let mut accepted_revision=state["revision"].as_u64().unwrap();
        let mut snapshots=0;let mut stale_rejections=0;let mut accepted_commands=0;
        for seq in 1..=12 {
            let superseded=seq%2==1 && !(connection==1 && seq==1);
            if superseded {
                // Complete PC replacement before the old phone frame is submitted.
                host.local_pc(None,Action::Seek {position_ms:seq*1000}).await.unwrap();
            }
            phone.send(Message::Text(json!({"type":"command","protocol":1,"epoch":state["epoch"],
                "generation":state["generation"],"seq":seq,"action":{"type":"pause"}}).to_string().into())).await.unwrap();
            let ack=tokio::time::timeout(std::time::Duration::from_secs(3),async {
                loop {
                    match phone.next().await.unwrap().unwrap() {
                        Message::Text(raw)=>{
                            let frame:Value=serde_json::from_str(&raw).unwrap();
                            if frame["type"]=="snapshot" {
                                assert_eq!(frame["state"]["epoch"],epoch);
                                accepted_revision=accepted_revision.max(frame["state"]["revision"].as_u64().unwrap());
                                snapshots+=1;
                            } else if frame["type"]=="ack" {break frame;}
                        },
                        Message::Ping(payload)=>phone.send(Message::Pong(payload)).await.unwrap(),
                        _=>{},
                    }
                }
            }).await.unwrap();
            assert_eq!(ack["seq"],seq);assert_eq!(ack["state"]["epoch"],epoch);
            let ack_revision=ack["state"]["revision"].as_u64().unwrap();
            assert!(ack_revision>=accepted_revision,"ACK older than accepted socket state");
            accepted_revision=ack_revision;
            if superseded {
                assert_eq!(ack["error"],"stale generation");stale_rejections+=1;
                assert_eq!(ack["state"]["prepare"]["position_ms"],seq*1000);
            } else {assert!(ack["error"].is_null());accepted_commands+=1;}
            if connection==1 && seq==1 {assert!(ack["error"].is_null(),"replacement socket must accept sequence1");}
            state=ack["state"].clone();
        }
        receipts.push(json!({"connection":connection,"first_sequence":1,"last_sequence":12,
            "stale_generation_rejections":stale_rejections,"accepted_commands":accepted_commands,
            "snapshots_observed":snapshots,"last_accepted_revision":accepted_revision}));
        phone.close(None).await.unwrap();drop(phone);
        tokio::time::timeout(std::time::Duration::from_secs(3),async {
            while host.snapshot().await.participants["phone"].connected {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        }).await.unwrap();
    }
    host.request_shutdown();
    tokio::time::timeout(std::time::Duration::from_secs(3),server).await.unwrap().unwrap().unwrap();
    host.flush_diagnostics().await.unwrap();let health=host.diagnostic_health().unwrap();
    let exported=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
    assert!(!exported.contains(PHONE) && !exported.contains(PC) && !exported.contains("Bearer") && !exported.contains("ws://"));
    std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&json!({
        "scope":"actual loopback server ACK ordering and socket sequence ownership; synthetic phone, no decoder",
        "connections":receipts,"journal_errors":health.io_errors,"journal_drops":health.dropped_records,
        "android_runtime_verified":false,"physical_gate_passed":false,"wan_verified":false
    })).unwrap()).unwrap();
    assert_eq!(health.io_errors,0);assert_eq!(health.dropped_records,0);
}

/// Server evidence for the client's causal ACK rule; no Android queue reimplementation.
#[tokio::test]
async fn causal_ack_result_distinguishes_own_advance_from_pc_replacement() {
    use together_session_server::{Host,Action};
    let directory=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/source-validation")
        .join(format!("causal-ack-{}",uuid::Uuid::new_v4()));
    let host=Host::new(PC.into(),PHONE.into()).unwrap();
    host.enable_diagnostics(directory.join("journal")).unwrap();
    host.attach_local_pc().await.unwrap();
    host.local_pc(None,Action::Load {media:together_sync_core::MediaIdentity::fixture()}).await.unwrap();
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address=listener.local_addr().unwrap();let url=format!("ws://{address}/session");
    let server_host=host.clone();let server=tokio::spawn(async move {server_host.serve(listener).await});
    let (mut phone,_)=connect_async(request(&url,PHONE)).await.unwrap();
    let mut state=read_type(&mut phone,"snapshot").await["state"].clone();
    let epoch=state["epoch"].clone();let mut sequence=0;let mut receipts=Vec::new();
    for action in [json!({"type":"play"}),json!({"type":"pause"}),json!({"type":"seek","position_ms":4000})] {
        let base=state["generation"].as_u64().unwrap();sequence+=1;
        let ack=command(&mut phone,&state,sequence,action.clone()).await;
        receipts.push(json!({"phase":"own_command","kind":action["type"],"base":base,"ack":ack}));
        std::fs::write(directory.join("partial.json"),serde_json::to_vec_pretty(&receipts).unwrap()).unwrap();
        assert_eq!(ack["seq"],sequence);assert!(ack["error"].is_null());assert_eq!(ack["state"]["epoch"],epoch);
        assert_eq!(ack["state"]["generation"],base+1);
        match action["type"].as_str().unwrap() {
            "play"=>assert_eq!(ack["state"]["desired_playing"],true),
            "pause"=>{assert_eq!(ack["state"]["desired_playing"],false);assert!(ack["state"]["prepare"].is_null());},
            "seek"=>assert_eq!(ack["state"]["prepare"]["position_ms"],4000),_=>unreachable!(),
        }
        let queued_base=ack["state"].clone();
        let pc_target=10000+sequence*1000;
        // Awaited authority command, not a timing sleep, establishes replacement.
        host.local_pc(Some(base+1),Action::Seek {position_ms:pc_target}).await.unwrap();
        let replacement=serde_json::to_value(host.snapshot().await).unwrap();sequence+=1;
        let rejected=command(&mut phone,&queued_base,sequence,json!({"type":"pause"})).await;
        receipts.push(json!({"phase":"pc_replaced","base":base+1,"ack":rejected}));
        assert_eq!(rejected["seq"],sequence);assert_eq!(rejected["error"],"stale generation");
        assert_eq!(rejected["state"]["generation"],base+2);
        for field in ["epoch","generation","media","timeline","prepare","desired_playing","ended","fault"] {
            assert_eq!(rejected["state"][field],replacement[field],"stale command changed {field}");
        }
        sequence+=1;
        let fresh=command(&mut phone,&rejected["state"],sequence,json!({"type":"pause"})).await;
        receipts.push(json!({"phase":"fresh_control","base":base+2,"ack":fresh}));
        assert!(fresh["error"].is_null());assert_eq!(fresh["seq"],sequence);
        assert_eq!(fresh["state"]["generation"],base+3);state=fresh["state"].clone();
    }
    let mut wrong_epoch=state.clone();wrong_epoch["epoch"]=json!("obsolete-test-room");sequence+=1;
    let rejected=command(&mut phone,&wrong_epoch,sequence,json!({"type":"pause"})).await;
    assert_eq!(rejected["error"],"stale epoch");assert_eq!(rejected["seq"],sequence);
    assert_eq!(rejected["state"]["generation"],state["generation"]);
    receipts.push(json!({"phase":"wrong_epoch","ack":rejected}));
    phone.close(None).await.unwrap();drop(phone);
    host.request_shutdown();tokio::time::timeout(std::time::Duration::from_secs(3),server).await.unwrap().unwrap().unwrap();
    host.flush_diagnostics().await.unwrap();let health=host.diagnostic_health().unwrap();
    let exported=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
    assert!(!exported.contains(PHONE) && !exported.contains(PC) && !exported.contains("Bearer") && !exported.contains("ws://"));
    assert_eq!(health.io_errors,0);assert_eq!(health.dropped_records,0);
    let rows:Vec<Value>=exported.lines().map(|line|serde_json::from_str(line).unwrap()).collect();
    let recorded:Vec<&Value>=rows.iter().filter(|row|row["record"]["event"]=="command" && row["record"]["sample"]["actor"]=="phone")
        .map(|row|&row["record"]["sample"]).collect();
    assert_eq!(recorded.len(),receipts.len());
    for (row,receipt) in recorded.iter().zip(&receipts) {
        let ack=&receipt["ack"];
        assert_eq!(row["sequence"],ack["seq"]);assert_eq!(row["resulting_generation"],ack["state"]["generation"]);
        assert_eq!(row["revision"],ack["state"]["revision"]);
        let expected=match receipt["phase"].as_str().unwrap() {"pc_replaced"=>"stale_generation","wrong_epoch"=>"stale_epoch",_=>"accepted"};
        assert_eq!(row["outcome"],expected);
    }
    let final_state=host.snapshot().await;assert!(!final_state.participants["phone"].connected);
    let replacement=tokio::net::TcpListener::bind(address).await.unwrap();drop(replacement);
    std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&json!({
        "scope":"real loopback server causal ACK results; synthetic phone commands, no decoder",
        "commands":receipts,"journal_errors":health.io_errors,"journal_drops":health.dropped_records,
        "phone_detached":true,"listener_released":true,"android_runtime_verified":false,
        "native_playback_verified":false,"physical_gate_passed":false,"wan_verified":false
    })).unwrap()).unwrap();
}

#[tokio::test]
async fn internal_feedback_acknowledgements_have_zero_or_one_causal_advance() {
    use together_session_server::{Host,Action};
    let directory=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/source-validation")
        .join(format!("feedback-ack-{}",uuid::Uuid::new_v4()));
    let host=Host::new(PC.into(),PHONE.into()).unwrap();host.enable_diagnostics(directory.join("journal")).unwrap();
    host.attach_local_pc().await.unwrap();host.local_pc(None,Action::Load {media:together_sync_core::MediaIdentity::fixture()}).await.unwrap();
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
    let url=format!("ws://{address}/session");let serving=host.clone();let server=tokio::spawn(async move {serving.serve(listener).await});
    let (mut phone,_)=connect_async(request(&url,PHONE)).await.unwrap();
    let mut state=read_type(&mut phone,"snapshot").await["state"].clone();let mut receipts=Vec::new();
    let cases=[
        (json!({"type":"ready","position_ms":0}),0),
        (json!({"type":"ready","position_ms":0}),0),
        (json!({"type":"buffering","active":true}),1),
        (json!({"type":"buffering","active":true}),0),
        (json!({"type":"buffering","active":false}),1),
        (json!({"type":"player_failed","code":"command_failed"}),1),
        (json!({"type":"player_failed","code":"command_failed"}),0),
        (json!({"type":"player_recovered"}),1),
        (json!({"type":"player_recovered"}),0),
        (json!({"type":"ready","position_ms":0}),0),
        (json!({"type":"executed","playing":false,"position_ms":0}),0),
        (json!({"type":"executed","playing":false,"position_ms":0}),0),
    ];
    for (index,(mut action,delta)) in cases.into_iter().enumerate() {
        if index==10 {
            host.local_pc(Some(state["generation"].as_u64().unwrap()),Action::Ready {position_ms:0}).await.unwrap();
            state=serde_json::to_value(host.snapshot().await).unwrap();assert!(state["prepare"].is_null());
        }
        if action["type"]=="executed" {action["observed_server_ms"]=json!(host.now());}
        let base=state["generation"].as_u64().unwrap();let sequence=index as u64+1;
        let ack=command(&mut phone,&state,sequence,action.clone()).await;
        receipts.push(json!({"kind":action["type"],"base":base,"expected_delta":delta,"ack":ack}));
        std::fs::write(directory.join("partial.json"),serde_json::to_vec_pretty(&receipts).unwrap()).unwrap();
        assert!(ack["error"].is_null());assert_eq!(ack["seq"],sequence);assert_eq!(ack["state"]["epoch"],state["epoch"]);
        assert_eq!(ack["state"]["generation"],base+delta);
        match action["type"].as_str().unwrap() {
            "player_failed"=>{assert_eq!(ack["state"]["fault"],"player_failed_phone");assert!(ack["state"]["prepare"].is_null());
                assert_eq!(ack["state"]["participants"]["phone"]["player_fault"],"command_failed");},
            "player_recovered"=>{assert!(ack["state"]["fault"].is_null());assert!(ack["state"]["participants"]["phone"]["player_fault"].is_null());},
            "buffering"=>assert_eq!(ack["state"]["participants"]["phone"]["buffering"],action["active"]),
            "executed"=>assert_eq!(ack["state"]["participants"]["phone"]["execution"]["playing"],false),_=>{},
        }
        state=ack["state"].clone();
    }
    phone.close(None).await.unwrap();drop(phone);host.request_shutdown();
    tokio::time::timeout(std::time::Duration::from_secs(3),server).await.unwrap().unwrap().unwrap();host.flush_diagnostics().await.unwrap();
    let health=host.diagnostic_health().unwrap();assert_eq!(health.io_errors,0);assert_eq!(health.dropped_records,0);
    let exported=String::from_utf8(host.export_diagnostics().await.unwrap()).unwrap();
    assert!(!exported.contains(PC) && !exported.contains(PHONE) && !exported.contains("Bearer") && !exported.contains("ws://"));
    let rows:Vec<Value>=exported.lines().map(|line|serde_json::from_str(line).unwrap()).collect();
    let recorded:Vec<&Value>=rows.iter().filter(|row|row["record"]["event"]=="command" && row["record"]["sample"]["actor"]=="phone")
        .map(|row|&row["record"]["sample"]).collect();assert_eq!(recorded.len(),receipts.len());
    for (row,receipt) in recorded.iter().zip(&receipts) {
        assert_eq!(row["sequence"],receipt["ack"]["seq"]);assert_eq!(row["outcome"],"accepted");
        assert_eq!(row["generation"],receipt["base"]);assert_eq!(row["resulting_generation"],receipt["ack"]["state"]["generation"]);
        assert_eq!(row["revision"],receipt["ack"]["state"]["revision"]);
    }
    assert!(!host.snapshot().await.participants["phone"].connected);
    let rebound=tokio::net::TcpListener::bind(address).await.unwrap();drop(rebound);
    std::fs::write(directory.join("receipt.json"),serde_json::to_vec_pretty(&json!({"commands":receipts,
        "journal_errors":health.io_errors,"journal_drops":health.dropped_records,"phone_detached":true,"listener_released":true,
        "scope":"actual loopback internal feedback ACK contract; synthetic readiness/execution/fault, no native player",
        "android_runtime_verified":false,"native_playback_verified":false,"physical_gate_passed":false,"wan_verified":false})).unwrap()).unwrap();
}

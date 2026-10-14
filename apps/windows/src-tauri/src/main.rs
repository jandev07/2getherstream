#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::Arc;
use together_session_server::Host;
use tauri::Manager;
mod player;
mod runtime_assets;
mod torbox;
mod stremio;
mod frontend_assets;
mod startup_proof;
mod owned_transition;
mod android_download;
struct TemporaryConnection {owner:Option<together_session_server::tunnel::QuickTunnel>,origin:Option<String>,error:Option<&'static str>}
struct AppLifecycle {
    public_origin:Option<String>,
    temporary:Arc<tokio::sync::Mutex<TemporaryConnection>>,
    temporary_executable:std::path::PathBuf,temporary_runtime:std::path::PathBuf,
    started:std::sync::atomic::AtomicBool,finished:std::sync::atomic::AtomicBool,
    tunnel:tokio::sync::Mutex<Option<together_session_server::tunnel::TunnelSupervisor>>,
    tunnel_status:Option<tokio::sync::watch::Receiver<together_session_server::tunnel::TunnelStatus>>,
    tunnel_setup_failed:bool,
    server:tokio::sync::Mutex<Option<tauri::async_runtime::JoinHandle<Result<(),&'static str>>>>,
}
fn lifecycle_event(app:&tauri::AppHandle,event:tauri::RunEvent) {
    use std::sync::atomic::Ordering;
    match event {
        tauri::RunEvent::WindowEvent {label,event:tauri::WindowEvent::CloseRequested {api,..},..} if label=="main" => {
            // Keep the video surface alive until its owner tears down libmpv.
            if let Some(lifecycle)=app.try_state::<AppLifecycle>() {
                if !lifecycle.finished.load(Ordering::Acquire) {api.prevent_close();app.exit(0);}
            }
        },
        tauri::RunEvent::ExitRequested {api,..} => {
            let Some(lifecycle)=app.try_state::<AppLifecycle>() else {return;};
            if lifecycle.finished.load(Ordering::Acquire) {return;}
            api.prevent_exit();
            if lifecycle.started.swap(true,Ordering::AcqRel) {return;}
            let app=app.clone();
            tauri::async_runtime::spawn(async move {
                use together_session_server::diagnostics::AppLifecycleKind as Stage;
                let host=app.state::<Arc<Host>>();host.record_app_lifecycle(Stage::ShutdownRequested);
                host.request_shutdown();
                let lifecycle=app.state::<AppLifecycle>();
                let temporary=lifecycle.temporary.clone().lock_owned().await;
                let result=owned_transition::blocking(temporary,|state| {
                    if let Some(mut owner)=state.owner.take() {owner.stop()} else {Ok(())}
                }).await;
                if !matches!(result,Ok(Ok(()))) {host.record_app_lifecycle(Stage::TunnelStopFailed);}
                if let Some(tunnel)=lifecycle.tunnel.lock().await.take() {
                    match tunnel.shutdown().await {
                        Ok(())=>host.record_app_lifecycle(Stage::TunnelStopped),
                        Err(code)=>{host.record_app_lifecycle(Stage::TunnelStopFailed);eprintln!("{code}");},
                    }
                }
                match app.state::<player::PlayerController>().shutdown().await {
                    Ok(())=>host.record_app_lifecycle(Stage::PlayerStopped),
                    Err(code)=>{host.record_app_lifecycle(Stage::PlayerStopFailed);eprintln!("{code}");},
                }
                let lifecycle=app.state::<AppLifecycle>();
                if let Some(mut server)=lifecycle.server.lock().await.take() {
                    match tokio::time::timeout(std::time::Duration::from_secs(3),&mut server).await {
                        Ok(Ok(Ok(())))=>host.record_app_lifecycle(Stage::ServerStopped),
                        Ok(_)=>{host.record_app_lifecycle(Stage::ServerStopFailed);eprintln!("session_shutdown_failed");},
                        Err(_)=>{server.abort();host.record_app_lifecycle(Stage::ServerStopFailed);eprintln!("session_shutdown_timeout");},
                    }
                }
                // ExitPermitted records the decision even after a cleanup failure;
                // it must not imply every worker successfully stopped.
                host.record_app_lifecycle(Stage::ExitPermitted);
                if let Err(code)=host.flush_diagnostics().await {eprintln!("{code}");}
                lifecycle.finished.store(true,Ordering::Release);app.exit(0);
            });
        },_=>{},
    }
}
#[tauri::command]
fn player_status(player: tauri::State<'_, player::PlayerController>) -> player::Status { player.status.lock().unwrap().clone() }
#[tauri::command]
fn private_alpha_account_links()->bool {torbox::private_alpha_account_links()}
#[tauri::command]
async fn stremio_connect(controller:tauri::State<'_,stremio::Controller>)->Result<stremio::Browser,String> {controller.connect().await.map_err(String::from)}
#[tauri::command]
async fn stremio_browser(controller:tauri::State<'_,stremio::Controller>)->Result<stremio::Browser,String> {Ok(controller.browser().await)}
#[tauri::command]
async fn stremio_browse(controller:tauri::State<'_,stremio::Controller>,revision:u64,index:usize,search:Option<String>)->Result<stremio::Browser,String> {controller.browse(revision,index,search).await.map_err(String::from)}
#[tauri::command]
async fn stremio_options(controller:tauri::State<'_,stremio::Controller>,torbox:tauri::State<'_,torbox::Controller>,revision:u64,index:usize)->Result<stremio::Options,String> {controller.options(&torbox,revision,index).await.map_err(String::from)}
#[tauri::command]
async fn stremio_select(controller:tauri::State<'_,stremio::Controller>,torbox:tauri::State<'_,torbox::Controller>,host:tauri::State<'_,Arc<Host>>,revision:u64,index:usize, player:tauri::State<'_,player::PlayerController>)->Result<(),String> {player.ensure_available().map_err(String::from)?;controller.select(&torbox,&host,revision,index).await.map_err(String::from)}
#[tauri::command]
async fn stremio_prepare(controller:tauri::State<'_,stremio::Controller>,torbox:tauri::State<'_,torbox::Controller>,revision:u64,index:usize)->Result<stremio::PreparationStatus,String> {controller.prepare(&torbox,revision,index).await.map_err(String::from)}
#[tauri::command]
async fn stremio_episodes(controller:tauri::State<'_,stremio::Controller>,revision:u64,index:usize)->Result<stremio::Browser,String> {controller.episodes(revision,index).await.map_err(String::from)}
#[tauri::command]
async fn stremio_cancel_lookup(controller:tauri::State<'_,stremio::Controller>)->Result<stremio::Browser,String> {controller.cancel_lookup().await.map_err(String::from)}
#[tauri::command]
async fn stremio_back(controller:tauri::State<'_,stremio::Controller>,revision:u64)->Result<stremio::Browser,String> {controller.back(revision).await.map_err(String::from)}
#[tauri::command]
async fn torbox_videos(controller:tauri::State<'_,torbox::Controller>)->Result<Vec<together_sync_core::MediaIdentity>,String> {
    controller.videos().await.map_err(String::from)
}
#[tauri::command]
async fn torbox_select(controller:tauri::State<'_,torbox::Controller>,host:tauri::State<'_,Arc<Host>>,torrent_id:u64,file_id:u64, player:tauri::State<'_,player::PlayerController>)->Result<(),String> {
    player.ensure_available().map_err(String::from)?;
    controller.select(&host,torrent_id,file_id).await.map_err(String::from)
}
#[tauri::command]
async fn torbox_refresh(controller:tauri::State<'_,torbox::Controller>,host:tauri::State<'_,Arc<Host>>, player:tauri::State<'_,player::PlayerController>)->Result<(),String> {
    player.ensure_available().map_err(String::from)?;
    controller.refresh(&host).await.map_err(String::from)
}
#[tauri::command]
async fn stremio_copied_source(controller:tauri::State<'_,torbox::Controller>,host:tauri::State<'_,Arc<Host>>,link:String, player:tauri::State<'_,player::PlayerController>)->Result<(),String> {
    player.ensure_available().map_err(String::from)?;
    controller.copied_source(&host,link).await.map_err(String::from)
}
#[tauri::command]
async fn player_intent(host: tauri::State<'_, Arc<Host>>, intent: String, position_ms: Option<u64>, player:tauri::State<'_,player::PlayerController>) -> Result<(),String> {
    player.ensure_available().map_err(String::from)?;
    use together_session_server::Action;
    let action = match intent.as_str() {
        "load_test" => Action::Load {media:together_sync_core::MediaIdentity::fixture()},
        "play" => Action::Play,
        "pause" => Action::Pause,
        "seek" => Action::Seek {position_ms:position_ms.ok_or("Missing seek target")?},
        _ => return Err("Unknown player intention".into()),
    };
    host.local_pc(None,action).await.map_err(String::from)
}
#[tauri::command]
fn video_bounds(window: tauri::WebviewWindow, player: tauri::State<'_,player::PlayerController>, x: f64, y: f64, width: f64, height: f64) -> Result<(),String> {
    if [x,y,width,height].iter().any(|n| !n.is_finite() || *n < 0.0 || *n > 10000.0) { return Err("Invalid bounds".into()); }
    let scale = window.scale_factor().map_err(|_| "Window scale unavailable")?;
    let hwnd = player.hwnd;
    window.run_on_main_thread(move || unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::MoveWindow(hwnd as _, (x*scale) as i32, (y*scale) as i32, (width*scale) as i32, (height*scale) as i32,1);
    }).map_err(|_| "Video surface unavailable".into())
}
#[tauri::command]
async fn session_snapshot(host: tauri::State<'_, Arc<Host>>) -> Result<together_sync_core::Snapshot, String> {
    Ok(host.snapshot().await)
}
#[tauri::command]
async fn open_pairing(host:tauri::State<'_,Arc<Host>>)->Result<String,String> {host.open_pairing().await.map_err(String::from)}
#[tauri::command]
async fn cancel_pairing(host:tauri::State<'_,Arc<Host>>)->Result<(),String> {host.cancel_pairing().await;Ok(())}
#[tauri::command]
fn diagnostic_health(host:tauri::State<'_,Arc<Host>>)->Option<together_session_server::diagnostics::Health> {host.diagnostic_health()}
#[tauri::command]
async fn export_session_log(app:tauri::AppHandle,host:tauri::State<'_,Arc<Host>>)->Result<String,String> {
    let bytes=host.export_diagnostics().await.map_err(String::from)?;
    let directory=app.path().app_local_data_dir().map_err(|_|"Diagnostic folder unavailable")?.join("diagnostics").join("exports");
    tokio::fs::create_dir_all(&directory).await.map_err(|_|"Could not create diagnostic export folder")?;
    let path=directory.join(format!("session-{}.jsonl",uuid::Uuid::new_v4()));
    let mut file=tokio::fs::OpenOptions::new().write(true).create_new(true).open(&path).await.map_err(|_|"Could not create diagnostic export")?;
    use tokio::io::AsyncWriteExt;
    file.write_all(&bytes).await.map_err(|_|"Could not write diagnostic export")?;
    file.flush().await.map_err(|_|"Could not flush diagnostic export")?;
    Ok(path.to_string_lossy().into_owned())
}
#[tauri::command]
fn tunnel_status(lifecycle:tauri::State<'_,AppLifecycle>)->serde_json::Value {
    match lifecycle.tunnel_status.as_ref() {
        Some(status)=>serde_json::json!({"mode":"configured","process":*status.borrow()}),
        None=>serde_json::json!({"mode":if lifecycle.tunnel_setup_failed {"setup_failed"}else{"unconfigured"}}),
    }
}
#[tauri::command]
async fn start_temporary_connection(lifecycle:tauri::State<'_,AppLifecycle>)->Result<(),String> {
    if !torbox::private_alpha_account_links() {return Err("Temporary connection is available in the private Alpha only".into());}
    if lifecycle.started.load(std::sync::atomic::Ordering::Acquire) {return Err("Windows is closing".into());}
    if lifecycle.public_origin.is_some() {return Err("A dedicated control tunnel is already configured".into());}
    let state=lifecycle.temporary.clone().lock_owned().await;
    if lifecycle.started.load(std::sync::atomic::Ordering::Acquire) {return Err("Windows is closing".into());}
    if state.owner.is_some(){return Ok(());}
    let executable=lifecycle.temporary_executable.clone();let runtime=lifecycle.temporary_runtime.clone();
    owned_transition::blocking(state,move |state| {
        match together_session_server::tunnel::QuickTunnel::start(&executable,&runtime) {
            Ok(owner)=>{state.owner=Some(owner);state.origin=None;state.error=None;Ok(())},
            Err(_)=>{state.error=Some("Could not start the temporary phone connection");Err("Could not start the temporary phone connection".into())},
        }
    }).await.map_err(|_|"Could not start the temporary phone connection".to_string())?
}
#[tauri::command]
async fn stop_temporary_connection(lifecycle:tauri::State<'_,AppLifecycle>)->Result<(),String> {
    let state=lifecycle.temporary.clone().lock_owned().await;
    owned_transition::blocking(state,|state| {
        if let Some(mut owner)=state.owner.take() {
            owner.stop().map_err(|_|"Could not close the temporary connection".to_string())?;
        }
        state.origin=None;state.error=None;Ok(())
    }).await.map_err(|_|"Could not close the temporary connection".to_string())?
}
#[tauri::command]
async fn pairing_connection(lifecycle:tauri::State<'_,AppLifecycle>)->Result<serde_json::Value,String> {
    let mut address=lifecycle.public_origin.clone();
    let mut tunnel=match lifecycle.tunnel_status.as_ref() {
        Some(status)=>serde_json::json!({"mode":"configured","process":*status.borrow()}),
        None=>serde_json::json!({"mode":if lifecycle.tunnel_setup_failed {"setup_failed"}else{"unconfigured"}}),
    };
    let state=lifecycle.temporary.lock().await;
    if state.owner.is_some() {
        address=state.origin.clone();tunnel=serde_json::json!({"mode":"temporary","process":{"state":if address.is_some() {"address_ready"}else{"starting"}}});
    }
    Ok(serde_json::json!({"emulator_address":"http://127.0.0.1:47820","phone_address":address,"tunnel":tunnel,"error":state.error}))
}

fn main() {
    let mut context=tauri::generate_context!();
    if std::env::args().any(|argument|argument=="--verify-alpha-assets") {
        match frontend_assets::verify(&context) {
            Ok(())=>{println!("ALPHA_EMBEDDED_FRONTEND_VERIFIED");std::process::exit(0);},
            Err(code)=>{eprintln!("{code}");std::process::exit(1);},
        }
    }
    if startup_proof::enabled() {
        for window in &mut context.config_mut().app.windows {
            window.visible=false;window.focus=false;
            window.title="2gether hidden startup validation".into();
        }
    }
    tauri::Builder::default()
        .setup(move |app| {
            // Reserve authority before allocating credentials, native surfaces,
            // players or diagnostics. A second instance cannot start those
            // owners while the user's existing Windows host is still running.
            let address=if startup_proof::enabled() {"127.0.0.1:47822"} else {"127.0.0.1:47820"};
            let listener = std::net::TcpListener::bind(address)?;
            listener.set_nonblocking(true)?;
            // Isolated proofs retain ephemeral credentials; normal startup uses DPAPI.
            let server=if cfg!(debug_assertions) && std::env::var("TOGETHER_SESSION_PROOF").as_deref()==Ok("1") {
                Host::new(uuid::Uuid::new_v4().to_string(),std::env::var("TOGETHER_PHONE_TOKEN")?)?
            } else {
                let credentials=together_session_server::credentials::Credentials::load_or_create(&app.path().app_local_data_dir()?.join("pairing.dpapi"))?;
                Host::new(credentials.pc.to_string(),credentials.phone.to_string())?
            };
            server.configure_android_updates(app.path().app_local_data_dir()?.join("updates/android-release.json"))?;
            app.manage(server.clone());
            let window = app.get_webview_window("main").ok_or("Main window missing")?;
            let parent = window.hwnd()?.0;
            let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
            let title = [0u16];
            let child = unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::*;
                CreateWindowExW(0, class.as_ptr(), title.as_ptr(), WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS,
                    0, 0, 1, 1, parent as _, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null())
            };
            if child.is_null() { return Err("Native video surface creation failed".into()); }
            let root = runtime_assets::root()?;
            startup_proof::install_shutdown(app,&root);
            let local_vault=app.path().app_local_data_dir()?.join("torbox.dpapi");
            // Existing development credentials remain in place. Packages do not
            // copy them; normal installed configuration uses the PC's private vault.
            let vault=if cfg!(debug_assertions) && !local_vault.is_file() {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../.secrets/torbox.dpapi")
            } else {local_vault};
            app.manage(torbox::Controller::new(vault));
            let local_stremio=app.path().app_local_data_dir()?.join("stremio.dpapi");
            let stremio_vault=if cfg!(debug_assertions) && !local_stremio.is_file() {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../.secrets/stremio.dpapi")
            }else {local_stremio};
            app.manage(stremio::Controller::new(stremio_vault));
            let diagnostics_directory=if cfg!(debug_assertions) && std::env::var("TOGETHER_SESSION_PROOF").as_deref()==Ok("1") {
                root.join("work/native-journal-proof")
            } else {app.path().app_local_data_dir()?.join("diagnostics")};
            app.manage(player::PlayerController::start(root.clone(), child as usize, server.clone()));
            if cfg!(debug_assertions) && std::env::var("TOGETHER_SESSION_PROOF").as_deref()==Ok("1") {
                let proof=server.clone();let proof_root=root.clone();
                tauri::async_runtime::spawn(async move {
                    let _=proof.local_pc(None,together_session_server::Action::Load {
                        media:together_sync_core::MediaIdentity::fixture()
                    }).await;
                    // A private, ignored nonce for the automated initial-pairing proof only.
                    if let Ok(code)=proof.open_pairing().await {
                        let _=tokio::fs::write(proof_root.join("work/pairing-code-proof.txt"),code).await;
                    }
                });
            }
            // Hidden normal-mode acceptance exercises DPAPI credentials and the
            // real pairing endpoint. The nonce stays in ignored private work.
            if startup_proof::enabled() && std::env::var("TOGETHER_SESSION_PROOF").as_deref()!=Ok("1") {
                let pairing_host=server.clone();let pairing_root=root.clone();
                tauri::async_runtime::spawn(async move {
                    if let Ok(code)=pairing_host.open_pairing().await {
                        let _=tokio::fs::write(pairing_root.join("work/pairing-code-proof.txt"),code).await;
                    }
                });
            }
            // The control endpoint was reserved before any component allocation.
            // Journal I/O stays off playback/server workers; failures are visible in health.
            let _=server.enable_diagnostics(diagnostics_directory);
            let tunnel_host=server.clone();
            let server_task=tauri::async_runtime::spawn(async move {
                let result=match tokio::net::TcpListener::from_std(listener) {
                    Ok(listener)=>server.clone().serve(listener).await.map_err(|_|"session_server_failed"),
                    Err(_)=>Err("session_listener_failed"),
                };
                server.request_shutdown();
                if let Err(code)=result {eprintln!("{code}");}result
            });
            // Proofs must never launch a provisioned public tunnel. Absence of
            // dedicated configuration leaves local development fully available.
            let (tunnel,tunnel_setup_failed,public_origin)=if cfg!(debug_assertions) && std::env::var("TOGETHER_SESSION_PROOF").as_deref()==Ok("1") {
                (None,false,None)
            } else {
                let directory=app.path().app_local_data_dir()?.join("control-tunnel");
                match together_session_server::tunnel::TunnelDeployment::load(&directory)
                    .and_then(|deployment|deployment.map(|d|{let origin=d.public_origin();d.supervise(&tunnel_host).map(|t|(t,origin))}).transpose()) {
                    Ok(Some((tunnel,origin)))=>(Some(tunnel),false,Some(origin)),
                    Ok(None)=>(None,false,None),
                    Err(code)=>{eprintln!("{code}");(None,true,None)},
                }
            };
            let tunnel_status=tunnel.as_ref().map(|tunnel|tunnel.subscribe());
            app.manage(AppLifecycle {public_origin,temporary:Arc::new(tokio::sync::Mutex::new(TemporaryConnection {owner:None,origin:None,error:None})),temporary_executable:root.join("native/vendor/windows/cloudflared.exe"),temporary_runtime:app.path().app_local_data_dir()?.join("control-tunnel/temporary"),tunnel:tokio::sync::Mutex::new(tunnel),tunnel_status,tunnel_setup_failed,started:std::sync::atomic::AtomicBool::new(false),finished:std::sync::atomic::AtomicBool::new(false),server:tokio::sync::Mutex::new(Some(server_task))});
            // Bound temporary connector capture and observe exit even while the
            // user is watching, independent of UI polling and playback workers.
            let temporary_app=app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    let lifecycle=temporary_app.state::<AppLifecycle>();
                    if lifecycle.started.load(std::sync::atomic::Ordering::Acquire) {break;}
                    let state=lifecycle.temporary.clone().lock_owned().await;
                    if state.owner.is_none() {continue;}
                    let _=owned_transition::blocking(state,|state| {
                        if let Some(mut owner)=state.owner.take() {
                            // Preserve the previous fixed failure notice for a
                            // panicking capture without releasing state early.
                            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(||owner.origin())) {
                                Ok(Ok(origin))=>{state.origin=origin;state.owner=Some(owner);},
                                _=>{state.origin=None;state.error=Some("Temporary connection stopped. Start it again and use the new address.");},
                            }
                        }
                    }).await;
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![android_download::android_download_info,android_download::open_android_download,startup_proof::startup_proof_enabled,startup_proof::startup_frontend_ready,session_snapshot, player_status, player_intent, video_bounds,open_pairing,cancel_pairing,diagnostic_health,export_session_log,tunnel_status,pairing_connection,start_temporary_connection,stop_temporary_connection,torbox_videos,torbox_select,torbox_refresh,private_alpha_account_links,stremio_connect,stremio_browser,stremio_browse,stremio_options,stremio_select,stremio_copied_source,stremio_prepare,stremio_episodes,stremio_back,stremio_cancel_lookup])
        .build(context).expect("Together 2.0 startup").run(lifecycle_event);
}

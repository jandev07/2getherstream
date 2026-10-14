use tauri::Manager;

pub fn install_shutdown(app:&tauri::App,root:&std::path::Path) {
    if !enabled() {return;}
    let stop=root.join("work/startup-proof-stop.txt");let handle=app.handle().clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            if stop.is_file() {handle.exit(0);break;}
        }
    });
}

pub fn enabled()->bool {
    cfg!(debug_assertions) && std::env::var("TOGETHER_HIDDEN_ALPHA_PROOF").as_deref()==Ok("1")

}
#[tauri::command]
pub fn startup_proof_enabled()->bool {enabled()}
#[tauri::command]
pub async fn startup_frontend_ready(app:tauri::AppHandle,rendered:bool)->bool {
    if !enabled() || !rendered {return false;}
    let player=app.state::<crate::player::PlayerController>();
    let valid=player.status.lock().is_ok_and(|status|status.initialized && status.error.is_none());
    let host=app.state::<std::sync::Arc<together_session_server::Host>>();
    let state=host.snapshot().await;
    if !valid || !state.participants.get("pc").is_some_and(|pc|pc.connected) {return false;}
    println!("ALPHA_RENDERED_UI_AND_NATIVE_STARTUP_VERIFIED");true
}

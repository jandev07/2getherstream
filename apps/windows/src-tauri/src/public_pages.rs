// User-invoked public pages only. No caller-provided URI or session data.
const DOWNLOAD_PAGE: &str = super::android_download::DOWNLOAD_PAGE;
const SOURCE_NOTICES: &str = "https://github.com/jandev07/2getherstream/blob/main/THIRD-PARTY-NOTICES.md";

fn open(page: &'static str) -> Result<(), &'static str> {
    std::process::Command::new("explorer.exe").arg(page).spawn()
        .map(|_|()).map_err(|_|"Could not open the public project page")
}

#[tauri::command]
pub fn open_android_download() -> Result<(), &'static str> { open(DOWNLOAD_PAGE) }

#[tauri::command]
pub fn open_source_notices() -> Result<(), &'static str> { open(SOURCE_NOTICES) }

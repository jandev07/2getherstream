use qrcodegen::{QrCode, QrCodeEcc};
use serde::Serialize;

pub const DOWNLOAD_PAGE: &str = "https://2gether-download.vgovind28.workers.dev/";

#[derive(Serialize)]
pub struct DownloadInfo { pub url: &'static str, pub qr_svg: String }

#[tauri::command]
pub fn android_download_info() -> Result<DownloadInfo, &'static str> {
    let qr=QrCode::encode_text(DOWNLOAD_PAGE,QrCodeEcc::Medium).map_err(|_|"Could not prepare the Android download code")?;
    let size=qr.size()+8;
    let mut path=String::new();
    for y in 0..qr.size() {for x in 0..qr.size() {if qr.get_module(x,y) {
        use std::fmt::Write;
        let _=write!(path,"M{},{}h1v1h-1z",x+4,y+4);
    }}}
    Ok(DownloadInfo {url:DOWNLOAD_PAGE,qr_svg:format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {size} {size}\" shape-rendering=\"crispEdges\"><rect width=\"100%\" height=\"100%\" fill=\"white\"/><path d=\"{path}\" fill=\"black\"/></svg>")})
}


#[cfg(test)] mod tests {
    use super::*;
    #[test] fn download_code_has_fixed_public_target_and_quiet_zone() {
        let info=android_download_info().unwrap();
        assert_eq!(info.url,DOWNLOAD_PAGE);assert!(!info.url.contains('?'));
        let qr=QrCode::encode_text(info.url,QrCodeEcc::Medium).unwrap();
        assert!(info.qr_svg.contains(&format!("viewBox=\"0 0 {} {}\"",qr.size()+8,qr.size()+8)));
        assert!(info.qr_svg.contains("M4,4h1v1h-1z"));
        assert!(!info.qr_svg.contains("href"));assert!(!info.qr_svg.contains("script"));
        // Retain the generated symbol for independent decode/render inspection.
        let output=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../work/source-validation/android-download-qr.svg");
        std::fs::write(output,info.qr_svg).unwrap();
    }
}

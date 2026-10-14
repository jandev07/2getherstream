use tauri::{Context,Wry};

/// Check shipped assets before constructing a window, server or native player.
pub fn verify(context:&Context<Wry>)->Result<(), &'static str> {
    if !cfg!(feature="custom-protocol") {return Err("embedded_frontend_required");}
    if !matches!(context.config().build.frontend_dist,Some(tauri::utils::config::FrontendDist::Directory(_))) {
        return Err("frontend_directory_required");
    }
    let assets=context.assets();
    let index=assets.get(&"index.html".into()).ok_or("frontend_index_missing")?;
    let html=std::str::from_utf8(&index).map_err(|_|"frontend_index_invalid")?;
    if !html.contains("id=\"app\"") {return Err("frontend_mount_missing");}
    let mut scripts=0;
    for prefix in ["src=\"/assets/","href=\"/assets/"] {
        for suffix in html.split(prefix).skip(1) {
            let name=suffix.split('"').next().ok_or("frontend_asset_invalid")?;
            let key=format!("assets/{name}");
            let bytes=assets.get(&key.into()).ok_or("frontend_asset_missing")?;
            if bytes.is_empty() {return Err("frontend_asset_empty");}
            if prefix.starts_with("src") && name.ends_with(".js") {scripts+=1;}
        }
    }
    if scripts==0 {return Err("frontend_script_missing");}
    Ok(())
}

#[cfg(test)] mod tests {
    #[test] fn candidate_relative_path_is_directory_not_drive_url() {
        use tauri::utils::config::FrontendDist;
        let absolute:FrontendDist=serde_json::from_str(r#""C:/candidate/frontend""#).unwrap();
        assert!(matches!(absolute,FrontendDist::Url(_)));
        let relative:FrontendDist=serde_json::from_str(r#""../../../work/candidate/frontend""#).unwrap();
        assert!(matches!(relative,FrontendDist::Directory(_)));
    }
}

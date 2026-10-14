//! Local, non-deploying renderer. Arguments are paths/identity, never tokens.
use std::{io::Write,path::PathBuf};
fn main()->Result<(),&'static str> {
    let args:Vec<String>=std::env::args().skip(1).collect();
    if args.len()!=4 {return Err("expected hostname, tunnel UUID, credential path and new YAML output path");}
    let credentials=PathBuf::from(&args[2]);let output=PathBuf::from(&args[3]);
    if !output.is_absolute() || output==credentials
        || !output.extension().is_some_and(|s|s=="yml" || s=="yaml") {return Err("invalid_tunnel_output_path");}
    let config=together_session_server::tunnel::ControlConfig::new(&args[0],&args[1],credentials)?;
    let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(output).map_err(|_|"tunnel_config_create_failed")?;
    file.write_all(config.yaml()?.as_bytes()).map_err(|_|"tunnel_config_write_failed")?;
    file.sync_all().map_err(|_|"tunnel_config_flush_failed")?;
    Ok(())
}

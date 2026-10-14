//! Own an isolated control host and temporary connector for explicit WAN checks.
use together_session_server::{Host,tunnel::QuickTunnel};
#[tokio::main]
async fn main()->Result<(),Box<dyn std::error::Error>> {
    let root=std::path::PathBuf::from(std::env::args_os().nth(1).ok_or("runtime required")?);
    let executable=std::path::PathBuf::from(std::env::args_os().nth(2).ok_or("connector required")?);
    let listener=tokio::net::TcpListener::bind("127.0.0.1:47820").await?;
    let host=Host::new(uuid::Uuid::new_v4().to_string(),format!("{}{}",uuid::Uuid::new_v4(),uuid::Uuid::new_v4()))?;
    std::fs::create_dir_all(&root)?;
    let code=host.open_pairing().await?;std::fs::write(root.join("code.txt"),code)?;
    let server_host=host.clone();let server=tokio::spawn(async move {server_host.serve(listener).await});
    let mut tunnel=QuickTunnel::start(&executable,&root)?;
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(90);
    let mut announced=false;
    loop {
        if root.join("stop.txt").is_file(){break;}
        if let Some(origin)=tunnel.origin()? {
            if !announced {std::fs::write(root.join("origin.txt"),origin)?;println!("TEMPORARY_CONTROL_ADDRESS_READY");announced=true;}
        }
        if std::time::Instant::now()>deadline {break;}
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    tunnel.stop()?;host.request_shutdown();let _=server.await?;
    if !announced {return Err("temporary control address deadline".into());}
    println!("TEMPORARY_CONTROL_STOPPED");Ok(())
}

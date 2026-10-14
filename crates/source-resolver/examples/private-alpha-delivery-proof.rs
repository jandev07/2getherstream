//! User-authorized private Alpha PC-only bounded CDN check; prints no URL/key.
use together_source_resolver::{secret,TorBox};
#[tokio::main]
async fn main()->anyhow::Result<()> {
    let vault=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.secrets/torbox.dpapi");
    let client=TorBox::new_private_alpha(secret::read_key(&vault)?)?;
    let videos=client.available_videos().await?;
    let media=videos.first().ok_or_else(||anyhow::anyhow!("No available video for delivery check"))?;
    if std::env::args().any(|argument|argument=="--inspect-host-only") {
        println!("CDN hostname for ownership verification: {}",client.inspect_download_host(media).await?);
        println!("No CDN request, path, query, credential export or Android handoff");return Ok(());
    }
    let pair=client.device_streams(media,None,None).await?;
    anyhow::ensure!(pair.pc.identity()==pair.phone.identity(),"Private Alpha media identity mismatch");
    let (pc,phone)=tokio::try_join!(client.probe(&pair.pc),client.probe(&pair.phone))?;
    println!("Private Alpha delivery: matching identity, two separate requests, PC range={pc} bytes, phone-candidate range={phone} bytes");
    println!("PC-only check. Account-bearing exception enabled; no Android handoff or physical-device acceptance");Ok(())
}

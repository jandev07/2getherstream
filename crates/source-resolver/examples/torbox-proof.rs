//! Read-only live API/CDN proof. Prints counts only; never prints URLs or credentials.
use together_source_resolver::{secret, TorBox};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let api = TorBox::new(secret::read_key(&root.join(".secrets/torbox.dpapi"))?)?;
    let videos = api.available_videos().await?;
    println!("Available video files: {}", videos.len());
    let Some(media) = videos.first() else { println!("No available video to probe"); return Ok(()); };
    let streams = api.device_streams(media, None, None).await?;
    anyhow::ensure!(streams.pc.identity() == streams.phone.identity(), "device media identity mismatch");
    let pc = api.probe(&streams.pc).await?;
    let phone = api.probe(&streams.phone).await?;
    println!("Two independent link requests, same media identity; direct CDN range reads PC={pc}, phone={phone} bytes");
    Ok(())
}

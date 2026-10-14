//! Explicit PC-only bounded stream inspection; never prints source/response strings.
use together_source_resolver::{secret,TorBox};
#[tokio::main]
async fn main()->anyhow::Result<()> {
    let vault=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.secrets/torbox.dpapi");
    let client=TorBox::new(secret::read_key(&vault)?)?;
    let videos=client.available_videos().await?;
    let media=videos.iter().find(|media|media.filename.as_deref().is_some_and(|name|name.to_ascii_lowercase().ends_with(".mp4")))
        .ok_or_else(||anyhow::anyhow!("No available MP4 for stream inspection"))?;
    let report=client.inspect_stream_contract(media).await?;
    println!("{}",serde_json::to_string(&report)?);
    println!("Scrobbling disabled; no URL followed/exported, no phone handoff or transcoding playback");Ok(())
}

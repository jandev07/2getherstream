//! Read-only PC diagnostic. Does not follow redirects or deliver anything to Android.
use together_source_resolver::{secret, TorBox};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let api=TorBox::new(secret::read_key(&root.join(".secrets/torbox.dpapi"))?)?;
    let videos=api.available_videos().await?;
    let media=videos.first().ok_or_else(||anyhow::anyhow!("No available video to inspect"))?;
    let (status,shape)=api.inspect_download_redirect(media).await?;
    println!("Fixed API redirect inspection HTTP {status}; location classification {shape:?}");
    println!("Location not followed, exported or delivered; safe media capability remains unverified");
    Ok(())
}

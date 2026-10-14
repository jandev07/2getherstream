//! Read-only account proof. Prints counts only; no titles, auth keys or addon URLs.
use together_source_resolver::{secret,stremio::Stremio};
#[tokio::main]
async fn main()->anyhow::Result<()> {
    let vault=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.secrets/stremio.dpapi");
    let client=Stremio::from_pc_secret(secret::read_key(&vault)?).await?;
    let library=client.library().await?;
    let addons=client.addons().await?;
    println!("Read-only Stremio library: movies={}, series={}",library.iter().filter(|v|v.kind=="movie").count(),library.iter().filter(|v|v.kind=="series").count());
    println!("Configured HTTPS addons: {}, stream providers: {}, catalogs: {}",addons.len(),addons.iter().filter(|v|v.has_streams()).count(),addons.iter().map(|v|v.catalog_count()).sum::<usize>());
    if let Some(catalog)=client.catalogs(&addons).into_iter().find(|catalog|catalog.kind=="movie" && !catalog.search_required) {
        let titles=client.catalog(&addons[catalog.addon_index],catalog.catalog_index,None).await?;
        println!("First available movie catalog returned {} sanitized title records",titles.len());
        if std::env::args().any(|argument|argument=="--inspect-streams" || argument=="--inspect-cache") {
            let title=titles.first().ok_or_else(||anyhow::anyhow!("No catalog title for inspection"))?;
            let addon=addons.iter().find(|addon|addon.has_streams()).ok_or_else(||anyhow::anyhow!("No configured HTTPS stream provider"))?;
            let choices=client.streams(addon,&title.kind,&title.id).await?;
            let torbox_vault=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.secrets/torbox.dpapi");
            let torbox=together_source_resolver::TorBox::new(secret::read_key(&torbox_vault)?)?;
            let available=torbox.available_videos().await?;
            let matched=choices.iter().filter(|choice|choice.canonical_media(&available).is_ok()).count();
            println!("Stream descriptors: {}, canonical available TorBox matches: {matched}",choices.len());
            if std::env::args().any(|argument|argument=="--inspect-cache") {
                let cached=torbox.cached_selections(&choices).await?;
                println!("Exact provider-cached video choices: {}",cached.len());
            }
        }
    }
    if std::env::args().any(|argument|argument=="--inspect-series") {
        let catalog=client.catalogs(&addons).into_iter().find(|catalog|catalog.kind=="series" && !catalog.search_required)
            .ok_or_else(||anyhow::anyhow!("No configured series catalog"))?;
        let titles=client.catalog(&addons[catalog.addon_index],catalog.catalog_index,None).await?;
        println!("First series catalog returned {} sanitized title records",titles.len());
        let title=titles.first().ok_or_else(||anyhow::anyhow!("No series title for inspection"))?;
        let mut verified=Vec::new();
        for addon in addons.iter().filter(|addon|addon.supports_meta("series",&title.id)) {
            if let Ok(episodes)=client.episodes(addon,&title.id).await {if !episodes.is_empty() {verified=episodes;break;}}
        }
        anyhow::ensure!(!verified.is_empty(),"No configured addon returned usable episodes");
        println!("Explicit sanitized episode identities: {}",verified.len());
        let episode=&verified[0];
        let addon=addons.iter().find(|addon|addon.supports_stream("series",&episode.id)).ok_or_else(||anyhow::anyhow!("No configured episode stream provider"))?;
        let choices=client.streams(addon,"series",&episode.id).await?;
        let torbox_vault=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.secrets/torbox.dpapi");
        let torbox=together_source_resolver::TorBox::new(secret::read_key(&torbox_vault)?)?;
        let cached=torbox.cached_selections(&choices).await?;
        println!("First episode stream descriptors: {}, exact cached choices: {}",choices.len(),cached.len());
    }
    println!("No account writes, addon media URL follows or playback handoffs performed");Ok(())
}

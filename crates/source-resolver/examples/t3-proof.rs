//! Explicit PC-only read-only capability investigation; no URL/account data output.
//! Listing uses header auth and never leaves the PC. No application integration.
use anyhow::{ensure, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use hmac::{Hmac, Mac};
use reqwest::{header::{HeaderValue, AUTHORIZATION}, Client, Response, Url};
use serde::Deserialize;
use sha1::Sha1;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use together_source_resolver::{secret, t3::{presign_get, classify_redirect, ExperimentalGet}};
use zeroize::Zeroizing;

#[derive(Deserialize)]
struct Profile { success: bool, data: Option<User> }
#[derive(Deserialize)]
struct User { auth_id: String }
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Objects {
    #[serde(default)] contents: Vec<Object>,
    #[serde(default)] is_truncated: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Object { key: String, size: u64 }

async fn bounded_body(mut response: Response) -> Result<Zeroizing<Vec<u8>>> {
    let mut bytes = Zeroizing::new(Vec::new());
    while let Some(chunk) = response.chunk().await.map_err(|_| anyhow::anyhow!("Diagnostic response interrupted"))? {
        ensure!(bytes.len() + chunk.len() <= 2 * 1024 * 1024, "Diagnostic response too large");
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

async fn listing(client: &Client, id: &str, key: &str) -> Result<Option<Objects>> {
    // Fixed path/query only. Ordinary max-keys query is excluded by S3 V2.
    let date = httpdate::fmt_http_date(SystemTime::now());
    let text = b"GET\n\n\n";
    let mut mac = Hmac::<Sha1>::new_from_slice(key.as_bytes())
        .map_err(|_| anyhow::anyhow!("Diagnostic signing failed"))?;
    mac.update(text); mac.update(date.as_bytes()); mac.update(b"\n/torbox/");
    let auth = Zeroizing::new(format!("AWS {id}:{}", STANDARD.encode(mac.finalize().into_bytes())));
    let mut header = HeaderValue::from_str(&auth).map_err(|_| anyhow::anyhow!("Diagnostic identity invalid"))?;
    header.set_sensitive(true);
    let response = client.get("https://t3.nexus/torbox/?max-keys=50").header(AUTHORIZATION, header)
        .header("Date", date).send().await.map_err(|_| anyhow::anyhow!("T3 listing request failed"))?;
    println!("T3 fixed-bucket header-auth listing HTTP {}", response.status().as_u16());
    if !response.status().is_success() { return Ok(None); }
    let kind = response.headers().get("Content-Type").and_then(|v|v.to_str().ok()).map_or("missing", |v| {
        if v.contains("xml") { "xml" } else if v.contains("json") { "json" }
        else if v.contains("html") { "html" } else { "other" }
    });
    println!("T3 listing content type: {kind}");
    let bytes = bounded_body(response).await?;
    let mut objects: Objects = match quick_xml::de::from_reader(bytes.as_slice()) {
        Ok(objects) => objects,
        Err(_) => { println!("T3 listing XML decoding failed"); return Ok(None); }
    };
    println!("T3 listing XML decoded: {} entries; requested max-keys respected: {}", objects.contents.len(), objects.contents.len() <= 50);
    // Provider may ignore max-keys. Body remains hard bounded; inspect only first50.
    objects.contents.truncate(50);
    Ok(Some(objects))
}

async fn status_only(client: &Client, capability: &ExperimentalGet, url: Url, key: &str, label: &str) -> Result<u16> {
    let mut response = client.get(url).header("Range", "bytes=0-1023").header("Accept-Encoding", "identity")
        .send().await.map_err(|_| anyhow::anyhow!("T3 capability request failed"))?;
    let mut previous = None;
    for _ in 0..3 {
        if !response.status().is_redirection() { break; }
        let location = response.headers().get("Location").and_then(|v|v.to_str().ok())
            .ok_or_else(||anyhow::anyhow!("T3 redirect missing"))?;
        let next = previous.as_ref().unwrap_or(capability).same_origin_redirect(location, key)?;
        response = client.get(next.url().clone()).header("Range", "bytes=0-1023").header("Accept-Encoding", "identity")
            .send().await.map_err(|_|anyhow::anyhow!("T3 negative redirected request failed"))?;
        previous = Some(next);
    }
    let status = response.status().as_u16();
    println!("{label} HTTP {status}");
    // Only guarded same-origin redirects; no error document/Location/signature output.
    Ok(status)
}

async fn run() -> Result<()> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let key = secret::read_key(&root.join(".secrets/torbox.dpapi"))?;
    let client = Client::builder().redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(25)).connect_timeout(Duration::from_secs(10))
        .user_agent("Together2-T3ReadOnlyDiagnostic/0.1").build()
        .map_err(|_| anyhow::anyhow!("Diagnostic client initialization failed"))?;
    let response = client.get("https://api.torbox.app/v1/api/user/me?settings=false")
        .bearer_auth(key.as_str()).send().await.map_err(|_| anyhow::anyhow!("Profile request failed"))?;
    println!("TorBox read-only profile HTTP {}", response.status().as_u16());
    ensure!(response.status().is_success(), "Profile unavailable");
    let body = bounded_body(response).await?;
    let profile: Profile = serde_json::from_slice(&body).map_err(|_| anyhow::anyhow!("Profile format invalid"))?;
    ensure!(profile.success, "Profile request rejected");
    let id = Zeroizing::new(profile.data.ok_or_else(|| anyhow::anyhow!("Profile missing"))?.auth_id);
    let Some(objects) = listing(&client, &id, &key).await? else { return Ok(()); };
    println!("Objects inspected: {}; listing truncated: {}", objects.contents.len(), objects.is_truncated);
    let Some(object) = objects.contents.iter().find(|o| o.size >= 1024 && o.key.rsplit('.').next()
        .is_some_and(|ext| ["mkv", "mp4", "webm", "mov", "m4v", "ts"].contains(&ext.to_ascii_lowercase().as_str())))
        else { println!("No video candidate in bounded untagged listing"); return Ok(()); };
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| anyhow::anyhow!("PC wall clock invalid"))?.as_secs();
    let signed = presign_get(&id, &key, "torbox", &object.key, now, Duration::from_secs(300))?;
    let mut redirected = None;
    let mut response = client.get(signed.url().clone()).header("Range", "bytes=0-1023").header("Accept-Encoding", "identity")
        .send().await.map_err(|_| anyhow::anyhow!("T3 range request failed"))?;
    println!("T3 presigned candidate range HTTP {}", response.status().as_u16());
    for hop in 1..=3 {
        if !response.status().is_redirection() { break; }
        let Some(location) = response.headers().get("Location").and_then(|v| v.to_str().ok())
            else { println!("T3 redirect Location missing or invalid"); return Ok(()); };
        println!("T3 raw redirect classification: {:?}", classify_redirect(location, &key));
        let previous = redirected.as_ref().unwrap_or(&signed);
        let next = match previous.same_origin_redirect(location, &key) {
            Ok(next) => next,
            Err(_) => { println!("T3 redirect rejected by credential/origin guard; NOT followed"); return Ok(()); }
        };
        response = client.get(next.url().clone()).header("Range", "bytes=0-1023").header("Accept-Encoding", "identity")
            .send().await.map_err(|_| anyhow::anyhow!("T3 redirected range request failed"))?;
        println!("T3 guarded same-origin redirect hop {hop} HTTP {}", response.status().as_u16());
        redirected = Some(next);
    }
    if response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        println!("Safe direct capability NOT established; rejection/expiry tests not reached");
        return Ok(());
    }
    let expected = format!("bytes 0-1023/{}", object.size);
    let range = response.headers().get("Content-Range").and_then(|v|v.to_str().ok());
    println!("T3 range header count: {}; exact listing-size range match: {}", response.headers().get_all("Content-Range").iter().count(), range == Some(expected.as_str()));
    println!("T3 range span matches: {}; total matches listing: {}", range.is_some_and(|v|v.starts_with("bytes 0-1023/")),
        range.and_then(|v|v.rsplit('/').next()).and_then(|v|v.parse::<u64>().ok()) == Some(object.size));
    // Print parsed offsets only, never arbitrary provider header text.
    if let Some((start, end)) = range.and_then(|v|v.strip_prefix("bytes ")).and_then(|v|v.split_once('/'))
        .and_then(|(span,_)|span.split_once('-')).and_then(|(a,b)|Some((a.parse::<u64>().ok()?,b.parse::<u64>().ok()?))) {
        println!("T3 returned numeric range offsets: {start}-{end}");
    }
    let valid_metadata = response.headers().get_all("Content-Range").iter().count() == 1
        && range == Some(expected.as_str())
        && response.headers().get_all("Content-Encoding").iter().count() <= 1
        && response.headers().get("Content-Encoding").map_or(true, |v|v.as_bytes().eq_ignore_ascii_case(b"identity"));
    // Separate 1024-byte body bound rather than the larger metadata helper.
    let mut length = 0;
    if valid_metadata {
        while let Some(chunk) = response.chunk().await.map_err(|_| anyhow::anyhow!("T3 range interrupted"))? {
            length += chunk.len(); ensure!(length <= 1024, "T3 range body exceeded bound");
        }
        ensure!(length == 1024, "T3 range incomplete");
    } else {
        println!("T3 range metadata rejected; original media delivery not accepted");
        let kind = response.headers().get("Content-Type").and_then(|v|v.to_str().ok()).map_or("missing", |v| {
            if v.contains("json") { "json" } else if v.contains("xml") { "xml" }
            else if v.contains("html") { "html" } else { "other" }
        });
        println!("T3 rejected range content type: {kind}");
        // If explicitly JSON, classify a tiny provider error response without
        // printing/storing its contents. Never parse arbitrary movie bytes.
        if kind == "json" {
            let mut bytes = Zeroizing::new(Vec::new());
            while let Some(chunk) = response.chunk().await.map_err(|_|anyhow::anyhow!("T3 error response interrupted"))? {
                ensure!(bytes.len() + chunk.len() <= 1024, "T3 error response too large");
                bytes.extend_from_slice(&chunk);
            }
            let rejected = serde_json::from_slice::<serde_json::Value>(&bytes).ok()
                .and_then(|v|v.get("success").and_then(|s|s.as_bool())).map(|v|!v);
            println!("T3 JSON declares request failure: {}", rejected == Some(true));
        }
    }
    drop(response);
    println!("T3 validated direct range bytes: {length}; canonical torrent/file identity UNVERIFIED");
    let expired = presign_get(&id, &key, "torbox", &object.key, now.saturating_sub(600), Duration::from_secs(30))?;
    let expired_status = status_only(&client, &expired, expired.url().clone(), &key, "Expired capability").await?;
    let mut tampered = signed.url().clone();
    tampered.set_query(None);
    tampered.query_pairs_mut().append_pair("AWSAccessKeyId", &id).append_pair("Expires", &signed.expires_unix_seconds().to_string())
        .append_pair("Signature", "AAAAAAAAAAAAAAAAAAAAAAAAAAA=");
    let tampered_status = status_only(&client, &signed, tampered, &key, "Tampered signature").await?;
    ensure!(expired_status == 403 && tampered_status == 403, "T3 negative capability checks unproven");
    ensure!(valid_metadata, "T3 range metadata invalid");
    println!("Capability rejection checks passed; torrent/file identity and Android delivery remain UNVERIFIED");
    Ok(())
}

#[tokio::main]
async fn main() {
    if run().await.is_err() {
        // Don't print even an error chain: future dependencies may retain request data.
        eprintln!("T3 diagnostic stopped; no application media delivery enabled");
        std::process::exit(1);
    }
}

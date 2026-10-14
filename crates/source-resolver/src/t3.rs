//! Experimental PC-only T3 GET signing, independently written from the S3 V2 spec.
//! Provider support and canonical torrent/file mapping are NOT established.
//! Nothing here creates a StreamLease or changes application delivery.
use anyhow::{ensure, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use hmac::{Hmac, Mac};
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use reqwest::Url;
use sha1::Sha1;
use std::time::Duration;
use zeroize::Zeroizing;

const UNRESERVED: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-').remove(b'.').remove(b'_').remove(b'~');
const ORIGIN: &str = "https://t3.nexus";

/// Sanitized diagnostic classification only; never follows or returns a redirect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedirectSafety { CredentialFree, AccountCredential, InsecureTransport, RelativeOrInvalid, Rejected }
pub fn classify_redirect(raw: &str, secret: &str) -> RedirectSafety {
    let Ok(url) = Url::parse(raw) else { return RedirectSafety::RelativeOrInvalid; };
    if url.scheme() != "https" { return RedirectSafety::InsecureTransport; }
    match super::validate_stream_url(raw, secret) {
        Ok(_) => RedirectSafety::CredentialFree,
        Err(error) if error.to_string() == "TorBox API credential leaked into media URL" => RedirectSafety::AccountCredential,
        Err(_) => RedirectSafety::Rejected,
    }
}

/// Bearer-sensitive, deliberately no Debug/Serialize. Keep on PC until verified.
pub struct ExperimentalGet {
    url: Url,
    expires_unix_seconds: u64,
}
impl ExperimentalGet {
    pub fn url(&self) -> &Url { &self.url }
    pub fn expires_unix_seconds(&self) -> u64 { self.expires_unix_seconds }
    /// Diagnostic-only bounded redirect handling: resolve relative Location, then
    /// reapply the guard. Only the fixed T3 origin is permitted at this stage.
    pub fn same_origin_redirect(&self, location: &str, secret: &str) -> Result<Self> {
        ensure!(location.len() <= 8192, "T3 redirect too large");
        let resolved = self.url.join(location).map_err(|_| anyhow::anyhow!("T3 redirect invalid"))?;
        let url = super::validate_stream_url(resolved.as_str(), secret)?;
        ensure!(url.origin() == self.url.origin(), "T3 external redirect unverified");
        Ok(Self { url, expires_unix_seconds: self.expires_unix_seconds })
    }
}

/// Sign one decoded object key, never a listing, subresource or arbitrary URL.
/// Auth ID is the provider's access-key ID, NOT the TorBox API secret.
/// `now` is Unix wall time (S3 expiry), unrelated to session monotonic clocks.
/// Range is unsigned by S3 V2 and may vary; method/path/expiry are signed.
pub fn presign_get(
    auth_id: &str, secret: &str, bucket: &str, object: &str,
    now_unix_seconds: u64, lifetime: Duration,
) -> Result<ExperimentalGet> {
    ensure!(!secret.is_empty(), "T3 signing credential is empty");
    ensure!(!auth_id.is_empty() && auth_id.len() <= 256
        && auth_id.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.@".contains(&b)),
        "T3 access identity invalid");
    ensure!(lifetime.subsec_nanos() == 0 && (30..=900).contains(&lifetime.as_secs()),
        "T3 capability lifetime invalid");
    let expires = now_unix_seconds.checked_add(lifetime.as_secs())
        .ok_or_else(|| anyhow::anyhow!("T3 capability expiry overflow"))?;
    // A single strict bucket segment; tags outside this subset need later review.
    ensure!(!bucket.is_empty() && bucket.len() <= 63
        && bucket.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        && bucket != "." && bucket != "..", "T3 bucket invalid");
    ensure!(!object.is_empty() && object.len() <= 4096
        && !object.chars().any(|c| c.is_control() || c == '\\'), "T3 object invalid");
    // Accept decoded keys only. Reject URL-normalized paths instead of guessing.
    // Literal '%' is encoded, so an apparent %2e is a literal object name.
    let mut resource = format!("/{bucket}");
    for segment in object.split('/') {
        ensure!(!segment.is_empty() && segment != "." && segment != "..",
            "T3 object path invalid");
        resource.push('/');
        resource.push_str(&utf8_percent_encode(segment, UNRESERVED).to_string());
    }
    let mut url = Url::parse(&format!("{ORIGIN}{resource}"))
        .map_err(|_| anyhow::anyhow!("T3 capability construction failed"))?;
    ensure!(url.path() == resource, "T3 canonical path changed");
    let text = Zeroizing::new(format!("GET\n\n\n{expires}\n{resource}"));
    let signature = Zeroizing::new(sign(secret.as_bytes(), text.as_bytes())?);
    url.query_pairs_mut().append_pair("AWSAccessKeyId", auth_id)
        .append_pair("Expires", &expires.to_string()).append_pair("Signature", &signature);
    // Retain the existing repeated-decoding account-key/HTTPS validation.
    let url = super::validate_stream_url(url.as_str(), secret)?;
    Ok(ExperimentalGet { url, expires_unix_seconds: expires })
}

fn sign(secret: &[u8], text: &[u8]) -> Result<String> {
    let mut mac = Hmac::<Sha1>::new_from_slice(secret)
        .map_err(|_| anyhow::anyhow!("T3 signing failed"))?;
    mac.update(text);
    Ok(STANDARD.encode(mac.finalize().into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    const KEY: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
    fn get(bucket: &str, object: &str, now: u64, seconds: u64) -> Result<ExperimentalGet> {
        presign_get("AKIAIOSFODNN7EXAMPLE", KEY, bucket, object, now, Duration::from_secs(seconds))
    }
    fn signature(get: &ExperimentalGet) -> String {
        get.url.query_pairs().find(|(key, _)| key == "Signature").unwrap().1.into_owned()
    }
    #[test]
    fn rfc2202_hmac_sha1_vector() {
        // RFC 2202 section 3, test case 1, public synthetic input.
        assert_eq!(sign(&[0x0b;20], b"Hi There").unwrap(), "thcxhlUFcmTii8C2+zeMjvFGvgA=");
    }
    #[test]
    fn complete_get_matches_independent_python_reference() {
        // Python stdlib hmac/hashlib/base64; exact decoded AWS documentation input.
        // The published page's renamed bucket does not match its printed signature.
        let signed = get("awsexamplebucket1", "photos/puppy.jpg", 1175139320, 300).unwrap();
        assert_eq!(signed.expires_unix_seconds(), 1175139620);
        assert_eq!(signature(&signed), "1No4mq5ETf02z8aet9voy6gui6E=");
        assert_eq!(signed.url.path(), "/awsexamplebucket1/photos/puppy.jpg");
        let pairs: Vec<_> = signed.url.query_pairs().collect();
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0].0, "AWSAccessKeyId");
        assert_eq!(pairs[1].0, "Expires");
        assert_eq!(signed.url.host_str(), Some("t3.nexus"));
        assert!(!signed.url.as_str().contains(KEY));
    }
    #[test]
    fn encoded_path_is_exact_and_query_cannot_escape() {
        let signed = get("torbox", "folder/a +?#%2e/日本.mkv", 1000, 300).unwrap();
        assert_eq!(signed.url.path(), "/torbox/folder/a%20%2B%3F%23%252e/%E6%97%A5%E6%9C%AC.mkv");
        assert_eq!(signed.url.query_pairs().count(), 3);
        let expected = sign(KEY.as_bytes(), format!("GET\n\n\n1300\n{}", signed.url.path()).as_bytes()).unwrap();
        assert_eq!(signature(&signed), expected);
        assert!(signed.url.fragment().is_none());
    }
    #[test]
    fn ambiguous_paths_and_invalid_expiry_rejected() {
        for object in ["", "/movie", "movie/", "a//b", ".", "..", "a/../b", "a/./b", "a\\b", "a\nb", "a\0b"] {
            assert!(get("torbox", object, 1000, 300).is_err());
        }
        for bucket in ["", ".", "..", "a/b", "a?acl", "a\\b", "a\nb"] {
            assert!(get(bucket, "movie", 1000, 300).is_err());
        }
        for seconds in [0, 29, 901, u64::MAX] {
            assert!(get("torbox", "movie", 1000, seconds).is_err());
        }
        assert!(get("torbox", "movie", u64::MAX, 30).is_err());
        assert!(presign_get("id", KEY, "torbox", "movie", 1000, Duration::from_millis(30001)).is_err());
        assert!(presign_get("id&injected=1", KEY, "torbox", "movie", 1000, Duration::from_secs(30)).is_err());
    }
    #[test]
    fn capability_changes_with_path_expiry_and_secret() {
        let original = get("torbox", "movie", 1000, 300).unwrap();
        for changed in [get("other", "movie", 1000, 300).unwrap(), get("torbox", "other", 1000, 300).unwrap(), get("torbox", "movie", 1001, 300).unwrap(),
            presign_get("id", "different-synthetic-secret", "torbox", "movie", 1000, Duration::from_secs(300)).unwrap()] {
            assert_ne!(signature(&original), signature(&changed));
        }
        assert_ne!(signature(&original), sign(KEY.as_bytes(), b"PUT\n\n\n1300\n/torbox/movie").unwrap());
    }
    #[test]
    fn existing_account_key_guard_cannot_be_bypassed() {
        for object in [KEY.to_string(), percent_encoding::utf8_percent_encode(KEY, NON_ALPHANUMERIC).to_string()] {
            let error = get("torbox", &object, 1000, 300).err().unwrap().to_string();
            assert!(!error.contains(KEY));
        }
        assert!(presign_get(KEY, KEY, "torbox", "movie", 1000, Duration::from_secs(300)).is_err());
    }
    #[test]
    fn redirect_classification_never_returns_sensitive_location() {
        assert_eq!(classify_redirect("https://cdn.invalid/movie?token=synthetic-secret", "synthetic-secret"), RedirectSafety::AccountCredential);
        assert_eq!(classify_redirect("https://cdn.invalid/movie?token=%73ynthetic-secret", "synthetic-secret"), RedirectSafety::AccountCredential);
        assert_eq!(classify_redirect("http://cdn.invalid/movie", "synthetic-secret"), RedirectSafety::InsecureTransport);
        assert_eq!(classify_redirect("/movie", "synthetic-secret"), RedirectSafety::RelativeOrInvalid);
        assert_eq!(classify_redirect("https://cdn.invalid/movie?signature=opaque", "synthetic-secret"), RedirectSafety::CredentialFree);
        let signed = get("torbox", "movie", 1000, 300).unwrap();
        assert!(signed.same_origin_redirect("/torbox/other?signature=opaque", KEY).is_ok());
        assert!(signed.same_origin_redirect("//cdn.invalid/movie", KEY).is_err());
        assert!(signed.same_origin_redirect("http://t3.nexus/movie", KEY).is_err());
        assert!(signed.same_origin_redirect(&format!("/movie?token={KEY}"), KEY).is_err());
    }
}

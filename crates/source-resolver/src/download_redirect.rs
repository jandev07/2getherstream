//! PC-only inspection of the documented requestdl redirect option.
//! https://api-docs.torbox.app/ (Request Download Link, redirect parameter).
//! Never follows Location, reads a media body, returns a URL or creates a lease.
use super::*;
use reqwest::header::{HeaderMap, LOCATION};

#[derive(Debug, PartialEq, Eq)]
pub enum RedirectShape {
    NotRedirect,
    MissingOrAmbiguousLocation,
    RejectedLocation(RedirectRejection),
    CredentialFreeHttpsLocation,
}

/// Fixed categories only: never carries Location, account credentials or errors.
#[derive(Debug, PartialEq, Eq)]
pub enum RedirectRejection {
    InvalidHeader,
    InvalidOrRelativeUrl,
    ApiOrigin,
    AccountCredential,
    UnsafeUrl,
    NestedEncoding,
}

fn shape(status: reqwest::StatusCode, headers: &HeaderMap, key: &str) -> RedirectShape {
    if !status.is_redirection() { return RedirectShape::NotRedirect; }
    if headers.get_all(LOCATION).iter().count() != 1 {
        return RedirectShape::MissingOrAmbiguousLocation;
    }
    let Some(raw) = headers.get(LOCATION).and_then(|h| h.to_str().ok()) else {
        return RedirectShape::RejectedLocation(RedirectRejection::InvalidHeader);
    };
    // Deliberately require an absolute URL and retain all existing account-key
    // guards. Passing this classification proves neither origin nor file scope.
    match validate_stream_url(raw, key) {
        Ok(_)=>RedirectShape::CredentialFreeHttpsLocation,
        Err(error)=>{
            // Map the validator's fixed messages, never expose error text. The
            // API-origin guard shares its credential message; distinguish it
            // using a local parse, with no request to the resulting origin.
            let reason=match error.to_string().as_str() {
                "Invalid media CDN URL"=>RedirectRejection::InvalidOrRelativeUrl,
                "TorBox API credential leaked into media URL"=>{
                    if Url::parse(raw).ok().is_some_and(|url|is_torbox_api_origin(&url)) {
                        RedirectRejection::ApiOrigin
                    } else {RedirectRejection::AccountCredential}
                },
                "Unsafe nested media CDN URL encoding"=>RedirectRejection::NestedEncoding,
                _=>RedirectRejection::UnsafeUrl,
            };
            RedirectShape::RejectedLocation(reason)
        }
    }
}

impl TorBox {
    /// Send credentials only to the fixed TorBox API. Automatic redirects remain
    /// disabled by TorBox::new; no returned CDN URL is requested or exposed.
    pub async fn inspect_download_redirect(&self, media: &MediaIdentity) -> Result<(u16, RedirectShape)> {
        ensure!(media.provider == MediaProvider::Torbox, "unsupported media provider");
        media.validate().map_err(anyhow::Error::msg)?;
        let mut query = vec![("token", self.key.to_string()),
            ("torrent_id", media.torrent_id.to_string()), ("file_id", media.file_id.to_string()),
            ("zip_link", "false".into()), ("redirect", "true".into())];
        let request = self.client.get("https://api.torbox.app/v1/api/torrents/requestdl").query(&query);
        for (_, value) in &mut query { use zeroize::Zeroize; value.zeroize(); }
        let response = request.send().await.map_err(|_| anyhow::anyhow!("TorBox redirect inspection failed"))?;
        Ok((response.status().as_u16(), shape(response.status(), response.headers(), &self.key)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::{header::HeaderValue, StatusCode};
    #[test]
    fn inspection_rejects_account_api_relative_and_insecure_locations() {
        let mut headers=HeaderMap::new();
        for (raw,reason) in [("https://cdn.invalid/file?token=synthetic-secret",RedirectRejection::AccountCredential),
            ("https://cdn.invalid/file?token=%73ynthetic-secret",RedirectRejection::AccountCredential),
            ("https://api.torbox.app/file?opaque=capability",RedirectRejection::ApiOrigin),
            ("https://API.TORBOX.APP./file?opaque=capability",RedirectRejection::ApiOrigin),
            ("/file?opaque=capability",RedirectRejection::InvalidOrRelativeUrl),
            ("http://cdn.invalid/file",RedirectRejection::UnsafeUrl),
            ("https://user:password@cdn.invalid/file",RedirectRejection::UnsafeUrl)] {
            headers.insert(LOCATION,HeaderValue::from_str(raw).unwrap());
            let result=shape(StatusCode::FOUND,&headers,"synthetic-secret");
            assert_eq!(result,RedirectShape::RejectedLocation(reason));
            let report=format!("{result:?}");
            for private in ["synthetic-secret","opaque","capability","cdn.invalid","password"] {
                assert!(!report.contains(private));
            }
        }
        headers.insert(LOCATION,HeaderValue::from_static("https://cdn.invalid/file?opaque=capability"));
        assert_eq!(shape(StatusCode::FOUND,&headers,"synthetic-secret"),RedirectShape::CredentialFreeHttpsLocation);
        assert_eq!(shape(StatusCode::OK,&headers,"synthetic-secret"),RedirectShape::NotRedirect);
    }
    #[test]
    fn opaque_headers_and_nested_encoding_have_fixed_rejection_reports() {
        let mut headers=HeaderMap::new();
        headers.insert(LOCATION,HeaderValue::from_bytes(&[0x80]).unwrap());
        assert_eq!(shape(StatusCode::FOUND,&headers,"synthetic-secret"),
            RedirectShape::RejectedLocation(RedirectRejection::InvalidHeader));
        headers.insert(LOCATION,HeaderValue::from_static("https://cdn.invalid/file?opaque=%252525252541"));
        assert_eq!(shape(StatusCode::FOUND,&headers,"synthetic-secret"),
            RedirectShape::RejectedLocation(RedirectRejection::NestedEncoding));
    }
    #[test]
    fn inspection_requires_exactly_one_location() {
        let mut headers=HeaderMap::new();
        assert_eq!(shape(StatusCode::TEMPORARY_REDIRECT,&headers,"synthetic-secret"),RedirectShape::MissingOrAmbiguousLocation);
        headers.append(LOCATION,HeaderValue::from_static("https://cdn.invalid/a"));
        headers.append(LOCATION,HeaderValue::from_static("https://cdn.invalid/b"));
        assert_eq!(shape(StatusCode::TEMPORARY_REDIRECT,&headers,"synthetic-secret"),RedirectShape::MissingOrAmbiguousLocation);
    }
}

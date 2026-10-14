//! Native boundary defense. The PC resolver must separately reject account keys.
pub(crate) fn validate(raw: &str) -> anyhow::Result<()> {
    let rejected = || anyhow::anyhow!("Invalid media HTTPS URL");
    if raw.len() > 8192 || !raw.starts_with("https://") || raw.contains('\\') ||
        raw.chars().any(|ch| ch.is_control() || ch.is_whitespace()) {
        return Err(rejected());
    }
    // URL parsing can normalize an empty userinfo delimiter or authority away.
    // Reject those raw forms too, matching Android's URI checks.
    let authority = raw[8..].split(['/', '?', '#']).next().unwrap_or("");
    if authority.is_empty() || authority.contains('@') { return Err(rejected()); }
    let parsed = url::Url::parse(raw).map_err(|_| rejected())?;
    if parsed.scheme() != "https" || parsed.host_str().is_none() ||
        !parsed.username().is_empty() || parsed.password().is_some() ||
        parsed.fragment().is_some() || parsed.host_str().is_some_and(|host|
            host.trim_end_matches('.').eq_ignore_ascii_case("api.torbox.app")) {
        return Err(rejected());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn safe_capabilities_and_encoded_paths_are_accepted() {
        for raw in ["https://cdn.invalid/movie.mp4?cap=file-token", "https://cdn.invalid:8443/a%20b.mp4", "https://[::1]/fixture.mp4",
            "https://cdn.invalid./file?cap=opaque", "https://api.torbox.app.cdn.invalid/file?cap=opaque"] {
            assert!(validate(raw).is_ok());
        }
    }
    #[test]
    fn unsafe_or_malformed_sources_never_expose_input_in_errors() {
        for raw in ["http://cdn.invalid/private-secret", "https://", "https:///private-secret",
            "https://user:private-secret@cdn.invalid/movie", "https://@cdn.invalid/movie",
            "https://cdn.invalid/movie#private-secret", "https://API.TORBOX.APP/v1?token=private-secret",
            "https://api.torbox.app./v1?token=private-secret", "https://API.TORBOX.APP.:8443/private-secret",
            "https://%61pi.torbox.app./private-secret", "https://api.torbox.app../private-secret",
            "https://cdn.invalid/\nprivate-secret", "https://cdn.invalid/\0private-secret",
            "https://cdn.invalid/private secret", "https://cdn.invalid\\private-secret", "https://cdn.invalid:invalid/private-secret"] {
            let error = validate(raw).unwrap_err().to_string();
            assert_eq!(error, "Invalid media HTTPS URL");
            assert!(!error.contains("private-secret"));
        }
        assert!(validate(&format!("https://cdn.invalid/{}", "a".repeat(8192))).is_err());
    }
}

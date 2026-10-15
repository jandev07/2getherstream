//! Explicit source selection for private provider proofs, never product policy.
use together_source_resolver::MediaIdentity;

pub struct Selection(Option<(u64, u64)>);

impl Selection {
    pub fn from_values(torrent: Option<&str>, file: Option<&str>) -> Result<Self, &'static str> {
        fn number(value: &str) -> Result<u64, &'static str> {
            if value.is_empty() || value.len() > 20 || !value.bytes().all(|b| b.is_ascii_digit()) {
                return Err("proof_media_selection_invalid");
            }
            value.parse().map_err(|_| "proof_media_selection_invalid")
        }
        match (torrent, file) {
            (None, None) => Ok(Self(None)),
            (Some(torrent), Some(file)) => {
                let torrent = number(torrent)?;
                let file = number(file)?;
                if torrent == 0 { return Err("proof_media_selection_invalid"); }
                Ok(Self(Some((torrent, file))))
            }
            _ => Err("proof_media_selection_invalid"),
        }
    }

    pub fn from_environment() -> Result<Self, &'static str> {
        fn value(name: &str) -> Result<Option<String>, &'static str> {
            match std::env::var(name) {
                Ok(value) => Ok(Some(value)),
                Err(std::env::VarError::NotPresent) => Ok(None),
                Err(_) => Err("proof_media_selection_invalid"),
            }
        }
        let torrent = value("TOGETHER_PROOF_TORRENT_ID")?;
        let file = value("TOGETHER_PROOF_FILE_ID")?;
        Self::from_values(torrent.as_deref(), file.as_deref())
    }

    pub fn mode(&self) -> &'static str {
        if self.0.is_some() { "explicit" } else { "first_ready_legacy" }
    }

    pub fn select<'a>(&self, videos: &'a [MediaIdentity]) -> Result<&'a MediaIdentity, &'static str> {
        match self.0 {
            Some((torrent, file)) => videos.iter().find(|media| media.torrent_id == torrent && media.file_id == file)
                .ok_or("proof_media_selection_unavailable"),
            None => videos.first().ok_or("native_provider_video_unavailable"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn media(torrent_id: u64, file_id: u64) -> MediaIdentity {
        MediaIdentity::torbox(torrent_id, file_id)
    }

    #[test]
    fn explicit_selection_survives_inventory_reordering() {
        let selection = Selection::from_values(Some("22"), Some("0")).unwrap();
        assert_eq!(selection.mode(), "explicit");
        let videos = [media(11, 0), media(22, 0), media(22, 1)];
        assert_eq!(selection.select(&videos).unwrap().torrent_id, 22);
        assert_eq!(selection.select(&[videos[2].clone(), videos[1].clone(), videos[0].clone()]).unwrap().file_id, 0);
    }

    #[test]
    fn absent_explicit_file_never_falls_back_to_first() {
        let selection = Selection::from_values(Some("22"), Some("1")).unwrap();
        assert_eq!(selection.select(&[media(22, 0)]).err(), Some("proof_media_selection_unavailable"));
        assert_eq!(selection.select(&[]).err(), Some("proof_media_selection_unavailable"));
    }

    #[test]
    fn legacy_first_ready_is_labelled() {
        let selection = Selection::from_values(None, None).unwrap();
        assert_eq!(selection.mode(), "first_ready_legacy");
        assert_eq!(selection.select(&[media(11, 0), media(22, 0)]).unwrap().torrent_id, 11);
        assert_eq!(selection.select(&[]).err(), Some("native_provider_video_unavailable"));
    }

    #[test]
    fn malformed_or_partial_selection_fails_without_echoing_input() {
        for (torrent, file) in [(Some("22"), None), (None, Some("0")), (Some("0"), Some("0")),
            (Some("-1"), Some("0")), (Some("22"), Some("+1")), (Some(" 22"), Some("0")),
            (Some("22"), Some("1.0")), (Some(""), Some("0")), (Some("18446744073709551616"), Some("0"))] {
            assert_eq!(Selection::from_values(torrent, file).err(), Some("proof_media_selection_invalid"));
        }
    }

    #[test]
    fn exact_unsigned_boundaries_are_preserved() {
        let selection = Selection::from_values(Some("18446744073709551615"), Some("18446744073709551615")).unwrap();
        let videos = [media(u64::MAX, u64::MAX)];
        assert_eq!(selection.select(&videos).unwrap().file_id, u64::MAX);
    }
}

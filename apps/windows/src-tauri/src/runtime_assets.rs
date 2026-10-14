//! Resolve packaged native assets independently of the development checkout.
use std::path::{Path, PathBuf};

fn has_runtime(root: &Path) -> bool {
    root.join("native/vendor/windows/libmpv-2.dll").is_file()
}

fn choose(executable: &Path, development: Option<&Path>) -> Result<PathBuf, &'static str> {
    // The package owns its runtime. Never silently substitute a checkout DLL
    // for a missing DLL in a release package.
    let packaged = executable.parent().ok_or("Application folder unavailable")?;
    if has_runtime(packaged) { return Ok(packaged.to_path_buf()); }
    if let Some(root) = development.filter(|root| has_runtime(root)) {
        return Ok(root.to_path_buf());
    }
    Err("Native playback runtime missing from the application folder")
}

pub fn root() -> Result<PathBuf, &'static str> {
    let executable = std::env::current_exe().map_err(|_| "Application folder unavailable")?;
    let development = if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().ok()
    } else { None };
    choose(&executable, development.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("together-assets-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir(&root).unwrap(); Self(root)
        }
        fn runtime(&self) {
            let native = self.0.join("native/vendor/windows");
            std::fs::create_dir_all(&native).unwrap();
            std::fs::write(native.join("libmpv-2.dll"), b"synthetic-path-test").unwrap();
        }
    }
    impl Drop for Directory { fn drop(&mut self) { std::fs::remove_dir_all(&self.0).unwrap(); } }
    #[test] fn moved_package_uses_its_own_runtime_before_development() {
        let package = Directory::new(); let checkout = Directory::new();
        package.runtime(); checkout.runtime();
        assert_eq!(choose(&package.0.join("2gether.exe"), Some(&checkout.0)).unwrap(), package.0);
    }
    #[test] fn incomplete_release_package_fails_without_checkout_fallback() {
        let package = Directory::new(); let checkout = Directory::new(); checkout.runtime();
        assert!(choose(&package.0.join("2gether.exe"), None).is_err());
        assert_eq!(choose(&package.0.join("2gether.exe"), Some(&checkout.0)).unwrap(), checkout.0);
    }
}

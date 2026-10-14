//! Reads the existing PowerShell ConvertFrom-SecureString DPAPI format.
use anyhow::{bail, ensure, Result};
use std::path::Path;
use zeroize::{Zeroize, Zeroizing};

#[cfg(windows)]
pub fn read_key(path: &Path) -> Result<Zeroizing<String>> {
    use windows_sys::Win32::{Foundation::LocalFree, Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN}};
    let encoded = std::fs::read_to_string(path).map_err(|_| anyhow::anyhow!("TorBox credential unavailable"))?;
    let mut encrypted = hex::decode(encoded.trim().trim_start_matches('\u{feff}')).map_err(|_| anyhow::anyhow!("TorBox credential format invalid"))?;
    let input = CRYPT_INTEGER_BLOB { cbData: encrypted.len() as u32, pbData: encrypted.as_mut_ptr() };
    let mut output = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
    let ok = unsafe { CryptUnprotectData(&input, std::ptr::null_mut(), std::ptr::null(), std::ptr::null(), std::ptr::null(), CRYPTPROTECT_UI_FORBIDDEN, &mut output) };
    ensure!(ok != 0, "TorBox credential cannot be decrypted by this Windows account");
    let bytes = unsafe { std::slice::from_raw_parts_mut(output.pbData, output.cbData as usize) };
    let mut units = Zeroizing::new(bytes.chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]])).collect::<Vec<_>>());
    let decoded = if bytes.len() % 2 == 0 { String::from_utf16(&units).ok() } else { None };
    bytes.zeroize(); units.zeroize();
    unsafe { LocalFree(output.pbData.cast()); }
    let key = Zeroizing::new(decoded.ok_or_else(|| anyhow::anyhow!("TorBox credential encoding invalid"))?);
    if key.is_empty() { bail!("TorBox credential is empty"); }
    Ok(key)
}
#[cfg(not(windows))]
pub fn read_key(_: &Path) -> Result<Zeroizing<String>> { bail!("TorBox credentials require Windows DPAPI") }

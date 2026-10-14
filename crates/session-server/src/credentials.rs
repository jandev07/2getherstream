//! Current-user Windows DPAPI vault. Never Serialize/Debug this wrapper or print tokens.
use anyhow::{ensure,Result};
use serde::{Deserialize,Serialize};
use std::{io::{Read,Write},path::Path};
const MAX_VAULT_BYTES:usize=16*1024;
use zeroize::Zeroizing;
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Data {version:u32,pc:String,phone:String}
pub struct Credentials {pub pc:Zeroizing<String>,pub phone:Zeroizing<String>}
// Same transport boundary as Android Pairing: a credential must survive a
// Bearer header unchanged. Never trim or normalize persisted identities.
pub(crate) fn valid_pairing_token(token:&str)->bool {
    (32..=256).contains(&token.len()) && token.bytes().all(|byte| (33..=126).contains(&byte))
}
impl Credentials {
    #[cfg(windows)]
    pub fn load_or_create(path:&Path)->Result<Self> {
        use windows_sys::Win32::{Foundation::LocalFree,Security::Cryptography::{CryptProtectData,CryptUnprotectData,CRYPT_INTEGER_BLOB,CRYPTPROTECT_UI_FORBIDDEN}};
        use zeroize::Zeroize;
        fn protect(input:&mut [u8],decrypt:bool)->Result<Vec<u8>> {
            let blob=CRYPT_INTEGER_BLOB {cbData:input.len().try_into()?,pbData:input.as_mut_ptr()};
            let mut output=CRYPT_INTEGER_BLOB {cbData:0,pbData:std::ptr::null_mut()};
            let result=unsafe {if decrypt {
                CryptUnprotectData(&blob,std::ptr::null_mut(),std::ptr::null(),std::ptr::null(),std::ptr::null(),CRYPTPROTECT_UI_FORBIDDEN,&mut output)
            } else {
                CryptProtectData(&blob,std::ptr::null(),std::ptr::null(),std::ptr::null(),std::ptr::null(),CRYPTPROTECT_UI_FORBIDDEN,&mut output)
            }};
            ensure!(result!=0,"Pairing vault could not be protected for this Windows account");
            let bytes=unsafe {std::slice::from_raw_parts_mut(output.pbData,output.cbData as usize)};
            let owned=bytes.to_vec();bytes.zeroize();unsafe {LocalFree(output.pbData.cast());}Ok(owned)
        }
        fn read_existing(file:std::fs::File)->Result<Zeroizing<Vec<u8>>> {
            // Bound the actual read on one handle, including a file growing
            // after it was opened. Never replace unreadable/invalid pairing.
            let mut encrypted=Vec::new();file.take((MAX_VAULT_BYTES+1) as u64).read_to_end(&mut encrypted)?;
            ensure!(!encrypted.is_empty() && encrypted.len()<=MAX_VAULT_BYTES,"Pairing vault is invalid");
            Ok(Zeroizing::new(protect(&mut encrypted,true)?))
        }
        let bytes=match std::fs::File::open(path) {
            Ok(file)=>read_existing(file)?,
            Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{
            let parent=path.parent().ok_or_else(||anyhow::anyhow!("Pairing vault path invalid"))?;std::fs::create_dir_all(parent)?;
            let mut data=Data {version:1,pc:format!("{}{}",uuid::Uuid::new_v4(),uuid::Uuid::new_v4()),phone:format!("{}{}",uuid::Uuid::new_v4(),uuid::Uuid::new_v4())};
            let mut raw=Zeroizing::new(serde_json::to_vec(&data)?);data.pc.zeroize();data.phone.zeroize();
            let encrypted=protect(&mut raw,false)?;
            // Do not overwrite an existing vault or silently replace unreadable pairing.
            let temporary=parent.join(format!("pairing-{}.tmp",uuid::Uuid::new_v4()));
            let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(&temporary)?;
            let saved=(||->std::io::Result<()> {file.write_all(&encrypted)?;file.sync_all()?;drop(file);std::fs::hard_link(&temporary,path)?;Ok(())})();
            let _=std::fs::remove_file(&temporary);
            match saved {
                Ok(())=>raw,
                // Another creator published first. Use its persisted identity,
                // never the losing credentials or an overwrite/retry creation.
                Err(error) if error.kind()==std::io::ErrorKind::AlreadyExists=>{
                    drop(raw);read_existing(std::fs::File::open(path)?)?
                },
                Err(error)=>return Err(error.into()),
            }
            },
            Err(error)=>return Err(error.into()),
        };
        let data:Data=serde_json::from_slice(&bytes).map_err(|_|anyhow::anyhow!("Pairing vault format invalid"))?;
        ensure!(data.version==1 && valid_pairing_token(&data.pc) && valid_pairing_token(&data.phone) && data.pc!=data.phone,"Pairing vault contents invalid");
        Ok(Self {pc:Zeroizing::new(data.pc),phone:Zeroizing::new(data.phone)})
    }
}
#[cfg(all(test,windows))] mod tests {
    use super::*;
    #[test] fn protected_header_unsafe_vault_is_rejected_without_replacement() {
        use windows_sys::Win32::{Foundation::LocalFree,Security::Cryptography::{CryptProtectData,CRYPT_INTEGER_BLOB,CRYPTPROTECT_UI_FORBIDDEN}};
        // Synthetic credentials only: exercise valid DPAPI ciphertext whose
        // plaintext identity cannot be transported unchanged in a Bearer header.
        let directory=std::env::temp_dir().join(format!("together2-vault-header-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();let path=directory.join("pairing.dpapi");
        for invalid in [" ".repeat(32),format!("{}\r\nPRIVATE_TOKEN", "x".repeat(32)),"é".repeat(32)] {
            for phone in [false,true] {
                let (pc,phone)=if phone {("p".repeat(32),invalid.clone())} else {(invalid.clone(),"f".repeat(32))};
                let mut raw=Zeroizing::new(serde_json::to_vec(&Data {version:1,pc,phone}).unwrap());
                let input=CRYPT_INTEGER_BLOB {cbData:raw.len().try_into().unwrap(),pbData:raw.as_mut_ptr()};
                let mut output=CRYPT_INTEGER_BLOB {cbData:0,pbData:std::ptr::null_mut()};
                assert_ne!(unsafe {CryptProtectData(&input,std::ptr::null(),std::ptr::null(),std::ptr::null(),std::ptr::null(),CRYPTPROTECT_UI_FORBIDDEN,&mut output)},0);
                let encrypted=unsafe {std::slice::from_raw_parts(output.pbData,output.cbData as usize)}.to_vec();
                unsafe {LocalFree(output.pbData.cast());}
                std::fs::write(&path,&encrypted).unwrap();
                let error=match Credentials::load_or_create(&path) {Ok(_)=>panic!("unsafe vault accepted"),Err(error)=>error};
                assert_eq!(error.to_string(),"Pairing vault contents invalid");
                assert_eq!(std::fs::read(&path).unwrap(),encrypted);
                assert_eq!(std::fs::read_dir(&directory).unwrap().count(),1);
            }
        }
        std::fs::remove_file(path).unwrap();std::fs::remove_dir(directory).unwrap();
    }
    #[test] fn concurrent_first_launches_converge_on_one_persisted_identity() {
        let directory=std::env::temp_dir().join(format!("together2-vault-race-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();let path=directory.join("pairing.dpapi");
        let barrier=std::sync::Arc::new(std::sync::Barrier::new(8));
        let workers:Vec<_>=(0..8).map(|_|{
            let path=path.clone();let barrier=barrier.clone();std::thread::spawn(move ||{barrier.wait();Credentials::load_or_create(&path)})
        }).collect();
        let credentials:Vec<_>=workers.into_iter().map(|worker|worker.join().unwrap().unwrap()).collect();
        let persisted=Credentials::load_or_create(&path).unwrap();
        assert!(credentials.iter().all(|c|c.pc.as_bytes()==persisted.pc.as_bytes() && c.phone.as_bytes()==persisted.phone.as_bytes()));
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(),1,"losing creators left temporary vaults");
        drop(credentials);drop(persisted);std::fs::remove_file(path).unwrap();std::fs::remove_dir(directory).unwrap();
    }
    #[test] fn empty_and_oversized_vaults_are_rejected_without_replacement() {
        let directory=std::env::temp_dir().join(format!("together2-vault-bound-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();let path=directory.join("pairing.dpapi");
        for size in [0,MAX_VAULT_BYTES+1] {
            let contents=vec![42;size];std::fs::write(&path,&contents).unwrap();
            assert!(Credentials::load_or_create(&path).is_err());assert_eq!(std::fs::read(&path).unwrap(),contents);
            assert_eq!(std::fs::read_dir(&directory).unwrap().count(),1);
        }
        std::fs::remove_file(path).unwrap();std::fs::remove_dir(directory).unwrap();
    }
    #[test] fn vault_round_trip_and_tampering_fails_without_replacement() {
        let directory=std::env::temp_dir().join(format!("together2-vault-test-{}",uuid::Uuid::new_v4()));
        let path=directory.join("pairing.dpapi");
        let first=Credentials::load_or_create(&path).unwrap();let second=Credentials::load_or_create(&path).unwrap();
        assert_eq!(first.pc.as_str(),second.pc.as_str());assert_eq!(first.phone.as_str(),second.phone.as_str());
        let raw=std::fs::read(&path).unwrap();assert!(!raw.windows(first.phone.len()).any(|v|v==first.phone.as_bytes()));
        std::fs::write(&path,b"tampered").unwrap();assert!(Credentials::load_or_create(&path).is_err());assert_eq!(std::fs::read(&path).unwrap(),b"tampered");
        std::fs::remove_file(path).unwrap();std::fs::remove_dir(directory).unwrap();
    }
}

//! Native libmpv adapter foundation. Own this on a dedicated player thread.
//! The session controller (not UI) is the only caller. Runtime library is external.
use anyhow::{bail, ensure, Context, Result};
use libloading::Library;
mod media_url;
use std::{ffi::{c_char, c_int, c_void, CString, CStr}, path::Path, ptr};
type Handle = c_void;
fn native_time_ms(value:f64)->Result<u64> {
    ensure!(value.is_finite() && (0.0..=604800.0).contains(&value),"invalid native timing");
    Ok((value*1000.0) as u64)
}
#[cfg(test)] mod native_time_tests {
    use super::native_time_ms;
    #[test] fn invalid_native_times_never_saturate_into_ready_metadata() {
        for value in [f64::NAN,f64::INFINITY,f64::NEG_INFINITY,-0.001,604800.001,f64::MAX] {
            assert!(native_time_ms(value).is_err());
        }
    }
    #[test] fn valid_native_times_preserve_frames_and_protocol_limit() {
        for (seconds,millis) in [(0.0,0),(0.0419,41),(12.345,12345),(604800.0,604800000)] {
            assert_eq!(native_time_ms(seconds).unwrap(),millis);
        }
    }
}
type Create = unsafe extern "C" fn() -> *mut Handle;
type Initialize = unsafe extern "C" fn(*mut Handle) -> c_int;
type Destroy = unsafe extern "C" fn(*mut Handle);
// Keep the newly created core owned across every fallible startup step. This
// guard drops before the library on failure; successful construction transfers
// the handle to Mpv, whose existing Drop owns the eventual release.
struct StartupCore { handle: *mut Handle, destroy: Destroy }
impl StartupCore {
    fn transfer(mut self) -> *mut Handle { std::mem::replace(&mut self.handle, ptr::null_mut()) }
}
impl Drop for StartupCore {
    fn drop(&mut self) { if !self.handle.is_null() { unsafe { (self.destroy)(self.handle); } } }
}

type SetString = unsafe extern "C" fn(*mut Handle, *const c_char, *const c_char) -> c_int;
type Command = unsafe extern "C" fn(*mut Handle, u64, *const *const c_char) -> c_int;
type CommandSync = unsafe extern "C" fn(*mut Handle,*const *const c_char) -> c_int;
type Get = unsafe extern "C" fn(*mut Handle, *const c_char, c_int, *mut c_void) -> c_int;
type Wait = unsafe extern "C" fn(*mut Handle, f64) -> *const RawEvent;
type GetString = unsafe extern "C" fn(*mut Handle, *const c_char) -> *mut c_char;
type Free = unsafe extern "C" fn(*mut c_void);
#[repr(C)]
struct RawEvent { id: c_int, error: c_int, userdata: u64, data: *mut c_void }
#[repr(C)]
struct RawEndFile {reason:c_int,error:c_int,playlist_entry_id:i64,playlist_insert_id:i64,playlist_insert_num_entries:c_int}
#[derive(Debug)]
pub struct Event { pub id: i32, pub error: i32, pub generation: u64, pub source_failure:bool }
pub struct Mpv {
    #[cfg(test)] last_end_file:Option<(c_int,c_int,i64)>,
    handle: *mut Handle, set_property: SetString, command: Command,
    get: Get, wait: Wait, destroy: Destroy, get_string: GetString, free: Free, command_sync:CommandSync, source_entry:Option<i64>, command_failed:bool, _library: Library,
}
// No Send/Sync implementation: construct and use on the same player worker.
impl Mpv {
    /// A fully qualified, trusted library path; never search the current directory.
    pub fn open(path: &Path) -> Result<Self> {
        Self::open_with_options(path, &[])
    }
    pub fn open_with_options(path: &Path, extra: &[(&str, &str)]) -> Result<Self> {
        ensure!(path.is_absolute(), "libmpv path must be absolute");
        unsafe {
            let library = Library::new(path).context("load native libmpv")?;
            let create: Create = *library.get(b"mpv_create\0")?;
            let initialize: Initialize = *library.get(b"mpv_initialize\0")?;
            let destroy: Destroy = *library.get(b"mpv_terminate_destroy\0")?;
            let option: SetString = *library.get(b"mpv_set_option_string\0")?;
            let set_property = *library.get(b"mpv_set_property_string\0")?;
            let command = *library.get(b"mpv_command_async\0")?;
            let command_sync=*library.get(b"mpv_command\0")?;
            let get = *library.get(b"mpv_get_property\0")?;
            let wait = *library.get(b"mpv_wait_event\0")?;
            let get_string = *library.get(b"mpv_get_property_string\0")?;
            let free = *library.get(b"mpv_free\0")?;
            let handle = create(); ensure!(!handle.is_null(), "mpv_create failed");
            let startup = StartupCore { handle, destroy };
            // HTTP/decoder messages may contain private URLs. Typed numeric
            // events still report failures with raw native messages disabled.
            for (key, value) in [("config", "no"), ("input-default-bindings", "no"), ("input-vo-keyboard", "no"), ("terminal", "no"), ("msg-level", "all=no"), ("ytdl", "no"), ("tls-verify", "yes"), ("pause", "yes"), ("keep-open", "yes"), ("hwdec", "auto-safe")] {
                let key = CString::new(key)?; let value = CString::new(value)?;
                if option(handle,key.as_ptr(),value.as_ptr()) < 0 { bail!("libmpv option initialization failed"); }
            }
            for (key, value) in extra {
                let key = CString::new(*key)?; let value = CString::new(*value)?;
                if option(handle,key.as_ptr(),value.as_ptr()) < 0 { bail!("libmpv test option failed"); }
            }
            if initialize(handle) < 0 { bail!("libmpv initialization failed"); }
            let handle = startup.transfer();
            Ok(Self { #[cfg(test)] last_end_file:None, handle, set_property, command, get, wait, destroy, get_string, free, command_sync, source_entry:None, command_failed:false, _library: library })
        }
    }
    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let key = CString::new(key)?; let value = CString::new(value)?;
        let code=unsafe { (self.set_property)(self.handle,key.as_ptr(),value.as_ptr()) };
        self.accept_command(code)
    }
    fn accept_command(&mut self,code:c_int)->Result<()> {
        if code<0 {self.command_failed=true;bail!("native playback command rejected");}
        Ok(())
    }
    /// Sticky until consumed by the owning controller; a later successful
    /// property write must not conceal a failed pause/seek/rate operation.
    pub fn take_command_failure(&mut self)->bool {std::mem::take(&mut self.command_failed)}
    pub fn pause(&mut self, paused: bool) -> Result<()> { self.set("pause", if paused { "yes" } else { "no" }) }
    pub fn speed(&mut self, value: f64) -> Result<()> {
        if !value.is_finite() || !(0.95..=1.005).contains(&value) {self.command_failed=true;bail!("invalid correction rate");}
        self.set("speed", &value.to_string())
    }
    pub fn queue(&mut self, generation: u64, args: &[&str]) -> Result<()> {
        let strings = args.iter().map(|s| CString::new(*s)).collect::<Result<Vec<_>,_>>()?;
        let mut pointers: Vec<_> = strings.iter().map(|s| s.as_ptr()).collect(); pointers.push(ptr::null());
        let code=unsafe { (self.command)(self.handle,generation,pointers.as_ptr()) };
        self.accept_command(code)
    }
    pub fn prepare_url(&mut self, url: &str, generation: u64) -> Result<()> {
        media_url::validate(url)?;
        let _=generation;self.load_source(url)
    }
    pub fn prepare_local(&mut self,path:&Path) -> Result<()> {
        ensure!(path.is_absolute(),"local source path must be absolute");self.load_source(&path.to_string_lossy())
    }
    fn integer_property(&mut self,name:&str) -> Result<i64> {
        let key=CString::new(name)?;let mut value=0i64;
        ensure!(unsafe {(self.get)(self.handle,key.as_ptr(),4,(&mut value as *mut i64).cast())}>=0,"native source identity unavailable");Ok(value)
    }
    fn load_source(&mut self,source:&str) -> Result<()> {
        self.source_entry=None;self.pause(true)?;self.speed(1.0)?;
        let strings=[CString::new("loadfile")?,CString::new(source)?,CString::new("replace")?];
        let pointers=[strings[0].as_ptr(),strings[1].as_ptr(),strings[2].as_ptr(),ptr::null()];
        let code=unsafe {(self.command_sync)(self.handle,pointers.as_ptr())};self.accept_command(code)?;
        self.source_entry=Some(self.integer_property("playlist/0/id")?);Ok(())
    }
    /// Match native playlist entry IDs, not anonymous FILE_LOADED callbacks or URL strings.
    pub fn source_ready(&mut self) -> bool {
        let Some(expected)=self.source_entry else {return false};
        let Ok(position)=self.integer_property("playlist-playing-pos") else {return false};
        if position<0 {return false;}
        self.integer_property(&format!("playlist/{position}/id")).is_ok_and(|id|id==expected)
            && self.position_ms().is_ok()
            && self.duration_seconds().is_ok_and(|d|d>0.0)
    }
    pub fn seek(&mut self, position_ms: u64, generation: u64) -> Result<()> {
        self.pause(true)?; self.speed(1.0)?;
        self.queue(generation, &["seek", &(position_ms as f64 / 1000.0).to_string(), "absolute+exact"])
    }
    pub fn position_ms(&mut self) -> Result<u64> {
        let key = CString::new("time-pos")?; let mut value = 0f64;
        // MPV_FORMAT_DOUBLE = 5.
        ensure!(unsafe { (self.get)(self.handle,key.as_ptr(),5,(&mut value as *mut f64).cast()) } >= 0, "position unavailable");
        native_time_ms(value)
    }
    pub fn duration_seconds(&mut self)->Result<f64> {
        let value=self.property("duration")?.parse::<f64>().map_err(|_|anyhow::anyhow!("invalid native timing"))?;
        native_time_ms(value)?;Ok(value)
    }
    /// Use only allowlisted diagnostic properties; never ask for path/stream URLs.
    pub fn property(&mut self, name: &str) -> Result<String> {
        ensure!(["duration", "pause", "paused-for-cache", "seeking", "eof-reached", "track-list/count", "mpv-version", "tls-verify"].contains(&name), "property not permitted");
        let key = CString::new(name)?;
        unsafe {
            let raw = (self.get_string)(self.handle, key.as_ptr());
            ensure!(!raw.is_null(), "property unavailable");
            let result = CStr::from_ptr(raw).to_string_lossy().into_owned();
            (self.free)(raw.cast()); Ok(result)
        }
    }
    pub fn poll(&mut self) -> Option<Event> {
        unsafe { let e = (self.wait)(self.handle, 0.0).as_ref()?;
            if e.id == 0 { None } else {
                // END_FILE payload is valid only until the next mpv_wait_event call.
                // Copy numeric fields now; never copy paths or native log messages.
                let end=if e.id==7 {e.data.cast::<RawEndFile>().as_ref()} else {None};
                #[cfg(test)] {self.last_end_file=end.map(|end|(end.reason,end.error,end.playlist_entry_id));}
                let source_failure=end.is_some_and(|end|end.reason==4 && Some(end.playlist_entry_id)==self.source_entry);
                Some(Event { id:e.id, error:e.error, generation:e.userdata,source_failure })
            } }
    }
}
impl Drop for Mpv { fn drop(&mut self) { unsafe { (self.destroy)(self.handle); } } }

#[cfg(all(test,windows))]
mod command_failure_tests {
    use super::*;
    fn player()->Mpv {
        let dll=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../native/vendor/windows/libmpv-2.dll");
        Mpv::open_with_options(&dll,&[("vo","null"),("ao","null")]).unwrap()
    }
    #[test] fn native_property_rejection_survives_a_later_success_until_consumed() {
        let mut p=player();assert!(!p.take_command_failure());
        assert!(p.set("together-invalid-property","yes").is_err());
        p.pause(true).unwrap();
        assert!(p.take_command_failure());assert!(!p.take_command_failure());
        p.speed(1.0).unwrap();assert!(!p.take_command_failure());
        // Reads can be unavailable while idle without being command failures.
        assert!(p.position_ms().is_err());assert!(!p.take_command_failure());
    }
    #[test] fn invalid_rate_cannot_silently_bypass_failure_observation() {
        let mut p=player();
        for rate in [f64::NAN,f64::INFINITY,0.9,1.1] {
            assert!(p.speed(rate).is_err());assert!(p.take_command_failure());
        }
    }
    #[test] fn rejected_async_command_is_observable_before_any_event_callback() {
        let mut p=player();
        assert!(p.queue(7,&["together-invalid-command"]).is_err());
        assert!(p.take_command_failure());assert!(!p.take_command_failure());
    }
}

#[cfg(all(test,windows))]
mod end_file_ownership_tests {
    use super::*;
    use std::time::{Duration,Instant,SystemTime,UNIX_EPOCH};
    fn await_idle(p:&mut Mpv) {
        let deadline=Instant::now()+Duration::from_secs(3);let mut samples=0;
        while Instant::now()<deadline {
            if p.integer_property("playlist-playing-pos").is_ok_and(|position|position<0) && p.position_ms().is_err() {
                samples+=1;if samples>=5 {return;}
            } else {samples=0;}
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("native rejected source did not reach idle");
    }
    #[test]
    fn queued_old_format_error_is_not_a_failure_of_replacement_entry() {
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
        let directory=std::env::var_os("TOGETHER_END_FILE_DIR").map(std::path::PathBuf::from).unwrap_or_else(||
            root.join("work/source-validation").join(format!("end-file-{}-{}",std::process::id(),SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos())));
        std::fs::create_dir_all(&directory).unwrap();
        let invalid=directory.join("invalid-local-format.mp4");std::fs::write(&invalid,b"PRIVATE_NOT_AN_MP4_SOURCE\n").unwrap();
        let dll=root.join("native/vendor/windows/libmpv-2.dll");
        let fixture=root.join("work/media/clock-test.mp4");
        let mut p=Mpv::open_with_options(&dll,&[("vo","null"),("ao","null")]).unwrap();
        p.prepare_local(&invalid).unwrap();let old_entry=p.source_entry.unwrap();
        // Confirm stable native idle before replacement; do not drain queued events.
        await_idle(&mut p);
        p.prepare_local(&fixture).unwrap();let replacement_entry=p.source_entry.unwrap();
        let deadline=Instant::now()+Duration::from_secs(3);
        let mut old_errors=0;let mut old_false_failures=0;let mut ready=false;
        let mut old_native_generation=0;let mut old_error_code=0;
        while Instant::now()<deadline {
            while let Some(event)=p.poll() {
                if let Some((reason,error,entry))=p.last_end_file {
                    if reason==4 && error<0 && entry==old_entry {
                        old_errors+=1;old_native_generation=event.generation;old_error_code=error;
                        if event.source_failure {old_false_failures+=1;}
                    }
                }
            }
            ready=p.source_ready();if old_errors>0 && ready {break;}
            std::thread::sleep(Duration::from_millis(5));
        }
        // A current-entry failure must still be reported; the guard cannot mute all errors.
        p.prepare_local(&invalid).unwrap();let current_entry=p.source_entry.unwrap();await_idle(&mut p);
        let deadline=Instant::now()+Duration::from_secs(3);let mut current_errors=0;let mut current_failures=0;
        let mut current_native_generation=0;let mut current_error_code=0;
        while Instant::now()<deadline {
            while let Some(event)=p.poll() {
                if let Some((reason,error,entry))=p.last_end_file {
                    if reason==4 && error<0 && entry==current_entry {
                        current_errors+=1;current_native_generation=event.generation;current_error_code=error;
                        if event.source_failure {current_failures+=1;}
                    }
                }
            }
            if current_errors>0 {break;}
            std::thread::sleep(Duration::from_millis(5));
        }
        let receipt=format!(r#"{{
  "scope": "real muted libmpv format errors and playlist entry ownership; no controller/phone",
  "old_entry": {old_entry},
  "replacement_entry": {replacement_entry},
  "current_entry": {current_entry},
  "old_error_events": {old_errors},
  "old_false_source_failures": {old_false_failures},
  "old_native_event_generation": {old_native_generation},
  "old_native_error_code": {old_error_code},
  "replacement_source_ready": {ready},
  "current_error_events": {current_errors},
  "current_source_failures": {current_failures},
  "current_native_event_generation": {current_native_generation},
  "current_native_error_code": {current_error_code},
  "physical_gate_passed": false
}}"#);
        std::fs::write(directory.join("receipt.json"),receipt).unwrap();
        assert_ne!(old_entry,replacement_entry);assert_ne!(replacement_entry,current_entry);
        assert!(old_errors>0,"no actual old EndFile format error observed");assert!(ready);
        assert_eq!(old_false_failures,0,"old entry error was attributed to replacement");
        assert!(current_errors>0);assert_eq!(current_failures,current_errors);
    }
}

#[cfg(all(test,windows))]
mod observation_latency_tests {
    use super::*;
    use std::io::Write;
    use std::time::{Duration,Instant,SystemTime,UNIX_EPOCH};
    #[test]
    fn native_position_observation_latency_uses_isolated_numeric_receipt() {
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
        let directory=std::env::var_os("TOGETHER_POSITION_LATENCY_DIR").map(std::path::PathBuf::from).unwrap_or_else(||
            root.join("work/source-validation").join(format!("position-latency-{}-{}",std::process::id(),SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos())));
        std::fs::create_dir_all(&directory).unwrap();
        let mut p=Mpv::open_with_options(&root.join("native/vendor/windows/libmpv-2.dll"),
            &[("vo","null"),("ao","null"),("mute","yes")]).unwrap();
        p.prepare_local(&root.join("work/media/clock-test.mp4")).unwrap();
        let deadline=Instant::now()+Duration::from_secs(3);
        while !p.source_ready() {
            assert!(Instant::now()<deadline,"fixture did not become ready");
            while let Some(event)=p.poll() {assert!(!event.source_failure);}
            std::thread::sleep(Duration::from_millis(5));
        }
        p.pause(false).unwrap();
        let deadline=Instant::now()+Duration::from_secs(3);
        while p.position_ms().unwrap()<50 {
            assert!(Instant::now()<deadline,"native playback did not advance");
            std::thread::sleep(Duration::from_millis(5));
        }
        let mut rows=std::fs::File::create(directory.join("observations.csv")).unwrap();
        writeln!(rows,"index,read_started_ns,position_sampled_ns,position_read_ns,duration_read_ns,position_ms").unwrap();
        let origin=Instant::now();let mut first=0;let mut last=0;let mut backwards=0;
        for index in 0..500 {
            let duration_start=Instant::now();let duration=p.duration_seconds().unwrap();let duration_ns=duration_start.elapsed().as_nanos();
            assert!(duration>0.0);
            let started=origin.elapsed().as_nanos();let position=p.position_ms().unwrap();let sampled=origin.elapsed().as_nanos();
            if index==0 {first=position;}else if position<last {backwards+=1;}
            last=position;
            writeln!(rows,"{index},{started},{sampled},{},{duration_ns},{position}",sampled-started).unwrap();
            std::thread::sleep(Duration::from_millis(10));
        }
        rows.flush().unwrap();p.pause(true).unwrap();assert!(!p.take_command_failure());
        std::fs::write(directory.join("receipt.json"),format!(r#"{{"scope":"real muted native adapter API latency; no controller or phone","observations":500,"first_position_ms":{first},"last_position_ms":{last},"backwards_observations":{backwards},"hard_latency_bound_verified":false,"frame_accuracy_verified":false,"physical_gate_passed":false}}"#)).unwrap();
        assert!(last>first);assert_eq!(backwards,0);
    }
}

#[cfg(test)]
mod startup_ownership_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    unsafe extern "C" fn release(handle: *mut Handle) {
        // Test handle points at a counter, never at a native core.
        unsafe { &*handle.cast::<AtomicUsize>() }.fetch_add(1, Ordering::SeqCst);
    }
    fn owned(count: &AtomicUsize) -> StartupCore {
        StartupCore { handle: (count as *const AtomicUsize).cast_mut().cast(), destroy: release }
    }
    #[test]
    fn option_conversion_early_return_releases_created_handle() {
        fn configure(count: &AtomicUsize, key: &str, value: &str) -> Result<()> {
            let _core = owned(count);
            let _key = CString::new(key)?;
            let _value = CString::new(value)?;
            Ok(())
        }
        for (key, value) in [("bad\0key", "yes"), ("pause", "bad\0value")] {
            let count = AtomicUsize::new(0);
            assert!(configure(&count, key, value).is_err());
            assert_eq!(count.load(Ordering::SeqCst), 1);
        }
    }
    #[test]
    fn rejection_and_unwind_release_once() {
        let count = AtomicUsize::new(0);
        let result = std::panic::catch_unwind(|| { let _core = owned(&count); panic!("controlled startup unwind"); });
        assert!(result.is_err());
        assert_eq!(count.load(Ordering::SeqCst), 1);
        fn reject(count: &AtomicUsize) -> Result<()> { let _core = owned(count); bail!("fixed startup rejection") }
        assert!(reject(&count).is_err());
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }
    #[test]
    fn successful_transfer_leaves_release_to_final_owner() {
        let count = AtomicUsize::new(0);
        let handle = owned(&count).transfer();
        assert_eq!(count.load(Ordering::SeqCst), 0);
        unsafe { release(handle); }
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}

#[cfg(all(test, windows))]
mod native_startup_tests {
    use super::*;
    #[test]
    fn invalid_options_reject_then_muted_core_starts() {
        let dll = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../native/vendor/windows/libmpv-2.dll");
        for extra in [[("bad\0key", "yes")], [("pause", "bad\0value")], [("together-invalid-option", "yes")]] {
            assert!(Mpv::open_with_options(&dll, &extra).is_err());
        }
        let mut core = Mpv::open_with_options(&dll, &[("vo", "null"), ("ao", "null"), ("mute", "yes")]).unwrap();
        assert_eq!(core.property("pause").unwrap(), "yes");
        assert_eq!(core.property("tls-verify").unwrap(), "yes");
        core.pause(true).unwrap();
    }
}

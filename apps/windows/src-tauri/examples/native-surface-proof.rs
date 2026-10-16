//! Owned, muted local-fixture presentation diagnostic. No session/provider access.
//! Inspect the window with the normal desktop capture tool; native readiness and
//! position alone do not attest a presented frame. Never run beside a native owner.
#[cfg(windows)]
mod proof {
    use std::{path::Path, sync::atomic::{AtomicBool, Ordering}, time::{Duration, Instant}};
    use together_player_mpv::Mpv;
    use windows_sys::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

    static CLOSE: AtomicBool = AtomicBool::new(false);
    unsafe extern "system" fn window_proc(hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
        if message == WM_CLOSE { CLOSE.store(true, Ordering::Release); return 0; }
        unsafe { DefWindowProcW(hwnd, message, w, l) }
    }
    struct Surface(HWND);
    impl Drop for Surface { fn drop(&mut self) { unsafe { DestroyWindow(self.0); } } }
    fn wide(text: &str) -> Vec<u16> { text.encode_utf16().chain(Some(0)).collect() }

    pub fn run() -> Result<serde_json::Value, &'static str> {
        let args: Vec<_> = std::env::args().collect();
        let dll = Path::new(args.get(1).ok_or("surface_dll_required")?);
        let fixture = Path::new(args.get(2).ok_or("surface_fixture_required")?);
        if !dll.is_absolute() || !fixture.is_absolute() || !fixture.is_file() { return Err("surface_local_inputs_required"); }
        let class = wide("STATIC");
        let title = wide("2gether muted native surface · paused load");
        let hwnd = unsafe { CreateWindowExW(0, class.as_ptr(), title.as_ptr(), WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            80, 80, 500, 320, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null()) };
        if hwnd.is_null() { return Err("surface_window_failed"); }
        let surface = Surface(hwnd);
        // Subclass only this diagnostic's own window so a close request cannot
        // destroy the native output underneath its still-owned player.
        if unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, window_proc as *const () as isize) } == 0 {
            return Err("surface_close_owner_failed");
        }
        let parent = (hwnd as usize).to_string();
        // player drops before surface even on failure; WM_CLOSE only requests it.
        let mut player = Mpv::open_with_options(dll, &[("wid", &parent), ("vo", "gpu"), ("gpu-context", "d3d11"),
            ("force-window", "yes"), ("mute", "yes"), ("ao", "null")]).map_err(|_| "surface_native_initialization_failed")?;
        player.prepare_local(fixture).map_err(|_| "surface_load_failed")?;
        let started = Instant::now();
        let mut phase = 0;
        let mut ready = false;
        let mut positions = Vec::new();
        let mut observed = 0;
        while started.elapsed() < Duration::from_secs(50) && !CLOSE.load(Ordering::Acquire) {
            let mut message: MSG = unsafe { std::mem::zeroed() };
            while unsafe { PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
                unsafe { TranslateMessage(&message); DispatchMessageW(&message); }
            }
            while let Some(event) = player.poll() {
                if event.error < 0 || event.source_failure { return Err("surface_native_event_failed"); }
            }
            ready |= player.source_ready();
            if !ready && started.elapsed() > Duration::from_secs(10) { return Err("surface_ready_timeout"); }
            let next = if started.elapsed() < Duration::from_secs(20) { 0 } else if started.elapsed() < Duration::from_secs(35) { 1 } else { 2 };
            if next != phase {
                player.pause(next != 1).map_err(|_| "surface_pause_failed")?;
                phase = next;
                positions.push(player.position_ms().map_err(|_| "surface_position_missing")?);
                let title = wide(if phase == 1 { "2gether muted native surface · playing" } else { "2gether muted native surface · paused after progress" });
                unsafe { SetWindowTextW(hwnd, title.as_ptr()); }
            }
            observed += 1;
            std::thread::sleep(Duration::from_millis(20));
        }
        let position = player.position_ms().map_err(|_| "surface_position_missing")?;
        let paused = player.property("pause").map_err(|_| "surface_pause_missing")? == "yes";
        drop(player);
        drop(surface);
        Ok(serde_json::json!({"native_ready":ready,"last_phase":phase,"observations":observed,"phase_positions_ms":positions,
            "position_ms":position,"paused":paused,"muted":true,"closed_by_user":CLOSE.load(Ordering::Acquire),
            "native_released":true,"presented_frame_verified_by_receipt":false,"session_or_phone_verified":false}))
    }
}

fn main() {
    #[cfg(windows)] {
        let result = proof::run();
        println!("{}", match &result { Ok(value) => value.clone(), Err(code) => serde_json::json!({"error":code}) });
        if result.is_err() { std::process::exit(1); }
    }
    #[cfg(not(windows))] { eprintln!("Windows presentation proof only"); std::process::exit(1); }
}

//! Optional Windows 11 DWM Integration (dynamically loaded so Win 7/8/10 remain compatible)

use windows::core::{s, w};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

type DwmSetWindowAttributeFn =
    unsafe extern "system" fn(HWND, u32, *const std::ffi::c_void, u32) -> i32;

const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
const DWMWCP_ROUND: u32 = 2;

/// Applies native Windows 11 rounded corners & drop shadow if supported by the OS.
/// Safely does nothing on Windows 7, 8, and 10.
pub fn enable_win11_rounded_corners(hwnd: HWND) {
    unsafe {
        let Ok(dwm) = LoadLibraryW(w!("dwmapi.dll")) else {
            return;
        };
        if let Some(proc) = GetProcAddress(dwm, s!("DwmSetWindowAttribute")) {
            let func: DwmSetWindowAttributeFn = std::mem::transmute(proc);
            let pref = DWMWCP_ROUND;
            let _ = func(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &pref as *const u32 as *const std::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            );
        }
    }
}

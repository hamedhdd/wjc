//! High-DPI Scaling Utilities (Per-Monitor aware on Win10+, safe fallback on Win7/8)

use windows::core::s;
use windows::core::w;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, ReleaseDC, LOGPIXELSY};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

type GetDpiForWindowFn = unsafe extern "system" fn(HWND) -> u32;

/// Queries the window's live monitor DPI via `GetDpiForWindow` (Win10 1607+),
/// falling back gracefully to `GetDeviceCaps(LOGPIXELSY)` on Windows 7/8.
pub fn get_dpi_for_window_or_system(hwnd: HWND) -> u32 {
    unsafe {
        if !hwnd.0.is_null() {
            if let Ok(user32) = GetModuleHandleW(w!("user32.dll")) {
                if let Some(proc) = GetProcAddress(user32, s!("GetDpiForWindow")) {
                    let func: GetDpiForWindowFn = std::mem::transmute(proc);
                    let dpi = func(hwnd);
                    if dpi >= 96 {
                        return dpi;
                    }
                }
            }
        }

        let hdc = GetDC(hwnd);
        if !hdc.is_invalid() {
            let dpi = GetDeviceCaps(hdc, LOGPIXELSY);
            ReleaseDC(hwnd, hdc);
            if dpi > 0 {
                return dpi as u32;
            }
        }
        96
    }
}

pub fn scale_for_dpi(val: i32, dpi: u32) -> i32 {
    (val * (dpi as i32) + 48) / 96
}

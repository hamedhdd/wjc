//! High-DPI Scaling Utilities

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, ReleaseDC, LOGPIXELSY};

pub fn get_dpi_for_window_or_system(hwnd: HWND) -> u32 {
    unsafe {
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

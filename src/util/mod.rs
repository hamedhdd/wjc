pub mod autostart;
pub mod dpi;
pub mod time;

use windows::Win32::Foundation::HWND;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SendHwnd(pub HWND);
unsafe impl Send for SendHwnd {}
unsafe impl Sync for SendHwnd {}

impl SendHwnd {
    pub fn get(self) -> HWND {
        self.0
    }
}

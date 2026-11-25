//! Windows System Theme Detection Helpers

use windows::core::w;
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ,
    REG_DWORD, REG_VALUE_TYPE,
};

/// Returns true if Windows system taskbar/tray uses light theme.
/// Checks `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\SystemUsesLightTheme`.
/// If the key does not exist (e.g. Windows 7/8/early 10), defaults to false (dark taskbar).
pub fn is_windows_system_light_theme() -> bool {
    unsafe {
        let subkey = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, 0, KEY_READ, &mut hkey).is_err() {
            return false;
        }

        let value_name = w!("SystemUsesLightTheme");
        let mut value_type = REG_VALUE_TYPE(0);
        let mut data: u32 = 0;
        let mut data_len = std::mem::size_of::<u32>() as u32;

        let status = RegQueryValueExW(
            hkey,
            value_name,
            None,
            Some(&mut value_type),
            Some(&mut data as *mut u32 as *mut u8),
            Some(&mut data_len),
        );

        let _ = RegCloseKey(hkey);

        if status.is_ok() && value_type == REG_DWORD {
            data == 1
        } else {
            false
        }
    }
}

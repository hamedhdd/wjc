//! Windows System Theme Detection Helpers

use windows::core::{w, PCWSTR};
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ,
    REG_DWORD, REG_VALUE_TYPE,
};

fn read_personalize_dword(value_name: PCWSTR) -> Option<u32> {
    unsafe {
        let subkey = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, 0, KEY_READ, &mut hkey).is_err() {
            return None;
        }

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
            Some(data)
        } else {
            None
        }
    }
}

/// Returns true if Windows system taskbar/tray uses light theme.
/// Defaults to false (dark taskbar) on Windows 7/8/early 10 where the key is absent.
pub fn is_windows_system_light_theme() -> bool {
    read_personalize_dword(w!("SystemUsesLightTheme")) == Some(1)
}

/// Returns true if Windows apps prefer light theme (`AppsUseLightTheme`).
pub fn is_windows_apps_light_theme() -> bool {
    read_personalize_dword(w!("AppsUseLightTheme")) == Some(1)
}

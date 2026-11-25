//! Windows Registry Autostart Management (HKCU\Software\Microsoft\Windows\CurrentVersion\Run)

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ, REG_VALUE_TYPE,
};

const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const APP_NAME: PCWSTR = w!("WJalaliCalendar");

fn current_exe_path() -> Option<String> {
    unsafe {
        let mut buf = [0u16; 1024];
        let len = GetModuleFileNameW(None, &mut buf) as usize;
        if len == 0 || len >= buf.len() {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..len]))
    }
}

/// Returns true only if the autostart Run entry exists AND points to the current executable.
pub fn is_autostart_enabled() -> bool {
    unsafe {
        let Some(expected) = current_exe_path() else {
            return false;
        };

        let mut hkey: HKEY = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, 0, KEY_READ, &mut hkey) != ERROR_SUCCESS {
            return false;
        }

        let mut data_type = REG_VALUE_TYPE(0);
        let mut buf = [0u16; 1024];
        let mut buf_bytes = (buf.len() * 2) as u32;

        let status = RegQueryValueExW(
            hkey,
            APP_NAME,
            None,
            Some(&mut data_type),
            Some(buf.as_mut_ptr() as *mut u8),
            Some(&mut buf_bytes),
        );

        let _ = RegCloseKey(hkey);
        if status != ERROR_SUCCESS || data_type != REG_SZ || buf_bytes < 2 {
            return false;
        }

        let u16_len = (buf_bytes as usize / 2).min(buf.len());
        let raw = String::from_utf16_lossy(&buf[..u16_len]);
        let stored = raw.trim_matches(|c| c == '\0' || c == ' ' || c == '"');
        stored.eq_ignore_ascii_case(&expected)
    }
}

pub fn set_autostart(enable: bool) -> bool {
    unsafe {
        let mut hkey: HKEY = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            RUN_KEY,
            0,
            KEY_READ | KEY_WRITE,
            &mut hkey,
        ) != ERROR_SUCCESS
        {
            return false;
        }

        let result = if enable {
            let Some(path_str) = current_exe_path() else {
                let _ = RegCloseKey(hkey);
                return false;
            };
            let quoted_path = format!("\"{path_str}\"");
            let wide: Vec<u16> = quoted_path.encode_utf16().chain(std::iter::once(0)).collect();

            RegSetValueExW(
                hkey,
                APP_NAME,
                0,
                REG_SZ,
                Some(std::slice::from_raw_parts(
                    wide.as_ptr() as *const u8,
                    wide.len() * 2,
                )),
            ) == ERROR_SUCCESS
        } else {
            let status = RegDeleteValueW(hkey, APP_NAME);
            status == ERROR_SUCCESS || status.0 == 2 // ERROR_FILE_NOT_FOUND
        };

        let _ = RegCloseKey(hkey);
        result
    }
}

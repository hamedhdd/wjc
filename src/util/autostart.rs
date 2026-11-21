//! Windows Registry Autostart Management (HKCU\Software\Microsoft\Windows\CurrentVersion\Run)

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{ERROR_SUCCESS, MAX_PATH};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ, REG_VALUE_TYPE,
};

const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const APP_NAME: PCWSTR = w!("WJalaliCalendar");

pub fn is_autostart_enabled() -> bool {
    unsafe {
        let mut hkey: HKEY = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, 0, KEY_READ, &mut hkey) != ERROR_SUCCESS {
            return false;
        }

        let mut data_type = REG_VALUE_TYPE(0);
        let mut buf_size = 0u32;

        let status = RegQueryValueExW(
            hkey,
            APP_NAME,
            None,
            Some(&mut data_type as *mut _),
            None,
            Some(&mut buf_size),
        );

        let _ = RegCloseKey(hkey);
        status == ERROR_SUCCESS && buf_size > 0
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
            let mut exe_path = [0u16; MAX_PATH as usize];
            let len = GetModuleFileNameW(None, &mut exe_path);
            if len == 0 {
                let _ = RegCloseKey(hkey);
                return false;
            }

            // Wrap in quotes
            let path_str = String::from_utf16_lossy(&exe_path[..len as usize]);
            let quoted_path = format!("\"{}\"", path_str);
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
            status == ERROR_SUCCESS || status.0 == 2 // ERROR_FILE_NOT_FOUND is acceptable
        };

        let _ = RegCloseKey(hkey);
        result
    }
}

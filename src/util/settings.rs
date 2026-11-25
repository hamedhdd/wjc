//! Persisted User Preferences (HKCU\Software\WJalaliCalendar)

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_DWORD, REG_OPTION_NON_VOLATILE, REG_VALUE_TYPE,
};

const APP_REG_KEY: PCWSTR = w!("Software\\WJalaliCalendar");
const VAL_THEME_MODE: PCWSTR = w!("ThemeMode");
const VAL_TRAY_STYLE: PCWSTR = w!("TrayStyle");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreference {
    Auto = 0,
    Dark = 1,
    Light = 2,
}

impl ThemePreference {
    fn from_u32(v: u32) -> Self {
        match v {
            1 => Self::Dark,
            2 => Self::Light,
            _ => Self::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayIconStyle {
    /// Adaptive: crisp large day badge at 16px, full month+day badge at 20px+
    Auto = 0,
    /// Always render month name + day number
    MonthDay = 1,
    /// Numeric day/month format (e.g. ۲۵/۵)
    Numeric = 2,
}

impl TrayIconStyle {
    fn from_u32(v: u32) -> Self {
        match v {
            1 => Self::MonthDay,
            2 => Self::Numeric,
            _ => Self::Auto,
        }
    }
}

fn read_dword(name: PCWSTR) -> Option<u32> {
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, APP_REG_KEY, 0, KEY_READ, &mut hkey) != ERROR_SUCCESS {
            return None;
        }
        let mut ty = REG_VALUE_TYPE(0);
        let mut val: u32 = 0;
        let mut len = std::mem::size_of::<u32>() as u32;
        let ok = RegQueryValueExW(
            hkey,
            name,
            None,
            Some(&mut ty),
            Some(&mut val as *mut u32 as *mut u8),
            Some(&mut len),
        ) == ERROR_SUCCESS;
        let _ = RegCloseKey(hkey);
        (ok && ty == REG_DWORD).then_some(val)
    }
}

fn write_dword(name: PCWSTR, val: u32) {
    unsafe {
        let mut hkey = HKEY::default();
        if RegCreateKeyExW(
            HKEY_CURRENT_USER,
            APP_REG_KEY,
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        ) != ERROR_SUCCESS
        {
            return;
        }
        let bytes = val.to_le_bytes();
        let _ = RegSetValueExW(hkey, name, 0, REG_DWORD, Some(&bytes));
        let _ = RegCloseKey(hkey);
    }
}

pub fn load_theme_preference() -> ThemePreference {
    read_dword(VAL_THEME_MODE).map(ThemePreference::from_u32).unwrap_or(ThemePreference::Auto)
}

pub fn save_theme_preference(pref: ThemePreference) {
    write_dword(VAL_THEME_MODE, pref as u32);
}

pub fn load_tray_style() -> TrayIconStyle {
    read_dword(VAL_TRAY_STYLE).map(TrayIconStyle::from_u32).unwrap_or(TrayIconStyle::Auto)
}

pub fn save_tray_style(style: TrayIconStyle) {
    write_dword(VAL_TRAY_STYLE, style as u32);
}

//! Dynamic System Tray Icon Generation and Notification Area Management

use std::mem::zeroed;
use windows::Win32::Foundation::{COLORREF, HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush, DeleteDC,
    DeleteObject, DrawTextW, FillRect, GetDC, ReleaseDC, SelectObject, SetBkMode,
    SetTextColor, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, DT_CENTER,
    DT_NOCLIP, DT_SINGLELINE, DT_VCENTER, FW_BOLD, TRANSPARENT,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW, NIF_ICON,
    NIF_MESSAGE, NIF_TIP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, CreatePopupMenu, DestroyIcon, DestroyMenu, GetCursorPos,
    InsertMenuItemW, SetForegroundWindow, TrackPopupMenu, HICON, HMENU, ICONINFO,
    MENUITEMINFOW, MFS_CHECKED, MFS_DEFAULT, MFS_UNCHECKED, MIIM_ID, MIIM_STATE,
    MIIM_STRING, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD, TPM_RIGHTBUTTON,
};

use crate::calendar::{get_events_for_date, to_persian_digits, FullDateInfo, JalaliDate};
use crate::util::autostart::is_autostart_enabled;

pub const WM_TRAYICON: u32 = 0x8000 + 101; // WM_APP + 101

// Menu action IDs
pub const IDM_SHOW_CALENDAR: u32 = 1001;
pub const IDM_TODAY: u32 = 1002;
pub const IDM_AUTOSTART: u32 = 1003;
pub const IDM_ABOUT: u32 = 1004;
pub const IDM_EXIT: u32 = 1005;

pub struct TrayIcon {
    hwnd: HWND,
    current_hicon: Option<HICON>,
    last_date_key: Option<String>,
}

unsafe impl Send for TrayIcon {}
unsafe impl Sync for TrayIcon {}

impl TrayIcon {
    pub fn new(hwnd: HWND) -> Self {
        Self {
            hwnd,
            current_hicon: None,
            last_date_key: None,
        }
    }

    /// Initializes tray icon into notification area
    pub fn init(&mut self, date_info: &FullDateInfo) {
        let hicon = self.generate_icon(&date_info.jalali);
        self.current_hicon = Some(hicon);
        self.last_date_key = Some(date_info.jalali.format_tray_short());

        let tooltip = self.format_tooltip(date_info);

        let mut nid: NOTIFYICONDATAW = unsafe { zeroed() };
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = WM_TRAYICON;
        nid.hIcon = hicon;

        let wide_tip: Vec<u16> = tooltip.encode_utf16().chain(std::iter::once(0)).collect();
        let copy_len = wide_tip.len().min(nid.szTip.len() - 1);
        nid.szTip[..copy_len].copy_from_slice(&wide_tip[..copy_len]);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_ADD, &nid);
        }
    }

    /// Updates tray icon if date has changed, or forced
    pub fn update(&mut self, date_info: &FullDateInfo, force: bool) {
        let key = date_info.jalali.format_tray_short();
        if !force && self.last_date_key.as_ref() == Some(&key) {
            return;
        }

        let new_icon = self.generate_icon(&date_info.jalali);
        let tooltip = self.format_tooltip(date_info);

        let mut nid: NOTIFYICONDATAW = unsafe { zeroed() };
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_TIP;
        nid.hIcon = new_icon;

        let wide_tip: Vec<u16> = tooltip.encode_utf16().chain(std::iter::once(0)).collect();
        let copy_len = wide_tip.len().min(nid.szTip.len() - 1);
        nid.szTip[..copy_len].copy_from_slice(&wide_tip[..copy_len]);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
            if let Some(old_icon) = self.current_hicon.take() {
                let _ = DestroyIcon(old_icon);
            }
        }

        self.current_hicon = Some(new_icon);
        self.last_date_key = Some(key);
    }

    pub fn remove(&mut self) {
        let mut nid: NOTIFYICONDATAW = unsafe { zeroed() };
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.hwnd;
        nid.uID = 1;

        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
            if let Some(icon) = self.current_hicon.take() {
                let _ = DestroyIcon(icon);
            }
        }
    }

    /// Generates a crisp, high-contrast dynamic HICON with current day (e.g. 25/5 or 25)
    fn generate_icon(&self, j: &JalaliDate) -> HICON {
        unsafe {
            let width = 32i32;
            let height = 32i32;

            let hdc_screen = GetDC(None);
            let hdc_mem = CreateCompatibleDC(hdc_screen);
            let hdc_mask = CreateCompatibleDC(hdc_screen);

            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: height,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..zeroed()
                },
                ..zeroed()
            };

            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let hbm_color = windows::Win32::Graphics::Gdi::CreateDIBSection(
                hdc_screen,
                &bmi,
                DIB_RGB_COLORS,
                &mut bits,
                None,
                0,
            )
            .unwrap_or_else(|_| CreateCompatibleBitmap(hdc_screen, width, height));

            let hbm_mask = CreateCompatibleBitmap(hdc_screen, width, height);

            let old_mem_bmp = SelectObject(hdc_mem, hbm_color);
            let old_mask_bmp = SelectObject(hdc_mask, hbm_mask);

            // Determine colors: Friday / Holiday = red badge, Regular = elegant dark slate / indigo badge
            let is_friday = j.weekday().is_weekend();
            let bg_color = if is_friday {
                COLORREF(0x002B2BDC) // Crimson Red (BGR format: 0x00BBGGRR -> 0xDC, 0x2B, 0x2B)
            } else {
                COLORREF(0x00322218) // Slate Indigo (0x18, 0x22, 0x32)
            };

            // Background badge (rounded-like rectangle)
            let brush = CreateSolidBrush(bg_color);
            let rect = RECT {
                left: 1,
                top: 1,
                right: width - 1,
                bottom: height - 1,
            };
            FillRect(hdc_mem, &rect, brush);
            let _ = DeleteObject(brush);

            // Draw header bar for month / style
            let header_brush = if is_friday {
                CreateSolidBrush(COLORREF(0x001A1AB8))
            } else {
                CreateSolidBrush(COLORREF(0x004A3324))
            };
            let header_rect = RECT {
                left: 1,
                top: 1,
                right: width - 1,
                bottom: 10,
            };
            FillRect(hdc_mem, &header_rect, header_brush);
            let _ = DeleteObject(header_brush);

            SetBkMode(hdc_mem, TRANSPARENT);

            // Draw Month on top bar (e.g. "مرداد" or "5")
            let font_month = CreateFontW(
                8,
                0,
                0,
                0,
                FW_BOLD.0 as i32,
                0,
                0,
                0,
                178, // ARABIC_CHARSET
                0,
                0,
                0,
                0,
                windows::core::w!("Tahoma"),
            );
            let old_font = SelectObject(hdc_mem, font_month);
            SetTextColor(hdc_mem, COLORREF(0x00E0E0E0)); // light grey

            let month_name_fa = j.month_name();
            let mut month_wide: Vec<u16> = month_name_fa.encode_utf16().collect();
            let mut top_rect = RECT {
                left: 2,
                top: 0,
                right: width - 2,
                bottom: 10,
            };
            DrawTextW(
                hdc_mem,
                &mut month_wide,
                &mut top_rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
            );

            // Draw Day number in main body (e.g. "25" in bold Persian digits)
            let font_day = CreateFontW(
                17,
                0,
                0,
                0,
                FW_BOLD.0 as i32,
                0,
                0,
                0,
                178, // ARABIC_CHARSET
                0,
                0,
                0,
                0,
                windows::core::w!("Segoe UI"),
            );
            SelectObject(hdc_mem, font_day);
            SetTextColor(hdc_mem, COLORREF(0x00FFFFFF)); // White

            let day_fa = to_persian_digits(j.day.to_string());
            let mut day_wide: Vec<u16> = day_fa.encode_utf16().collect();
            let mut day_rect = RECT {
                left: 1,
                top: 9,
                right: width - 1,
                bottom: height - 1,
            };
            DrawTextW(
                hdc_mem,
                &mut day_wide,
                &mut day_rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
            );

            // Create mask (0 = opaque, 1 = transparent)
            let white_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
            let full_rect = RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: height,
            };
            FillRect(hdc_mask, &full_rect, white_brush);
            let _ = DeleteObject(white_brush);

            let black_brush = CreateSolidBrush(COLORREF(0x00000000));
            FillRect(hdc_mask, &rect, black_brush);
            let _ = DeleteObject(black_brush);

            // Restore objects
            SelectObject(hdc_mem, old_font);
            let _ = DeleteObject(font_month);
            let _ = DeleteObject(font_day);
            SelectObject(hdc_mem, old_mem_bmp);
            SelectObject(hdc_mask, old_mask_bmp);
            let _ = DeleteDC(hdc_mem);
            let _ = DeleteDC(hdc_mask);
            ReleaseDC(None, hdc_screen);

            let mut icon_info = ICONINFO {
                fIcon: true.into(),
                xHotspot: 0,
                yHotspot: 0,
                hbmMask: hbm_mask,
                hbmColor: hbm_color,
            };

            let hicon = CreateIconIndirect(&mut icon_info).unwrap_or(HICON::default());

            let _ = DeleteObject(hbm_color);
            let _ = DeleteObject(hbm_mask);

            hicon
        }
    }

    /// Formats tooltip text for tray icon hover
    fn format_tooltip(&self, d: &FullDateInfo) -> String {
        let weekday_name = d.jalali.weekday().name_fa();
        let events = get_events_for_date(&d.jalali, &d.lunar);
        let event_summary = if let Some(e) = events.first() {
            format!("\n{}", e.title)
        } else {
            String::new()
        };

        format!(
            "{} {}\n{} / {}{}",
            weekday_name,
            d.jalali.format_long_fa(),
            d.gregorian.format_long(),
            d.lunar.format_long_fa(),
            event_summary
        )
    }

    /// Displays the right-click context menu
    pub fn show_context_menu(&self) -> u32 {
        unsafe {
            let hmenu = CreatePopupMenu().unwrap_or(HMENU::default());
            if hmenu.is_invalid() {
                return 0;
            }

            let autostart = is_autostart_enabled();

            let add_item = |id: u32, text: &str, checked: bool, is_default: bool| {
                let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
                let mut state = if checked { MFS_CHECKED } else { MFS_UNCHECKED };
                if is_default {
                    state |= MFS_DEFAULT;
                }
                let item = MENUITEMINFOW {
                    cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32,
                    fMask: MIIM_ID | MIIM_STRING | MIIM_STATE,
                    wID: id,
                    dwTypeData: windows::core::PWSTR(wide.as_ptr() as *mut u16),
                    fState: state,
                    ..zeroed()
                };
                let _ = InsertMenuItemW(hmenu, id, false, &item);
            };

            add_item(IDM_SHOW_CALENDAR, "📅 نمایش تقویم (Show Calendar)", false, true);
            add_item(IDM_TODAY, "🎯 رفتن به امروز (Go to Today)", false, false);
            add_item(
                IDM_AUTOSTART,
                "🚀 اجرا هنگام بالا آمدن ویندوز (Start with Windows)",
                autostart,
                false,
            );
            add_item(IDM_ABOUT, "ℹ️ درباره (About)", false, false);
            add_item(IDM_EXIT, "❌ خروج (Exit)", false, false);

            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);

            // Set foreground window to handle menu dismiss properly
            let _ = SetForegroundWindow(self.hwnd);

            let cmd = TrackPopupMenu(
                hmenu,
                TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_BOTTOMALIGN | TPM_LEFTALIGN,
                pt.x,
                pt.y,
                0,
                self.hwnd,
                None,
            );

            let _ = DestroyMenu(hmenu);
            cmd.0 as u32
        }
    }
}

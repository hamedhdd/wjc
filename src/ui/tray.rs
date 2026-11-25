//! Dynamic System Tray Icon Generation and Notification Area Management

use std::mem::zeroed;
use windows::Win32::Foundation::{COLORREF, HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush, DeleteDC,
    DeleteObject, DrawTextW, FillRect, FrameRect, GetDC, ReleaseDC, SelectObject, SetBkMode,
    SetTextColor, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, DT_CENTER,
    DT_NOCLIP, DT_SINGLELINE, DT_VCENTER, FW_BOLD, TRANSPARENT,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW, NIF_ICON,
    NIF_MESSAGE, NIF_TIP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, CreatePopupMenu, DestroyIcon, DestroyMenu, GetCursorPos,
    GetSystemMetrics, InsertMenuItemW, SetForegroundWindow, TrackPopupMenu, HICON, HMENU,
    ICONINFO, MENUITEMINFOW, MFS_CHECKED, MFS_DEFAULT, MFS_UNCHECKED, MIIM_ID, MIIM_STATE,
    MIIM_STRING, SM_CXSMICON, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD,
    TPM_RIGHTBUTTON,
};

use crate::calendar::{get_events_for_date, is_official_holiday, to_persian_digits, FullDateInfo};
use crate::util::autostart::is_autostart_enabled;
use crate::util::theme_detect::is_windows_system_light_theme;

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
    last_light_theme: bool,
}

unsafe impl Send for TrayIcon {}
unsafe impl Sync for TrayIcon {}

impl TrayIcon {
    pub fn new(hwnd: HWND) -> Self {
        Self {
            hwnd,
            current_hicon: None,
            last_date_key: None,
            last_light_theme: false,
        }
    }

    /// Initializes tray icon into notification area
    pub fn init(&mut self, date_info: &FullDateInfo) {
        let is_light = is_windows_system_light_theme();
        let hicon = self.generate_icon(date_info, is_light);
        self.current_hicon = Some(hicon);
        self.last_date_key = Some(date_info.jalali.format_tray_short());
        self.last_light_theme = is_light;

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

    /// Updates tray icon if date has changed, theme has changed, or forced
    pub fn update(&mut self, date_info: &FullDateInfo, force: bool) {
        let key = date_info.jalali.format_tray_short();
        let is_light = is_windows_system_light_theme();
        let theme_changed = self.last_light_theme != is_light;

        if !force && !theme_changed && self.last_date_key.as_ref() == Some(&key) {
            return;
        }

        let new_icon = self.generate_icon(date_info, is_light);
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
        self.last_light_theme = is_light;
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

    /// Generates a crisp, high-contrast, theme-aware dynamic HICON.
    ///
    /// Design characteristics:
    /// - Maximize typography size for instant readability at 16x16, 24x24, and 32x32.
    /// - Header: month name rendered in clean, condensed typography with contrasting header band.
    /// - Body: extra-large, bold Persian day digits occupying maximum available vertical & horizontal space.
    /// - Theme adaptive: calibrated for Windows Dark Taskbars (charcoal/slate base with crisp white text)
    ///   and Light Taskbars (pure white badge with subtle boundary and deep charcoal text).
    /// - Holiday / Friday: prominent Carmine Red header and badge tint.
    fn generate_icon(&self, date_info: &FullDateInfo, is_light_taskbar: bool) -> HICON {
        unsafe {
            // Respect system small icon metric, clamped to at least 32 for crisp downsampling
            let sys_sm = GetSystemMetrics(SM_CXSMICON);
            let icon_dim = if sys_sm > 32 { sys_sm } else { 32 };
            let width = icon_dim;
            let height = icon_dim;

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

            let j = &date_info.jalali;
            let is_holiday = is_official_holiday(j, &date_info.lunar);

            // BGR color constants for Windows GDI (0x00BBGGRR)
            let (bg_color, border_color, header_bg, month_text_color, day_text_color) = if is_holiday {
                if is_light_taskbar {
                    // Holiday on light taskbar: Soft light rose body, vivid carmine header, deep red day text
                    (
                        COLORREF(0x00EFE8FF), // Light crimson tint
                        COLORREF(0x002B2BE0), // Border red
                        COLORREF(0x002B2BDC), // Header carmine (0xDC, 0x2B, 0x2B)
                        COLORREF(0x00FFFFFF), // Header text pure white
                        COLORREF(0x001B1BB8), // Bold red day number
                    )
                } else {
                    // Holiday on dark taskbar: Dark crimson slate body, bright carmine header, white day text
                    (
                        COLORREF(0x00221A30), // Deep crimson body
                        COLORREF(0x0033264A), // Subtle border
                        COLORREF(0x002A2BE0), // Header carmine
                        COLORREF(0x00FFFFFF), // Header text pure white
                        COLORREF(0x00FFFFFF), // Day number crisp white
                    )
                }
            } else if is_light_taskbar {
                // Regular day on Light Windows Taskbar:
                // Clean crisp white badge with clear contrast border, deep blue/charcoal header, dark charcoal numbers
                (
                    COLORREF(0x00FFFFFF), // Crisp pure white body
                    COLORREF(0x009E968E), // Clear outline border so it stands out against light grey/white taskbars
                    COLORREF(0x00665544), // Neutral dark slate header band
                    COLORREF(0x00FFFFFF), // White text on header band
                    COLORREF(0x00181410), // Deep rich charcoal day number
                )
            } else {
                // Regular day on Dark Windows Taskbar:
                // Deep graphite slate body, subtle hairline border, dark neutral header band, bright white day number
                (
                    COLORREF(0x0026201E), // Slate graphite body
                    COLORREF(0x00423A36), // Hairline border
                    COLORREF(0x0052443C), // Muted dark header
                    COLORREF(0x00EDE8E4), // Light grey header text
                    COLORREF(0x00FFFFFF), // Pure white day number
                )
            };

            // 1. Draw rounded badge background
            let brush = CreateSolidBrush(bg_color);
            let badge_rect = RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: height,
            };
            FillRect(hdc_mem, &badge_rect, brush);
            let _ = DeleteObject(brush);

            // 2. Header Bar for Month Name
            // Calibrated header height: ~34% of icon height to give month name ample room
            let header_h = ((height as f32) * 0.34).round() as i32;
            let header_brush = CreateSolidBrush(header_bg);
            let header_rect = RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: header_h,
            };
            FillRect(hdc_mem, &header_rect, header_brush);
            let _ = DeleteObject(header_brush);

            // 3. Crisp outline border around entire badge
            let border_brush = CreateSolidBrush(border_color);
            FrameRect(hdc_mem, &badge_rect, border_brush);
            let _ = DeleteObject(border_brush);

            SetBkMode(hdc_mem, TRANSPARENT);

            // 4. Render Month Name
            // Select optimal font size for month name
            let month_font_size = if height >= 32 { 10 } else { 8 };
            let font_month = CreateFontW(
                month_font_size,
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
            let old_font = SelectObject(hdc_mem, font_month);
            SetTextColor(hdc_mem, month_text_color);

            let month_name_fa = j.month_name();
            let mut month_wide: Vec<u16> = month_name_fa.encode_utf16().collect();
            let mut top_rect = RECT {
                left: 1,
                top: 0,
                right: width - 1,
                bottom: header_h,
            };
            DrawTextW(
                hdc_mem,
                &mut month_wide,
                &mut top_rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
            );

            // 5. Render Extra-Large Day Number
            // Maximize day number font height to fit the remaining body space cleanly
            let day_font_size = if height >= 32 { 21 } else { 13 };
            let font_day = CreateFontW(
                day_font_size,
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
            SetTextColor(hdc_mem, day_text_color);

            let day_fa = to_persian_digits(j.day.to_string());
            let mut day_wide: Vec<u16> = day_fa.encode_utf16().collect();
            let mut day_rect = RECT {
                left: 1,
                top: header_h - 1,
                right: width - 1,
                bottom: height,
            };
            DrawTextW(
                hdc_mem,
                &mut day_wide,
                &mut day_rect,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
            );

            // 6. Create 1-bit transparency mask (0 = opaque, 1 = transparent)
            // Outer rectangle is opaque (0), slight corner rounding masked out (1)
            let black_brush = CreateSolidBrush(COLORREF(0x00000000));
            FillRect(hdc_mask, &badge_rect, black_brush);
            let _ = DeleteObject(black_brush);

            // Restore GDI objects
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

            add_item(IDM_SHOW_CALENDAR, "نمایش تقویم (Show Calendar)", false, true);
            add_item(IDM_TODAY, "امروز (Today)", false, false);
            add_item(
                IDM_AUTOSTART,
                "اجرا هنگام بالا آمدن ویندوز (Start with Windows)",
                autostart,
                false,
            );
            add_item(IDM_ABOUT, "درباره (About)", false, false);
            add_item(IDM_EXIT, "خروج (Exit)", false, false);

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

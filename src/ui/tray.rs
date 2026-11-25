//! Dynamic System Tray Icon Generation and Notification Area Management
//!
//! Renders pixel-exact icons at the native `SM_CXSMICON` resolution with proper
//! 32-bit premultiplied alpha, a genuine 1-bit monochrome mask, high-contrast
//! palettes for both Windows Dark and Light taskbars, and selectable styles.

use std::mem::zeroed;
use windows::Win32::Foundation::{COLORREF, HWND, POINT, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    CreateBitmap, CreateCompatibleDC, CreateDIBSection, CreateFontW, CreateSolidBrush, DeleteDC,
    DeleteObject, DrawTextW, FillRect, FrameRect, GdiFlush, GetDC, GetTextExtentPoint32W,
    ReleaseDC, SelectObject, SetBkMode, SetTextColor, ANTIALIASED_QUALITY, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, DT_CENTER, DT_NOCLIP, DT_SINGLELINE, DT_VCENTER,
    FW_BOLD, HDC, NONANTIALIASED_QUALITY, TRANSPARENT,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, CreatePopupMenu, DestroyIcon, DestroyMenu, GetCursorPos, GetSystemMetrics,
    InsertMenuItemW, SetForegroundWindow, TrackPopupMenu, HICON, HMENU, ICONINFO, MENUITEMINFOW,
    MFS_CHECKED, MFS_DEFAULT, MFS_UNCHECKED, MFT_SEPARATOR, MIIM_FTYPE, MIIM_ID, MIIM_STATE,
    MIIM_STRING, MIIM_SUBMENU, SM_CXSMICON, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD,
    TPM_RIGHTBUTTON,
};

use crate::calendar::{get_events_for_date, is_official_holiday, to_persian_digits, FullDateInfo};
use crate::ui::theme::rgb;
use crate::util::autostart::is_autostart_enabled;
use crate::util::settings::{load_tray_style, save_tray_style, TrayIconStyle};
use crate::util::theme_detect::is_windows_system_light_theme;

pub const WM_TRAYICON: u32 = 0x8000 + 101; // WM_APP + 101

// Menu action IDs
pub const IDM_SHOW_CALENDAR: u32 = 1001;
pub const IDM_TODAY: u32 = 1002;
pub const IDM_AUTOSTART: u32 = 1003;
pub const IDM_ABOUT: u32 = 1004;
pub const IDM_EXIT: u32 = 1005;
pub const IDM_STYLE_AUTO: u32 = 1010;
pub const IDM_STYLE_MONTH_DAY: u32 = 1011;
pub const IDM_STYLE_NUMERIC: u32 = 1012;
pub const IDM_RELOAD_HOLIDAYS: u32 = 1015;

pub struct TrayIcon {
    hwnd: HWND,
    current_hicon: Option<HICON>,
    last_date_key: Option<String>,
    last_light_theme: bool,
    last_icon_dim: i32,
    style: TrayIconStyle,
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
            last_icon_dim: 0,
            style: load_tray_style(),
        }
    }

    pub fn set_style(&mut self, style: TrayIconStyle, date_info: &FullDateInfo) {
        self.style = style;
        save_tray_style(style);
        self.update(date_info, true);
    }

    /// Initializes tray icon into notification area.
    pub fn init(&mut self, date_info: &FullDateInfo) {
        self.add_or_readd(date_info);
    }

    /// Re-registers the tray icon after `explorer.exe` restarts (`TaskbarCreated`).
    pub fn readd(&mut self, date_info: &FullDateInfo) {
        self.add_or_readd(date_info);
    }

    fn add_or_readd(&mut self, date_info: &FullDateInfo) {
        let is_light = is_windows_system_light_theme();
        let dim = current_tray_icon_dim();
        let hicon = self.generate_icon(date_info, is_light, dim);

        let tooltip = self.format_tooltip(date_info);
        let nid = self.build_nid(hicon, &tooltip, NIF_ICON | NIF_MESSAGE | NIF_TIP);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_ADD, &nid);
            if let Some(old_icon) = self.current_hicon.replace(hicon) {
                let _ = DestroyIcon(old_icon);
            }
        }

        self.last_date_key = Some(date_info.jalali.format_tray_short());
        self.last_light_theme = is_light;
        self.last_icon_dim = dim;
    }

    /// Updates tray icon if date, taskbar theme, icon DPI, or style changed (or forced).
    pub fn update(&mut self, date_info: &FullDateInfo, force: bool) {
        let key = date_info.jalali.format_tray_short();
        let is_light = is_windows_system_light_theme();
        let dim = current_tray_icon_dim();

        let unchanged = self.last_date_key.as_ref() == Some(&key)
            && self.last_light_theme == is_light
            && self.last_icon_dim == dim;
        if !force && unchanged {
            return;
        }

        let new_icon = self.generate_icon(date_info, is_light, dim);
        let tooltip = self.format_tooltip(date_info);
        let nid = self.build_nid(new_icon, &tooltip, NIF_ICON | NIF_TIP);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
            if let Some(old_icon) = self.current_hicon.replace(new_icon) {
                let _ = DestroyIcon(old_icon);
            }
        }

        self.last_date_key = Some(key);
        self.last_light_theme = is_light;
        self.last_icon_dim = dim;
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

    fn build_nid(
        &self,
        hicon: HICON,
        tooltip: &str,
        flags: windows::Win32::UI::Shell::NOTIFY_ICON_DATA_FLAGS,
    ) -> NOTIFYICONDATAW {
        let mut nid: NOTIFYICONDATAW = unsafe { zeroed() };
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.hwnd;
        nid.uID = 1;
        nid.uFlags = flags;
        nid.uCallbackMessage = WM_TRAYICON;
        nid.hIcon = hicon;

        let wide_tip: Vec<u16> = tooltip.encode_utf16().chain(std::iter::once(0)).collect();
        let copy_len = wide_tip.len().min(nid.szTip.len() - 1);
        nid.szTip[..copy_len].copy_from_slice(&wide_tip[..copy_len]);
        nid
    }

    /// Generates a pixel-exact, high-contrast, theme-adaptive HICON at `dim x dim`.
    fn generate_icon(&self, date_info: &FullDateInfo, is_light_taskbar: bool, dim: i32) -> HICON {
        unsafe {
            let width = dim;
            let height = dim;

            let hdc_screen = GetDC(None);
            let hdc_mem = CreateCompatibleDC(hdc_screen);

            // Top-down 32-bit BGRA DIBSection
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..zeroed()
                },
                ..zeroed()
            };

            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let Ok(hbm_color) =
                CreateDIBSection(hdc_screen, &bmi, DIB_RGB_COLORS, &mut bits, None, 0)
            else {
                let _ = DeleteDC(hdc_mem);
                ReleaseDC(None, hdc_screen);
                return HICON::default();
            };

            let old_mem_bmp = SelectObject(hdc_mem, hbm_color);

            let j = &date_info.jalali;
            let is_holiday = is_official_holiday(j, &date_info.lunar);
            let palette = tray_palette(is_holiday, is_light_taskbar);

            let full_rect = RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: height,
            };

            // 1. Fill badge body
            let bg_brush = CreateSolidBrush(palette.body);
            FillRect(hdc_mem, &full_rect, bg_brush);
            let _ = DeleteObject(bg_brush);

            SetBkMode(hdc_mem, TRANSPARENT);
            let quality = if dim <= 16 {
                NONANTIALIASED_QUALITY.0 as u32
            } else {
                ANTIALIASED_QUALITY.0 as u32
            };

            let day_fa = to_persian_digits(j.day.to_string());
            let month_fa = j.month_name();

            // 2. Render style-specific header + typography
            let effective_style = match self.style {
                TrayIconStyle::Auto if dim <= 18 => TrayRenderMode::DayBadge,
                TrayIconStyle::Auto | TrayIconStyle::MonthDay => TrayRenderMode::MonthAndDay,
                TrayIconStyle::Numeric => TrayRenderMode::Numeric,
            };

            match effective_style {
                TrayRenderMode::DayBadge => {
                    // Crisp top accent bar (3px at 16px) + huge bold day numeral filling body
                    let hdr_h = (height as f32 * 0.22).round().max(3.0) as i32;
                    let hdr_rect = RECT {
                        left: 0,
                        top: 0,
                        right: width,
                        bottom: hdr_h,
                    };
                    let hb = CreateSolidBrush(palette.header);
                    FillRect(hdc_mem, &hdr_rect, hb);
                    let _ = DeleteObject(hb);

                    let day_font_h = ((height - hdr_h) as f32 * 0.96).round() as i32;
                    let font_day = CreateFontW(
                        -day_font_h,
                        0,
                        0,
                        0,
                        FW_BOLD.0 as i32,
                        0,
                        0,
                        0,
                        178,
                        0,
                        0,
                        quality,
                        0,
                        windows::core::w!("Tahoma"),
                    );
                    let old_f = SelectObject(hdc_mem, font_day);
                    SetTextColor(hdc_mem, palette.day_text);
                    let mut dr = RECT {
                        left: 0,
                        top: hdr_h - 1,
                        right: width,
                        bottom: height,
                    };
                    let mut wide: Vec<u16> = day_fa.encode_utf16().collect();
                    DrawTextW(
                        hdc_mem,
                        &mut wide,
                        &mut dr,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
                    );
                    SelectObject(hdc_mem, old_f);
                    let _ = DeleteObject(font_day);
                }
                TrayRenderMode::MonthAndDay => {
                    let hdr_h = ((height as f32) * 0.38).round() as i32;
                    let hdr_rect = RECT {
                        left: 0,
                        top: 0,
                        right: width,
                        bottom: hdr_h,
                    };
                    let hb = CreateSolidBrush(palette.header);
                    FillRect(hdc_mem, &hdr_rect, hb);
                    let _ = DeleteObject(hb);

                    // Fit month text cleanly inside header width
                    let max_m_h = (hdr_h as f32 * 0.88).round().max(6.0) as i32;
                    let month_font = fit_font_to_width(
                        hdc_mem,
                        month_fa,
                        width - 2,
                        max_m_h,
                        6,
                        quality,
                        windows::core::w!("Tahoma"),
                    );
                    let old_f = SelectObject(hdc_mem, month_font);
                    SetTextColor(hdc_mem, palette.month_text);
                    let mut mr = RECT {
                        left: 0,
                        top: 0,
                        right: width,
                        bottom: hdr_h,
                    };
                    let mut m_wide: Vec<u16> = month_fa.encode_utf16().collect();
                    DrawTextW(
                        hdc_mem,
                        &mut m_wide,
                        &mut mr,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
                    );
                    SelectObject(hdc_mem, old_f);
                    let _ = DeleteObject(month_font);

                    // Bold day number below header
                    let day_font_h = ((height - hdr_h) as f32 * 0.90).round() as i32;
                    let font_day = CreateFontW(
                        -day_font_h,
                        0,
                        0,
                        0,
                        FW_BOLD.0 as i32,
                        0,
                        0,
                        0,
                        178,
                        0,
                        0,
                        quality,
                        0,
                        windows::core::w!("Tahoma"),
                    );
                    let old_f = SelectObject(hdc_mem, font_day);
                    SetTextColor(hdc_mem, palette.day_text);
                    let mut dr = RECT {
                        left: 0,
                        top: hdr_h,
                        right: width,
                        bottom: height,
                    };
                    let mut d_wide: Vec<u16> = day_fa.encode_utf16().collect();
                    DrawTextW(
                        hdc_mem,
                        &mut d_wide,
                        &mut dr,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
                    );
                    SelectObject(hdc_mem, old_f);
                    let _ = DeleteObject(font_day);
                }
                TrayRenderMode::Numeric => {
                    let hdr_h = (height / 8).max(2);
                    let hdr_rect = RECT {
                        left: 0,
                        top: 0,
                        right: width,
                        bottom: hdr_h,
                    };
                    let hb = CreateSolidBrush(palette.header);
                    FillRect(hdc_mem, &hdr_rect, hb);
                    let _ = DeleteObject(hb);

                    let label = to_persian_digits(format!("{}/{}", j.day, j.month));
                    let max_h = ((height - hdr_h) as f32 * 0.85).round() as i32;
                    let font_num = fit_font_to_width(
                        hdc_mem,
                        &label,
                        width - 2,
                        max_h,
                        7,
                        quality,
                        windows::core::w!("Tahoma"),
                    );
                    let old_f = SelectObject(hdc_mem, font_num);
                    SetTextColor(hdc_mem, palette.day_text);
                    let mut dr = RECT {
                        left: 0,
                        top: hdr_h,
                        right: width,
                        bottom: height,
                    };
                    let mut wide: Vec<u16> = label.encode_utf16().collect();
                    DrawTextW(
                        hdc_mem,
                        &mut wide,
                        &mut dr,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
                    );
                    SelectObject(hdc_mem, old_f);
                    let _ = DeleteObject(font_num);
                }
            }

            // 3. Subtle outline border so icon stands out on light/dark taskbars
            if let Some(border_color) = palette.border {
                let border_brush = CreateSolidBrush(border_color);
                FrameRect(hdc_mem, &full_rect, border_brush);
                let _ = DeleteObject(border_brush);
            }

            // 4. Flush GDI and fix up 32-bit ARGB alpha channel (GDI leaves alpha = 0x00)
            let _ = GdiFlush();
            if !bits.is_null() {
                let pixels =
                    std::slice::from_raw_parts_mut(bits as *mut u32, (width * height) as usize);
                for px in pixels.iter_mut() {
                    *px |= 0xFF00_0000;
                }
                // Round the 4 outer corner pixels transparently
                let w = width as usize;
                let h = height as usize;
                pixels[0] = 0;
                pixels[w - 1] = 0;
                pixels[(h - 1) * w] = 0;
                pixels[h * w - 1] = 0;
            }

            // 5. Genuine 1-bit monochrome AND mask (0 = opaque, 1 = transparent)
            let stride_bytes = (((width + 15) / 16) * 2) as usize;
            let mut mask_bits = vec![0u8; stride_bytes * height as usize];
            // Mark the 4 corner pixels transparent in the 1-bit mask too
            let last_col = (width - 1) as usize;
            let set_mask_bit = |buf: &mut [u8], x: usize, y: usize| {
                let byte_idx = y * stride_bytes + (x / 8);
                let bit_mask = 0x80u8 >> (x % 8);
                buf[byte_idx] |= bit_mask;
            };
            set_mask_bit(&mut mask_bits, 0, 0);
            set_mask_bit(&mut mask_bits, last_col, 0);
            set_mask_bit(&mut mask_bits, 0, (height - 1) as usize);
            set_mask_bit(&mut mask_bits, last_col, (height - 1) as usize);

            let hbm_mask = CreateBitmap(
                width,
                height,
                1,
                1,
                Some(mask_bits.as_ptr() as *const std::ffi::c_void),
            );

            // Detach bitmap from DC before CreateIconIndirect
            SelectObject(hdc_mem, old_mem_bmp);
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(None, hdc_screen);

            let icon_info = ICONINFO {
                fIcon: true.into(),
                xHotspot: 0,
                yHotspot: 0,
                hbmMask: hbm_mask,
                hbmColor: hbm_color,
            };

            let hicon = CreateIconIndirect(&icon_info).unwrap_or_default();

            let _ = DeleteObject(hbm_color);
            let _ = DeleteObject(hbm_mask);

            hicon
        }
    }

    /// Formats tooltip text for tray icon hover.
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

    /// Displays the right-click context menu.
    pub fn show_context_menu(&self) -> u32 {
        unsafe {
            let hmenu = CreatePopupMenu().unwrap_or_default();
            if hmenu.is_invalid() {
                return 0;
            }

            let autostart = is_autostart_enabled();

            let add_item = |menu: HMENU, id: u32, text: &str, checked: bool, is_default: bool| {
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
                let _ = InsertMenuItemW(menu, u32::MAX, true, &item);
            };

            let add_sep = |menu: HMENU| {
                let item = MENUITEMINFOW {
                    cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32,
                    fMask: MIIM_FTYPE,
                    fType: MFT_SEPARATOR,
                    ..zeroed()
                };
                let _ = InsertMenuItemW(menu, u32::MAX, true, &item);
            };

            add_item(hmenu, IDM_SHOW_CALENDAR, "نمایش تقویم (Show Calendar)", false, true);
            add_item(hmenu, IDM_TODAY, "امروز (Today)", false, false);
            add_sep(hmenu);

            // Tray icon style submenu
            let style_menu = CreatePopupMenu().unwrap_or_default();
            if !style_menu.is_invalid() {
                add_item(
                    style_menu,
                    IDM_STYLE_AUTO,
                    "خودکار (پیشنهادی)",
                    self.style == TrayIconStyle::Auto,
                    false,
                );
                add_item(
                    style_menu,
                    IDM_STYLE_MONTH_DAY,
                    "نام ماه و روز",
                    self.style == TrayIconStyle::MonthDay,
                    false,
                );
                add_item(
                    style_menu,
                    IDM_STYLE_NUMERIC,
                    "عددی (۲۵/۵)",
                    self.style == TrayIconStyle::Numeric,
                    false,
                );

                let wide: Vec<u16> = "نمای آیکون (Tray Style)"
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();
                let sub_item = MENUITEMINFOW {
                    cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32,
                    fMask: MIIM_STRING | MIIM_SUBMENU,
                    hSubMenu: style_menu,
                    dwTypeData: windows::core::PWSTR(wide.as_ptr() as *mut u16),
                    ..zeroed()
                };
                let _ = InsertMenuItemW(hmenu, u32::MAX, true, &sub_item);
            }

            add_item(
                hmenu,
                IDM_RELOAD_HOLIDAYS,
                "بارگذاری مجدد تعطیلات (Reload Holidays)",
                false,
                false,
            );
            add_item(
                hmenu,
                IDM_AUTOSTART,
                "اجرا هنگام بالا آمدن ویندوز (Start with Windows)",
                autostart,
                false,
            );
            add_sep(hmenu);
            add_item(hmenu, IDM_ABOUT, "درباره (About)", false, false);
            add_item(hmenu, IDM_EXIT, "خروج (Exit)", false, false);

            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);
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

#[derive(Clone, Copy)]
enum TrayRenderMode {
    DayBadge,
    MonthAndDay,
    Numeric,
}

struct TrayPalette {
    header: COLORREF,
    body: COLORREF,
    border: Option<COLORREF>,
    month_text: COLORREF,
    day_text: COLORREF,
}

fn tray_palette(is_holiday: bool, is_light_taskbar: bool) -> TrayPalette {
    if is_holiday {
        if is_light_taskbar {
            TrayPalette {
                header: rgb(220, 38, 38),
                body: rgb(255, 241, 242),
                border: Some(rgb(185, 28, 28)),
                month_text: rgb(255, 255, 255),
                day_text: rgb(185, 28, 28),
            }
        } else {
            TrayPalette {
                header: rgb(220, 38, 38),
                body: rgb(48, 20, 24),
                border: Some(rgb(239, 68, 68)),
                month_text: rgb(255, 255, 255),
                day_text: rgb(255, 255, 255),
            }
        }
    } else if is_light_taskbar {
        TrayPalette {
            header: rgb(79, 70, 229), // Royal Indigo
            body: rgb(255, 255, 255),
            border: Some(rgb(100, 116, 139)),
            month_text: rgb(255, 255, 255),
            day_text: rgb(15, 23, 42),
        }
    } else {
        // Dark taskbar: crisp slate body with vivid indigo header & pure white numerals
        TrayPalette {
            header: rgb(99, 102, 241), // Vivid Indigo
            body: rgb(36, 40, 52),
            border: Some(rgb(100, 116, 139)),
            month_text: rgb(255, 255, 255),
            day_text: rgb(255, 255, 255),
        }
    }
}

fn current_tray_icon_dim() -> i32 {
    let sm = unsafe { GetSystemMetrics(SM_CXSMICON) };
    sm.clamp(16, 64)
}

unsafe fn fit_font_to_width(
    hdc: HDC,
    text: &str,
    max_width: i32,
    max_h: i32,
    min_h: i32,
    quality: u32,
    face: windows::core::PCWSTR,
) -> windows::Win32::Graphics::Gdi::HFONT {
    let wide: Vec<u16> = text.encode_utf16().collect();
    let mut h = max_h;
    while h > min_h {
        let font = CreateFontW(
            -h,
            0,
            0,
            0,
            FW_BOLD.0 as i32,
            0,
            0,
            0,
            178,
            0,
            0,
            quality,
            0,
            face,
        );
        let old = SelectObject(hdc, font);
        let mut sz = SIZE::default();
        let _ = GetTextExtentPoint32W(hdc, &wide, &mut sz);
        SelectObject(hdc, old);
        if sz.cx <= max_width {
            return font;
        }
        let _ = DeleteObject(font);
        h -= 1;
    }

    CreateFontW(
        -min_h,
        0,
        0,
        0,
        FW_BOLD.0 as i32,
        0,
        0,
        0,
        178,
        0,
        0,
        quality,
        0,
        face,
    )
}

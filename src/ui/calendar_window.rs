//! Full-featured Persian / Jalali Calendar Window Popup

use std::sync::Mutex;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreatePen,
    CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint, FillRect, FrameRect,
    InvalidateRect, MoveToEx, PS_SOLID, RoundRect, SelectObject, SetBkMode, SetTextColor,
    DT_CENTER, DT_LEFT, DT_NOCLIP, DT_RIGHT, DT_SINGLELINE, DT_TOP, DT_VCENTER,
    DT_WORDBREAK, FW_BOLD, FW_NORMAL, FW_SEMIBOLD, HBRUSH, HDC, PAINTSTRUCT,
    TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VK_ESCAPE, VK_HOME, VK_NEXT, VK_PRIOR,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetClientRect, GetCursorPos, GetSystemMetrics,
    RegisterClassExW, SetForegroundWindow, SetWindowPos, ShowWindow, CS_DROPSHADOW,
    CS_HREDRAW, CS_VREDRAW, HCURSOR, HICON, HWND_TOPMOST, SWP_NOACTIVATE,
    SWP_SHOWWINDOW, SW_HIDE, SW_SHOW, SYSTEM_METRICS_INDEX, WM_ACTIVATE,
    WM_ERASEBKGND, WM_KEYDOWN, WM_KILLFOCUS, WM_LBUTTONDOWN, WM_MOUSEMOVE,
    WM_PAINT, WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

use crate::calendar::{
    get_events_for_date, is_official_holiday, to_persian_digits,
    GregorianDate, JalaliDate, LunarDate, JALALI_MONTH_NAMES,
};
use crate::ui::theme::Theme;
use crate::util::dpi::{get_dpi_for_window_or_system, scale_for_dpi};
use crate::util::time::get_current_jalali;
use crate::util::SendHwnd;

const CALENDAR_CLASS: PCWSTR = w!("WJalaliCalendarPopupClass");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Calendar,
    Converter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonId {
    PrevMonth,
    NextMonth,
    PrevYear,
    NextYear,
    Today,
    ThemeToggle,
    _ConverterToggle,
}

pub struct CalendarWindowState {
    pub is_visible: bool,
    pub view_mode: ViewMode,
    pub theme: Theme,
    pub today_jalali: JalaliDate,
    pub viewing_year: i32,
    pub viewing_month: u32,
    pub selected_date: JalaliDate,
    pub hovered_day_index: Option<usize>,
    pub hovered_button: Option<ButtonId>,
    pub dpi: u32,
}

static GLOBAL_STATE: Mutex<Option<CalendarWindowState>> = Mutex::new(None);
static GLOBAL_HWND: Mutex<Option<SendHwnd>> = Mutex::new(None);

pub fn init_calendar_window() -> HWND {
    unsafe {
        let instance = GetModuleHandleW(None).unwrap();

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_DROPSHADOW,
            lpfnWndProc: Some(calendar_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: HICON::default(),
            hCursor: HCURSOR::default(),
            hbrBackground: HBRUSH::default(),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: CALENDAR_CLASS,
            hIconSm: HICON::default(),
        };

        let _ = RegisterClassExW(&wnd_class);

        let today = get_current_jalali();

        {
            let mut state_guard = GLOBAL_STATE.lock().unwrap();
            *state_guard = Some(CalendarWindowState {
                is_visible: false,
                view_mode: ViewMode::Calendar,
                theme: Theme::dark(),
                today_jalali: today,
                viewing_year: today.year,
                viewing_month: today.month,
                selected_date: today,
                hovered_day_index: None,
                hovered_button: None,
                dpi: 96,
            });
        }

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            CALENDAR_CLASS,
            w!("تقویم خورشیدی"),
            WS_POPUP,
            0,
            0,
            420,
            580,
            None,
            None,
            instance,
            None,
        )
        .unwrap_or(HWND::default());

        {
            let mut hwnd_guard = GLOBAL_HWND.lock().unwrap();
            *hwnd_guard = Some(SendHwnd(hwnd));
        }

        let dpi = get_dpi_for_window_or_system(hwnd);
        {
            let mut state_guard = GLOBAL_STATE.lock().unwrap();
            if let Some(state) = state_guard.as_mut() {
                state.dpi = dpi;
            }
        }

        hwnd
    }
}

pub fn toggle_calendar_window() {
    let state_guard = GLOBAL_STATE.lock().unwrap();
    if let Some(state) = state_guard.as_ref() {
        if state.is_visible {
            drop(state_guard);
            hide_calendar_window();
        } else {
            drop(state_guard);
            show_calendar_window();
        }
    }
}

pub fn show_calendar_window() {
    let hwnd_opt = { *GLOBAL_HWND.lock().unwrap() };
    if let Some(send_hwnd) = hwnd_opt {
        let hwnd = send_hwnd.get();
        let today = get_current_jalali();
        {
            let mut state_guard = GLOBAL_STATE.lock().unwrap();
            if let Some(state) = state_guard.as_mut() {
                state.today_jalali = today;
                state.is_visible = true;
            }
        }

        position_window_near_tray(hwnd);
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetForegroundWindow(hwnd);
            let _ = InvalidateRect(hwnd, None, true);
        }
    }
}

pub fn hide_calendar_window() {
    let hwnd_opt = { *GLOBAL_HWND.lock().unwrap() };
    if let Some(send_hwnd) = hwnd_opt {
        let hwnd = send_hwnd.get();
        {
            let mut state_guard = GLOBAL_STATE.lock().unwrap();
            if let Some(state) = state_guard.as_mut() {
                state.is_visible = false;
            }
        }
        unsafe {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
    }
}

pub fn go_to_today() {
    let today = get_current_jalali();
    {
        let mut state_guard = GLOBAL_STATE.lock().unwrap();
        if let Some(state) = state_guard.as_mut() {
            state.today_jalali = today;
            state.viewing_year = today.year;
            state.viewing_month = today.month;
            state.selected_date = today;
        }
    }
    show_calendar_window();
}

fn position_window_near_tray(hwnd: HWND) {
    unsafe {
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);

        let screen_w = GetSystemMetrics(SYSTEM_METRICS_INDEX(0));
        let screen_h = GetSystemMetrics(SYSTEM_METRICS_INDEX(1));

        let dpi = get_dpi_for_window_or_system(hwnd);
        let win_w = scale_for_dpi(420, dpi);
        let win_h = scale_for_dpi(580, dpi);

        // Position window docked above taskbar near cursor
        let mut x = pt.x - (win_w / 2);
        let mut y = pt.y - win_h - 10;

        // Keep within monitor bounds
        if x + win_w > screen_w - 10 {
            x = screen_w - win_w - 10;
        }
        if x < 10 {
            x = 10;
        }
        if y < 10 {
            y = pt.y + 25; // if tray is on top
        }
        if y + win_h > screen_h - 10 {
            y = screen_h - win_h - 45;
        }

        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            x,
            y,
            win_w,
            win_h,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    }
}

unsafe extern "system" fn calendar_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            let state_guard = GLOBAL_STATE.lock().unwrap();
            if let Some(state) = state_guard.as_ref() {
                paint_calendar(hwnd, hdc, state);
            }
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1), // Prevent flicker
        WM_ACTIVATE => {
            let active = (wparam.0 & 0xFFFF) != 0;
            if !active {
                hide_calendar_window();
            }
            LRESULT(0)
        }
        WM_KILLFOCUS => {
            hide_calendar_window();
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let key = wparam.0 as u16;
            if key == VK_ESCAPE.0 {
                hide_calendar_window();
                return LRESULT(0);
            }
            let mut state_guard = GLOBAL_STATE.lock().unwrap();
            if let Some(state) = state_guard.as_mut() {
                if key == VK_HOME.0 {
                    let today = get_current_jalali();
                    state.viewing_year = today.year;
                    state.viewing_month = today.month;
                    state.selected_date = today;
                    let _ = InvalidateRect(hwnd, None, false);
                } else if key == VK_PRIOR.0 {
                    // Previous month
                    let prev = JalaliDate::new(state.viewing_year, state.viewing_month, 1).prev_month();
                    state.viewing_year = prev.year;
                    state.viewing_month = prev.month;
                    let _ = InvalidateRect(hwnd, None, false);
                } else if key == VK_NEXT.0 {
                    // Next month
                    let next = JalaliDate::new(state.viewing_year, state.viewing_month, 1).next_month();
                    state.viewing_year = next.year;
                    state.viewing_month = next.month;
                    let _ = InvalidateRect(hwnd, None, false);
                }
            }
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let mut state_guard = GLOBAL_STATE.lock().unwrap();
            if let Some(state) = state_guard.as_mut() {
                let prev_btn = state.hovered_button;
                let prev_day = state.hovered_day_index;

                state.hovered_button = hit_test_buttons(x, y, state);
                state.hovered_day_index = hit_test_grid(x, y, state);

                if prev_btn != state.hovered_button || prev_day != state.hovered_day_index {
                    let _ = InvalidateRect(hwnd, None, false);
                }
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let mut state_guard = GLOBAL_STATE.lock().unwrap();
            if let Some(state) = state_guard.as_mut() {
                if let Some(btn) = hit_test_buttons(x, y, state) {
                    match btn {
                        ButtonId::PrevMonth => {
                            let p = JalaliDate::new(state.viewing_year, state.viewing_month, 1)
                                .prev_month();
                            state.viewing_year = p.year;
                            state.viewing_month = p.month;
                        }
                        ButtonId::NextMonth => {
                            let n = JalaliDate::new(state.viewing_year, state.viewing_month, 1)
                                .next_month();
                            state.viewing_year = n.year;
                            state.viewing_month = n.month;
                        }
                        ButtonId::PrevYear => {
                            state.viewing_year -= 1;
                        }
                        ButtonId::NextYear => {
                            state.viewing_year += 1;
                        }
                        ButtonId::Today => {
                            let today = get_current_jalali();
                            state.viewing_year = today.year;
                            state.viewing_month = today.month;
                            state.selected_date = today;
                        }
                        ButtonId::ThemeToggle => {
                            if state.theme.is_dark {
                                state.theme = Theme::light();
                            } else {
                                state.theme = Theme::dark();
                            }
                        }
                        ButtonId::_ConverterToggle => {
                            state.view_mode = match state.view_mode {
                                ViewMode::Calendar => ViewMode::Converter,
                                ViewMode::Converter => ViewMode::Calendar,
                            };
                        }
                    }
                    let _ = InvalidateRect(hwnd, None, false);
                } else if let Some(day_idx) = hit_test_grid(x, y, state) {
                    let cells = get_month_grid_cells(state.viewing_year, state.viewing_month);
                    if let Some(cell) = cells.get(day_idx) {
                        if cell.is_current_month {
                            state.selected_date =
                                JalaliDate::new(state.viewing_year, state.viewing_month, cell.day);
                            let _ = InvalidateRect(hwnd, None, false);
                        }
                    }
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// -----------------------------------------------------------------------
// Rendering Logic (GDI Double-Buffered)
// -----------------------------------------------------------------------

struct GridCell {
    day: u32,
    is_current_month: bool,
    jalali: JalaliDate,
    gregorian: GregorianDate,
    lunar: LunarDate,
    is_today: bool,
    is_holiday: bool,
}

fn get_month_grid_cells(year: i32, month: u32) -> Vec<GridCell> {
    let mut cells = Vec::with_capacity(42);
    let today = get_current_jalali();

    let first_day = JalaliDate::new(year, month, 1);
    let first_weekday = first_day.weekday() as usize; // 0 = Shanbeh .. 6 = Jomeh
    let days_in_cur_month = JalaliDate::days_in_year_month(year, month);

    let prev_m = first_day.prev_month();
    let days_in_prev_month = JalaliDate::days_in_year_month(prev_m.year, prev_m.month);

    // Days from previous month
    for i in (0..first_weekday).rev() {
        let d = days_in_prev_month - i as u32;
        let j = JalaliDate::new(prev_m.year, prev_m.month, d);
        let g = j.to_gregorian();
        let l = LunarDate::from_gregorian(g);
        let is_holiday = is_official_holiday(&j, &l);
        cells.push(GridCell {
            day: d,
            is_current_month: false,
            jalali: j,
            gregorian: g,
            lunar: l,
            is_today: j == today,
            is_holiday,
        });
    }

    // Days of current month
    for d in 1..=days_in_cur_month {
        let j = JalaliDate::new(year, month, d);
        let g = j.to_gregorian();
        let l = LunarDate::from_gregorian(g);
        let is_holiday = is_official_holiday(&j, &l);
        cells.push(GridCell {
            day: d,
            is_current_month: true,
            jalali: j,
            gregorian: g,
            lunar: l,
            is_today: j == today,
            is_holiday,
        });
    }

    // Fill remaining cells from next month up to 35 or 42
    let total_cells = if cells.len() <= 35 { 35 } else { 42 };
    let next_m = first_day.next_month();
    let mut next_d = 1;
    while cells.len() < total_cells {
        let j = JalaliDate::new(next_m.year, next_m.month, next_d);
        let g = j.to_gregorian();
        let l = LunarDate::from_gregorian(g);
        let is_holiday = is_official_holiday(&j, &l);
        cells.push(GridCell {
            day: next_d,
            is_current_month: false,
            jalali: j,
            gregorian: g,
            lunar: l,
            is_today: j == today,
            is_holiday,
        });
        next_d += 1;
    }

    cells
}

unsafe fn paint_calendar(hwnd: HWND, hdc: HDC, state: &CalendarWindowState) {
    let mut client_rect = RECT::default();
    let _ = GetClientRect(hwnd, &mut client_rect);
    let width = client_rect.right - client_rect.left;
    let height = client_rect.bottom - client_rect.top;

    // Create double-buffer memory DC
    let hdc_mem = CreateCompatibleDC(hdc);
    let hbm_mem = CreateCompatibleBitmap(hdc, width, height);
    let old_bmp = SelectObject(hdc_mem, hbm_mem);

    let t = state.theme;

    // Background fill
    let bg_brush = CreateSolidBrush(t.background);
    FillRect(hdc_mem, &client_rect, bg_brush);
    let _ = DeleteObject(bg_brush);

    // Border
    let border_brush = CreateSolidBrush(t.border);
    FrameRect(hdc_mem, &client_rect, border_brush);
    let _ = DeleteObject(border_brush);

    SetBkMode(hdc_mem, TRANSPARENT);

    let dpi = state.dpi;
    let pad = scale_for_dpi(12, dpi);

    // 1. Header Bar
    paint_header(hdc_mem, width, pad, state);

    if state.view_mode == ViewMode::Calendar {
        // 2. Weekday Header Row
        let grid_top = scale_for_dpi(70, dpi);
        paint_weekdays(hdc_mem, width, grid_top, pad, state);

        // 3. Calendar Day Grid
        let cells_top = grid_top + scale_for_dpi(28, dpi);
        let grid_bottom = paint_grid(hdc_mem, width, cells_top, pad, state);

        // 4. Selected Day Details Card
        paint_day_details(hdc_mem, width, height, grid_bottom + scale_for_dpi(8, dpi), pad, state);
    } else {
        // Converter View
        paint_converter_view(hdc_mem, width, height, scale_for_dpi(75, dpi), pad, state);
    }

    // Blit memory DC to screen
    let _ = windows::Win32::Graphics::Gdi::BitBlt(
        hdc,
        0,
        0,
        width,
        height,
        hdc_mem,
        0,
        0,
        windows::Win32::Graphics::Gdi::SRCCOPY,
    );

    SelectObject(hdc_mem, old_bmp);
    let _ = DeleteObject(hbm_mem);
    let _ = DeleteDC(hdc_mem);
}

unsafe fn paint_header(hdc: HDC, width: i32, pad: i32, state: &CalendarWindowState) {
    let t = state.theme;
    let dpi = state.dpi;

    let header_h = scale_for_dpi(60, dpi);
    let header_rect = RECT {
        left: 2,
        top: 2,
        right: width - 2,
        bottom: header_h,
    };
    let h_brush = CreateSolidBrush(t.header_bg);
    FillRect(hdc, &header_rect, h_brush);
    let _ = DeleteObject(h_brush);

    // Bottom line of header
    let line_pen = CreatePen(PS_SOLID, 1, t.border);
    let old_pen = SelectObject(hdc, line_pen);
    let _ = MoveToEx(hdc, 0, header_h, None);
    let _ = windows::Win32::Graphics::Gdi::LineTo(hdc, width, header_h);
    SelectObject(hdc, old_pen);
    let _ = DeleteObject(line_pen);

    // Navigation Buttons (Right-to-Left logical order):
    // PrevYear (<<), PrevMonth (<), Title, NextMonth (>), NextYear (>>), Today, Theme
    let btn_w = scale_for_dpi(26, dpi);
    let btn_h = scale_for_dpi(26, dpi);
    let btn_y = scale_for_dpi(17, dpi);

    // Draw buttons
    let draw_btn = |hdc: HDC, rect: RECT, text: &str, btn_id: ButtonId, is_highlight: bool| {
        let is_hovered = state.hovered_button == Some(btn_id);
        let btn_bg = if is_hovered {
            t.surface_hover
        } else if is_highlight {
            t.primary
        } else {
            t.surface
        };
        let b_brush = CreateSolidBrush(btn_bg);
        FillRect(hdc, &rect, b_brush);
        let _ = DeleteObject(b_brush);

        let b_border = CreateSolidBrush(t.border);
        FrameRect(hdc, &rect, b_border);
        let _ = DeleteObject(b_border);

        let font = CreateFontW(
            scale_for_dpi(12, dpi),
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
            0,
            0,
            windows::core::w!("Segoe UI"),
        );
        let old_f = SelectObject(hdc, font);
        SetTextColor(
            hdc,
            if is_highlight {
                COLORREF(0x00FFFFFF)
            } else {
                t.text_primary
            },
        );
        let mut r = rect;
        let mut wide: Vec<u16> = text.encode_utf16().collect();
        DrawTextW(
            hdc,
            &mut wide,
            &mut r,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
        );
        SelectObject(hdc, old_f);
        let _ = DeleteObject(font);
    };

    // Right-side navigation (RTL: Previous month/year is on the right)
    let r1 = RECT {
        left: width - pad - btn_w,
        top: btn_y,
        right: width - pad,
        bottom: btn_y + btn_h,
    };
    draw_btn(hdc, r1, "▶", ButtonId::PrevMonth, false);

    let r2 = RECT {
        left: r1.left - btn_w - scale_for_dpi(4, dpi),
        top: btn_y,
        right: r1.left - scale_for_dpi(4, dpi),
        bottom: btn_y + btn_h,
    };
    draw_btn(hdc, r2, "▶▶", ButtonId::PrevYear, false);

    // Left-side navigation (Next month/year)
    let l1 = RECT {
        left: pad,
        top: btn_y,
        right: pad + btn_w,
        bottom: btn_y + btn_h,
    };
    draw_btn(hdc, l1, "◀", ButtonId::NextMonth, false);

    let l2 = RECT {
        left: l1.right + scale_for_dpi(4, dpi),
        top: btn_y,
        right: l1.right + scale_for_dpi(4, dpi) + btn_w,
        bottom: btn_y + btn_h,
    };
    draw_btn(hdc, l2, "◀◀", ButtonId::NextYear, false);

    // Today Button
    let today_w = scale_for_dpi(48, dpi);
    let today_rect = RECT {
        left: l2.right + scale_for_dpi(8, dpi),
        top: btn_y,
        right: l2.right + scale_for_dpi(8, dpi) + today_w,
        bottom: btn_y + btn_h,
    };
    draw_btn(hdc, today_rect, "امروز", ButtonId::Today, true);

    // Theme toggle button
    let mode_rect = RECT {
        left: r2.left - scale_for_dpi(32, dpi),
        top: btn_y,
        right: r2.left - scale_for_dpi(4, dpi),
        bottom: btn_y + btn_h,
    };
    let mode_icon = if state.theme.is_dark { "☀️" } else { "🌙" };
    draw_btn(hdc, mode_rect, mode_icon, ButtonId::ThemeToggle, false);

    // Title (Month & Year in Center)
    let month_name = if state.viewing_month >= 1 && state.viewing_month <= 12 {
        JALALI_MONTH_NAMES[(state.viewing_month - 1) as usize]
    } else {
        ""
    };
    let title_text = format!(
        "{} {}",
        month_name,
        to_persian_digits(state.viewing_year.to_string())
    );
    let mut title_wide: Vec<u16> = title_text.encode_utf16().collect();

    let font_title = CreateFontW(
        scale_for_dpi(18, dpi),
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
        0,
        0,
        windows::core::w!("Segoe UI"),
    );
    let old_f = SelectObject(hdc, font_title);
    SetTextColor(hdc, t.text_primary);

    let mut title_rect = RECT {
        left: today_rect.right,
        top: 2,
        right: mode_rect.left,
        bottom: header_h,
    };
    DrawTextW(
        hdc,
        &mut title_wide,
        &mut title_rect,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
    );
    SelectObject(hdc, old_f);
    let _ = DeleteObject(font_title);
}

unsafe fn paint_weekdays(
    hdc: HDC,
    width: i32,
    top: i32,
    pad: i32,
    state: &CalendarWindowState,
) {
    let t = state.theme;
    let dpi = state.dpi;
    let col_w = (width - 2 * pad) / 7;

    let weekdays = ["ش", "ی", "د", "س", "چ", "پ", "ج"];

    let font_wd = CreateFontW(
        scale_for_dpi(13, dpi),
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
        0,
        0,
        windows::core::w!("Segoe UI"),
    );
    let old_f = SelectObject(hdc, font_wd);

    // Render columns from right to left (RTL: Saturday is rightmost)
    for (i, name) in weekdays.iter().enumerate() {
        let col_x = width - pad - ((i + 1) as i32 * col_w);
        let mut r = RECT {
            left: col_x,
            top,
            right: col_x + col_w,
            bottom: top + scale_for_dpi(24, dpi),
        };

        let is_friday = i == 6;
        SetTextColor(hdc, if is_friday { t.holiday_red } else { t.text_secondary });

        let mut wide: Vec<u16> = name.encode_utf16().collect();
        DrawTextW(
            hdc,
            &mut wide,
            &mut r,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
        );
    }

    SelectObject(hdc, old_f);
    let _ = DeleteObject(font_wd);
}

unsafe fn paint_grid(
    hdc: HDC,
    width: i32,
    top: i32,
    pad: i32,
    state: &CalendarWindowState,
) -> i32 {
    let t = state.theme;
    let dpi = state.dpi;
    let col_w = (width - 2 * pad) / 7;
    let row_h = scale_for_dpi(40, dpi);

    let cells = get_month_grid_cells(state.viewing_year, state.viewing_month);

    let font_day = CreateFontW(
        scale_for_dpi(16, dpi),
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
        0,
        0,
        windows::core::w!("Segoe UI"),
    );
    let font_small = CreateFontW(
        scale_for_dpi(9, dpi),
        0,
        0,
        0,
        FW_NORMAL.0 as i32,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        windows::core::w!("Segoe UI"),
    );

    let mut cur_y = top;

    for (idx, cell) in cells.iter().enumerate() {
        let col = idx % 7;
        let row = idx / 7;

        // RTL: col 0 is rightmost
        let x = width - pad - ((col + 1) as i32 * col_w);
        let y = top + (row as i32 * row_h);
        cur_y = y + row_h;

        let cell_rect = RECT {
            left: x + 2,
            top: y + 2,
            right: x + col_w - 2,
            bottom: y + row_h - 2,
        };

        let is_selected = cell.is_current_month && cell.jalali == state.selected_date;
        let is_hovered = state.hovered_day_index == Some(idx);

        // Draw Cell Background
        let cell_bg = if is_selected {
            t.surface_selected
        } else if is_hovered {
            t.surface_hover
        } else if cell.is_holiday && cell.is_current_month {
            t.holiday_bg
        } else {
            t.surface
        };

        let brush = CreateSolidBrush(cell_bg);
        FillRect(hdc, &cell_rect, brush);
        let _ = DeleteObject(brush);

        // Draw selection / today rings
        if cell.is_today && cell.is_current_month {
            let today_pen = CreatePen(PS_SOLID, 2, t.today_ring);
            let old_p = SelectObject(hdc, today_pen);
            let old_b = SelectObject(hdc, windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_BRUSH));
            let _ = RoundRect(hdc, cell_rect.left, cell_rect.top, cell_rect.right, cell_rect.bottom, 6, 6);
            SelectObject(hdc, old_b);
            SelectObject(hdc, old_p);
            let _ = DeleteObject(today_pen);
        } else if is_selected {
            let sel_pen = CreatePen(PS_SOLID, 1, t.primary_light);
            let old_p = SelectObject(hdc, sel_pen);
            let old_b = SelectObject(hdc, windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_BRUSH));
            let _ = RoundRect(hdc, cell_rect.left, cell_rect.top, cell_rect.right, cell_rect.bottom, 4, 4);
            SelectObject(hdc, old_b);
            SelectObject(hdc, old_p);
            let _ = DeleteObject(sel_pen);
        }

        // Draw Main Jalali Day Number
        SelectObject(hdc, font_day);
        let text_color = if !cell.is_current_month {
            t.text_muted
        } else if cell.is_holiday {
            t.holiday_red
        } else if is_selected {
            t.primary_light
        } else {
            t.text_primary
        };
        SetTextColor(hdc, text_color);

        let day_str = to_persian_digits(cell.day.to_string());
        let mut day_wide: Vec<u16> = day_str.encode_utf16().collect();
        let mut text_rect = RECT {
            left: cell_rect.left,
            top: cell_rect.top + scale_for_dpi(6, dpi),
            right: cell_rect.right,
            bottom: cell_rect.bottom,
        };
        DrawTextW(
            hdc,
            &mut day_wide,
            &mut text_rect,
            DT_CENTER | DT_TOP | DT_SINGLELINE | DT_NOCLIP,
        );

        // Draw Small Gregorian Day in top-left
        if cell.is_current_month {
            SelectObject(hdc, font_small);
            SetTextColor(hdc, t.text_muted);
            let greg_str = cell.gregorian.day.to_string();
            let mut greg_wide: Vec<u16> = greg_str.encode_utf16().collect();
            let mut greg_rect = RECT {
                left: cell_rect.left + scale_for_dpi(3, dpi),
                top: cell_rect.top + scale_for_dpi(1, dpi),
                right: cell_rect.right,
                bottom: cell_rect.bottom,
            };
            DrawTextW(
                hdc,
                &mut greg_wide,
                &mut greg_rect,
                DT_LEFT | DT_TOP | DT_SINGLELINE | DT_NOCLIP,
            );

            // Draw Small Lunar Hijri Day in top-right
            let lunar_str = to_persian_digits(cell.lunar.day.to_string());
            let mut lunar_wide: Vec<u16> = lunar_str.encode_utf16().collect();
            let mut lunar_rect = RECT {
                left: cell_rect.left,
                top: cell_rect.top + scale_for_dpi(1, dpi),
                right: cell_rect.right - scale_for_dpi(3, dpi),
                bottom: cell_rect.bottom,
            };
            DrawTextW(
                hdc,
                &mut lunar_wide,
                &mut lunar_rect,
                DT_RIGHT | DT_TOP | DT_SINGLELINE | DT_NOCLIP,
            );

            // Holiday dot indicator
            if cell.is_holiday {
                let dot_brush = CreateSolidBrush(t.holiday_red);
                let dot_rect = RECT {
                    left: (cell_rect.left + cell_rect.right) / 2 - 2,
                    top: cell_rect.bottom - 4,
                    right: (cell_rect.left + cell_rect.right) / 2 + 2,
                    bottom: cell_rect.bottom - 1,
                };
                FillRect(hdc, &dot_rect, dot_brush);
                let _ = DeleteObject(dot_brush);
            }
        }
    }

    let _ = DeleteObject(font_day);
    let _ = DeleteObject(font_small);

    cur_y
}

unsafe fn paint_day_details(
    hdc: HDC,
    width: i32,
    height: i32,
    top: i32,
    pad: i32,
    state: &CalendarWindowState,
) {
    let t = state.theme;
    let dpi = state.dpi;

    let card_rect = RECT {
        left: pad,
        top,
        right: width - pad,
        bottom: height - pad,
    };

    // Draw bottom details card
    let card_brush = CreateSolidBrush(t.surface);
    FillRect(hdc, &card_rect, card_brush);
    let _ = DeleteObject(card_brush);

    let border_brush = CreateSolidBrush(t.border);
    FrameRect(hdc, &card_rect, border_brush);
    let _ = DeleteObject(border_brush);

    let sel = state.selected_date;
    let g = sel.to_gregorian();
    let l = LunarDate::from_gregorian(g);
    let events = get_events_for_date(&sel, &l);

    let font_header = CreateFontW(
        scale_for_dpi(15, dpi),
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
        0,
        0,
        windows::core::w!("Segoe UI"),
    );
    let font_sub = CreateFontW(
        scale_for_dpi(12, dpi),
        0,
        0,
        0,
        FW_NORMAL.0 as i32,
        0,
        0,
        0,
        178,
        0,
        0,
        0,
        0,
        windows::core::w!("Segoe UI"),
    );
    let font_event = CreateFontW(
        scale_for_dpi(12, dpi),
        0,
        0,
        0,
        FW_SEMIBOLD.0 as i32,
        0,
        0,
        0,
        178,
        0,
        0,
        0,
        0,
        windows::core::w!("Segoe UI"),
    );

    let inner_pad = scale_for_dpi(10, dpi);
    let mut line_y = top + inner_pad;

    // Line 1: Jalali full date + Weekday (e.g. "یکشنبه ۲۵ مرداد ۱۴۰۵")
    let is_friday = sel.weekday().is_weekend();
    let is_holiday = is_official_holiday(&sel, &l);

    SelectObject(hdc, font_header);
    SetTextColor(
        hdc,
        if is_holiday {
            t.holiday_red
        } else {
            t.primary_light
        },
    );

    let line1 = format!("{} {}", sel.weekday().name_fa(), sel.format_long_fa());
    let mut line1_wide: Vec<u16> = line1.encode_utf16().collect();
    let mut r1 = RECT {
        left: card_rect.left + inner_pad,
        top: line_y,
        right: card_rect.right - inner_pad,
        bottom: line_y + scale_for_dpi(22, dpi),
    };
    DrawTextW(
        hdc,
        &mut line1_wide,
        &mut r1,
        DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
    );
    line_y += scale_for_dpi(24, dpi);

    // Line 2: Gregorian & Lunar Equivalents
    SelectObject(hdc, font_sub);
    SetTextColor(hdc, t.text_secondary);

    let line2 = format!(
        "میلادی: {}  |  قمری: {}",
        g.format_long(),
        l.format_long_fa()
    );
    let mut line2_wide: Vec<u16> = line2.encode_utf16().collect();
    let mut r2 = RECT {
        left: card_rect.left + inner_pad,
        top: line_y,
        right: card_rect.right - inner_pad,
        bottom: line_y + scale_for_dpi(20, dpi),
    };
    DrawTextW(
        hdc,
        &mut line2_wide,
        &mut r2,
        DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
    );
    line_y += scale_for_dpi(24, dpi);

    // Line 3 & 4: Events & Holidays
    SelectObject(hdc, font_event);
    if events.is_empty() {
        if is_friday {
            SetTextColor(hdc, t.holiday_red);
            let mut ev_wide: Vec<u16> = "🔴 تعطیل پایان هفته (جمعه)".encode_utf16().collect();
            let mut ev_rect = RECT {
                left: card_rect.left + inner_pad,
                top: line_y,
                right: card_rect.right - inner_pad,
                bottom: card_rect.bottom - inner_pad,
            };
            DrawTextW(
                hdc,
                &mut ev_wide,
                &mut ev_rect,
                DT_RIGHT | DT_TOP | DT_WORDBREAK,
            );
        } else {
            SetTextColor(hdc, t.text_muted);
            let mut ev_wide: Vec<u16> = "رویداد خاصی ثبت نشده است.".encode_utf16().collect();
            let mut ev_rect = RECT {
                left: card_rect.left + inner_pad,
                top: line_y,
                right: card_rect.right - inner_pad,
                bottom: card_rect.bottom - inner_pad,
            };
            DrawTextW(
                hdc,
                &mut ev_wide,
                &mut ev_rect,
                DT_RIGHT | DT_TOP | DT_WORDBREAK,
            );
        }
    } else {
        for ev in events.iter().take(2) {
            let prefix = if ev.is_holiday {
                "🔴 [تعطیل رسمی] "
            } else {
                "⭐ "
            };
            SetTextColor(
                hdc,
                if ev.is_holiday {
                    t.holiday_red
                } else {
                    t.event_accent
                },
            );
            let ev_str = format!("{}{}", prefix, ev.title);
            let mut ev_wide: Vec<u16> = ev_str.encode_utf16().collect();
            let mut ev_rect = RECT {
                left: card_rect.left + inner_pad,
                top: line_y,
                right: card_rect.right - inner_pad,
                bottom: line_y + scale_for_dpi(20, dpi),
            };
            DrawTextW(
                hdc,
                &mut ev_wide,
                &mut ev_rect,
                DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
            );
            line_y += scale_for_dpi(20, dpi);
        }
    }

    let _ = DeleteObject(font_header);
    let _ = DeleteObject(font_sub);
    let _ = DeleteObject(font_event);
}

unsafe fn paint_converter_view(
    hdc: HDC,
    width: i32,
    _height: i32,
    top: i32,
    pad: i32,
    state: &CalendarWindowState,
) {
    let t = state.theme;
    let dpi = state.dpi;

    let font_title = CreateFontW(
        scale_for_dpi(16, dpi),
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
        0,
        0,
        windows::core::w!("Segoe UI"),
    );
    let font_body = CreateFontW(
        scale_for_dpi(13, dpi),
        0,
        0,
        0,
        FW_NORMAL.0 as i32,
        0,
        0,
        0,
        178,
        0,
        0,
        0,
        0,
        windows::core::w!("Segoe UI"),
    );

    SelectObject(hdc, font_title);
    SetTextColor(hdc, t.primary_light);

    let mut title_wide: Vec<u16> = "🔄 تبدیل تاریخ سه‌گانه".encode_utf16().collect();
    let mut tr = RECT {
        left: pad,
        top: top + scale_for_dpi(10, dpi),
        right: width - pad,
        bottom: top + scale_for_dpi(35, dpi),
    };
    DrawTextW(
        hdc,
        &mut title_wide,
        &mut tr,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
    );

    SelectObject(hdc, font_body);
    SetTextColor(hdc, t.text_primary);

    let j = state.selected_date;
    let g = j.to_gregorian();
    let l = LunarDate::from_gregorian(g);

    let lines = [
        format!("📅 تاریخ خورشیدی (شمسی): {}", j.format_long_fa()),
        format!("🌍 تاریخ میلادی (Gregorian): {}", g.format_long()),
        format!("🌙 تاریخ قمری (Hijri): {}", l.format_long_fa()),
        format!("📌 روز هفته: {}", j.weekday().name_fa()),
        format!("🔢 عدد روز در سال: {} از ۳۶۵", if j.month <= 6 { (j.month - 1) * 31 + j.day } else { 186 + (j.month - 7) * 30 + j.day }),
        format!("⏳ وضعیت سال: {}", if JalaliDate::is_leap_year(j.year) { "سال کبیسه (Leap Year)" } else { "سال عادی" }),
    ];

    let mut y = top + scale_for_dpi(55, dpi);
    for line in lines {
        let card = RECT {
            left: pad,
            top: y,
            right: width - pad,
            bottom: y + scale_for_dpi(38, dpi),
        };
        let b = CreateSolidBrush(t.surface);
        FillRect(hdc, &card, b);
        let _ = DeleteObject(b);

        let fb = CreateSolidBrush(t.border);
        FrameRect(hdc, &card, fb);
        let _ = DeleteObject(fb);

        let mut line_wide: Vec<u16> = line.encode_utf16().collect();
        let mut r = RECT {
            left: card.left + scale_for_dpi(10, dpi),
            top: card.top,
            right: card.right - scale_for_dpi(10, dpi),
            bottom: card.bottom,
        };
        DrawTextW(
            hdc,
            &mut line_wide,
            &mut r,
            DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
        );

        y += scale_for_dpi(46, dpi);
    }

    let _ = DeleteObject(font_title);
    let _ = DeleteObject(font_body);
}

// -----------------------------------------------------------------------
// Hit Testing
// -----------------------------------------------------------------------

fn hit_test_buttons(x: i32, y: i32, state: &CalendarWindowState) -> Option<ButtonId> {
    let dpi = state.dpi;
    let pad = scale_for_dpi(12, dpi);
    let btn_w = scale_for_dpi(26, dpi);
    let btn_h = scale_for_dpi(26, dpi);
    let btn_y = scale_for_dpi(17, dpi);
    let width = scale_for_dpi(420, dpi);

    if y < btn_y || y > btn_y + btn_h {
        return None;
    }

    // PrevMonth
    let r1_left = width - pad - btn_w;
    if x >= r1_left && x <= width - pad {
        return Some(ButtonId::PrevMonth);
    }

    // PrevYear
    let r2_left = r1_left - btn_w - scale_for_dpi(4, dpi);
    if x >= r2_left && x < r1_left {
        return Some(ButtonId::PrevYear);
    }

    // Theme Toggle
    let mode_left = r2_left - scale_for_dpi(32, dpi);
    if x >= mode_left && x < r2_left {
        return Some(ButtonId::ThemeToggle);
    }

    // NextMonth
    if x >= pad && x <= pad + btn_w {
        return Some(ButtonId::NextMonth);
    }

    // NextYear
    let l2_left = pad + btn_w + scale_for_dpi(4, dpi);
    if x >= l2_left && x <= l2_left + btn_w {
        return Some(ButtonId::NextYear);
    }

    // Today
    let today_w = scale_for_dpi(48, dpi);
    let today_left = l2_left + btn_w + scale_for_dpi(8, dpi);
    if x >= today_left && x <= today_left + today_w {
        return Some(ButtonId::Today);
    }

    None
}

fn hit_test_grid(x: i32, y: i32, state: &CalendarWindowState) -> Option<usize> {
    if state.view_mode != ViewMode::Calendar {
        return None;
    }

    let dpi = state.dpi;
    let pad = scale_for_dpi(12, dpi);
    let grid_top = scale_for_dpi(70, dpi) + scale_for_dpi(28, dpi);
    let width = scale_for_dpi(420, dpi);
    let col_w = (width - 2 * pad) / 7;
    let row_h = scale_for_dpi(40, dpi);

    let cells = get_month_grid_cells(state.viewing_year, state.viewing_month);
    let total_rows = (cells.len() + 6) / 7;
    let grid_bottom = grid_top + (total_rows as i32 * row_h);

    if y < grid_top || y >= grid_bottom || x < pad || x >= width - pad {
        return None;
    }

    let row = ((y - grid_top) / row_h) as usize;
    let col_from_right = ((width - pad - x) / col_w) as usize;

    if col_from_right >= 7 {
        return None;
    }

    let idx = row * 7 + col_from_right;
    if idx < cells.len() {
        Some(idx)
    } else {
        None
    }
}

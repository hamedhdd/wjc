//! Popup calendar window lifecycle, multi-monitor docking, and Win32 event loop.

pub mod layout;
pub mod paint;
pub mod state;

use std::sync::Mutex;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, GetMonitorInfoW, InvalidateRect, MonitorFromPoint, HBRUSH, MONITORINFO,
    MONITOR_DEFAULTTONEAREST, PAINTSTRUCT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT, VK_CONTROL, VK_DOWN, VK_ESCAPE,
    VK_HOME, VK_LEFT, VK_NEXT, VK_PRIOR, VK_RIGHT, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetClientRect, GetCursorPos, KillTimer, RegisterClassExW,
    SetForegroundWindow, SetTimer, SetWindowPos, ShowWindow, CS_DROPSHADOW, CS_HREDRAW, CS_VREDRAW,
    HCURSOR, HICON, HWND_TOPMOST, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE, SW_SHOW, WM_ACTIVATE,
    WM_DPICHANGED, WM_ERASEBKGND, WM_KEYDOWN, WM_KILLFOCUS, WM_LBUTTONDOWN,
    WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_PAINT, WM_TIMER, WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_POPUP,
};

use self::layout::{PopupLayout, BASE_HEIGHT, BASE_WIDTH};
use self::paint::paint_calendar;
use self::state::{ButtonId, CalendarWindowState, ViewMode};
use crate::util::clipboard::copy_text_to_clipboard;
use crate::util::dpi::{get_dpi_for_window_or_system, scale_for_dpi};
use crate::util::dwm::enable_win11_rounded_corners;
use crate::util::time::get_current_jalali;
use crate::util::SendHwnd;

const CALENDAR_CLASS: PCWSTR = w!("WJalaliCalendarPopupClass");
const TIMER_COPY_FEEDBACK: usize = 10;
const TRAY_TOGGLE_DEBOUNCE_MS: u64 = 220;

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

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            CALENDAR_CLASS,
            w!("تقویم خورشیدی"),
            WS_POPUP,
            0,
            0,
            BASE_WIDTH,
            BASE_HEIGHT,
            None,
            None,
            instance,
            None,
        )
        .unwrap_or_default();

        enable_win11_rounded_corners(hwnd);

        let dpi = get_dpi_for_window_or_system(hwnd);
        let today = get_current_jalali();

        {
            let mut state_guard = GLOBAL_STATE.lock().unwrap();
            *state_guard = Some(CalendarWindowState::new(today, dpi));
        }
        {
            let mut hwnd_guard = GLOBAL_HWND.lock().unwrap();
            *hwnd_guard = Some(SendHwnd(hwnd));
        }

        hwnd
    }
}

/// Toggles the popup window, debouncing the focus-loss hide that fires right
/// before a tray left-click arrives.
pub fn toggle_calendar_window() {
    let should_show = {
        let state_guard = GLOBAL_STATE.lock().unwrap();
        if let Some(state) = state_guard.as_ref() {
            if state.is_visible {
                false
            } else {
                let elapsed = unsafe { GetTickCount64() }.saturating_sub(state.last_hidden_ms);
                elapsed >= TRAY_TOGGLE_DEBOUNCE_MS
            }
        } else {
            false
        }
    };

    if should_show {
        show_calendar_window();
    } else {
        hide_calendar_window();
    }
}

pub fn show_calendar_window() {
    let hwnd_opt = { *GLOBAL_HWND.lock().unwrap() };
    let Some(send_hwnd) = hwnd_opt else { return };
    let hwnd = send_hwnd.get();

    let today = get_current_jalali();
    let dpi = position_window_near_tray(hwnd);

    {
        let mut state_guard = GLOBAL_STATE.lock().unwrap();
        if let Some(state) = state_guard.as_mut() {
            state.dpi = dpi;
            state.today_jalali = today;
            state.sync_theme_with_system();
            state.ensure_grid_cache();
            state.hovered_button = None;
            state.hovered_day_index = None;
            state.hovered_converter_row = None;
            state.is_visible = true;
        }
    }

    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        let _ = InvalidateRect(hwnd, None, true);
    }
}

pub fn hide_calendar_window() {
    let hwnd_opt = { *GLOBAL_HWND.lock().unwrap() };
    let Some(send_hwnd) = hwnd_opt else { return };
    let hwnd = send_hwnd.get();

    {
        let mut state_guard = GLOBAL_STATE.lock().unwrap();
        if let Some(state) = state_guard.as_mut() {
            if state.is_visible {
                state.is_visible = false;
                state.last_hidden_ms = unsafe { GetTickCount64() };
                state.hovered_button = None;
                state.hovered_day_index = None;
                state.hovered_converter_row = None;
                state.tracking_mouse = false;
            }
        }
    }
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

pub fn go_to_today() {
    let today = get_current_jalali();
    {
        let mut state_guard = GLOBAL_STATE.lock().unwrap();
        if let Some(state) = state_guard.as_mut() {
            state.today_jalali = today;
            state.view_mode = ViewMode::Calendar;
            state.select_date(today);
        }
    }
    show_calendar_window();
}

/// Called when Windows system theme changes (`WM_SETTINGCHANGE`).
pub fn refresh_system_theme() {
    let hwnd_opt = { *GLOBAL_HWND.lock().unwrap() };
    let mut changed = false;
    {
        let mut state_guard = GLOBAL_STATE.lock().unwrap();
        if let Some(state) = state_guard.as_mut() {
            changed = state.sync_theme_with_system();
        }
    }
    if changed {
        if let Some(send_hwnd) = hwnd_opt {
            unsafe {
                let _ = InvalidateRect(send_hwnd.get(), None, false);
            }
        }
    }
}

/// Called when `hijri_months.txt` is reloaded or system date rolls over.
pub fn refresh_calendar_data() {
    let hwnd_opt = { *GLOBAL_HWND.lock().unwrap() };
    let today = get_current_jalali();
    {
        let mut state_guard = GLOBAL_STATE.lock().unwrap();
        if let Some(state) = state_guard.as_mut() {
            state.today_jalali = today;
            state.invalidate_grid_cache();
            state.ensure_grid_cache();
        }
    }
    if let Some(send_hwnd) = hwnd_opt {
        unsafe {
            let _ = InvalidateRect(send_hwnd.get(), None, false);
        }
    }
}

/// Docks the popup neatly inside the active monitor's work area (`rcWork`),
/// supporting multi-monitor setups and arbitrary taskbar edges.
fn position_window_near_tray(hwnd: HWND) -> u32 {
    unsafe {
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);

        let hmon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let work = if GetMonitorInfoW(hmon, &mut mi).as_bool() {
            mi.rcWork
        } else {
            RECT {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1040,
            }
        };

        // Move briefly to target monitor so GetDpiForWindow reads that monitor's DPI
        let _ = SetWindowPos(hwnd, HWND_TOPMOST, pt.x, pt.y, 0, 0, SWP_NOACTIVATE | windows::Win32::UI::WindowsAndMessaging::SWP_NOSIZE);
        let dpi = get_dpi_for_window_or_system(hwnd);

        let win_w = scale_for_dpi(BASE_WIDTH, dpi);
        let win_h = scale_for_dpi(BASE_HEIGHT, dpi);
        let margin = scale_for_dpi(10, dpi);

        // Anchor horizontally around cursor, clamped to monitor work area
        let mut x = pt.x - win_w / 2;
        if x + win_w > work.right - margin {
            x = work.right - win_w - margin;
        }
        if x < work.left + margin {
            x = work.left + margin;
        }

        // Dock vertically inside work area near the cursor's taskbar edge
        let mut y = if pt.y >= work.bottom {
            work.bottom - win_h - margin
        } else if pt.y <= work.top {
            work.top + margin
        } else if pt.y - win_h - margin >= work.top {
            pt.y - win_h - margin
        } else {
            work.bottom - win_h - margin
        };

        if y < work.top + margin {
            y = work.top + margin;
        }
        if y + win_h > work.bottom - margin {
            y = work.bottom - win_h - margin;
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
        dpi
    }
}

fn current_layout(hwnd: HWND, state: &CalendarWindowState) -> PopupLayout {
    let mut r = RECT::default();
    unsafe {
        let _ = GetClientRect(hwnd, &mut r);
    }
    let w = (r.right - r.left).max(scale_for_dpi(BASE_WIDTH, state.dpi));
    let h = (r.bottom - r.top).max(scale_for_dpi(BASE_HEIGHT, state.dpi));
    PopupLayout::compute(w, h, state.dpi, state.grid_rows())
}

fn trigger_copy(hwnd: HWND, state: &mut CalendarWindowState, text: &str) {
    if copy_text_to_clipboard(hwnd, text) {
        unsafe {
            state.copied_until_ms = GetTickCount64() + 1400;
            let _ = SetTimer(hwnd, TIMER_COPY_FEEDBACK, 1450, None);
            let _ = InvalidateRect(hwnd, None, false);
        }
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
            if let Ok(mut state_guard) = GLOBAL_STATE.lock() {
                if let Some(state) = state_guard.as_mut() {
                    paint_calendar(hwnd, hdc, state);
                }
            }
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_ACTIVATE => {
            if (wparam.0 & 0xFFFF) == 0 {
                hide_calendar_window();
            }
            LRESULT(0)
        }
        WM_KILLFOCUS => {
            hide_calendar_window();
            LRESULT(0)
        }
        WM_DPICHANGED => {
            let new_dpi = (wparam.0 & 0xFFFF) as u32;
            if new_dpi >= 96 {
                if let Ok(mut guard) = GLOBAL_STATE.lock() {
                    if let Some(state) = guard.as_mut() {
                        state.dpi = new_dpi;
                        state.fonts = None;
                    }
                }
                let _ = InvalidateRect(hwnd, None, false);
            }
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == TIMER_COPY_FEEDBACK {
                let _ = KillTimer(hwnd, TIMER_COPY_FEEDBACK);
                let _ = InvalidateRect(hwnd, None, false);
            }
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            let delta = ((wparam.0 >> 16) & 0xFFFF) as i16;
            if delta != 0 {
                if let Ok(mut guard) = GLOBAL_STATE.lock() {
                    if let Some(state) = guard.as_mut() {
                        // Wheel up -> previous month, wheel down -> next month
                        state.navigate_month(if delta > 0 { -1 } else { 1 });
                        let _ = InvalidateRect(hwnd, None, false);
                    }
                }
            }
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let key = wparam.0 as u16;
            if key == VK_ESCAPE.0 {
                hide_calendar_window();
                return LRESULT(0);
            }

            let ctrl_down = (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;

            if let Ok(mut guard) = GLOBAL_STATE.lock() {
                if let Some(state) = guard.as_mut() {
                    if ctrl_down && (key == b'C' as u16) {
                        let sel = state.selected_date;
                        let text = format!("{} {}", sel.weekday().name_fa(), sel.format_long_fa());
                        trigger_copy(hwnd, state, &text);
                        return LRESULT(0);
                    }
                    match key {
                        k if k == VK_HOME.0 => {
                            let today = get_current_jalali();
                            state.today_jalali = today;
                            state.select_date(today);
                        }
                        k if k == VK_PRIOR.0 => state.navigate_month(-1),
                        k if k == VK_NEXT.0 => state.navigate_month(1),
                        // RTL calendar grid: Left moves forward (+1), Right moves backward (-1)
                        k if k == VK_LEFT.0 => state.select_date(state.selected_date.add_days(1)),
                        k if k == VK_RIGHT.0 => state.select_date(state.selected_date.add_days(-1)),
                        k if k == VK_UP.0 => state.select_date(state.selected_date.add_days(-7)),
                        k if k == VK_DOWN.0 => state.select_date(state.selected_date.add_days(7)),
                        _ => return LRESULT(0),
                    }
                    let _ = InvalidateRect(hwnd, None, false);
                }
            }
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            if let Ok(mut guard) = GLOBAL_STATE.lock() {
                if let Some(state) = guard.as_mut() {
                    if !state.tracking_mouse {
                        let mut tme = TRACKMOUSEEVENT {
                            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                            dwFlags: TME_LEAVE,
                            hwndTrack: hwnd,
                            dwHoverTime: 0,
                        };
                        let _ = TrackMouseEvent(&mut tme);
                        state.tracking_mouse = true;
                    }

                    let layout = current_layout(hwnd, state);
                    let prev = (
                        state.hovered_button,
                        state.hovered_day_index,
                        state.hovered_converter_row,
                    );

                    state.hovered_button = layout.hit_test_button(x, y, state.view_mode);
                    state.hovered_day_index =
                        layout.hit_test_grid(x, y, state.view_mode, state.cells().len());
                    state.hovered_converter_row =
                        layout.hit_test_converter_row(x, y, state.view_mode);

                    if prev
                        != (
                            state.hovered_button,
                            state.hovered_day_index,
                            state.hovered_converter_row,
                        )
                    {
                        let _ = InvalidateRect(hwnd, None, false);
                    }
                }
            }
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            if let Ok(mut guard) = GLOBAL_STATE.lock() {
                if let Some(state) = guard.as_mut() {
                    state.tracking_mouse = false;
                    if state.hovered_button.is_some()
                        || state.hovered_day_index.is_some()
                        || state.hovered_converter_row.is_some()
                    {
                        state.hovered_button = None;
                        state.hovered_day_index = None;
                        state.hovered_converter_row = None;
                        let _ = InvalidateRect(hwnd, None, false);
                    }
                }
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            if let Ok(mut guard) = GLOBAL_STATE.lock() {
                if let Some(state) = guard.as_mut() {
                    let layout = current_layout(hwnd, state);
                    if let Some(btn) = layout.hit_test_button(x, y, state.view_mode) {
                        match btn {
                            ButtonId::PrevMonth => state.navigate_month(-1),
                            ButtonId::NextMonth => state.navigate_month(1),
                            ButtonId::PrevYear => state.navigate_year(-1),
                            ButtonId::NextYear => state.navigate_year(1),
                            ButtonId::Today => {
                                let today = get_current_jalali();
                                state.today_jalali = today;
                                state.select_date(today);
                            }
                            ButtonId::ThemeToggle => state.toggle_theme(),
                            ButtonId::ViewToggle => {
                                state.view_mode = match state.view_mode {
                                    ViewMode::Calendar => ViewMode::Converter,
                                    ViewMode::Converter => ViewMode::Calendar,
                                };
                            }
                            ButtonId::CopyDate => {
                                let sel = state.selected_date;
                                let text =
                                    format!("{} {}", sel.weekday().name_fa(), sel.format_long_fa());
                                trigger_copy(hwnd, state, &text);
                                return LRESULT(0);
                            }
                        }
                        let _ = InvalidateRect(hwnd, None, false);
                    } else if let Some(day_idx) =
                        layout.hit_test_grid(x, y, state.view_mode, state.cells().len())
                    {
                        if let Some(cell) = state.cells().get(day_idx).cloned() {
                            // Clicking any day (including prev/next month dimmed cells) selects & navigates
                            state.select_date(cell.jalali);
                            let _ = InvalidateRect(hwnd, None, false);
                        }
                    } else if let Some(row_idx) =
                        layout.hit_test_converter_row(x, y, state.view_mode)
                    {
                        let rows = state.converter_rows();
                        if let Some((_, copy_val)) = rows.get(row_idx) {
                            let val = copy_val.clone();
                            trigger_copy(hwnd, state, &val);
                        }
                    }
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

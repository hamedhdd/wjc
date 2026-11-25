#![windows_subsystem = "windows"]

mod calendar;
mod ui;
mod util;

use std::mem::zeroed;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WPARAM,
};
use windows::Win32::Graphics::Gdi::HBRUSH;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, FindWindowW, GetMessageW,
    MessageBoxW, PostMessageW, PostQuitMessage, RegisterClassExW, RegisterWindowMessageW,
    SetTimer, TranslateMessage, HCURSOR, HICON, MB_ICONINFORMATION, MB_OK, MB_RTLREADING, MSG,
    WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP, WM_SETTINGCHANGE, WM_TIMECHANGE,
    WM_TIMER, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};

use crate::calendar::lunar::reload_iran_hijri;
use crate::calendar::{to_persian_digits, FullDateInfo};
use crate::ui::tray::{
    TrayIcon, IDM_ABOUT, IDM_AUTOSTART, IDM_EXIT, IDM_RELOAD_HOLIDAYS, IDM_SHOW_CALENDAR,
    IDM_STYLE_AUTO, IDM_STYLE_MONTH_DAY, IDM_STYLE_NUMERIC, IDM_TODAY, WM_TRAYICON,
};
use crate::ui::{
    go_to_today, init_calendar_window, refresh_calendar_data, refresh_system_theme,
    show_calendar_window, toggle_calendar_window,
};
use crate::util::autostart::{is_autostart_enabled, set_autostart};
use crate::util::settings::TrayIconStyle;
use crate::util::time::get_current_gregorian;

const MAIN_WINDOW_CLASS: PCWSTR = w!("WJalaliCalendarMainClass");
const TIMER_DATE_CHECK: usize = 1;
const WM_SHOW_EXISTING: u32 = 0x8000 + 202; // WM_APP + 202

static GLOBAL_TRAY: Mutex<Option<TrayIcon>> = Mutex::new(None);
static WM_TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);

fn main() {
    unsafe {
        // Single instance check via Named Mutex
        let mutex_name = w!("Global\\WJalaliCalendarUniqueMutex");
        let mutex = CreateMutexW(None, true, mutex_name).unwrap_or_default();
        if GetLastError() == ERROR_ALREADY_EXISTS {
            // Focus / open the running instance's calendar instead of showing a modal nag
            if let Ok(existing) = FindWindowW(MAIN_WINDOW_CLASS, PCWSTR::null()) {
                let _ = PostMessageW(existing, WM_SHOW_EXISTING, WPARAM(0), LPARAM(0));
            }
            let _ = CloseHandle(mutex);
            return;
        }

        // Register Explorer restart recovery message
        let taskbar_msg = RegisterWindowMessageW(w!("TaskbarCreated"));
        WM_TASKBAR_CREATED.store(taskbar_msg, Ordering::Relaxed);

        let instance = GetModuleHandleW(None).unwrap();

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: windows::Win32::UI::WindowsAndMessaging::WNDCLASS_STYLES(0),
            lpfnWndProc: Some(main_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: HICON::default(),
            hCursor: HCURSOR::default(),
            hbrBackground: HBRUSH::default(),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: MAIN_WINDOW_CLASS,
            hIconSm: HICON::default(),
        };

        let _ = RegisterClassExW(&wnd_class);

        // Hidden top-level window (top-level so it receives WM_SETTINGCHANGE & TaskbarCreated)
        let hwnd_main = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            MAIN_WINDOW_CLASS,
            w!("WJalaliCalendarDaemon"),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            None,
            None,
            instance,
            None,
        )
        .unwrap_or_default();

        // Initialize Calendar Window Popup
        let hwnd_calendar = init_calendar_window();

        // Initialize Tray Icon
        let mut tray = TrayIcon::new(hwnd_main);
        let date_info = FullDateInfo::from_gregorian(get_current_gregorian());
        tray.init(&date_info);

        {
            let mut tray_guard = GLOBAL_TRAY.lock().unwrap();
            *tray_guard = Some(tray);
        }

        // Check midnight date transition every 30 seconds
        let _ = SetTimer(hwnd_main, TIMER_DATE_CHECK, 30_000, None);

        // Win32 Standard Message Loop
        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = DestroyWindow(hwnd_calendar);
        let _ = ReleaseMutex(mutex);
        let _ = CloseHandle(mutex);
    }
}

unsafe extern "system" fn main_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let taskbar_created = WM_TASKBAR_CREATED.load(Ordering::Relaxed);
    if taskbar_created != 0 && msg == taskbar_created {
        let date_info = FullDateInfo::from_gregorian(get_current_gregorian());
        if let Ok(mut tray_guard) = GLOBAL_TRAY.lock() {
            if let Some(tray) = tray_guard.as_mut() {
                tray.readd(&date_info);
            }
        }
        return LRESULT(0);
    }

    match msg {
        WM_SHOW_EXISTING => {
            show_calendar_window();
            LRESULT(0)
        }
        WM_TRAYICON => {
            let event = (lparam.0 & 0xFFFF) as u32;
            match event {
                WM_LBUTTONUP | WM_LBUTTONDBLCLK => {
                    toggle_calendar_window();
                }
                WM_RBUTTONUP => {
                    let cmd = {
                        let tray_guard = GLOBAL_TRAY.lock().unwrap();
                        tray_guard
                            .as_ref()
                            .map(|tray| tray.show_context_menu())
                            .unwrap_or(0)
                    };
                    if cmd != 0 {
                        handle_menu_command(hwnd, cmd);
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_SETTINGCHANGE => {
            // Instant reaction to Windows Light/Dark theme or DPI/metrics change
            let date_info = FullDateInfo::from_gregorian(get_current_gregorian());
            if let Ok(mut tray_guard) = GLOBAL_TRAY.lock() {
                if let Some(tray) = tray_guard.as_mut() {
                    tray.update(&date_info, false);
                }
            }
            refresh_system_theme();
            LRESULT(0)
        }
        WM_TIMECHANGE => {
            // Instant reaction to system clock/timezone change
            let date_info = FullDateInfo::from_gregorian(get_current_gregorian());
            if let Ok(mut tray_guard) = GLOBAL_TRAY.lock() {
                if let Some(tray) = tray_guard.as_mut() {
                    tray.update(&date_info, true);
                }
            }
            refresh_calendar_data();
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == TIMER_DATE_CHECK {
                let date_info = FullDateInfo::from_gregorian(get_current_gregorian());
                if let Ok(mut tray_guard) = GLOBAL_TRAY.lock() {
                    if let Some(tray) = tray_guard.as_mut() {
                        tray.update(&date_info, false);
                    }
                }
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            if let Ok(mut tray_guard) = GLOBAL_TRAY.lock() {
                if let Some(tray) = tray_guard.as_mut() {
                    tray.remove();
                }
            }
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn handle_menu_command(hwnd: HWND, cmd: u32) {
    match cmd {
        IDM_SHOW_CALENDAR => show_calendar_window(),
        IDM_TODAY => go_to_today(),
        IDM_STYLE_AUTO | IDM_STYLE_MONTH_DAY | IDM_STYLE_NUMERIC => {
            let style = match cmd {
                IDM_STYLE_MONTH_DAY => TrayIconStyle::MonthDay,
                IDM_STYLE_NUMERIC => TrayIconStyle::Numeric,
                _ => TrayIconStyle::Auto,
            };
            let date_info = FullDateInfo::from_gregorian(get_current_gregorian());
            if let Ok(mut tray_guard) = GLOBAL_TRAY.lock() {
                if let Some(tray) = tray_guard.as_mut() {
                    tray.set_style(style, &date_info);
                }
            }
        }
        IDM_RELOAD_HOLIDAYS => {
            reload_iran_hijri();
            let date_info = FullDateInfo::from_gregorian(get_current_gregorian());
            if let Ok(mut tray_guard) = GLOBAL_TRAY.lock() {
                if let Some(tray) = tray_guard.as_mut() {
                    tray.update(&date_info, true);
                }
            }
            refresh_calendar_data();
        }
        IDM_AUTOSTART => {
            let new_state = !is_autostart_enabled();
            set_autostart(new_state);
            let msg = if new_state {
                w!("اجرای خودکار برنامه هنگام روشن شدن ویندوز فعال شد.")
            } else {
                w!("اجرای خودکار برنامه هنگام روشن شدن ویندوز غیرفعال شد.")
            };
            let _ = MessageBoxW(
                hwnd,
                msg,
                w!("تنظیمات شروع خودکار"),
                MB_OK | MB_ICONINFORMATION | MB_RTLREADING,
            );
        }
        IDM_ABOUT => {
            let version_fa = to_persian_digits(env!("CARGO_PKG_VERSION"));
            let about_str = format!(
                "تقویم خورشیدی ویندوز (Windows Jalali Calendar)\n\n\
                نسخه: {version_fa}\n\
                توسعه یافته با Rust خالص و Win32 Native API\n\
                سبک، سریع و بدون وابستگی به دات‌نت (.NET)\n\
                سازگار با ویندوز ۷ تا ۱۱، همراه با مناسبت‌ها و تعطیلات رسمی ایران\n\
                برای افزودن سال‌های جدید، فایل hijri_months.txt را کنار wjc.exe قرار دهید."
            );
            let wide: Vec<u16> = about_str.encode_utf16().chain(std::iter::once(0)).collect();
            let _ = MessageBoxW(
                hwnd,
                PCWSTR(wide.as_ptr()),
                w!("درباره تقویم خورشیدی"),
                MB_OK | MB_ICONINFORMATION | MB_RTLREADING,
            );
        }
        IDM_EXIT => {
            if let Ok(mut tray_guard) = GLOBAL_TRAY.lock() {
                if let Some(tray) = tray_guard.as_mut() {
                    tray.remove();
                }
            }
            PostQuitMessage(0);
        }
        _ => {}
    }
}

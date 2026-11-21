#![windows_subsystem = "windows"]

mod calendar;
mod ui;
mod util;

use std::mem::zeroed;
use std::sync::Mutex;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::HBRUSH;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, MessageBoxW,
    PostQuitMessage, RegisterClassExW, SetTimer, TranslateMessage, HCURSOR, HICON,
    MB_ICONINFORMATION, MB_OK, MB_RTLREADING, MSG, WM_DESTROY, WM_LBUTTONDBLCLK,
    WM_LBUTTONUP, WM_RBUTTONUP, WM_TIMER, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};

use crate::calendar::FullDateInfo;
use crate::ui::tray::{
    TrayIcon, IDM_ABOUT, IDM_AUTOSTART, IDM_EXIT, IDM_SHOW_CALENDAR, IDM_TODAY, WM_TRAYICON,
};
use crate::ui::{go_to_today, init_calendar_window, show_calendar_window, toggle_calendar_window};
use crate::util::autostart::{is_autostart_enabled, set_autostart};
use crate::util::time::get_current_gregorian;

const MAIN_WINDOW_CLASS: PCWSTR = w!("WJalaliCalendarMainClass");
const TIMER_DATE_CHECK: usize = 1;

static GLOBAL_TRAY: Mutex<Option<TrayIcon>> = Mutex::new(None);

fn main() {
    unsafe {
        // Single instance check via Named Mutex
        let mutex_name = w!("Global\\WJalaliCalendarUniqueMutex");
        let mutex = CreateMutexW(None, true, mutex_name).unwrap_or_default();
        if GetLastError() == ERROR_ALREADY_EXISTS {
            // Already running - user tried to open again, notify or exit
            let _ = MessageBoxW(
                None,
                w!("برنامه تقویم خورشیدی در حال حاضر در تسک‌بار (سیستم‌تری) در حال اجراست."),
                w!("تقویم خورشیدی"),
                MB_OK | MB_ICONINFORMATION | MB_RTLREADING,
            );
            return;
        }

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

        // Create hidden message-only window
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
        .unwrap_or(HWND::default());

        // Initialize Calendar Window Popup
        let _hwnd_calendar = init_calendar_window();

        // Initialize Tray Icon
        let mut tray = TrayIcon::new(hwnd_main);
        let greg = get_current_gregorian();
        let date_info = FullDateInfo::from_gregorian(greg);
        tray.init(&date_info);

        {
            let mut tray_guard = GLOBAL_TRAY.lock().unwrap();
            *tray_guard = Some(tray);
        }

        // Check date transition every 30 seconds
        let _ = SetTimer(hwnd_main, TIMER_DATE_CHECK, 30_000, None);

        // Win32 Standard Message Loop
        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = ReleaseMutex(mutex);
    }
}

unsafe extern "system" fn main_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            let event = (lparam.0 & 0xFFFF) as u32;
            match event {
                WM_LBUTTONUP | WM_LBUTTONDBLCLK => {
                    toggle_calendar_window();
                }
                WM_RBUTTONUP => {
                    let cmd = {
                        let tray_guard = GLOBAL_TRAY.lock().unwrap();
                        if let Some(tray) = tray_guard.as_ref() {
                            tray.show_context_menu()
                        } else {
                            0
                        }
                    };
                    if cmd != 0 {
                        handle_menu_command(hwnd, cmd);
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == TIMER_DATE_CHECK {
                // Periodic date check and update
                let greg = get_current_gregorian();
                let date_info = FullDateInfo::from_gregorian(greg);
                let mut tray_guard = GLOBAL_TRAY.lock().unwrap();
                if let Some(tray) = tray_guard.as_mut() {
                    tray.update(&date_info, false);
                }
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            {
                let mut tray_guard = GLOBAL_TRAY.lock().unwrap();
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
        IDM_SHOW_CALENDAR => {
            show_calendar_window();
        }
        IDM_TODAY => {
            go_to_today();
        }
        IDM_AUTOSTART => {
            let current = is_autostart_enabled();
            let new_state = !current;
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
            let about_text = w!(
                "تقویم خورشیدی ویندوز (Windows Jalali Calendar)\n\n\
                نسخه: ۱.۰.۰\n\
                توسعه یافته با Rust خالص و Win32 Native API\n\
                فوق‌العاده سبک، سریع، بدون وابستگی به دات‌نت (.NET)\n\
                دارای آیکون زنده در تسک‌بار و تقویم کامل همراه با مناسبت‌ها و تعطیلات رسمی ایران."
            );
            let _ = MessageBoxW(
                hwnd,
                about_text,
                w!("درباره تقویم خورشیدی"),
                MB_OK | MB_ICONINFORMATION | MB_RTLREADING,
            );
        }
        IDM_EXIT => {
            {
                let mut tray_guard = GLOBAL_TRAY.lock().unwrap();
                if let Some(tray) = tray_guard.as_mut() {
                    tray.remove();
                }
            }
            PostQuitMessage(0);
        }
        _ => {}
    }
}

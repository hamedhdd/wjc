# Project Codebase Map: Windows Jalali Calendar (`wjc`)

- **GitHub Repository**: [https://github.com/hamedhdd/wjc](https://github.com/hamedhdd/wjc)
- **Active Branch**: `main`
- **Version**: `1.4.0`

## 1. Directory Tree & Component Index

```
c:\Users\Hamed\Desktop\Projects\wjc\
├── Cargo.toml               # Cargo package configuration (v1.4.0), Win32 features, release profile
├── app.manifest             # Windows Application Manifest (PerMonitorV2 DPI, Common Controls v6)
├── app.ico                  # Embedded application icon resource
├── build.rs                 # Win32 icon + manifest resource embedding via winres
├── README.md                # Comprehensive documentation in Persian & English
├── CHANGELOG.md             # Keep-a-Changelog release history
├── PROJECT_MAP.md           # Architecture map and codebase index (this file)
├── data/
│   └── hijri_months.txt     # Official Iranian lunar month starts (Shawwal 1445 - Shawwal 1449)
└── src/
    ├── main.rs              # Win32 daemon entry, single-instance handoff, TaskbarCreated/theme/time hooks
    ├── calendar/
    │   ├── mod.rs           # Module exports for calendar engine
    │   ├── jalali.rs        # Deduplicated Jalali <-> Gregorian <-> JDN engine, leap years, arithmetic
    │   ├── astro.rs         # Meeus new moon conjunction & Tehran crescent visibility fallback
    │   ├── hijri_table.rs   # Parser for hijri_months.txt format
    │   ├── hijri_source.rs  # Embedded month-start loader + external hijri_months.txt override
    │   ├── lunar.rs         # LunarDate, RwLock-backed reloadable HijriCalendar, estimate detection
    │   ├── holidays.rs      # Fixed solar & Hijri lunar holiday/observance rules + verification tests
    │   └── converter.rs     # FullDateInfo unified 3-calendar representation
    ├── ui/
    │   ├── mod.rs           # UI module exports
    │   ├── theme.rs         # Unified Graphite neutral ramp palettes (Dark & Light)
    │   ├── gdi.rs           # RAII GDI wrappers, DPI-cached PopupFonts, crisp vector icon primitives
    │   ├── tray.rs          # Native-DPI ARGB+1-bit mask tray icon renderer, styles, context menu
    │   └── popup/
    │       ├── mod.rs       # Popup window lifecycle, multi-monitor docking, mouse/keyboard/wheel events
    │       ├── state.rs     # CalendarWindowState, cached month GridCell list, view & theme state
    │       ├── layout.rs    # Single-source-of-truth 4px-grid geometry & hit-testing
    │       └── paint.rs     # Double-buffered GDI renderer for header, grid, details card, converter view
    └── util/
        ├── mod.rs           # Utility exports & SendHwnd wrapper
        ├── autostart.rs     # Registry Run key management with exe path verification
        ├── clipboard.rs     # Win32 UTF-16 clipboard copy helper
        ├── dpi.rs           # Dynamic GetDpiForWindow (Win10+) with Win7/8 GetDeviceCaps fallback
        ├── dwm.rs           # Dynamic Windows 11 DWM rounded corners (graceful Win7-10 no-op)
        ├── settings.rs      # Persisted ThemePreference & TrayIconStyle (HKCU\Software\WJalaliCalendar)
        ├── theme_detect.rs  # Windows taskbar (SystemUsesLightTheme) & app (AppsUseLightTheme) detection
        └── time.rs          # Local system time helpers
```

## 2. Data & Event Flow

```
[Windows Local Time / WM_TIMECHANGE / WM_SETTINGCHANGE / TaskbarCreated]
       │
       ▼
[FullDateInfo] = (GregorianDate, JalaliDate, LunarDate)
       │         └─ HijriCalendar (RwLock: embedded + hijri_months.txt override + Meeus astro)
       │
       ├─────────────────────────────┬─────────────────────────────────┐
       ▼                             ▼                                 ▼
[TrayIcon (src/ui/tray.rs)]   [PopupState (popup/state.rs)]    [Settings (util/settings.rs)]
- Native SM_CXSMICON bitmap   - Cached 35/42 GridCell array    - HKCU\Software\WJalaliCalendar
- 32-bit ARGB + 1-bit mask    - DPI-cached PopupFonts          - ThemePreference & TrayIconStyle
- Auto / MonthDay / Numeric   - Unified PopupLayout rects
- Dark & Light taskbar ramps  - Double-buffered GDI paint
- Explorer restart recovery   - Multi-monitor rcWork docking
```

## 3. Architecture & SOLID Principles Adherence

- **Single Responsibility Principle (SRP)**: Calendar math (`jalali`, `astro`, `hijri_table`, `hijri_source`, `lunar`, `holidays`), Windows OS integrations (`autostart`, `clipboard`, `dpi`, `dwm`, `settings`, `theme_detect`), GDI resource ownership (`ui/gdi.rs`), tray rendering (`ui/tray.rs`), and popup state/layout/paint/windowing (`ui/popup/*`) each live in focused modules.
- **Open/Closed Principle (OCP)**: Future Hijri years (1407+) are added via `hijri_months.txt` next to `wjc.exe` and reloaded live from the tray menu without recompiling.
- **Dependency Inversion & Safety**: Optional Win10/11 APIs (`GetDpiForWindow`, `DwmSetWindowAttribute`) are resolved dynamically so the binary runs natively on Windows 7 through Windows 11; RAII GDI wrappers prevent resource leaks.

## 4. UI Layout & Controls

- **Tray Context Menu**: Show Calendar (default), Today, Tray Style submenu (Auto / Month+Day / Numeric), Reload Holidays, Start with Windows, About, Exit.
- **Popup Header**: RTL Prev/Next Month & Year vector chevrons, Today button, Converter/Calendar view toggle icon, Sun/Moon theme toggle icon, centered Persian Month + Year title.
- **Calendar View**: 7-column RTL weekday bar, cached 5–6 row day grid (Jalali + Gregorian + Hijri + holiday pip), details card with Copy button (`Ctrl+C`), wrapped event list, and astronomical estimate tag when beyond official table.
- **Converter / Details View**: 6 clickable cards (Jalali, Gregorian, Hijri, Weekday, Day of Year / 365–366, Leap Year status) — click any card to copy to clipboard.

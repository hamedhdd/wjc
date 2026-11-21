# Project Codebase Map: Windows Jalali Calendar (`wjc`)

## 1. Directory Tree & Component Index

```
c:\Users\Hamed\Desktop\Projects\wjc\
├── Cargo.toml               # Cargo package configuration, dependencies, release profile
├── app.manifest             # Windows Application Manifest (DPI awareness, Common Controls v6)
├── build.rs                 # Win32 resource / manifest compilation build script
├── README.md                # Comprehensive documentation in Persian & English
├── CHANGELOG.md             # Standard Keep-a-Changelog record of project releases
├── PROJECT_MAP.md           # Architecture map and codebase index (this file)
├── data/
│   └── hijri_months.txt     # Official Iranian lunar month starts (Shawwal 1445 - Shawwal 1449)
└── src/
    ├── main.rs              # Win32 application entry point, single-instance mutex, main message loop
    ├── calendar/
    │   ├── mod.rs           # Module exports for calendar engine
    │   ├── jalali.rs        # Jalali (Solar Hijri) <-> Gregorian conversions & weekday formatting
    │   ├── astro.rs         # Astronomical Meeus conjunction & Tehran crescent visibility algorithms
    │   ├── hijri_table.rs   # Parser for hijri_months.txt format and JDN coordinate mapping
    │   ├── hijri_source.rs  # Embedded month-start loader + external file override discovery
    │   ├── lunar.rs         # LunarDate struct, Iran-calibrated Hijri Calendar engine, OnceLock instance
    │   ├── holidays.rs      # Solar & Lunar event definitions, holiday detection, official date tests
    │   └── converter.rs     # FullDateInfo unified representation across 3 calendars
    ├── ui/
    │   ├── mod.rs           # UI module exports
    │   ├── theme.rs         # Dark / Light theme color palettes, brush caches, GDI colors
    │   ├── tray.rs          # System tray icon (NOTIFYICONDATAW), dynamic icon rasterization, tooltips
    │   └── calendar_window.rs# Modern popup calendar window (GDI double-buffering, RTL grid, event list)
    └── util/
        ├── mod.rs           # Utility module exports
        ├── autostart.rs     # Windows Registry Run key management (Start with Windows)
        ├── dpi.rs           # Per-monitor DPI scaling utilities
        └── time.rs          # Local time helpers and day change detection
```

## 2. Data & Telemetry Flow

```
[System Time (Local)]
       │
       ▼
[GregorianDate] ──────► [JalaliDate] (Arithmetic conversion)
       │                      │
       │                      ▼
       └──────────────► [iran_hijri() / LunarDate]
                              │ (via official hijri_months.txt table or Meeus astro fallback)
                              ▼
                       [FullDateInfo]
                              │
          ┌───────────────────┴───────────────────┐
          ▼                                       ▼
  [Tray Icon Engine]                      [Calendar Popup UI]
  - Dynamic GDI bitmap generation         - Double-buffered GDI rendering
  - Persian digit tray text (e.g. 25/5)   - RTL 7x6 month day grid
  - Red tint on holidays & Fridays        - Header navigation (prev/next month/year)
  - Rich multiline tooltip                - Detail card with 3 calendar formats
                                          - Iranian official events & holidays list
```

## 3. Architecture & SOLID Principles Adherence

- **Single Responsibility Principle (SRP)**:
  - `calendar/astro.rs`: Solely responsible for astronomical calculations (Meeus algorithm, sunset, crescent age).
  - `calendar/hijri_table.rs`: Only parses and validates month-start definitions.
  - `calendar/hijri_source.rs`: Responsible for file discovery and fallback between embedded data and user override.
  - `calendar/holidays.rs`: Encapsulates holiday rules without hardcoding yearly date shifts.
  - `ui/tray.rs` vs `ui/calendar_window.rs`: Separates system notification area presence from full calendar view.
- **Open/Closed Principle (OCP)**:
  - New years of lunar holidays (1407+) can be added simply by dropping a `hijri_months.txt` next to the executable without changing or recompiling any Rust code.
- **Liskov Substitution & Dependency Inversion**:
  - The calendar lookup operates on abstract date models (`JalaliDate`, `LunarDate`), cleanly decoupled from UI and OS subsystems.

## 4. UI Layout & View Elements

- **Tray Area**:
  - Dynamic `16x16` or `32x32` generated icon based on monitor DPI.
  - Tray Context Menu: "امروز" (Go to today), "اجرای خودکار با ویندوز" (Toggle Autostart), "درباره" (About), "خروج" (Exit).
- **Calendar Popup Window**:
  - **Header**: Navigation buttons (⟨, ⟩, «, »), current month and year title in Persian.
  - **Weekday Bar**: شنبه to جمعه with customized header styling.
  - **Grid Area**: Up to 42 cells representing the active month + leading/trailing days.
  - **Event Panel**: Daily events list, with holidays accented in red.
  - **Today Footer Bar**: Quick jump button returning to current day.

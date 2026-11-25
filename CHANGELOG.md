# Changelog

All notable changes to this project are documented here, following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [1.4.0] - 2026-11-25

### Added
- **Selectable Tray Icon Styles** persisted in Registry (`HKCU\Software\WJalaliCalendar`): Auto (large crisp day at 16px, month + day at 20px+), Always Month+Day, and Numeric (`۲۵/۵`).
- **Explorer Restart Recovery**: Registered `TaskbarCreated` message so the tray icon automatically reappears if `explorer.exe` restarts.
- **Instant System Sync**: Added `WM_SETTINGCHANGE` and `WM_TIMECHANGE` handlers for immediate Light/Dark theme and system clock updates.
- **Live Holiday Reload**: Context menu option to reload `hijri_months.txt` without restarting (`RwLock`-backed Hijri calendar).
- **Rich Popup Navigation & Clipboard**: Mouse wheel month scrolling, RTL arrow keys (`Left`/`Right`/`Up`/`Down`), clickable leading/trailing month cells, wired Converter/Details view toggle button, click-to-copy rows & `Ctrl+C` shortcut, and Windows 11 native DWM rounded corners (with graceful Win7–10 fallback).
- **Astronomical Estimate Indicator**: Displays `(تقریبی)` next to Hijri dates outside the official verified table.

### Changed
- **Modular SOLID UI Architecture**: Split the 1,474-line `calendar_window.rs` monolith into `src/ui/gdi.rs` (RAII GDI handles + DPI-cached fonts + vector icons) and `src/ui/popup/{mod,state,layout,paint}.rs` with cached month grids and unified paint/hit-test layout geometry.
- **Pixel-Exact Tray Icon Engine**: Renders directly at native `SM_CXSMICON` with negative em-height bold `Tahoma`, explicit 32-bit premultiplied alpha fixup, genuine 1-bit monochrome mask (`CreateBitmap`), and high-contrast palettes for Dark & Light taskbars.
- **Second Instance Behavior**: Launching a second instance now opens the existing calendar popup instead of showing a modal dialog.
- **Unified Jalali Core & Version Sync**: Deduplicated Borkowski/Behdad calendar logic with full boundary leap-year handling, shared `gregorian_to_jdn`, synced `Cargo.toml` / `app.manifest` / About dialog to `1.4.0`, and fixed all `cargo clippy` errors & warnings.

### Fixed
- Tray icon downscale blur at 100% DPI (16×16), invalid 32-bit mask bitmap, and zeroed GDI DIB alpha channel.
- Clicking the tray icon while the popup is open now cleanly closes it (220ms focus-loss debounce) instead of flickering and re-opening.
- Multi-monitor docking (`MonitorFromPoint` + `GetMonitorInfoW`) and live per-monitor DPI updates (`GetDpiForWindow` + `WM_DPICHANGED`).
- Stuck hover highlights when cursor exits the popup (`TrackMouseEvent` + `WM_MOUSELEAVE`).
- Header theme button hit-test offset mismatch and converter leap-year day count (`366` vs `365` with Persian digits).
- `is_autostart_enabled()` now verifies that the registered Run path actually matches the current executable path.

## [1.3.0] - 2026-11-25

### Added
- Dynamic Windows system taskbar theme detection (`src/util/theme_detect.rs`) checking `HKCU\...\Themes\Personalize\SystemUsesLightTheme`.

### Changed
- **Enhanced System Tray Icon Readability & Multi-Theme Compatibility**:
  - Significantly enlarged Persian month name typography and day numbers.
  - Full adaptive color palette support:
    - **Windows Light Taskbar**: Pure white calendar badge with high-contrast outline border, dark neutral header band, and deep rich charcoal day digits.
    - **Windows Dark Taskbar**: Graphite slate body with clean hairline border and bright white day digits.
    - **Official Holidays / Fridays**: Distinct carmine red header band with tinted body and high-contrast red numerals.
  - Automatic icon refresh on periodic date check whenever the user switches Windows theme between light and dark.

## [1.2.1] - 2026-11-21

### Changed
- Replaced the temporary text labels (`روز` / `شب`) for theme toggle with crisp DPI-scaled native GDI vector icons (Sun with 8 rays in dark mode, Crescent Moon in light mode) avoiding all emoji rendering issues.

## [1.2.0] - 2026-11-21

### Changed
- **Unslop Visual Overhaul**:
  - Restructured color system in `src/ui/theme.rs` to a single unified graphite neutral ramp with calibrated hairlines and high-contrast text roles.
  - Eliminated decorative emoji clutter from tray menus (`نمایش تقویم`, `امروز`, `اجرای خودکار با ویندوز`, `درباره`, `خروج`), converter cards, and event lists.
  - Replaced the calendar header theme toggle emoji (`☀️` / `🌙`) with clean, subtle Persian typography badges (`روز` / `شب`).
  - Redesigned holiday indicators in the day cells into centered, restrained 4px circular status pips.
  - Balanced typography weights across cells and headers for native Windows feel.

## [1.1.0] - 2026-11-21

### Added
- `data/hijri_months.txt`: official Iranian Hijri month starts (1403–1406), embedded at build time.
- Optional `hijri_months.txt` next to `wjc.exe` overrides/extends the table without recompiling.
- Astronomical crescent estimate (`src/calendar/astro.rs`) for months not in the table, so years after 1406 keep working.
- Lunar observances (Mother's Day, Laylat al-Qadr, Arafah, ...) now shown for every year.

### Changed
- Lunar holidays are defined once as Hijri-date rules instead of per-year Jalali tables.
- The Hijri date shown in calendar cells and the tooltip now follows Iran's observed calendar.
- Removed the `hijri_date` dependency.

### Fixed
- Wrong holiday dates in the previous table: Fatima martyrdom 1404 (3 Azar), Ghadir 1405 (14 Khordad),
  Arbaeen / 28 Safar / Imam Reza 1405 (13, 21, 22 Mordad), Prophet's birthday 1405 (8 Shahrivar),
  Fatima martyrdom 1405 (22 Aban), Eid al-Fitr 1405 (19–20 Esfand), Ghadir 1406 (4 Khordad),
  Fatima martyrdom 1406 (11 Aban), Imam Ali martyrdom 1406 (28 Bahman).
- Removed the incorrect extra oil-nationalization holiday on 30 Esfand in leap years.

## [1.0.0] - 2026-08-16

### Added
- System tray icon with the current Jalali date, popup month calendar, Iranian holidays, autostart.

# Changelog

All notable changes to this project are documented here, following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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

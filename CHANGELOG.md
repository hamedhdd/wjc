# Changelog

All notable changes to this project are documented here, following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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

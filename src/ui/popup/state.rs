//! State model and month-grid caching for the calendar popup window.

use crate::calendar::{
    is_official_holiday, GregorianDate, JalaliDate, LunarDate,
};
use crate::ui::gdi::PopupFonts;
use crate::ui::theme::Theme;
use crate::util::settings::{load_theme_preference, save_theme_preference, ThemePreference};
use crate::util::theme_detect::is_windows_apps_light_theme;

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
    ViewToggle,
    CopyDate,
}

#[derive(Debug, Clone)]
pub struct GridCell {
    pub day: u32,
    pub is_current_month: bool,
    pub jalali: JalaliDate,
    pub gregorian: GregorianDate,
    pub lunar: LunarDate,
    pub is_today: bool,
    pub is_holiday: bool,
}

pub struct CalendarWindowState {
    pub is_visible: bool,
    pub view_mode: ViewMode,
    pub theme_pref: ThemePreference,
    pub theme: Theme,
    pub today_jalali: JalaliDate,
    pub viewing_year: i32,
    pub viewing_month: u32,
    pub selected_date: JalaliDate,
    pub hovered_day_index: Option<usize>,
    pub hovered_button: Option<ButtonId>,
    pub hovered_converter_row: Option<usize>,
    pub dpi: u32,
    pub tracking_mouse: bool,
    pub last_hidden_ms: u64,
    pub copied_until_ms: u64,
    pub hijri_rev: u64,
    // Cached month grid (invalidated only when year/month/today/hijri_rev change)
    cached_key: Option<(i32, u32, JalaliDate, u64)>,
    cached_cells: Vec<GridCell>,
    // Cached DPI-keyed fonts
    pub fonts: Option<PopupFonts>,
}

impl CalendarWindowState {
    pub fn new(today: JalaliDate, dpi: u32) -> Self {
        let theme_pref = load_theme_preference();
        let theme = resolve_theme(theme_pref);
        let mut s = Self {
            is_visible: false,
            view_mode: ViewMode::Calendar,
            theme_pref,
            theme,
            today_jalali: today,
            viewing_year: today.year,
            viewing_month: today.month,
            selected_date: today,
            hovered_day_index: None,
            hovered_button: None,
            hovered_converter_row: None,
            dpi,
            tracking_mouse: false,
            last_hidden_ms: 0,
            copied_until_ms: 0,
            hijri_rev: 0,
            cached_key: None,
            cached_cells: Vec::with_capacity(42),
            fonts: None,
        };
        s.ensure_grid_cache();
        s
    }

    pub fn sync_theme_with_system(&mut self) -> bool {
        if self.theme_pref != ThemePreference::Auto {
            return false;
        }
        let next = resolve_theme(self.theme_pref);
        if next.is_dark != self.theme.is_dark {
            self.theme = next;
            true
        } else {
            false
        }
    }

    pub fn toggle_theme(&mut self) {
        self.theme_pref = if self.theme.is_dark {
            ThemePreference::Light
        } else {
            ThemePreference::Dark
        };
        self.theme = resolve_theme(self.theme_pref);
        save_theme_preference(self.theme_pref);
    }

    pub fn invalidate_grid_cache(&mut self) {
        self.hijri_rev = self.hijri_rev.wrapping_add(1);
        self.cached_key = None;
    }

    pub fn ensure_grid_cache(&mut self) {
        let key = (
            self.viewing_year,
            self.viewing_month,
            self.today_jalali,
            self.hijri_rev,
        );
        if self.cached_key == Some(key) {
            return;
        }
        self.cached_cells = build_month_grid(self.viewing_year, self.viewing_month, self.today_jalali);
        self.cached_key = Some(key);
    }

    pub fn cells(&self) -> &[GridCell] {
        &self.cached_cells
    }

    pub fn grid_rows(&self) -> usize {
        self.cached_cells.len().div_ceil(7).max(5)
    }

    pub fn select_date(&mut self, date: JalaliDate) {
        self.selected_date = date;
        self.viewing_year = date.year;
        self.viewing_month = date.month;
        self.ensure_grid_cache();
    }

    pub fn navigate_month(&mut self, delta: i32) {
        let base = JalaliDate::new(self.viewing_year, self.viewing_month, 1);
        let next = if delta < 0 { base.prev_month() } else { base.next_month() };
        self.viewing_year = next.year;
        self.viewing_month = next.month;
        self.ensure_grid_cache();
    }

    pub fn navigate_year(&mut self, delta: i32) {
        self.viewing_year = (self.viewing_year + delta).clamp(1, 3000);
        self.ensure_grid_cache();
    }

    pub fn converter_rows(&self) -> [(String, String); 6] {
        let j = self.selected_date;
        let g = j.to_gregorian();
        let l = LunarDate::from_gregorian(g);
        let total_days = JalaliDate::days_in_year(j.year);
        let doy = j.day_of_year();
        let lunar_val = if l.is_estimated() {
            format!("{} (تقریبی)", l.format_long_fa())
        } else {
            l.format_long_fa()
        };

        [
            (
                format!("تاریخ خورشیدی: {}", j.format_long_fa()),
                j.format_long_fa(),
            ),
            (
                format!("تاریخ میلادی: {}", g.format_long()),
                g.format_long(),
            ),
            (
                format!("تاریخ قمری: {lunar_val}"),
                l.format_long_fa(),
            ),
            (
                format!("روز هفته: {}", j.weekday().name_fa()),
                j.weekday().name_fa().to_string(),
            ),
            (
                format!(
                    "روز سال: {} از {}",
                    crate::calendar::to_persian_digits(doy.to_string()),
                    crate::calendar::to_persian_digits(total_days.to_string())
                ),
                format!("{doy}/{total_days}"),
            ),
            (
                format!(
                    "وضعیت سال {}: {}",
                    crate::calendar::to_persian_digits(j.year.to_string()),
                    if JalaliDate::is_leap_year(j.year) {
                        "سال کبیسه (۳۶۶ روز)"
                    } else {
                        "سال عادی (۳۶۵ روز)"
                    }
                ),
                j.format_numeric(),
            ),
        ]
    }
}

fn resolve_theme(pref: ThemePreference) -> Theme {
    match pref {
        ThemePreference::Dark => Theme::dark(),
        ThemePreference::Light => Theme::light(),
        ThemePreference::Auto => {
            if is_windows_apps_light_theme() {
                Theme::light()
            } else {
                Theme::dark()
            }
        }
    }
}

fn build_month_grid(year: i32, month: u32, today: JalaliDate) -> Vec<GridCell> {
    let mut cells = Vec::with_capacity(42);

    let first_day = JalaliDate::new(year, month, 1);
    let first_weekday = first_day.weekday() as usize; // 0 = Shanbeh .. 6 = Jomeh
    let days_in_cur = JalaliDate::days_in_year_month(year, month);

    let prev_m = first_day.prev_month();
    let days_in_prev = JalaliDate::days_in_year_month(prev_m.year, prev_m.month);

    for i in (0..first_weekday).rev() {
        let d = days_in_prev - i as u32;
        cells.push(make_cell(JalaliDate::new(prev_m.year, prev_m.month, d), false, today));
    }
    for d in 1..=days_in_cur {
        cells.push(make_cell(JalaliDate::new(year, month, d), true, today));
    }

    let total_cells = if cells.len() <= 35 { 35 } else { 42 };
    let next_m = first_day.next_month();
    let mut next_d = 1;
    while cells.len() < total_cells {
        cells.push(make_cell(JalaliDate::new(next_m.year, next_m.month, next_d), false, today));
        next_d += 1;
    }

    cells
}

fn make_cell(jalali: JalaliDate, is_current_month: bool, today: JalaliDate) -> GridCell {
    let gregorian = jalali.to_gregorian();
    let lunar = LunarDate::from_gregorian(gregorian);
    let is_holiday = is_official_holiday(&jalali, &lunar);
    GridCell {
        day: jalali.day,
        is_current_month,
        jalali,
        gregorian,
        lunar,
        is_today: jalali == today,
        is_holiday,
    }
}

//! Hijri lunar (Qamari) calendar as observed in Iran.
//!
//! Month starts come from the official table (`hijri_source`), and fall back to
//! the astronomical estimate in `astro` for months the table doesn't cover.

use std::sync::OnceLock;

use super::astro::{estimated_month_start, hijri_of_lunation, lunation_of};
use super::hijri_source::load_month_starts;
use super::hijri_table::{gregorian_to_jdn, MonthStarts};
use super::jalali::{to_persian_digits, GregorianDate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LunarDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

pub const LUNAR_MONTH_NAMES_FA: [&str; 12] = [
    "محرم",
    "صفر",
    "ربیع‌الاول",
    "ربیع‌الثانی",
    "جمادی‌الاول",
    "جمادی‌الثانی",
    "رجب",
    "شعبان",
    "رمضان",
    "شوال",
    "ذی‌القعده",
    "ذی‌الحجه",
];

/// Table entries further than this from the astronomical estimate are treated
/// as typos and ignored, which also keeps month starts strictly increasing.
const MAX_TABLE_DEVIATION_DAYS: i64 = 3;

/// Resolves civil days to Hijri dates using a month-start table plus estimates.
pub struct HijriCalendar {
    starts: MonthStarts,
}

impl HijriCalendar {
    pub fn new(starts: MonthStarts) -> Self {
        let starts = starts
            .into_iter()
            .filter(|&((y, m), jdn)| {
                (jdn - estimated_month_start(lunation_of(y, m))).abs() <= MAX_TABLE_DEVIATION_DAYS
            })
            .collect();
        Self { starts }
    }

    fn start_of_lunation(&self, k: i64) -> i64 {
        let key = hijri_of_lunation(k);
        self.starts
            .get(&key)
            .copied()
            .unwrap_or_else(|| estimated_month_start(k))
    }

    /// Number of days (29 or 30) in a Hijri month.
    pub fn month_length(&self, year: i32, month: u32) -> u32 {
        let k = lunation_of(year, month);
        (self.start_of_lunation(k + 1) - self.start_of_lunation(k)) as u32
    }

    pub fn date_of_jdn(&self, jdn: i64) -> LunarDate {
        // Lunation containing jdn: start from a mean-motion guess, then adjust.
        let mut k = ((jdn as f64 - 2451550.1) / 29.530588861).floor() as i64;
        while self.start_of_lunation(k) > jdn {
            k -= 1;
        }
        while self.start_of_lunation(k + 1) <= jdn {
            k += 1;
        }
        let (year, month) = hijri_of_lunation(k);
        let day = (jdn - self.start_of_lunation(k) + 1) as u32;
        LunarDate { year, month, day }
    }
}

/// The process-wide calendar, loaded once on first use.
pub fn iran_hijri() -> &'static HijriCalendar {
    static CALENDAR: OnceLock<HijriCalendar> = OnceLock::new();
    CALENDAR.get_or_init(|| HijriCalendar::new(load_month_starts()))
}

impl LunarDate {
    pub const fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    pub fn month_name(&self) -> &'static str {
        if (1..=12).contains(&self.month) {
            LUNAR_MONTH_NAMES_FA[(self.month - 1) as usize]
        } else {
            ""
        }
    }

    /// Converts a Gregorian date to the Hijri date observed in Iran.
    pub fn from_gregorian(g: GregorianDate) -> Self {
        iran_hijri().date_of_jdn(gregorian_to_jdn(g.year, g.month, g.day))
    }

    /// True if this is the last day of its month (e.g. 29 or 30 Safar).
    pub fn is_last_day_of_month(&self) -> bool {
        self.day == iran_hijri().month_length(self.year, self.month)
    }

    pub fn format_long_fa(&self) -> String {
        format!(
            "{} {} {}",
            to_persian_digits(self.day.to_string()),
            self.month_name(),
            to_persian_digits(self.year.to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::hijri_source::embedded_month_starts;
    use super::*;

    fn lunar(cal: &HijriCalendar, y: i32, m: u32, d: u32) -> (i32, u32, u32) {
        let l = cal.date_of_jdn(gregorian_to_jdn(y, m, d));
        (l.year, l.month, l.day)
    }

    #[test]
    fn table_dates() {
        let cal = HijriCalendar::new(embedded_month_starts());
        assert_eq!(lunar(&cal, 2026, 8, 16), (1448, 3, 3)); // 25 Mordad 1405
        assert_eq!(lunar(&cal, 2024, 7, 16), (1446, 1, 10)); // Ashura 1403
        assert_eq!(lunar(&cal, 2026, 3, 21), (1447, 10, 1)); // Eid al-Fitr, 1 Farvardin 1405
    }

    #[test]
    fn embedded_table_is_fully_accepted_and_consistent() {
        let starts = embedded_month_starts();
        let cal = HijriCalendar::new(starts.clone());
        assert_eq!(cal.starts.len(), starts.len(), "an entry was rejected as a typo");
        for &(y, m) in starts.keys() {
            let len = cal.month_length(y, m);
            assert!(len == 29 || len == 30, "{y}/{m} has {len} days");
        }
    }

    #[test]
    fn estimate_is_continuous_far_from_table() {
        let cal = HijriCalendar::new(MonthStarts::new());
        let mut prev = cal.date_of_jdn(gregorian_to_jdn(2030, 1, 1));
        for jdn in gregorian_to_jdn(2030, 1, 2)..gregorian_to_jdn(2034, 1, 1) {
            let cur = cal.date_of_jdn(jdn);
            if cur.day != prev.day + 1 {
                assert_eq!(cur.day, 1);
                assert!(prev.day == 29 || prev.day == 30);
            }
            prev = cur;
        }
    }

    #[test]
    fn typo_entries_are_ignored() {
        let mut starts = MonthStarts::new();
        starts.insert((1448, 3), gregorian_to_jdn(2026, 9, 30)); // a month too late
        let cal = HijriCalendar::new(starts);
        assert!(cal.starts.is_empty());
    }
}

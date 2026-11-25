//! Jalali (Solar Hijri) Calendar Conversion & Utility Functions

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JalaliDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GregorianDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    Shanbeh = 0,       // Saturday
    Yekshanbeh = 1,    // Sunday
    Doshanbeh = 2,     // Monday
    Seshanbeh = 3,     // Tuesday
    Chaharshanbeh = 4, // Wednesday
    Panjshanbeh = 5,   // Thursday
    Jomeh = 6,         // Friday (Weekend)
}

impl Weekday {
    pub fn name_fa(&self) -> &'static str {
        match self {
            Weekday::Shanbeh => "شنبه",
            Weekday::Yekshanbeh => "یک‌شنبه",
            Weekday::Doshanbeh => "دوشنبه",
            Weekday::Seshanbeh => "سه‌شنبه",
            Weekday::Chaharshanbeh => "چهارشنبه",
            Weekday::Panjshanbeh => "پنج‌شنبه",
            Weekday::Jomeh => "جمعه",
        }
    }

    pub fn short_name_fa(&self) -> &'static str {
        match self {
            Weekday::Shanbeh => "ش",
            Weekday::Yekshanbeh => "ی",
            Weekday::Doshanbeh => "د",
            Weekday::Seshanbeh => "س",
            Weekday::Chaharshanbeh => "چ",
            Weekday::Panjshanbeh => "پ",
            Weekday::Jomeh => "ج",
        }
    }

    pub fn is_weekend(&self) -> bool {
        matches!(self, Weekday::Jomeh)
    }
}

pub const JALALI_MONTH_NAMES: [&str; 12] = [
    "فروردین",
    "اردیبهشت",
    "خرداد",
    "تیر",
    "مرداد",
    "شهریور",
    "مهر",
    "آبان",
    "آذر",
    "دی",
    "بهمن",
    "اسفند",
];

pub const GREGORIAN_MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

pub const GREGORIAN_MONTH_NAMES_FA: [&str; 12] = [
    "ژانویه", "فوریه", "مارس", "آوریل", "مه", "ژوئن",
    "ژوئیه", "اوت", "سپتامبر", "اکتبر", "نوامبر", "دسامبر",
];

/// Converts English digits to Persian digits (e.g., "25/5" -> "۲۵/۵")
pub fn to_persian_digits<S: AsRef<str>>(s: S) -> String {
    s.as_ref()
        .chars()
        .map(|c| match c {
            '0' => '۰',
            '1' => '۱',
            '2' => '۲',
            '3' => '۳',
            '4' => '۴',
            '5' => '۵',
            '6' => '۶',
            '7' => '۷',
            '8' => '۸',
            '9' => '۹',
            other => other,
        })
        .collect()
}

/// Converts Persian digits to English digits
pub fn to_english_digits<S: AsRef<str>>(s: S) -> String {
    s.as_ref()
        .chars()
        .map(|c| match c {
            '۰' => '0',
            '۱' => '1',
            '۲' => '2',
            '۳' => '3',
            '۴' => '4',
            '۵' => '5',
            '۶' => '6',
            '۷' => '7',
            '۸' => '8',
            '۹' => '9',
            other => other,
        })
        .collect()
}

impl JalaliDate {
    pub const fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Determines whether the Jalali year is a leap year (Kabiseh).
    pub fn is_leap_year(jy: i32) -> bool {
        jalali_calendar(jy).is_leap
    }

    /// Total days in the given Jalali year (365 or 366).
    pub fn days_in_year(year: i32) -> u32 {
        if Self::is_leap_year(year) { 366 } else { 365 }
    }

    /// 1-based ordinal day within the Jalali year (1..=365 or 366).
    pub fn day_of_year(self) -> u32 {
        if self.month <= 6 {
            (self.month - 1) * 31 + self.day
        } else {
            186 + (self.month - 7) * 30 + self.day
        }
    }

    /// Number of days in the current Jalali month.
    pub fn days_in_month(&self) -> u32 {
        Self::days_in_year_month(self.year, self.month)
    }

    pub fn days_in_year_month(year: i32, month: u32) -> u32 {
        if (1..=6).contains(&month) {
            31
        } else if (7..=11).contains(&month) {
            30
        } else if month == 12 {
            if Self::is_leap_year(year) { 30 } else { 29 }
        } else {
            0
        }
    }

    /// Month name in Persian (e.g. "فروردین").
    pub fn month_name(&self) -> &'static str {
        if (1..=12).contains(&self.month) {
            JALALI_MONTH_NAMES[(self.month - 1) as usize]
        } else {
            ""
        }
    }

    /// Converts Gregorian date to Jalali date.
    pub fn from_gregorian(g: GregorianDate) -> Self {
        let (jy, jm, jd) = gregorian_to_jalali(g.year, g.month as i32, g.day as i32);
        Self {
            year: jy,
            month: jm as u32,
            day: jd as u32,
        }
    }

    /// Converts this Jalali date to Gregorian date.
    pub fn to_gregorian(self) -> GregorianDate {
        let (gy, gm, gd) = jalali_to_gregorian(self.year, self.month as i32, self.day as i32);
        GregorianDate {
            year: gy,
            month: gm as u32,
            day: gd as u32,
        }
    }

    /// Returns a date offset by `delta` days (positive or negative).
    pub fn add_days(self, delta: i64) -> Self {
        let jdn = jalali_to_jdn(self.year, self.month as i32, self.day as i32) + delta;
        let (jy, jm, jd) = jdn_to_jalali(jdn);
        Self::new(jy, jm as u32, jd as u32)
    }

    /// Gets day of week (0 = Shanbeh .. 6 = Jomeh).
    pub fn weekday(&self) -> Weekday {
        self.to_gregorian().weekday()
    }

    /// Returns previous month date (day clamped to target month length).
    pub fn prev_month(&self) -> Self {
        let (y, m) = if self.month == 1 {
            (self.year - 1, 12)
        } else {
            (self.year, self.month - 1)
        };
        let max_d = Self::days_in_year_month(y, m);
        Self::new(y, m, self.day.min(max_d))
    }

    /// Returns next month date (day clamped to target month length).
    pub fn next_month(&self) -> Self {
        let (y, m) = if self.month == 12 {
            (self.year + 1, 1)
        } else {
            (self.year, self.month + 1)
        };
        let max_d = Self::days_in_year_month(y, m);
        Self::new(y, m, self.day.min(max_d))
    }

    /// Formatted string: "۲۵ مرداد ۱۴۰۵"
    pub fn format_long_fa(&self) -> String {
        format!(
            "{} {} {}",
            to_persian_digits(self.day.to_string()),
            self.month_name(),
            to_persian_digits(self.year.to_string())
        )
    }

    /// Formatted string: "1405/05/25"
    pub fn format_numeric(&self) -> String {
        format!("{:04}/{:02}/{:02}", self.year, self.month, self.day)
    }

    /// Formatted short for tray icon: "25/5"
    pub fn format_tray_short(&self) -> String {
        format!("{}/{}", self.day, self.month)
    }
}

impl GregorianDate {
    pub const fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Calculate day of week: Saturday = 0, Sunday = 1, ..., Friday = 6
    pub fn weekday(&self) -> Weekday {
        let jdn = gregorian_to_jdn(self.year, self.month, self.day);
        // JDN 0 is Monday; (jdn + 2) % 7 maps Saturday -> 0 .. Friday -> 6
        match (jdn + 2).rem_euclid(7) {
            0 => Weekday::Shanbeh,
            1 => Weekday::Yekshanbeh,
            2 => Weekday::Doshanbeh,
            3 => Weekday::Seshanbeh,
            4 => Weekday::Chaharshanbeh,
            5 => Weekday::Panjshanbeh,
            _ => Weekday::Jomeh,
        }
    }

    pub fn format_long(&self) -> String {
        let month_name = if (1..=12).contains(&self.month) {
            GREGORIAN_MONTH_NAMES[(self.month - 1) as usize]
        } else {
            ""
        };
        format!("{} {} {}", self.day, month_name, self.year)
    }

    pub fn format_long_fa(&self) -> String {
        let month_name = if (1..=12).contains(&self.month) {
            GREGORIAN_MONTH_NAMES_FA[(self.month - 1) as usize]
        } else {
            ""
        };
        format!(
            "{} {} {}",
            to_persian_digits(self.day.to_string()),
            month_name,
            to_persian_digits(self.year.to_string())
        )
    }
}

// -----------------------------------------------------------------------
// Accurate Algorithms for Gregorian <-> Julian Day Number <-> Jalali
// -----------------------------------------------------------------------

const JALALI_BREAKS: [i32; 20] = [
    -61, 9, 38, 199, 426, 686, 756, 818, 1111, 1181, 1210, 1635, 2060, 2097, 2192, 2262, 2324,
    2394, 2456, 3178,
];

struct JalaliCalInfo {
    gy: i32,
    march_day: i32,
    is_leap: bool,
}

/// Core Borkowski / Behdad Jalali calendar parameters for a given Jalali year.
fn jalali_calendar(jy: i32) -> JalaliCalInfo {
    let gy = jy + 621;
    let mut leap_j = -14;
    let mut jp = JALALI_BREAKS[0];

    if jy < jp || jy >= JALALI_BREAKS[JALALI_BREAKS.len() - 1] {
        // Fallback 33-year Birashk approximation outside break table
        let rem = (jy - 474).rem_euclid(2820).rem_euclid(474);
        let is_leap = [1, 5, 9, 13, 17, 22, 26, 30].contains(&(rem % 33));
        return JalaliCalInfo {
            gy,
            march_day: 20,
            is_leap,
        };
    }

    let mut jump = 0;
    for &jm in &JALALI_BREAKS[1..] {
        jump = jm - jp;
        if jy < jm {
            break;
        }
        leap_j += (jump / 33) * 8 + (jump % 33) / 4;
        jp = jm;
    }

    let mut n = jy - jp;
    leap_j += (n / 33) * 8 + (n % 33 + 3) / 4;
    if jump % 33 == 4 && jump - n == 4 {
        leap_j += 1;
    }

    let leap_g = gy / 4 - ((gy / 100) + 1) * 3 / 4 - 150;
    let march_day = 20 + leap_j - leap_g;

    if jump - n < 6 {
        n = n - jump + ((jump + 4) / 33) * 33;
    }
    let mut leap = ((n + 1).rem_euclid(33) - 1).rem_euclid(4);
    if leap == -1 {
        leap = 4;
    }

    JalaliCalInfo {
        gy,
        march_day,
        is_leap: leap == 0,
    }
}

/// Julian Day Number of a proleptic Gregorian date.
pub fn gregorian_to_jdn(gy: i32, gm: u32, gd: u32) -> i64 {
    let (y, m, d) = (gy as i64, gm as i64, gd as i64);
    let a = (14 - m) / 12;
    let y = y + 4800 - a;
    let m = m + 12 * a - 3;
    d + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

pub fn jdn_to_gregorian(jdn: i64) -> (i32, u32, u32) {
    let a = jdn + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (146097 * b) / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;
    let gd = e - (153 * m + 2) / 5 + 1;
    let gm = m + 3 - 12 * (m / 10);
    let gy = 100 * b + d - 4800 + m / 10;
    (gy as i32, gm as u32, gd as u32)
}

pub fn jalali_to_jdn(jy: i32, jm: i32, jd: i32) -> i64 {
    let cal = jalali_calendar(jy);
    let g_jdn_march = gregorian_to_jdn(cal.gy, 3, cal.march_day as u32);
    let day_in_j_year = if jm <= 6 {
        (jm - 1) * 31 + jd
    } else {
        186 + (jm - 7) * 30 + jd
    };
    g_jdn_march + day_in_j_year as i64 - 1
}

pub fn jdn_to_jalali(jdn: i64) -> (i32, i32, i32) {
    let (gy, _, _) = jdn_to_gregorian(jdn);
    let mut jy = gy - 621;
    let cal = jalali_calendar(jy);
    let jdn_1f = gregorian_to_jdn(gy, 3, cal.march_day as u32);
    let mut k = jdn - jdn_1f;

    if k < 0 {
        jy -= 1;
        let cal_prev = jalali_calendar(jy);
        let jdn_1f_prev = gregorian_to_jdn(cal_prev.gy, 3, cal_prev.march_day as u32);
        k = jdn - jdn_1f_prev;
    }

    if k <= 185 {
        let jm = 1 + k / 31;
        let jd = 1 + (k % 31);
        (jy, jm as i32, jd as i32)
    } else {
        let k_sub = k - 186;
        let jm = 7 + k_sub / 30;
        let jd = 1 + (k_sub % 30);
        (jy, jm as i32, jd as i32)
    }
}

pub fn gregorian_to_jalali(gy: i32, gm: i32, gd: i32) -> (i32, i32, i32) {
    let jdn = gregorian_to_jdn(gy, gm as u32, gd as u32);
    jdn_to_jalali(jdn)
}

pub fn jalali_to_gregorian(jy: i32, jm: i32, jd: i32) -> (i32, i32, i32) {
    let jdn = jalali_to_jdn(jy, jm, jd);
    let (gy, gm, gd) = jdn_to_gregorian(jdn);
    (gy, gm as i32, gd as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_today_2026_08_16() {
        let g = GregorianDate::new(2026, 8, 16);
        let j = JalaliDate::from_gregorian(g);
        assert_eq!(j, JalaliDate::new(1405, 5, 25));
        assert_eq!(j.format_tray_short(), "25/5");
        assert_eq!(j.to_gregorian(), g);
    }

    #[test]
    fn test_nowruz_and_leap_years() {
        assert_eq!(JalaliDate::from_gregorian(GregorianDate::new(2024, 3, 20)), JalaliDate::new(1403, 1, 1));
        assert_eq!(JalaliDate::from_gregorian(GregorianDate::new(2025, 3, 20)), JalaliDate::new(1403, 12, 30));
        assert_eq!(JalaliDate::from_gregorian(GregorianDate::new(2025, 3, 21)), JalaliDate::new(1404, 1, 1));

        // Known Iranian leap & non-leap years plus break boundaries
        assert!(JalaliDate::is_leap_year(1399));
        assert!(JalaliDate::is_leap_year(1403));
        assert!(!JalaliDate::is_leap_year(1404));
        assert!(!JalaliDate::is_leap_year(1405));
        assert!(JalaliDate::is_leap_year(1408));
        // Consistency check: is_leap_year(y) == (jdn(y+1,1,1) - jdn(y,1,1) == 366)
        for y in [1209, 1210, 1211, 1399, 1403, 1404, 1408, 1634, 1635, 1636, 2092, 2093, 2094] {
            let len = jalali_to_jdn(y + 1, 1, 1) - jalali_to_jdn(y, 1, 1);
            assert_eq!(JalaliDate::is_leap_year(y), len == 366, "mismatch on year {y} (len={len})");
        }
    }

    #[test]
    fn test_add_days_and_day_of_year() {
        let last_1403 = JalaliDate::new(1403, 12, 30);
        assert_eq!(last_1403.day_of_year(), 366);
        assert_eq!(last_1403.add_days(1), JalaliDate::new(1404, 1, 1));
        assert_eq!(JalaliDate::new(1404, 1, 1).add_days(-1), last_1403);
    }

    #[test]
    fn test_persian_digits() {
        assert_eq!(to_persian_digits("25/5"), "۲۵/۵");
        assert_eq!(to_persian_digits("1405"), "۱۴۰۵");
        assert_eq!(to_english_digits("۲۵/۵"), "25/5");
    }
}

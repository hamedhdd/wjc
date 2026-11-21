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
    Shanbeh = 0,    // Saturday
    Yekshanbeh = 1, // Sunday
    Doshanbeh = 2,  // Monday
    Seshanbeh = 3,  // Tuesday
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
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Determines whether the Jalali year is a leap year (Kabiseh).
    /// Uses the accurate 2820-year Birashk cycle and 33-year sub-cycles.
    pub fn is_leap_year(jy: i32) -> bool {
        let breaks = [
            -61, 9, 38, 199, 426, 686, 756, 818, 1111, 1181, 1210, 1635, 2060, 2097, 2192, 2262,
            2324, 2394, 2456, 3178,
        ];
        let mut jp = breaks[0];
        let mut jump;
        let mut leap_j = -14;

        if jy < jp || jy >= breaks[breaks.len() - 1] {
            // Fallback formula for years outside break array
            let rem = (jy - 474).rem_euclid(2820).rem_euclid(474);
            return [1, 5, 9, 13, 17, 22, 26, 30].contains(&(rem % 33));
        }

        for &jm in &breaks[1..] {
            jump = jm - jp;
            if jy < jm {
                let n = jy - jp;
                let leap = leap_j + n / 33 * 8 + (n % 33 + 3) / 4;
                let prev_leap = leap_j + (n - 1) / 33 * 8 + ((n - 1) % 33 + 3) / 4;
                return leap - prev_leap == 1;
            }
            leap_j += jump / 33 * 8 + (jump % 33) / 4;
            jp = jm;
        }
        false
    }

    /// Number of days in the current Jalali month
    pub fn days_in_month(&self) -> u32 {
        Self::days_in_year_month(self.year, self.month)
    }

    pub fn days_in_year_month(year: i32, month: u32) -> u32 {
        if (1..=6).contains(&month) {
            31
        } else if (7..=11).contains(&month) {
            30
        } else if month == 12 {
            if Self::is_leap_year(year) {
                30
            } else {
                29
            }
        } else {
            0
        }
    }

    /// Month name in Persian (e.g. "فروردین")
    pub fn month_name(&self) -> &'static str {
        if self.month >= 1 && self.month <= 12 {
            JALALI_MONTH_NAMES[(self.month - 1) as usize]
        } else {
            ""
        }
    }

    /// Converts Gregorian date to Jalali date
    pub fn from_gregorian(g: GregorianDate) -> Self {
        let (jy, jm, jd) = gregorian_to_jalali(g.year, g.month as i32, g.day as i32);
        Self {
            year: jy,
            month: jm as u32,
            day: jd as u32,
        }
    }

    /// Converts this Jalali date to Gregorian date
    pub fn to_gregorian(&self) -> GregorianDate {
        let (gy, gm, gd) = jalali_to_gregorian(self.year, self.month as i32, self.day as i32);
        GregorianDate {
            year: gy,
            month: gm as u32,
            day: gd as u32,
        }
    }

    /// Gets day of week (0 = Shanbeh .. 6 = Jomeh)
    pub fn weekday(&self) -> Weekday {
        let g = self.to_gregorian();
        g.weekday()
    }

    /// Returns previous month date
    pub fn prev_month(&self) -> Self {
        if self.month == 1 {
            let y = self.year - 1;
            let m = 12;
            let max_d = Self::days_in_year_month(y, m);
            Self::new(y, m, self.day.min(max_d))
        } else {
            let m = self.month - 1;
            let max_d = Self::days_in_year_month(self.year, m);
            Self::new(self.year, m, self.day.min(max_d))
        }
    }

    /// Returns next month date
    pub fn next_month(&self) -> Self {
        if self.month == 12 {
            let y = self.year + 1;
            let m = 1;
            let max_d = Self::days_in_year_month(y, m);
            Self::new(y, m, self.day.min(max_d))
        } else {
            let m = self.month + 1;
            let max_d = Self::days_in_year_month(self.year, m);
            Self::new(self.year, m, self.day.min(max_d))
        }
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
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Calculate day of week: Saturday = 0, Sunday = 1, ..., Friday = 6
    pub fn weekday(&self) -> Weekday {
        let jdn = gregorian_to_jdn(self.year, self.month as i32, self.day as i32);
        // JDN modulo 7: JDN 0 is Monday, (jdn + 2) % 7 -> Saturday is 0
        let w = (jdn + 2).rem_euclid(7);
        match w {
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
        let month_name = if self.month >= 1 && self.month <= 12 {
            GREGORIAN_MONTH_NAMES[(self.month - 1) as usize]
        } else {
            ""
        };
        format!("{} {} {}", self.day, month_name, self.year)
    }

    pub fn format_long_fa(&self) -> String {
        let month_name = if self.month >= 1 && self.month <= 12 {
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

fn gregorian_to_jdn(gy: i32, gm: i32, gd: i32) -> i32 {
    let a = (14 - gm) / 12;
    let y = gy + 4800 - a;
    let m = gm + 12 * a - 3;
    gd + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

fn jdn_to_gregorian(jdn: i32) -> (i32, i32, i32) {
    let a = jdn + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (146097 * b) / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;
    let gd = e - (153 * m + 2) / 5 + 1;
    let gm = m + 3 - 12 * (m / 10);
    let gy = 100 * b + d - 4800 + m / 10;
    (gy, gm, gd)
}

fn jalali_to_jdn(jy: i32, jm: i32, jd: i32) -> i32 {
    let breaks = [
        -61, 9, 38, 199, 426, 686, 756, 818, 1111, 1181, 1210, 1635, 2060, 2097, 2192, 2262,
        2324, 2394, 2456, 3178,
    ];
    let mut jp = breaks[0];
    let mut jump;
    let mut leap_j = -14;

    let gy = jy + 621;
    let mut march_day = 0;

    for &jm_b in &breaks[1..] {
        jump = jm_b - jp;
        if jy < jm_b {
            let n = jy - jp;
            let leap = leap_j + n / 33 * 8 + (n % 33 + 3) / 4;
            let leap_g = gy / 4 - ((gy / 100) + 1) * 3 / 4 - 150;
            march_day = 20 + leap - leap_g;
            break;
        }
        leap_j += jump / 33 * 8 + (jump % 33) / 4;
        jp = jm_b;
    }

    if march_day == 0 {
        // Fallback
        march_day = 20;
    }

    let g_jdn_march = gregorian_to_jdn(gy, 3, march_day);
    let day_in_j_year = if jm <= 6 {
        (jm - 1) * 31 + jd
    } else {
        6 * 31 + (jm - 7) * 30 + jd
    };

    g_jdn_march + day_in_j_year - 1
}

fn jdn_to_jalali(jdn: i32) -> (i32, i32, i32) {
    let (gy, _, _) = jdn_to_gregorian(jdn);
    let mut jy = gy - 621;

    let breaks = [
        -61, 9, 38, 199, 426, 686, 756, 818, 1111, 1181, 1210, 1635, 2060, 2097, 2192, 2262,
        2324, 2394, 2456, 3178,
    ];
    let mut jp = breaks[0];
    let mut jump;
    let mut leap_j = -14;
    let mut march_day = 20;

    for &jm_b in &breaks[1..] {
        jump = jm_b - jp;
        if jy < jm_b {
            let n = jy - jp;
            let leap = leap_j + n / 33 * 8 + (n % 33 + 3) / 4;
            let leap_g = gy / 4 - ((gy / 100) + 1) * 3 / 4 - 150;
            march_day = 20 + leap - leap_g;
            break;
        }
        leap_j += jump / 33 * 8 + (jump % 33) / 4;
        jp = jm_b;
    }

    let g_jdn_march = gregorian_to_jdn(gy, 3, march_day);
    let mut k = jdn - g_jdn_march;

    if k >= 0 {
        if k <= 185 {
            let jm = 1 + k / 31;
            let jd = 1 + (k % 31);
            return (jy, jm, jd);
        } else {
            k -= 186;
            let jm = 7 + k / 30;
            let jd = 1 + (k % 30);
            return (jy, jm, jd);
        }
    } else {
        jy -= 1;
        let _k_prev = k + 179 + if JalaliDate::is_leap_year(jy) { 1 } else { 0 };
        let mut jp_p = breaks[0];
        let mut leap_j_p = -14;
        let mut march_day_prev = 20;
        let gy_prev = jy + 621;

        for &jm_b in &breaks[1..] {
            let jump_p = jm_b - jp_p;
            if jy < jm_b {
                let n = jy - jp_p;
                let leap = leap_j_p + n / 33 * 8 + (n % 33 + 3) / 4;
                let leap_g = gy_prev / 4 - ((gy_prev / 100) + 1) * 3 / 4 - 150;
                march_day_prev = 20 + leap - leap_g;
                break;
            }
            leap_j_p += jump_p / 33 * 8 + (jump_p % 33) / 4;
            jp_p = jm_b;
        }
        let g_jdn_march_prev = gregorian_to_jdn(gy_prev, 3, march_day_prev);
        let k_actual = jdn - g_jdn_march_prev;

        if k_actual <= 185 {
            let jm = 1 + k_actual / 31;
            let jd = 1 + (k_actual % 31);
            return (jy, jm, jd);
        } else {
            let k_sub = k_actual - 186;
            let jm = 7 + k_sub / 30;
            let jd = 1 + (k_sub % 30);
            return (jy, jm, jd);
        }
    }
}

pub fn gregorian_to_jalali(gy: i32, gm: i32, gd: i32) -> (i32, i32, i32) {
    let jdn = gregorian_to_jdn(gy, gm, gd);
    jdn_to_jalali(jdn)
}

pub fn jalali_to_gregorian(jy: i32, jm: i32, jd: i32) -> (i32, i32, i32) {
    let jdn = jalali_to_jdn(jy, jm, jd);
    jdn_to_gregorian(jdn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_today_2026_08_16() {
        // 2026-08-16 should be 1405-05-25 (Mordad 25, 1405)
        let g = GregorianDate::new(2026, 8, 16);
        let j = JalaliDate::from_gregorian(g);
        assert_eq!(j.year, 1405);
        assert_eq!(j.month, 5);
        assert_eq!(j.day, 25);
        assert_eq!(j.format_tray_short(), "25/5");

        let g_back = j.to_gregorian();
        assert_eq!(g_back.year, 2026);
        assert_eq!(g_back.month, 8);
        assert_eq!(g_back.day, 16);
    }

    #[test]
    fn test_nowruz_conversions() {
        // 2024-03-20 was 1403-01-01
        let g = GregorianDate::new(2024, 3, 20);
        let j = JalaliDate::from_gregorian(g);
        assert_eq!(j.year, 1403);
        assert_eq!(j.month, 1);
        assert_eq!(j.day, 1);

        // 2024 is a leap year; 1403 is a 366-day leap year
        // 2025-03-20 was 1403-12-30 (last day of leap year 1403)
        let g_last = GregorianDate::new(2025, 3, 20);
        let j_last = JalaliDate::from_gregorian(g_last);
        assert_eq!(j_last.year, 1403);
        assert_eq!(j_last.month, 12);
        assert_eq!(j_last.day, 30);

        // 2025-03-21 was 1404-01-01 (Nowruz 1404)
        let g2 = GregorianDate::new(2025, 3, 21);
        let j2 = JalaliDate::from_gregorian(g2);
        assert_eq!(j2.year, 1404);
        assert_eq!(j2.month, 1);
        assert_eq!(j2.day, 1);
    }

    #[test]
    fn test_persian_digits() {
        assert_eq!(to_persian_digits("25/5"), "۲۵/۵");
        assert_eq!(to_persian_digits("1405"), "۱۴۰۵");
        assert_eq!(to_english_digits("۲۵/۵"), "25/5");
    }
}

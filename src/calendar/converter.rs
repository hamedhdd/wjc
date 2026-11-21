//! Date Converter utilities across Jalali, Gregorian, and Lunar Hijri calendars

use super::jalali::{GregorianDate, JalaliDate};
use super::lunar::LunarDate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullDateInfo {
    pub jalali: JalaliDate,
    pub gregorian: GregorianDate,
    pub lunar: LunarDate,
}

impl FullDateInfo {
    pub fn from_gregorian(g: GregorianDate) -> Self {
        let jalali = JalaliDate::from_gregorian(g);
        let lunar = LunarDate::from_gregorian(g);
        Self {
            jalali,
            gregorian: g,
            lunar,
        }
    }

    pub fn from_jalali(j: JalaliDate) -> Self {
        let gregorian = j.to_gregorian();
        let lunar = LunarDate::from_gregorian(gregorian);
        Self {
            jalali: j,
            gregorian,
            lunar,
        }
    }
}

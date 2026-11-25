//! Iranian holidays and observances.
//!
//! Solar holidays are fixed Jalali dates. Lunar holidays are fixed Hijri dates;
//! because `LunarDate` follows Iran's officially observed month starts (see
//! `lunar.rs` and `data/hijri_months.txt`), no per-year holiday table is needed.

use super::jalali::JalaliDate;
use super::lunar::LunarDate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarEvent {
    pub title: &'static str,
    pub is_holiday: bool,
}

impl CalendarEvent {
    pub const fn holiday(title: &'static str) -> Self {
        Self { title, is_holiday: true }
    }
    pub const fn event(title: &'static str) -> Self {
        Self { title, is_holiday: false }
    }
}

/// When a lunar observance falls within its Hijri month.
#[derive(Clone, Copy)]
enum LunarDay {
    Day(u32),
    /// Last day of the month (29th or 30th), e.g. Imam Reza's martyrdom in Safar.
    Last,
}

struct LunarRule {
    month: u32,
    day: LunarDay,
    event: CalendarEvent,
}

const fn rule(month: u32, day: u32, event: CalendarEvent) -> LunarRule {
    LunarRule { month, day: LunarDay::Day(day), event }
}

const H: fn(&'static str) -> CalendarEvent = CalendarEvent::holiday;
const E: fn(&'static str) -> CalendarEvent = CalendarEvent::event;

fn lunar_rules() -> [LunarRule; 25] {
    [
        rule(1, 9, H("تاسوعای حسینی")),
        rule(1, 10, H("عاشورای حسینی")),
        rule(2, 20, H("اربعین حسینی")),
        rule(2, 28, H("رحلت پیامبر اکرم (ص) و شهادت امام حسن مجتبی (ع)")),
        LunarRule { month: 2, day: LunarDay::Last, event: H("شهادت حضرت امام رضا (ع)") },
        rule(3, 8, H("شهادت امام حسن عسکری (ع)")),
        rule(3, 17, H("ولادت حضرت رسول اکرم (ص) و امام جعفر صادق (ع)")),
        rule(5, 5, E("ولادت حضرت زینب (س) و روز پرستار")),
        rule(6, 3, H("شهادت حضرت فاطمه زهرا (س)")),
        rule(6, 20, E("ولادت حضرت فاطمه زهرا (س) و روز مادر")),
        rule(7, 13, H("ولادت حضرت امام علی (ع) و روز پدر")),
        rule(7, 27, H("مبعث حضرت رسول اکرم (ص)")),
        rule(8, 3, E("ولادت امام حسین (ع) و روز پاسدار")),
        rule(8, 4, E("ولادت حضرت ابوالفضل (ع) و روز جانباز")),
        rule(8, 15, H("ولادت حضرت قائم (عج) - نیمه شعبان")),
        rule(9, 19, E("ضربت خوردن حضرت علی (ع) / شب قدر")),
        rule(9, 21, H("شهادت حضرت علی (ع)")),
        rule(9, 23, E("شب قدر")),
        rule(10, 1, H("عید سعید فطر")),
        rule(10, 2, H("تعطیل به مناسبت عید سعید فطر")),
        rule(10, 25, H("شهادت امام جعفر صادق (ع)")),
        rule(11, 11, E("ولادت حضرت امام رضا (ع)")),
        rule(12, 9, E("روز عرفه")),
        rule(12, 10, H("عید سعید قربان")),
        rule(12, 18, H("عید سعید غدیر خم")),
    ]
}

fn lunar_events(l: &LunarDate, out: &mut Vec<CalendarEvent>) {
    for r in lunar_rules() {
        let matches = r.month == l.month
            && match r.day {
                LunarDay::Day(d) => d == l.day,
                LunarDay::Last => l.is_last_day_of_month(),
            };
        if matches {
            out.push(r.event);
        }
    }
}

fn solar_events(j: &JalaliDate, out: &mut Vec<CalendarEvent>) {
    let ev = match (j.month, j.day) {
        (1, 1) => H("جشن نوروز / سال نو"),
        (1, 2) | (1, 3) | (1, 4) => H("عید نوروز"),
        (1, 12) => H("روز جمهوری اسلامی ایران"),
        (1, 13) => H("روز طبیعت (سیزده‌بدر)"),
        (1, 18) => E("روز بزرگداشت عطار نیشابوری"),
        (1, 25) => E("روز بزرگداشت سعدی"),
        (2, 10) => E("روز ملی خلیج فارس"),
        (2, 12) => E("روز معلم و بزرگداشت شهید مطهری"),
        (2, 25) => E("روز بزرگداشت فردوسی"),
        (3, 3) => E("روز آزادسازی خرمشهر"),
        (3, 14) => H("رحلت حضرت امام خمینی (ره)"),
        (3, 15) => H("قیام خونین ۱۵ خرداد"),
        (5, 17) => E("روز خبرنگار"),
        (6, 1) => E("روز پزشک و بزرگداشت ابوعلی سینا"),
        (6, 27) => E("روز شعر و ادب فارسی و بزرگداشت شهریار"),
        (7, 8) => E("روز بزرگداشت مولوی"),
        (7, 20) => E("روز بزرگداشت حافظ"),
        (8, 7) => E("روز بزرگداشت کوروش بزرگ"),
        (8, 24) => E("روز کتاب و کتابخوانی"),
        (9, 16) => E("روز دانشجو"),
        (9, 25) => E("روز پژوهش"),
        (9, 30) => E("شب یلدا"),
        (10, 5) => E("سالروز زلزله بم / روز ایمنی"),
        (11, 22) => H("سالروز پیروزی انقلاب اسلامی"),
        (12, 5) => E("روز مهندس و بزرگداشت خواجه نصیرالدین طوسی"),
        (12, 15) => E("روز درختکاری"),
        (12, 29) => H("روز ملی شدن صنعت نفت ایران"),
        _ => return,
    };
    out.push(ev);
}

/// All events and holidays for a day.
pub fn get_events_for_date(j: &JalaliDate, l: &LunarDate) -> Vec<CalendarEvent> {
    let mut events = Vec::new();
    solar_events(j, &mut events);
    lunar_events(l, &mut events);
    events
}

/// True if the day is a weekend or an official public holiday (red day).
pub fn is_official_holiday(j: &JalaliDate, l: &LunarDate) -> bool {
    j.weekday().is_weekend() || get_events_for_date(j, l).iter().any(|e| e.is_holiday)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn events(y: i32, m: u32, d: u32) -> Vec<CalendarEvent> {
        let j = JalaliDate::new(y, m, d);
        get_events_for_date(&j, &LunarDate::from_gregorian(j.to_gregorian()))
    }

    fn has(y: i32, m: u32, d: u32, needle: &str) -> bool {
        events(y, m, d).iter().any(|e| e.is_holiday && e.title.contains(needle))
    }

    /// (jalali date, part of the holiday title) from Iran's official calendars.
    const OFFICIAL: &[(i32, u32, u32, &str)] = &[
        // 1403
        (1403, 1, 22, "فطر"), (1403, 2, 15, "صادق"), (1403, 3, 28, "قربان"), (1403, 4, 5, "غدیر"),
        (1403, 4, 25, "تاسوعا"), (1403, 4, 26, "عاشورا"), (1403, 6, 4, "اربعین"),
        (1403, 6, 12, "رحلت پیامبر"), (1403, 6, 14, "امام رضا"), (1403, 6, 22, "عسکری"),
        (1403, 6, 31, "ولادت حضرت رسول"), (1403, 9, 15, "فاطمه"), (1403, 10, 25, "امام علی"),
        (1403, 11, 9, "مبعث"), (1403, 11, 26, "قائم"),
        // 1404
        (1404, 1, 11, "فطر"), (1404, 2, 4, "صادق"), (1404, 3, 16, "قربان"), (1404, 3, 24, "غدیر"),
        (1404, 4, 14, "تاسوعا"), (1404, 4, 15, "عاشورا"), (1404, 5, 23, "اربعین"),
        (1404, 5, 31, "رحلت پیامبر"), (1404, 6, 2, "امام رضا"), (1404, 6, 10, "عسکری"),
        (1404, 6, 19, "ولادت حضرت رسول"), (1404, 9, 3, "فاطمه"), (1404, 10, 13, "امام علی"),
        (1404, 10, 27, "مبعث"), (1404, 11, 15, "قائم"), (1404, 12, 20, "شهادت حضرت علی"),
        // 1405
        (1405, 1, 1, "فطر"), (1405, 1, 25, "صادق"), (1405, 3, 6, "قربان"), (1405, 3, 14, "غدیر"),
        (1405, 4, 4, "تاسوعا"), (1405, 4, 5, "عاشورا"), (1405, 5, 13, "اربعین"),
        (1405, 5, 21, "رحلت پیامبر"), (1405, 5, 22, "امام رضا"), (1405, 5, 30, "عسکری"),
        (1405, 6, 8, "ولادت حضرت رسول"), (1405, 8, 22, "فاطمه"), (1405, 10, 2, "امام علی"),
        (1405, 10, 16, "مبعث"), (1405, 11, 4, "قائم"), (1405, 12, 9, "شهادت حضرت علی"),
        (1405, 12, 19, "فطر"), (1405, 12, 20, "فطر"),
        // 1406
        (1406, 1, 14, "صادق"), (1406, 2, 27, "قربان"), (1406, 3, 4, "غدیر"),
        (1406, 3, 24, "تاسوعا"), (1406, 3, 25, "عاشورا"), (1406, 5, 3, "اربعین"),
        (1406, 8, 11, "فاطمه"), (1406, 11, 28, "شهادت حضرت علی"), (1406, 12, 8, "فطر"),
        (1406, 12, 9, "فطر"),
    ];

    #[test]
    fn official_lunar_holidays() {
        let wrong: Vec<_> = OFFICIAL
            .iter()
            .filter(|(y, m, d, t)| !has(*y, *m, *d, t))
            .collect();
        assert!(wrong.is_empty(), "missing official holidays: {wrong:?}");
    }

    #[test]
    fn no_neighbouring_duplicates() {
        // The day before and after each official lunar holiday must not carry it too.
        // Eid al-Fitr is a two-day holiday, so its neighbour legitimately mentions it.
        for &(y, m, d, t) in OFFICIAL.iter().filter(|o| o.3 != "فطر") {
            let j = JalaliDate::new(y, m, d);
            for delta in [-1i64, 1] {
                let g = j.to_gregorian();
                let jdn = super::super::hijri_table::gregorian_to_jdn(g.year, g.month, g.day) + delta;
                let l = super::super::lunar::with_iran_hijri(|cal| cal.date_of_jdn(jdn));
                let mut evs = Vec::new();
                lunar_events(&l, &mut evs);
                let same_title = OFFICIAL.iter().any(|o| {
                    let oj = JalaliDate::new(o.0, o.1, o.2).to_gregorian();
                    super::super::hijri_table::gregorian_to_jdn(oj.year, oj.month, oj.day) == jdn
                        && o.3 == t
                });
                if !same_title {
                    assert!(
                        !evs.iter().any(|e| e.is_holiday && e.title.contains(t)),
                        "{t} also appears next to {y}/{m}/{d}"
                    );
                }
            }
        }
    }

    #[test]
    fn solar_fixed_holidays() {
        for (m, d) in [(1, 1), (1, 4), (1, 12), (1, 13), (3, 14), (3, 15), (11, 22), (12, 29)] {
            let j = JalaliDate::new(1405, m, d);
            assert!(is_official_holiday(&j, &LunarDate::from_gregorian(j.to_gregorian())));
        }
    }
}

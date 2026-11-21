//! Parser for the Hijri month-start table (`hijri_months.txt`).
//!
//! Line format: `<hijri year>/<hijri month>  <jalali yyyy/mm/dd>   # optional comment`

use std::collections::BTreeMap;

use super::jalali::jalali_to_gregorian;

/// Day 1 of each listed Hijri month, as a Julian Day Number.
pub type MonthStarts = BTreeMap<(i32, u32), i64>;

/// Julian Day Number of a proleptic Gregorian date.
pub fn gregorian_to_jdn(year: i32, month: u32, day: u32) -> i64 {
    let (y, m, d) = (year as i64, month as i64, day as i64);
    let a = (14 - m) / 12;
    let y = y + 4800 - a;
    let m = m + 12 * a - 3;
    d + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

pub fn jalali_to_jdn(year: i32, month: u32, day: u32) -> i64 {
    let (gy, gm, gd) = jalali_to_gregorian(year, month as i32, day as i32);
    gregorian_to_jdn(gy, gm as u32, gd as u32)
}

fn parse_pair(s: &str) -> Option<(i32, u32)> {
    let (a, b) = s.split_once('/')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

fn parse_line(line: &str) -> Option<((i32, u32), i64)> {
    let content = line.split('#').next()?.trim();
    if content.is_empty() {
        return None;
    }
    let mut parts = content.split_whitespace();
    let (hy, hm) = parse_pair(parts.next()?)?;
    let mut date = parts.next()?.splitn(3, '/');
    let jy: i32 = date.next()?.parse().ok()?;
    let jm: u32 = date.next()?.parse().ok()?;
    let jd: u32 = date.next()?.parse().ok()?;
    let valid = (1..=12).contains(&hm) && (1..=12).contains(&jm) && (1..=31).contains(&jd);
    valid.then(|| ((hy, hm), jalali_to_jdn(jy, jm, jd)))
}

/// Parses the table. Malformed lines are skipped.
pub fn parse_month_starts(text: &str) -> MonthStarts {
    text.lines().filter_map(parse_line).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_entries_and_skips_comments() {
        let t = "# header\n\n1448/03  1405/05/23   # note\nbad line\n1448/13 1405/01/01\n";
        let m = parse_month_starts(t);
        assert_eq!(m.len(), 1);
        assert_eq!(m[&(1448, 3)], gregorian_to_jdn(2026, 8, 14));
    }

    #[test]
    fn jdn_known_value() {
        assert_eq!(gregorian_to_jdn(2000, 1, 1), 2451545);
    }
}

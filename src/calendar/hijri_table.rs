//! Parser for the Hijri month-start table (`hijri_months.txt`).
//!
//! Line format: `<hijri year>/<hijri month>  <jalali yyyy/mm/dd>   # optional comment`

use std::collections::BTreeMap;

pub use super::jalali::gregorian_to_jdn;
use super::jalali::jalali_to_jdn;

/// Day 1 of each listed Hijri month, as a Julian Day Number.
pub type MonthStarts = BTreeMap<(i32, u32), i64>;

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
    valid.then(|| ((hy, hm), jalali_to_jdn(jy, jm as i32, jd as i32)))
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

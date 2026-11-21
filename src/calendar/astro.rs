//! Astronomical estimate of Hijri month starts as observed in Iran.
//!
//! Used only for months that are not listed in the official month-start table
//! (`data/hijri_months.txt` or the user's override file). The estimate is:
//!   1. compute the conjunction (new moon) with Meeus, "Astronomical Algorithms" ch. 49;
//!   2. find the first evening in Tehran where the moon is at least
//!      `MIN_AGE_HOURS` old at sunset;
//!   3. the month starts on the following day.
//!
//! The age threshold was calibrated against Iran's official calendars for
//! 1403–1406 and matches most months; it can still be off by one day, so the
//! official table always takes precedence.

/// Tehran, used as the reference location for crescent sighting.
const LAT_DEG: f64 = 35.69;
const LON_DEG: f64 = 51.39;
/// Minimum moon age at sunset (hours) for the crescent to count as sighted.
const MIN_AGE_HOURS: f64 = 18.0;

/// Lunation number (Meeus `k`, 0 = new moon of 6 Jan 2000) of Muharram 1446.
const K_MUHARRAM_1446: i64 = 303;

/// Meeus lunation number for a Hijri month.
pub fn lunation_of(hijri_year: i32, hijri_month: u32) -> i64 {
    K_MUHARRAM_1446 + (hijri_year as i64 - 1446) * 12 + (hijri_month as i64 - 1)
}

/// Hijri (year, month) of a Meeus lunation number.
pub fn hijri_of_lunation(k: i64) -> (i32, u32) {
    let idx = k - K_MUHARRAM_1446;
    let year = 1446 + idx.div_euclid(12);
    let month = idx.rem_euclid(12) + 1;
    (year as i32, month as u32)
}

/// Julian Ephemeris Day of the true new moon for lunation `k`.
pub fn new_moon_jde(k: i64) -> f64 {
    let k = k as f64;
    let t = k / 1236.85;
    let t2 = t * t;
    let jde = 2451550.09766 + 29.530588861 * k + 0.00015437 * t2 - 0.000000150 * t2 * t
        + 0.00000000073 * t2 * t2;
    let e = 1.0 - 0.002516 * t - 0.0000074 * t2;
    let m = (2.5534 + 29.10535670 * k - 0.0000014 * t2).to_radians();
    let mp = (201.5643 + 385.81693528 * k + 0.0107582 * t2).to_radians();
    let f = (160.7108 + 390.67050284 * k - 0.0016118 * t2).to_radians();
    let om = (124.7746 - 1.56375588 * k + 0.0020672 * t2).to_radians();
    let c = -0.40720 * mp.sin()
        + 0.17241 * e * m.sin()
        + 0.01608 * (2.0 * mp).sin()
        + 0.01039 * (2.0 * f).sin()
        + 0.00739 * e * (mp - m).sin()
        - 0.00514 * e * (mp + m).sin()
        + 0.00208 * e * e * (2.0 * m).sin()
        - 0.00111 * (mp - 2.0 * f).sin()
        - 0.00057 * (mp + 2.0 * f).sin()
        + 0.00056 * e * (2.0 * mp + m).sin()
        - 0.00042 * (3.0 * mp).sin()
        + 0.00042 * e * (m + 2.0 * f).sin()
        + 0.00038 * e * (m - 2.0 * f).sin()
        - 0.00024 * e * (2.0 * mp - m).sin()
        - 0.00017 * om.sin();
    jde + c
}

/// Julian Date (UT) of sunset in Tehran on the civil day with number `jdn`.
fn sunset_jd(jdn: i64) -> f64 {
    let n = (jdn - 2451545) as f64 + 0.0008;
    let js = n - LON_DEG / 360.0;
    let m = (357.5291 + 0.98560028 * js).rem_euclid(360.0);
    let mr = m.to_radians();
    let c = 1.9148 * mr.sin() + 0.02 * (2.0 * mr).sin() + 0.0003 * (3.0 * mr).sin();
    let lam = (m + c + 180.0 + 102.9372).rem_euclid(360.0).to_radians();
    let transit = 2451545.0 + js + 0.0053 * mr.sin() - 0.0069 * (2.0 * lam).sin();
    let dec = (lam.sin() * 23.44f64.to_radians().sin()).asin();
    let lat = LAT_DEG.to_radians();
    let cos_w = ((-0.833f64).to_radians().sin() - lat.sin() * dec.sin()) / (lat.cos() * dec.cos());
    transit + cos_w.acos().to_degrees() / 360.0
}

/// Estimated civil day number (JDN) of day 1 of the month for lunation `k`.
pub fn estimated_month_start(k: i64) -> i64 {
    let conjunction = new_moon_jde(k);
    let mut day = (conjunction + 0.5).floor() as i64;
    loop {
        let age_hours = (sunset_jd(day) - conjunction) * 24.0;
        if age_hours >= MIN_AGE_HOURS {
            return day + 1;
        }
        day += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lunation_round_trip() {
        for k in 250..400 {
            let (y, m) = hijri_of_lunation(k);
            assert_eq!(lunation_of(y, m), k);
        }
        assert_eq!(hijri_of_lunation(303), (1446, 1));
        assert_eq!(hijri_of_lunation(302), (1445, 12));
    }

    #[test]
    fn new_moon_matches_known_conjunction() {
        // New moon of 5 Jul 2024 ~22:57 UT => JD 2460497.456
        assert!((new_moon_jde(303) - 2460497.456).abs() < 0.01);
    }
}

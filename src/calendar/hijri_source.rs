//! Loads the Hijri month-start table: the copy embedded at build time, then an
//! optional `hijri_months.txt` next to the executable that overrides it.

use std::path::PathBuf;

use super::hijri_table::{parse_month_starts, MonthStarts};

const EMBEDDED_TABLE: &str = include_str!("../../data/hijri_months.txt");
pub const OVERRIDE_FILE_NAME: &str = "hijri_months.txt";

fn override_path() -> Option<PathBuf> {
    Some(std::env::current_exe().ok()?.parent()?.join(OVERRIDE_FILE_NAME))
}

/// Embedded table merged with the user's override file, if present.
pub fn load_month_starts() -> MonthStarts {
    let mut starts = parse_month_starts(EMBEDDED_TABLE);
    if let Some(text) = override_path().and_then(|p| std::fs::read_to_string(p).ok()) {
        starts.extend(parse_month_starts(text.trim_start_matches('\u{feff}')));
    }
    starts
}

/// The embedded table only (used by tests).
pub fn embedded_month_starts() -> MonthStarts {
    parse_month_starts(EMBEDDED_TABLE)
}

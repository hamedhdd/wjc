//! Visual Theme and Color Palette for Jalali Calendar

use windows::Win32::Foundation::COLORREF;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub is_dark: bool,
    pub background: COLORREF,
    pub surface: COLORREF,
    pub surface_hover: COLORREF,
    pub surface_selected: COLORREF,
    pub border: COLORREF,
    pub primary: COLORREF,
    pub primary_light: COLORREF,
    pub text_primary: COLORREF,
    pub text_secondary: COLORREF,
    pub text_muted: COLORREF,
    pub holiday_red: COLORREF,
    pub holiday_bg: COLORREF,
    pub today_ring: COLORREF,
    pub event_accent: COLORREF,
    pub header_bg: COLORREF,
}

pub const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF(r as u32 | ((g as u32) << 8) | ((b as u32) << 16))
}

impl Theme {
    /// Modern dark theme (Deep slate & indigo)
    pub const fn dark() -> Self {
        Self {
            is_dark: true,
            background: rgb(24, 26, 32),          // Deep dark slate
            surface: rgb(34, 37, 46),             // Card background
            surface_hover: rgb(46, 51, 64),       // Hover state
            surface_selected: rgb(55, 65, 81),    // Selected day cell
            border: rgb(50, 54, 66),              // Borders & dividers
            primary: rgb(99, 102, 241),           // Indigo accent
            primary_light: rgb(129, 140, 248),    // Bright indigo
            text_primary: rgb(243, 244, 246),     // Bright white-grey
            text_secondary: rgb(156, 163, 175),   // Muted grey
            text_muted: rgb(107, 114, 128),       // Dim text
            holiday_red: rgb(248, 113, 113),      // Vibrant holiday red
            holiday_bg: rgb(69, 26, 26),          // Subtle red card highlight
            today_ring: rgb(56, 189, 248),        // Cyan / Teal today ring
            event_accent: rgb(251, 191, 36),      // Amber event accent
            header_bg: rgb(30, 32, 40),           // Slightly darker header
        }
    }

    /// Modern crisp light theme
    pub const fn light() -> Self {
        Self {
            is_dark: false,
            background: rgb(248, 250, 252),       // Light crisp slate
            surface: rgb(255, 255, 255),          // Pure white card
            surface_hover: rgb(241, 245, 249),    // Light hover
            surface_selected: rgb(224, 231, 255), // Indigo tint selected
            border: rgb(226, 232, 240),           // Clean border
            primary: rgb(79, 70, 229),            // Royal indigo
            primary_light: rgb(99, 102, 241),     // Bright indigo
            text_primary: rgb(15, 23, 42),        // Dark slate text
            text_secondary: rgb(71, 85, 105),     // Medium slate
            text_muted: rgb(148, 163, 184),       // Light muted text
            holiday_red: rgb(220, 38, 38),        // Official holiday red
            holiday_bg: rgb(254, 242, 242),       // Light red tint
            today_ring: rgb(2, 132, 199),         // Deep cyan today ring
            event_accent: rgb(217, 119, 6),       // Amber event accent
            header_bg: rgb(241, 245, 249),
        }
    }
}

//! Visual Theme and Color Palette for Jalali Calendar
//! Follows unslop & impeccable design principles:
//! - Single unified graphite neutral ramp
//! - Three clear text roles (primary, secondary, muted)
//! - One quiet accent for active selection
//! - Dedicated restrained carmine for holidays

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
    /// Refined dark theme: Quiet graphite ramp, subtle hairline borders, clear typography
    pub const fn dark() -> Self {
        Self {
            is_dark: true,
            background: rgb(20, 20, 24),          // Base graphite (#141418)
            surface: rgb(28, 28, 34),             // Card & panel surface (#1C1C22)
            surface_hover: rgb(38, 38, 46),       // Subtle interactive hover
            surface_selected: rgb(48, 50, 62),    // Active cell highlight
            border: rgb(44, 44, 54),              // Clean hairline divider (#2C2C36)
            primary: rgb(99, 102, 241),           // Restrained indigo (#6366F1)
            primary_light: rgb(165, 180, 252),    // Legible primary text accent
            text_primary: rgb(244, 244, 246),     // High contrast white-slate
            text_secondary: rgb(156, 161, 174),   // Quiet subtitle text
            text_muted: rgb(100, 104, 118),       // Dim auxiliary text
            holiday_red: rgb(239, 68, 68),        // Carmine red (#EF4444)
            holiday_bg: rgb(46, 26, 28),          // Very soft red tint
            today_ring: rgb(129, 140, 248),       // Soft indigo today ring
            event_accent: rgb(224, 160, 48),      // Restrained amber
            header_bg: rgb(24, 24, 30),           // Anchored header surface
        }
    }

    /// Refined light theme: Crisp warm-tinted paper ramp, clear reading contrast
    pub const fn light() -> Self {
        Self {
            is_dark: false,
            background: rgb(246, 246, 248),       // Light tinted background (#F6F6F8)
            surface: rgb(255, 255, 255),          // Pure white card
            surface_hover: rgb(240, 241, 245),    // Subtle hover
            surface_selected: rgb(228, 232, 248), // Gentle accent tint
            border: rgb(224, 226, 234),           // Subtle hairline border
            primary: rgb(79, 70, 229),            // Royal indigo
            primary_light: rgb(99, 102, 241),     // Bright indigo
            text_primary: rgb(24, 24, 32),        // Deep charcoal text
            text_secondary: rgb(90, 94, 108),     // Balanced secondary
            text_muted: rgb(148, 152, 166),       // Light muted text
            holiday_red: rgb(220, 38, 38),        // Official holiday carmine (#DC2626)
            holiday_bg: rgb(254, 242, 242),       // Light red tint
            today_ring: rgb(79, 70, 229),         // Indigo today ring
            event_accent: rgb(180, 83, 9),        // Amber event note
            header_bg: rgb(240, 241, 246),        // Soft header
        }
    }
}

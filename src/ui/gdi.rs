//! Lightweight RAII GDI helpers, cached font sets, and crisp vector icons.

use std::f32::consts::FRAC_1_SQRT_2;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, Ellipse, FillRect,
    FrameRect, GetStockObject, LineTo, MoveToEx, RoundRect, SelectObject, SetTextColor,
    ANTIALIASED_QUALITY, DRAW_TEXT_FORMAT, FW_MEDIUM, FW_NORMAL, FW_SEMIBOLD, HBRUSH, HDC, HFONT,
    HPEN, NULL_BRUSH, NULL_PEN, PS_SOLID,
};

use crate::util::dpi::scale_for_dpi;

pub struct OwnedBrush(pub HBRUSH);
impl OwnedBrush {
    pub unsafe fn solid(color: COLORREF) -> Self {
        Self(CreateSolidBrush(color))
    }
}
impl Drop for OwnedBrush {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_invalid() {
                let _ = DeleteObject(self.0);
            }
        }
    }
}

pub struct OwnedPen(pub HPEN);
impl OwnedPen {
    pub unsafe fn solid(width: i32, color: COLORREF) -> Self {
        Self(CreatePen(PS_SOLID, width, color))
    }
}
impl Drop for OwnedPen {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_invalid() {
                let _ = DeleteObject(self.0);
            }
        }
    }
}

pub struct OwnedFont(pub HFONT);
impl OwnedFont {
    pub unsafe fn create(height: i32, weight: i32, charset: u32, face: PCWSTR) -> Self {
        Self(CreateFontW(
            -height, // negative height = exact character em-height
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            charset,
            0,
            0,
            ANTIALIASED_QUALITY.0 as u32,
            0,
            face,
        ))
    }
}
impl Drop for OwnedFont {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_invalid() {
                let _ = DeleteObject(self.0);
            }
        }
    }
}

/// DPI-keyed cached font set so we don't recreate GDI fonts on every mouse hover.
pub struct PopupFonts {
    pub dpi: u32,
    pub title: OwnedFont,
    pub button: OwnedFont,
    pub weekday: OwnedFont,
    pub day: OwnedFont,
    pub day_sub: OwnedFont,
    pub detail_title: OwnedFont,
    pub detail_body: OwnedFont,
    pub detail_event: OwnedFont,
}

unsafe impl Send for PopupFonts {}
unsafe impl Sync for PopupFonts {}

impl PopupFonts {
    pub unsafe fn new(dpi: u32) -> Self {
        let face = windows::core::w!("Segoe UI");
        Self {
            dpi,
            title: OwnedFont::create(scale_for_dpi(16, dpi), FW_SEMIBOLD.0 as i32, 178, face),
            button: OwnedFont::create(scale_for_dpi(12, dpi), FW_MEDIUM.0 as i32, 178, face),
            weekday: OwnedFont::create(scale_for_dpi(12, dpi), FW_SEMIBOLD.0 as i32, 178, face),
            day: OwnedFont::create(scale_for_dpi(15, dpi), FW_SEMIBOLD.0 as i32, 178, face),
            day_sub: OwnedFont::create(scale_for_dpi(9, dpi), FW_NORMAL.0 as i32, 178, face),
            detail_title: OwnedFont::create(scale_for_dpi(14, dpi), FW_SEMIBOLD.0 as i32, 178, face),
            detail_body: OwnedFont::create(scale_for_dpi(12, dpi), FW_NORMAL.0 as i32, 178, face),
            detail_event: OwnedFont::create(scale_for_dpi(12, dpi), FW_MEDIUM.0 as i32, 178, face),
        }
    }
}

pub unsafe fn fill_rect(hdc: HDC, rect: &RECT, color: COLORREF) {
    let b = OwnedBrush::solid(color);
    FillRect(hdc, rect, b.0);
}

pub unsafe fn frame_rect(hdc: HDC, rect: &RECT, color: COLORREF) {
    let b = OwnedBrush::solid(color);
    FrameRect(hdc, rect, b.0);
}

pub unsafe fn fill_round_rect(
    hdc: HDC,
    rect: &RECT,
    radius: i32,
    fill: COLORREF,
    border: Option<(i32, COLORREF)>,
) {
    let b = OwnedBrush::solid(fill);
    let old_b = SelectObject(hdc, b.0);
    let pen_guard = border.map(|(w, c)| OwnedPen::solid(w, c));
    let old_p = match &pen_guard {
        Some(p) => SelectObject(hdc, p.0),
        None => SelectObject(hdc, GetStockObject(NULL_PEN)),
    };
    let d = radius * 2;
    let _ = RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, d, d);
    SelectObject(hdc, old_p);
    SelectObject(hdc, old_b);
}

pub unsafe fn stroke_round_rect(hdc: HDC, rect: &RECT, radius: i32, width: i32, color: COLORREF) {
    let p = OwnedPen::solid(width, color);
    let old_p = SelectObject(hdc, p.0);
    let old_b = SelectObject(hdc, GetStockObject(NULL_BRUSH));
    let d = radius * 2;
    let _ = RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, d, d);
    SelectObject(hdc, old_b);
    SelectObject(hdc, old_p);
}

pub unsafe fn draw_text(
    hdc: HDC,
    text: &str,
    rect: &RECT,
    color: COLORREF,
    format: DRAW_TEXT_FORMAT,
) {
    SetTextColor(hdc, color);
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    let mut r = *rect;
    DrawTextW(hdc, &mut wide, &mut r, format);
}

/// Crisp geometric chevron (`left = true` points left `<`, `false` points right `>`).
pub unsafe fn draw_chevron(hdc: HDC, rect: &RECT, dpi: u32, left: bool, double: bool, color: COLORREF) {
    let stroke = scale_for_dpi(1, dpi).max(1);
    let pen = OwnedPen::solid(stroke, color);
    let old_p = SelectObject(hdc, pen.0);

    let cx = (rect.left + rect.right) / 2;
    let cy = (rect.top + rect.bottom) / 2;
    let half_h = scale_for_dpi(4, dpi);
    let w = scale_for_dpi(3, dpi);
    let dir = if left { -1 } else { 1 };

    let offsets: &[i32] = if double {
        &[-scale_for_dpi(2, dpi), scale_for_dpi(2, dpi)]
    } else {
        &[0]
    };

    for &dx in offsets {
        let tip_x = cx + dx + dir * (w / 2);
        let tail_x = cx + dx - dir * (w - w / 2);
        let _ = MoveToEx(hdc, tail_x, cy - half_h, None);
        let _ = LineTo(hdc, tip_x, cy);
        let _ = LineTo(hdc, tail_x, cy + half_h);
    }

    SelectObject(hdc, old_p);
}

/// Crisp 8-ray sun icon for toggling dark -> light theme.
pub unsafe fn draw_sun_icon(hdc: HDC, rect: &RECT, dpi: u32, color: COLORREF) {
    let cx = (rect.left + rect.right) / 2;
    let cy = (rect.top + rect.bottom) / 2;
    let pen = OwnedPen::solid(1, color);
    let brush = OwnedBrush::solid(color);
    let old_p = SelectObject(hdc, pen.0);
    let old_b = SelectObject(hdc, brush.0);

    let r = scale_for_dpi(3, dpi);
    let _ = Ellipse(hdc, cx - r, cy - r, cx + r + 1, cy + r + 1);

    let ray_in = scale_for_dpi(5, dpi);
    let ray_out = scale_for_dpi(7, dpi);
    let d_in = (ray_in as f32 * FRAC_1_SQRT_2).round() as i32;
    let d_out = (ray_out as f32 * FRAC_1_SQRT_2).round() as i32;

    let rays = [
        (0, -ray_in, 0, -ray_out),
        (0, ray_in, 0, ray_out),
        (-ray_in, 0, -ray_out, 0),
        (ray_in, 0, ray_out, 0),
        (-d_in, -d_in, -d_out, -d_out),
        (d_in, -d_in, d_out, -d_out),
        (-d_in, d_in, -d_out, d_out),
        (d_in, d_in, d_out, d_out),
    ];

    for (x1, y1, x2, y2) in rays {
        let _ = MoveToEx(hdc, cx + x1, cy + y1, None);
        let _ = LineTo(hdc, cx + x2, cy + y2);
    }

    SelectObject(hdc, old_b);
    SelectObject(hdc, old_p);
}

/// Crisp crescent moon icon for toggling light -> dark theme.
pub unsafe fn draw_moon_icon(hdc: HDC, rect: &RECT, dpi: u32, color: COLORREF, bg: COLORREF) {
    let cx = (rect.left + rect.right) / 2;
    let cy = (rect.top + rect.bottom) / 2;
    let mr = scale_for_dpi(6, dpi);
    let pen = OwnedPen::solid(1, color);
    let brush = OwnedBrush::solid(color);
    let bg_pen = OwnedPen::solid(1, bg);
    let bg_brush = OwnedBrush::solid(bg);

    let old_p = SelectObject(hdc, pen.0);
    let old_b = SelectObject(hdc, brush.0);
    let _ = Ellipse(hdc, cx - mr, cy - mr, cx + mr + 1, cy + mr + 1);

    SelectObject(hdc, bg_pen.0);
    SelectObject(hdc, bg_brush.0);
    let cut_offset = scale_for_dpi(3, dpi);
    let cut_r = scale_for_dpi(5, dpi);
    let _ = Ellipse(
        hdc,
        cx - cut_r + cut_offset,
        cy - cut_r - cut_offset,
        cx + cut_r + cut_offset + 1,
        cy + cut_r - cut_offset + 1,
    );

    SelectObject(hdc, old_b);
    SelectObject(hdc, old_p);
}

/// Vector icon for switching between Calendar grid and Date Info/Converter view.
pub unsafe fn draw_view_toggle_icon(
    hdc: HDC,
    rect: &RECT,
    dpi: u32,
    is_converter: bool,
    color: COLORREF,
) {
    let cx = (rect.left + rect.right) / 2;
    let cy = (rect.top + rect.bottom) / 2;
    let pen = OwnedPen::solid(1, color);
    let old_p = SelectObject(hdc, pen.0);
    let old_b = SelectObject(hdc, GetStockObject(NULL_BRUSH));

    if is_converter {
        // Draw small 3x3 calendar grid icon to return to calendar view
        let s = scale_for_dpi(5, dpi);
        let box_r = RECT {
            left: cx - s,
            top: cy - s,
            right: cx + s + 1,
            bottom: cy + s + 1,
        };
        let _ = RoundRect(hdc, box_r.left, box_r.top, box_r.right, box_r.bottom, 2, 2);
        let bar_y = cy - s + scale_for_dpi(3, dpi);
        let _ = MoveToEx(hdc, box_r.left + 1, bar_y, None);
        let _ = LineTo(hdc, box_r.right - 1, bar_y);
    } else {
        // Draw clean two-way horizontal exchange arrows (convert / triple dates)
        let w = scale_for_dpi(5, dpi);
        let dy = scale_for_dpi(2, dpi);
        let tip = scale_for_dpi(2, dpi);
        // Top arrow pointing right
        let _ = MoveToEx(hdc, cx - w, cy - dy, None);
        let _ = LineTo(hdc, cx + w, cy - dy);
        let _ = MoveToEx(hdc, cx + w - tip, cy - dy - tip, None);
        let _ = LineTo(hdc, cx + w, cy - dy);
        let _ = LineTo(hdc, cx + w - tip, cy - dy + tip);
        // Bottom arrow pointing left
        let _ = MoveToEx(hdc, cx + w, cy + dy, None);
        let _ = LineTo(hdc, cx - w, cy + dy);
        let _ = MoveToEx(hdc, cx - w + tip, cy + dy - tip, None);
        let _ = LineTo(hdc, cx - w, cy + dy);
        let _ = LineTo(hdc, cx - w + tip, cy + dy + tip);
    }

    SelectObject(hdc, old_b);
    SelectObject(hdc, old_p);
}

/// Crisp dual-square clipboard copy vector icon.
pub unsafe fn draw_copy_icon(hdc: HDC, rect: &RECT, dpi: u32, color: COLORREF) {
    let cx = (rect.left + rect.right) / 2;
    let cy = (rect.top + rect.bottom) / 2;
    let pen = OwnedPen::solid(1, color);
    let old_p = SelectObject(hdc, pen.0);
    let old_b = SelectObject(hdc, GetStockObject(NULL_BRUSH));

    let s = scale_for_dpi(4, dpi);
    let off = scale_for_dpi(2, dpi);
    // Front square
    let _ = RoundRect(hdc, cx - s, cy - s + off, cx + s - off + 1, cy + s + 1, 2, 2);
    // Back L-outline
    let _ = MoveToEx(hdc, cx - s + off, cy - s + off, None);
    let _ = LineTo(hdc, cx - s + off, cy - s);
    let _ = LineTo(hdc, cx + s, cy - s);
    let _ = LineTo(hdc, cx + s, cy + s - off);
    let _ = LineTo(hdc, cx + s - off, cy + s - off);

    SelectObject(hdc, old_b);
    SelectObject(hdc, old_p);
}

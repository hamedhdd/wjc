//! Double-buffered GDI rendering for the calendar popup window.

use windows::Win32::Foundation::{COLORREF, HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetStockObject,
    LineTo, MoveToEx, RoundRect, SelectObject, SetBkMode, DT_CENTER, DT_LEFT, DT_NOCLIP, DT_RIGHT,
    DT_SINGLELINE, DT_TOP, DT_VCENTER, DT_WORDBREAK, HDC, NULL_PEN, SRCCOPY, TRANSPARENT,
};
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

use super::layout::PopupLayout;
use super::state::{ButtonId, CalendarWindowState, ViewMode};
use crate::calendar::{
    get_events_for_date, is_official_holiday, to_persian_digits, LunarDate, JALALI_MONTH_NAMES,
};
use crate::ui::gdi::{
    draw_chevron, draw_copy_icon, draw_moon_icon, draw_sun_icon, draw_text, draw_view_toggle_icon,
    fill_rect, fill_round_rect, frame_rect, stroke_round_rect, OwnedBrush, OwnedPen, PopupFonts,
};
use crate::util::dpi::scale_for_dpi;

pub unsafe fn paint_calendar(hwnd: HWND, hdc: HDC, state: &mut CalendarWindowState) {
    let mut client = RECT::default();
    let _ = GetClientRect(hwnd, &mut client);
    let width = client.right - client.left;
    let height = client.bottom - client.top;
    if width <= 0 || height <= 0 {
        return;
    }

    state.ensure_grid_cache();
    let dpi = state.dpi;
    if state.fonts.as_ref().map(|f| f.dpi) != Some(dpi) {
        state.fonts = Some(PopupFonts::new(dpi));
    }

    let layout = PopupLayout::compute(width, height, dpi, state.grid_rows());

    // Double-buffer memory DC
    let hdc_mem = CreateCompatibleDC(hdc);
    let hbm_mem = CreateCompatibleBitmap(hdc, width, height);
    let old_bmp = SelectObject(hdc_mem, hbm_mem);

    let t = state.theme;
    fill_rect(hdc_mem, &client, t.background);
    frame_rect(hdc_mem, &client, t.border);
    SetBkMode(hdc_mem, TRANSPARENT);

    let fonts = state.fonts.as_ref().unwrap();

    paint_header(hdc_mem, &layout, state, fonts);

    match state.view_mode {
        ViewMode::Calendar => {
            paint_weekdays(hdc_mem, &layout, state, fonts);
            paint_grid(hdc_mem, &layout, state, fonts);
            paint_day_details(hdc_mem, &layout, state, fonts);
        }
        ViewMode::Converter => {
            paint_converter_view(hdc_mem, &layout, state, fonts);
        }
    }

    let _ = BitBlt(hdc, 0, 0, width, height, hdc_mem, 0, 0, SRCCOPY);

    SelectObject(hdc_mem, old_bmp);
    let _ = DeleteObject(hbm_mem);
    let _ = DeleteDC(hdc_mem);
}

unsafe fn paint_header(
    hdc: HDC,
    layout: &PopupLayout,
    state: &CalendarWindowState,
    fonts: &PopupFonts,
) {
    let t = state.theme;
    let dpi = state.dpi;
    let r_btn = scale_for_dpi(6, dpi);

    fill_rect(hdc, &layout.header_rect, t.header_bg);

    // Subtle hairline bottom divider
    let pen = OwnedPen::solid(1, t.border);
    let old_p = SelectObject(hdc, pen.0);
    let _ = MoveToEx(hdc, 1, layout.header_rect.bottom, None);
    let _ = LineTo(hdc, layout.width - 1, layout.header_rect.bottom);
    SelectObject(hdc, old_p);

    let draw_chrome_btn = |rect: &RECT, id: ButtonId, highlight: bool| -> (COLORREF, COLORREF) {
        let hovered = state.hovered_button == Some(id);
        let bg = if highlight {
            if hovered { t.primary_light } else { t.primary }
        } else if hovered {
            t.surface_hover
        } else {
            t.surface
        };
        let border = if highlight { None } else { Some((1, t.border)) };
        fill_round_rect(hdc, rect, r_btn, bg, border);
        let fg = if highlight {
            COLORREF(0x00FFFFFF)
        } else if hovered {
            t.text_primary
        } else {
            t.text_secondary
        };
        (bg, fg)
    };

    // RTL Navigation arrows: right side = previous, left side = next
    let (_, fg) = draw_chrome_btn(&layout.prev_month, ButtonId::PrevMonth, false);
    draw_chevron(hdc, &layout.prev_month, dpi, false, false, fg);

    let (_, fg) = draw_chrome_btn(&layout.prev_year, ButtonId::PrevYear, false);
    draw_chevron(hdc, &layout.prev_year, dpi, false, true, fg);

    let (_, fg) = draw_chrome_btn(&layout.next_month, ButtonId::NextMonth, false);
    draw_chevron(hdc, &layout.next_month, dpi, true, false, fg);

    let (_, fg) = draw_chrome_btn(&layout.next_year, ButtonId::NextYear, false);
    draw_chevron(hdc, &layout.next_year, dpi, true, true, fg);

    // Today button
    let (_, fg) = draw_chrome_btn(&layout.today, ButtonId::Today, true);
    let old_f = SelectObject(hdc, fonts.button.0);
    draw_text(
        hdc,
        "امروز",
        &layout.today,
        fg,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
    );

    // View toggle button (Calendar <-> Triple Date Info / Converter)
    let is_conv = state.view_mode == ViewMode::Converter;
    let (_, fg) = draw_chrome_btn(&layout.view_toggle, ButtonId::ViewToggle, is_conv);
    draw_view_toggle_icon(hdc, &layout.view_toggle, dpi, is_conv, fg);

    // Theme toggle button (Vector Sun / Moon)
    let (bg, fg) = draw_chrome_btn(&layout.theme_toggle, ButtonId::ThemeToggle, false);
    if state.theme.is_dark {
        draw_sun_icon(hdc, &layout.theme_toggle, dpi, fg);
    } else {
        draw_moon_icon(hdc, &layout.theme_toggle, dpi, fg, bg);
    }

    // Center title (Month & Year)
    let month_name = if (1..=12).contains(&state.viewing_month) {
        JALALI_MONTH_NAMES[(state.viewing_month - 1) as usize]
    } else {
        ""
    };
    let title_text = format!(
        "{month_name} {}",
        to_persian_digits(state.viewing_year.to_string())
    );
    SelectObject(hdc, fonts.title.0);
    draw_text(
        hdc,
        &title_text,
        &layout.title,
        t.text_primary,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
    );

    SelectObject(hdc, old_f);
}

unsafe fn paint_weekdays(
    hdc: HDC,
    layout: &PopupLayout,
    state: &CalendarWindowState,
    fonts: &PopupFonts,
) {
    let t = state.theme;
    let weekdays = ["ش", "ی", "د", "س", "چ", "پ", "ج"];
    let old_f = SelectObject(hdc, fonts.weekday.0);

    for (i, name) in weekdays.iter().enumerate() {
        let r = layout.weekday_rect(i);
        let color = if i == 6 { t.holiday_red } else { t.text_secondary };
        draw_text(
            hdc,
            name,
            &r,
            color,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
        );
    }
    SelectObject(hdc, old_f);
}

unsafe fn paint_grid(
    hdc: HDC,
    layout: &PopupLayout,
    state: &CalendarWindowState,
    fonts: &PopupFonts,
) {
    let t = state.theme;
    let dpi = state.dpi;
    let cell_radius = scale_for_dpi(6, dpi);
    let old_f = SelectObject(hdc, fonts.day.0);

    for (idx, cell) in state.cells().iter().enumerate() {
        let rect = layout.cell_rect(idx, dpi);
        let is_selected = cell.is_current_month && cell.jalali == state.selected_date;
        let is_hovered = state.hovered_day_index == Some(idx);

        let bg = if is_selected {
            t.surface_selected
        } else if is_hovered {
            t.surface_hover
        } else if cell.is_holiday && cell.is_current_month {
            t.holiday_bg
        } else {
            t.surface
        };

        fill_round_rect(hdc, &rect, cell_radius, bg, None);

        if cell.is_today && cell.is_current_month {
            stroke_round_rect(hdc, &rect, cell_radius, scale_for_dpi(2, dpi).max(1), t.today_ring);
        } else if is_selected {
            stroke_round_rect(hdc, &rect, cell_radius, 1, t.primary_light);
        }

        // Main Jalali day numeral
        SelectObject(hdc, fonts.day.0);
        let text_color = if !cell.is_current_month {
            t.text_muted
        } else if cell.is_holiday {
            t.holiday_red
        } else if is_selected {
            t.primary_light
        } else {
            t.text_primary
        };

        let day_str = to_persian_digits(cell.day.to_string());
        let text_rect = RECT {
            left: rect.left,
            top: rect.top + scale_for_dpi(7, dpi),
            right: rect.right,
            bottom: rect.bottom,
        };
        draw_text(
            hdc,
            &day_str,
            &text_rect,
            text_color,
            DT_CENTER | DT_TOP | DT_SINGLELINE | DT_NOCLIP,
        );

        if cell.is_current_month {
            SelectObject(hdc, fonts.day_sub.0);

            // Gregorian day (top-left)
            let greg_str = cell.gregorian.day.to_string();
            let greg_rect = RECT {
                left: rect.left + scale_for_dpi(4, dpi),
                top: rect.top + scale_for_dpi(2, dpi),
                right: rect.right,
                bottom: rect.bottom,
            };
            draw_text(
                hdc,
                &greg_str,
                &greg_rect,
                t.text_muted,
                DT_LEFT | DT_TOP | DT_SINGLELINE | DT_NOCLIP,
            );

            // Lunar Hijri day (top-right)
            let lunar_str = to_persian_digits(cell.lunar.day.to_string());
            let lunar_rect = RECT {
                left: rect.left,
                top: rect.top + scale_for_dpi(2, dpi),
                right: rect.right - scale_for_dpi(4, dpi),
                bottom: rect.bottom,
            };
            draw_text(
                hdc,
                &lunar_str,
                &lunar_rect,
                t.text_muted,
                DT_RIGHT | DT_TOP | DT_SINGLELINE | DT_NOCLIP,
            );

            // Holiday indicator: centered 4px circular pip
            if cell.is_holiday {
                let dot_brush = OwnedBrush::solid(t.holiday_red);
                let cx = (rect.left + rect.right) / 2;
                let cy = rect.bottom - scale_for_dpi(5, dpi);
                let r = scale_for_dpi(2, dpi).max(2);
                let old_b = SelectObject(hdc, dot_brush.0);
                let old_p = SelectObject(hdc, GetStockObject(NULL_PEN));
                let _ = RoundRect(hdc, cx - r, cy - r, cx + r, cy + r, r, r);
                SelectObject(hdc, old_p);
                SelectObject(hdc, old_b);
            }
        }
    }

    SelectObject(hdc, old_f);
}

unsafe fn paint_day_details(
    hdc: HDC,
    layout: &PopupLayout,
    state: &CalendarWindowState,
    fonts: &PopupFonts,
) {
    let t = state.theme;
    let dpi = state.dpi;
    let card = layout.details_card;
    let card_radius = scale_for_dpi(8, dpi);

    fill_round_rect(hdc, &card, card_radius, t.surface, Some((1, t.border)));

    let sel = state.selected_date;
    let g = sel.to_gregorian();
    let l = LunarDate::from_gregorian(g);
    let events = get_events_for_date(&sel, &l);

    let pad = scale_for_dpi(12, dpi);
    let mut line_y = card.top + pad;

    let is_friday = sel.weekday().is_weekend();
    let is_holiday = is_official_holiday(&sel, &l);

    // Subtle copy button on the top-left of details card
    let copy_hovered = state.hovered_button == Some(ButtonId::CopyDate);
    let copy_bg = if copy_hovered { t.surface_hover } else { t.surface };
    fill_round_rect(
        hdc,
        &layout.copy_btn,
        scale_for_dpi(4, dpi),
        copy_bg,
        if copy_hovered { Some((1, t.border)) } else { None },
    );
    let copy_fg = if copy_hovered { t.text_primary } else { t.text_muted };
    draw_copy_icon(hdc, &layout.copy_btn, dpi, copy_fg);

    // Show brief "کپی شد" badge next to copy button if recently copied
    let old_f = SelectObject(hdc, fonts.day_sub.0);
    if GetTickCount64() < state.copied_until_ms {
        let feedback_rect = RECT {
            left: layout.copy_btn.right + scale_for_dpi(4, dpi),
            top: layout.copy_btn.top,
            right: layout.copy_btn.right + scale_for_dpi(60, dpi),
            bottom: layout.copy_btn.bottom,
        };
        draw_text(
            hdc,
            "کپی شد",
            &feedback_rect,
            t.primary_light,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
        );
    }

    // Line 1: Jalali full date + weekday
    SelectObject(hdc, fonts.detail_title.0);
    let line1 = format!("{} {}", sel.weekday().name_fa(), sel.format_long_fa());
    let r1 = RECT {
        left: layout.copy_btn.right + scale_for_dpi(8, dpi),
        top: line_y,
        right: card.right - pad,
        bottom: line_y + scale_for_dpi(22, dpi),
    };
    draw_text(
        hdc,
        &line1,
        &r1,
        if is_holiday { t.holiday_red } else { t.primary_light },
        DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
    );
    line_y += scale_for_dpi(26, dpi);

    // Line 2: Gregorian + Hijri equivalents (with subtle "(تقریبی)" tag beyond official table)
    SelectObject(hdc, fonts.detail_body.0);
    let lunar_text = if l.is_estimated() {
        format!("{} (تقریبی)", l.format_long_fa())
    } else {
        l.format_long_fa()
    };
    let line2 = format!("میلادی: {}  |  قمری: {lunar_text}", g.format_long());
    let r2 = RECT {
        left: card.left + pad,
        top: line_y,
        right: card.right - pad,
        bottom: line_y + scale_for_dpi(20, dpi),
    };
    draw_text(
        hdc,
        &line2,
        &r2,
        t.text_secondary,
        DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
    );
    line_y += scale_for_dpi(24, dpi);

    // Events & Holidays (word-wrapped cleanly, supports up to 3 lines)
    SelectObject(hdc, fonts.detail_event.0);
    let remaining_bottom = card.bottom - scale_for_dpi(8, dpi);

    if events.is_empty() {
        let (msg, color) = if is_friday {
            ("تعطیل پایان هفته (جمعه)", t.holiday_red)
        } else {
            ("رویداد خاصی ثبت نشده است", t.text_muted)
        };
        let ev_rect = RECT {
            left: card.left + pad,
            top: line_y,
            right: card.right - pad,
            bottom: remaining_bottom,
        };
        draw_text(hdc, msg, &ev_rect, color, DT_RIGHT | DT_TOP | DT_WORDBREAK);
    } else {
        let line_step = scale_for_dpi(20, dpi);
        for ev in events.iter().take(3) {
            if line_y + scale_for_dpi(14, dpi) > remaining_bottom {
                break;
            }
            let prefix = if ev.is_holiday {
                "تعطیل رسمی: "
            } else {
                "مناسبت: "
            };
            let color = if ev.is_holiday {
                t.holiday_red
            } else {
                t.event_accent
            };
            let ev_str = format!("{prefix}{}", ev.title);
            let ev_rect = RECT {
                left: card.left + pad,
                top: line_y,
                right: card.right - pad,
                bottom: remaining_bottom,
            };
            draw_text(
                hdc,
                &ev_str,
                &ev_rect,
                color,
                DT_RIGHT | DT_TOP | DT_WORDBREAK,
            );
            line_y += line_step;
        }
    }

    SelectObject(hdc, old_f);
}

unsafe fn paint_converter_view(
    hdc: HDC,
    layout: &PopupLayout,
    state: &CalendarWindowState,
    fonts: &PopupFonts,
) {
    let t = state.theme;
    let dpi = state.dpi;
    let old_f = SelectObject(hdc, fonts.detail_title.0);

    draw_text(
        hdc,
        "جزئیات و تبدیل تاریخ سه‌گانه",
        &layout.converter_title,
        t.primary_light,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
    );

    let rows = state.converter_rows();
    let radius = scale_for_dpi(6, dpi);
    let pad = scale_for_dpi(12, dpi);

    SelectObject(hdc, fonts.detail_body.0);
    for (i, ((label, _), rect)) in rows.iter().zip(layout.converter_rows.iter()).enumerate() {
        let hovered = state.hovered_converter_row == Some(i);
        let bg = if hovered { t.surface_hover } else { t.surface };
        fill_round_rect(hdc, rect, radius, bg, Some((1, t.border)));

        let text_rect = RECT {
            left: rect.left + pad,
            top: rect.top,
            right: rect.right - pad,
            bottom: rect.bottom,
        };
        draw_text(
            hdc,
            label,
            &text_rect,
            t.text_primary,
            DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
        );

        if hovered {
            let hint_rect = RECT {
                left: rect.left + pad,
                top: rect.top,
                right: rect.left + scale_for_dpi(60, dpi),
                bottom: rect.bottom,
            };
            draw_text(
                hdc,
                "کپی",
                &hint_rect,
                t.text_muted,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOCLIP,
            );
        }
    }

    // Footer hint
    let hint_rect = RECT {
        left: layout.pad,
        top: layout.converter_rows[5].bottom + scale_for_dpi(12, dpi),
        right: layout.width - layout.pad,
        bottom: layout.height - layout.pad,
    };
    let hint = if GetTickCount64() < state.copied_until_ms {
        "تاریخ در حافظه کپی شد"
    } else {
        "برای کپی روی هر ردیف کلیک کنید یا Ctrl+C بزنید"
    };
    draw_text(
        hdc,
        hint,
        &hint_rect,
        if GetTickCount64() < state.copied_until_ms {
            t.primary_light
        } else {
            t.text_muted
        },
        DT_CENTER | DT_TOP | DT_SINGLELINE | DT_NOCLIP,
    );

    SelectObject(hdc, old_f);
}

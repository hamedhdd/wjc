//! Unified geometry & hit-testing layout shared between painter and input handlers.
//! Keeps every spacing value aligned to a 4px grid and guarantees 0px drift
//! between drawn rects and hit-tested targets.

use windows::Win32::Foundation::RECT;

use super::state::{ButtonId, ViewMode};
use crate::util::dpi::scale_for_dpi;

pub const BASE_WIDTH: i32 = 420;
pub const BASE_HEIGHT: i32 = 576;

pub struct PopupLayout {
    pub width: i32,
    pub height: i32,
    pub pad: i32,
    pub header_rect: RECT,
    pub prev_month: RECT,
    pub prev_year: RECT,
    pub theme_toggle: RECT,
    pub next_month: RECT,
    pub next_year: RECT,
    pub today: RECT,
    pub view_toggle: RECT,
    pub title: RECT,
    pub weekday_top: i32,
    pub weekday_h: i32,
    pub grid_top: i32,
    pub grid_bottom: i32,
    pub col_w: i32,
    pub row_h: i32,
    pub details_card: RECT,
    pub copy_btn: RECT,
    pub converter_title: RECT,
    pub converter_rows: [RECT; 6],
}

impl PopupLayout {
    pub fn compute(width: i32, height: i32, dpi: u32, total_rows: usize) -> Self {
        let s = |v: i32| scale_for_dpi(v, dpi);
        let pad = s(12);
        let header_h = s(56);
        let btn_s = s(28);
        let btn_y = (header_h - btn_s) / 2;
        let gap = s(4);

        // Right cluster (RTL: Previous Month, Previous Year, Theme Toggle)
        let prev_month = RECT {
            left: width - pad - btn_s,
            top: btn_y,
            right: width - pad,
            bottom: btn_y + btn_s,
        };
        let prev_year = RECT {
            left: prev_month.left - gap - btn_s,
            top: btn_y,
            right: prev_month.left - gap,
            bottom: btn_y + btn_s,
        };
        let theme_toggle = RECT {
            left: prev_year.left - gap - btn_s,
            top: btn_y,
            right: prev_year.left - gap,
            bottom: btn_y + btn_s,
        };

        // Left cluster (Next Month, Next Year, Today, View Toggle)
        let next_month = RECT {
            left: pad,
            top: btn_y,
            right: pad + btn_s,
            bottom: btn_y + btn_s,
        };
        let next_year = RECT {
            left: next_month.right + gap,
            top: btn_y,
            right: next_month.right + gap + btn_s,
            bottom: btn_y + btn_s,
        };
        let today_w = s(48);
        let today = RECT {
            left: next_year.right + gap,
            top: btn_y,
            right: next_year.right + gap + today_w,
            bottom: btn_y + btn_s,
        };
        let view_toggle = RECT {
            left: today.right + gap,
            top: btn_y,
            right: today.right + gap + btn_s,
            bottom: btn_y + btn_s,
        };

        let title = RECT {
            left: view_toggle.right + gap,
            top: 0,
            right: theme_toggle.left - gap,
            bottom: header_h,
        };

        let weekday_top = header_h + s(8);
        let weekday_h = s(24);
        let grid_top = weekday_top + weekday_h + s(4);
        let col_w = (width - 2 * pad) / 7;
        let row_h = s(40);
        let grid_bottom = grid_top + (total_rows as i32 * row_h);

        let details_card = RECT {
            left: pad,
            top: grid_bottom + s(12),
            right: width - pad,
            bottom: height - pad,
        };

        let copy_s = s(24);
        let copy_btn = RECT {
            left: details_card.left + s(10),
            top: details_card.top + s(10),
            right: details_card.left + s(10) + copy_s,
            bottom: details_card.top + s(10) + copy_s,
        };

        let conv_top = header_h + s(16);
        let converter_title = RECT {
            left: pad,
            top: conv_top,
            right: width - pad,
            bottom: conv_top + s(28),
        };

        let row_card_h = s(40);
        let row_gap = s(8);
        let mut converter_rows = [RECT::default(); 6];
        let mut cy = converter_title.bottom + s(12);
        for slot in &mut converter_rows {
            *slot = RECT {
                left: pad,
                top: cy,
                right: width - pad,
                bottom: cy + row_card_h,
            };
            cy += row_card_h + row_gap;
        }

        Self {
            width,
            height,
            pad,
            header_rect: RECT {
                left: 1,
                top: 1,
                right: width - 1,
                bottom: header_h,
            },
            prev_month,
            prev_year,
            theme_toggle,
            next_month,
            next_year,
            today,
            view_toggle,
            title,
            weekday_top,
            weekday_h,
            grid_top,
            grid_bottom,
            col_w,
            row_h,
            details_card,
            copy_btn,
            converter_title,
            converter_rows,
        }
    }

    pub fn cell_rect(&self, idx: usize, dpi: u32) -> RECT {
        let col = idx % 7;
        let row = idx / 7;
        let inset = scale_for_dpi(2, dpi);
        // Center the 7*col_w grid symmetrically within (width - 2*pad)
        let grid_w = self.col_w * 7;
        let right_edge = (self.width + grid_w) / 2;
        let x = right_edge - ((col + 1) as i32 * self.col_w);
        let y = self.grid_top + (row as i32 * self.row_h);
        RECT {
            left: x + inset,
            top: y + inset,
            right: x + self.col_w - inset,
            bottom: y + self.row_h - inset,
        }
    }

    pub fn weekday_rect(&self, col: usize) -> RECT {
        let grid_w = self.col_w * 7;
        let right_edge = (self.width + grid_w) / 2;
        let x = right_edge - ((col + 1) as i32 * self.col_w);
        RECT {
            left: x,
            top: self.weekday_top,
            right: x + self.col_w,
            bottom: self.weekday_top + self.weekday_h,
        }
    }

    pub fn hit_test_button(&self, x: i32, y: i32, mode: ViewMode) -> Option<ButtonId> {
        let candidates = [
            (self.prev_month, ButtonId::PrevMonth),
            (self.prev_year, ButtonId::PrevYear),
            (self.theme_toggle, ButtonId::ThemeToggle),
            (self.next_month, ButtonId::NextMonth),
            (self.next_year, ButtonId::NextYear),
            (self.today, ButtonId::Today),
            (self.view_toggle, ButtonId::ViewToggle),
        ];
        for (r, id) in candidates {
            if pt_in_rect(x, y, &r) {
                return Some(id);
            }
        }
        if mode == ViewMode::Calendar && pt_in_rect(x, y, &self.copy_btn) {
            return Some(ButtonId::CopyDate);
        }
        None
    }

    pub fn hit_test_grid(&self, x: i32, y: i32, mode: ViewMode, cell_count: usize) -> Option<usize> {
        if mode != ViewMode::Calendar {
            return None;
        }
        let grid_w = self.col_w * 7;
        let left_edge = (self.width - grid_w) / 2;
        let right_edge = left_edge + grid_w;

        if y < self.grid_top || y >= self.grid_bottom || x < left_edge || x >= right_edge {
            return None;
        }
        let row = ((y - self.grid_top) / self.row_h) as usize;
        let col_from_right = ((right_edge - 1 - x) / self.col_w) as usize;
        if col_from_right >= 7 {
            return None;
        }
        let idx = row * 7 + col_from_right;
        (idx < cell_count).then_some(idx)
    }

    pub fn hit_test_converter_row(&self, x: i32, y: i32, mode: ViewMode) -> Option<usize> {
        if mode != ViewMode::Converter {
            return None;
        }
        self.converter_rows
            .iter()
            .position(|r| pt_in_rect(x, y, r))
    }
}

pub fn pt_in_rect(x: i32, y: i32, r: &RECT) -> bool {
    x >= r.left && x < r.right && y >= r.top && y < r.bottom
}

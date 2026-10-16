use crate::theme::{BAR_HEIGHT, FG, FG_DIM};

use chrono::{Local, Timelike};
use gtk4::DrawingArea;
use gtk4::cairo::Context;
use gtk4::prelude::*;

fn draw_bit(cr: &Context, x: f64, y: f64, on: bool) {
    let color = if on { FG } else { FG_DIM };
    cr.set_source_rgba(color.0, color.1, color.2, color.3);
    cr.rectangle(x, y, 7.0, 7.0);
    let _ = cr.fill();
}

fn draw_number(cr: &Context, value: u32, bits: usize, start_x: f64, y: f64) -> f64 {
    let mut x = start_x;
    for bit in (0..bits).rev() {
        let on = value & (1u32 << bit) != 0;
        draw_bit(cr, x, y, on);
        x += 10.0;
    }
    x
}

fn draw_separator(cr: &Context, x: f64, y: f64) {
    cr.set_source_rgba(FG_DIM.0, FG_DIM.1, FG_DIM.2, 0.6);
    cr.rectangle(x, y + 2.0, 2.0, 2.0);
    let _ = cr.fill();
}

fn draw(cr: &Context) {
    let now = Local::now();
    let y = 10.0;
    let mut x = 4.0;
    // HH
    x = draw_number(cr, now.hour(), 6, x, y);
    x += 5.0;
    draw_separator(cr, x, y);
    x += 7.0;
    // MM
    x = draw_number(cr, now.minute(), 6, x, y);
    x += 5.0;
    draw_separator(cr, x, y);
    x += 7.0;
    // SS
    draw_number(cr, now.second(), 6, x, y);
}

pub fn widget() -> DrawingArea {
    let area = DrawingArea::new();
    area.set_content_width(198);
    area.set_content_height(BAR_HEIGHT);
    area.set_draw_func(|_, cr, _, _| {
        draw(cr);
    });
    area
}

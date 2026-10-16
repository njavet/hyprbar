use chrono::Local;
use gtk4::cairo::Context;
fn draw_bit(cr: &Context, x: f64, y: f64, on: bool) {
    if on {
        cr.set_source_rgba(0.8, 1.0, 0.85, 1.0);
    } else {
        cr.set_source_rgba(0.8, 1.0, 0.85, 0.18);
    }

    cr.rectangle(x, y, 7.0, 7.0);
    cr.fill().unwrap();
}

fn draw_binary_number(cr: &Context, value: u32, bits: usize, start_x: f64, y: f64) -> f64 {
    let mut x = start_x;

    for bit in (0..bits).rev() {
        let on = value & (1 << bit) != 0;

        draw_bit(cr, x, y, on);

        x += 10.0;
    }
    x
}
fn draw_binary_clock(cr: &Context) {
    let now = Local::now();

    let mut x = 4.0;
    let y = 10.0;

    // HH: 5 bits
    x = draw_binary_number(cr, now.hour(), 5, x, y);

    x += 5.0;

    // separator
    cr.set_source_rgba(0.8, 1.0, 0.85, 0.5);
    cr.rectangle(x, y + 2.0, 2.0, 2.0);
    cr.fill().unwrap();

    x += 7.0;

    // MM: 6 bits
    x = draw_binary_number(cr, now.minute(), 6, x, y);

    x += 5.0;
    cr.rectangle(x, y + 2.0, 2.0, 2.0);
    cr.fill().unwrap();

    x += 7.0;

    // SS: 6 bits
    draw_binary_number(cr, now.second(), 6, x, y);
}

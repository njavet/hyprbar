use chrono::{Local, Offset, Timelike};
use gtk4::cairo::Context;
use gtk4::prelude::*;
use gtk4::{
    Application,
    ApplicationWindow,
    Box as GtkBox,
    DrawingArea,
    Label,
    Orientation,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::fs;

const APP_ID: &str = "dev.noe.rustbar";
const BAR_HEIGHT: i32 = 28;

fn battery() -> Option<u8> {
    fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .ok()?
        .trim()
        .parse()
        .ok()
}

fn network() -> String {
    let Ok(entries) = fs::read_dir("/sys/class/net") else {
        return "NET ?".into();
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if name == "lo" {
            continue;
        }

        let path = format!("/sys/class/net/{name}/operstate");

        if let Ok(state) = fs::read_to_string(path) {
            if state.trim() == "up" {
                return format!("NET {name}");
            }
        }
    }

    "NET DOWN".into()
}

fn nato_zone(offset_seconds: i32) -> char {
    match offset_seconds / 3600 {
        0 => 'Z',

        1 => 'A',
        2 => 'B',
        3 => 'C',
        4 => 'D',
        5 => 'E',
        6 => 'F',
        7 => 'G',
        8 => 'H',
        9 => 'I',
        10 => 'K',
        11 => 'L',
        12 => 'M',

        -1 => 'N',
        -2 => 'O',
        -3 => 'P',
        -4 => 'Q',
        -5 => 'R',
        -6 => 'S',
        -7 => 'T',
        -8 => 'U',
        -9 => 'V',
        -10 => 'W',
        -11 => 'X',
        -12 => 'Y',

        _ => '?',
    }
}

fn military_time() -> String {
    let now = Local::now();

    let offset = now.offset().fix().local_minus_utc();
    let zone = nato_zone(offset);

    format!("{}{zone}", now.format("%H%M"))
}

fn draw_bit(
    cr: &Context,
    x: f64,
    y: f64,
    on: bool,
) {
    if on {
        cr.set_source_rgba(0.8, 1.0, 0.85, 1.0);
    } else {
        cr.set_source_rgba(0.8, 1.0, 0.85, 0.18);
    }

    cr.rectangle(x, y, 7.0, 7.0);
    cr.fill().unwrap();
}

fn draw_binary_number(
    cr: &Context,
    value: u32,
    bits: usize,
    start_x: f64,
    y: f64,
) -> f64 {
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
    x = draw_binary_number(
        cr,
        now.hour(),
        5,
        x,
        y,
    );

    x += 5.0;

    // separator
    cr.set_source_rgba(0.8, 1.0, 0.85, 0.5);
    cr.rectangle(x, y + 2.0, 2.0, 2.0);
    cr.fill().unwrap();

    x += 7.0;

    // MM: 6 bits
    x = draw_binary_number(
        cr,
        now.minute(),
        6,
        x,
        y,
    );

    x += 5.0;

    cr.rectangle(x, y + 2.0, 2.0, 2.0);
    cr.fill().unwrap();

    x += 7.0;

    // SS: 6 bits
    draw_binary_number(
        cr,
        now.second(),
        6,
        x,
        y,
    );
}

fn draw_battery(
    cr: &Context,
    capacity: u8,
) {
    let x = 5.0;
    let y = 9.0;

    let width = 50.0;
    let height = 10.0;

    // Outline
    cr.set_source_rgba(0.8, 1.0, 0.85, 0.4);
    cr.rectangle(x, y, width, height);
    cr.stroke().unwrap();

    // Charge
    let charge_width =
        (width - 4.0) * capacity as f64 / 100.0;

    cr.set_source_rgba(0.8, 1.0, 0.85, 0.9);

    cr.rectangle(
        x + 2.0,
        y + 2.0,
        charge_width,
        height - 4.0,
    );

    cr.fill().unwrap();

    // Battery terminal
    cr.rectangle(
        x + width + 1.0,
        y + 3.0,
        3.0,
        height - 6.0,
    );

    cr.fill().unwrap();
}

fn build_ui(app: &Application) {
    let container = GtkBox::new(
        Orientation::Horizontal,
        12,
    );

    container.set_margin_start(10);
    container.set_margin_end(10);

    //
    // Network
    //

    let network_label = Label::new(None);
    network_label.set_text(&network());

    container.append(&network_label);

    //
    // Battery graphic
    //

    let battery_area = DrawingArea::new();

    battery_area.set_content_width(60);
    battery_area.set_content_height(BAR_HEIGHT);

    battery_area.set_draw_func(
        |_area, cr, _width, _height| {
            let capacity = battery().unwrap_or(0);

            draw_battery(cr, capacity);
        },
    );

    container.append(&battery_area);

    //
    // Spacer
    //

    let spacer = GtkBox::new(
        Orientation::Horizontal,
        0,
    );

    spacer.set_hexpand(true);

    container.append(&spacer);

    //
    // Binary clock
    //

    let clock_area = DrawingArea::new();

    clock_area.set_content_width(200);
    clock_area.set_content_height(BAR_HEIGHT);

    clock_area.set_draw_func(
        |_area, cr, _width, _height| {
            draw_binary_clock(cr);
        },
    );

    container.append(&clock_area);

    //
    // Normal military/NATO time
    //

    let time_label = Label::new(None);
    time_label.set_text(&military_time());

    container.append(&time_label);

    //
    // Window
    //

    let window = ApplicationWindow::builder()
        .application(app)
        .height_request(BAR_HEIGHT)
        .child(&container)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Top);

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    window.set_exclusive_zone(BAR_HEIGHT);

    //
    // CSS
    //

    let css = gtk4::CssProvider::new();

    css.load_from_data(
        r#"
        window {
            background-color: rgba(0, 0, 0, 0.64);
        }

        label {
            color: rgba(210, 255, 220, 0.95);
            font-family: monospace;
            font-size: 13px;
        }
        "#,
    );

    gtk4::style_context_add_provider_for_display(
        &WidgetExt::display(&window),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    //
    // Updates
    //

    let clock_clone = clock_area.clone();
    let battery_clone = battery_area.clone();
    let time_clone = time_label.clone();
    let network_clone = network_label.clone();

    glib::timeout_add_seconds_local(1, move || {
        // Causes Cairo draw function to run again.
        clock_clone.queue_draw();

        // Battery doesn't really need 1 Hz, but this is
        // insignificant for now.
        battery_clone.queue_draw();

        time_clone.set_text(&military_time());

        glib::ControlFlow::Continue
    });

    // Network can update much less frequently.
    glib::timeout_add_seconds_local(10, move || {
        network_clone.set_text(&network());

        glib::ControlFlow::Continue
    });

    window.present();
}

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    app.run();
}

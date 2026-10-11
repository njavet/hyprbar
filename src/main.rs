use chrono::{Local, Offset, Timelike};
use gtk4::cairo::Context;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::fs;
use gtk4::{
    Application,
    ApplicationWindow,
    Box as GtkBox,
    DrawingArea,
    Label,
    Orientation,
    Overlay,
};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

const APP_ID: &str = "dev.noe.rustbar";
const BAR_HEIGHT: i32 = 28;
const HISTORY: usize = 40;

// ------------------------------------------------------------
// Graph history
// ------------------------------------------------------------
struct History {
    values: VecDeque<f64>,
}
impl History {
    fn new() -> Self {
        Self {
            values: VecDeque::from(vec![0.0; HISTORY]),
        }
    }

    fn push(&mut self, value: f64) {
        if self.values.len() >= HISTORY {
            self.values.pop_front();
        }

        self.values.push_back(value.clamp(0.0, 1.0));
    }
}

// ------------------------------------------------------------
// CPU
// ------------------------------------------------------------

#[derive(Default)]
struct CpuSample {
    idle: u64,
    total: u64,
}

fn cpu_sample() -> Option<CpuSample> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    let line = stat.lines().next()?;

    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|x| x.parse().ok())
        .collect();

    if values.len() < 4 {
        return None;
    }

    let idle = values[3] + values.get(4).copied().unwrap_or(0);
    let total = values.iter().sum();

    Some(CpuSample { idle, total })
}

fn cpu_usage(
    previous: &mut Option<CpuSample>,
) -> f64 {
    let Some(current) = cpu_sample() else {
        return 0.0;
    };

    let usage = if let Some(old) = previous {
        let total_delta =
            current.total.saturating_sub(old.total);

        let idle_delta =
            current.idle.saturating_sub(old.idle);

        if total_delta == 0 {
            0.0
        } else {
            1.0 - idle_delta as f64 / total_delta as f64
        }
    } else {
        0.0
    };

    *previous = Some(current);

    usage
}

// ------------------------------------------------------------
// RAM
// ------------------------------------------------------------

fn ram_usage() -> f64 {
    let Ok(meminfo) = fs::read_to_string("/proc/meminfo") else {
        return 0.0;
    };

    let mut total = None;
    let mut available = None;

    for line in meminfo.lines() {
        if let Some(value) = line.strip_prefix("MemTotal:") {
            total = value
                .split_whitespace()
                .next()
                .and_then(|x| x.parse::<f64>().ok());
        }

        if let Some(value) = line.strip_prefix("MemAvailable:") {
            available = value
                .split_whitespace()
                .next()
                .and_then(|x| x.parse::<f64>().ok());
        }
    }

    match (total, available) {
        (Some(total), Some(available)) if total > 0.0 => {
            1.0 - available / total
        }

        _ => 0.0,
    }
}

// ------------------------------------------------------------
// Network throughput
// ------------------------------------------------------------

fn network_bytes() -> Option<u64> {
    let data = fs::read_to_string("/proc/net/dev").ok()?;

    let mut total = 0;

    for line in data.lines().skip(2) {
        let (interface, values) = line.split_once(':')?;

        if interface.trim() == "lo" {
            continue;
        }

        let fields: Vec<&str> =
            values.split_whitespace().collect();

        if fields.len() < 9 {
            continue;
        }

        let rx = fields[0].parse::<u64>().unwrap_or(0);
        let tx = fields[8].parse::<u64>().unwrap_or(0);

        total += rx + tx;
    }

    Some(total)
}

fn network_activity(
    previous: &mut Option<u64>,
) -> f64 {
    let Some(current) = network_bytes() else {
        return 0.0;
    };

    let delta = previous
        .map(|old| current.saturating_sub(old))
        .unwrap_or(0);

    *previous = Some(current);

    // 1 MiB/s fills the graph.
    // Change this depending on how sensitive you want it.
    const SCALE: f64 = 1024.0 * 1024.0;

    (delta as f64 / SCALE).clamp(0.0, 1.0)
}

// ------------------------------------------------------------
// Graph drawing
// ------------------------------------------------------------

fn draw_graph(
    cr: &gtk4::cairo::Context,
    width: i32,
    height: i32,
    history: &History,
) {
    if history.values.len() < 2 {
        return;
    }

    let width = width as f64;
    let height = height as f64;

    let top = 5.0;
    let bottom = height - 5.0;
    let graph_height = bottom - top;

    cr.set_source_rgba(0.8, 1.0, 0.85, 0.9);
    cr.set_line_width(1.0);

    let step =
        width / (history.values.len() - 1) as f64;

    for (i, value) in history.values.iter().enumerate() {
        let x = i as f64 * step;
        let y = bottom - value * graph_height;

        if i == 0 {
            cr.move_to(x, y);
        } else {
            cr.line_to(x, y);
        }
    }

    cr.stroke().unwrap();
}

fn graph_widget(
    history: Rc<RefCell<History>>,
) -> DrawingArea {
    let area = DrawingArea::new();

    area.set_content_width(70);
    area.set_content_height(BAR_HEIGHT);

    area.set_draw_func(
        move |_area, cr, width, height| {
            draw_graph(
                cr,
                width,
                height,
                &history.borrow(),
            );
        },
    );

    area
}

// ------------------------------------------------------------
// Battery text
// ------------------------------------------------------------

fn battery_hex() -> String {
    match battery() {
        Some(capacity) => format!("{capacity:02X}"),
        None => "??".into(),
    }
}

// ------------------------------------------------------------
// UI
// ------------------------------------------------------------

fn build_ui(app: &Application) {
    let overlay = Overlay::new();

    // ========================================================
    // BASE LAYOUT
    //
    // LEFT --------------------------------------------- RIGHT
    // ========================================================

    let base = GtkBox::new(
        Orientation::Horizontal,
        10,
    );

    base.set_margin_start(10);
    base.set_margin_end(10);

    // --------------------------------------------------------
    // Histories
    // --------------------------------------------------------

    let cpu_history =
        Rc::new(RefCell::new(History::new()));

    let ram_history =
        Rc::new(RefCell::new(History::new()));

    let net_history =
        Rc::new(RefCell::new(History::new()));

    // --------------------------------------------------------
    // LEFT
    // --------------------------------------------------------

    let left = GtkBox::new(
        Orientation::Horizontal,
        5,
    );

    let cpu_label = Label::new(Some("CPU"));
    let cpu_graph =
        graph_widget(cpu_history.clone());

    let ram_label = Label::new(Some("RAM"));
    let ram_graph =
        graph_widget(ram_history.clone());

    let net_graph_label = Label::new(Some("NET"));
    let net_graph =
        graph_widget(net_history.clone());

    left.append(&cpu_label);
    left.append(&cpu_graph);

    left.append(&ram_label);
    left.append(&ram_graph);

    left.append(&net_graph_label);
    left.append(&net_graph);

    base.append(&left);

    // --------------------------------------------------------
    // Flexible space
    // --------------------------------------------------------

    let spacer = GtkBox::new(
        Orientation::Horizontal,
        0,
    );

    spacer.set_hexpand(true);

    base.append(&spacer);

    // --------------------------------------------------------
    // RIGHT
    // --------------------------------------------------------

    let right = GtkBox::new(
        Orientation::Horizontal,
        10,
    );

    let network_label =
        Label::new(Some(&network()));

    right.append(&network_label);

    // Battery graphic

    let battery_area = DrawingArea::new();

    battery_area.set_content_width(60);
    battery_area.set_content_height(BAR_HEIGHT);

    battery_area.set_draw_func(
        |_area, cr, _width, _height| {
            draw_battery(
                cr,
                battery().unwrap_or(0),
            );
        },
    );

    right.append(&battery_area);

    // Battery percentage encoded as hex

    let battery_hex_label =
        Label::new(Some(&battery_hex()));

    right.append(&battery_hex_label);

    // NATO time

    let nato_label =
        Label::new(Some(&military_time()));

    right.append(&nato_label);

    base.append(&right);

    overlay.set_child(Some(&base));

    // ========================================================
    // CENTER
    //
    // GtkOverlay means this is centered independently from
    // left/right widths.
    // ========================================================

    let binary_clock = DrawingArea::new();

    binary_clock.set_content_width(200);
    binary_clock.set_content_height(BAR_HEIGHT);

    binary_clock.set_halign(gtk4::Align::Center);
    binary_clock.set_valign(gtk4::Align::Center);

    binary_clock.set_draw_func(
        |_area, cr, _width, _height| {
            draw_binary_clock(cr);
        },
    );

    overlay.add_overlay(&binary_clock);

    // ========================================================
    // Window
    // ========================================================

    let window = ApplicationWindow::builder()
        .application(app)
        .height_request(BAR_HEIGHT)
        .child(&overlay)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Top);

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    window.set_exclusive_zone(BAR_HEIGHT);

    // --------------------------------------------------------
    // CSS
    // --------------------------------------------------------

    let css = gtk4::CssProvider::new();

    css.load_from_data(
        r#"
        window {
            background-color: rgba(0, 0, 0, 0.64);
        }

        label {
            color: rgba(210, 255, 220, 0.95);
            font-family: monospace;
            font-size: 12px;
        }
        "#,
    );

    gtk4::style_context_add_provider_for_display(
        &WidgetExt::display(&window),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // ========================================================
    // Sampling
    // ========================================================

    let previous_cpu =
        Rc::new(RefCell::new(None::<CpuSample>));

    let previous_net =
        Rc::new(RefCell::new(None::<u64>));

    let cpu_history_tick = cpu_history.clone();
    let ram_history_tick = ram_history.clone();
    let net_history_tick = net_history.clone();

    let cpu_graph_tick = cpu_graph.clone();
    let ram_graph_tick = ram_graph.clone();
    let net_graph_tick = net_graph.clone();

    let binary_clock_tick = binary_clock.clone();

    let previous_cpu_tick = previous_cpu.clone();
    let previous_net_tick = previous_net.clone();

    // Once per second:
    //
    // CPU
    // RAM
    // network traffic
    // binary clock

    glib::timeout_add_seconds_local(
        1,
        move || {
            let cpu = cpu_usage(
                &mut previous_cpu_tick.borrow_mut(),
            );

            let ram = ram_usage();

            let net = network_activity(
                &mut previous_net_tick.borrow_mut(),
            );

            cpu_history_tick
                .borrow_mut()
                .push(cpu);

            ram_history_tick
                .borrow_mut()
                .push(ram);

            net_history_tick
                .borrow_mut()
                .push(net);

            cpu_graph_tick.queue_draw();
            ram_graph_tick.queue_draw();
            net_graph_tick.queue_draw();

            binary_clock_tick.queue_draw();

            glib::ControlFlow::Continue
        },
    );

    // --------------------------------------------------------
    // Slower status updates
    // --------------------------------------------------------

    let battery_area_tick =
        battery_area.clone();

    let battery_hex_tick =
        battery_hex_label.clone();

    let network_tick =
        network_label.clone();

    let nato_tick =
        nato_label.clone();

    glib::timeout_add_seconds_local(
        10,
        move || {
            battery_area_tick.queue_draw();

            battery_hex_tick
                .set_text(&battery_hex());

            network_tick
                .set_text(&network());

            nato_tick
                .set_text(&military_time());

            glib::ControlFlow::Continue
        },
    );

    window.present();
}


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

/*
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
*/

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    app.run();
}

use chrono::{Local, Offset};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::fs;
use std::process::Command;

const APP_ID: &str = "dev.noe.rustbar";

// J is skipped, TODO explicit matter
fn nato_zone(offset_seconds: i32) -> char {
    match offset_seconds / 3600 {
        1..=12 => (b'A' + (offset_seconds / 3600 - 1) as u8) as char,
        -12..=-1 => (b'N' + (-offset_seconds / 3600 - 1) as u8) as char,
        0 => 'Z',
        _ => '?',
    }
}

fn time() -> String {
    let now = Local::now();
    let offset = now.offset().fix().local_minus_utc();
    let zone = nato_zone(offset);
    format!("{}{zone}", now.format("%H%M"))
}

fn battery() -> Option<(u8, String)> {
    let base = "/sys/class/power_supply/BAT0";
    let capacity = fs::read_to_string(format!("{base}/capacity"))
        .ok()?
        .trim()
        .parse::<u8>()
        .ok()?;
    let status = fs::read_to_string(format!("{base}/status"))
        .ok()?
        .trim()
        .to_string();
    Some((capacity, status))
}

fn network() -> String {
    let output = Command::new("iwctl")
        .args(["station", "wlan0", "show"])
        .output();
    let Ok(output) = output else {
        return "NET ?".into();
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        if let Some(network) = line.strip_prefix("Connected network") {
            return format!("WIFI {}", network.trim());
        }
    }
    "WIFI DOWN".into()
}

fn status_text() -> String {
    let time = time();
    let battery = match battery() {
        Some((capacity, status)) => {
            let marker = match status.as_str() {
                "Charging" => "+",
                "Full" => "=",
                _ => "",
            };
            format!("BAT {capacity}%{marker}")
        }
        None => "BAT ?".into(),
    };
    let network = network();
    format!("{network}    {battery}    {time}")
}

fn build_ui(app: &Application) {
    let label = Label::new(None);
    label.set_xalign(1.0);
    label.set_margin_end(12);
    let window = ApplicationWindow::builder()
        .application(app)
        .title("rustbar")
        .height_request(28)
        .child(&label)
        .build();
    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    // Reserve 28 pixels so Hyprland doesn't place windows underneath it.
    window.set_exclusive_zone(28);
    label.set_text(&status_text());
    let label_clone = label.clone();
    glib::timeout_add_seconds_local(30, move || {
        label_clone.set_text(&status_text());
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

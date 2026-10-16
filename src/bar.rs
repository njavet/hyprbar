use crate::system::{hyprland, procfs, sysfs, nato};
use crate::theme;
use crate::widgets::{battery, binary_clock, graph};

use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Label, Orientation, Overlay};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

// -----------------------------------------------------------------------------
// Labels
// -----------------------------------------------------------------------------
fn battery_perc() -> String {
    sysfs::battery()
        .map(|bat| format!("{:02}%", bat.capacity))
        .unwrap_or_else(|| "--,-%".into())
}

fn battery_power() -> String {
    sysfs::battery()
        .map(|bat| format!("{:.1}W", bat.power_w))
        .unwrap_or_else(|| "--.-W".into())
}

fn cpu_temperature() -> String {
    sysfs::temperature()
        .map(|temp| format!("{:02}°C", temp.round() as u32))
        .unwrap_or_else(|| "??°".into())
}

fn network_name() -> String {
    sysfs::active_network().unwrap_or_else(|| "OFFLINE".into())
}

fn workspace_text() -> String {
    let workspaces = hyprland::workspaces();
    let active = hyprland::active_workspace();
    workspaces
        .into_iter()
        .map(|id| {
            if Some(id) == active {
                format!("[{id}]")
            } else {
                id.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
// -----------------------------------------------------------------------------
// Main bar
// -----------------------------------------------------------------------------
pub fn build(app: &Application) {
    // Overlay gives us:
    // [ left ------------------------------- right ]
    //                    +
    //             [ exact center ]
    //
    // Therefore unequal left/right widths do NOT move the binary clock.
    let overlay = Overlay::new();

    // =========================================================================
    // Base layout
    // =========================================================================
    let base = GtkBox::new(Orientation::Horizontal, 16);
    base.set_margin_start(16);
    base.set_margin_end(16);

    // =========================================================================
    // LEFT
    // =========================================================================
    let left = GtkBox::new(Orientation::Horizontal, 5);
    let workspace_label = Label::new(Some(&workspace_text()));
    let ascii_spacer0 = Label::new(Some(" - "));
    workspace_label.add_css_class("workspaces");
    left.append(&workspace_label);
    left.append(&ascii_spacer0);

    // -------------------------------------------------------------------------
    // Graph state
    // -------------------------------------------------------------------------
    let cpu_history = Rc::new(RefCell::new(graph::History::new()));
    let ram_history = Rc::new(RefCell::new(graph::History::new()));
    let net_history = Rc::new(RefCell::new(graph::History::new()));

    // -------------------------------------------------------------------------
    // CPU graph
    // -------------------------------------------------------------------------
    let temp_label = Label::new(None);
    let cpu_label = Label::new(Some("CPU:"));
    let cpu_graph = graph::widget(cpu_history.clone());
    let ascii_spacer1 = Label::new(Some("|"));
    temp_label.set_text(&cpu_temperature());
    left.append(&cpu_label);
    left.append(&temp_label);
    left.append(&ascii_spacer1);
    left.append(&cpu_graph);

    // -------------------------------------------------------------------------
    // RAM graph
    // -------------------------------------------------------------------------
    let ram_label = Label::new(Some("RAM"));
    let ram_graph = graph::widget(ram_history.clone());
    let ascii_spacer2 = Label::new(Some("|"));
    let ascii_spacer3 = Label::new(Some("|"));
    let ascii_spacer4 = Label::new(Some("|"));
    left.append(&ascii_spacer2);
    left.append(&ram_label);
    left.append(&ascii_spacer3);
    left.append(&ram_graph);
    left.append(&ascii_spacer4);

    // -------------------------------------------------------------------------
    // Network graph
    // -------------------------------------------------------------------------
    let net_label = Label::new(Some("NET:"));
    let net_graph = graph::widget(net_history.clone());
    let network_label = Label::new(None);
    let ascii_spacer5 = Label::new(Some("|"));
    let ascii_spacer6 = Label::new(Some("|"));
    left.append(&net_label);
    left.append(&network_label);
    network_label.set_text(&network_name());
    left.append(&ascii_spacer5);
    left.append(&net_graph);
    left.append(&ascii_spacer6);

    base.append(&left);

    // =========================================================================
    // Flexible space
    // =========================================================================
    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    base.append(&spacer);

    // =========================================================================
    // RIGHT
    // =========================================================================
    let right = GtkBox::new(Orientation::Horizontal, 8);

    // -------------------------------------------------------------------------
    // Battery graphic
    // -------------------------------------------------------------------------
    let battery_area = battery::widget();
    right.append(&battery_area);

    // -------------------------------------------------------------------------
    // Battery as hexadecimal percentage
    // -------------------------------------------------------------------------
    let battery_label = Label::new(None);
    battery_label.set_text(&battery_perc());
    right.append(&battery_label);

    // -------------------------------------------------------------------------
    // Battery power
    // -------------------------------------------------------------------------
    let battery_power_label = Label::new(None);
    battery_power_label.set_text(&battery_power());
    right.append(&battery_power_label);

    // -------------------------------------------------------------------------
    // NATO time
    // -------------------------------------------------------------------------
    //let nato_label = Label::new(None);
    //nato_label.set_text(&nato::nato_time());
    //right.append(&nato_label);
    base.append(&right);
    overlay.set_child(Some(&base));

    // =========================================================================
    // CENTER
    // =========================================================================
    let clock = binary_clock::widget();
    clock.set_halign(Align::Center);
    clock.set_valign(Align::Center);
    overlay.add_overlay(&clock);

    // =========================================================================
    // Window
    // =========================================================================
    let window = ApplicationWindow::builder()
        .application(app)
        .height_request(theme::BAR_HEIGHT)
        .child(&overlay)
        .build();
    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_exclusive_zone(theme::BAR_HEIGHT);
    theme::install(&window);

    // =========================================================================
    // Sampling state
    // =========================================================================
    let previous_cpu = Rc::new(RefCell::new(None::<procfs::CpuSample>));
    let previous_network = Rc::new(RefCell::new(None::<u64>));
    // =========================================================================
    // 1 Hz
    //
    // Things which genuinely benefit from frequent sampling:
    //
    // - CPU
    // - RAM
    // - network throughput
    // - seconds in binary clock
    // =========================================================================
    let cpu_history_tick = cpu_history.clone();
    let ram_history_tick = ram_history.clone();
    let net_history_tick = net_history.clone();
    let cpu_graph_tick = cpu_graph.clone();
    let ram_graph_tick = ram_graph.clone();
    let net_graph_tick = net_graph.clone();
    let clock_tick = clock.clone();
    let previous_cpu_tick = previous_cpu.clone();
    let previous_network_tick = previous_network.clone();
    glib::timeout_add_seconds_local(1, move || {

        // ---------------------------------------------------------------------
        // CPU
        // ---------------------------------------------------------------------
        let cpu = procfs::cpu_usage(&mut previous_cpu_tick.borrow_mut());
        cpu_history_tick.borrow_mut().push(cpu);

        // ---------------------------------------------------------------------
        // RAM
        // ---------------------------------------------------------------------
        let ram = procfs::ram_usage();
        ram_history_tick.borrow_mut().push(ram);

        // ---------------------------------------------------------------------
        // Network throughput
        // ---------------------------------------------------------------------
        let current_network = procfs::network_bytes().unwrap_or(0);
        let mut previous = previous_network_tick.borrow_mut();
        let delta = previous
            .map(|old| current_network.saturating_sub(old))
            .unwrap_or(0);
        *previous = Some(current_network);
        // 1 MiB/s = full graph height.
        //
        // We can improve this later with logarithmic scaling.
        let net = (delta as f64 / (1024.0 * 1024.0)).clamp(0.0, 1.0);
        net_history_tick.borrow_mut().push(net);

        // ---------------------------------------------------------------------
        // Redraw
        // ---------------------------------------------------------------------
        cpu_graph_tick.queue_draw();
        ram_graph_tick.queue_draw();
        net_graph_tick.queue_draw();
        clock_tick.queue_draw();
        glib::ControlFlow::Continue
    });

    // =========================================================================
    // 5 Hz? No.
    //
    // 1 second polling for Hyprland isn't necessary either, but for this first
    // version polling every 2 seconds is simple. Later we'll replace it with
    // Hyprland's event socket.
    // =========================================================================
    let workspace_tick = workspace_label.clone();
    glib::timeout_add_seconds_local(2, move || {
        workspace_tick.set_text(&workspace_text());
        glib::ControlFlow::Continue
    });

    // =========================================================================
    // Slow system information
    //
    // None of these need 1 Hz updates.
    // =========================================================================
    let temp_tick = temp_label.clone();
    let network_tick = network_label.clone();
    let battery_area_tick = battery_area.clone();
    let battery_tick = battery_label.clone();
    let battery_power_tick = battery_power_label.clone();
    //let nato_tick = nato_label.clone();
    glib::timeout_add_seconds_local(10, move || {
        temp_tick.set_text(&cpu_temperature());
        network_tick.set_text(&network_name());
        battery_area_tick.queue_draw();
        battery_tick.set_text(&battery_perc());
        battery_power_tick.set_text(&battery_power());
     //   nato_tick.set_text(&nato::nato_time());
        glib::ControlFlow::Continue
    });

    // =========================================================================
    // Show it
    // =========================================================================
    window.present();
}

// Conceptually:

let left = GtkBox::new(...);

left.append(&workspace_widget);
left.append(&cpu_graph);
left.append(&ram_graph);
left.append(&net_graph);
left.append(&window_title);

let center = binary_clock::widget();

let right = GtkBox::new(...);

right.append(&temperature);
right.append(&network);
right.append(&battery);
right.append(&battery_hex);
right.append(&battery_power);
right.append(&nato_time);

use gtk4::cairo::Context;

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

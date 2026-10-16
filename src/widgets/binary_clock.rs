use gtk4::cairo::Context;

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

use crate::system::sysfs;
use crate::theme::{BAR_HEIGHT, FG, FG_DIM};

use gtk4::DrawingArea;
use gtk4::prelude::*;

pub fn widget() -> DrawingArea {
    let area = DrawingArea::new();
    area.set_content_width(54);
    area.set_content_height(BAR_HEIGHT);
    area.set_draw_func(|_, cr, _, _| {
        let capacity = sysfs::battery()
            .map(|battery| battery.capacity)
            .unwrap_or(0);
        let x = 2.0;
        let y = 9.0;
        let width = 44.0;
        let height = 10.0;
        // Outline
        cr.set_source_rgba(FG_DIM.0, FG_DIM.1, FG_DIM.2, 0.6);
        cr.rectangle(x, y, width, height);
        let _ = cr.stroke();
        // Filled part
        let fill = (width - 4.0) * capacity as f64 / 100.0;
        cr.set_source_rgba(FG.0, FG.1, FG.2, FG.3);
        cr.rectangle(x + 2.0, y + 2.0, fill, height - 4.0);
        let _ = cr.fill();
        // Battery terminal
        cr.rectangle(x + width + 2.0, y + 3.0, 3.0, height - 6.0);
        let _ = cr.fill();
    });
    area
}

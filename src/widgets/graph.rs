use crate::theme::{BAR_HEIGHT, FG};
use gtk4::DrawingArea;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

const HISTORY: usize = 40;

pub struct History {
    values: VecDeque<f64>,
}

impl History {
    pub fn new() -> Self {
        Self {
            values: VecDeque::from(
                vec![0.0; HISTORY]
            ),
        }
    }

    pub fn push(&mut self, value: f64) {
        if self.values.len() == HISTORY {
            self.values.pop_front();
        }

        self.values
            .push_back(value.clamp(0.0, 1.0));
    }
}

pub fn widget(
    history: Rc<RefCell<History>>,
) -> DrawingArea {
    let area = DrawingArea::new();

    area.set_content_width(60);
    area.set_content_height(BAR_HEIGHT);

    area.set_draw_func(
        move |_, cr, width, height| {
            let history = history.borrow();

            if history.values.len() < 2 {
                return;
            }

            let bottom = height as f64 - 5.0;
            let graph_height = height as f64 - 10.0;

            let step =
                width as f64
                    / (history.values.len() - 1) as f64;

            cr.set_source_rgba(
                FG.0, FG.1, FG.2, FG.3,
            );

            cr.set_line_width(1.0);

            for (i, value) in
                history.values.iter().enumerate()
            {
                let x = i as f64 * step;
                let y =
                    bottom - value * graph_height;

                if i == 0 {
                    cr.move_to(x, y);
                } else {
                    cr.line_to(x, y);
                }
            }

            let _ = cr.stroke();
        },
    );

    area
}

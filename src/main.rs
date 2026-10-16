mod bar;
mod system;
mod theme;
mod widgets;

use gtk4::Application;
use gtk4::prelude::*;

const APP_ID: &str = "dev.noe.rustbar";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(bar::build);
    app.run();
}

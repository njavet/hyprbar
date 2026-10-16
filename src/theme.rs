use gtk4::prelude::*;

pub const BAR_HEIGHT: i32 = 28;

pub const FG: (f64, f64, f64, f64) = (0.80, 1.00, 0.85, 0.95);

pub const FG_DIM: (f64, f64, f64, f64) = (0.80, 1.00, 0.85, 0.20);

pub fn install(window: &impl IsA<gtk4::Widget>) {
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
        &window.display(),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

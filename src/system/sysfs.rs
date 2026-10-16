use std::fs;

#[derive(Debug, Clone)]
pub struct Battery {
    pub capacity: u8,
    pub power_w: Option<f64>,
}

pub fn battery() -> Option<Battery> {
    let base = "/sys/class/power_supply/BAT0";

    let capacity = fs::read_to_string(
        format!("{base}/capacity"),
    )
    .ok()?
    .trim()
    .parse()
    .ok()?;

    let power_w = fs::read_to_string(
        format!("{base}/power_now"),
    )
    .ok()
    .and_then(|v| v.trim().parse::<f64>().ok())
    .map(|uw| uw / 1_000_000.0);

    Some(Battery {
        capacity,
        power_w,
    })
}

pub fn temperature() -> Option<f64> {
    let entries =
        fs::read_dir("/sys/class/thermal").ok()?;

    for entry in entries.flatten() {
        let path = entry.path();

        if !path
            .file_name()?
            .to_string_lossy()
            .starts_with("thermal_zone")
        {
            continue;
        }

        let temp = fs::read_to_string(
            path.join("temp"),
        )
        .ok()?;

        if let Ok(value) = temp.trim().parse::<f64>() {
            return Some(value / 1000.0);
        }
    }

    None
}

pub fn active_network() -> Option<String> {
    let entries = fs::read_dir("/sys/class/net").ok()?;

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if name == "lo" {
            continue;
        }

        let state = fs::read_to_string(
            format!("/sys/class/net/{name}/operstate"),
        )
        .ok();

        if state.as_deref().map(str::trim) == Some("up") {
            return Some(name.into_owned());
        }
    }

    None
}

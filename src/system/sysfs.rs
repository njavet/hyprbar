use std::fs;

#[derive(Debug, Clone)]
pub struct Battery {
    pub capacity: u8,
    pub power_w: f64,
    pub charging: bool,
}

pub fn battery() -> Option<Battery> {
    let base = "/sys/class/power_supply/BAT0";

    // TODO how does rust know it's u8 parse ?
    let capacity = fs::read_to_string(format!("{base}/capacity"))
        .ok()?
        .trim()
        .parse()
        .ok()?;

    let power_w = fs::read_to_string(format!("{base}/power_now"))
        .ok()?
        .trim()
        .parse()
        .map(|uw: f64| uw / 1_000_000.0)
        .ok()?;

    let charging;
    if fs::read_to_string(format!("{base}/status")).ok()?.trim() == "Not charging" {
        charging = false;
    } else {
        charging = true;
    }

    Some(Battery {
        capacity,
        power_w,
        charging,
    })
}

pub fn temperature() -> Option<f64> {
    let entries = fs::read_dir("/sys/class/thermal").ok()?;

    for entry in entries.flatten() {
        let path = entry.path();

        if !path
            .file_name()?
            .to_string_lossy()
            .starts_with("thermal_zone")
        {
            continue;
        }

        let temp = fs::read_to_string(path.join("temp")).ok()?;

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
        let state = fs::read_to_string(format!("/sys/class/net/{name}/operstate")).ok();
        if state.as_deref().map(str::trim) == Some("up") {
            return Some(name.into_owned());
        }
    }
    None
}

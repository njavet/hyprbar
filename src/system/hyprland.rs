use std::process::Command;
use serde_json::Value;

pub fn active_window() -> Option<String> {
    let output = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
        .ok()?;

    let json: Value =
        serde_json::from_slice(&output.stdout).ok()?;

    json["title"]
        .as_str()
        .map(str::to_owned)
}

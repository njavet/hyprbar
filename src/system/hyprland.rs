use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct Workspace {
    id: i32,
}

#[derive(Debug, Deserialize)]
struct ActiveWorkspace {
    id: i32,
}

fn hyprctl(args: &[&str]) -> Option<Vec<u8>> {
    let output = Command::new("hyprctl").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(output.stdout)
}

pub fn workspaces() -> Vec<i32> {
    let Some(data) = hyprctl(&["workspaces", "-j"]) else {
        return Vec::new();
    };
    let Ok(workspaces) = serde_json::from_slice::<Vec<Workspace>>(&data) else {
        return Vec::new();
    };
    let mut ids: Vec<i32> = workspaces
        .into_iter()
        .map(|workspace| workspace.id)
        // Ignore Hyprland special workspaces.
        .filter(|id| *id > 0)
        .collect();
    ids.sort_unstable();
    ids
}

pub fn active_workspace() -> Option<i32> {
    let data = hyprctl(&["activeworkspace", "-j"])?;
    let workspace = serde_json::from_slice::<ActiveWorkspace>(&data).ok()?;
    Some(workspace.id)
}

pub fn active_window() -> Option<String> {
    let data = hyprctl(&["activewindow", "-j"])?;
    let json: serde_json::Value = serde_json::from_slice(&data).ok()?;
    json["title"].as_str().map(str::to_owned)
}

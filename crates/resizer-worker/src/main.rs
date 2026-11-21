#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use resizer_core::{AppConfig, ToastPayload, process_batch};

fn find_ui_binary() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    let dir = exe.parent()?;
    let name = if cfg!(target_os = "windows") {
        "imageresizej.exe"
    } else {
        "imageresizej"
    };
    let candidate = dir.join(name);
    candidate.is_file().then_some(candidate)
}

fn show_toast(payload: &ToastPayload) {
    if let Some(ui_bin) = find_ui_binary()
        && let Ok(json) = serde_json::to_string(payload)
    {
        let _ = Command::new(ui_bin).arg("--toast").arg(json).spawn();
    }
}

fn collect_input_paths() -> Vec<PathBuf> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 2 && args[0] == "--from-file" {
        let list_path = PathBuf::from(&args[1]);
        let content = fs::read_to_string(&list_path).unwrap_or_default();
        let _ = fs::remove_file(&list_path);
        let text = content.strip_prefix('\u{feff}').unwrap_or(&content);
        return text
            .lines()
            .map(|line| line.trim_end_matches('\r'))
            .filter(|line| !line.is_empty())
            .map(PathBuf::from)
            .collect();
    }
    args.into_iter().map(PathBuf::from).collect()
}

fn main() {
    let config = AppConfig::load_or_create();
    let paths = collect_input_paths();

    if paths.is_empty() {
        if let Some(ui_bin) = find_ui_binary() {
            let _ = Command::new(ui_bin).spawn();
        }
        return;
    }

    let summary = process_batch(&paths, &config);

    if config.show_notifications {
        let payload = ToastPayload {
            succeeded: summary.succeeded,
            failed: summary.failed,
            elapsed_ms: summary.elapsed.as_millis() as u64,
            format: config.format,
            theme: config.theme,
            log_path: summary.log_path.map(|p| p.display().to_string()),
        };
        show_toast(&payload);
    }
}

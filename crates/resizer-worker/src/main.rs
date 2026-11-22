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

#[cfg(not(target_os = "linux"))]
fn show_toast(payload: &ToastPayload) {
    if let Some(ui_bin) = find_ui_binary()
        && let Ok(json) = serde_json::to_string(payload)
    {
        let _ = Command::new(ui_bin).arg("--toast").arg(json).spawn();
    }
}

#[cfg(target_os = "linux")]
fn show_toast(payload: &ToastPayload) {
    use notify_rust::{Notification, Timeout, Urgency};
    use resizer_core::ResolvedLanguage;

    let is_ru = payload.language == ResolvedLanguage::Ru;
    let sec = if payload.elapsed_ms < 1000 {
        format!("{:.2}", payload.elapsed_ms as f64 / 1000.0)
    } else {
        format!("{:.1}", payload.elapsed_ms as f64 / 1000.0)
    };

    let (title, body, timeout_ms, urgency) = if payload.failed > 0 {
        let title = match (is_ru, payload.succeeded > 0) {
            (true, true) => "Готово с ошибками",
            (true, false) => "Ошибка обработки",
            (false, true) => "Done with errors",
            (false, false) => "Processing failed",
        }
        .to_string();

        let mut body = if is_ru {
            format!(
                "Успешно: {} · Ошибок: {} ({sec} с)",
                payload.succeeded, payload.failed
            )
        } else {
            format!(
                "Succeeded: {} · Failed: {} ({sec} s)",
                payload.succeeded, payload.failed
            )
        };

        if let Some(log) = &payload.log_path {
            let label = if is_ru { "Лог" } else { "Log" };
            body.push_str(&format!("\n{label}: {log}"));
        }
        (title, body, 6000, Urgency::Critical)
    } else {
        let fmt = payload.format.extension().to_uppercase();
        let (title, body) = if is_ru {
            (
                format!("Готово · файлов: {}", payload.succeeded),
                format!("Сохранено в {fmt} за {sec} с"),
            )
        } else {
            (
                format!("Done · files: {}", payload.succeeded),
                format!("Saved as {fmt} in {sec} s"),
            )
        };
        (title, body, 3500, Urgency::Normal)
    };

    let _ = Notification::new()
        .appname("ImageResizerJ")
        .summary(&title)
        .body(&body)
        .icon("imageresizej")
        .urgency(urgency)
        .timeout(Timeout::Milliseconds(timeout_ms))
        .show();
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
            language: config.resolved_language(),
            log_path: summary.log_path.map(|p| p.display().to_string()),
        };
        show_toast(&payload);
    }
}
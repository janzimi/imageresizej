#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use std::time::Duration;

use resizer_core::{AppConfig, ToastPayload};
use serde::{Deserialize, Serialize};
use tao::dpi::{LogicalSize, PhysicalPosition};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::{Icon, Window, WindowBuilder};
use wry::{WebContext, WebView, WebViewBuilder};

const SETTINGS_HTML: &str = include_str!("../assets/settings.html");
const TOAST_HTML: &str = include_str!("../assets/toast.html");
const APP_ICON_BYTES: &[u8] = include_bytes!("../../../assets/icon.ico");

/// Гарантирует правильный порядок освобождения ресурсов (сверху вниз):
/// сначала WebView, затем нативное окно, и только потом WebContext.
struct ManagedWebView {
    _webview: WebView,
    window: Window,
    _web_context: WebContext,
}

#[derive(Serialize)]
struct SettingsState {
    config: AppConfig,
    config_path: String,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum IpcMessage {
    Ready { height: f64 },
    Save { config: AppConfig },
    Drag,
    Minimize,
    Close,
    OpenLog { path: String },
}

enum UserEvent {
    Ready(f64),
    Drag,
    Minimize,
    Close,
    OpenLog(String),
}

fn load_window_icon() -> Option<Icon> {
    let img = image::load_from_memory(APP_ICON_BYTES).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    Icon::from_rgba(img.into_raw(), w, h).ok()
}

fn escape_inline_json(json: &str) -> String {
    json.replace("</", "<\\/")
}

fn open_file_in_os(path: &str) {
    #[cfg(target_os = "windows")]
    let _ = Command::new("notepad.exe").arg(path).spawn();

    #[cfg(target_os = "linux")]
    let _ = Command::new("xdg-open").arg(path).spawn();
}

fn log_ui_error(msg: &str) {
    let log_path = AppConfig::webview_data_dir("logs").join("ui_error.log");
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = writeln!(file, "{msg}");
    }
}

fn build_webview(builder: WebViewBuilder<'_>, window: &Window) -> Option<WebView> {
    let res = {
        #[cfg(not(target_os = "linux"))]
        {
            builder.build(window)
        }
        #[cfg(target_os = "linux")]
        {
            use tao::platform::unix::WindowExtUnix;
            use wry::WebViewBuilderExtUnix;
            let vbox = window.default_vbox().expect("GTK vbox");
            builder.build_gtk(vbox)
        }
    };

    match res {
        Ok(wv) => Some(wv),
        Err(e) => {
            log_ui_error(&format!("Ошибка инициализации WebView: {e}"));
            None
        }
    }
}

fn run_settings() {
    let config = AppConfig::load_or_create();
    let state = SettingsState {
        config,
        config_path: AppConfig::config_path().display().to_string(),
    };
    let json = escape_inline_json(&serde_json::to_string(&state).unwrap());
    let html = SETTINGS_HTML.replace("__INITIAL_STATE__", &json);

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let win_w = 480.0_f64;

    let mut wb = WindowBuilder::new()
        .with_title("ImageResizerJ")
        .with_inner_size(LogicalSize::new(win_w, 200.0))
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(false)
        .with_visible(false)
        .with_window_icon(load_window_icon());

    #[cfg(target_os = "windows")]
    {
        use tao::platform::windows::WindowBuilderExtWindows;
        wb = wb.with_undecorated_shadow(false);
    }

    let window = wb.build(&event_loop).expect("Не удалось создать окно");

    let mut web_context = WebContext::new(Some(AppConfig::webview_data_dir("webview")));
    let ipc_proxy = proxy.clone();
    let builder = WebViewBuilder::with_web_context(&mut web_context)
        .with_transparent(true)
        .with_background_color((0, 0, 0, 0))
        .with_html(html)
        .with_ipc_handler(move |req| {
            if let Ok(msg) = serde_json::from_str::<IpcMessage>(req.body()) {
                match msg {
                    IpcMessage::Ready { height } => {
                        let _ = ipc_proxy.send_event(UserEvent::Ready(height));
                    }
                    IpcMessage::Save { config } => {
                        let _ = config.save();
                    }
                    IpcMessage::Drag => {
                        let _ = ipc_proxy.send_event(UserEvent::Drag);
                    }
                    IpcMessage::Minimize => {
                        let _ = ipc_proxy.send_event(UserEvent::Minimize);
                    }
                    IpcMessage::Close => {
                        let _ = ipc_proxy.send_event(UserEvent::Close);
                    }
                    _ => {}
                }
            }
        });

    let Some(webview) = build_webview(builder, &window) else {
        return;
    };

    let managed = ManagedWebView {
        _webview: webview,
        window,
        _web_context: web_context,
    };

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1500));
        let _ = proxy.send_event(UserEvent::Ready(620.0));
    });

    let mut is_shown = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::Ready(content_h)) if !is_shown => {
                is_shown = true;
                let mut win_h = content_h.ceil().clamp(320.0, 920.0);

                if let Some(monitor) = managed.window.primary_monitor() {
                    let scale = monitor.scale_factor();
                    let mon_pos = monitor.position();
                    let mon_size = monitor.size();
                    let screen_h = mon_size.height as f64 / scale;
                    win_h = win_h.min((screen_h - 56.0).max(320.0));

                    let win_w_phys = (win_w * scale).round() as i32;
                    let win_h_phys = (win_h * scale).round() as i32;
                    let x = mon_pos.x + ((mon_size.width as i32 - win_w_phys) / 2).max(0);
                    let y = mon_pos.y + ((mon_size.height as i32 - win_h_phys) / 2).max(0);

                    managed
                        .window
                        .set_inner_size(LogicalSize::new(win_w, win_h));
                    managed
                        .window
                        .set_outer_position(PhysicalPosition::new(x, y));
                } else {
                    managed
                        .window
                        .set_inner_size(LogicalSize::new(win_w, win_h));
                }

                managed.window.set_visible(true);
                managed.window.set_focus();
            }
            Event::UserEvent(UserEvent::Drag) => {
                let _ = managed.window.drag_window();
            }
            Event::UserEvent(UserEvent::Minimize) => {
                managed.window.set_minimized(true);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::UserEvent(UserEvent::Close) => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

fn run_toast(payload_json: &str) {
    let payload: ToastPayload = match serde_json::from_str(payload_json) {
        Ok(p) => p,
        Err(_) => return,
    };

    let safe_json = escape_inline_json(&serde_json::to_string(&payload).unwrap());
    let html = TOAST_HTML.replace("__TOAST_DATA__", &safe_json);

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let toast_w = 368.0_f64;
    let toast_h = 98.0_f64;

    let mut wb = WindowBuilder::new()
        .with_title("ImageResizerJ")
        .with_inner_size(LogicalSize::new(toast_w, toast_h))
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top(true)
        .with_resizable(false)
        .with_focused(false);

    #[cfg(target_os = "windows")]
    {
        use tao::platform::windows::WindowBuilderExtWindows;
        wb = wb.with_skip_taskbar(true).with_undecorated_shadow(false);
    }

    let window = wb
        .build(&event_loop)
        .expect("Не удалось создать окно тоста");

    if let Some(monitor) = window.primary_monitor() {
        let scale = monitor.scale_factor();
        let mon_pos = monitor.position();
        let mon_size = monitor.size();
        let toast_w_phys = ((toast_w + 10.0) * scale).round() as i32;
        let toast_h_phys = ((toast_h + 54.0) * scale).round() as i32;
        let x = mon_pos.x + (mon_size.width as i32 - toast_w_phys).max(0);
        let y = mon_pos.y + (mon_size.height as i32 - toast_h_phys).max(0);
        window.set_outer_position(PhysicalPosition::new(x, y));
    }

    let mut web_context = WebContext::new(Some(AppConfig::webview_data_dir("webview-toast")));
    let ipc_proxy = proxy.clone();
    let builder = WebViewBuilder::with_web_context(&mut web_context)
        .with_transparent(true)
        .with_background_color((0, 0, 0, 0))
        .with_html(html)
        .with_ipc_handler(move |req| {
            if let Ok(msg) = serde_json::from_str::<IpcMessage>(req.body()) {
                match msg {
                    IpcMessage::Close => {
                        let _ = ipc_proxy.send_event(UserEvent::Close);
                    }
                    IpcMessage::OpenLog { path } => {
                        let _ = ipc_proxy.send_event(UserEvent::OpenLog(path));
                    }
                    _ => {}
                }
            }
        });

    let Some(webview) = build_webview(builder, &window) else {
        return;
    };

    let managed = ManagedWebView {
        _webview: webview,
        window,
        _web_context: web_context,
    };

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(20));
        let _ = proxy.send_event(UserEvent::Close);
    });

    event_loop.run(move |event, _, control_flow| {
        let _ = &managed;
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::UserEvent(UserEvent::Close) => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(UserEvent::OpenLog(path)) => {
                open_file_in_os(&path);
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 3 && args[1] == "--toast" {
        run_toast(&args[2]);
    } else {
        run_settings();
    }
}

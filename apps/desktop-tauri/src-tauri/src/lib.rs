mod launcher;
mod models;
mod service;

use models::FrontendState;
use service::QuotaService;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State, WindowEvent,
};

#[tauri::command]
fn get_frontend_state(service: State<'_, QuotaService>) -> FrontendState {
    service.state()
}

#[tauri::command]
fn connect_codex(service: State<'_, QuotaService>) -> Result<(), String> {
    service.connect()
}

#[tauri::command]
fn refresh_codex(service: State<'_, QuotaService>) -> Result<(), String> {
    service.refresh()
}

#[tauri::command]
fn reconnect_codex(service: State<'_, QuotaService>) -> Result<(), String> {
    service.reconnect()
}

fn toggle_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, "toggle", "显示/隐藏", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "刷新额度", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle, &refresh, &quit])?;

    let mut builder = TrayIconBuilder::with_id("quota-tray")
        .tooltip("Codex 额度")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle" => toggle_window(app),
            "refresh" => {
                let _ = app.state::<QuotaService>().refresh();
            }
            "quit" => {
                let _ = app.state::<QuotaService>().disconnect();
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                toggle_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .invoke_handler(tauri::generate_handler![
            get_frontend_state,
            connect_codex,
            refresh_codex,
            reconnect_codex
        ])
        .setup(|app| {
            let service = QuotaService::start(app.handle().clone());
            app.manage(service.clone());
            build_tray(app.handle())?;

            if std::env::args().any(|argument| argument == "--hidden") {
                if let Some(window) = app.get_webview_window("main") {
                    window.hide()?;
                }
            }
            service.connect()?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Codex Quota Tool");
}

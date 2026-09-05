mod launcher;
mod models;
mod service;
mod settings;

use std::sync::RwLock;

use models::{AppSettings, FrontendState, ResolvedLanguage, SavedPosition};
use service::QuotaService;
use settings::SettingsStore;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, PhysicalPosition, State, WindowEvent, Wry,
};
use tauri_plugin_autostart::ManagerExt;

pub(crate) struct LanguageState(RwLock<ResolvedLanguage>);

impl Default for LanguageState {
    fn default() -> Self {
        Self(RwLock::new(ResolvedLanguage::English))
    }
}

impl LanguageState {
    fn set(&self, language: ResolvedLanguage) {
        *self.0.write().unwrap() = language;
    }

    pub(crate) fn get(&self) -> ResolvedLanguage {
        *self.0.read().unwrap()
    }
}

pub(crate) fn resolved_language(app: &AppHandle) -> ResolvedLanguage {
    app.try_state::<LanguageState>()
        .map(|state| state.get())
        .unwrap_or_default()
}

#[derive(Clone)]
struct TrayMenuItems {
    show: MenuItem<Wry>,
    widget: MenuItem<Wry>,
    refresh: MenuItem<Wry>,
    startup: MenuItem<Wry>,
    settings: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

impl TrayMenuItems {
    fn localize(&self, language: ResolvedLanguage) -> tauri::Result<()> {
        let chinese = language == ResolvedLanguage::SimplifiedChinese;
        self.show.set_text(if chinese {
            "Codex 用量"
        } else {
            "Codex Usage"
        })?;
        self.widget.set_text(if chinese {
            "显示桌面悬浮窗"
        } else if cfg!(target_os = "windows") {
            "Show Widget"
        } else {
            "Floating Widget"
        })?;
        self.refresh
            .set_text(if chinese { "刷新" } else { "Refresh" })?;
        self.startup.set_text(if chinese {
            if cfg!(target_os = "windows") {
                "开机时启动"
            } else {
                "登录时启动"
            }
        } else if cfg!(target_os = "windows") {
            "Launch at Startup"
        } else {
            "Launch at Login"
        })?;
        self.settings
            .set_text(if chinese { "设置" } else { "Settings" })?;
        self.quit.set_text(if chinese {
            "退出 CodexMeter"
        } else {
            "Quit CodexMeter"
        })?;
        Ok(())
    }
}

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

#[tauri::command]
fn quit_codexmeter(app: AppHandle, service: State<'_, QuotaService>) {
    let _ = service.disconnect();
    app.exit(0);
}

#[tauri::command]
fn save_settings(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    service: State<'_, QuotaService>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let saved = store.replace(settings)?;
    service.update_settings(saved.clone())?;
    apply_widget_settings(&app, &saved);
    Ok(saved)
}

#[tauri::command]
fn set_widget_visible(app: AppHandle, visible: bool) -> Result<(), String> {
    update_widget_visibility(&app, visible)
}

#[tauri::command]
fn set_ui_language(
    app: AppHandle,
    service: State<'_, QuotaService>,
    language: ResolvedLanguage,
) -> Result<(), String> {
    app.state::<LanguageState>().set(language);
    app.state::<TrayMenuItems>()
        .localize(language)
        .map_err(|error| error.to_string())?;
    service.refresh_ui(&app);
    Ok(())
}

fn show_panel(app: &AppHandle, settings_view: bool) {
    let Some(window) = app.get_webview_window("panel") else {
        return;
    };
    let _ = window.show();
    let _ = window.set_focus();
    if settings_view {
        let _ = app.emit("ui://open-settings", ());
    }
}

fn toggle_panel(app: &AppHandle) {
    let Some(window) = app.get_webview_window("panel") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        show_panel(app, false);
    }
}

fn update_widget_visibility(app: &AppHandle, visible: bool) -> Result<(), String> {
    let store = app.state::<SettingsStore>();
    let mut settings = store.get();
    settings.widget_visible = visible;
    let settings = store.replace(settings)?;
    app.state::<QuotaService>()
        .update_settings(settings.clone())?;
    apply_widget_settings(app, &settings);
    Ok(())
}

fn apply_widget_settings(app: &AppHandle, settings: &AppSettings) {
    let Some(window) = app.get_webview_window("widget") else {
        return;
    };
    let _ = window.set_focusable(false);
    if let Some(position) = &settings.widget_position {
        let _ = window.set_position(PhysicalPosition::new(position.x, position.y));
    }
    if settings.widget_visible {
        let _ = window.show();
    } else {
        let _ = window.hide();
    }
}

fn toggle_widget(app: &AppHandle) {
    let visible = app
        .get_webview_window("widget")
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);
    let _ = update_widget_visibility(app, !visible);
}

fn toggle_autostart(app: &AppHandle) {
    let manager = app.autolaunch();
    match manager.is_enabled() {
        Ok(true) => {
            let _ = manager.disable();
        }
        Ok(false) => {
            let _ = manager.enable();
        }
        Err(_) => {}
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<TrayMenuItems> {
    let show = MenuItem::with_id(app, "show", "Codex Usage", true, None::<&str>)?;
    let widget_label = if cfg!(target_os = "windows") {
        "Show Widget"
    } else {
        "Floating Widget"
    };
    let startup_label = if cfg!(target_os = "windows") {
        "Launch at Startup"
    } else {
        "Launch at Login"
    };
    let widget = MenuItem::with_id(app, "widget", widget_label, true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "Refresh", true, None::<&str>)?;
    let startup = MenuItem::with_id(app, "startup", startup_label, true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit CodexMeter", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &show, &widget, &refresh, &startup, &settings, &separator, &quit,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id("codexmeter-tray")
        .tooltip("CodexMeter")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_panel(app, false),
            "widget" => toggle_widget(app),
            "refresh" => {
                let _ = app.state::<QuotaService>().refresh();
            }
            "startup" => toggle_autostart(app),
            "settings" => show_panel(app, true),
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
                toggle_panel(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(TrayMenuItems {
        show,
        widget,
        refresh,
        startup,
        settings,
        quit,
    })
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
            reconnect_codex,
            quit_codexmeter,
            save_settings,
            set_widget_visible,
            set_ui_language
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            app.manage(LanguageState::default());
            let settings = SettingsStore::load(app.handle());
            let service = QuotaService::start(app.handle().clone(), settings.clone());
            app.manage(settings.clone());
            app.manage(service.clone());
            let tray_items = build_tray(app.handle())?;
            app.manage(tray_items);
            apply_widget_settings(app.handle(), &settings.get());

            if !std::env::args().any(|argument| argument == "--hidden") {
                show_panel(app.handle(), false);
            }
            service.connect()?;
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
                if window.label() == "widget" {
                    let _ = update_widget_visibility(window.app_handle(), false);
                }
            }
            WindowEvent::Moved(position) if window.label() == "widget" => {
                let store = window.app_handle().state::<SettingsStore>();
                let mut settings = store.get();
                let position = SavedPosition {
                    x: position.x,
                    y: position.y,
                };
                if settings.widget_position.as_ref() != Some(&position) {
                    settings.widget_position = Some(position);
                    if let Ok(settings) = store.replace(settings) {
                        let _ = window
                            .app_handle()
                            .state::<QuotaService>()
                            .update_settings(settings);
                    }
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running CodexMeter");
}

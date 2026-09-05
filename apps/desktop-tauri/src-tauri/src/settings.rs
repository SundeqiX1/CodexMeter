use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use tauri::{AppHandle, Manager};

use crate::models::AppSettings;

#[derive(Clone)]
pub struct SettingsStore {
    path: PathBuf,
    value: Arc<RwLock<AppSettings>>,
}

impl SettingsStore {
    pub fn load(app: &AppHandle) -> Self {
        let path = app
            .path()
            .app_config_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("settings.json");
        let value = fs::read_to_string(&path)
            .ok()
            .and_then(|contents| serde_json::from_str::<AppSettings>(&contents).ok())
            .unwrap_or_default()
            .normalized();
        harden_permissions(&path);
        Self {
            path,
            value: Arc::new(RwLock::new(value)),
        }
    }

    pub fn get(&self) -> AppSettings {
        self.value.read().unwrap().clone()
    }

    pub fn replace(&self, value: AppSettings) -> Result<AppSettings, String> {
        let value = value.normalized();
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "Invalid settings path".to_owned())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Unable to create settings folder: {error}"))?;

        let temporary = self.path.with_extension("json.tmp");
        let encoded = serde_json::to_vec_pretty(&value)
            .map_err(|error| format!("Unable to encode settings: {error}"))?;
        let mut options = OpenOptions::new();
        options.create(true).truncate(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&temporary)
            .map_err(|error| format!("Unable to open settings: {error}"))?;
        file.write_all(&encoded)
            .and_then(|_| file.sync_all())
            .map_err(|error| format!("Unable to write settings: {error}"))?;
        #[cfg(windows)]
        if self.path.exists() {
            fs::remove_file(&self.path)
                .map_err(|error| format!("Unable to replace settings: {error}"))?;
        }
        fs::rename(&temporary, &self.path)
            .map_err(|error| format!("Unable to save settings: {error}"))?;
        harden_permissions(&self.path);
        *self.value.write().unwrap() = value.clone();
        Ok(value)
    }
}

#[cfg(unix)]
fn harden_permissions(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = fs::metadata(path) {
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        let _ = fs::set_permissions(path, permissions);
    }
}

#[cfg(not(unix))]
fn harden_permissions(_path: &std::path::Path) {}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitsEnvelope {
    #[serde(default)]
    pub rate_limits: Option<RateLimitSnapshot>,
    #[serde(default)]
    pub rate_limits_by_limit_id: Option<HashMap<String, RateLimitSnapshot>>,
    #[serde(default)]
    pub rate_limit_reset_credits: Option<RateLimitResetCreditsSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitSnapshot {
    pub primary: Option<RateLimitWindow>,
    pub secondary: Option<RateLimitWindow>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitWindow {
    pub used_percent: f64,
    pub window_duration_mins: Option<u64>,
    pub resets_at: Option<f64>,
}

impl RateLimitWindow {
    pub fn remaining_percent(&self) -> f64 {
        (100.0 - self.used_percent).clamp(0.0, 100.0)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitResetCreditsSummary {
    pub available_count: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Stale,
    Failed,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionState {
    pub status: ConnectionStatus,
    pub message: Option<String>,
    pub executable: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendState {
    pub connection: ConnectionState,
    pub snapshot: Option<RateLimitsEnvelope>,
    pub last_updated: Option<u64>,
    pub platform: String,
    pub settings: AppSettings,
}

impl Default for FrontendState {
    fn default() -> Self {
        Self {
            connection: ConnectionState::default(),
            snapshot: None,
            last_updated: None,
            platform: std::env::consts::OS.to_owned(),
            settings: AppSettings::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum AppLanguage {
    #[default]
    #[serde(rename = "system")]
    System,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "zh-CN")]
    SimplifiedChinese,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum ResolvedLanguage {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "zh-CN")]
    SimplifiedChinese,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    pub language: AppLanguage,
    pub refresh_interval_secs: u64,
    pub compact_menu_bar: bool,
    pub hide_missing_windows: bool,
    pub widget_visible: bool,
    pub widget_position: Option<SavedPosition>,
    pub codex_binary_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: AppLanguage::System,
            refresh_interval_secs: 60,
            compact_menu_bar: false,
            hide_missing_windows: false,
            widget_visible: false,
            widget_position: None,
            codex_binary_path: None,
        }
    }
}

impl AppSettings {
    pub fn normalized(mut self) -> Self {
        self.refresh_interval_secs = self.refresh_interval_secs.clamp(30, 60);
        self.codex_binary_path = self.codex_binary_path.and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.chars().take(2_048).collect())
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_current_rate_limit_shape() {
        let fixture = include_str!("../tests/fixtures/current-rate-limits.json");
        let decoded: RateLimitsEnvelope = serde_json::from_str(fixture).unwrap();

        let legacy = decoded.rate_limits.as_ref().unwrap();
        assert_eq!(legacy.primary.as_ref().unwrap().remaining_percent(), 54.0);
        assert_eq!(decoded.rate_limit_reset_credits.unwrap().available_count, 1);
    }

    #[test]
    fn decodes_keyed_only_rate_limit_shape() {
        let decoded: RateLimitsEnvelope = serde_json::from_value(serde_json::json!({
            "rateLimitsByLimitId": {
                "codex": {
                    "primary": { "usedPercent": 12, "windowDurationMins": 10080 }
                }
            }
        }))
        .unwrap();

        assert!(decoded.rate_limits.is_none());
        assert_eq!(
            decoded.rate_limits_by_limit_id.unwrap()["codex"]
                .primary
                .as_ref()
                .unwrap()
                .window_duration_mins,
            Some(10_080)
        );
    }

    #[test]
    fn remaining_percentage_is_clamped() {
        let overused = RateLimitWindow {
            used_percent: 140.0,
            window_duration_mins: None,
            resets_at: None,
        };
        let negative = RateLimitWindow {
            used_percent: -10.0,
            window_duration_mins: None,
            resets_at: None,
        };

        assert_eq!(overused.remaining_percent(), 0.0);
        assert_eq!(negative.remaining_percent(), 100.0);
    }

    #[test]
    fn old_settings_without_language_follow_the_system() {
        let settings: AppSettings = serde_json::from_value(serde_json::json!({
            "refreshIntervalSecs": 30,
            "compactMenuBar": true
        }))
        .unwrap();

        assert_eq!(settings.language, AppLanguage::System);
        assert_eq!(settings.refresh_interval_secs, 30);
    }
}

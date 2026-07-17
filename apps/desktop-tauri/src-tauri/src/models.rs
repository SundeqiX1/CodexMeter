use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitsEnvelope {
    pub rate_limits: RateLimitSnapshot,
    pub rate_limits_by_limit_id: Option<HashMap<String, RateLimitSnapshot>>,
    pub rate_limit_reset_credits: Option<RateLimitResetCreditsSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitSnapshot {
    pub limit_id: Option<String>,
    pub limit_name: Option<String>,
    pub primary: Option<RateLimitWindow>,
    pub secondary: Option<RateLimitWindow>,
    pub credits: Option<CreditsSnapshot>,
    pub individual_limit: Option<SpendControlLimitSnapshot>,
    pub spend_control_reached: Option<bool>,
    pub plan_type: Option<String>,
    pub rate_limit_reached_type: Option<String>,
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
pub struct CreditsSnapshot {
    pub has_credits: bool,
    pub unlimited: bool,
    pub balance: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpendControlLimitSnapshot {
    pub limit: String,
    pub used: String,
    pub remaining_percent: f64,
    pub resets_at: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitResetCreditsSummary {
    pub available_count: u64,
    pub credits: Option<Vec<RateLimitResetCredit>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitResetCredit {
    pub id: String,
    pub reset_type: String,
    pub status: String,
    pub granted_at: f64,
    pub expires_at: Option<f64>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
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
}

impl Default for FrontendState {
    fn default() -> Self {
        Self {
            connection: ConnectionState::default(),
            snapshot: None,
            last_updated: None,
            platform: std::env::consts::OS.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_current_rate_limit_shape() {
        let fixture = include_str!("../tests/fixtures/current-rate-limits.json");
        let decoded: RateLimitsEnvelope = serde_json::from_str(fixture).unwrap();

        assert_eq!(decoded.rate_limits.limit_id.as_deref(), Some("codex"));
        assert_eq!(decoded.rate_limits.plan_type.as_deref(), Some("prolite"));
        assert_eq!(
            decoded
                .rate_limits
                .primary
                .as_ref()
                .unwrap()
                .remaining_percent(),
            54.0
        );
        assert_eq!(decoded.rate_limit_reset_credits.unwrap().available_count, 1);
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
}

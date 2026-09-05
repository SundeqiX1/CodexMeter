use std::{
    process::Stdio,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{Child, ChildStdin, Command},
    sync::mpsc,
    time::{self, Instant},
};

use crate::{
    launcher::{locate_codex, LaunchSpec},
    models::{
        AppSettings, ConnectionStatus, FrontendState, RateLimitSnapshot, RateLimitWindow,
        RateLimitsEnvelope, ResolvedLanguage,
    },
    resolved_language,
    settings::SettingsStore,
};

const INITIALIZE_REQUEST_ID: i64 = 1;
const INITIALIZE_TIMEOUT: Duration = Duration::from_secs(20);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(25);

#[derive(Clone)]
pub struct QuotaService {
    commands: mpsc::UnboundedSender<ServiceCommand>,
    state: Arc<RwLock<FrontendState>>,
}

impl QuotaService {
    pub fn start(app: AppHandle, settings: SettingsStore) -> Self {
        let (commands, command_rx) = mpsc::unbounded_channel();
        let mut initial_state = FrontendState::default();
        initial_state.settings = settings.get();
        let state = Arc::new(RwLock::new(initial_state));
        let worker = Worker::new(app, Arc::clone(&state), command_rx);
        tauri::async_runtime::spawn(worker.run());
        Self { commands, state }
    }

    pub fn state(&self) -> FrontendState {
        self.state.read().unwrap().clone()
    }

    pub fn connect(&self) -> Result<(), String> {
        self.send(ServiceCommand::Connect)
    }

    pub fn refresh(&self) -> Result<(), String> {
        self.send(ServiceCommand::Refresh)
    }

    pub fn reconnect(&self) -> Result<(), String> {
        self.send(ServiceCommand::Reconnect)
    }

    pub fn disconnect(&self) -> Result<(), String> {
        self.send(ServiceCommand::Disconnect)
    }

    pub fn update_settings(&self, settings: AppSettings) -> Result<(), String> {
        self.send(ServiceCommand::UpdateSettings(settings))
    }

    pub fn refresh_ui(&self, app: &AppHandle) {
        update_tray(app, &self.state());
    }

    fn send(&self, command: ServiceCommand) -> Result<(), String> {
        self.commands
            .send(command)
            .map_err(|_| "额度后台服务已经停止".to_owned())
    }
}

enum ServiceCommand {
    Connect,
    Refresh,
    Reconnect,
    Disconnect,
    UpdateSettings(AppSettings),
}

enum ProcessEvent {
    Line(u64, String),
    Stderr(u64, String),
    Eof(u64),
}

struct Session {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    generation: u64,
    executable: String,
    initialized: bool,
    initialize_deadline: Instant,
    pending_request: Option<(i64, Instant)>,
}

impl Session {
    async fn send(&mut self, message: Value) -> Result<(), String> {
        let mut encoded = serde_json::to_vec(&message).map_err(|error| error.to_string())?;
        encoded.push(b'\n');
        self.stdin
            .write_all(&encoded)
            .await
            .map_err(|error| error.to_string())?;
        self.stdin.flush().await.map_err(|error| error.to_string())
    }

    async fn request_rate_limits(&mut self, request_id: i64) -> Result<(), String> {
        self.send(json!({
            "method": "account/rateLimits/read",
            "id": request_id
        }))
        .await?;
        self.pending_request = Some((request_id, Instant::now() + REQUEST_TIMEOUT));
        Ok(())
    }
}

struct Worker {
    app: AppHandle,
    state: Arc<RwLock<FrontendState>>,
    command_rx: mpsc::UnboundedReceiver<ServiceCommand>,
    process_tx: mpsc::UnboundedSender<ProcessEvent>,
    process_rx: mpsc::UnboundedReceiver<ProcessEvent>,
    session: Option<Session>,
    generation: u64,
    next_request_id: i64,
    error_tail: String,
}

impl Worker {
    fn new(
        app: AppHandle,
        state: Arc<RwLock<FrontendState>>,
        command_rx: mpsc::UnboundedReceiver<ServiceCommand>,
    ) -> Self {
        let (process_tx, process_rx) = mpsc::unbounded_channel();
        Self {
            app,
            state,
            command_rx,
            process_tx,
            process_rx,
            session: None,
            generation: 0,
            next_request_id: 10,
            error_tail: String::new(),
        }
    }

    async fn run(mut self) {
        let mut maintenance = time::interval(Duration::from_secs(1));
        maintenance.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
        let mut next_auto_refresh = Instant::now() + self.refresh_interval();

        loop {
            tokio::select! {
                command = self.command_rx.recv() => {
                    let Some(command) = command else {
                        self.stop_session().await;
                        return;
                    };
                    match command {
                        ServiceCommand::Connect => {
                            if self.session.is_none() {
                                self.connect().await;
                            }
                        }
                        ServiceCommand::Refresh => self.refresh().await,
                        ServiceCommand::Reconnect => {
                            self.stop_session().await;
                            self.connect().await;
                        }
                        ServiceCommand::Disconnect => {
                            self.stop_session().await;
                            self.set_connection(ConnectionStatus::Disconnected, None, None);
                        }
                        ServiceCommand::UpdateSettings(settings) => {
                            let previous_binary = self
                                .state
                                .read()
                                .unwrap()
                                .settings
                                .codex_binary_path
                                .clone();
                            let reconnect = previous_binary != settings.codex_binary_path;
                            self.update_state(|state| state.settings = settings);
                            next_auto_refresh = Instant::now() + self.refresh_interval();
                            if reconnect {
                                self.stop_session().await;
                                self.connect().await;
                            }
                        }
                    }
                }
                event = self.process_rx.recv() => {
                    if let Some(event) = event {
                        self.handle_process_event(event).await;
                    }
                }
                _ = maintenance.tick() => {
                    self.check_timeouts().await;
                    if Instant::now() >= next_auto_refresh {
                        self.refresh().await;
                        next_auto_refresh = Instant::now() + self.refresh_interval();
                    }
                }
            }
        }
    }

    async fn connect(&mut self) {
        let configured_path = self
            .state
            .read()
            .unwrap()
            .settings
            .codex_binary_path
            .clone();
        let Some(spec) = locate_codex(configured_path.as_deref()) else {
            self.set_connection(
                ConnectionStatus::Failed,
                Some(self.text(
                    "Codex was not found. Install Codex CLI or configure its path in Settings.",
                    "找不到 Codex。请安装 Codex CLI，或在设置中指定程序路径。",
                )),
                None,
            );
            return;
        };

        self.set_connection(
            ConnectionStatus::Connecting,
            None,
            Some(spec.display.clone()),
        );
        match self.spawn_session(spec).await {
            Ok(session) => self.session = Some(session),
            Err(message) => self.set_connection(ConnectionStatus::Failed, Some(message), None),
        }
    }

    async fn spawn_session(&mut self, spec: LaunchSpec) -> Result<Session, String> {
        self.generation += 1;
        let generation = self.generation;
        self.error_tail.clear();

        let mut command = Command::new(&spec.program);
        command
            .args(&spec.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.as_std_mut().creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = command.spawn().map_err(|error| {
            self.with_detail(
                "Unable to start Codex App Server",
                "无法启动 Codex App Server",
                &error,
            )
        })?;
        let stdin = child.stdin.take().ok_or_else(|| {
            self.text(
                "Codex input channel is unavailable.",
                "Codex 输入通道不可用。",
            )
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            self.text(
                "Codex output channel is unavailable.",
                "Codex 输出通道不可用。",
            )
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            self.text(
                "Codex error channel is unavailable.",
                "Codex 错误通道不可用。",
            )
        })?;

        let stdout_tx = self.process_tx.clone();
        tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if stdout_tx
                    .send(ProcessEvent::Line(generation, line))
                    .is_err()
                {
                    return;
                }
            }
            let _ = stdout_tx.send(ProcessEvent::Eof(generation));
        });

        let stderr_tx = self.process_tx.clone();
        tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if stderr_tx
                    .send(ProcessEvent::Stderr(generation, line))
                    .is_err()
                {
                    return;
                }
            }
        });

        let mut session = Session {
            child,
            stdin: BufWriter::new(stdin),
            generation,
            executable: spec.display,
            initialized: false,
            initialize_deadline: Instant::now() + INITIALIZE_TIMEOUT,
            pending_request: None,
        };
        session
            .send(json!({
                "method": "initialize",
                "id": INITIALIZE_REQUEST_ID,
                "params": {
                    "clientInfo": {
                        "name": "codex_meter",
                        "title": "CodexMeter",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            }))
            .await
            .map_err(|error| {
                self.with_detail(
                    "Unable to initialize Codex App Server",
                    "无法初始化 Codex App Server",
                    &error,
                )
            })?;

        Ok(session)
    }

    async fn refresh(&mut self) {
        let Some(session) = self.session.as_mut() else {
            self.connect().await;
            return;
        };
        if !session.initialized || session.pending_request.is_some() {
            return;
        }

        let request_id = self.next_request_id;
        self.next_request_id += 1;
        if let Err(error) = session.request_rate_limits(request_id).await {
            let message = self.with_detail(
                "Unable to send the quota request",
                "额度查询发送失败",
                &error,
            );
            self.fail(message).await;
        }
    }

    async fn handle_process_event(&mut self, event: ProcessEvent) {
        match event {
            ProcessEvent::Line(generation, line) => {
                if self.session.as_ref().map(|session| session.generation) == Some(generation) {
                    self.handle_line(&line).await;
                }
            }
            ProcessEvent::Stderr(generation, line) => {
                if self.session.as_ref().map(|session| session.generation) == Some(generation)
                    && !line.contains("WARNING: proceeding")
                    && !line.trim().is_empty()
                {
                    if !self.error_tail.is_empty() {
                        self.error_tail.push('\n');
                    }
                    self.error_tail.push_str(&line);
                    if self.error_tail.len() > 2_000 {
                        let from = self.error_tail.len() - 2_000;
                        self.error_tail = self.error_tail[from..].to_owned();
                    }
                }
            }
            ProcessEvent::Eof(generation) => {
                if self.session.as_ref().map(|session| session.generation) == Some(generation) {
                    let status_code = self
                        .session
                        .as_mut()
                        .and_then(|session| session.child.try_wait().ok().flatten())
                        .and_then(|status| status.code());
                    let detail =
                        self.error_tail
                            .lines()
                            .rev()
                            .find(|line| !line.trim().is_empty())
                            .map(str::to_owned)
                            .unwrap_or_else(|| match status_code {
                                Some(code) => self.text(
                                    &format!("Codex App Server exited (status {code})."),
                                    &format!("Codex App Server 已退出（状态码 {code}）。"),
                                ),
                                None => self
                                    .text("Codex App Server exited.", "Codex App Server 已退出。"),
                            });
                    self.fail(detail).await;
                }
            }
        }
    }

    async fn handle_line(&mut self, line: &str) {
        let Ok(message) = serde_json::from_str::<Value>(line) else {
            return;
        };

        if message.get("method").and_then(Value::as_str) == Some("account/rateLimits/updated") {
            self.refresh().await;
            return;
        }

        let response_id = message.get("id").and_then(Value::as_i64);
        if response_id == Some(INITIALIZE_REQUEST_ID) && message.get("result").is_some() {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            if let Err(error) = session
                .send(json!({ "method": "initialized", "params": {} }))
                .await
            {
                let message = self.with_detail(
                    "Codex App Server handshake failed",
                    "Codex App Server 握手失败",
                    &error,
                );
                self.fail(message).await;
                return;
            }
            session.initialized = true;
            let executable = session.executable.clone();
            self.set_connection(ConnectionStatus::Connected, None, Some(executable));
            self.refresh().await;
            return;
        }

        if let Some(error) = message.get("error") {
            let detail = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_else(|| {
                    if resolved_language(&self.app) == ResolvedLanguage::SimplifiedChinese {
                        "未知 App Server 错误"
                    } else {
                        "Unknown App Server error"
                    }
                })
                .to_owned();
            if let Some(session) = self.session.as_mut() {
                session.pending_request = None;
            }
            self.set_connection(ConnectionStatus::Failed, Some(detail), None);
            return;
        }

        let Some(result) = message.get("result") else {
            return;
        };
        if result.get("rateLimits").is_none() && result.get("rateLimitsByLimitId").is_none() {
            return;
        }

        match serde_json::from_value::<RateLimitsEnvelope>(result.clone()) {
            Ok(snapshot) => {
                if let Some(session) = self.session.as_mut() {
                    session.pending_request = None;
                    let executable = session.executable.clone();
                    self.update_state(|state| {
                        state.snapshot = Some(snapshot);
                        state.last_updated = Some(now_millis());
                        state.connection.status = ConnectionStatus::Connected;
                        state.connection.message = None;
                        state.connection.executable = Some(executable);
                    });
                }
            }
            Err(error) => {
                self.set_connection(
                    ConnectionStatus::Failed,
                    Some(self.with_detail(
                        "Unable to parse quota data",
                        "无法解析额度数据",
                        &error,
                    )),
                    None,
                );
            }
        }
    }

    async fn check_timeouts(&mut self) {
        let Some(session) = self.session.as_ref() else {
            return;
        };
        let now = Instant::now();
        if !session.initialized && now >= session.initialize_deadline {
            let message = self.text(
                "Codex connection timed out. Confirm that Codex is signed in.",
                "连接 Codex 超时。请确认已经登录 Codex。",
            );
            self.fail(message).await;
            return;
        }
        if session
            .pending_request
            .is_some_and(|(_, deadline)| now >= deadline)
        {
            if let Some(session) = self.session.as_mut() {
                session.pending_request = None;
            }
            self.set_connection(
                ConnectionStatus::Failed,
                Some(self.text(
                    "The quota request timed out. CodexMeter will retry automatically.",
                    "额度查询超时，稍后会自动重试。",
                )),
                None,
            );
        }
    }

    async fn fail(&mut self, message: String) {
        let executable = self
            .session
            .as_ref()
            .map(|session| session.executable.clone());
        self.stop_session().await;
        self.set_connection(ConnectionStatus::Failed, Some(message), executable);
    }

    async fn stop_session(&mut self) {
        if let Some(mut session) = self.session.take() {
            let _ = session.child.kill().await;
            let _ = session.child.wait().await;
        }
    }

    fn set_connection(
        &self,
        status: ConnectionStatus,
        message: Option<String>,
        executable: Option<String>,
    ) {
        self.update_state(|state| {
            state.connection.status = effective_status(status, state.snapshot.is_some());
            state.connection.message = message;
            if executable.is_some() || state.connection.executable.is_none() {
                state.connection.executable = executable;
            }
        });
    }

    fn update_state(&self, mutate: impl FnOnce(&mut FrontendState)) {
        let snapshot = {
            let mut state = self.state.write().unwrap();
            mutate(&mut state);
            state.clone()
        };
        let _ = self.app.emit("quota://updated", snapshot.clone());
        update_tray(&self.app, &snapshot);
    }

    fn refresh_interval(&self) -> Duration {
        Duration::from_secs(
            self.state
                .read()
                .unwrap()
                .settings
                .refresh_interval_secs
                .clamp(30, 60),
        )
    }

    fn text(&self, english: &str, chinese: &str) -> String {
        if resolved_language(&self.app) == ResolvedLanguage::SimplifiedChinese {
            chinese.to_owned()
        } else {
            english.to_owned()
        }
    }

    fn with_detail(&self, english: &str, chinese: &str, detail: &impl std::fmt::Display) -> String {
        if resolved_language(&self.app) == ResolvedLanguage::SimplifiedChinese {
            format!("{chinese}：{detail}")
        } else {
            format!("{english}: {detail}")
        }
    }
}

fn effective_status(status: ConnectionStatus, has_snapshot: bool) -> ConnectionStatus {
    if status == ConnectionStatus::Failed && has_snapshot {
        ConnectionStatus::Stale
    } else {
        status
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn update_tray(app: &AppHandle, state: &FrontendState) {
    let Some(tray) = app.tray_by_id("codexmeter-tray") else {
        return;
    };
    let limit = state.snapshot.as_ref().and_then(codex_limit);
    let five_hour = limit.and_then(|value| window_by_duration(value, 300));
    let weekly = limit.and_then(|value| window_by_duration(value, 10_080));
    let title = tray_title(
        five_hour,
        weekly,
        state.settings.compact_menu_bar,
        state.settings.hide_missing_windows,
    );
    let chinese = resolved_language(app) == ResolvedLanguage::SimplifiedChinese;
    let status = match (chinese, &state.connection.status) {
        (true, ConnectionStatus::Connected) => "实时",
        (true, ConnectionStatus::Connecting) => "连接中",
        (true, ConnectionStatus::Stale) => "数据已过期",
        (true, ConnectionStatus::Disconnected) => "未连接",
        (true, ConnectionStatus::Failed) => "暂不可用",
        (false, ConnectionStatus::Connected) => "Live",
        (false, ConnectionStatus::Connecting) => "Connecting",
        (false, ConnectionStatus::Stale) => "Stale",
        (false, ConnectionStatus::Disconnected) => "Disconnected",
        (false, ConnectionStatus::Failed) => "Unavailable",
    };
    let tooltip = format!("CodexMeter — {status} — {title}");
    let _ = tray.set_tooltip(Some(&tooltip));

    #[cfg(target_os = "macos")]
    let _ = tray.set_title(Some(&title));
}

fn codex_limit(snapshot: &RateLimitsEnvelope) -> Option<&RateLimitSnapshot> {
    snapshot
        .rate_limits_by_limit_id
        .as_ref()
        .and_then(|limits| limits.get("codex"))
        .or(snapshot.rate_limits.as_ref())
}

fn window_by_duration(limit: &RateLimitSnapshot, minutes: u64) -> Option<&RateLimitWindow> {
    [limit.primary.as_ref(), limit.secondary.as_ref()]
        .into_iter()
        .flatten()
        .find(|window| window.window_duration_mins == Some(minutes))
}

fn tray_title(
    five_hour: Option<&RateLimitWindow>,
    weekly: Option<&RateLimitWindow>,
    compact: bool,
    hide_missing: bool,
) -> String {
    let percent = |window: Option<&RateLimitWindow>| {
        window
            .map(|value| format!("{:.0}%", value.remaining_percent()))
            .unwrap_or_else(|| "--".to_owned())
    };
    let mut parts = Vec::new();
    if five_hour.is_some() || !hide_missing {
        parts.push(if compact {
            percent(five_hour)
        } else {
            format!("5h {}", percent(five_hour))
        });
    }
    if weekly.is_some() || !hide_missing {
        parts.push(if compact {
            percent(weekly)
        } else {
            format!("W {}", percent(weekly))
        });
    }
    if parts.is_empty() {
        "CodexMeter".to_owned()
    } else {
        parts.join(" · ")
    }
}

#[cfg(test)]
mod tray_tests {
    use super::*;

    fn window(used: f64, minutes: u64) -> RateLimitWindow {
        RateLimitWindow {
            used_percent: used,
            window_duration_mins: Some(minutes),
            resets_at: None,
        }
    }

    #[test]
    fn labels_windows_by_duration_and_never_invents_missing_data() {
        let weekly = window(57.0, 10_080);
        assert_eq!(
            tray_title(None, Some(&weekly), false, false),
            "5h -- · W 43%"
        );
        assert_eq!(tray_title(None, Some(&weekly), false, true), "W 43%");
    }

    #[test]
    fn pro_style_weekly_only_payload_does_not_create_a_five_hour_window() {
        let envelope: RateLimitsEnvelope = serde_json::from_value(serde_json::json!({
            "rateLimitsByLimitId": {
                "codex": {
                    "primary": {
                        "usedPercent": 57,
                        "windowDurationMins": 10080,
                        "resetsAt": 1_800_000_000
                    }
                }
            }
        }))
        .unwrap();
        let limit = codex_limit(&envelope).unwrap();

        assert!(window_by_duration(limit, 300).is_none());
        assert!(window_by_duration(limit, 10_080).is_some());
        assert_eq!(
            tray_title(None, window_by_duration(limit, 10_080), false, false),
            "5h -- · W 43%"
        );
    }

    #[test]
    fn compact_title_omits_window_names_only() {
        let five = window(24.0, 300);
        let weekly = window(57.0, 10_080);
        assert_eq!(
            tray_title(Some(&five), Some(&weekly), true, false),
            "76% · 43%"
        );
    }

    #[test]
    fn failed_connection_is_stale_only_when_a_valid_snapshot_exists() {
        assert_eq!(
            effective_status(ConnectionStatus::Failed, true),
            ConnectionStatus::Stale
        );
        assert_eq!(
            effective_status(ConnectionStatus::Failed, false),
            ConnectionStatus::Failed
        );
    }
}

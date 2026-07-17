import { getCurrentWindow } from "@tauri-apps/api/window";
import { useMemo } from "react";
import { Gauge } from "./components/Gauge";
import { CloseIcon, LinkIcon, RefreshIcon } from "./components/Icons";
import { WindowRow } from "./components/WindowRow";
import { useQuota } from "./hooks/useQuota";
import {
  criticalWindow,
  displayName,
  orderedLimits,
  planName,
  remainingPercent,
  resetLabel,
  windowLabel,
  windowsFor,
} from "./lib/quota";

const STATUS_LABELS = {
  disconnected: "未连接",
  connecting: "连接中",
  connected: "实时",
  failed: "连接异常",
} as const;

function App() {
  const {
    state,
    launchAtLogin,
    settingsMessage,
    refresh,
    reconnect,
    setLaunchAtLoginEnabled,
  } = useQuota();

  const limits = useMemo(() => orderedLimits(state.snapshot), [state.snapshot]);
  const primaryLimit = limits[0] ?? null;
  const windows = useMemo(() => windowsFor(primaryLimit), [primaryLimit]);
  const critical = useMemo(() => criticalWindow(state.snapshot), [state.snapshot]);
  const mainRemaining = critical ? remainingPercent(critical) : null;
  const updatedLabel = state.lastUpdated
    ? new Intl.DateTimeFormat("zh-CN", { hour: "2-digit", minute: "2-digit" }).format(
        new Date(state.lastUpdated),
      )
    : "尚未同步";

  const closeWindow = async () => {
    await getCurrentWindow().hide();
  };

  return (
    <main className="shell">
      <div className="surface">
        <header className="titlebar" data-tauri-drag-region>
          <div className="brand" data-tauri-drag-region>
            <span className="brand-mark" aria-hidden="true">
              <i />
              <i />
              <i />
            </span>
            <div data-tauri-drag-region>
              <strong>CODEX / QUOTA</strong>
              <span>{state.platform === "windows" ? "WINDOWS DESKTOP" : "MAC DESKTOP"}</span>
            </div>
          </div>
          <div className="titlebar-actions">
            <span className={`status status--${state.connection.status}`} aria-live="polite">
              <i />
              {STATUS_LABELS[state.connection.status]}
            </span>
            <button className="icon-button" onClick={() => void closeWindow()} aria-label="隐藏窗口">
              <CloseIcon />
            </button>
          </div>
        </header>

        <div className="rule" />

        <Gauge
          percent={mainRemaining}
          label={critical ? windowLabel(critical.windowDurationMins) : "等待额度数据"}
          detail={critical ? `${resetLabel(critical.resetsAt)}重置` : "连接本机 Codex 后自动同步"}
        />

        {state.connection.status === "failed" ? (
          <section className="notice notice--error">
            <span>CONNECTION REPORT</span>
            <p>{state.connection.message || "Codex App Server 连接失败。"}</p>
            <button onClick={() => void reconnect()}>
              <LinkIcon />
              重新连接
            </button>
          </section>
        ) : null}

        <section className="limits-panel">
          <div className="section-heading">
            <div>
              <span className="eyebrow">LIMIT WINDOWS</span>
              <h3>{primaryLimit ? displayName(primaryLimit) : "额度窗口"}</h3>
            </div>
            {planName(primaryLimit?.planType) ? <span className="plan-badge">{planName(primaryLimit?.planType)}</span> : null}
          </div>

          <div className="window-list">
            {windows.length > 0 ? (
              windows.map((window, index) => (
                <WindowRow
                  key={`${window.windowDurationMins ?? "unknown"}-${window.resetsAt ?? 0}-${index}`}
                  window={window}
                />
              ))
            ) : (
              <div className="empty-row">
                <span className="scanner" />
                正在等待服务器快照
              </div>
            )}
          </div>
        </section>

        <section className="metrics">
          <div>
            <span>积分余额</span>
            <strong>
              {primaryLimit?.credits?.unlimited ? "无限" : primaryLimit?.credits?.balance ?? "—"}
            </strong>
          </div>
          <div>
            <span>重置券</span>
            <strong>{state.snapshot?.rateLimitResetCredits?.availableCount ?? "—"}</strong>
          </div>
          <div>
            <span>同步时间</span>
            <strong className="metrics__time">{updatedLabel}</strong>
          </div>
        </section>

        <footer>
          <label className="switch-row">
            <span>
              <strong>开机启动</strong>
              <small>登录系统后保持额度可见</small>
            </span>
            <input
              type="checkbox"
              checked={launchAtLogin}
              onChange={(event) => void setLaunchAtLoginEnabled(event.target.checked)}
            />
            <i aria-hidden="true" />
          </label>

          {settingsMessage ? <p className="settings-message">{settingsMessage}</p> : null}

          <div className="footer-actions">
            <span title={state.connection.executable ?? undefined}>
              {state.connection.executable ? "LOCAL SESSION" : "NO SESSION"}
            </span>
            <button
              className="refresh-button"
              onClick={() => void refresh()}
              disabled={state.connection.status === "connecting"}
            >
              <RefreshIcon />
              刷新额度
            </button>
          </div>
        </footer>
      </div>
    </main>
  );
}

export default App;

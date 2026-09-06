import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { useEffect, useMemo, useState } from "react";
import { useQuota } from "./hooks/useQuota";
import { resolveLanguage, translations } from "./i18n";
import type { Translation } from "./i18n";
import { remainingPercent, resetLabel, subscriptionLabel, windowForDuration } from "./lib/quota";
import type { AppSettings, RateLimitWindow, ResolvedLanguage } from "./types";

const currentWindow = getCurrentWindow();

function percent(window: RateLimitWindow | null): string {
  return window ? `${Math.round(remainingPercent(window))}%` : "--";
}

function UsageRow({
  label,
  window,
  language,
  copy,
}: {
  label: string;
  window: RateLimitWindow | null;
  language: ResolvedLanguage;
  copy: Translation;
}) {
  const remaining = window ? remainingPercent(window) : null;
  return (
    <section className="usage-row">
      <div className="usage-row__top">
        <span>{label}</span>
        <strong>{remaining == null ? "--" : `${Math.round(remaining)}%`}</strong>
      </div>
      <div className="meter" aria-label={remaining == null ? `${label} ${copy.statusFailed}` : `${remaining}% ${copy.remaining}`}>
        <i style={{ width: `${remaining ?? 0}%` }} />
      </div>
      <small>{window ? `${copy.reset} ${resetLabel(window.resetsAt, language, copy.unknown)}` : copy.notReported}</small>
    </section>
  );
}

function Widget() {
  const { state, setWidgetVisible } = useQuota();
  const [expanded, setExpanded] = useState(false);
  const language = resolveLanguage(state.settings.language);
  const copy = translations[language];
  const fiveHour = useMemo(() => windowForDuration(state.snapshot, 300), [state.snapshot]);
  const weekly = useMemo(() => windowForDuration(state.snapshot, 10_080), [state.snapshot]);
  const hideMissing = state.settings.hideMissingWindows;
  const showFiveHour = Boolean(fiveHour) || !hideMissing;
  const showWeekly = Boolean(weekly) || !hideMissing;
  const visibleWindowCount = Number(showFiveHour) + Number(showWeekly);
  const detailsOpen = visibleWindowCount > 0 && expanded;
  const widgetWidth = visibleWindowCount > 1 ? 224 : 164;

  useEffect(() => {
    void currentWindow.setSize(new LogicalSize(widgetWidth, detailsOpen ? 118 : 48));
  }, [detailsOpen, widgetWidth]);

  return (
    <main
      className={`widget ${detailsOpen ? "widget--expanded" : ""}`}
      onClick={(event) => {
        if ((event.target as HTMLElement).closest("button")) return;
        event.stopPropagation();
        if (visibleWindowCount > 0) setExpanded((current) => !current);
      }}
      onContextMenu={(event) => event.preventDefault()}
    >
      <div className="widget__card">
        <div className="widget__summary">
          <i className={`status-dot status-dot--${state.connection.status}`} data-tauri-drag-region />
          {showFiveHour ? <strong data-tauri-drag-region>5h {percent(fiveHour)}</strong> : null}
          {showWeekly ? <span data-tauri-drag-region>W {percent(weekly)}</span> : null}
          {visibleWindowCount === 0 ? <strong data-tauri-drag-region>{copy.noData}</strong> : null}
          <button
            aria-label={copy.closeWidget}
            onClick={(event) => {
              event.stopPropagation();
              setExpanded(false);
              void setWidgetVisible(false);
            }}
          >×</button>
        </div>
        <div
          className={`widget__details ${visibleWindowCount === 1 ? "widget__details--single" : ""}`}
          aria-hidden={!detailsOpen}
          data-tauri-drag-region
        >
          {showFiveHour ? <span>5h <b>{percent(fiveHour)}</b><small>{fiveHour ? `${copy.reset} ${resetLabel(fiveHour.resetsAt, language, copy.unknown)}` : copy.noData}</small></span> : null}
          {showWeekly ? <span>{copy.weekly} <b>{percent(weekly)}</b><small>{weekly ? `${copy.reset} ${resetLabel(weekly.resetsAt, language, copy.unknown)}` : copy.noData}</small></span> : null}
        </div>
      </div>
    </main>
  );
}

function SettingsView({
  value,
  message,
  onCancel,
  onSave,
  copy,
}: {
  value: AppSettings;
  message: string | null;
  onCancel: () => void;
  onSave: (settings: AppSettings) => void;
  copy: Translation;
}) {
  const [draft, setDraft] = useState(value);
  useEffect(() => setDraft(value), [value]);

  return (
    <div className="settings-view">
      <label>
        {copy.language}
        <select
          value={draft.language}
          onChange={(event) => setDraft({ ...draft, language: event.target.value as AppSettings["language"] })}
        >
          <option value="system">{copy.followSystem}</option>
          <option value="en">{copy.english}</option>
          <option value="zh-CN">{copy.simplifiedChinese}</option>
        </select>
      </label>
      <label>
        {copy.refreshInterval}
        <select
          value={draft.refreshIntervalSecs}
          onChange={(event) => setDraft({ ...draft, refreshIntervalSecs: Number(event.target.value) })}
        >
          <option value={30}>{copy.seconds30}</option>
          <option value={60}>{copy.seconds60}</option>
        </select>
      </label>
      <label>
        {copy.codexBinaryPath}
        <input
          value={draft.codexBinaryPath ?? ""}
          onChange={(event) => setDraft({ ...draft, codexBinaryPath: event.target.value || null })}
          placeholder={copy.autoDetect}
          spellCheck={false}
        />
      </label>
      <label className="check-row">
        <input
          type="checkbox"
          checked={draft.compactMenuBar}
          onChange={(event) => setDraft({ ...draft, compactMenuBar: event.target.checked })}
        />
        {copy.compactMenuBar}
      </label>
      <label className="check-row">
        <input
          type="checkbox"
          checked={draft.hideMissingWindows}
          onChange={(event) => setDraft({ ...draft, hideMissingWindows: event.target.checked })}
        />
        {copy.hideMissingWindows}
      </label>
      <label className="check-row">
        <input
          type="checkbox"
          checked={draft.widgetVisible}
          onChange={(event) => setDraft({ ...draft, widgetVisible: event.target.checked })}
        />
        {copy.showFloatingWidget}
      </label>
      <p className="privacy-note">{copy.privacyNote}</p>
      {message ? <p className="settings-message">{message}</p> : null}
      <div className="settings-actions">
        <button className="button button--quiet" onClick={onCancel}>{copy.cancel}</button>
        <button className="button button--primary" onClick={() => onSave(draft)}>{copy.save}</button>
      </div>
    </div>
  );
}

function Panel() {
  const {
    state,
    launchAtLogin,
    settingsMessage,
    refresh,
    reconnect,
    saveSettings,
    setWidgetVisible,
    setLaunchAtLoginEnabled,
  } = useQuota();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const language = resolveLanguage(state.settings.language);
  const copy = translations[language];
  const fiveHour = useMemo(() => windowForDuration(state.snapshot, 300), [state.snapshot]);
  const weekly = useMemo(() => windowForDuration(state.snapshot, 10_080), [state.snapshot]);
  const subscription = useMemo(() => subscriptionLabel(state.snapshot), [state.snapshot]);
  const hideMissing = state.settings.hideMissingWindows;

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen("ui://open-settings", () => setSettingsOpen(true)).then((dispose) => {
      unlisten = dispose;
    });
    return () => unlisten?.();
  }, []);

  const statusLabels = {
    disconnected: copy.statusDisconnected,
    connecting: copy.statusConnecting,
    connected: copy.statusConnected,
    stale: copy.statusStale,
    failed: copy.statusFailed,
  };
  const updated = state.lastUpdated
    ? new Intl.DateTimeFormat(language, { hour: "2-digit", minute: "2-digit", second: "2-digit" }).format(
        new Date(state.lastUpdated),
      )
    : copy.never;

  const close = () => void currentWindow.hide();

  return (
    <main className="panel-shell">
      <div className="panel">
        <header className="panel__header">
          <div data-tauri-drag-region>
            <span className="panel__brand">CODEXMETER</span>
            <div className="panel__title-row" data-tauri-drag-region>
              <h1>{copy.codexUsage}</h1>
              {subscription ? <span className="plan-badge">{subscription}</span> : null}
            </div>
          </div>
          <button aria-label={copy.closePanel} onClick={close}>×</button>
        </header>

        {settingsOpen ? (
          <SettingsView
            value={state.settings}
            message={settingsMessage}
            copy={copy}
            onCancel={() => setSettingsOpen(false)}
            onSave={async (settings) => {
              if (await saveSettings(settings)) setSettingsOpen(false);
            }}
          />
        ) : (
          <>
            <div className="connection-row">
              <span><i className={`status-dot status-dot--${state.connection.status}`} />{statusLabels[state.connection.status]}</span>
              <small>{copy.lastUpdated} {updated}</small>
            </div>

            <div className="usage-list">
              {fiveHour || !hideMissing ? <UsageRow label={copy.fiveHour} window={fiveHour} language={language} copy={copy} /> : null}
              {weekly || !hideMissing ? <UsageRow label={copy.weekly} window={weekly} language={language} copy={copy} /> : null}
              {!fiveHour && !weekly && hideMissing ? <p className="empty">{copy.noRecognizedWindows}</p> : null}
            </div>

            {state.connection.message ? <p className="connection-message">{state.connection.message}</p> : null}

            <div className="primary-actions">
              <button className="button button--primary" onClick={() => void refresh()}>{copy.refresh}</button>
              {(state.connection.status === "failed" || state.connection.status === "stale") ? (
                <button className="button button--quiet" onClick={() => void reconnect()}>{copy.reconnect}</button>
              ) : null}
            </div>

            <div className="menu-list">
              <button onClick={() => void setWidgetVisible(!state.settings.widgetVisible)}>
                <span>{copy.floatingWidget}</span><b>{state.settings.widgetVisible ? copy.on : copy.off}</b>
              </button>
              <button onClick={() => void setLaunchAtLoginEnabled(!launchAtLogin)}>
                <span>{state.platform === "windows" ? copy.launchAtStartup : copy.launchAtLogin}</span><b>{launchAtLogin ? copy.on : copy.off}</b>
              </button>
              <button onClick={() => setSettingsOpen(true)}><span>{copy.settings}</span><b>›</b></button>
              <button onClick={() => void invoke("quit_codexmeter")}>
                <span>{copy.quit}</span><b>×</b>
              </button>
            </div>

            <details>
              <summary>{copy.details}</summary>
              <div className="details-grid">
                <span>{copy.subscription}</span><b>{subscription ?? "--"}</b>
                <span>{copy.resetCredits}</span><b>{state.snapshot?.rateLimitResetCredits?.availableCount ?? "--"}</b>
                <span>{copy.dataSource}</span><b>{copy.localAppServer}</b>
              </div>
            </details>
          </>
        )}
      </div>
    </main>
  );
}

export default function App() {
  return currentWindow.label === "widget" ? <Widget /> : <Panel />;
}

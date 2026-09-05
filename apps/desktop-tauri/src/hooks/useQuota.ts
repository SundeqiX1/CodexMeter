import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { useCallback, useEffect, useState } from "react";
import { resolveLanguage, translations } from "../i18n";
import type { AppSettings, FrontendState } from "../types";

const INITIAL_STATE: FrontendState = {
  connection: { status: "disconnected" },
  snapshot: null,
  lastUpdated: null,
  platform: "unknown",
  settings: {
    language: "system",
    refreshIntervalSecs: 60,
    compactMenuBar: false,
    hideMissingWindows: false,
    widgetVisible: false,
    widgetPosition: null,
    codexBinaryPath: null,
  },
};

export function useQuota() {
  const [state, setState] = useState<FrontendState>(INITIAL_STATE);
  const [launchAtLogin, setLaunchAtLogin] = useState(false);
  const [settingsMessage, setSettingsMessage] = useState<string | null>(null);
  const language = resolveLanguage(state.settings.language);
  const copy = translations[language];

  useEffect(() => {
    let disposed = false;
    let disposeListener: (() => void) | undefined;

    const initialize = async () => {
      try {
        disposeListener = await listen<FrontendState>("quota://updated", (event) => {
          if (!disposed) setState(event.payload);
        });

        const [initialState, autostartEnabled] = await Promise.all([
          invoke<FrontendState>("get_frontend_state"),
          isEnabled(),
        ]);

        if (disposed) return;
        setState(initialState);
        setLaunchAtLogin(autostartEnabled);
        await invoke("connect_codex");
      } catch (error) {
        if (!disposed) {
          setState((current) => ({
            ...current,
            connection: { status: "failed", message: String(error) },
          }));
        }
      }
    };

    void initialize();
    return () => {
      disposed = true;
      disposeListener?.();
    };
  }, []);

  useEffect(() => {
    document.documentElement.lang = language;
    void invoke("set_ui_language", { language });
  }, [language]);

  const refresh = useCallback(async () => {
    await invoke("refresh_codex");
  }, []);

  const reconnect = useCallback(async () => {
    await invoke("reconnect_codex");
  }, []);

  const setLaunchAtLoginEnabled = useCallback(async (enabled: boolean) => {
    setSettingsMessage(null);
    try {
      if (enabled) await enable();
      else await disable();
      const confirmed = await isEnabled();
      setLaunchAtLogin(confirmed);
      if (confirmed !== enabled) setSettingsMessage(copy.autostartNotConfirmed);
    } catch (error) {
      setSettingsMessage(`${copy.autostartFailed}: ${String(error)}`);
    }
  }, [copy]);

  const saveSettings = useCallback(async (settings: AppSettings) => {
    setSettingsMessage(null);
    try {
      const saved = await invoke<AppSettings>("save_settings", { settings });
      setState((current) => ({ ...current, settings: saved }));
      return true;
    } catch (error) {
      setSettingsMessage(`${copy.saveFailed}: ${String(error)}`);
      return false;
    }
  }, [copy]);

  const setWidgetVisible = useCallback(async (visible: boolean) => {
    await invoke("set_widget_visible", { visible });
  }, []);

  return {
    state,
    launchAtLogin,
    settingsMessage,
    refresh,
    reconnect,
    saveSettings,
    setWidgetVisible,
    setLaunchAtLoginEnabled,
  };
}

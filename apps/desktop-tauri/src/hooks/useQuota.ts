import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { useCallback, useEffect, useState } from "react";
import type { FrontendState } from "../types";

const INITIAL_STATE: FrontendState = {
  connection: { status: "disconnected" },
  snapshot: null,
  lastUpdated: null,
  platform: "unknown",
};

export function useQuota() {
  const [state, setState] = useState<FrontendState>(INITIAL_STATE);
  const [launchAtLogin, setLaunchAtLogin] = useState(false);
  const [settingsMessage, setSettingsMessage] = useState<string | null>(null);

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
      if (confirmed !== enabled) setSettingsMessage("系统没有确认这次开机启动设置。");
    } catch (error) {
      setSettingsMessage(`开机启动设置失败：${String(error)}`);
    }
  }, []);

  return {
    state,
    launchAtLogin,
    settingsMessage,
    refresh,
    reconnect,
    setLaunchAtLoginEnabled,
  };
}

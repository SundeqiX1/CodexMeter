# Architecture

## Decision

CodexMeter directly evolves the Tauri implementation in `changzhengithub/codex-quota-tool` instead of starting over. That code already had the most failure-prone pieces: cross-platform Codex discovery, a newline-delimited App Server handshake, child-process cleanup, request timeouts, sparse-update refetching, and Tauri tray integration.

The alternative reference, `cpys/codex-quota-overlay`, has valuable release validation and window-behavior work, but its Electron runtime, foreground-window helpers, quota history, predictions, activity dashboard, and optional telemetry are outside CodexMeter's scope. Porting it would increase binary size and maintenance burden.

The duplicate native Swift implementation inherited from the upstream repository was removed. Tauri 2, Rust, and one React frontend are the only maintained application path.

## Runtime flow

```text
menu bar / tray / widget
          │
          ▼
   Tauri command/event boundary
          │ normalized fields only
          ▼
 Rust quota service ── JSONL over stdio ── codex app-server
          │
          ├── current snapshot (memory only)
          └── allowlisted settings.json (UI settings only)
```

The Rust service owns process execution and quota state. The webviews receive typed, normalized fields and have no shell or filesystem permissions. App Server messages are not logged or saved.

## Protocol behavior

1. Locate the configured Codex binary, `CODEX_BINARY`, a standard macOS application binary, or `codex` in `PATH`.
2. Start `codex app-server --stdio` with piped stdin/stdout/stderr and no console window on Windows.
3. Send `initialize`, then the `initialized` notification.
4. Send `account/rateLimits/read` immediately and every configured interval.
5. On `account/rateLimits/updated`, read a complete snapshot instead of trying to persist raw sparse updates.
6. Prefer `rateLimitsByLimitId.codex`; fall back to `rateLimits`.
7. Match 5 Hour and Weekly by duration (`300` and `10080` minutes respectively).

If the child process exits or a request fails, the last valid snapshot is retained in memory and the connection becomes stale. The next refresh interval attempts to start or query the App Server again.

## Local state boundary

`settings.json` is decoded into a fixed Rust structure. Unknown keys are ignored and never copied forward. Accepted values are interface language, refresh interval, compact-title mode, missing-window behavior, widget visibility/position, and a custom executable path. No quota or identity field exists in the persisted schema.

## Window model

- `panel`: compact on-demand quota details and settings; hidden instead of terminating the app.
- `widget`: transparent, always-on-top, non-focusable, taskbar-hidden, draggable, and independently closable.
- `tray`: left click toggles the panel; the context menu exposes widget, refresh, startup, settings, and quit actions.
- macOS uses accessory activation policy so it does not occupy the Dock.

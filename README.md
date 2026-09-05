# CodexMeter

CodexMeter is a small, private, cross-platform menu bar and system tray monitor for Codex quota. It uses the local Codex App Server, keeps quota snapshots in memory, and focuses on the two windows people need at a glance: 5 Hour and Weekly.

> This is an independent community project and is not affiliated with or endorsed by OpenAI.

[简体中文](README.zh-CN.md)

## Highlights

- macOS menu bar title such as `5h 76% · W 43%`, with an optional compact mode.
- Windows 10/11 system tray with left-click access to the usage panel.
- Optional draggable, always-on-top floating widget that does not take keyboard focus.
- Follow-system, English, and Simplified Chinese UI, including the tray menu.
- Native light/dark appearance, a compact layout, and no dashboard or Electron runtime.
- Immediate refresh at startup, automatic refresh every 30 or 60 seconds, manual refresh, and reconnect support.
- Last valid values remain visible as `Disconnected · Stale` during temporary failures.
- No screenshots, OCR, browser cookies, copied tokens, quota history, or telemetry.

## Data source

CodexMeter starts the official local process:

```text
codex app-server --stdio
```

It completes the JSONL initialization handshake and calls the read-only JSON-RPC method:

```json
{"method":"account/rateLimits/read","id":10}
```

Both `rateLimitsByLimitId.codex` and the legacy `rateLimits` response are accepted. Windows are identified only by `windowDurationMins`:

| Duration | Display |
| ---: | --- |
| 300 minutes | 5 Hour |
| 10,080 minutes | Weekly |

`primary` and `secondary` are not treated as fixed labels. If the server omits a window, CodexMeter shows `--` or hides it when that preference is enabled. It never estimates a missing quota window.

This behavior is plan-agnostic. Plus, Pro 5x, Pro 20x, Business, and future plans use the same parser: CodexMeter displays only the windows returned for `codex`. A weekly-only Pro response therefore renders `5h -- · W …` by default, or only `W …` when **Hide windows not reported by the server** is enabled. Plan names and multipliers are never used to manufacture a limit. OpenAI's current plan documentation notes that Pro offers 5x or 20x usage and that weekly limits may also apply, while the actual windows available to an account remain server-controlled; see [OpenAI Codex pricing and usage limits](https://learn.chatgpt.com/docs/pricing).

## Supported targets

| Platform | Target | Packaging |
| --- | --- | --- |
| macOS 13+ | Apple Silicon (`aarch64`) | DMG and ZIP |
| macOS 13+ | Intel (`x86_64`) | DMG and ZIP |
| Windows 10/11 | x64 | NSIS setup EXE and portable ZIP |

Public macOS releases should be Developer ID signed and notarized, and public Windows installers should be code signed. Unsigned development packages will trigger operating-system warnings.

## Requirements

- A working `codex` executable and an existing ChatGPT-managed Codex sign-in.
- Node.js 20+ and Rust stable when building from source.
- Microsoft C++ Build Tools and Microsoft Edge WebView2 on Windows (WebView2 is normally already installed on Windows 10/11).

CodexMeter auto-detects the executable in `PATH` and the standard macOS ChatGPT/Codex application locations. A custom path can be selected in Settings. `CODEX_BINARY` remains available as an environment override; Windows users may explicitly opt into a WSL fallback with `CODEX_USE_WSL=1`.

For official Codex installation and sign-in instructions, see the [Codex CLI documentation](https://learn.chatgpt.com/docs/codex/cli). Windows source builds also need the dependencies listed in the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

## Install and use

### macOS

1. Open the architecture-matched DMG (`arm64` for Apple Silicon, `x64` for Intel), then drag CodexMeter to Applications. A development build can also be launched directly from `CodexMeter.app`.
2. Start CodexMeter. If an unsigned preview is blocked, Control-click the app, choose **Open**, then confirm **Open**. Public releases should be signed and notarized.
3. Click the `5h … · W …` title in the menu bar to open or hide the quota panel. CodexMeter runs as a menu-bar accessory and does not occupy the Dock.
4. Turn on **Floating Widget** in the panel, choose it from the menu-bar context menu, or enable **Show floating widget** in Settings and save. The widget first appears in the center of the screen and remembers where you drag it.
5. Open Settings and choose **Follow system**, **English**, or **Simplified Chinese**. The panel, widget, status labels, dates, and tray menu update together.

The `×` in the quota panel hides the panel. The `×` in the widget hides only the widget. Neither action quits CodexMeter; use **Quit CodexMeter** from the panel or menu-bar menu to stop the process.

### Windows 10/11 x64

1. Run `CodexMeter-Windows-Setup-<version>-x64.exe`, or extract the portable ZIP and launch `CodexMeter.exe`.
2. Make sure the native Windows `codex` command is available in `PATH`. If it is installed somewhere else, enter the full path to `codex.exe`, `codex.cmd`, or `codex.bat` in Settings.
3. Find CodexMeter in the notification area; Windows may place it behind the hidden-icons arrow. Left-click the tray icon to open or hide the panel. Right-click it for **Show Widget**, **Refresh**, **Launch at Startup**, **Settings**, and **Quit CodexMeter**.

The current checkout can produce macOS packages on a Mac. Windows packages are built on Windows or by the release workflow after the repository is pushed to GitHub.

## Run in development

Install the platform prerequisites, then run:

```bash
cd apps/desktop-tauri
npm ci
npm run tauri dev
```

This starts Vite and the native Tauri process together. Running only `npm run dev` opens the web frontend without the Rust backend, tray, App Server connection, or native widget.

## Build from source

```bash
cd apps/desktop-tauri
npm ci
npm run build
cargo test --locked --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

On Windows, use the same commands from PowerShell. Tauri packages must be built on the target operating system. The GitHub Actions workflows run frontend checks and Rust tests on macOS and Windows, build each release target, generate portable ZIPs, and publish SHA-256 checksum files for tags matching `v*`.

For a local package with an easy-to-find output path, use the platform command instead:

```bash
# macOS
env APPLE_SIGNING_IDENTITY=- npm run package:mac

# Windows PowerShell
npm run package:windows
```

Both commands copy the finished app/installer and `SHA256SUMS.txt` to the repository-root [`artifacts`](artifacts) directory. The deep Tauri target directory remains only an implementation detail.

## Privacy

Quota snapshots, account metadata returned by the App Server, and operational errors stay in process memory. The settings file is allowlisted and may contain only language, widget position, UI preferences, refresh interval, widget visibility, and a custom Codex binary path. CodexMeter does not persist tokens, account identity, raw App Server responses, quota history, or chat content. See [PRIVACY.md](PRIVACY.md).

## Project lineage

CodexMeter is a maintained derivative of the MIT-licensed [changzhengithub/codex-quota-tool](https://github.com/changzhengithub/codex-quota-tool). Its original copyright notice and license are preserved. The release validation and window-behavior ideas were also reviewed against [cpys/codex-quota-overlay](https://github.com/cpys/codex-quota-overlay); CodexMeter does not use its Electron runtime, telemetry, persisted history, or forecasting features.

## Contributing and security

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. Please report vulnerabilities using [SECURITY.md](SECURITY.md) and never post real account identifiers, tokens, raw responses, or local paths in a public issue.

## License

MIT. See [LICENSE](LICENSE).

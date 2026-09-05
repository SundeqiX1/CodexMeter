# Contributing to CodexMeter

Thanks for helping keep CodexMeter small, stable, and private.

## Scope

Changes should preserve the project's status-tool focus. Prefer clear 5 Hour/Weekly/reset-time behavior over dashboards, forecasts, history, or social features. Do not add screenshot capture, OCR, browser-cookie access, token handling, remote quota proxies, telemetry, or persisted quota samples.

## Development setup

Install Node.js 20 or newer, Rust stable, the platform's Tauri prerequisites, and a working Codex CLI. On Windows, use Windows 10/11 x64 with WebView2. On macOS, install Xcode Command Line Tools; full Xcode is required for some packaging and signing tasks.

```bash
cd apps/desktop-tauri
npm ci
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --locked --manifest-path src-tauri/Cargo.toml
```

Use `npm run package:mac` or `npm run package:windows` when you need a local package in the repository-root `artifacts/` directory.

The live account integration is intentionally not part of normal tests. Use synthetic fixtures for parsing tests. If a local live test is necessary, do not record its raw output and do not commit account data.

## Pull requests

- Keep changes focused and explain user-visible behavior.
- Add tests for quota parsing, missing-window behavior, reconnect logic, and platform-specific code where practical.
- Run the frontend build, Rust formatter, and Rust tests.
- Update both English and Chinese documentation for user-visible changes.
- Do not commit `node_modules`, `target`, packaged applications, credentials, raw responses, or local settings.
- Preserve the original MIT license notice and project attribution.

## Release changes

Version changes must remain synchronized in `package.json`, `Cargo.toml`, `tauri.conf.json`, and the release tag. Release workflows should produce checksums for every binary artifact. Signing secrets belong in GitHub Actions secrets or a trusted signing service, never in the repository.

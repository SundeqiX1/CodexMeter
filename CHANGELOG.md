# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Cross-platform Tauri 2 desktop application for macOS and Windows.
- Windows system tray integration, launch at login, and Codex CLI discovery.
- Shared React interface with a compact always-on-top quota dashboard.
- macOS and Windows CI coverage for the Tauri frontend and Rust backend.

### Planned

- Developer ID signing and Apple notarization for downloadable releases.
- Windows code signing and automated multi-platform release artifacts.

## [0.1.0] - 2026-07-17

### Added

- Native macOS menu bar quota indicator.
- Draggable always-on-top floating quota card.
- Codex App Server JSONL client using the current local Codex login.
- Quota windows, reset times, credit balance, spend controls, and reset credits.
- Five-minute refresh, sparse-update refetching, reconnection, and launch at login.
- Unit tests and an opt-in live integration test.

[Unreleased]: https://github.com/changzhengithub/codex-quota-tool/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/changzhengithub/codex-quota-tool/releases/tag/v0.1.0

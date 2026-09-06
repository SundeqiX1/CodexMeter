# Changelog

All notable changes are documented here. The project follows Semantic Versioning.

## [Unreleased]

### Fixed

- Apply the missing-window visibility preference consistently to the floating widget.
- Make widget details a deterministic click-to-expand/click-to-collapse interaction without opening the panel or Settings.
- Shrink the floating widget automatically when only one quota window is visible.
- Show the App Server-reported subscription tier in the main panel without persisting account metadata.
- Prevent release builds from opening an extra console window on Windows.

## [0.1.0-preview.1] - 2026-09-06

### Added

- CodexMeter identity and bilingual public documentation.
- Compact menu bar/tray usage panel and an optional non-focusable floating widget.
- Configurable 30/60 second refresh and allowlisted local settings.
- Windows x64, macOS arm64, and macOS x64 release workflows with portable archives and SHA-256 checksums.
- Follow-system, English, and Simplified Chinese UI and tray-menu localization.
- Root-level local artifact export with SHA-256 checksums.

### Changed

- Quota windows are selected by `windowDurationMins`, not `primary`/`secondary` position.
- Both keyed-only `rateLimitsByLimitId.codex` and legacy `rateLimits` payloads are accepted.
- Missing windows display `--` or can be hidden; no missing value is estimated.
- Failed connections retain the last valid in-memory snapshot as stale data.
- Removed the duplicate legacy Swift implementation so Tauri is the single maintained desktop codebase.
- Updated the frontend build dependency lock to patched `browserslist`, `postcss`, and `nanoid` versions.

## Upstream history

CodexMeter derives from the MIT-licensed `changzhengithub/codex-quota-tool`. Earlier history remains available in Git, and the upstream copyright notice remains in `LICENSE`.

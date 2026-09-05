# Releasing

Version `0.1.0` is a preview. Do not mark a release stable until the generated Windows x64, macOS arm64, and macOS x64 artifacts have each passed an installation and launch smoke test on matching hardware or a trusted virtual machine. The application also depends on the experimental Codex App Server interface, so every release must be tested against the supported Codex CLI version range.

## Before tagging

1. Synchronize the version in `apps/desktop-tauri/package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. Update `CHANGELOG.md`.
3. Run the frontend build, Rust formatting check, and Rust tests on both supported operating systems.
4. Test the tray, menu bar title, missing-window behavior, widget dragging/position restore, startup toggle, and stale recovery on physical machines.
5. Confirm the repository contains no account data, settings files, raw responses, certificates, or credentials.

For an easy-to-find local preview build, run `npm run package:mac` or `npm run package:windows` from `apps/desktop-tauri`. The export step places the native app/installer and SHA-256 checksum file in the repository-root `artifacts/` directory.

## Signing

For public macOS downloads, configure Developer ID Application signing, Hardened Runtime, notarization, and stapling. The workflow accepts Tauri's standard Apple signing/notarization secrets and falls back to an ad-hoc signature when no identity is configured; that fallback is only a development preview. Validate final applications with `codesign`, `spctl`, and `xcrun stapler`.

For public Windows downloads, configure a trusted Authenticode certificate or signing service and sign both the application and NSIS installer. Unsigned builds are development previews and display an unknown-publisher warning.

## Tag and publish

For a preview, append a prerelease suffix to the synchronized application version:

```bash
git tag -a v0.1.0-preview.1 -m "CodexMeter v0.1.0 preview 1"
git push origin v0.1.0-preview.1
```

The workflow accepts `v<version>-<prerelease>` and marks any hyphenated tag as a GitHub prerelease. After matching-hardware smoke tests, publish the stable tag without a suffix:

```bash
git tag -a v0.1.0 -m "CodexMeter v0.1.0"
git push origin v0.1.0
```

The release workflow builds Windows x64, macOS arm64, and macOS x64. It publishes:

- Windows setup EXE and portable ZIP;
- arm64 and x64 macOS DMG and ZIP;
- per-target SHA-256 checksum files.

Download every artifact from the release, verify checksums, and install it on a clean matching platform before promoting a preview to stable.

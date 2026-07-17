# Contributing

Thanks for helping improve CodexQuotaWidget. Small, focused pull requests are
easier to review and are encouraged.

## Development requirements

- macOS 13+ or Windows 10/11
- Node.js 20 or later and npm
- Current stable Rust toolchain
- Platform prerequisites for Tauri 2
- Xcode 16+ with Swift 6 tooling when changing the native macOS implementation
- Codex or the ChatGPT desktop app for live integration testing

## Set up the project

```bash
git clone https://github.com/changzhengithub/codex-quota-tool.git
cd codex-quota-tool
cd apps/desktop-tauri
npm ci
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

Run the cross-platform desktop app in development mode:

```bash
npm run tauri dev
```

Open `Package.swift` in Xcode only when working on the retained native SwiftUI
implementation. Its local bundle script is `./Scripts/build-app.sh`.

## Tests

Run deterministic tests before submitting a pull request:

```bash
cd apps/desktop-tauri
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml

cd ../..
swift test
```

The live integration test is opt-in because it uses the current machine's
signed-in Codex account:

```bash
LIVE_CODEX_TEST=1 swift test --filter CodexAppServerIntegrationTests
```

Never enable the live test in public CI, and never add credentials or Codex
session files to a fixture.

## Protocol changes

`codex app-server` is experimental. When its response shape changes:

1. Keep new server fields optional unless they are guaranteed by the stable protocol.
2. Update both Rust and Swift models plus their decoding fixtures when applicable.
3. Preserve the full-snapshot refetch after sparse `account/rateLimits/updated` notifications.
4. Mention the tested Codex version in the pull request.

## Pull requests

- Explain the user-visible problem and the chosen behavior.
- Add or update tests for parsing and state changes.
- Keep unrelated formatting and refactors out of the change.
- Confirm the Tauri frontend build and Rust tests pass; run Swift tests when changing shared behavior or Swift code.
- Do not commit `.app`, `.zip`, `.build`, signing keys, or notarization credentials.

By contributing, you agree that your contribution is licensed under the MIT
License in this repository.

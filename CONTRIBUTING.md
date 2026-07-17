# Contributing

Thanks for helping improve CodexQuotaWidget. Small, focused pull requests are
easier to review and are encouraged.

## Development requirements

- macOS 14 or later
- Xcode 16 or later with Swift 6 tooling
- Codex or the ChatGPT desktop app for live integration testing

## Set up the project

```bash
git clone https://github.com/changzhengithub/codex-quota-tool.git
cd codex-quota-tool
swift test
```

Open `Package.swift` in Xcode to run or debug the menu bar app. To build a local
`.app` bundle:

```bash
./Scripts/build-app.sh
open dist/CodexQuotaWidget.app
```

## Tests

Run deterministic tests before submitting a pull request:

```bash
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
2. Update `RateLimitModels.swift` and its decoding fixtures together.
3. Preserve the full-snapshot refetch after sparse `account/rateLimits/updated` notifications.
4. Mention the tested Codex version in the pull request.

## Pull requests

- Explain the user-visible problem and the chosen behavior.
- Add or update tests for parsing and state changes.
- Keep unrelated formatting and refactors out of the change.
- Confirm `swift test` and `swift build -c release` pass.
- Do not commit `.app`, `.zip`, `.build`, signing keys, or notarization credentials.

By contributing, you agree that your contribution is licensed under the MIT
License in this repository.

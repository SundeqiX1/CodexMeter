# Open-source publishing checklist

- [x] Create the public `CodexMeter` GitHub repository, add it as `origin`, and retain the source project as `upstream`.
- [x] Keep the upstream MIT copyright notice and attribution.
- [x] Enable private vulnerability reporting and Dependabot/security scanning.
- [ ] Protect `main` and require the macOS and Windows CI jobs.
- [x] Review the repository for tokens, account identifiers, raw App Server responses, local settings, screenshots, and personal paths.
- [x] Confirm the independent-community-project disclaimer is prominent in both READMEs.
- [ ] Configure Apple Developer ID/notarization and Windows signing before describing downloads as trusted production packages.
- [x] Build and inspect macOS arm64, macOS x64, and Windows x64 artifacts and verify every SHA-256 checksum.
- [x] Complete the macOS arm64 installation and live App Server smoke test.
- [ ] Complete matching-hardware installation and live App Server smoke tests on macOS x64 and Windows x64 before marking the release stable.

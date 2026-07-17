# Releasing

Source releases and downloadable macOS binaries have different trust
requirements. Source code can be tagged immediately; public binaries should be
signed with a Developer ID Application certificate and notarized by Apple.

## Prepare a release

1. Update `CHANGELOG.md` and the versions in `Packaging/Info.plist`.
2. Run `swift test`.
3. Run the opt-in live integration test on a disposable development account when possible.
4. Build and test on every advertised architecture.
5. Confirm the repository contains no generated binaries or credentials.

## Local development build

```bash
./Scripts/build-app.sh
codesign --verify --deep --strict dist/CodexQuotaWidget.app
```

The script uses ad-hoc signing and is intended for local testing only.

## Public binary release

Public releases should use a Developer ID Application identity, Hardened
Runtime, a secure timestamp, Apple notarization, and a stapled notarization
ticket. Keep certificate files and notary credentials outside the repository.

Typical verification commands after signing and notarization:

```bash
codesign --verify --deep --strict --verbose=2 CodexQuotaWidget.app
xcrun stapler validate CodexQuotaWidget.app
spctl --assess --type execute --verbose=4 CodexQuotaWidget.app
```

Create the final archive only after stapling the app:

```bash
ditto -c -k --sequesterRsrc --keepParent CodexQuotaWidget.app CodexQuotaWidget.zip
shasum -a 256 CodexQuotaWidget.zip
```

Attach the ZIP and checksum to a GitHub Release rather than committing them to
the source tree.

## Tagging

```bash
git tag -s v0.1.0 -m "CodexQuotaWidget v0.1.0"
git push origin v0.1.0
```

If signed tags are not configured, use an annotated tag and document that
choice in the release notes.

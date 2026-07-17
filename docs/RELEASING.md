# Releasing

Source releases and downloadable desktop binaries have different trust
requirements. Source code can be tagged immediately. Public macOS binaries
should be signed and notarized by Apple; Windows installers should be signed
with a trusted code-signing certificate.

## Prepare a release

1. Update `CHANGELOG.md`, `apps/desktop-tauri/package.json`,
   `apps/desktop-tauri/src-tauri/Cargo.toml`, `tauri.conf.json`, and the native
   `Packaging/Info.plist` when it is included in the release.
2. Run the frontend build, Rust tests, and relevant Swift tests.
3. Run the opt-in live integration test on a disposable development account when possible.
4. Build and test on every advertised architecture.
5. Confirm the repository contains no generated binaries or credentials.

## Cross-platform local builds

Build on each target operating system rather than copying one platform's
bundle to another:

```bash
cd apps/desktop-tauri
npm ci
npm run tauri build
```

Artifacts are placed under `src-tauri/target/release/bundle/`. macOS produces
application/disk-image bundles; Windows produces MSI/NSIS installers.

## Native macOS development build

```bash
./Scripts/build-app.sh
codesign --verify --deep --strict dist/CodexQuotaWidget.app
```

The script uses ad-hoc signing and is intended for local testing only.

## Public macOS binary release

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

## Public Windows binary release

Build on a clean Windows runner. Sign the application executable, MSI, and NSIS
installer with the project's Windows code-signing certificate, then verify the
signature before uploading. Keep the certificate and its password in GitHub
Actions secrets or an external signing service; never commit either one.

Unsigned Windows packages are acceptable for local development, but they will
show an unknown-publisher warning and should not be advertised as a trusted
public download.

## Tagging

```bash
git tag -s v0.2.0 -m "Codex Quota Tool v0.2.0"
git push origin v0.2.0
```

If signed tags are not configured, use an annotated tag and document that
choice in the release notes.

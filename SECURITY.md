# Security Policy

## Supported versions

Security fixes are applied to the latest released version and the `main` branch.

## Reporting a vulnerability

Please use GitHub's private Security Advisories feature for this repository. If it is unavailable, contact the repository maintainers privately before opening a public issue.

Include the affected version, platform, impact, reproduction steps, and a minimal proof of concept. Do not include real tokens, cookies, account identifiers, raw App Server responses, chat content, or personally identifying local paths. Replace those values with obvious placeholders.

Please allow a reasonable period for validation and a coordinated fix before public disclosure.

## Security boundaries

CodexMeter communicates only with the local `codex app-server` over child-process stdio. It must not directly read Codex authentication storage or browser cookies. Quota data is memory-only, renderer access is limited by Tauri capabilities, and the settings store accepts only documented UI and executable-path fields.

Release maintainers should use supported GitHub Actions releases, review dependency changes, publish SHA-256 checksums, and sign/notarize distributed binaries whenever signing credentials are available. Never commit signing certificates, passwords, notarization credentials, tokens, or generated account fixtures.

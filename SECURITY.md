# Security Policy

## Supported versions

Security fixes are provided for the latest release and the current `main`
branch. Older builds may stop working as the experimental Codex App Server
protocol evolves.

## Reporting a vulnerability

Please use GitHub's private vulnerability reporting feature for this
repository. Do not open a public issue for vulnerabilities involving:

- authentication or Codex session data;
- command or executable path injection;
- unintended disclosure of account, quota, or billing information;
- release signing or update integrity;
- persistent access outside the documented local Codex connection.

Include reproduction steps, affected versions, impact, and a minimal proof of
concept when possible. Do not include real access tokens, cookies, credentials,
or private Codex session files.

## Security boundaries

CodexQuotaWidget launches the locally installed `codex app-server` executable
and exchanges newline-delimited JSON over standard input and output. It should
not read, copy, log, or store Codex authentication tokens directly. Quota
snapshots are held in memory for display and are not uploaded by the widget.

The Codex process uses the user's existing OpenAI connection. Users should only
install binaries from a trusted release and verify the published checksum.

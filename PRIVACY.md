# Privacy Policy

CodexMeter is local-first and has no telemetry, analytics SDK, advertising SDK, update tracking, or remote control plane.

## Data it processes

CodexMeter launches the locally installed `codex app-server --stdio` process and asks it for `account/rateLimits/read`. The App Server uses the user's existing Codex sign-in. CodexMeter does not read, copy, or receive the underlying token.

The response may contain quota windows, reset times, plan metadata, credits, reset-credit details, and an account identifier. CodexMeter deserializes only the fields needed by the UI. Account identifiers and unknown response fields are discarded. Current quota state and errors remain in process memory and disappear when the application exits.

The App Server may communicate with OpenAI as part of its normal operation. CodexMeter itself does not send quota data anywhere.

## Data it stores

The allowlisted `settings.json` file may contain only:

- interface language preference;
- refresh interval;
- compact menu bar preference;
- missing-window display preference;
- floating-widget visibility and position;
- a user-selected Codex executable path.

The operating system separately manages the launch-at-login/startup registration through Tauri's official autostart plugin.

CodexMeter does not store authentication tokens, cookies, account identity, raw App Server responses, quota history, chat content, prompts, or operational logs.

## Permissions

CodexMeter needs permission to launch the local Codex executable, show its menu bar/tray UI, keep the optional widget above other windows, and register startup when the user explicitly enables it. It does not need screen recording, accessibility inspection, browser data, microphone, camera, contacts, or location access.

## Deleting local data

Disable launch at login/startup from the application, quit CodexMeter, and delete its operating-system application configuration folder. No server-side CodexMeter account or cloud copy exists.

## Changes

Material privacy changes must be documented in the changelog and reviewed like code. A future network or telemetry feature must not be added silently; it requires a separate, explicit design and user consent.

# Open-source publishing checklist

Complete these repository-owner-specific steps before making the repository public.

## Repository identity

- [x] Set the repository owner to `changzhengithub` in repository URLs.
- [x] Set the public repository name to `codex-quota-tool`.
- [x] Set the bundle identifier to `io.github.changzhengithub.codexquotatool` in `Packaging/Info.plist`.
- [ ] Configure Git with the public author name and a GitHub `noreply` email if personal-email privacy matters.

## Repository settings

- [ ] Create an empty GitHub repository without auto-generating a README, license, or `.gitignore`.
- [ ] Push the `main` branch and verify that the CI workflow passes.
- [ ] Enable private vulnerability reporting under **Settings → Security**.
- [ ] Add repository topics such as `macos`, `swiftui`, `codex`, and `menu-bar-app`.
- [ ] Review branch protection or rulesets for `main` if multiple people will contribute.

## Binary releases

- [ ] Decide whether the first public release is source-only or includes a downloadable app.
- [ ] For a public app download, sign with an Apple Developer ID certificate and notarize it. Do not present the ad-hoc-signed local build as a trusted public release.
- [ ] Test the final archive on a clean macOS user account without development tools installed.
- [ ] Follow `docs/RELEASING.md`, update `CHANGELOG.md`, then create the version tag and GitHub Release.

## Final review

- [ ] Confirm `.build/`, `dist/`, `.DS_Store`, certificates, provisioning profiles, and environment files are not tracked.
- [ ] Review the staged diff for tokens, personal paths, account identifiers, and screenshots containing private data.
- [ ] Confirm that the unofficial-project notice and experimental API limitation remain prominent in `README.md`.

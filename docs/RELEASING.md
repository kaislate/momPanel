# Releasing momPanel (and how auto-update works)

momPanel auto-updates from **GitHub Releases** of `kaislate/momPanel`. On launch the
app checks the release's `latest.json`, and if a newer **signed** version exists it
downloads and installs it silently, then restarts. Only builds signed with our private
key can install, so the keypair must be guarded.

## One-time setup: GitHub secrets

A signing keypair was generated locally and stored **outside the repo** at
`~/.tauri/momPanel_updater.key` (private) and `…key.pub` (public). The public key is
already committed in `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`). The
private key must live only in CI as a secret.

In the `kaislate/momPanel` GitHub repo, go to **Settings → Secrets and variables →
Actions → New repository secret** and add:

| Secret name | Value |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | the full contents of `~/.tauri/momPanel_updater.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | empty (the key was generated with no password) |

> The private key file content is a single base64 line. Never commit it. If it leaks
> or is lost, generate a new keypair (`npm run tauri signer generate`), update the
> `pubkey` in `tauri.conf.json`, and ship a normal (non-update) install — old installs
> can no longer auto-update across a key change.

## Cutting a release

1. Bump `version` in `src-tauri/tauri.conf.json` (e.g. `0.0.2`).
2. Commit, then tag and push:
   ```bash
   git tag v0.0.2
   git push origin v0.0.2
   ```
3. The **Release** workflow (`.github/workflows/release.yml`) builds and signs:
   - **Linux:** `momPanel_<version>_amd64.AppImage` (the self-updatable bundle)
   - **Windows:** NSIS installer (dev/test target)
   - plus `latest.json` (the update manifest the app reads)
   and publishes them to the GitHub Release for the tag.
4. Existing installs pick up the update automatically on their next launch.

## Notes

- **Linux must ship as AppImage** for self-update — the `.deb` format cannot update
  itself. The workflow already produces the AppImage.
- The updater is non-blocking: if GitHub is unreachable or there's no newer version,
  launch proceeds normally.
- To test the manifest without affecting users, push a tag to a throwaway repo/branch
  or mark the release as a draft/prerelease first.

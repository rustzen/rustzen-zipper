# Signing Key Management

## Tauri Updater Key Pair

Rustzen Zipper uses a dedicated Tauri updater key pair.

- Public key: committed in `apps/macos/src-tauri/tauri.conf.json`.
- Private key: stored outside the repository at `~/.tauri/rustzen-zipper-updater.key`.
- Public key file: `~/.tauri/rustzen-zipper-updater.key.pub`.
- iCloud backup: `iCloud Drive/Rustzen/keys/rustzen-zipper/`.

The private key is not password protected and must stay mode `600`.

## Local Release Build

```bash
export TAURI_SIGNING_PRIVATE_KEY="$(cat "$HOME/.tauri/rustzen-zipper-updater.key")"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
cd apps/macos
bun run tauri build
```

Do not commit the updater private key, Apple signing assets, or notarization credentials.

`apps/macos/src-tauri/tauri.conf.json` enables `bundle.createUpdaterArtifacts`, so release builds must provide the updater private key content through `TAURI_SIGNING_PRIVATE_KEY`.

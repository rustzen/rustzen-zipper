# Rustzen Zipper macOS App

This is the first desktop shell for `rustzen-zipper`.

## Scope

- ZIP-first archive utility.
- Drag and drop `.zip` files into the app.
- Choose an output directory.
- Inspect archive metadata.
- Extract archive entries through the shared Rust core.
- Check updates through Tauri updater.

## Visual direction

The app follows the same Zen Blue Glass direction used by Rustzen Clear:

- soft blue-white translucent surfaces
- low-contrast gradients
- light shadows
- rounded macOS overlay window chrome
- restrained icon detail
- abstract zipper `Z` rather than a heavy realistic zipper

Brand source assets live in:

```text
apps/macos/assets/brand/
```

Runtime PNG/ICNS/ICO icon files are committed directly under `src-tauri/icons/`.
The Tauri bundle config consumes those runtime assets, not an SVG source file.

## Development

From the repository root:

```bash
bun run desktop:dev
```

Or from this directory:

```bash
bun install
bun run tauri dev
```

## Build

```bash
export TAURI_SIGNING_PRIVATE_KEY="$(cat "$HOME/.tauri/rustzen-zipper-updater.key")"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
bun run desktop:build
```

## Updater

`src-tauri/tauri.conf.json` uses the Rustzen Cloud updater endpoint:

```text
https://cloud.rustzen.dev/api/updates/check?product=rustzen-zipper
```

The updater public key is committed in `src-tauri/tauri.conf.json`. The private key is stored outside the repository at `~/.tauri/rustzen-zipper-updater.key` and backed up to iCloud Drive under `Rustzen/keys/rustzen-zipper/`.

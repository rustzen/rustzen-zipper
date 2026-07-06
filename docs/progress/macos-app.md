# macOS App Progress

## Branch

`feat/macos-app`

## Goal

Build the first Rustzen Zipper macOS desktop shell while keeping the CLI usable.

## Scope completed

- Created reusable Rust library entry in `src/lib.rs`.
- Moved pack logic behind `rustzen_zipper::pack` through `src/archive_pack.rs`.
- Added reusable unzip logic in `src/unpack.rs`.
- Added reusable inspect logic in `src/inspect.rs`.
- Kept CLI default behavior compatible: `rz-zip` still packs by default.
- Added explicit commands:
  - `rz-zip pack`
  - `rz-zip unpack`
  - `rz-zip inspect`
- Added `apps/macos` Tauri desktop shell.
- Added UI for archive selection, drag/drop path handling, output directory selection, inspection, extraction, and update checking.
- Added Tauri updater configuration.
- Added macOS `.zip` document association.
- Added root scripts:
  - `bun run desktop:dev`
  - `bun run desktop:build`

## Rustzen Clear alignment

The first visual direction was rejected because it felt too heavy, saturated, and object-realistic compared with Rustzen Clear.

The current direction now follows the Zen Blue Glass system observed in `rustzen-clear/zen-gui`:

- `#77A8F7` primary blue
- `#A8CBFB` secondary blue
- `#D3E7FD` highlight
- `#EEF4FE` panel background
- translucent white-blue surfaces
- low contrast shadows
- rounded macOS overlay chrome
- restrained, abstract zipper detail

## Rustzen Clear icon metric audit

The uploaded Rustzen Clear 1024 icon was checked as the visual metric baseline.

Observed reference metrics:

- Canvas: `1024 x 1024`.
- Main card visual bounds: about `x=104 y=104 w=816 h=816`.
- Shadow extends downward toward `y=960`.
- Main card is white-blue, not saturated blue.
- Central symbol has generous padding and does not touch the card boundary.

Correction applied:

- Runtime PNG/ICNS/ICO icons now use the uploaded Rustzen Zipper PNG artwork
  converted to real alpha.
- The Tauri app consumes committed assets under `apps/macos/src-tauri/icons/`.
- There is no SVG app-icon source in the runtime chain.

New source assets:

- `apps/macos/assets/brand/logo-lockup.svg`
- `apps/macos/assets/brand/menubar-icon.svg`
- `apps/macos/assets/brand/menubar-variants.svg`
- `apps/macos/assets/brand/menu-popover-concept.svg`
- `apps/macos/assets/brand/README.md`

2026-07-03 follow-up audit: runtime PNG/ICNS/ICO icons now follow the Rustzen
Clear visual baseline with a real alpha background, large white-blue rounded
card, low-saturation blue details, and right-top bubble marks.

## Runtime assets

Runtime PNG/ICNS/ICO app icons were regenerated as committed raster assets on
2026-07-03 after the Rustzen Clear metric follow-up. The Tauri app consumes
`apps/macos/src-tauri/icons/` directly; there is no SVG app-icon source in the
runtime chain.

Generated outputs include:

- `apps/macos/src-tauri/icons/32x32.png`
- `apps/macos/src-tauri/icons/128x128.png`
- `apps/macos/src-tauri/icons/128x128@2x.png`
- `apps/macos/src-tauri/icons/icon.png`
- `apps/macos/src-tauri/icons/icon.icns`
- `apps/macos/src-tauri/icons/icon.ico`

The signed app bundle was verified to include `Contents/Resources/icon.icns` and `CFBundleIconFile = icon.icns`.

## Updater signing

The Tauri updater key pair was generated on 2026-07-03.

- Public key: committed in `apps/macos/src-tauri/tauri.conf.json`.
- Private key: stored outside the repository at `~/.tauri/rustzen-zipper-updater.key`.
- iCloud backup: `iCloud Drive/Rustzen/keys/rustzen-zipper/`.

## Updater endpoint

The updater endpoint has been aligned to Rustzen Cloud:

```text
https://cloud.rustzen.dev/api/updates/check?product=rustzen-zipper
```

## Verification

Verified on 2026-07-03 from `feat/macos-app` after the visual-alignment patch and icon regeneration:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -- --help
cargo run -- pack --help
cargo run -- unpack --help
cargo run -- inspect --help
cd apps/macos
bun install
bun run build
export TAURI_SIGNING_PRIVATE_KEY="$(cat "$HOME/.tauri/rustzen-zipper-updater.key")"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
bun run tauri build
```

Follow-up verification on 2026-07-03 after the Rustzen Clear metric correction
and runtime icon regeneration:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -- --help
cargo run -- pack --help
cargo run -- unpack --help
cargo run -- inspect --help
cd apps/macos
bun run build
export TAURI_SIGNING_PRIVATE_KEY="$(cat "$HOME/.tauri/rustzen-zipper-updater.key")"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
bun run tauri build
```

Results:

- `cargo fmt --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test`: passed, 26 integration tests.
- CLI help commands: passed.
- `bun run build`: passed.
- `bun run tauri build`: passed outside the sandbox after the first sandboxed
  DMG packaging attempt failed at `bundle_dmg.sh`.

Fixes from the previous verification pass:

- Restored the `--overwrite skip` compatibility message expected by CLI tests.
- Moved `.zip` file association config to Tauri v2 `bundle.fileAssociations`.
- Added app icon assets required by Tauri bundling.
- Added macOS app ignore rules for local build artifacts.
- Generated and configured the Tauri updater public key.

## Known review points

- Confirm Rustzen Cloud updater metadata before release.
- Confirm React/Tauri drag-drop event payload type against the installed Tauri API version.
- Confirm macOS document association behavior after a real app build.

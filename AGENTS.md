# rustzen-zipper Agent Guide

## Scope

`rustzen-zipper` is a Rust CLI and npm package for creating zip archives. It is
not a Web/Rust service, dashboard, Tauri client, or deployable Linux service.

Do not apply `apps/server`, `apps/web`, `/opt` runtime layout, systemd, Vercel,
or SQLite migration rules to this repository.

## Source Layout

- `src/main.rs`: Rust CLI implementation for `rz-zip`.
- `tests/cli.rs`: integration tests that execute the compiled CLI.
- `index.js`: npm bin wrapper that launches the platform binary.
- `scripts/install.js`: npm install hook that installs or downloads the
  platform binary.
- `scripts/test.js`: package smoke test used after local builds.
- `bin/`: published platform binaries.
- `.github/workflows/`: CI and release asset builds.
- `README.md`: user-facing command and package documentation.

## Working Rules

- Run `git status --short --branch` before editing, testing, staging, or
  committing.
- Keep npm package metadata, README command docs, CLI flags, tests, and release
  asset names aligned.
- Prefer focused Rust changes in `src/main.rs` and focused JS changes in
  `index.js` or `scripts/`.
- Do not add a frontend build system, server runtime, database layer, or service
  deployment assets unless explicitly requested.
- Preserve existing uncommitted changes and generated/local artifacts.

## Commands

Use the commands that exist in `package.json` and GitHub Actions:

```bash
cargo fmt --check
cargo build --locked
cargo test --locked
cargo clippy --locked -- -D warnings
pnpm test
pnpm run ci
```

`pnpm run ci` is the broadest local gate because it matches the package script.

## Special Cases

- SQLite startup migration is not applicable; this package has no runtime
  database.
- Web API, handler/service/repo layering, and frontend component organization are
  not applicable; this package exposes only a CLI and npm bin wrapper.
- Release artifacts are GitHub Release binaries named
  `rustzen-zipper-<target-triple>` with `.exe` for Windows.

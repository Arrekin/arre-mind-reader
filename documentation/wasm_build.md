# WASM Build Guide

## Prerequisites

- Rust with `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`

## Development

```bash
trunk serve --no-default-features
```

Serves at `http://127.0.0.1:8080/` with hot-reload on file changes.

## Release Build

```bash
trunk build --release --no-default-features
```

Produces a `dist/` directory with all static files:
- `index.html`
- `.js` glue code (wasm-bindgen)
- `.wasm` binary
- `assets/` (fonts)

## Hosting

Serve the `dist/` directory from any static file server (Nginx, Caddy, GitHub Pages, etc.).

Ensure `.wasm` files are served with MIME type `application/wasm`. Enable gzip/brotli compression — the `.wasm` file compresses very well (70-80% reduction).

## Architecture Notes

### Cargo Features

- `default = ["native"]` — includes `bevy/dynamic_linking` (faster native rebuilds)
- WASM builds use `--no-default-features` to exclude native-only features
- Platform-specific deps are in `[target.'cfg(...)'.dependencies]` sections:
  - **Native only:** `dirs` (filesystem config directory)
  - **WASM only:** `gloo-storage` (localStorage), `web-sys`

### Platform-Specific Code (`#[cfg]` splits)

- **`persistence.rs`** — Native uses `dirs` + `std::fs`, WASM uses `gloo_storage::LocalStorage`
- **`main.rs`** — `AssetMetaCheck::Never` required to prevent Bevy from fetching nonexistent `.meta` files over HTTP

### Known WASM Limitations

- **File dialog extra confirm:** `rfd` on WASM shows a browser `confirm()` dialog before the file picker. This is because Bevy processes input on the next animation frame, by which point the browser's transient user activation has expired. The confirm dialog re-establishes a user gesture. This is expected behavior.
- **File dialog cancel:** The `rfd` WASM future may not resolve when the user cancels the file picker. The Cancel button in the New Tab dialog drops the pending task to recover.
- **First build is slow:** Compiling the full Bevy dependency tree for `wasm32` takes ~10 minutes. Incremental rebuilds are fast (~10s).

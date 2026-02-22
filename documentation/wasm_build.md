# WASM Build Guide

## Prerequisites

- Rust with `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`

## Development

```bash
trunk serve --no-default-features
```

Serves at `http://127.0.0.1:8080/` with hot-reload on file changes.

For local testing with the configured subpath, use:

```bash
trunk serve --no-default-features --public-url /arre-mind-reader/webapp/
```

## Release Build

```bash
trunk build --release --no-default-features
```

Produces a `dist/` directory with all static files:
- `index.html`
- `.js` glue code (wasm-bindgen)
- `.wasm` binary
- `assets/` (fonts)

`Trunk.toml` sets `public_url = "/arre-mind-reader/webapp/"`, so generated asset URLs are absolute from that subpath.

## Deployment Artifacts (copy all)

Copy the full contents of `dist/` into your server path for `/arre-mind-reader/webapp/`.

Minimum expected files after build:
- `dist/index.html`
- `dist/arre_mind_reader-*.js`
- `dist/arre_mind_reader-*_bg.wasm`
- `dist/assets/**` (all font files)

Do not omit `assets/`, otherwise runtime font loading fails.

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

### Optimization Notes

- Use `wasm_opt` in versions 126 or higher
- `wasm_opt` run is enabled by the index.html code: `<link data-trunk rel="rust" href="Cargo.toml" data-wasm-opt="z" data-wasm-opt-params="--enable-reference-types --enable-bulk-memory" />`
- Use `-v` flag for trunk build to see details and whether `wasm_opt` is being used


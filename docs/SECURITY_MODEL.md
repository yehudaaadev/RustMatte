# RustMatte Security Model (Tauri 2 Desktop App)

Status: Phase 3A Foundation (`crates/app`)

## 1. Threat Boundaries and Principles
- **Local-only processing**: All image decoding, ONNX Runtime inference, edge refinement, and file export execute in Rust inside `rustmatte-core` and `rustmatte-app`. The webview never reads arbitrary files directly and never performs inference.
- **Zero telemetry and zero background network**: The webview makes zero outbound network requests. The only network operation in the entire application is explicit, user-consented model weight download in Rust (`rustmatte_core::download::download_model`), verified against pinned SHA-256 checksums.
- **No full-resolution image data over IPC**: Full-resolution images never cross the Tauri IPC bridge as Base64 or raw byte buffers. Only bounded preview thumbnails (`<= 1600 px` longest side) written to `$APPCACHE/previews/` are exposed to the webview via Tauri's `asset:` protocol.

## 2. Content Security Policy (CSP)
Configured in `crates/app/src-tauri/tauri.conf.json`:
- `default-src 'self'`: Restricts all default resource loads to bundled local static assets.
- `connect-src ipc: http://ipc.localhost https://ipc.localhost`: Permits only Tauri IPC calls; blocks `fetch`, XHR, and WebSocket connections to any remote host.
- `img-src 'self' asset: http://asset.localhost https://asset.localhost blob: data:`: Permits local bundled images and Tauri's scoped asset protocol for bounded preview thumbnails.
- `script-src 'self'`: Disallows remote scripts and `'unsafe-eval'` (`eval()` / dynamic code execution). Tauri injects build-time cryptographic hashes/nonces for bundled SvelteKit assets.
- `style-src 'self' 'unsafe-inline'`: Allows local stylesheets and Svelte scoped/reactive inline styles without external CSS CDNs.
- `font-src 'self'`, `object-src 'none'`, `frame-src 'none'`, `base-uri 'self'`: Blocks external fonts, plugins, iframes, and base tag hijacking.

## 3. Capabilities and Permissions
Configured in `crates/app/src-tauri/capabilities/default.json` for window `["main"]`:

| Permission | Reason Enabled |
| :--- | :--- |
| `core:default` | Required baseline Tauri 2 window, event, and core IPC permissions. |
| `dialog:allow-open` | Allows the user to select input image files or a batch output directory via the native OS file picker. |
| `dialog:allow-save` | Allows the user to choose the destination path when exporting a full-resolution transparent PNG. |

Explicitly **omitted**:
- `tauri-plugin-shell` (not installed; folder opening uses a strictly validated canonical directory check in Rust `open_output_folder` directly invoking the OS file manager without shell interpolation).
- `tauri-plugin-fs` (not installed; frontend never gets direct filesystem read/write permissions).
- `devtools` feature (not enabled in `crates/app/src-tauri/Cargo.toml`; devtools remain disabled in release builds).

## 4. Asset Protocol Scope & Preview Cache Hygiene
- `tauri` enables only the `protocol-asset` feature.
- `app.security.assetProtocol` in `tauri.conf.json` restricts `scope.allow` strictly to `["$APPCACHE/previews", "$APPCACHE/previews/**/*"]`. Arbitrary filesystem paths cannot be fetched via `asset:`.
- Preview cache size is bounded (`128 MB` / `80` files max via LRU eviction) and wiped both on startup and on `RunEvent::Exit`.

## 5. Backend Path Validation & Atomic Writes
Every path received from the frontend is validated in `crates/app/src-tauri/src/validation.rs`:
- Input paths must exist, canonicalize cleanly, refer to a regular file, and carry a supported image extension (`jpg`, `jpeg`, `png`, `webp`, `bmp`, case-insensitive).
- Output save paths and batch output paths are normalized/canonicalized and rejected if they match any canonicalized input file path (preventing accidental overwrite of source photos).
- Settings (`settings.json` in `app_config_dir`) and PNG exports write to temporary `.tmp` files and commit via atomic `rename`.

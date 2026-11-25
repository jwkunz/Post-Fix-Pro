# Post Fix Pro

![Post_Fix_Pro Logo](web/Post_Fix_Pro.png)

A browser-based Reverse Polish Notation (RPN) calculator with:

- Rust backend calculator engine
- WebAssembly runtime for browser execution
- Thin JavaScript UI wrapper
- Dark-mode responsive interface with scalar, matrix, scientific, complex, and memory tools

**Use it online:** <https://jwkunz.github.io/Post-Fix-Pro/>. That is the single-file web build, served by GitHub Pages and redeployed on every push to `master` (`.github/workflows/pages.yml`). The same build is bundled into the Android app and attached to each GitHub release.

## Highlights

- Core calculator logic is implemented in Rust (`src/lib.rs`)
- API + wasm bindings are exposed from `src/api.rs`
- `Post_Fix_Pro.html` hosts the UI and calls wasm methods
- `dist/` can be zipped and shared as a standalone package

## Repository Layout

- `web/` the calculator
  - `web/src/` Rust calculator engine and API bindings
  - `web/Post_Fix_Pro.html` canonical UI source (edit this file)
  - `web/help.md` in-app user help
  - `web/scripts/` wasm build, `dist/` packaging and single-file packaging
  - `web/dist/` generated output (git-ignored)
- `Android/` Gradle project: a WebView app that bundles the single-file build of `web/` (works offline)
- `scripts/version.mjs` keeps the version in step everywhere
- `VERSION` the one place the project version lives
- `docs/` build notes, privacy policy
- `.github/workflows/` CI (tests, single HTML, debug APK, releases) and GitHub Pages deploy

## Prerequisites

- Rust toolchain (with `wasm32-unknown-unknown` target available)
- `wasm-pack`  
  Install guide: <https://rustwasm.github.io/wasm-pack/installer/>

## Build and Launch

### 1. Build release wasm + dist package

```bash
cd web && ./scripts/package_dist_folder.sh
```

This generates/refreshes:

- `pkg/` wasm bindings
- `dist/Post_Fix_Pro.html`
- `dist/pkg/`
- `dist/wasm_base64.js`
- `dist/help.md`
- `dist/README.txt`

`web/Post_Fix_Pro.html` is the single source of truth for the UI. The packaging script copies it to `web/dist/Post_Fix_Pro.html`; edit only that file.

### 2. Run the app

Primary flow:

1. Open `web/dist/Post_Fix_Pro.html` in your browser.

Fallback for stricter browser file policies:

1. Start a static server in `web/dist/`.
2. Open the served `Post_Fix_Pro.html`.

Example server command:

```bash
cd web/dist
python3 -m http.server 8000
```

Then browse to: `http://localhost:8000`

### 3. Optional single-file package

```bash
cd web && ./scripts/single_file_package.sh
```

This creates `web/post_fix_pro_vX_Y_Z.html`, using the version in `VERSION`. It is a generated release artifact and is intentionally not tracked by Git.

## Development Loop

- Edit Rust engine/API files in `web/src/`
- Edit UI in `web/Post_Fix_Pro.html`
- Run tests:

```bash
cd web && cargo test
```

- Rebuild wasm and refresh dist:

```bash
cd web && ./scripts/package_dist_folder.sh
```

## Android

Needs JDK 21, the Android SDK, Rust and `wasm-pack`:

```bash
cd Android
./gradlew :app:assembleDebug     # builds web/ into one HTML file, then the APK
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

Pass `-PskipWebBuild` to reuse an existing `web/post_fix_pro_v*.html`.

## Versioning and releases

```bash
node scripts/version.mjs set 6.2.0   # VERSION, web/Cargo.toml, web/Cargo.lock (also update the version text in the UI HTML)
git tag v6.2.0 && git push --tags     # CI publishes post-fix-pro-v6.2.0.html and the debug APK as a GitHub release
```

## Usage Notes

- RPN workflow: enter number, press `Enter`, then apply operators.
- Stack display is bottom-first with top-of-stack highlighted.
- A `Help` button in the app loads `help.md` in an overlay.

## License

MIT. See [LICENSE](LICENSE).

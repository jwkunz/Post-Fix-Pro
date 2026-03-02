# Post_Fix_Pro

A browser-based Reverse Polish Notation (RPN) calculator with:

- Rust backend calculator engine
- WebAssembly runtime for browser execution
- Thin JavaScript UI wrapper
- Dark-mode responsive interface with scalar, matrix, scientific, complex, and memory tools

## Highlights

- Core calculator logic is implemented in Rust (`src/lib.rs`)
- API + wasm bindings are exposed from `src/api.rs`
- `Post_Fix_Pro.html` hosts the UI and calls wasm methods
- `dist/` can be zipped and shared as a standalone package

## Repository Layout

- `src/` Rust calculator engine and API bindings
- `Post_Fix_Pro.html` main application UI
- `scripts/build_wasm.sh` production build + distribution packaging
- `docs/build_notes/` milestone notes by version
- `docs/wasm_build.md` build and distribution notes
- `help.md` in-app user help
- `dist/` distributable output

## Prerequisites

- Rust toolchain (with `wasm32-unknown-unknown` target available)
- `wasm-pack`  
  Install guide: <https://rustwasm.github.io/wasm-pack/installer/>

## Build and Launch

### 1. Build release wasm + dist package

```bash
./scripts/build_wasm.sh
```

This generates/refreshes:

- `pkg/` wasm bindings
- `dist/Post_Fix_Pro.html`
- `dist/pkg/`
- `dist/wasm_base64.js`
- `dist/help.md`
- `dist/README.txt`

### 2. Run the app

Primary flow:

1. Open `dist/Post_Fix_Pro.html` in your browser.

Fallback for stricter browser file policies:

1. Start a static server in `dist/`.
2. Open the served `Post_Fix_Pro.html`.

Example server command:

```bash
cd dist
python3 -m http.server 8000
```

Then browse to: `http://localhost:8000`

## Development Loop

- Edit Rust engine/API files in `src/`
- Edit UI in `Post_Fix_Pro.html`
- Run tests:

```bash
cargo test
```

- Rebuild wasm and refresh dist:

```bash
./scripts/build_wasm.sh
```

## Usage Notes

- RPN workflow: enter number, press `Enter`, then apply operators.
- Stack display is bottom-first with top-of-stack highlighted.
- A `Help` button in the app loads `help.md` in an overlay.

## License

MIT. See [LICENSE](LICENSE).

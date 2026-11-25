# WASM Build Path

Run from the `web/` folder:

```bash
./scripts/package_dist_folder.sh
```

This command:
- runs `wasm-pack build --target no-modules --out-dir pkg`
- copies wasm bindings to `web/dist/pkg/`
- refreshes `web/dist/Post_Fix_Pro.html`
- copies `help.md` to `web/dist/help.md`
- generates `web/dist/wasm_base64.js` (embedded wasm bytes for local-file startup)
- writes `web/dist/README.txt`

## Distribution Flow

1. Run `./scripts/package_dist_folder.sh`
2. Zip the `web/dist/` folder
3. Share the zip
4. Recipient unzips and opens `web/dist/Post_Fix_Pro.html`

Expected `web/dist/` contents:
- `Post_Fix_Pro.html`
- `pkg/` (generated glue + wasm artifacts)
- `wasm_base64.js`
- `help.md`
- `README.txt`

If `wasm-pack` is missing, install instructions:
- https://rustwasm.github.io/wasm-pack/installer/

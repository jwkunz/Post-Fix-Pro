# WASM Build Path

Use the repository script:

```bash
./scripts/package_dist_folder.sh
```

This command:
- runs `wasm-pack build --target no-modules --out-dir pkg`
- copies wasm bindings to `dist/pkg/`
- refreshes `dist/Post_Fix_Pro.html`
- copies `help.md` to `dist/help.md`
- generates `dist/wasm_base64.js` (embedded wasm bytes for local-file startup)
- writes `dist/README.txt`

## Distribution Flow

1. Run `./scripts/package_dist_folder.sh`
2. Zip the `dist/` folder
3. Share the zip
4. Recipient unzips and opens `dist/Post_Fix_Pro.html`

Expected `dist/` contents:
- `Post_Fix_Pro.html`
- `pkg/` (generated glue + wasm artifacts)
- `wasm_base64.js`
- `help.md`
- `README.txt`

If `wasm-pack` is missing, install instructions:
- https://rustwasm.github.io/wasm-pack/installer/

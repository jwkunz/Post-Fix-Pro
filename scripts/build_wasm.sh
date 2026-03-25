#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "error: wasm-pack is required but not installed." >&2
  echo "install: https://rustwasm.github.io/wasm-pack/installer/" >&2
  exit 1
fi

wasm-pack build --target no-modules --out-dir pkg

mkdir -p dist/pkg
cp -r pkg/. dist/pkg/
cp Post_Fix_Pro.html dist/Post_Fix_Pro.html
cp Post_Fix_Pro.png dist/Post_Fix_Pro.png
cp VERSION dist/VERSION
cp help.md dist/help.md
rm -f dist/index.html

WASM_PATH="dist/pkg/webcalculator_backend_bg.wasm"
BASE64_PATH="dist/wasm_base64.js"

if [[ ! -f "${WASM_PATH}" ]]; then
  echo "error: expected wasm file not found at ${WASM_PATH}" >&2
  exit 1
fi

{
  echo "window.__WASM_BASE64 = \"\\"
  base64 -w 0 "${WASM_PATH}"
  echo "\";"
} > "${BASE64_PATH}"

cat > dist/README.txt <<'EOF'
Distribution folder for Post_Fix_Pro.

Open Post_Fix_Pro.html directly in a browser.
If your browser blocks local file WASM execution, run a local static server from this folder.
EOF

echo "WASM build complete:"
echo "  - pkg/ bindings generated (no-modules target)"
echo "  - dist/Post_Fix_Pro.html refreshed"
echo "  - dist/Post_Fix_Pro.png copied"
echo "  - dist/VERSION copied"
echo "  - dist/help.md copied"
echo "  - dist/pkg/ copied"
echo "  - dist/wasm_base64.js generated"

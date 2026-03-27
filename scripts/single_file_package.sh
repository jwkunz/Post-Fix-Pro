#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
BUILD_FIRST=1

usage() {
  cat <<'EOF'
Usage: ./scripts/single_file_package.sh [--no-build]

Builds the distributable app, then packages dist/ into a single self-contained
HTML file in the project root:

  post_fix_pro_vX_Y_Z.html

Options:
  --no-build    Skip ./scripts/package_dist_folder.sh and package the current dist/ tree
EOF
}

while (($# > 0)); do
  case "$1" in
    --no-build)
      BUILD_FIRST=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

require_file() {
  local path="$1"
  if [[ ! -f "${path}" ]]; then
    echo "error: required file not found: ${path}" >&2
    exit 1
  fi
}

if [[ "${BUILD_FIRST}" -eq 1 ]]; then
  "${ROOT_DIR}/scripts/package_dist_folder.sh"
fi

require_file "${DIST_DIR}/Post_Fix_Pro.html"
require_file "${DIST_DIR}/Post_Fix_Pro.png"
require_file "${DIST_DIR}/help.md"
require_file "${DIST_DIR}/VERSION"
require_file "${DIST_DIR}/pkg/webcalculator_backend.js"
require_file "${DIST_DIR}/pkg/webcalculator_backend_bg.wasm"

VERSION="$(tr -d '[:space:]' < "${DIST_DIR}/VERSION")"
if [[ -z "${VERSION}" ]]; then
  echo "error: dist/VERSION is empty" >&2
  exit 1
fi
OUTPUT_PATH="${ROOT_DIR}/post_fix_pro_v${VERSION//./_}.html"

TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

HELP_HTML_PATH="${TMP_DIR}/help.html"
PNG_DATA_URL_PATH="${TMP_DIR}/png_data_url.txt"
INLINE_RUNTIME_JS_PATH="${TMP_DIR}/inline_runtime.js"
INLINE_ASSETS_JS_PATH="${TMP_DIR}/inline_assets.js"

awk '
function escape_html(text, value) {
  value = text
  gsub(/&/, "\\&amp;", value)
  gsub(/</, "\\&lt;", value)
  gsub(/>/, "\\&gt;", value)
  return value
}

function slugify(text, value) {
  value = tolower(text)
  gsub(/[^a-z0-9]+/, "-", value)
  gsub(/^-+/, "", value)
  gsub(/-+$/, "", value)
  if (value == "") {
    value = "section"
  }
  return value
}

function flush_pre() {
  if (in_pre) {
    print "</pre>"
    in_pre = 0
  }
}

function print_body_line(text) {
  if (!in_pre) {
    print "<pre>"
    in_pre = 1
  }
  print escape_html(text)
}

BEGIN {
  in_pre = 0
  in_code_fence = 0
  print "<!doctype html>"
  print "<html lang=\"en\">"
  print "<head>"
  print "  <meta charset=\"utf-8\">"
  print "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
  print "  <title>Post Fix Pro Help</title>"
  print "  <style>"
  print "    :root { color-scheme: dark; }"
  print "    body { margin: 0; padding: 24px; background: #0d1420; color: #e9f0fb; font-family: \"Trebuchet MS\", \"Gill Sans\", sans-serif; line-height: 1.5; }"
  print "    h1, h2, h3 { margin: 1.6rem 0 0.7rem; color: #72c6ff; scroll-margin-top: 16px; }"
  print "    h1:first-child { margin-top: 0; }"
  print "    pre { margin: 0 0 1rem; padding: 14px 16px; white-space: pre-wrap; word-break: break-word; border: 1px solid #304561; border-radius: 12px; background: rgba(18, 26, 40, 0.92); color: #e9f0fb; font: 13px/1.5 \"Courier New\", monospace; }"
  print "  </style>"
  print "</head>"
  print "<body>"
}

/^```/ {
  in_code_fence = !in_code_fence
  next
}

!in_code_fence && /^### / {
  flush_pre()
  title = substr($0, 5)
  printf "<h3 id=\"%s\">%s</h3>\n", slugify(title), escape_html(title)
  next
}

!in_code_fence && /^## / {
  flush_pre()
  title = substr($0, 4)
  printf "<h2 id=\"%s\">%s</h2>\n", slugify(title), escape_html(title)
  next
}

!in_code_fence && /^# / {
  flush_pre()
  title = substr($0, 3)
  printf "<h1 id=\"%s\">%s</h1>\n", slugify(title), escape_html(title)
  next
}

{
  print_body_line($0)
}

END {
  flush_pre()
  print "</body>"
  print "</html>"
}
' "${DIST_DIR}/help.md" > "${HELP_HTML_PATH}"

printf 'data:image/png;base64,%s\n' "$(base64 -w 0 "${DIST_DIR}/Post_Fix_Pro.png")" > "${PNG_DATA_URL_PATH}"
cp "${DIST_DIR}/pkg/webcalculator_backend.js" "${INLINE_RUNTIME_JS_PATH}"
{
  printf 'window.__WASM_BASE64 = "%s";\n' "$(base64 -w 0 "${DIST_DIR}/pkg/webcalculator_backend_bg.wasm")"
  printf 'window.__PFP_HELP_MD_BASE64 = "%s";\n' "$(base64 -w 0 "${DIST_DIR}/help.md")"
  printf 'window.__PFP_HELP_HTML_BASE64 = "%s";\n' "$(base64 -w 0 "${HELP_HTML_PATH}")"
} > "${INLINE_ASSETS_JS_PATH}"

perl - "${DIST_DIR}/Post_Fix_Pro.html" \
  "${INLINE_RUNTIME_JS_PATH}" \
  "${INLINE_ASSETS_JS_PATH}" \
  "${PNG_DATA_URL_PATH}" \
  "${OUTPUT_PATH}" <<'PERL'
use strict;
use warnings;

my ($input_html_path, $runtime_js_path, $assets_js_path, $png_data_url_path, $output_html_path) = @ARGV;

sub slurp {
    my ($path) = @_;
    open my $fh, '<', $path or die "unable to read $path: $!";
    local $/;
    return <$fh>;
}

sub escape_script_body {
    my ($text) = @_;
    $text =~ s{</script}{<\\/script}gi;
    return $text;
}

my $html = slurp($input_html_path);
my $runtime_js = escape_script_body(slurp($runtime_js_path));
my $assets_js = escape_script_body(slurp($assets_js_path));
my $png_data_url = slurp($png_data_url_path);
$png_data_url =~ s/\s+\z//;

my $tooltip_loader = <<'JS';
    function decodeInlineText(base64Text) {
      const binary = atob(String(base64Text || ""));
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i += 1) {
        bytes[i] = binary.charCodeAt(i);
      }
      return new TextDecoder().decode(bytes);
    }

    async function loadTooltipMap() {
      try {
        tooltipMap = parseHelpTooltipMap(decodeInlineText(window.__PFP_HELP_MD_BASE64));
      } catch (_) {
        tooltipMap = new Map();
      }
    }
JS

my $tooltip_loader_old = <<'JS';
    async function loadTooltipMap() {
      try {
        const response = await fetch("./help.md", { cache: "no-cache" });
        if (!response.ok) return;
        tooltipMap = parseHelpTooltipMap(await response.text());
      } catch (_) {
        tooltipMap = new Map();
      }
    }
JS

my $help_open = <<'JS';
    function openHelpSection(anchor = "") {
      helpFrame.onload = null;
      helpFrame.srcdoc = decodeInlineText(window.__PFP_HELP_HTML_BASE64 || "");
      helpLoaded = true;
      openModal(helpModal);

      const targetAnchor = String(anchor || "").trim();
      if (!targetAnchor) return;

      helpFrame.onload = () => {
        const target = helpFrame.contentDocument?.getElementById(targetAnchor);
        if (target) {
          target.scrollIntoView({ block: "start" });
        }
        helpFrame.onload = null;
      };
    }
JS

my $help_open_old = <<'JS';
    function openHelpSection(anchor = "") {
      helpFrame.src = anchor ? `./help.md#${anchor}` : "./help.md";
      helpLoaded = true;
      openModal(helpModal);
    }
JS

my $wasm_init_old = <<'JS';
        if (typeof window.__WASM_BASE64 === "string" && window.__WASM_BASE64.length > 0) {
          await wasm_bindgen(decodeBase64ToBytes(window.__WASM_BASE64));
        } else {
          await wasm_bindgen("./pkg/webcalculator_backend_bg.wasm");
        }
JS

my $wasm_init_new = <<'JS';
        if (typeof window.__WASM_BASE64 !== "string" || window.__WASM_BASE64.length === 0) {
          throw new Error("embedded WASM payload is unavailable");
        }
        await wasm_bindgen(decodeBase64ToBytes(window.__WASM_BASE64));
JS

$html =~ s{<script src="\./pkg/webcalculator_backend\.js"></script>}{<script>\n$runtime_js\n</script>}s
    or die "failed to inline webcalculator_backend.js\n";
$html =~ s{<script src="\./wasm_base64\.js"></script>}{<script>\n$assets_js\n</script>}s
    or die "failed to inline wasm/help asset block\n";
$html =~ s{\./Post_Fix_Pro\.png}{$png_data_url}g;
$html =~ s/\Q$tooltip_loader_old\E/$tooltip_loader/s
    or die "failed to replace tooltip loader\n";
$html =~ s/\Q$help_open_old\E/$help_open/s
    or die "failed to replace help modal loader\n";
$html =~ s/\Q$wasm_init_old\E/$wasm_init_new/s
    or die "failed to replace wasm init block\n";

open my $out, '>', $output_html_path or die "unable to write $output_html_path: $!";
print {$out} $html;
close $out or die "unable to close $output_html_path: $!";
PERL

echo "Single-file package created:"
echo "  ${OUTPUT_PATH}"

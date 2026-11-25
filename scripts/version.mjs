#!/usr/bin/env node
// The project's version lives in one file, VERSION, at the repository root.
//   - the Android build reads it directly (versionName, and versionCode = major*10000 + minor*100 + patch);
//   - the web build copies it into dist/ and the Rust crate needs it in Cargo.toml / Cargo.lock, which this script keeps in step.
//
//   node scripts/version.mjs                 print the version
//   node scripts/version.mjs set 6.2.0       change the version everywhere
//   node scripts/version.mjs check [tag]     fail if anything disagrees (and if `tag`, e.g. v6.2.0, is not this version)
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

const root = fileURLToPath(new URL('..', import.meta.url));
const read = (p) => readFileSync(join(root, p), 'utf8');
const SEMVER = /^\d+\.\d+\.\d+$/;
const CRATE = 'webcalculator_backend';
const MANIFEST_RE = /^(version = ")[^"]*(")/m;
const LOCK_RE = new RegExp(`(name = "${CRATE}"\\r?\\nversion = ")[^"]*(")`);

export const currentVersion = () => read('VERSION').trim();

function mismatches(version) {
  const bad = [];
  const manifest = read('web/Cargo.toml').match(MANIFEST_RE);
  if (!manifest || manifest[0] !== `version = "${version}"`) bad.push("web/Cargo.toml version differs");
  const lock = read('web/Cargo.lock').match(new RegExp(`name = "${CRATE}"\\r?\\nversion = "([^"]*)"`));
  if (!lock || lock[1] !== version) bad.push(`web/Cargo.lock says ${lock && lock[1]}`);
  if (!/rootProject\.file\("\.\.\/VERSION"\)/.test(read('Android/app/build.gradle.kts'))) bad.push('Android/app/build.gradle.kts does not read VERSION');
  return bad;
}

const [cmd, arg] = process.argv.slice(2);
if (!cmd) {
  console.log(currentVersion());
} else if (cmd === 'set') {
  if (!SEMVER.test(arg || '')) { console.error('Usage: node scripts/version.mjs set MAJOR.MINOR.PATCH'); process.exit(2); }
  writeFileSync(join(root, 'VERSION'), `${arg}\n`);
  writeFileSync(join(root, 'web/Cargo.toml'), read('web/Cargo.toml').replace(MANIFEST_RE, `$1${arg}$2`));
  writeFileSync(join(root, 'web/Cargo.lock'), read('web/Cargo.lock').replace(LOCK_RE, `$1${arg}$2`));
  console.log(`Version is now ${arg} (VERSION, web/Cargo.toml and web/Cargo.lock). Also update the "Version" text in web/Post_Fix_Pro.html.`);
} else if (cmd === 'check') {
  const v = currentVersion();
  const problems = SEMVER.test(v) ? mismatches(v) : [`VERSION is "${v}", expected MAJOR.MINOR.PATCH`];
  if (arg && arg !== `v${v}`) problems.push(`the tag ${arg} does not match VERSION ${v} (expected v${v})`);
  if (problems.length) { console.error(`Version problems:\n  - ${problems.join('\n  - ')}`); process.exit(1); }
  console.log(`All version files agree: ${v}${arg ? ` (tag ${arg})` : ''}`);
} else {
  console.error('Usage: node scripts/version.mjs [set X.Y.Z | check [tag]]'); process.exit(2);
}

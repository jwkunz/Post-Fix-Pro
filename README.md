# Post Fix Pro

![Post_Fix_Pro Logo](web/Post_Fix_Pro.png)

A Reverse Polish Notation (RPN) calculator with scalar, matrix, scientific, complex, statistics and memory tools, plus a small scripting language. The calculator engine is written in Rust and runs as WebAssembly behind a dark-mode, responsive HTML interface. Everything runs on your device and works offline.

## Get it

| Where | What |
|-------|------|
| **[jwkunz.github.io/Post-Fix-Pro](https://jwkunz.github.io/Post-Fix-Pro/)** | The web app, always the latest `master` (deployed by GitHub Pages) |
| **[Releases](https://github.com/jwkunz/Post-Fix-Pro/releases)** | `post-fix-pro-vX.Y.Z.html`, a single self-contained file you can save and open locally, and `post-fix-pro-vX.Y.Z-debug.apk`, the Android app |

## Calculator features

Post Fix Pro is a stack calculator: push values, then apply operators. Operands are typed on the scalar keypad or built in the matrix editor, and `undo` reverses the last operation. Real, complex and matrix values share one stack. The in-app **Help** button has the full reference (also in [web/help.md](web/help.md)).

- **Stack:** `enter`, `drop`, `dup`, `swap`, `rot`, `pick`, undo, and a degrees/radians toggle.
- **Arithmetic:** add, subtract, multiply, divide, reciprocal, square, roots, powers, `exp`/`ln`, base-2 and base-10 logs, arbitrary-base log, percent and signum.
- **Trigonometry:** sin, cos, tan, sec, csc and cot with their inverses, a hyperbolic mode, `atan2`, and degree/radian conversion.
- **Complex numbers:** magnitude, phase, conjugate, real and imaginary parts, and composing or decomposing in rectangular, polar and normalized-polar form.
- **Matrices and vectors:**
  - Build them in a matrix editor (with presets and identity), import and export CSV, and stack or unstack with `hstack`, `vstack`, `hravel` and `vravel`.
  - Operators: dot and cross products, determinant, trace, transpose, inverse, Hermitian, matrix power and exponential, p-norms, and element-wise multiply and divide.
  - Linear solves: `A x = B` and least squares.
  - Decompositions: QR, LU, SVD and eigendecomposition.
- **Special functions and number theory:** factorial, combinations, permutations, modulo, GCD, LCM, gamma, error functions, Bessel J0 and I0, and sinc.
- **Statistics:** mean, median, mode, quartiles, variance, population and sample standard deviation, min, max and random numbers.
- **Rounding:** round, floor, ceiling and decimal part.
- **Constants:** pi, e, Euler-Mascheroni, golden ratio, speed of light, Avogadro, Boltzmann, reduced Planck, vacuum permittivity and permeability, gravitational constant and electron charge.
- **Memory:** store, recall and clear values in registers.
- **Scripting:** run RPN command streams with comments, real, complex and matrix literals, and symbol aliases such as `+` and `*`. Errors report the line and column. See the Scripting Guide in the help.

## Repository layout

```
VERSION                  the one place the project version lives
web/                     the calculator (Rust engine + HTML UI)
  src/                     Rust engine, wasm API bindings, scripting, tests
  Post_Fix_Pro.html        the UI source (edit this)
  Post_Fix_Pro.png         logo
  help.md                  in-app help
  scripts/                 package_dist_folder.sh, single_file_package.sh
Android/                 Gradle project: a WebView app that bundles the web build
scripts/version.mjs      keeps VERSION, Cargo.toml and Cargo.lock in step
docs/                    build notes per version, wasm build notes, privacy policy
.github/workflows/       ci.yml (tests, builds, releases) and pages.yml (site deploy)
```

The web app is edited in exactly one place, `web/`. Both the GitHub Pages site and the Android app are built from it, so a push to `master` updates the site, and the next APK build picks up the same code.

## How it fits together

1. `wasm-pack` compiles the Rust crate in `web/` to WebAssembly.
2. `web/scripts/single_file_package.sh` inlines the wasm, JS glue, help text and logo into one HTML file, `web/post_fix_pro_vX_Y_Z.html` (git-ignored).
3. That file is:
   - copied to the site by `pages.yml`,
   - bundled by Gradle into the Android app as `assets/index.html`, which `MainActivity` loads in a WebView,
   - attached to each GitHub release.

## Building locally

**Web.** Needs Rust with the `wasm32-unknown-unknown` target and [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/).

```bash
cd web
cargo test                          # Rust tests
./scripts/single_file_package.sh    # builds web/post_fix_pro_vX_Y_Z.html; open it in a browser
./scripts/package_dist_folder.sh    # alternative: a multi-file web/dist/ folder (see docs/wasm_build.md)
```

**Android.** Needs JDK 21, the Android SDK, and the web prerequisites above.

```bash
cd Android
./gradlew :app:assembleDebug        # builds the web page first, then the APK
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

Pass `-PskipWebBuild` to reuse an existing `web/post_fix_pro_v*.html`. The APK is a debug build; a Play Store upload needs a signed release build, which is not set up here.

## Continuous integration

- **`ci.yml`** runs on pushes to `master`, tags and pull requests: checks the version files agree, runs `cargo test`, builds the single HTML file (and verifies it has no external references), and builds the debug APK.
- **On a `v*` tag** it also publishes a GitHub release containing the HTML file and the APK.
- **`pages.yml`** builds the single HTML file and deploys it to GitHub Pages on every push to `master`.

## Versioning and releasing

```bash
node scripts/version.mjs                # print the version
node scripts/version.mjs set 6.2.0      # update VERSION, web/Cargo.toml, web/Cargo.lock
node scripts/version.mjs check          # fail if they disagree
```

Also update the "Version" text in `web/Post_Fix_Pro.html`, add notes under `docs/build_notes/`, then commit and tag:

```bash
git tag v6.2.0 && git push origin v6.2.0
```

CI refuses to release if the tag doesn't match `VERSION`. The Android `versionName` and `versionCode` (`major*10000 + minor*100 + patch`) are derived from `VERSION`.

## Privacy

The app has no ads, analytics, accounts or network access. See [docs/Post_Fix_Pro_privacy_policy.md](docs/Post_Fix_Pro_privacy_policy.md).

## License

MIT. See [LICENSE](LICENSE).

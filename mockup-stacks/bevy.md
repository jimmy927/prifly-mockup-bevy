---
id: bevy
name: "Bevy (Rust game engine, wasm build)"
aliases: [bevy, bevy engine, bevy_ui, rust game, bevy wasm]
detect:
  deps: ["bevy"]
  files: ["Trunk.toml"]
route: wasm
kind: game
where: local
toolchain: "rustc/cargo 1.98.1 + wasm32-unknown-unknown, wasm-bindgen-cli 0.2.129 (cargo install --root ~/.cache/mockup-toolchains/wasm-bindgen-0.2.129), binaryen wasm-opt 133 (release tarball, optional), kit pack-wasm.py generic"
versions: "bevy 0.19.1 (features 2d + ui, webgl2), wasm-bindgen 0.2.129, binaryen 133"
renderedBy: ["<scratchpad>/bevy-game", "~/.cache/mockup-toolchains/wasm-bindgen-0.2.129/bin"]
verified: 2026-10-01
verifiedBy: "verified by prifly sessions on 2026-10-01"
fixture: fixtures/bevy
---

## Steps
Proof project: `fixtures/bevy/` (Star Catcher: sprites, a bevy_ui HUD and Game over panel, two `States`). Copy it, or add the same pieces to the project's own crate.
1. `Cargo.toml`: `bevy = { version = "0.19.1", default-features = false, features = ["2d", "ui"] }` (no 3d, audio, gilrs), a `cfg(target_arch = "wasm32")` dependency on `web-sys` (`Window`, `Location`) and `js-sys`, and `[profile.release] opt-level = "z", lto = "fat", codegen-units = 1, panic = "abort", strip = "debuginfo"`. Keep `Cargo.lock`.
2. Window: `Window { canvas: Some("#bevy-canvas".into()), fit_canvas_to_parent: true, prevent_default_event_handling: false, .. }`. The page owns the canvas and gives it a fixed-size frame; use `ScalingMode::FixedVertical` on the camera so the world fits whatever the frame is.
3. Toolchain once: `cargo install wasm-bindgen-cli --version <the wasm-bindgen in Cargo.lock> --locked --root ~/.cache/mockup-toolchains/wasm-bindgen-<ver> -j 8` (65 s). Optional: binaryen release tarball (`binaryen-version_133-x86_64-linux.tar.gz`) unpacked into `~/.cache/mockup-toolchains/binaryen-133`.
4. Build: `CARGO_TARGET_DIR=<scratchpad>/target nice cargo build --release --target wasm32-unknown-unknown -j 8` (cold, bevy and all its crates: 8 m 17 s on 16 cores with -j 8; warm rebuild of only the game crate with fat LTO: about 1 minute). Raw `.wasm`: 43 MB.
5. Glue: `wasm-bindgen --target no-modules --no-typescript --remove-name-section --remove-producers-section --out-dir dist/pkg <target>/wasm32-unknown-unknown/release/<name>.wasm`, then `wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals --enable-reference-types --enable-simd -o X X`. Result 17.3 MB wasm (43 MB raw, 19.9 MB of that was the name section).
6. Pack: `mockup-kit pack-wasm generic --wasm dist/pkg/<name>_bg.wasm --script dist/pkg/<name>.js --boot boot.js --no-reload-on-hash --out dist/page.html` -> 7.3 MB single file (gzip+base64 wasm; fetch and instantiateStreaming answer from the embedded bytes). `mockup-kit check` passes. The glue file is a classic script (`no-modules`), so it inlines; `--target web` would need the module route.
7. `boot.js` (in the fixture) builds the page chrome (a mockup-control bar outside the frame, a 1024x576 frame holding `<canvas id="bevy-canvas">`), sets `window.__mockupReady = false`, starts `wasm_bindgen({ module_or_path: await __mockupBytes('<name>_bg.wasm') })` and sets it true once the game has reported 8 frames in `window.__bevyFrames` (a Rust system writes it through `js_sys::Reflect`).
8. Post with `mcp__prifly__mockup`: `check: true` first, then once, `renderedBy: [<project folder>, <wasm-bindgen bin folder>]`, title "Bevy mockup — <scene>". Whole fixture, build to post: about 11 minutes cold, 25 s to repack.

## Gotchas
- `wasm-bindgen-cli` must match the `wasm-bindgen` crate version in `Cargo.lock` exactly (0.2.129 here), or it refuses the module. Read it from the lock after the first build.
- The raw release wasm is 43 MB mostly because of the name section; `--remove-name-section` plus `wasm-opt -Oz` give 17 MB and a 7.3 MB page. Without wasm-opt the page works, just larger.
- `wasm-opt` needs the `--enable-*` flags above for bulk-memory, sign-ext, simd and reference types, or it rejects Rust's output.
- Winit used to end start-up by throwing "Using exceptions for control flow"; `boot.js` ignores an error that matches /control flow/ and shows any other.
- Bevy has no JS hook for "first frame": report it yourself (`window.__bevyFrames`) and flip `__mockupReady` from the page. The kit's own ready promise is replaced by `false` then `true`.
- WebGL2 worked in the picture step (`webgl2` comes with the `2d`/`ui` platform set); no WebGPU needed.
- The scene is live, so the picture is a moment of a running game: score and positions differ a little between loads (score 130, 150). Drive the game by a fixed step per frame and an LCG, never by wall time or `rand`, so it is the same kind of picture every time. For an exact still, stop the simulation after N frames.
- Never let the cargo build run with more than `-j 8` here; fat LTO plus codegen-units 1 is serial at the end, so the last minute uses one core.
- Keep the build cache outside the repo (`CARGO_TARGET_DIR`); a bevy target dir is several GB.
- Setting `location.hash` from a page script before the wasm starts is how a state is forced for a `check: true` run, because the tool accepts a file path without a `#fragment`.

## States
The game reads `location.hash` every frame (web-sys): `#playing` (default, mid-game demo numbers: score 120, 2 lives) and `#gameover` (panel with Retry, score 240). A hash change switches state without a reload (`--no-reload-on-hash`). The two links in the bar above the frame are the labelled mockup control. Retry is a real `bevy_ui` Button that restarts the round. To check `#gameover` with the tool, make a copy of the page with `<script>location.hash="#gameover"</script>` after `<body>`.

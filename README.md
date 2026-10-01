# prifly-mockup-bevy

A prifly mockup recipe for **Bevy**, the Rust game engine, built to wasm. prifly
does not cover Bevy itself; this extension adds it. A mockup of a Bevy scene is rendered by real Bevy (compiled to
`wasm32-unknown-unknown`, drawn through WebGL2) and packaged as one self-contained HTML file.

## Use it in prifly

This repo is a prifly extension. In prifly, open **Extensions**, install from the git URL
`https://github.com/jimmy927/prifly-mockup-bevy.git`, then enable it. Sessions then get the recipe from
`mcp__prifly__mockup_recipe` (layer order: project `.prifly/mockup-stacks/` > user folder > enabled extensions >
built-in). A project can also copy `mockup-stacks/bevy.md` into its own `.prifly/mockup-stacks/`.

## The recipe in one paragraph

Build the project with `cargo build --release --target wasm32-unknown-unknown` (features `2d` + `ui`, size-optimised
release profile), run `wasm-bindgen --target no-modules --remove-name-section`, shrink with binaryen's `wasm-opt -Oz`,
and pack the wasm and the glue with prifly's `mockup-kit pack-wasm generic` plus a small `boot.js` that creates the
canvas, starts the module from the embedded bytes and sets `window.__mockupReady` after the game reports its first
frames. The proof project (a 2D arcade scene: sprites, a `bevy_ui` HUD, two `States`) builds to a 7.3 MB page; a cold
build takes about 8 minutes of cargo, a repack 25 seconds. States are chosen by `location.hash`. Details, timings and
gotchas are in [`mockup-stacks/bevy.md`](mockup-stacks/bevy.md).

## Layout

- `mockup-stacks/bevy.md`: the recipe (frontmatter validated by `scripts/check.ts`).
- `mockup-stacks/fixtures/bevy/`: the proof project (`build.sh` checks the manifest; `FULL=1 ./build.sh` builds the page).
- `scripts/check.ts`: bun-only validation of every recipe, used by CI.

## Check

```
bun scripts/check.ts
```

From a prifly checkout, `bun run mockup-stacks:check --dir <path-to-this-repo>/mockup-stacks` runs the full checker on this folder.

## Screenshots

![The prifly card of the Bevy mockup](docs/prifly-card.png)

![The Bevy mockup opened](docs/prifly-open.png)

## License

MIT, see [LICENSE](LICENSE). Bevy is MIT or Apache-2.0.

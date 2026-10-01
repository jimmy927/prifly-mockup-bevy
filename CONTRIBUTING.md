# Contributing

Recipes here are for stacks prifly's built-in set does not cover, written from a mockup that was really built and posted.

- One file per stack in `mockup-stacks/`, named `<id>.md`, with the frontmatter prifly's loader expects: `id`, `name`,
  `aliases`, `detect` (`deps`, `files`, `globs`), `route`, `verified` (YYYY-MM-DD), and ideally `kind`, `where`,
  `toolchain`, `versions`, `renderedBy`, `verifiedBy`, `fixture`.
- Body sections `## Steps`, `## Gotchas`, `## States`, from what actually worked, with versions, timings and sizes.
  No invented steps.
- A `fixture` is a small proof project under `mockup-stacks/fixtures/<id>/` with a `build.sh` (second line
  `# needs: <programs>`). Source only: no `target/`, `node_modules`, `dist/` or binaries.
- A mockup is rendered by the real stack, never drawn by hand with HTML, CSS or SVG.
- Nothing private: no home-directory paths (use `~` or `<scratchpad>`), machine names, IPs, tokens, session ids.
- Run `bun scripts/check.ts` before opening a pull request; CI runs the same.

# diagctl walking skeleton — design

**Date:** 2026-06-06 **Status:** approved (stack + scope locked with user
2026-06-06) **Tracks:**
[metapowers#14](https://github.com/driftsys/metapowers/issues/14) (full design)
**Boundary:** [metapowers#38](https://github.com/driftsys/metapowers/issues/38)
(vision self-check — OUT OF SCOPE)

## Goal

Scaffold `diagctl` into a **walking skeleton**: minimal end-to-end structure
that proves the chosen architecture, not the full tool. One real vertical slice
through `check`, the heavy SVG-parsing dependency build-proven in CI,
cross-platform release binaries wired, suite green. The geometry-heavy Layer-2
checks and the `optimize`/`ascii`/`freshness` bodies are explicitly the **next**
milestone.

## Stack decision — Rust

Resolved the open `#14` fork (Rust vs Deno/TS) in favour of **Rust**.

- The deciding fork was `optimize` (SVGO): JS-native, no battle-tested Rust
  port. This skeleton **defers `optimize` to a stub**, neutralising Rust's only
  weakness.
- Everything load-bearing favours Rust: the Layer-2 geometry checks (the
  differentiating value, next milestone) build on **usvg/resvg**
  (transform-resolved geometry, accurate bounding boxes) + **lyon** (path
  intersection); a static binary is ~2–8 MB vs `deno compile`'s ~80–100 MB;
  category fit with dprint/typos/biome/oxc.
- `optimize`, when it lands, uses `oxvg`/`usvg`-normalize — reimplemented, never
  glued to a Node `svgo` subprocess.

### Alternatives considered

- **Deno/TS** — would make `optimize` exact (real `npm:svgo`) but makes the
  geometry checks (the actual differentiator) the weak spot (hand-rolled
  bbox/transform math or headless Chrome `getBBox`) and ships an ~80–100 MB
  binary. Rejected because the heart of the tool is `check`/geometry, and
  `optimize` is deferred.

## Scope

### In scope (this milestone)

1. Rust project init; `diagctl --help` and `diagctl --version` work.
2. One real vertical slice: `diagctl check <file.svg>` implementing two checks:
   - **Layer-0 `viewbox-present`** — root `<svg>` carries a `viewBox` attribute.
   - **Layer-2 `aspect-ratio`** — viewBox long/short ratio; fail when `> 2.5`.
3. The slice exercises **usvg** (parses the SVG) so the foundational dependency
   is build-proven cross-platform now, not at the geometry milestone.
4. Cross-platform release binaries wired in CI (GitHub Actions matrix +
   cargo-dist).
5. README updated with build/test/run; test suite green.
6. `optimize` / `ascii` / `freshness` stubbed: print "not yet implemented" +
   tracking note, exit `2`.

### Out of scope (next milestone / other stories)

- Layer-2 geometry: node–node overlap, edge-through-node, edge–edge crossings,
  label overflow (the usvg/lyon-heavy work).
- Remaining Layer-0/1 conformance: fixed width/height, stroke/font thresholds,
  font-family chain, pair naming, WCAG contrast.
- Real bodies of `optimize`, `ascii`, `freshness`.
- The vision self-check loop (metapowers#38) — entirely separate story.

## Architecture

Subcommands as separate focused modules; a small check registry the next
milestone extends by adding files under `checks/`.

```
src/
  main.rs            thin: build CLI, dispatch, map command Result -> process exit code
  cli.rs             clap (derive): subcommands check/optimize/ascii/freshness; --version/--help
  svg.rs             load(path) -> Svg { raw text + parsed usvg::Tree }; the foundational SVG path
  checks/
    mod.rs           CheckResult { id, layer, status: Pass|Fail, detail }; ordered registry
    viewbox.rs       Layer-0 viewbox-present
    aspect.rs        Layer-2 aspect-ratio (threshold 2.5)
  commands/
    check.rs         run registry over file; print one line per check; exit 0 pass / 1 any-fail
    optimize.rs      stub -> exit 2
    ascii.rs         stub -> exit 2
    freshness.rs     stub -> exit 2
tests/
  cli.rs             integration tests: run the compiled binary against fixtures (assert_cmd)
  fixtures/
    in-band.svg      viewBox present, ratio <= 2.5  -> exit 0
    strip.svg        viewBox present, ratio  > 2.5  -> aspect fails, exit 1
    no-viewbox.svg   square, no viewBox             -> viewbox-present fails, exit 1
```

### The two checks

- **`viewbox-present` (Layer-0).** usvg infers a size when `viewBox` is absent,
  so it cannot report _absence_. Presence is therefore read from the raw root
  `<svg>` attributes via `roxmltree` (already a transitive dep of usvg). Fail if
  the root `<svg>` element has no `viewBox` attribute.
- **`aspect-ratio` (Layer-2).** Parse with **usvg**; take the resolved size
  `W×H`; `ratio = max(W,H) / min(W,H)`; **fail when `ratio > 2.5`**. Mirrors
  metapowers PR #34 `check-aspect.sh` (threshold 2.5, `<=` passes). Computing
  from the usvg tree — not a raw regex — is the deliberate exercise of the
  foundational parser.

### Data flow

`main` → parse args (`cli`) → dispatch to a `commands::*` handler. `check` calls
`svg::load` once, runs each registered check against the loaded `Svg`, prints a
line per check (`PASS`/`FAIL` + id + layer + detail), and returns an exit code:
`0` if all pass, `1` if any fails. Stub commands return `2`.

### Error handling

- File unreadable / not found → stderr message, exit `2` (operational error,
  distinct from a check failure).
- SVG unparseable by usvg → stderr message, exit `2`.
- A _check failing_ is a normal, expected outcome → exit `1`, not an error.
- Exit-code contract: `0` clean · `1` ≥1 check failed · `2` operational error /
  stub. The `1` vs `2` split keeps "the gate found a problem" distinct from "the
  tool could not run", so CI can tell them apart.

### Testing (TDD, no mocks)

Integration tests in `tests/cli.rs` drive the **compiled binary** with
`assert_cmd` + `predicates`, asserting exit codes and stdout/stderr against the
three committed fixtures. Failing tests are written first (red), then the
implementation (green). Real SVG fixtures, real binary, real assertions — no
mocking. Unit tests on `aspect`/`viewbox` may accompany but the binary-level
tests are the contract.

### CI / release

- **CI workflow** (`.github/workflows/ci.yml`): `cargo fmt --check`,
  `cargo clippy
  -D warnings`, `cargo test` on a `{macos, ubuntu, windows}`
  matrix.
- **Release** via **cargo-dist** (now `dist`): generates
  `.github/workflows/release.yml` building static binaries for macOS (x86_64 +
  aarch64), Linux (x86_64, musl), Windows (x86_64), attached to GitHub Releases
  on tag.

## Dependencies

- `clap` (derive) — CLI parsing, `--help`/`--version`.
- `usvg` — SVG parse + resolved geometry (foundational; geometry milestone
  builds on it).
- `roxmltree` — raw root-attribute read for `viewbox-present`.
- `anyhow` — ergonomic operational-error propagation in `main`/commands.
- dev: `assert_cmd`, `predicates` — binary integration testing.

## Conventions

MIT © driftsys (LICENSE present). Strict TDD on real fixtures. Subcommands as
focused modules. Small commits. One PR. `Id:`/MarkSpec rules not applicable (no
MarkSpec entry blocks in this repo).

## Verifiable success criteria

1. `diagctl --version` prints the crate version; `diagctl --help` lists all four
   subcommands → verify: run both, assert output.
2. `diagctl check tests/fixtures/in-band.svg` → exit `0`.
3. `diagctl check tests/fixtures/strip.svg` → exit `1`, output names the failing
   `aspect-ratio` check.
4. `diagctl check tests/fixtures/no-viewbox.svg` → exit `1`, output names the
   failing `viewbox-present` check.
5. `diagctl optimize|ascii|freshness <any>` → exit `2` with a "not yet
   implemented" message referencing metapowers#14.
6. `cargo test` green; `cargo fmt --check` and `cargo clippy -D warnings` clean.
7. CI matrix green on macOS/Linux/Windows; release workflow present and valid.

```
```

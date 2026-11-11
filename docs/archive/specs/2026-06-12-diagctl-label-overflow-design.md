# diagctl label overflow — design

**Date:** 2026-06-12 **Status:** approved (design locked with user 2026-06-12)
**Tracks:** [driftsys/diagctl#2](https://github.com/driftsys/diagctl/issues/2) —
milestone **2c** **Builds on:** 2a `geom` (labels are already classified
`Kind::Label`), the `svg::load` path, and the `font-family` check's
defensive-skip pattern

## Goal

Add the Layer-2 **`label-overflow`** check: flag a text label whose rendered
bounding box extends past the **viewBox (canvas) edge** — i.e. text clipped at
the diagram boundary. This is the 10th check and lifts the item deferred in the
README ("label overflow … needs font loading, which makes text metrics
non-deterministic across platforms").

**Scope decision:** we detect **label-vs-canvas** overflow, not
**label-vs-container**. Container-fit would compare _our_ approximate label bbox
against a node the renderer sized with _its own_ font, producing false positives
on clean auto-layout diagrams. Canvas overflow compares the label bbox against
the viewBox, which is exact and renderer-independent — one source of
approximation instead of two. Container overflow is a possible follow-up once
the metrics are trusted.

## The core problem & approach

A rendered SVG references fonts **by name** (`font-family: Helvetica`); it does
not embed glyph outlines. Today [`svg::load`](../../../src/svg.rs) parses with
an empty `fontdb`, so text carries no reliable geometry, and loading system
fonts would make text metrics differ per platform. Either way there is no
deterministic label bbox — the reason this check was parked.

**Approach — bundle one metrics font and force it.** Embed a single
permissively-licensed sans font in the binary, load it into a `fontdb`
containing **exactly that one font**, and point all default families (serif /
sans-serif / monospace / cursive / fantasy) at it. usvg's fallback then resolves
_every_ `font-family` name to the bundled font, so `text.abs_bounding_box()`
returns **real, platform-identical** geometry (usvg handles `<tspan>`,
`text-anchor`, `dx`/`dy`, transforms, and multi-line for us).

The bundled font is not the diagram's true font, so widths carry a **bounded
systematic offset** (~10–15 % between two sans-serifs). This is inherent to
_any_ deterministic method — bundling just makes the offset small and identical
everywhere. It is absorbed by a size-proportional tolerance (below), so only
gross clips fail. The gate stays "additive, never a blocker."

## Architecture

```
src/
  svg.rs                 add: shaped_tree(svg) -> anyhow::Result<usvg::Tree>
                              (re-parses svg.raw with a bundled-font Options)
                         add: embedded font bytes via include_bytes!
  checks/
    mod.rs               register label_overflow in run_all (10th check)
    label_overflow.rs    NEW — Layer-2 label-past-viewBox check
tests/
  fixtures/
    d2-clean.svg           reuse — labels fit                 -> Pass (regression guard)
    plantuml-clean.svg     reuse — labels fit                 -> Pass (regression guard)
    in-band.svg            reuse — labels fit                 -> Pass (regression guard)
    label-overflow.svg     NEW — long label past viewBox edge -> Fail
  cli.rs                 integration: check label-overflow.svg exit 1
```

### Isolation decision — re-parse, don't change the shared load path

The other 8 checks depend on the empty-font tree from `svg::load`. Enabling
fonts globally would alter the parsed tree for every check (text nodes gain
geometry) and force re-validation of all of them. Instead,
`label_overflow::check` obtains a **separate** font-enabled tree via
`svg::shaped_tree(svg)`, which re-parses `svg.raw` once with the bundled-font
`Options`. Blast radius stays limited to this one check; `svg::load` and the
other 9 checks are untouched.

```rust
// svg.rs
pub fn shaped_tree(svg: &Svg) -> anyhow::Result<usvg::Tree>;
```

- The bundled font bytes are embedded with `include_bytes!` (no runtime file
  dependency — keeps the binary hermetic).
- A lazily-initialised shared `fontdb` (`std::sync::OnceLock`) avoids re-loading
  the font on every call.
- `Options.font_family` (and the generic-family defaults) are set to the bundled
  font's family name, so unmatched names fall back to it.

### `checks::label_overflow` (Layer 2)

```rust
pub const OVERFLOW_ABS_FLOOR_PX: f32 = 2.0; // antialiasing / rounding headroom
pub const OVERFLOW_SIZE_FRACTION: f32 = 0.15; // absorbs the bundled-font width offset
```

```text
tree   = svg::shaped_tree(svg)          // font-enabled re-parse
canvas = (0, 0, tree.size().w, tree.size().h)   // resolved viewBox rect
overflowing = 0; worst = 0.0
for each Text label in tree:
    b = label.abs_bounding_box()
    tol_x  = max(OVERFLOW_ABS_FLOOR_PX, OVERFLOW_SIZE_FRACTION * b.w)
    tol_y  = max(OVERFLOW_ABS_FLOOR_PX, OVERFLOW_SIZE_FRACTION * b.h)
    h_over = max(0, canvas.left - b.left, b.right  - canvas.right)   // horiz protrusion
    v_over = max(0, canvas.top  - b.top,  b.bottom - canvas.bottom)  // vert protrusion
    if h_over > tol_x or v_over > tol_y:
        overflowing += 1; worst = max(worst, h_over, v_over)
Fail if overflowing > 0 else Pass; detail reports count + worst px.
```

- `CheckResult { id: "label-overflow", layer: 2, status, detail }`; registered
  in `run_all` after `edge_node_overlap`.
- **Tolerance rationale:** the font-offset error scales with text _length_, not
  a fixed pixel count, so the tolerance is proportional to the label's own size
  (`0.15 × width` horizontally, `0.15 × height` vertically) with a small
  absolute floor. A gross clip (label half off-canvas) exceeds it and fails; a
  near-edge label within font-offset noise passes. Defaults are tunable against
  fixtures.
- **Coordinate space:** label `abs_bounding_box` and `tree.size()` share the
  resolved viewBox coordinate system. Origin handling for a nonzero-`minX`
  viewBox is confirmed against a fixture during implementation before locking
  the tolerance.

## Data flow

`check` → `svg::load` (unchanged, empty-font tree used by the other 9 checks) →
`run_all` also calls `label_overflow::check`, which calls `svg::shaped_tree` for
its own font-enabled tree. Exit-code contract unchanged (0 / 1 / 2).

## Error handling

- No text / no labels → 0 overflowing → **Pass** ("no labels").
- `shaped_tree` re-parse fails, or the bundled font fails to load → **Pass**
  with `"skipped"`, mirroring
  [`font_family`](../../../src/checks/font_family.rs)'s defensive skip. The
  check never turns a rendering hiccup into a blocker.
- Rotated text: `abs_bounding_box` is the axis-aligned box of the rotated glyphs
  — a valid, conservative bbox.

## Testing (TDD, real + hand-authored, no mocks)

- **Regression guard (critical):** `label_overflow::check` on `d2-clean.svg`,
  `plantuml-clean.svg`, and `in-band.svg` → **Pass** (their labels fit their
  canvases).
- **Font-load safety test:** assert `shaped_tree` shapes a known non-empty
  string to a **non-zero-width** bbox. Without this, a silent font-load failure
  would make the check a no-op that passes everything (false negatives) — this
  test fails loudly instead.
- **Fail fixture:** `label-overflow.svg` — a hand-authored SVG with a long text
  label positioned to clearly extend past the viewBox edge, authored so viewBox
  / aspect / node-overlap all pass, so **only** `label-overflow` fails →
  **Fail** (and id `"label-overflow"`).
- **Integration (`tests/cli.rs`):** `check label-overflow.svg` → exit 1, output
  names `label-overflow`.
- Optional edge fixture: a label protruding ~1px past the edge → **Pass**
  (tolerance proven).

## Bundled font & binary size

- A single permissively-licensed sans (**DejaVu Sans** or **Roboto**,
  Apache-2.0) is embedded — chosen for redistributability under the project's
  MIT license. Adds ~0.7 MB to the ~2–8 MB binary (in line with the README's
  stated size budget).
- The font's `LICENSE`/attribution ships alongside the binary source as
  required.

## Scope discipline (YAGNI)

Ships only `label-overflow` (canvas variant) plus the `shaped_tree` helper. No
new crate dependencies — reuses the `usvg` text stack already compiled in.
Container-vs-label overflow is **not** implemented (deferred as a possible
follow-up). After this milestone the Layer-2 geometry checks are complete;
remaining diagctl work: 2d mechanical Layer-0/1 items, source+SVG freshness, and
the `optimize`/`ascii`/`freshness` subcommand bodies.

## Docs

- README: sweep `9-check` → `10-check` gate; move **label overflow** from "Not
  yet implemented" into the checks table (`label-overflow` | 2 | "label text
  clipped at the viewBox edge"); reword the deferred bullet to note the
  bundled-font approach resolved the determinism blocker and that container
  overflow remains a possible follow-up.

## Alternatives considered

- **Font-free width estimation** (char-count × advance × font-size from raw
  XML): rejected — cruder, and hand-rolls `text-anchor` / `<tspan>` / transform
  / multi-line handling that usvg already does correctly.
- **Container-vs-label overflow** (the README's original framing): deferred —
  false-positive risk comparing an approximate label bbox against renderer-sized
  nodes; canvas overflow is the robust first cut.
- **Shared font-enabled `svg::load`**: rejected — would change the parsed tree
  for all 9 existing checks and require re-validating each; the isolated
  `shaped_tree` re-parse keeps the blast radius to this check.

## Verifiable success criteria

1. `svg::shaped_tree` shapes a known non-empty string to a bbox with width > 0
   (unit-tested) — proves the bundled font loads and the check is not a silent
   no-op.
2. `label_overflow::check(d2-clean)`, `(plantuml-clean)`, `(in-band)` → **Pass**
   (regression guard).
3. `label_overflow::check(label-overflow)` → **Fail** (and id
   `"label-overflow"`).
4. `check label-overflow.svg` → exit 1, output names `label-overflow`.
5. The other 9 checks' results are byte-identical before and after this change
   (isolation proven — `svg::load` untouched).
6. `cargo test` green; `cargo fmt --check` + `cargo clippy -D warnings` clean;
   CI matrix + musl guard green.

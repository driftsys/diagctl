# diagctl geometry foundation + node–node overlap — design

**Date:** 2026-06-07 **Status:** approved (scope, classification strategy,
fixtures locked with user 2026-06-07) **Tracks:**
[driftsys/diagctl#2](https://github.com/driftsys/diagctl/issues/2) — milestone
**2a** of the geometry work **Parent design:**
[metapowers#14](https://github.com/driftsys/metapowers/issues/14) **Boundary:**
vision self-check
([metapowers#38](https://github.com/driftsys/metapowers/issues/38)) — out of
scope

## Goal

Build the **geometry foundation** for diagctl's Layer-2 checks and prove it with
one check: **node–node overlap**. The foundation extracts a classified geometric
model from the rendered SVG (nodes / edges / labels / background) that later
checks (edge overlap, edge crossings, label overflow) consume. This milestone
ships only the node-overlap check; edges are _classified_ but not yet checked.

## Decomposition (issue #2)

Issue #2 bundles ~9 checks across two kinds of work. Sequenced to de-risk:

- **2a (this spec)** — geometry foundation (`geom`) + **node–node overlap**
  (AABB, no lyon).
- **2b** — edge checks (edge–node overlap, edge–edge crossings); adds `lyon`.
- **2c** — label overflow (text bbox vs container bbox; needs font loading).
- **2d** — mechanical Layer-0/1 (fixed width/height, stroke/font thresholds,
  font-family chain, WCAG contrast, pair naming) — extends the existing raw-attr
  pattern.

## Classification strategy — group structure + fill

Settled after probing real D2 and PlantUML renders through usvg 0.45 (the exact
parser/version diagctl already uses). Findings that drive the design:

- **usvg preserves the per-node group structure** — each logical node and each
  edge is its own `<g>` under the root, for both renderers (groups are _not_
  flattened).
- **usvg drops CSS classes** (D2's `class="shape"`/`"connection"` are gone; node
  `id` came back `""`). So class/id-based classification is out — geometry it
  is.
- **Reliable renderer-agnostic signals:** group structure + paint:
  - **Node** group → a filled path covering most of the group bbox; bbox <
    canvas.
  - **Edge** group → dominant path is `fill=none` + stroke (the connector) plus
    a small filled arrowhead.
  - **Background** → a top-level filled path whose bbox ≈ canvas.
- Grouping gives **"unrelated shapes" for free**: a node's box + its label live
  in the same group (one bbox), so they are never flagged against each other;
  overlap is tested only _between different node groups_. A flat, group-blind
  scan fails here — PlantUML wraps **4 filled sub-rects per node** (component
  ports) that a flat scan would miscount as overlapping shapes.

### Alternatives considered

- **Flat geometry (group-blind):** every filled path is a node; "related" only
  via containment. Rejected — miscounts multi-rect nodes (PlantUML) as overlaps.
- **Renderer-aware (detect D2/PlantUML/draw.io):** parse each renderer's
  class/id conventions. Rejected — usvg drops classes, ids are inconsistent (D2
  empty), brittle across renderer versions, against the "one renderer-agnostic
  binary" rule.

## Architecture

```
src/
  geom.rs            NEW — classified DiagramModel extracted from usvg::Tree
  checks/
    mod.rs           register node_overlap in run_all
    node_overlap.rs  NEW — Layer-2 node–node bbox overlap check
tests/
  fixtures/
    d2-clean.svg         real D2 render (2 separated nodes)        -> Pass
    plantuml-clean.svg   real PlantUML render (multi-rect nodes)   -> Pass
    overlap.svg          hand-authored, two node groups overlapping -> Fail
  cli.rs             integration: check <fixture> exit 0/1
```

### `geom.rs` — the foundation

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }

impl BBox {
    pub fn area(&self) -> f32;                 // w*h (clamped >= 0)
    pub fn intersects(&self, o: &BBox) -> bool;        // any positive-area overlap
    pub fn intersection_area(&self, o: &BBox) -> f32;  // 0 if disjoint
    pub fn contains(&self, o: &BBox) -> bool;          // self fully covers o
    pub fn inset(&self, d: f32) -> BBox;               // shrink by d on all sides
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind { Node, Edge, Label, Background, Other }

#[derive(Debug, Clone)]
pub struct Element { pub kind: Kind, pub bbox: BBox }

/// Classify the diagram's top-level units from the usvg tree.
pub fn model(svg: &crate::svg::Svg) -> Vec<Element>;

/// Convenience: just the node bboxes.
pub fn nodes(svg: &crate::svg::Svg) -> Vec<BBox>;
```

**Classification algorithm:**

1. **Find the content group.** From `tree.root()`, descend while the current
   group has exactly one child that is itself a group (trivial wrappers). Stop
   at the first group with >1 child (or whose single child is a leaf). Its
   children are the units. (Verified: D2 root→inner=4 children; PlantUML root=3
   children.)
2. **Classify each unit** (a `Node::Group` or leaf):
   - Compute the unit's abs bbox (`abs_bounding_box()`).
   - **Background:** a filled path whose bbox area ≥ `BG_CANVAS_FRACTION` (0.95)
     of the canvas area → `Background` (excluded from checks).
   - **Node:** the unit contains a filled path whose bbox area ≥
     `NODE_FILL_FRACTION` (0.50) of the unit bbox area, and the unit is not
     Background → `Node`.
   - **Edge:** not Node/Background, and the unit's dominant path is
     `fill=none` + stroke → `Edge` (tagged now; consumed in 2b).
   - **Label:** a `Node::Text` unit → `Label`.
   - else → `Other`.
   - "Filled path within a unit" is found by recursing the unit's descendants;
     area is approximated by path **bbox** area (a filled rect's bbox area ≈ its
     area; an arrowhead's bbox area is small — sufficient to separate nodes from
     edges).

usvg API used (validated by the probe): `Tree::root() -> &Group`,
`Group::children() -> &[Node]`, `Node::{Group,Path,Text,Image}`,
`{Group,Path,Text}::abs_bounding_box() -> Rect` with
`.x()/.y()/.width()/.height()`, `Path::fill() -> Option<&Fill>`,
`Path::stroke() -> Option<&Stroke>`.

Font loading is **not** required: node classification reads only filled-path
geometry, independent of text shaping. `svg::load` stays as-is
(`Options::default()`). Label-overflow (2c) will add font loading; out of scope
here.

### `checks/node_overlap.rs` — the check (Layer 2)

```rust
pub const OVERLAP_TOLERANCE_PX: f32 = 1.0; // ignore <=1px border adjacency
```

- Get `geom::nodes(svg)`. For each unordered pair `(a, b)`:
  - Skip if `a.contains(b)` or `b.contains(a)` (container/child — expected).
  - Flag if
    `a.inset(OVERLAP_TOLERANCE_PX/2).intersects(&b.inset(OVERLAP_TOLERANCE_PX/2))`
    (shrinking both by ½ the tolerance, so mere border-adjacency from layout
    rounding does not trigger).
- `Status::Fail` if ≥1 flagged pair; `detail` = count + the offending bboxes.
  `Status::Pass` (0 pairs) otherwise.
- `CheckResult { id: "node-overlap", layer: 2, status, detail }` — existing
  shape.
- Register in `checks::run_all` after the existing checks.

Tunable constants (`BG_CANVAS_FRACTION`, `NODE_FILL_FRACTION`,
`OVERLAP_TOLERANCE_PX`) are named consts, not magic numbers.

## Data flow

`check` command → `svg::load` (unchanged) → `checks::run_all` now also calls
`node_overlap::check`, which calls `geom::nodes`. No change to the exit-code
contract (`0` pass / `1` any fail / `2` operational error).

## Error handling

- `geom::model` operates on the already-parsed `usvg::Tree`; it cannot fail
  (returns a possibly-empty `Vec`). A diagram with 0 or 1 node → no pairs →
  `node-overlap` Pass.
- Unreadable/unparseable SVG is already handled upstream by `svg::load` →
  exit 2.

## Testing (TDD, real fixtures, no mocks)

- **`geom` unit tests** (in `geom.rs`): against each committed fixture, assert
  the classified counts and node bboxes — e.g. `d2-clean.svg` → 2 Nodes, ≥1
  Edge, 1 Background; `plantuml-clean.svg` → 2 Nodes (each a single logical node
  despite 4 sub-rects). Plus `BBox` math unit tests (intersects / contains /
  intersection_area / inset) with hand-computed values.
- **`node_overlap` unit tests**: `d2-clean` & `plantuml-clean` → Pass; `overlap`
  → Fail.
- **Integration** (`tests/cli.rs`): `check d2-clean.svg` → exit 0;
  `check overlap.svg` → exit 1 with stdout naming `node-overlap`.
  **`plantuml-clean` is not used in an integration test** — its 2-node render is
  an inherent strip (≈170×59, aspect 2.88) that the independent aspect-ratio
  check flags, which is unrelated to node-overlap. Its cross-renderer value is
  captured in the `geom`/`node_overlap` **unit** tests, which bypass the other
  checks.
- Fixtures: `d2-clean.svg` and `plantuml-clean.svg` are **actual renders**
  committed as static files (tests stay hermetic — no renderer at test time);
  `overlap.svg` is hand-authored to mirror the group+filled-rect structure with
  two overlapping node groups, sized so viewBox-present and aspect both pass (so
  only node-overlap fails).

## Scope discipline (YAGNI)

Ships **only** `node-overlap`. Edges are classified but unchecked (2b + lyon).
No label-overflow, no Layer-0/1, no lyon dependency, no `svg::load` change.
`geom` is built as the shared substrate the later geometry checks consume.

## Verifiable success criteria

1. `geom::model(d2-clean)` → exactly 2 `Node` elements with bboxes matching the
   render (~53×66 each), ≥1 `Edge`, 1 `Background`.
2. `geom::model(plantuml-clean)` → exactly 2 `Node` elements (not 8) → verify
   the multi-sub-rect grouping holds.
3. `check d2-clean.svg` → exit 0 (all checks PASS, including `node-overlap`).
4. `node_overlap::check(plantuml-clean)` → Pass (unit test) — not run through
   the full `check` command, whose aspect check independently flags the strip
   render.
5. `check overlap.svg` → exit 1; output names `node-overlap` with the
   overlapping pair.
6. `cargo test` green; `cargo fmt --check` + `cargo clippy -D warnings` clean;
   musl guard + CI matrix green.

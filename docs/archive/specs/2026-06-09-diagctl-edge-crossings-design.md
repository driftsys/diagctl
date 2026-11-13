# diagctl edge segment foundation + edge–edge crossings — design

**Date:** 2026-06-09 **Status:** approved (scope, geometry approach, threshold
locked with user 2026-06-09) **Tracks:**
[driftsys/diagctl#2](https://github.com/driftsys/diagctl/issues/2) — milestone
**2b** **Builds on:** 2a `geom` foundation (PR #4)

## Goal

Extend the `geom` foundation with **edge geometry** (flattened polylines, not
just bboxes) and ship the first edge check: Layer-2 **`edge-crossings`** — count
edge–edge segment intersections in open whitespace and fail above a threshold.

## Decomposition

Milestone 2b ships **only** `edge-crossings`. **edge–node overlap** needs an
endpoint→node mapping heuristic and is deferred to **2b-2**. (Remaining: 2c
label overflow, 2d mechanical Layer-0/1.)

## Decisions (locked with user)

- **Scope:** edge segment foundation + `edge-crossings` only.
- **Geometry math: hand-rolled, zero new deps.** The need is small (bezier
  flattening + segment intersection) and fully unit-testable; matches diagctl's
  reimplement / minimal-deps / small-binary principle. (`lyon`/`lyon_geom`
  considered and rejected as heavier than the requirement.)
- **Crossing threshold: 0** (any open-space crossing fails); tunable const.

## Findings that drive the design (probe of real renders, usvg 0.45)

- D2 edges are **cubic béziers** (`C` commands) → curves must be flattened to
  polylines before intersection tests.
- usvg exposes the geometry needed: `Path::data().segments()` yields
  `MoveTo/LineTo/QuadTo/CubicTo/Close` in **local** coords;
  `Path::abs_transform()` + `Transform::map_points` convert to **absolute**
  canvas coords.
- D2's dagre layout **avoids crossings** even for a triangle+diagonal graph (0
  crossings) → the _passing_ fixture can be a real multi-edge render; the
  _failing_ fixture is hand-authored.

## Architecture

```
src/
  geom.rs            extend: re-export Point; add edges() polyline extractor
  geom/seg.rs        NEW — pure geometry math (no usvg): Point, flatten, segment_intersection
  checks/
    mod.rs           register edge_crossings in run_all
    edge_crossings.rs NEW — Layer-2 edge–edge crossing check
tests/
  fixtures/
    cross-clean.svg       real D2 render, ≥2 edges, no crossings        -> Pass
    crossing.svg          hand-authored, 2 edges cross in open space     -> Fail
    crossing-at-node.svg  hand-authored, 2 edges cross inside a node bbox -> Pass (exclusion)
  cli.rs             integration: check crossing.svg exit 1
```

### `geom::seg` — pure math (hand-rolled, unit-tested, no usvg)

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point { pub x: f32, pub y: f32 }

pub const FLATTEN_TOLERANCE_PX: f32 = 0.25;

/// Adaptive de Casteljau subdivision; appends flattened points (excluding p0) to `out`.
pub fn flatten_cubic(p0: Point, p1: Point, p2: Point, p3: Point, out: &mut Vec<Point>);
pub fn flatten_quad(p0: Point, p1: Point, p2: Point, out: &mut Vec<Point>);

/// Proper crossing point of segments a–b and c–d, or None if parallel/collinear/non-crossing.
pub fn segment_intersection(a: Point, b: Point, c: Point, d: Point) -> Option<Point>;
```

- `flatten_*`: recurse while the control polygon's deviation from the chord
  exceeds `FLATTEN_TOLERANCE_PX`; emit the curve endpoint at the base case.
  Bounded recursion depth (e.g. 16) as a safety cap.
- `segment_intersection`: orientation (cross-product) signs to detect a proper
  crossing, then parametric `t` to compute the point. Parallel/collinear →
  `None` (overlapping collinear edges are a separate concern, out of scope
  here).

### `geom::edges` — edge polylines from the usvg tree

```rust
pub fn edges(svg: &crate::svg::Svg) -> Vec<Vec<seg::Point>>;
```

For each top-level unit classified `Kind::Edge` (reusing 2a's classification),
find its dominant `fill=none` stroke path (the connector), iterate
`path.data().segments()`, map each point through `path.abs_transform()`, and
flatten `CubicTo`/`QuadTo` via `geom::seg`. Returns one polyline (≥2 points) per
edge. Arrowhead polygons (filled) are ignored. `nodes()`/`model()` are
unchanged.

### `checks::edge_crossings` — the check (Layer 2)

```rust
pub const CROSSING_THRESHOLD: usize = 0; // fail if open-space crossings > 0
pub const NODE_MARGIN_PX: f32 = 1.0;     // node bboxes expanded by this for the exclusion
```

- `let edges = geom::edges(svg); let nodes = geom::nodes(svg);`
- For each unordered pair of distinct edges, for each segment pair, compute
  `seg::segment_intersection`. If a crossing point exists **and lies outside
  every node bbox inset by `NODE_MARGIN_PX`**, count it.
- `Fail` if `crossings > CROSSING_THRESHOLD`, else `Pass`. Detail reports the
  count.
- `CheckResult { id: "edge-crossings", layer: 2, status, detail }`; registered
  in `run_all` after `node_overlap`.

**Semantics rationale:** edges converge at and terminate on nodes by design, so
intersections inside node bboxes are not layout defects (and are visually hidden
by the node). Only crossings in open whitespace indicate a readability problem.
The node-bbox exclusion is what makes the check robust to fan-out at shared
endpoints.

## Data flow

`check` → `svg::load` (unchanged) → `run_all` now also calls
`edge_crossings::check`, which uses `geom::edges` + `geom::nodes`. Exit-code
contract unchanged (0/1/2).

## Error handling

- `geom::edges` operates on the parsed tree; returns a possibly-empty `Vec`. 0
  or 1 edge → no pairs → Pass.
- Degenerate/empty connector paths (< 2 points) are skipped.
- Recursion in flattening is depth-capped to avoid pathological inputs.

## Testing (TDD, real + hand-authored fixtures, no mocks)

- **`seg` unit tests:** `segment_intersection` on a known crossing (returns the
  expected point), parallel (None), and touching-but-not-crossing (None);
  `flatten_cubic` on a straight-line "curve" (collinear control points) yields
  collinear points ending at p3.
- **`geom::edges` unit tests:** `cross-clean.svg` → expected edge count, each
  polyline ≥ 2 points.
- **`edge_crossings::check` unit tests:** `cross-clean` → Pass; `crossing` →
  Fail; `crossing-at-node` → Pass (exclusion exercised).
- **Integration (`tests/cli.rs`):** `check crossing.svg` → exit 1 naming
  `edge-crossings` (authored so viewBox + aspect pass, so only edge-crossings
  fails). `cross-clean.svg` used in integration only if its render passes
  viewBox + aspect; otherwise unit-test-only (decided at fixture-generation
  time, as with `plantuml-clean` in 2a).

## Scope discipline (YAGNI)

Only `edge-crossings`. No edge–node overlap, no lyon, no new deps, no change to
`nodes()`/`model()`/`svg::load`. `geom::seg` + `geom::edges` are the additive
substrate edge–node overlap (2b-2) will reuse.

## Verifiable success criteria

1. `seg::segment_intersection` returns the correct point for a known X crossing
   and `None` for parallel segments (unit-tested with exact values).
2. `geom::edges(cross-clean)` → the render's edge count, each polyline ≥ 2
   points.
3. `edge_crossings::check(cross-clean)` → Pass (real multi-edge diagram, no
   false positive).
4. `edge_crossings::check(crossing)` → Fail;
   `edge_crossings::check(crossing-at-node)` → Pass (node-bbox exclusion
   proven).
5. `check crossing.svg` → exit 1, output names `edge-crossings`.
6. `cargo test` green; `cargo fmt --check` + `cargo clippy -D warnings` clean;
   CI matrix
   - musl guard green.

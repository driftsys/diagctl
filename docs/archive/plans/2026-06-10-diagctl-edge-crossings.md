# diagctl Edge Segment Foundation + Edge–Edge Crossings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend `geom` with edge polyline extraction (bezier flattening +
segment math) and add a Layer-2 `edge-crossings` check that counts edge–edge
crossings in open whitespace (milestone 2b of diagctl#2).

**Architecture:** A pure-math `geom::seg` submodule (hand-rolled de Casteljau
flattening + segment intersection, no usvg, no new deps) feeds `geom::edges()`,
which extracts each edge's `fill=none` connector path from the usvg tree, maps
it to absolute coords, and flattens curves to polylines.
`checks::edge_crossings` counts segment intersections between distinct edges
whose point lies outside every node bbox (so fan-out at nodes is not flagged).

**Tech Stack:** Rust, usvg 0.45 (existing; `Path::data().segments()`,
`abs_transform()`, `Transform::map_points` validated by probe). No new
dependencies.

---

## File Structure

- `src/geom/seg.rs` — NEW. `Point`, `flatten_cubic`/`flatten_quad`,
  `segment_intersection`. Pure math.
- `src/geom.rs` — MODIFY. `pub mod seg;`; add `edges()` +
  `path_to_polyline`/`edge_polyline` helpers.
- `src/checks/edge_crossings.rs` — NEW. Layer-2 edge–edge crossing check.
- `src/checks/mod.rs` — MODIFY. Register `edge_crossings` in `run_all`.
- `tests/fixtures/cross-clean.svg` — NEW (rendered). 4 edges, no crossings.
- `tests/fixtures/crossing.svg` — NEW (hand-authored). 2 edges cross in open
  space.
- `tests/fixtures/crossing-at-node.svg` — NEW (hand-authored). 2 edges cross
  inside a node bbox.
- `tests/cli.rs` — MODIFY. Integration tests for crossing (exit 1) + cross-clean
  (exit 0).

### Type/interface contract (keep names stable)

```rust
// src/geom/seg.rs
pub struct Point { pub x: f32, pub y: f32 }   // derive Debug, Clone, Copy, PartialEq
impl Point { pub fn new(x: f32, y: f32) -> Point }
pub const FLATTEN_TOLERANCE_PX: f32 = 0.25;
pub fn flatten_cubic(p0: Point, p1: Point, p2: Point, p3: Point, out: &mut Vec<Point>);
pub fn flatten_quad(p0: Point, p1: Point, p2: Point, out: &mut Vec<Point>);
pub fn segment_intersection(a: Point, b: Point, c: Point, d: Point) -> Option<Point>;

// src/geom.rs (additions)
pub mod seg;
pub fn edges(svg: &crate::svg::Svg) -> Vec<Vec<seg::Point>>;

// src/checks/edge_crossings.rs
pub const CROSSING_THRESHOLD: usize = 0;
pub const NODE_MARGIN_PX: f32 = 1.0;
pub fn check(svg: &crate::svg::Svg) -> crate::checks::CheckResult; // id "edge-crossings", layer 2
```

> **usvg API (probe-validated):** `Path::data() -> &tiny_skia_path::Path`,
> `tiny_skia_path::PathSegment::{MoveTo,LineTo,QuadTo,CubicTo,Close}` (Point
> args with `.x/.y`), `Path::abs_transform() -> Transform`,
> `Transform::map_points(&mut [Point])`, `usvg::tiny_skia_path::Point::from_xy`.
> usvg re-exports `tiny_skia_path`.

---

## Task 1: `geom::seg` pure geometry math

**Files:**

- Create: `src/geom/seg.rs`
- Modify: `src/geom.rs`

- [ ] **Step 1: Write `src/geom/seg.rs`**

```rust
//! Pure geometry math for edge checks: bezier flattening + segment intersection.
//! No usvg, no I/O — fully unit-testable with exact values.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }
}

pub const FLATTEN_TOLERANCE_PX: f32 = 0.25;
const MAX_FLATTEN_DEPTH: u8 = 16;

fn mid(a: Point, b: Point) -> Point {
    Point {
        x: (a.x + b.x) / 2.0,
        y: (a.y + b.y) / 2.0,
    }
}

fn point_line_dist(p: Point, a: Point, b: Point) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 {
        return ((p.x - a.x).powi(2) + (p.y - a.y).powi(2)).sqrt();
    }
    (dx * (a.y - p.y) - (a.x - p.x) * dy).abs() / len
}

fn cubic_is_flat(p0: Point, p1: Point, p2: Point, p3: Point) -> bool {
    point_line_dist(p1, p0, p3) <= FLATTEN_TOLERANCE_PX
        && point_line_dist(p2, p0, p3) <= FLATTEN_TOLERANCE_PX
}

/// Append the flattened cubic to `out`, excluding p0 and including p3.
pub fn flatten_cubic(p0: Point, p1: Point, p2: Point, p3: Point, out: &mut Vec<Point>) {
    subdiv_cubic(p0, p1, p2, p3, 0, out);
    out.push(p3);
}

fn subdiv_cubic(p0: Point, p1: Point, p2: Point, p3: Point, depth: u8, out: &mut Vec<Point>) {
    if depth >= MAX_FLATTEN_DEPTH || cubic_is_flat(p0, p1, p2, p3) {
        return;
    }
    let p01 = mid(p0, p1);
    let p12 = mid(p1, p2);
    let p23 = mid(p2, p3);
    let p012 = mid(p01, p12);
    let p123 = mid(p12, p23);
    let p0123 = mid(p012, p123);
    subdiv_cubic(p0, p01, p012, p0123, depth + 1, out);
    out.push(p0123);
    subdiv_cubic(p0123, p123, p23, p3, depth + 1, out);
}

/// Append the flattened quadratic to `out`, excluding p0 and including p2.
pub fn flatten_quad(p0: Point, p1: Point, p2: Point, out: &mut Vec<Point>) {
    // Elevate quadratic to cubic, then reuse the cubic flattener.
    let c1 = Point {
        x: p0.x + 2.0 / 3.0 * (p1.x - p0.x),
        y: p0.y + 2.0 / 3.0 * (p1.y - p0.y),
    };
    let c2 = Point {
        x: p2.x + 2.0 / 3.0 * (p1.x - p2.x),
        y: p2.y + 2.0 / 3.0 * (p1.y - p2.y),
    };
    flatten_cubic(p0, c1, c2, p2, out);
}

/// Proper crossing point of segments a–b and c–d, or None if they do not
/// properly cross (parallel, collinear, or only touching at an endpoint).
pub fn segment_intersection(a: Point, b: Point, c: Point, d: Point) -> Option<Point> {
    let rx = b.x - a.x;
    let ry = b.y - a.y;
    let sx = d.x - c.x;
    let sy = d.y - c.y;
    let denom = rx * sy - ry * sx;
    if denom.abs() < 1e-9 {
        return None;
    }
    let qpx = c.x - a.x;
    let qpy = c.y - a.y;
    let t = (qpx * sy - qpy * sx) / denom;
    let u = (qpx * ry - qpy * rx) / denom;
    if t > 0.0 && t < 1.0 && u > 0.0 && u < 1.0 {
        Some(Point {
            x: a.x + t * rx,
            y: a.y + t * ry,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersecting_segments_return_crossing_point() {
        let p = segment_intersection(
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
            Point::new(10.0, 0.0),
        );
        assert_eq!(p, Some(Point::new(5.0, 5.0)));
    }

    #[test]
    fn parallel_segments_return_none() {
        let p = segment_intersection(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(0.0, 5.0),
            Point::new(10.0, 5.0),
        );
        assert_eq!(p, None);
    }

    #[test]
    fn segments_sharing_endpoint_do_not_properly_cross() {
        let p = segment_intersection(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
        );
        assert_eq!(p, None);
    }

    #[test]
    fn straight_cubic_flattens_to_single_endpoint() {
        let mut out = Vec::new();
        flatten_cubic(
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0),
            Point::new(3.0, 3.0),
            &mut out,
        );
        assert_eq!(out, vec![Point::new(3.0, 3.0)]);
    }

    #[test]
    fn curved_cubic_subdivides_and_ends_at_p3() {
        let mut out = Vec::new();
        flatten_cubic(
            Point::new(0.0, 0.0),
            Point::new(0.0, 10.0),
            Point::new(10.0, 10.0),
            Point::new(10.0, 0.0),
            &mut out,
        );
        assert!(out.len() > 1, "curved cubic should subdivide");
        assert_eq!(out.last().copied(), Some(Point::new(10.0, 0.0)));
    }
}
```

- [ ] **Step 2: Declare the submodule in `src/geom.rs`**

Add as the first line of `src/geom.rs` (above the `//!` is not allowed; put it
right after the module doc comment / existing top). Concretely, add near the top
of `src/geom.rs`:

```rust
pub mod seg;
```

- [ ] **Step 3: Run the seg tests**

Run: `cargo test --bin diagctl geom::seg` Expected: PASS (5 tests).

- [ ] **Step 4: Commit**

```bash
git add src/geom/seg.rs src/geom.rs
git commit -m "feat: geom::seg bezier flattening + segment intersection"
```

---

## Task 2: Fixtures

**Files:**

- Create: `tests/fixtures/cross-clean.svg` (rendered)
- Create: `tests/fixtures/crossing.svg` (hand-authored)
- Create: `tests/fixtures/crossing-at-node.svg` (hand-authored)

- [ ] **Step 1: Render `cross-clean.svg`**

```bash
printf 'a -> b\nb -> c\nc -> a\na -> c\n' > /tmp/diagctl-cross.d2
D2_LAYOUT=dagre d2 --omit-version /tmp/diagctl-cross.d2 tests/fixtures/cross-clean.svg
```

- [ ] **Step 2: Hand-author `tests/fixtures/crossing.svg`**

Two diagonals crossing at (100,60) in open space (no nodes); viewBox 200×120
(aspect 1.67):

```xml
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="120" viewBox="0 0 200 120">
  <g>
    <path d="M 20 20 L 180 100" fill="none" stroke="#333333"/>
    <path d="M 20 100 L 180 20" fill="none" stroke="#333333"/>
  </g>
</svg>
```

- [ ] **Step 3: Hand-author `tests/fixtures/crossing-at-node.svg`**

Same crossing point (100,60), now inside a node rect (70,40)–(130,80) so it is
excluded:

```xml
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="120" viewBox="0 0 200 120">
  <g>
    <g><rect x="70" y="40" width="60" height="40" fill="#eeeeee" stroke="#333333"/></g>
    <path d="M 20 20 L 180 100" fill="none" stroke="#333333"/>
    <path d="M 20 100 L 180 20" fill="none" stroke="#333333"/>
  </g>
</svg>
```

- [ ] **Step 4: Verify they exist and parse**

Run:

```bash
ls -l tests/fixtures/cross-clean.svg tests/fixtures/crossing.svg tests/fixtures/crossing-at-node.svg
grep -o 'viewBox="[^"]*"' tests/fixtures/cross-clean.svg | head -1
```

Expected: all three exist and non-empty; cross-clean root viewBox ≈
`0 0 322 600`.

- [ ] **Step 5: Commit**

```bash
git add tests/fixtures/cross-clean.svg tests/fixtures/crossing.svg tests/fixtures/crossing-at-node.svg
git commit -m "test: edge-crossing fixtures (real clean + hand-authored crossings)"
```

---

## Task 3: `geom::edges` polyline extraction

**Files:**

- Modify: `src/geom.rs`

- [ ] **Step 1: Add the edge extractor + helpers to `src/geom.rs`**

Add `use usvg::tiny_skia_path::PathSegment;` next to the existing
`use usvg::{Group, Node};`, and add the following (place above the existing
`#[cfg(test)] mod model_tests`):

```rust
use seg::Point;

fn path_to_polyline(p: &usvg::Path) -> Vec<Point> {
    let t = p.abs_transform();
    let map = |x: f32, y: f32| -> Point {
        let mut pts = [usvg::tiny_skia_path::Point::from_xy(x, y)];
        t.map_points(&mut pts);
        Point { x: pts[0].x, y: pts[0].y }
    };
    let mut poly: Vec<Point> = Vec::new();
    let mut cur = Point { x: 0.0, y: 0.0 };
    for s in p.data().segments() {
        match s {
            PathSegment::MoveTo(pt) => {
                cur = map(pt.x, pt.y);
                poly.push(cur);
            }
            PathSegment::LineTo(pt) => {
                cur = map(pt.x, pt.y);
                poly.push(cur);
            }
            PathSegment::QuadTo(a, b) => {
                let (a, b) = (map(a.x, a.y), map(b.x, b.y));
                seg::flatten_quad(cur, a, b, &mut poly);
                cur = b;
            }
            PathSegment::CubicTo(a, b, c) => {
                let (a, b, c) = (map(a.x, a.y), map(b.x, b.y), map(c.x, c.y));
                seg::flatten_cubic(cur, a, b, c, &mut poly);
                cur = c;
            }
            PathSegment::Close => {}
        }
    }
    poly
}

fn polyline_length(poly: &[Point]) -> f32 {
    poly.windows(2)
        .map(|w| ((w[1].x - w[0].x).powi(2) + (w[1].y - w[0].y).powi(2)).sqrt())
        .sum()
}

/// The connector polyline of an edge unit: its longest `fill=none` stroke path.
fn edge_polyline(unit: &Node) -> Option<Vec<Point>> {
    let mut best: Option<Vec<Point>> = None;
    let mut best_len = 0.0f32;
    for_each_path(unit, &mut |p| {
        if p.fill().is_none() && p.stroke().is_some() {
            let poly = path_to_polyline(p);
            let len = polyline_length(&poly);
            if len > best_len {
                best_len = len;
                best = Some(poly);
            }
        }
    });
    best.filter(|poly| poly.len() >= 2)
}

/// Flattened absolute polylines for every classified edge unit.
pub fn edges(svg: &Svg) -> Vec<Vec<Point>> {
    let size = svg.tree.size();
    let canvas_area = size.width() * size.height();
    let content = content_group(svg.tree.root());
    let mut out = Vec::new();
    for u in content.children() {
        if classify(u, canvas_area).kind == Kind::Edge {
            if let Some(poly) = edge_polyline(u) {
                out.push(poly);
            }
        }
    }
    out
}
```

- [ ] **Step 2: Add edge-extraction tests to `src/geom.rs`**

Append a test module (after `model_tests`):

```rust
#[cfg(test)]
mod edge_tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    #[test]
    fn cross_clean_extracts_four_edges() {
        let svg = load(Path::new("tests/fixtures/cross-clean.svg")).unwrap();
        let es = edges(&svg);
        assert_eq!(es.len(), 4, "cross-clean edge count");
        assert!(es.iter().all(|p| p.len() >= 2), "every edge polyline has >=2 points");
    }

    #[test]
    fn crossing_fixture_extracts_two_edges() {
        let svg = load(Path::new("tests/fixtures/crossing.svg")).unwrap();
        assert_eq!(edges(&svg).len(), 2, "crossing edge count");
    }
}
```

- [ ] **Step 3: Run the edge tests**

Run: `cargo test --bin diagctl geom::edge_tests` Expected: PASS (2 tests). If
`cross_clean_extracts_four_edges` finds a different count, inspect with a
temporary `dbg!(edges(&svg).len())` — D2 emits one `fill=none` connector per
edge (4 here); arrowhead polygons are filled and correctly ignored.

- [ ] **Step 4: Commit**

```bash
git add src/geom.rs
git commit -m "feat: geom::edges extracts flattened absolute edge polylines"
```

---

## Task 4: `edge-crossings` check + registry + integration tests

**Files:**

- Create: `src/checks/edge_crossings.rs`
- Modify: `src/checks/mod.rs`
- Modify: `tests/cli.rs`

- [ ] **Step 1: Write `src/checks/edge_crossings.rs` with failing unit tests**

```rust
use crate::checks::{CheckResult, Status};
use crate::geom::{self, seg, BBox};
use crate::svg::Svg;

pub const CROSSING_THRESHOLD: usize = 0;
pub const NODE_MARGIN_PX: f32 = 1.0;

pub fn check(svg: &Svg) -> CheckResult {
    let edges = geom::edges(svg);
    let nodes = geom::nodes(svg);
    let mut crossings = 0usize;
    for i in 0..edges.len() {
        for j in (i + 1)..edges.len() {
            crossings += count_open_crossings(&edges[i], &edges[j], &nodes);
        }
    }
    let status = if crossings > CROSSING_THRESHOLD {
        Status::Fail
    } else {
        Status::Pass
    };
    CheckResult {
        id: "edge-crossings",
        layer: 2,
        status,
        detail: format!("{crossings} open-space edge crossing(s)"),
    }
}

fn count_open_crossings(e1: &[seg::Point], e2: &[seg::Point], nodes: &[BBox]) -> usize {
    let mut n = 0;
    for a in e1.windows(2) {
        for b in e2.windows(2) {
            if let Some(p) = seg::segment_intersection(a[0], a[1], b[0], b[1]) {
                if !point_near_any_node(p, nodes) {
                    n += 1;
                }
            }
        }
    }
    n
}

fn point_near_any_node(p: seg::Point, nodes: &[BBox]) -> bool {
    let m = NODE_MARGIN_PX;
    nodes.iter().any(|n| {
        p.x >= n.x - m && p.x <= n.right() + m && p.y >= n.y - m && p.y <= n.bottom() + m
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    #[test]
    fn cross_clean_passes() {
        let svg = load(Path::new("tests/fixtures/cross-clean.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }

    #[test]
    fn open_crossing_fails() {
        let svg = load(Path::new("tests/fixtures/crossing.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "edge-crossings");
    }

    #[test]
    fn crossing_inside_node_is_excluded() {
        let svg = load(Path::new("tests/fixtures/crossing-at-node.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }
}
```

- [ ] **Step 2: Register the check in `src/checks/mod.rs`**

Add `pub mod edge_crossings;` with the other `pub mod` lines, and extend
`run_all`:

```rust
pub fn run_all(svg: &Svg) -> Vec<CheckResult> {
    vec![
        viewbox::check(svg),
        aspect::check(svg),
        node_overlap::check(svg),
        edge_crossings::check(svg),
    ]
}
```

- [ ] **Step 3: Run the check unit tests**

Run: `cargo test --bin diagctl edge_crossings` Expected: PASS (3 tests). If
`crossing_inside_node_is_excluded` fails (reports a crossing), confirm the node
rect in `crossing-at-node.svg` actually contains (100,60) and that the rect is
classified as a Node (filled).

- [ ] **Step 4: Add integration tests to `tests/cli.rs`**

Append:

```rust
#[test]
fn check_crossing_exits_1_naming_edge_crossings() {
    diagctl()
        .args(["check", "tests/fixtures/crossing.svg"])
        .assert()
        .code(1)
        .stdout(contains("edge-crossings"));
}

#[test]
fn check_cross_clean_exits_0() {
    diagctl()
        .args(["check", "tests/fixtures/cross-clean.svg"])
        .assert()
        .code(0)
        .stdout(contains("edge-crossings"));
}
```

- [ ] **Step 5: Run the full suite**

Run: `cargo test` Expected: PASS — all prior tests plus the new `seg`,
`edge_tests`, `edge_crossings`, and integration tests. (`crossing.svg` exits 1
because only edge-crossings fails; `cross-clean` exits 0 because all four checks
pass.)

- [ ] **Step 6: Commit**

```bash
git add src/checks/edge_crossings.rs src/checks/mod.rs tests/cli.rs
git commit -m "feat: Layer-2 edge-crossings check on the edge polyline foundation"
```

---

## Task 5: Final verification

- [ ] **Step 1: Format, lint, test**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Expected: fmt clean (run `cargo fmt --all` + re-commit if not), clippy no
warnings, all tests pass. Fix any clippy findings (e.g. a `needless_range_loop`
would not apply here since the check uses `.windows(2)`).

- [ ] **Step 2: Manual smoke check**

Run:

```bash
cargo run -q -- check tests/fixtures/crossing.svg; echo "rc=$?"
cargo run -q -- check tests/fixtures/cross-clean.svg; echo "rc=$?"
```

Expected: first prints
`FAIL  [L2] edge-crossings  1 open-space edge crossing(s)`, `rc=1`; second
prints a `PASS  [L2] edge-crossings  0 open-space edge crossing(s)` line,
`rc=0`.

- [ ] **Step 3: Commit any fmt/clippy fixups**

```bash
git add -A
git commit -m "chore: fmt + clippy clean for edge-crossings" || echo "nothing to commit"
```

---

## Self-Review

**Spec coverage:**

- `geom::seg` (Point, flatten, segment_intersection) → Task 1. ✓
- `geom::edges` polyline extraction (abs transform + flatten) → Task 3. ✓
- `edge-crossings` check (open-space crossings, node-bbox exclusion,
  threshold 0) → Task 4. ✓
- Registry wiring → Task 4 Step 2. ✓
- Fixtures (real clean + open crossing + crossing-at-node) → Task 2. ✓
- Node-bbox exclusion proven by `crossing-at-node` → Task 4 unit test. ✓
- TDD, real + hand-authored fixtures, no mocks → every check task asserts
  against committed SVGs. ✓
- No new deps; `nodes()`/`model()`/`svg::load` unchanged → confirmed (only
  additions). ✓
- Success criteria 1–6 → Task 1 (seg), Task 3 (edges), Task 4 (check +
  integration), Task 5 (fmt/clippy/test). ✓

**Placeholder scan:** No TBD / "handle edge cases" / "similar to". All code is
complete; fixtures rendered by exact commands; the node-margin semantics are
"expand node bbox by NODE_MARGIN_PX", not inset. ✓

**Type consistency:** `seg::Point{x,y}` + `Point::new`,
`flatten_cubic/flatten_quad`, `segment_intersection`,
`geom::edges(&Svg)->Vec<Vec<Point>>`, `edge_crossings::check(&Svg)->CheckResult`
id `"edge-crossings"`, constants
`FLATTEN_TOLERANCE_PX`/`CROSSING_THRESHOLD`/`NODE_MARGIN_PX` — used identically
across Tasks 1, 3, 4. ✓

```
```

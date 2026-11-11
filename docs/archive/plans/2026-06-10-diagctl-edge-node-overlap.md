# diagctl Edge–Node Overlap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Layer-2 `edge-node-overlap` check that flags an edge routing
through an unrelated node's bbox (a node that is not one of the edge's endpoint
nodes), milestone 2b-2 of diagctl#2.

**Architecture:** Reuses `geom::edges` (flattened edge polylines) and
`geom::nodes` (bboxes). A new pure-math `seg::segment_enters_rect` answers "does
this segment enter this rectangle". The check associates each edge with the
nodes near its endpoints (within a 4px margin, derived from a probe of real
renders) and flags the edge if any segment enters a non-endpoint node.

**Tech Stack:** Rust, usvg 0.45 (existing). No new dependencies — reuses the
2a/2b foundation.

---

## File Structure

- `src/geom/seg.rs` — MODIFY. Add
  `segment_enters_rect(a, b, rx, ry, rw, rh) -> bool`.
- `src/checks/edge_node_overlap.rs` — NEW. Layer-2 edge-through-unrelated-node
  check.
- `src/checks/mod.rs` — MODIFY. Register `edge_node_overlap` in `run_all`.
- `tests/fixtures/edge-through-node.svg` — NEW (hand-authored). Edge A→C through
  B.
- `tests/fixtures/edge-touches-endpoint.svg` — NEW (hand-authored). Edge ends
  inside its own nodes.
- `tests/cli.rs` — MODIFY. Integration test for edge-through-node (exit 1).
- (`tests/fixtures/d2-clean.svg` reused as the clean Pass fixture — already
  committed.)

### Type/interface contract

```rust
// src/geom/seg.rs (addition)
pub fn segment_enters_rect(a: Point, b: Point, rx: f32, ry: f32, rw: f32, rh: f32) -> bool;

// src/checks/edge_node_overlap.rs
pub const ENDPOINT_MARGIN_PX: f32 = 4.0;
pub fn check(svg: &crate::svg::Svg) -> crate::checks::CheckResult; // id "edge-node-overlap", layer 2
```

---

## Task 1: `seg::segment_enters_rect`

**Files:**

- Modify: `src/geom/seg.rs`

- [ ] **Step 1: Add the function above the `#[cfg(test)] mod tests` block in
      `src/geom/seg.rs`**

```rust
/// True iff segment a–b intersects the axis-aligned rect (rx, ry, rw, rh):
/// either endpoint inside, or the segment properly crosses one of the 4 sides.
pub fn segment_enters_rect(a: Point, b: Point, rx: f32, ry: f32, rw: f32, rh: f32) -> bool {
    let inside = |p: Point| p.x >= rx && p.x <= rx + rw && p.y >= ry && p.y <= ry + rh;
    if inside(a) || inside(b) {
        return true;
    }
    let tl = Point::new(rx, ry);
    let tr = Point::new(rx + rw, ry);
    let br = Point::new(rx + rw, ry + rh);
    let bl = Point::new(rx, ry + rh);
    segment_intersection(a, b, tl, tr).is_some()
        || segment_intersection(a, b, tr, br).is_some()
        || segment_intersection(a, b, br, bl).is_some()
        || segment_intersection(a, b, bl, tl).is_some()
}
```

- [ ] **Step 2: Add unit tests inside the existing `mod tests` in
      `src/geom/seg.rs`**

```rust
    #[test]
    fn segment_through_rect_enters() {
        assert!(segment_enters_rect(
            Point::new(0.0, 5.0),
            Point::new(20.0, 5.0),
            5.0,
            0.0,
            10.0,
            10.0
        ));
    }

    #[test]
    fn segment_missing_rect_does_not_enter() {
        assert!(!segment_enters_rect(
            Point::new(0.0, 50.0),
            Point::new(20.0, 50.0),
            5.0,
            0.0,
            10.0,
            10.0
        ));
    }

    #[test]
    fn segment_with_endpoint_inside_enters() {
        assert!(segment_enters_rect(
            Point::new(10.0, 5.0),
            Point::new(30.0, 5.0),
            5.0,
            0.0,
            10.0,
            10.0
        ));
    }
```

- [ ] **Step 3: Run the seg tests**

Run: `cargo test --bin diagctl geom::seg` Expected: PASS (8 tests — 5 existing +
3 new).

- [ ] **Step 4: Commit**

```bash
git add src/geom/seg.rs
git commit -m "feat: seg::segment_enters_rect for edge-node overlap"
```

---

## Task 2: Fixtures

**Files:**

- Create: `tests/fixtures/edge-through-node.svg`
- Create: `tests/fixtures/edge-touches-endpoint.svg`

- [ ] **Step 1: Hand-author `tests/fixtures/edge-through-node.svg`**

Three nodes in a row; edge A→C (y=70) routes straight through B's bbox. viewBox
300×140 (aspect 2.14 ≤ 2.5) so only `edge-node-overlap` fails:

```xml
<svg xmlns="http://www.w3.org/2000/svg" width="300" height="140" viewBox="0 0 300 140">
  <g>
    <g><rect x="10" y="55" width="40" height="30" fill="#eeeeee" stroke="#333333"/></g>
    <g><rect x="130" y="55" width="40" height="30" fill="#eeeeee" stroke="#333333"/></g>
    <g><rect x="250" y="55" width="40" height="30" fill="#eeeeee" stroke="#333333"/></g>
    <path d="M 50 70 L 250 70" fill="none" stroke="#333333"/>
  </g>
</svg>
```

- [ ] **Step 2: Hand-author `tests/fixtures/edge-touches-endpoint.svg`**

Edge whose endpoints sit deep inside its own two nodes — must NOT be flagged
(endpoint exclusion). viewBox 200×120 (aspect 1.67):

```xml
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="120" viewBox="0 0 200 120">
  <g>
    <g><rect x="20" y="40" width="50" height="40" fill="#eeeeee" stroke="#333333"/></g>
    <g><rect x="130" y="40" width="50" height="40" fill="#eeeeee" stroke="#333333"/></g>
    <path d="M 40 60 L 155 60" fill="none" stroke="#333333"/>
  </g>
</svg>
```

- [ ] **Step 3: Verify they exist**

Run:
`ls -l tests/fixtures/edge-through-node.svg tests/fixtures/edge-touches-endpoint.svg`
Expected: both exist and are non-empty.

- [ ] **Step 4: Commit**

```bash
git add tests/fixtures/edge-through-node.svg tests/fixtures/edge-touches-endpoint.svg
git commit -m "test: edge-node-overlap fixtures (through-node + endpoint-touch)"
```

---

## Task 3: `edge-node-overlap` check + registry + integration

**Files:**

- Create: `src/checks/edge_node_overlap.rs`
- Modify: `src/checks/mod.rs`
- Modify: `tests/cli.rs`

- [ ] **Step 1: Write `src/checks/edge_node_overlap.rs` with failing unit
      tests**

```rust
use crate::checks::{CheckResult, Status};
use crate::geom::{self, seg, BBox};
use crate::svg::Svg;

pub const ENDPOINT_MARGIN_PX: f32 = 4.0;

pub fn check(svg: &Svg) -> CheckResult {
    let edges = geom::edges(svg);
    let nodes = geom::nodes(svg);
    let mut violations = 0usize;
    for e in &edges {
        if e.len() < 2 {
            continue;
        }
        let first = e[0];
        let last = e[e.len() - 1];
        for n in &nodes {
            // Endpoint node: the edge legitimately terminates here — not a defect.
            if point_near(first, n) || point_near(last, n) {
                continue;
            }
            let enters = e
                .windows(2)
                .any(|w| seg::segment_enters_rect(w[0], w[1], n.x, n.y, n.w, n.h));
            if enters {
                violations += 1;
            }
        }
    }
    let status = if violations == 0 {
        Status::Pass
    } else {
        Status::Fail
    };
    CheckResult {
        id: "edge-node-overlap",
        layer: 2,
        status,
        detail: format!("{violations} edge-through-unrelated-node case(s)"),
    }
}

fn point_near(p: seg::Point, n: &BBox) -> bool {
    let m = ENDPOINT_MARGIN_PX;
    p.x >= n.x - m && p.x <= n.right() + m && p.y >= n.y - m && p.y <= n.bottom() + m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    #[test]
    fn d2_clean_passes() {
        let svg = load(Path::new("tests/fixtures/d2-clean.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }

    #[test]
    fn edge_through_node_fails() {
        let svg = load(Path::new("tests/fixtures/edge-through-node.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "edge-node-overlap");
    }

    #[test]
    fn edge_into_own_endpoint_passes() {
        let svg = load(Path::new("tests/fixtures/edge-touches-endpoint.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }
}
```

- [ ] **Step 2: Register the check in `src/checks/mod.rs`**

Add `pub mod edge_node_overlap;` with the other `pub mod` lines (alphabetical:
after `edge_crossings`), and extend `run_all`:

```rust
pub fn run_all(svg: &Svg) -> Vec<CheckResult> {
    vec![
        viewbox::check(svg),
        aspect::check(svg),
        node_overlap::check(svg),
        edge_crossings::check(svg),
        edge_node_overlap::check(svg),
    ]
}
```

- [ ] **Step 3: Run the check unit tests**

Run: `cargo test --bin diagctl edge_node_overlap` Expected: PASS (3 tests). If
`edge_through_node_fails` passes-as-Pass, confirm node B in
`edge-through-node.svg` is between the edge endpoints and that the edge's y (70)
lies within B's bbox (55–85). If `edge_into_own_endpoint_passes` fails, confirm
the edge's endpoints (40,60)/(155,60) fall within A/B expanded by 4px.

- [ ] **Step 4: Add an integration test to `tests/cli.rs`**

Append:

```rust
#[test]
fn check_edge_through_node_exits_1() {
    diagctl()
        .args(["check", "tests/fixtures/edge-through-node.svg"])
        .assert()
        .code(1)
        .stdout(contains("edge-node-overlap"));
}
```

- [ ] **Step 5: Run the full suite**

Run: `cargo test` Expected: PASS — all prior tests plus the new `seg`,
`edge_node_overlap`, and integration tests. (`edge-through-node.svg` exits 1
because only edge-node-overlap fails.)

- [ ] **Step 6: Commit**

```bash
git add src/checks/edge_node_overlap.rs src/checks/mod.rs tests/cli.rs
git commit -m "feat: Layer-2 edge-node-overlap check"
```

---

## Task 4: Final verification

- [ ] **Step 1: Format, lint, test**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Expected: fmt clean (run `cargo fmt --all` + re-commit if not), clippy no
warnings, all tests pass.

- [ ] **Step 2: Manual smoke check**

Run:

```bash
cargo run -q -- check tests/fixtures/edge-through-node.svg; echo "rc=$?"
cargo run -q -- check tests/fixtures/d2-clean.svg; echo "rc=$?"
```

Expected: first prints a
`FAIL  [L2] edge-node-overlap  1 edge-through-unrelated-node
case(s)` line,
`rc=1`; second prints all PASS lines including `edge-node-overlap`, `rc=0`.

- [ ] **Step 3: Commit any fmt/clippy fixups**

```bash
git add -A
git commit -m "chore: fmt + clippy clean for edge-node-overlap" || echo "nothing to commit"
```

---

## Self-Review

**Spec coverage:**

- `seg::segment_enters_rect` (endpoint-inside or crosses a side) → Task 1. ✓
- `edge-node-overlap` check (endpoint-proximity association, 4px margin,
  through-test) → Task 3. ✓
- Registry wiring → Task 3 Step 2. ✓
- Fixtures (reuse d2-clean; through-node fail; endpoint-touch pass) → Task 2 (+
  existing). ✓
- Endpoint exclusion proven by `edge-touches-endpoint` → Task 3 unit test. ✓
- TDD, real + hand-authored fixtures, no mocks → every check task asserts
  against committed SVGs. ✓
- No new deps; reuses geom::edges/nodes/seg → confirmed (only additions). ✓
- Success criteria 1–6 → Task 1 (seg), Task 3 (check + integration), Task 4
  (fmt/clippy/test). ✓

**Placeholder scan:** No TBD / "handle edge cases" / "similar to". All code
complete; fixtures hand-authored with exact coordinates; failing fixture sized
so aspect (2.14) and the other three checks pass, isolating edge-node-overlap. ✓

**Type consistency:** `segment_enters_rect(Point,Point,f32,f32,f32,f32)->bool`,
`edge_node_overlap::check(&Svg)->CheckResult` id `"edge-node-overlap"`, const
`ENDPOINT_MARGIN_PX`, `point_near(seg::Point,&BBox)`, reuse of
`geom::edges`/`geom::nodes` returning `Vec<Vec<seg::Point>>`/`Vec<BBox>` —
consistent across Tasks 1, 3. ✓

```
```

# diagctl Geometry Foundation + Node–Node Overlap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `geom` module that classifies a rendered SVG into
nodes/edges/labels/background from `usvg`'s tree, and a Layer-2 `node-overlap`
check built on it (milestone 2a of diagctl#2).

**Architecture:** `geom.rs` walks the `usvg::Tree` (which `svg::load` already
produces), finds the "content group", and classifies each top-level unit by
group structure + fill (node = filled path ≥50% of unit bbox; background =
filled path ≈ canvas; edge = open stroked path). `checks/node_overlap.rs` runs
pairwise AABB intersection over node bboxes (containment + 1px tolerance
excluded) and reports via the existing `CheckResult` registry.

**Tech Stack:** Rust, `usvg` 0.45 (already a dep; API validated by probe), no
new dependencies. Fixtures rendered by D2 0.7 + PlantUML 1.2025 (committed as
static SVGs; renderers not needed at test time).

---

## File Structure

- `src/geom.rs` — NEW. `BBox` type + math; `Kind`/`Element`; `model()`/`nodes()`
  classification.
- `src/checks/node_overlap.rs` — NEW. Layer-2 node–node overlap check.
- `src/checks/mod.rs` — MODIFY. Register `node_overlap` in `run_all`.
- `src/main.rs` — MODIFY. Add `mod geom;`.
- `tests/fixtures/d2-clean.svg` — NEW (rendered). 2 separated nodes + edge +
  background.
- `tests/fixtures/plantuml-clean.svg` — NEW (rendered). 2 multi-sub-rect nodes +
  edge.
- `tests/fixtures/overlap.svg` — NEW (hand-authored). 2 overlapping node groups.
- `tests/cli.rs` — MODIFY. Add integration tests for d2-clean (exit 0) + overlap
  (exit 1).

### Type/interface contract (keep names stable across tasks)

```rust
// src/geom.rs
pub struct BBox { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }
//   methods: area, right, bottom, intersection_area, intersects, contains, inset
pub enum Kind { Node, Edge, Label, Background, Other }   // derive Debug, Clone, Copy, PartialEq, Eq
pub struct Element { pub kind: Kind, pub bbox: BBox }     // derive Debug, Clone
pub fn model(svg: &crate::svg::Svg) -> Vec<Element>;
pub fn nodes(svg: &crate::svg::Svg) -> Vec<BBox>;

// tunable constants (geom.rs)
pub const BG_CANVAS_FRACTION: f32 = 0.95;
pub const NODE_FILL_FRACTION: f32 = 0.50;

// src/checks/node_overlap.rs
pub const OVERLAP_TOLERANCE_PX: f32 = 1.0;
pub fn check(svg: &crate::svg::Svg) -> crate::checks::CheckResult;  // id "node-overlap", layer 2
```

> **usvg API (validated by a throwaway probe against real D2 + PlantUML
> renders):** `Tree::root() -> &Group`, `Group::children() -> &[Node]`,
> `Node::{Group(Box<Group>),Path(Box<Path>),Image(Box<Image>),Text(Box<Text>)}`,
> `{Group,Path,Image,Text}::abs_bounding_box() -> usvg::Rect` with
> `.x()/.y()/.width()/.height() -> f32`, `Path::fill() -> Option<&Fill>`,
> `Path::stroke() -> Option<&Stroke>`. If a signature differs in the resolved
> patch version, fix to the compiler — expected TDD drift.

---

## Task 1: `BBox` type and geometry math

**Files:**

- Create: `src/geom.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write `src/geom.rs` with `BBox` + failing-by-unimplemented math
      and tests**

```rust
//! Geometry foundation: classify a rendered SVG into nodes/edges/labels/background.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl BBox {
    pub fn area(&self) -> f32 {
        self.w.max(0.0) * self.h.max(0.0)
    }
    pub fn right(&self) -> f32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
    pub fn intersection_area(&self, o: &BBox) -> f32 {
        let ox = (self.right().min(o.right()) - self.x.max(o.x)).max(0.0);
        let oy = (self.bottom().min(o.bottom()) - self.y.max(o.y)).max(0.0);
        ox * oy
    }
    pub fn intersects(&self, o: &BBox) -> bool {
        self.intersection_area(o) > 0.0
    }
    pub fn contains(&self, o: &BBox) -> bool {
        self.x <= o.x && self.y <= o.y && self.right() >= o.right() && self.bottom() >= o.bottom()
    }
    pub fn inset(&self, d: f32) -> BBox {
        BBox {
            x: self.x + d,
            y: self.y + d,
            w: self.w - 2.0 * d,
            h: self.h - 2.0 * d,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(x: f32, y: f32, w: f32, h: f32) -> BBox {
        BBox { x, y, w, h }
    }

    #[test]
    fn overlapping_boxes_intersect() {
        let a = b(0.0, 0.0, 10.0, 10.0);
        let c = b(5.0, 5.0, 10.0, 10.0);
        assert!(a.intersects(&c));
        assert_eq!(a.intersection_area(&c), 25.0);
    }

    #[test]
    fn touching_edges_do_not_intersect() {
        let a = b(0.0, 0.0, 10.0, 10.0);
        let c = b(10.0, 0.0, 10.0, 10.0);
        assert!(!a.intersects(&c));
        assert_eq!(a.intersection_area(&c), 0.0);
    }

    #[test]
    fn containment_detected_one_way() {
        let outer = b(0.0, 0.0, 100.0, 100.0);
        let inner = b(10.0, 10.0, 20.0, 20.0);
        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn inset_shrinks_on_all_sides() {
        let a = b(0.0, 0.0, 10.0, 10.0).inset(1.0);
        assert_eq!(a, b(1.0, 1.0, 8.0, 8.0));
    }
}
```

- [ ] **Step 2: Add `mod geom;` to `src/main.rs`**

The module list becomes (alphabetical, matching existing style):

```rust
mod checks;
mod cli;
mod commands;
mod geom;
mod svg;
```

(Leave the rest of `main.rs` unchanged.)

- [ ] **Step 3: Run the math tests**

Run: `cargo test --bin diagctl geom::tests` Expected: PASS (4 tests). The math
is implemented inline, so this is green immediately — these tests lock the
geometry contract the rest of the module depends on.

- [ ] **Step 4: Commit**

```bash
git add src/geom.rs src/main.rs
git commit -m "feat: BBox geometry primitives for Layer-2 checks"
```

---

## Task 2: Committed SVG fixtures

**Files:**

- Create: `tests/fixtures/d2-clean.svg` (rendered)
- Create: `tests/fixtures/plantuml-clean.svg` (rendered)
- Create: `tests/fixtures/overlap.svg` (hand-authored)

- [ ] **Step 1: Render `d2-clean.svg`**

```bash
printf 'a -> b\n' > /tmp/diagctl-d2.d2
D2_LAYOUT=dagre d2 --omit-version /tmp/diagctl-d2.d2 tests/fixtures/d2-clean.svg
```

- [ ] **Step 2: Render `plantuml-clean.svg`**

```bash
printf '@startuml\nleft to right direction\n[A] --> [B]\n@enduml\n' > /tmp/diagctl-puml.puml
plantuml -tsvg /tmp/diagctl-puml.puml
cp /tmp/diagctl-puml.svg tests/fixtures/plantuml-clean.svg
```

- [ ] **Step 3: Hand-author `tests/fixtures/overlap.svg`**

Two filled node groups whose bboxes overlap; viewBox present and aspect 200/120
= 1.67 (≤ 2.5) so only `node-overlap` will fail:

```xml
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="120" viewBox="0 0 200 120">
  <g>
    <g>
      <rect x="20" y="20" width="80" height="60" fill="#eeeeee" stroke="#333333"/>
      <text x="55" y="55" font-size="14">A</text>
    </g>
    <g>
      <rect x="70" y="40" width="80" height="60" fill="#eeeeee" stroke="#333333"/>
      <text x="105" y="75" font-size="14">B</text>
    </g>
  </g>
</svg>
```

- [ ] **Step 4: Verify the renders parse and have the expected shape**

Run:

```bash
ls -l tests/fixtures/*.svg
grep -o 'viewBox="[^"]*"' tests/fixtures/d2-clean.svg | head -1
grep -o 'viewBox="[^"]*"' tests/fixtures/plantuml-clean.svg | head -1
```

Expected: all three files exist and are non-empty; `d2-clean.svg` root viewBox ≈
`0 0 255 434`; `plantuml-clean.svg` has a viewBox. (Exact numbers may differ
slightly by renderer build — that is fine; tests assert structural counts, not
pixel values.)

- [ ] **Step 5: Commit**

```bash
git add tests/fixtures/d2-clean.svg tests/fixtures/plantuml-clean.svg tests/fixtures/overlap.svg
git commit -m "test: committed D2/PlantUML/overlap SVG fixtures for geom"
```

---

## Task 3: `geom::model` classification

**Files:**

- Modify: `src/geom.rs`

- [ ] **Step 1: Add `Kind`, `Element`, the constants, and failing classification
      tests**

Append to `src/geom.rs` (above the existing `#[cfg(test)] mod tests`):

```rust
use crate::svg::Svg;
use usvg::{Group, Node};

pub const BG_CANVAS_FRACTION: f32 = 0.95;
pub const NODE_FILL_FRACTION: f32 = 0.50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Node,
    Edge,
    Label,
    Background,
    Other,
}

#[derive(Debug, Clone)]
pub struct Element {
    pub kind: Kind,
    pub bbox: BBox,
}

fn rect_to_bbox(r: usvg::Rect) -> BBox {
    BBox { x: r.x(), y: r.y(), w: r.width(), h: r.height() }
}

fn node_bbox(n: &Node) -> BBox {
    let r = match n {
        Node::Group(g) => g.abs_bounding_box(),
        Node::Path(p) => p.abs_bounding_box(),
        Node::Image(i) => i.abs_bounding_box(),
        Node::Text(t) => t.abs_bounding_box(),
    };
    rect_to_bbox(r)
}

/// Apply `f` to every Path under `n` (recursing into groups).
fn for_each_path(n: &Node, f: &mut impl FnMut(&usvg::Path)) {
    match n {
        Node::Path(p) => f(p),
        Node::Group(g) => {
            for c in g.children() {
                for_each_path(c, f);
            }
        }
        _ => {}
    }
}

/// Descend through trivial single-child wrapper groups to the diagram "content" group.
fn content_group(root: &Group) -> &Group {
    let mut g = root;
    while g.children().len() == 1 {
        match &g.children()[0] {
            Node::Group(inner) => g = inner.as_ref(),
            _ => break,
        }
    }
    g
}

fn classify(unit: &Node, canvas_area: f32) -> Element {
    if let Node::Text(t) = unit {
        return Element { kind: Kind::Label, bbox: rect_to_bbox(t.abs_bounding_box()) };
    }
    let bbox = node_bbox(unit);
    let mut max_filled = 0.0f32;
    let mut has_open_stroke = false;
    for_each_path(unit, &mut |p| {
        let a = rect_to_bbox(p.abs_bounding_box()).area();
        if p.fill().is_some() {
            if a > max_filled {
                max_filled = a;
            }
        } else if p.stroke().is_some() {
            has_open_stroke = true;
        }
    });
    let kind = if max_filled >= BG_CANVAS_FRACTION * canvas_area {
        Kind::Background
    } else if bbox.area() > 0.0 && max_filled >= NODE_FILL_FRACTION * bbox.area() {
        Kind::Node
    } else if has_open_stroke {
        Kind::Edge
    } else {
        Kind::Other
    };
    Element { kind, bbox }
}

pub fn model(svg: &Svg) -> Vec<Element> {
    let size = svg.tree.size();
    let canvas_area = size.width() * size.height();
    let content = content_group(svg.tree.root());
    content.children().iter().map(|u| classify(u, canvas_area)).collect()
}

pub fn nodes(svg: &Svg) -> Vec<BBox> {
    model(svg)
        .into_iter()
        .filter(|e| e.kind == Kind::Node)
        .map(|e| e.bbox)
        .collect()
}

#[cfg(test)]
mod model_tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    fn count(svg: &Svg, k: Kind) -> usize {
        model(svg).into_iter().filter(|e| e.kind == k).count()
    }

    #[test]
    fn d2_clean_has_two_nodes_and_background() {
        let svg = load(Path::new("tests/fixtures/d2-clean.svg")).unwrap();
        assert_eq!(count(&svg, Kind::Node), 2, "d2 node count");
        assert_eq!(count(&svg, Kind::Background), 1, "d2 background count");
        assert!(count(&svg, Kind::Edge) >= 1, "d2 should have an edge");
    }

    #[test]
    fn plantuml_clean_collapses_subrects_to_two_nodes() {
        let svg = load(Path::new("tests/fixtures/plantuml-clean.svg")).unwrap();
        // Each PlantUML node wraps several filled sub-rects; grouping must yield 2 nodes, not more.
        assert_eq!(count(&svg, Kind::Node), 2, "plantuml node count");
    }

    #[test]
    fn overlap_fixture_has_two_nodes() {
        let svg = load(Path::new("tests/fixtures/overlap.svg")).unwrap();
        assert_eq!(count(&svg, Kind::Node), 2, "overlap node count");
    }
}
```

- [ ] **Step 2: Run the classification tests to verify they pass**

Run: `cargo test --bin diagctl geom::model_tests` Expected: PASS (3 tests). If a
count is off:

- If `plantuml` yields >2 nodes, the content-group descent stopped too shallow —
  confirm `content_group` descends to the group whose children are the entities
  (inspect with a temporary `dbg!(model(&svg))`).
- If `d2` background is counted as a Node, confirm the `Background` branch
  (canvas fraction) runs **before** the `Node` branch (it does, above).

These tests are written before the assertions are known to pass only in the
sense that the implementation lands in the same step; the counts are derived
from a verified probe of the real renders, so they are the contract. Treat any
mismatch as a real bug to fix here, not a number to massage.

- [ ] **Step 3: Commit**

```bash
git add src/geom.rs
git commit -m "feat: geom::model classifies nodes/edges/background from usvg tree"
```

---

## Task 4: `node-overlap` check + registry wiring + integration tests

**Files:**

- Create: `src/checks/node_overlap.rs`
- Modify: `src/checks/mod.rs`
- Modify: `tests/cli.rs`

- [ ] **Step 1: Write `src/checks/node_overlap.rs` with failing unit tests**

```rust
use crate::checks::{CheckResult, Status};
use crate::geom;
use crate::svg::Svg;

pub const OVERLAP_TOLERANCE_PX: f32 = 1.0;

pub fn check(svg: &Svg) -> CheckResult {
    let nodes = geom::nodes(svg);
    let t = OVERLAP_TOLERANCE_PX / 2.0;
    let mut overlaps = 0usize;
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let (a, b) = (nodes[i], nodes[j]);
            // Skip true containment (a node inside a strictly larger container).
            if (a.contains(&b) && a.area() > b.area()) || (b.contains(&a) && b.area() > a.area()) {
                continue;
            }
            if a.inset(t).intersects(&b.inset(t)) {
                overlaps += 1;
            }
        }
    }
    let status = if overlaps == 0 { Status::Pass } else { Status::Fail };
    let detail = if overlaps == 0 {
        format!("{} node(s), no overlapping pairs", nodes.len())
    } else {
        format!("{overlaps} overlapping node pair(s) among {} nodes", nodes.len())
    };
    CheckResult { id: "node-overlap", layer: 2, status, detail }
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
    fn plantuml_clean_passes() {
        let svg = load(Path::new("tests/fixtures/plantuml-clean.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }

    #[test]
    fn overlap_fixture_fails() {
        let svg = load(Path::new("tests/fixtures/overlap.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "node-overlap");
    }
}
```

- [ ] **Step 2: Register the check in `src/checks/mod.rs`**

Add `pub mod node_overlap;` next to the other `pub mod` lines, and extend
`run_all`:

```rust
pub fn run_all(svg: &Svg) -> Vec<CheckResult> {
    vec![
        viewbox::check(svg),
        aspect::check(svg),
        node_overlap::check(svg),
    ]
}
```

- [ ] **Step 3: Run the check unit tests**

Run: `cargo test --bin diagctl node_overlap` Expected: PASS (3 tests). If
`overlap_fixture_fails` passes-as-Pass, the two rects were classified as
containment or did not both become Nodes — re-check `overlap.svg` coordinates
(they must partially overlap, neither containing the other).

- [ ] **Step 4: Add integration tests to `tests/cli.rs`**

Append:

```rust
#[test]
fn check_d2_clean_exits_0() {
    diagctl()
        .args(["check", "tests/fixtures/d2-clean.svg"])
        .assert()
        .code(0)
        .stdout(contains("node-overlap"));
}

#[test]
fn check_overlap_exits_1_naming_node_overlap() {
    diagctl()
        .args(["check", "tests/fixtures/overlap.svg"])
        .assert()
        .code(1)
        .stdout(contains("node-overlap"));
}
```

- [ ] **Step 5: Run the full suite**

Run: `cargo test` Expected: PASS — all prior tests plus the new `geom`,
`node_overlap`, and integration tests. (`d2-clean` exits 0 because it passes
viewBox + aspect + node-overlap; `overlap` exits 1 because only node-overlap
fails.)

- [ ] **Step 6: Commit**

```bash
git add src/checks/node_overlap.rs src/checks/mod.rs tests/cli.rs
git commit -m "feat: Layer-2 node-overlap check on the geom foundation"
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

Expected: fmt clean (run `cargo fmt --all` and re-commit if not), clippy no
warnings, all tests pass. Fix any clippy findings (e.g. collapse the
`if p.fill().is_some()` / inner `if` with `&&` if clippy suggests) and re-run.

- [ ] **Step 2: Manual smoke check**

Run:

```bash
cargo run -q -- check tests/fixtures/d2-clean.svg; echo "rc=$?"
cargo run -q -- check tests/fixtures/overlap.svg; echo "rc=$?"
```

Expected: first prints three PASS lines incl. `node-overlap`, `rc=0`; second
prints a `FAIL  [L2] node-overlap` line, `rc=1`.

- [ ] **Step 3: Commit any fmt/clippy fixups**

```bash
git add -A
git commit -m "chore: fmt + clippy clean for geom node-overlap" || echo "nothing to commit"
```

---

## Self-Review

**Spec coverage:**

- `geom` foundation (BBox + classification) → Tasks 1, 3. ✓
- Group-structure + fill classification (content-group descent,
  node/edge/background/label) → Task 3. ✓
- node–node overlap check (containment + 1px tolerance, Fail on ≥1 pair) →
  Task 4. ✓
- Registry wiring → Task 4 Step 2. ✓
- Real D2 + PlantUML fixtures + hand-authored overlap → Task 2. ✓
- TDD, real fixtures, no mocks → every check task asserts against committed
  SVGs. ✓
- `svg::load` unchanged (no font loading) → not modified by any task. ✓
- Success criteria 1–6 → Task 3 (counts), Task 4 (pass/pass/fail + integration),
  Task 5 (fmt/clippy/test). ✓
- Plantuml-clean used only in unit tests (aspect strip) → Task 4 unit test, not
  integration. ✓

**Placeholder scan:** No TBD / "handle edge cases" / "similar to". The fixture
renders are generated by exact commands; counts are derived from a verified usvg
probe. ✓

**Type consistency:** `BBox{x,y,w,h}` + methods,
`Kind::{Node,Edge,Label,Background,Other}`, `Element{kind,bbox}`,
`model(&Svg)->Vec<Element>`, `nodes(&Svg)->Vec<BBox>`,
`node_overlap::check(&Svg)->CheckResult` with id `"node-overlap"`, constants
`BG_CANVAS_FRACTION`/`NODE_FILL_FRACTION`/`OVERLAP_TOLERANCE_PX` — used
identically across Tasks 1, 3, 4. ✓

# Label Overflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the Layer-2 `label-overflow` check that flags a text label whose
rendered bounding box extends past the viewBox (canvas) edge.

**Architecture:** A single permissively-licensed metrics font (Roboto Regular,
Apache-2.0) is embedded in the binary. A new `svg::shaped_tree` helper re-parses
the SVG with a font DB containing only that font — all generic and named
families resolve to it — giving deterministic, platform-identical text geometry.
The check reads each `Text` node's absolute bbox from that shaped tree and fails
any label protruding past the `(0,0,W,H)` canvas by more than a
size-proportional tolerance. The shared `svg::load` path and the other 9 checks
are untouched.

**Tech Stack:** Rust, `usvg` 0.45.1 (+ its `fontdb` 0.23 / rustybuzz text stack,
already compiled in), `roxmltree`, `clap`, `anyhow`.

## Global Constraints

- Rust toolchain **1.95+**.
- **No new crate dependencies** — reuse the `usvg` text stack already present;
  the font is embedded via `include_bytes!` (compile-time, not a dependency).
- Bundled font must be **permissively licensed** and **attributed** (MIT
  project): ship its LICENSE alongside the vendored file.
- Every task ends green: `cargo test`, `cargo fmt --check`, and
  `cargo clippy -- -D warnings` all clean.
- Check contract: `pub fn check(svg: &Svg) -> CheckResult` returning
  `CheckResult { id: "label-overflow", layer: 2, status, detail }`; exit-code
  contract (0 pass / 1 fail / 2 error) unchanged.
- The check is **additive, never a blocker**: any shaping/parse failure returns
  `Pass` (skipped), never `Fail`.

---

### Task 1: Vendor the metrics font and add `svg::shaped_tree`

**Files:**

- Create: `assets/fonts/Roboto-Regular.ttf` (vendored binary)
- Create: `assets/fonts/LICENSE-Roboto.txt` (Apache-2.0 text)
- Create: `tests/fixtures/label-fits.svg`
- Modify: `src/svg.rs`
- Test: `src/svg.rs` (unit test module)

**Interfaces:**

- Consumes: `Svg { raw: String, tree: usvg::Tree }` and
  `svg::load(path) -> anyhow::Result<Svg>` (existing).
- Produces: `pub fn shaped_tree(svg: &Svg) -> anyhow::Result<usvg::Tree>` — a
  font-enabled re-parse of `svg.raw` in which all text shapes with the bundled
  font.

- [ ] **Step 1: Vendor the font and its license**

Download Roboto Regular (Apache-2.0, ~170 KB) and its license, then verify:

```bash
mkdir -p assets/fonts
curl -fL -o assets/fonts/Roboto-Regular.ttf \
  https://github.com/google/fonts/raw/main/apache/roboto/static/Roboto-Regular.ttf
curl -fL -o assets/fonts/LICENSE-Roboto.txt \
  https://github.com/google/fonts/raw/main/apache/roboto/LICENSE.txt
file assets/fonts/Roboto-Regular.ttf   # expect: TrueType Font data
wc -c  assets/fonts/Roboto-Regular.ttf # expect: ~100KB–800KB, non-trivial
```

Expected: `file` reports TrueType/OpenType font data and size is non-trivial.
Fallback if the URL 404s: fetch `DejaVuSans.ttf` from the dejavu-fonts release
instead, place it at `assets/fonts/DejaVuSans.ttf` with its `LICENSE`, and in
Step 4 set the embedded path and `BUNDLED_FAMILY` to `"DejaVu Sans"`.

- [ ] **Step 2: Create the `label-fits.svg` fixture (short label, well inside
      canvas)**

Create `tests/fixtures/label-fits.svg`:

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 200">
  <rect x="0" y="0" width="400" height="200" fill="#ffffff"/>
  <text x="160" y="100" font-size="16" font-family="Arial, sans-serif" fill="#111111">Fits</text>
</svg>
```

- [ ] **Step 3: Write the failing test for `shaped_tree`**

Add to the `#[cfg(test)] mod tests` in `src/svg.rs`:

```rust
    #[test]
    fn shaped_tree_gives_text_nonzero_bbox() {
        let svg = load(Path::new("tests/fixtures/label-fits.svg")).unwrap();
        let tree = shaped_tree(&svg).unwrap();

        fn any_text_shaped(g: &usvg::Group) -> bool {
            g.children().iter().any(|n| match n {
                usvg::Node::Group(gg) => any_text_shaped(gg),
                usvg::Node::Text(t) => t.abs_bounding_box().width() > 0.0,
                _ => false,
            })
        }

        assert!(
            any_text_shaped(tree.root()),
            "bundled font failed to shape text to a non-zero-width bbox"
        );
    }
```

- [ ] **Step 4: Run the test to verify it fails**

Run: `cargo test --lib svg::tests::shaped_tree_gives_text_nonzero_bbox`
Expected: FAIL to compile — `cannot find function shaped_tree in this scope`.

- [ ] **Step 5: Implement `shaped_tree` and the embedded font**

Add to `src/svg.rs` (after the `load` function):

```rust
/// Roboto Regular (Apache-2.0) — a single deterministic metrics font embedded so text
/// shaping is identical on every platform. Attribution: assets/fonts/LICENSE-Roboto.txt.
const BUNDLED_FONT: &[u8] = include_bytes!("../assets/fonts/Roboto-Regular.ttf");
const BUNDLED_FAMILY: &str = "Roboto";

/// Re-parse the SVG with a font DB containing exactly the bundled font, with every generic
/// family pointed at it and it set as the default family. Any `font-family` therefore
/// resolves to the bundled font, yielding deterministic, platform-identical text geometry.
///
/// Used only by the `label-overflow` check; the shared `load` path keeps its empty-font tree
/// so the other checks are unaffected.
pub fn shaped_tree(svg: &Svg) -> anyhow::Result<usvg::Tree> {
    let mut opt = usvg::Options::default();
    {
        let db = opt.fontdb_mut();
        db.load_font_data(BUNDLED_FONT.to_vec());
        db.set_serif_family(BUNDLED_FAMILY);
        db.set_sans_serif_family(BUNDLED_FAMILY);
        db.set_monospace_family(BUNDLED_FAMILY);
        db.set_cursive_family(BUNDLED_FAMILY);
        db.set_fantasy_family(BUNDLED_FAMILY);
    }
    opt.font_family = BUNDLED_FAMILY.to_string();
    Ok(usvg::Tree::from_str(&svg.raw, &opt)?)
}
```

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test --lib svg::tests::shaped_tree_gives_text_nonzero_bbox`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add assets/fonts/Roboto-Regular.ttf assets/fonts/LICENSE-Roboto.txt \
        tests/fixtures/label-fits.svg src/svg.rs
git commit -m "feat: embed metrics font + svg::shaped_tree for deterministic text geometry"
```

---

### Task 2: The `label_overflow` check, registration, fixtures, and tests

**Files:**

- Create: `src/checks/label_overflow.rs`
- Create: `tests/fixtures/label-overflow.svg`
- Modify: `src/checks/mod.rs` (declare module + register in `run_all`)
- Test: `src/checks/label_overflow.rs` (unit test module)

**Interfaces:**

- Consumes: `svg::shaped_tree` (Task 1); `crate::checks::{CheckResult, Status}`;
  `crate::svg::Svg`; `usvg::{Group, Node, Rect}`.
- Produces: `pub fn check(svg: &Svg) -> CheckResult`;
  `pub const OVERFLOW_ABS_FLOOR_PX: f32`;
  `pub const OVERFLOW_SIZE_FRACTION: f32`.

- [ ] **Step 1: Create the `label-overflow.svg` fail fixture**

Create `tests/fixtures/label-overflow.svg` — a long label starting near the
right edge so it clearly overruns the viewBox; authored so every other check
passes (viewBox present, no fixed dims, aspect 200/120 ≈ 1.67 ≤ 2.5, no
nodes/edges, no strokes, font-family has a fallback chain):

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 120">
  <rect x="0" y="0" width="200" height="120" fill="#ffffff"/>
  <text x="120" y="64" font-size="20" font-family="Arial, sans-serif" fill="#111111">Overflowing label text</text>
</svg>
```

- [ ] **Step 2: Write the check module with failing tests**

Create `src/checks/label_overflow.rs`:

```rust
use crate::checks::{CheckResult, Status};
use crate::svg::{self, Svg};

/// Antialiasing / rounding headroom, in px.
pub const OVERFLOW_ABS_FLOOR_PX: f32 = 2.0;
/// Fraction of a label's own size tolerated past an edge — absorbs the bounded width
/// difference between the bundled font and the diagram's real font.
pub const OVERFLOW_SIZE_FRACTION: f32 = 0.15;

pub fn check(svg: &Svg) -> CheckResult {
    let tree = match svg::shaped_tree(svg) {
        Ok(t) => t,
        Err(_) => {
            return CheckResult {
                id: "label-overflow",
                layer: 2,
                status: Status::Pass,
                detail: "unshapeable; skipped".to_string(),
            };
        }
    };
    let size = tree.size();
    let (cw, ch) = (size.width(), size.height());

    let mut bboxes: Vec<usvg::Rect> = Vec::new();
    collect_text_bboxes(tree.root(), &mut bboxes);

    let mut overflowing = 0usize;
    let mut worst = 0.0f32;
    for b in &bboxes {
        let (x, y, w, h) = (b.x(), b.y(), b.width(), b.height());
        let tol_x = OVERFLOW_ABS_FLOOR_PX.max(OVERFLOW_SIZE_FRACTION * w);
        let tol_y = OVERFLOW_ABS_FLOOR_PX.max(OVERFLOW_SIZE_FRACTION * h);
        let h_over = (-x).max(x + w - cw).max(0.0);
        let v_over = (-y).max(y + h - ch).max(0.0);
        if h_over > tol_x || v_over > tol_y {
            overflowing += 1;
            worst = worst.max(h_over).max(v_over);
        }
    }

    let status = if overflowing == 0 { Status::Pass } else { Status::Fail };
    let detail = if bboxes.is_empty() {
        "no labels".to_string()
    } else if overflowing == 0 {
        format!("{} label(s) within canvas", bboxes.len())
    } else {
        format!("{overflowing} label(s) overflow viewBox; worst {worst:.0}px")
    };
    CheckResult {
        id: "label-overflow",
        layer: 2,
        status,
        detail,
    }
}

/// Collect the absolute bounding box of every `Text` node, recursing through groups.
/// Canvas origin is assumed `(0,0)` — the near-universal case for renderer viewBoxes.
fn collect_text_bboxes(g: &usvg::Group, out: &mut Vec<usvg::Rect>) {
    for n in g.children() {
        match n {
            usvg::Node::Group(gg) => collect_text_bboxes(gg, out),
            usvg::Node::Text(t) => out.push(t.abs_bounding_box()),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    fn run(fixture: &str) -> CheckResult {
        check(&load(Path::new(fixture)).unwrap())
    }

    #[test]
    fn label_fits_passes() {
        assert_eq!(run("tests/fixtures/label-fits.svg").status, Status::Pass);
    }

    #[test]
    fn overflowing_label_fails() {
        let r = run("tests/fixtures/label-overflow.svg");
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "label-overflow");
    }

    #[test]
    fn d2_clean_passes() {
        assert_eq!(run("tests/fixtures/d2-clean.svg").status, Status::Pass);
    }

    #[test]
    fn plantuml_clean_passes() {
        assert_eq!(run("tests/fixtures/plantuml-clean.svg").status, Status::Pass);
    }

    #[test]
    fn in_band_no_labels_passes() {
        assert_eq!(run("tests/fixtures/in-band.svg").status, Status::Pass);
    }
}
```

- [ ] **Step 3: Register the module and check**

In `src/checks/mod.rs`, add the module declaration in alphabetical order among
the existing `pub mod` lines:

```rust
pub mod label_overflow;
```

and add the check call in `run_all` immediately after
`edge_node_overlap::check(svg),` (keeping the Layer-2 geometry checks grouped):

```rust
edge_node_overlap::check(svg),
label_overflow::check(svg),
```

- [ ] **Step 4: Run the check tests to verify they pass**

Run: `cargo test --lib checks::label_overflow` Expected: all five tests PASS.

If `d2_clean_passes` or `plantuml_clean_passes` FAILS (a real clean diagram
flagged — the bundled font measured wider than the renderer's), tune before
proceeding: raise `OVERFLOW_SIZE_FRACTION` in steps toward `0.25`, re-running
until the clean fixtures pass **while `overflowing_label_fails` still fails**.
If 0.25 is insufficient, switch the bundled font to the narrower DejaVu Sans
fallback from Task 1. Do not weaken the tolerance past the point where
`label-overflow.svg` still fails.

- [ ] **Step 5: Commit**

```bash
git add src/checks/label_overflow.rs src/checks/mod.rs tests/fixtures/label-overflow.svg
git commit -m "feat: add Layer-2 label-overflow check (label bbox past viewBox edge)"
```

---

### Task 3: CLI integration test

**Files:**

- Modify: `tests/cli.rs`

**Interfaces:**

- Consumes: the `diagctl()` command helper and `predicates::str::contains`
  (existing in `tests/cli.rs`); the `label-overflow.svg` fixture (Task 2).
- Produces: nothing consumed downstream.

- [ ] **Step 1: Add the failing integration test**

Append to `tests/cli.rs`:

```rust
#[test]
fn check_label_overflow_exits_1() {
    diagctl()
        .args(["check", "tests/fixtures/label-overflow.svg"])
        .assert()
        .code(1)
        .stdout(contains("label-overflow"));
}
```

- [ ] **Step 2: Run the integration test**

Run: `cargo test --test cli check_label_overflow_exits_1` Expected: PASS (exit
code 1, stdout names `label-overflow`).

- [ ] **Step 3: Run the full binary test suite to confirm no regressions**

Run: `cargo test --test cli` Expected: all integration tests PASS — in
particular `check_in_band_exits_0` and `check_d2_clean_exits_0` still exit 0
(their labels fit; the new check does not flip them).

- [ ] **Step 4: Commit**

```bash
git add tests/cli.rs
git commit -m "test: CLI integration for label-overflow (exit 1 naming the check)"
```

---

### Task 4: README docs and final gate

**Files:**

- Modify: `README.md`

**Interfaces:**

- Consumes: nothing.
- Produces: nothing.

- [ ] **Step 1: Bump the gate size in the status line**

In `README.md`, replace:

```
> **Status:** `check` runs a **9-check** deterministic gate across Layers 0–2
```

with:

```
> **Status:** `check` runs a **10-check** deterministic gate across Layers 0–2
```

- [ ] **Step 2: Add the check to the table**

Immediately after the `edge-node-overlap` table row, add:

```
| `label-overflow` | 2 | label text clipped at the viewBox edge |
```

- [ ] **Step 3: Note the deterministic-font mechanism**

Append to the Layer-2 paragraph that begins "Layer-2 geometry is computed from
the rendered SVG via `usvg`…" the sentence:

```
`label-overflow` additionally shapes text with an embedded metrics font so text geometry is deterministic across platforms.
```

- [ ] **Step 4: Reword the deferred bullet**

Replace the "Not yet implemented" bullet:

```
- **label overflow** — text bbox vs container; needs font loading, which makes text
  metrics non-deterministic across platforms. Deferred pending a deterministic approach.
```

with:

```
- **label overflow (container variant)** — label vs its enclosing node. The canvas
  variant (`label-overflow`) now ships, using an embedded metrics font for deterministic
  geometry; container-fit is deferred pending trusted per-node font metrics.
```

- [ ] **Step 5: Update the "nine checks" prose**

Replace:

```
`check` runs the nine checks in the table above.
```

with:

```
`check` runs the ten checks in the table above.
```

- [ ] **Step 6: Run the full gate**

Run each and confirm clean:

```bash
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```

Expected: `cargo test` all green; `fmt --check` no diff; `clippy` no warnings.

- [ ] **Step 7: Commit**

```bash
git add README.md
git commit -m "docs: README reflects the 10-check gate incl. label-overflow"
```

---

## Self-Review

**Spec coverage:**

- Canvas-scope check, Layer 2, id `label-overflow` → Task 2. ✓
- Bundle one metrics font, force it via fresh parse, isolated from other checks
  → Task 1 (`shaped_tree`). ✓
- Tolerance `max(2px, 0.15 × label size)` per axis → Task 2 constants +
  algorithm. ✓
- Regression guard on clean fixtures → Task 2 unit tests + Task 3 integration. ✓
- Font-load safety test (non-zero bbox) → Task 1 Step 3. ✓
- Defensive skip on parse/shaping failure → Task 2 `check` early return. ✓
- Fail fixture, only `label-overflow` fails → Task 2 Step 1 fixture design. ✓
- Docs sweep 9→10 + table + deferred bullet → Task 4. ✓
- No new deps; permissive font + attribution → Global Constraints + Task 1. ✓
- Success criteria (shaping non-zero, clean pass, fail fixture fails, exit 1,
  other checks unchanged, green gate) → covered across Tasks 1–4. ✓

**Placeholder scan:** No TBD/TODO; every code and command step is complete. The
one contingency (tolerance tuning in Task 2 Step 4) is bounded with exact values
and a stop condition, not a placeholder. ✓

**Type consistency:** `shaped_tree(&Svg) -> anyhow::Result<usvg::Tree>` defined
in Task 1, consumed in Task 2. `check(&Svg) -> CheckResult` matches the existing
check signature and `run_all` call site.
`collect_text_bboxes(&usvg::Group, &mut Vec<usvg::Rect>)` and
`t.abs_bounding_box()` match the `usvg::Rect` usage already in `geom.rs`.
Constants `OVERFLOW_ABS_FLOOR_PX` / `OVERFLOW_SIZE_FRACTION` named identically
at definition and use. ✓

# diagctl contrast (greyscale readability) Implementation Plan

> REQUIRED SUB-SKILL: superpowers:executing-plans. Steps use `- [ ]`.

**Goal:** Layer-1 `contrast` check — every solid stroke meets WCAG ≥ 3:1 vs the
canvas background.

**Architecture:** Pure `color` module (luminance + ratio) + a `contrast` check
that walks the usvg tree for solid stroke colors and the full-canvas background
fill. Additive; deterministic; no fonts, no new deps.

---

## File Structure

- `src/color.rs` — NEW pure math.
- `src/main.rs` — MODIFY: `mod color;`.
- `src/checks/contrast.rs` — NEW check.
- `src/checks/mod.rs` — MODIFY: register `contrast`.
- `tests/fixtures/low-contrast.svg` — NEW.
- `tests/cli.rs` — MODIFY: integration test.

### Contract

```rust
// color.rs
pub fn relative_luminance(r: u8, g: u8, b: u8) -> f32;
pub fn contrast_ratio(l1: f32, l2: f32) -> f32;
// checks/contrast.rs
pub const MIN_CONTRAST: f32 = 3.0;
pub fn check(svg: &crate::svg::Svg) -> crate::checks::CheckResult; // id "contrast", layer 1
```

---

## Task 1: `color` module

- [ ] **Step 1: Write `src/color.rs`**

```rust
//! WCAG color math for the contrast check. Pure — no usvg.

fn linearize(c: u8) -> f32 {
    let s = c as f32 / 255.0;
    if s <= 0.03928 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG relative luminance of an sRGB color (0.0–1.0).
pub fn relative_luminance(r: u8, g: u8, b: u8) -> f32 {
    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

/// WCAG contrast ratio between two luminances (1.0–21.0).
pub fn contrast_ratio(l1: f32, l2: f32) -> f32 {
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_and_white_luminance() {
        assert!(relative_luminance(0, 0, 0).abs() < 1e-6);
        assert!((relative_luminance(255, 255, 255) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn black_on_white_is_21_to_1() {
        let ratio = contrast_ratio(
            relative_luminance(255, 255, 255),
            relative_luminance(0, 0, 0),
        );
        assert!((ratio - 21.0).abs() < 1e-4, "got {ratio}");
    }
}
```

- [ ] **Step 2: Add `mod color;` to `src/main.rs`** (alphabetical: after
      `mod checks;` line group — put `mod color;` between
      `mod cli;`/`mod commands;` appropriately; final order
      `checks, cli, color, commands, geom, svg`).

- [ ] **Step 3:** Run `cargo test --bin diagctl color::` → PASS (2 tests).

- [ ] **Step 4:**
      `git add src/color.rs src/main.rs && git commit -m "feat: color module (WCAG luminance + contrast ratio)"`

---

## Task 2: `low-contrast.svg` fixture

- [ ] **Step 1: Write `tests/fixtures/low-contrast.svg`** (light-grey stroke on
      white; passes all other checks):

```xml
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="120" viewBox="0 0 200 120">
  <g>
    <g><rect x="40" y="30" width="120" height="60" fill="#ffffff" stroke="#cccccc"/></g>
  </g>
</svg>
```

- [ ] **Step 2:**
      `git add tests/fixtures/low-contrast.svg && git commit -m "test: low-contrast fixture"`

---

## Task 3: `contrast` check + registry + integration

- [ ] **Step 1: Write `src/checks/contrast.rs`**

```rust
use crate::checks::{CheckResult, Status};
use crate::color;
use crate::svg::Svg;
use usvg::{Group, Node, Paint};

pub const MIN_CONTRAST: f32 = 3.0;

pub fn check(svg: &Svg) -> CheckResult {
    let size = svg.tree.size();
    let canvas_area = size.width() * size.height();
    let bg = background_color(svg.tree.root(), canvas_area).unwrap_or((255, 255, 255));
    let bg_lum = color::relative_luminance(bg.0, bg.1, bg.2);

    let mut strokes = Vec::new();
    collect_strokes(svg.tree.root(), &mut strokes);

    let mut low = 0usize;
    let mut min_ratio = f32::INFINITY;
    for (r, g, b) in strokes {
        let ratio = color::contrast_ratio(color::relative_luminance(r, g, b), bg_lum);
        if ratio < MIN_CONTRAST {
            low += 1;
        }
        if ratio < min_ratio {
            min_ratio = ratio;
        }
    }

    let status = if low == 0 { Status::Pass } else { Status::Fail };
    let detail = if min_ratio.is_finite() {
        format!("{low} low-contrast stroke(s); min ratio {min_ratio:.2} (limit {MIN_CONTRAST})")
    } else {
        "no solid strokes".to_string()
    };
    CheckResult {
        id: "contrast",
        layer: 1,
        status,
        detail,
    }
}

fn rgb(p: &Paint) -> Option<(u8, u8, u8)> {
    match p {
        Paint::Color(c) => Some((c.red, c.green, c.blue)),
        _ => None,
    }
}

fn collect_strokes(g: &Group, out: &mut Vec<(u8, u8, u8)>) {
    for n in g.children() {
        match n {
            Node::Group(gg) => collect_strokes(gg, out),
            Node::Path(p) => {
                if let Some(c) = p.stroke().and_then(|s| rgb(s.paint())) {
                    out.push(c);
                }
            }
            _ => {}
        }
    }
}

fn background_color(g: &Group, canvas_area: f32) -> Option<(u8, u8, u8)> {
    for n in g.children() {
        match n {
            Node::Group(gg) => {
                if let Some(c) = background_color(gg, canvas_area) {
                    return Some(c);
                }
            }
            Node::Path(p) => {
                let b = p.abs_bounding_box();
                if b.width() * b.height() >= 0.95 * canvas_area {
                    if let Some(c) = p.fill().and_then(|f| rgb(f.paint())) {
                        return Some(c);
                    }
                }
            }
            _ => {}
        }
    }
    None
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
    fn low_contrast_fails() {
        let svg = load(Path::new("tests/fixtures/low-contrast.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "contrast");
    }
}
```

- [ ] **Step 2: Register in `src/checks/mod.rs`** — add `pub mod contrast;`
      (alphabetical, after `aspect`) and append `contrast::check(svg),` to the
      `run_all` vec (after `edge_node_overlap`).

- [ ] **Step 3:** Run `cargo test --bin diagctl contrast` → PASS (2 tests).

- [ ] **Step 4: Add integration test to `tests/cli.rs`**

```rust
#[test]
fn check_low_contrast_exits_1() {
    diagctl()
        .args(["check", "tests/fixtures/low-contrast.svg"])
        .assert()
        .code(1)
        .stdout(contains("contrast"));
}
```

- [ ] **Step 5:** Run `cargo test` → all PASS (existing fixtures still pass —
      additive).

- [ ] **Step 6:**
      `git add -A && git commit -m "feat: Layer-1 contrast check (WCAG stroke vs background)"`

---

## Task 4: Final verification

- [ ] **Step 1:** `cargo fmt --all -- --check` (apply + recommit if needed);
      `cargo clippy --all-targets -- -D warnings`; `cargo test`.
- [ ] **Step 2:** Smoke:
      `cargo run -q -- check tests/fixtures/low-contrast.svg; echo $?` → FAIL
      contrast line, rc=1;
      `cargo run -q -- check tests/fixtures/d2-clean.svg; echo $?` → all PASS,
      rc=0.
- [ ] **Step 3:** Commit fixups if any.

## Self-Review

- color math → Task 1; contrast check + bg detection → Task 3; additive
  (existing fixtures pass) → Task 3 Step 5. Threshold 3.0, MIN_CONTRAST const.
  Types consistent (`relative_luminance`, `contrast_ratio`, `contrast::check` id
  `"contrast"`). No placeholders.

# diagctl contrast (greyscale readability) — design

**Date:** 2026-06-11 **Status:** autonomous loop (user authorized continuous
progress 2026-06-11) **Tracks:**
[driftsys/diagctl#2](https://github.com/driftsys/diagctl/issues/2) — Layer-1
conformance

## Goal

Add a Layer-1 **`contrast`** check: every solid **stroke** color in the diagram
must meet a minimum WCAG contrast ratio against the canvas background, so
lines/borders/labels remain distinguishable in greyscale / print. Deterministic
(color math only; no fonts).

## Why strokes, not fills

Diagrams routinely use light-filled boxes (e.g. `#f7f8fe`) delineated by a dark
**border**; the fill–vs–canvas contrast is intentionally low. What must read in
greyscale is the **stroke** (borders, connectors, text rendered as
stroked/filled paths). So the check measures stroke colors against the
background and leaves fills alone — avoiding false positives on normal
light-fill diagrams. (Probe: d2-clean strokes `#0d32b2` → 9.9:1 vs white; node
fills `#f7f8fe` → 1.06:1, correctly ignored.)

## Architecture

```
src/
  color.rs            NEW — pure: relative_luminance(r,g,b), contrast_ratio(l1,l2)
  checks/
    mod.rs            register contrast in run_all
    contrast.rs       NEW — Layer-1 stroke-vs-background contrast check
tests/
  fixtures/
    low-contrast.svg  NEW — one node, light-grey (#cccccc) stroke on white -> Fail
  cli.rs              integration: check low-contrast.svg exit 1
```

### `color.rs` (pure math, no usvg)

```rust
/// WCAG relative luminance of an sRGB color (0.0–1.0).
pub fn relative_luminance(r: u8, g: u8, b: u8) -> f32;
/// WCAG contrast ratio between two luminances (1.0–21.0).
pub fn contrast_ratio(l1: f32, l2: f32) -> f32;
```

- `relative_luminance`: linearize each channel
  (`c<=0.03928 ? c/12.92 : ((c+0.055)/1.055)^2.4`), then
  `0.2126 R + 0.7152 G + 0.0722 B`.
- `contrast_ratio`: `(max(l1,l2)+0.05)/(min(l1,l2)+0.05)`.

### `checks::contrast` (Layer 1)

```rust
pub const MIN_CONTRAST: f32 = 3.0; // WCAG graphical-objects minimum
```

- **Background luminance:** walk the tree for a filled path whose abs bbox area
  ≥ 95% of the canvas (the full-canvas background rect); use its color. If none,
  assume white.
- **Strokes:** walk the tree; for every `Path` whose `stroke().paint()` is a
  solid `Paint::Color`, compute
  `contrast_ratio(luminance(stroke), background_luminance)`.
- `Fail` if any stroke ratio < `MIN_CONTRAST`; detail reports the low-contrast
  count and the minimum ratio seen. `Pass` otherwise (including diagrams with no
  solid strokes).
- Gradient/pattern strokes are skipped (no single color to score).
- `CheckResult { id: "contrast", layer: 1, ... }`; registered in `run_all` after
  the geometry checks.

usvg API (probe-validated): `Path::stroke() -> Option<&Stroke>`,
`Stroke::paint() -> &Paint`, `Paint::Color(usvg::Color)` with
`.red/.green/.blue: u8`; same shape for `fill()`.

## Data flow

`check` → `svg::load` (unchanged) → `run_all` also calls `contrast::check`.
Exit-code contract unchanged (0/1/2). Existing fixtures (dark strokes) keep
passing — additive.

## Error handling

- Pure over parsed geometry; returns a count. No solid strokes → Pass.
- Non-color paints skipped, not errors.

## Testing (TDD, no mocks)

- **`color` unit tests:** `relative_luminance(0,0,0)=0.0`, `(255,255,255)=1.0`;
  `contrast_ratio(1.0, 0.0)=21.0` (black on white).
- **`contrast::check` unit tests:** `d2-clean` → Pass (real render, `#0d32b2`
  strokes ~9.9:1); `low-contrast` → Fail.
- **Integration:** `check low-contrast.svg` → exit 1 naming `contrast` (sized so
  viewBox, aspect, node-overlap, edge-crossings, edge-node-overlap all pass —
  only contrast fails).

## Scope (YAGNI)

Stroke-vs-background only. No fill-vs-fill, no text-specific contrast (that
needs font loading; deferred). No new deps. Existing checks/fixtures unchanged.

## Verifiable success criteria

1. `relative_luminance` and `contrast_ratio` return the canonical values above
   (unit-tested).
2. `contrast::check(d2-clean)` → Pass.
3. `contrast::check(low-contrast)` → Fail.
4. `check low-contrast.svg` → exit 1, output names `contrast`.
5. All previously-passing fixtures still pass `check` (additive).
6. `cargo test` green; fmt + clippy clean; CI matrix + musl guard green.

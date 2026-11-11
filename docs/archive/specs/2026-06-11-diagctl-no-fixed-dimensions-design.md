# diagctl no-fixed-dimensions — design

**Date:** 2026-06-11 **Status:** autonomous loop (user selected 2026-06-11)
**Tracks:** [driftsys/diagctl#2](https://github.com/driftsys/diagctl/issues/2) —
Layer-0 conformance

## Goal

Layer-0 **`no-fixed-dimensions`** check: the root `<svg>` must carry **no**
fixed `width`/ `height` attribute (viewBox only), so the diagram scales
responsively when embedded.

## Approach

Raw-XML (`roxmltree`), like `viewbox-present`. `doc.root_element()`; flag if it
has a `width` or `height` attribute. Deterministic, no fonts, no geometry
impact.

**Key finding:** D2's _root_ `<svg>` carries only `viewBox` (the
`width`/`height` sit on a nested inner `<svg>`), so real D2/PlantUML renders
pass at the root level. Among committed fixtures, only the hand-authored
`in-band.svg` had root width/height — stripped (size still derives from viewBox,
so every geometry/aspect result is unchanged). `no-viewbox.svg` keeps its
width/height (it has no viewBox and needs a size); it already fails
`viewbox-present`, so the added failure doesn't change its exit-1 outcome.

## Fixtures

- `in-band.svg` — width/height stripped → now passes (kept as the all-pass
  exit-0 fixture).
- `fixed-dims.svg` — NEW: viewBox + width/height, otherwise clean → fails only
  `no-fixed-dimensions`.

## Testing

- `no_fixed_dimensions::check`: `d2-clean` → Pass (root has only viewBox);
  `in-band` → Pass (post-strip); `fixed-dims` → Fail.
- Integration: `check fixed-dims.svg` → exit 1 naming `no-fixed-dimensions`; the
  three exit-0 fixtures (`in-band`, `d2-clean`, `cross-clean`) still exit 0.

## Success criteria

1. `check(d2-clean)`/`check(in-band)` → Pass; `check(fixed-dims)` → Fail.
2. `check fixed-dims.svg` → exit 1 naming the check; exit-0 fixtures unchanged.
3. `cargo test` green; fmt + clippy clean; CI + musl green.

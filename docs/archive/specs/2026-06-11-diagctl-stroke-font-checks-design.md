# diagctl min-stroke-width + font-family — design

**Date:** 2026-06-11 **Status:** autonomous loop (user selected these checks
2026-06-11) **Tracks:**
[driftsys/diagctl#2](https://github.com/driftsys/diagctl/issues/2) — Layer-1
conformance

Two small, **additive**, deterministic Layer-1 checks (existing fixtures keep
passing). Grouped in one PR; no new deps.

## Check A — `min-stroke-width` (Layer 1)

Hairline strokes vanish in greyscale/print. Flag any solid stroke whose resolved
width is below a conservative floor.

- `pub const MIN_STROKE_WIDTH: f32 = 0.5;`
- Walk the usvg tree; for each `Path` with a stroke, read
  `stroke().width().get()` (probe: d2 = 2.0). Flag widths `< MIN_STROKE_WIDTH`.
- `Fail` if any; detail = count + the minimum width seen. `Pass` otherwise.
- Conservative floor (0.5) chosen so legitimate renderer output passes (PlantUML
  emits 0.5px borders; D2 2px; hand-authored 1px default) — only degenerate
  sub-0.5px hairlines fail. `id "min-stroke-width"`, layer 1.

## Check B — `font-family` (Layer 1)

Text should declare a font-family **fallback chain** (or a generic family) so it
renders predictably. Read from the **raw XML** (`roxmltree`) — deterministic, no
font loading.

- For each `<text>`/`<tspan>` element with a `font-family` (attribute or inline
  `style`), flag it iff the value is a **single, non-generic** family: no comma
  **and** not one of
  `serif, sans-serif, monospace, cursive, fantasy, system-ui, ui-*`
  (case-insensitive, quotes stripped).
- Text with **no** `font-family` is **skipped** (it may be set via CSS class,
  e.g. D2) — this keeps the check additive (d2-clean text carries no
  `font-family` attribute → not flagged).
- `Fail` if any flagged; detail = count. `id "font-family"`, layer 1.

## Architecture

```
src/checks/
  mod.rs              register min_stroke_width + font_family in run_all
  min_stroke_width.rs NEW — usvg stroke-width walk
  font_family.rs      NEW — roxmltree font-family inspection
tests/fixtures/
  thin-stroke.svg     NEW — node with 0.2px stroke           -> min-stroke-width Fail
  font-single.svg     NEW — text font-family="Arial"          -> font-family Fail
tests/cli.rs          integration: each fail fixture exits 1
```

usvg API (probe-validated): `Stroke::width() -> NonZeroPositiveF32`,
`.get() -> f32`. roxmltree: `Document::parse(&raw)`, `descendants()`,
`Node::tag_name().name()`, `Node::attribute("font-family")`,
`Node::attribute("style")`.

## Data flow & error handling

`run_all` gains two checks. `min_stroke_width` operates on the parsed tree.
`font_family` re-parses `svg.raw` with roxmltree; a parse error (cannot happen —
usvg already parsed it) → treated as no text → Pass. Exit-code contract
unchanged.

## Testing (TDD, additive, no mocks)

- `min_stroke_width::check`: `d2-clean` (2px) → Pass; `thin-stroke` (0.2px) →
  Fail.
- `font_family::check`: `d2-clean` (no font-family attr) → Pass; `font-single`
  (`"Arial"`) → Fail; assert a fixture with `"Arial, sans-serif"` (or generic)
  passes.
- Integration: `check thin-stroke.svg` → exit 1 naming `min-stroke-width`;
  `check font-single.svg` → exit 1 naming `font-family`.
- Existing fixtures unchanged and still pass `check` (additive).

## Verifiable success criteria

1. `min_stroke_width::check` Pass on d2-clean, Fail on thin-stroke.
2. `font_family::check` Pass on d2-clean, Fail on font-single, Pass on a
   fallback-chain text.
3. `check thin-stroke.svg` / `check font-single.svg` → exit 1 naming the right
   check.
4. All previously-passing fixtures still pass `check`.
5. `cargo test` green; fmt + clippy clean; CI matrix + musl guard green.

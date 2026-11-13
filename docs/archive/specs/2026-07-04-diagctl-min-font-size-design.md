# diagctl min-font-size — design

**Date:** 2026-07-04 **Status:** approved (user selected this check + 8px floor
2026-07-04) **Tracks:**
[driftsys/diagctl#2](https://github.com/driftsys/diagctl/issues/2) — completes
the "stroke-width / font-size thresholds" Layer-1 item (stroke-width already
shipped).

One small, **additive**, deterministic Layer-1 check (existing fixtures keep
passing). Single PR, no new deps. It is the **11th** check in `run_all`
(label-overflow landed as the 10th via
[#11](https://github.com/driftsys/diagctl/pull/11), merged 2026-07-04).

## Check — `min-font-size` (Layer 1)

Tiny text is unreadable in print/greyscale. Flag any `<text>`/`<tspan>` whose
declared `font-size` resolves below a conservative floor.

- `pub const MIN_FONT_SIZE: f32 = 8.0;`
- Read from the **raw XML** (`roxmltree`) — deterministic, no font loading. This
  mirrors `font-family`: `usvg::Options::default()` loads no fonts, so the
  resolved tree has **no text nodes** to inspect.
- For each `<text>`/`<tspan>` element carrying a `font-size` (attribute **or**
  inline `style`), parse the numeric value:
  - **unitless** or **`px`** suffix → compare the number against the floor.
  - any other unit (`em`, `rem`, `%`, `pt`, `ex`, …) → **skip** (cannot resolve
    deterministically without parent/DPI context). Non-parseable value →
    **skip**. Skipping means the element never fails — keeps the check
    false-positive-free.
- Text with **no** `font-size` is **skipped** (size may come from a CSS class,
  e.g. D2) — keeps the check additive, exactly as `font-family` skips text with
  no `font-family`.
- `Fail` if any parsed size `< 8.0`; detail = count of tiny elements + the
  smallest size seen. `Pass` otherwise. `id "min-font-size"`, layer 1.

### Why 8.0

Mirrors `min-stroke-width`'s conservative-floor philosophy: 8px sits **below**
every in-scope renderer default (D2 ~16px, mermaid ~16px, PlantUML ~13px,
draw.io ~12px), so legitimate output always passes and only
degenerate/hand-shrunk text fails.

### Limitation (accepted)

No transform-resolution: a small `font-size` inside a scaling
`<g transform="scale(…)">` renders larger than declared and would still be
flagged. Accepted — the in-scope diagram renderers emit text at 1:1, the same
assumption `font-family` already relies on.

## Architecture

```
src/checks/
  mod.rs             register min_font_size in run_all (grouped with Layer-1 checks)
  min_font_size.rs   NEW — roxmltree font-size inspection
tests/fixtures/
  tiny-font.svg      NEW — text font-size="6"  -> min-font-size Fail
  font-ok.svg        NEW — text font-size="14" -> min-font-size Pass (explicit accept path)
tests/cli.rs         integration: check tiny-font.svg exits 1 naming min-font-size
README.md            "10-check" -> "11-check"; add table row; tick #2's font-size item
```

roxmltree API (probe-validated in `font_family`): `Document::parse(&raw)`,
`descendants()`, `Node::tag_name().name()`, `Node::attribute("font-size")`,
`Node::attribute("style")`.

Parsing helper: strip a trailing `px` (case-insensitive), reject any remaining
non-numeric suffix, `str::parse::<f32>()` the rest. Reuse the `style`-attribute
walk shape from `font_family::font_family_of`.

## Data flow & error handling

`run_all` gains one check. `min_font_size` re-parses `svg.raw` with roxmltree; a
parse error (cannot happen — usvg already parsed it) → treated as no text →
`Pass`. Exit-code contract unchanged (`0`/`1`/`2`).

## Testing (TDD, additive, no mocks)

- `min_font_size::check`:
  - `tiny-font` (6px) → `Fail`, `id == "min-font-size"`.
  - `font-ok` (14px) → `Pass` (explicit accept path).
  - `d2-clean` (no `font-size` attr → skip path) → `Pass`.
- Integration: `check tiny-font.svg` → exit `1` naming `min-font-size`.
- Every previously-passing fixture still passes `check` (additive).

## Verifiable success criteria

1. `min_font_size::check` Fail on tiny-font (6px), Pass on font-ok (14px), Pass
   on d2-clean.
2. A `font-size="8em"` / `font-size="6pt"` element does **not** fail (unit-skip
   path).
3. `check tiny-font.svg` → exit 1 naming `min-font-size`.
4. All previously-passing fixtures still pass `check`.
5. `run_all` now returns 11 checks; README says "11-check".
6. `cargo test` green; fmt + clippy clean; CI matrix + musl guard green.

# diagctl

A single, hermetic, cross-platform CLI that automates the **tech-diagramming**
quality gate — the Phase-2 tooling for the
[metapowers](https://github.com/driftsys/metapowers) diagramming skill family.

> **Status:** `check` runs a **10-check** deterministic gate across Layers 0–2
> (no vision, near-zero cost). `optimize`/`ascii`/`freshness` are stubbed. Full
> design, scope, and acceptance criteria live in
> **[metapowers#14](https://github.com/driftsys/metapowers/issues/14)**; remaining
> checks are tracked in **[#2](https://github.com/driftsys/diagctl/issues/2)**.

## What it does

One standalone binary (zero runtime dependencies) with these subcommands:

- **`check`** — the diagram gate. Each check prints `PASS`/`FAIL` with a layer tag;
  exit `0` clean · `1` ≥1 check failed · `2` operational error.

  | Check | Layer | What it flags |
  | --- | --- | --- |
  | `viewbox-present` | 0 | root `<svg>` missing a `viewBox` |
  | `no-fixed-dimensions` | 0 | root `<svg>` with fixed `width`/`height` |
  | `contrast` | 1 | stroke vs background below WCAG 3:1 |
  | `min-stroke-width` | 1 | hairline strokes below 0.5px |
  | `font-family` | 1 | text with a single, non-generic family (no fallback) |
  | `aspect-ratio` | 2 | viewBox long/short ratio above 2.5 |
  | `node-overlap` | 2 | bounding-box overlap of unrelated nodes |
  | `edge-crossings` | 2 | edge–edge crossings in open whitespace |
  | `edge-node-overlap` | 2 | an edge routed through an unrelated node |
  | `label-overflow` | 2 | label text clipped at the viewBox edge |

  Layer-2 geometry is computed from the rendered SVG via `usvg` (resolved
  geometry + transforms) with hand-rolled bezier-flattening / segment math — no
  vision, near-zero cost. `label-overflow` additionally shapes text with an
  embedded metrics font so text geometry is deterministic across platforms.

- **`optimize`** — conservative SVGO-style pass _(not yet implemented; exit 2)_.
- **`ascii`** — deterministic grid render + width-aware alignment _(not yet implemented; exit 2)_.
- **`freshness`** — re-render → optimize → byte-diff _(not yet implemented; exit 2)_.

It **reimplements** these checks in one binary rather than gluing svgo / svglint /
xmllint / contrast tools together — so consumers get a single dependency-free gate.

### Not yet implemented (tracked in [#2](https://github.com/driftsys/diagctl/issues/2))

- **label overflow (container variant)** — label vs its enclosing node. The canvas
  variant (`label-overflow`) now ships, using an embedded metrics font for deterministic
  geometry; container-fit is deferred pending trusted per-node font metrics.
- **source+SVG pair naming + freshness** — a repo/filesystem concern, distinct from the
  single-SVG checks.
- **`optimize` / `ascii` / `freshness`** subcommand bodies.

## Relationship to metapowers

The Phase-1 tech-diagramming skills do this gate by hand (skill self-check + human
review). `diagctl` makes Layers 0–2 deterministic and automatable in CI —
**additive, never a blocker**. The complementary _vision_ self-check (judgment
calls geometry can't score) is tracked separately in
[metapowers#38](https://github.com/driftsys/metapowers/issues/38).

## Stack

**Rust.** The deciding fork (metapowers#14) was `optimize` (SVGO) — JS-native, no
proven Rust port — and it is deferred to a stub, neutralising Rust's only weakness.
Everything load-bearing favours Rust: the Layer-2 geometry checks build on
`usvg`/`resvg` (resolved geometry + transforms) with hand-rolled bezier/segment math
(no heavy geometry dep), and a static binary is ~2–8 MB vs `deno compile`'s ~80–100 MB.

## Build, test, run

Requires a Rust toolchain (1.95+).

```bash
cargo build            # debug build
cargo test             # run the suite (unit + binary integration tests)
cargo run -- --help    # see the subcommands
cargo run -- --version

# the diagram gate (10 checks, Layers 0–2)
cargo run -- check path/to/diagram.svg   # exit 0 pass / 1 fail / 2 error
```

`check` runs the ten checks in the table above. `optimize`, `ascii`, and `freshness`
are stubbed (exit 2) pending later milestones.

## License

MIT © driftsys — see [LICENSE](LICENSE).

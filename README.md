# diagctl

A single, hermetic, cross-platform CLI that automates the **tech-diagramming**
quality gate — the Phase-2 tooling for the
[metapowers](https://github.com/driftsys/metapowers) diagramming skill family.

> **Status:** not yet implemented. Design, scope, and acceptance criteria live in
> **[metapowers#14](https://github.com/driftsys/metapowers/issues/14)**.

## What it will do

One standalone binary (zero runtime dependencies) with these subcommands:

- **`check`** — the diagram gate.
  - _Layer-0/1 conformance:_ `viewBox` present, no fixed `width`/`height`,
    stroke/font thresholds, font-family chain, source+SVG pair naming, WCAG
    contrast.
  - _Layer-2 geometry_ (computed deterministically from the rendered SVG — no
    vision, near-zero cost): node–node overlap, edge-through-node, edge–edge
    crossings, label overflow, aspect ratio.
- **`optimize`** — conservative SVGO-style pass (keeps `viewBox`/`title`/`desc`).
- **`ascii`** — deterministic grid render + width-aware alignment validation.
- **`freshness`** — re-render → optimize → byte-diff (CI-stable pair freshness).

It **reimplements** these checks in one binary rather than gluing svgo / svglint /
xmllint / contrast tools together — so consumers get a single dependency-free gate.

## Relationship to metapowers

The Phase-1 tech-diagramming skills do this gate by hand (skill self-check + human
review). `diagctl` makes Layers 0–2 deterministic and automatable in CI —
**additive, never a blocker**. The complementary _vision_ self-check (judgment
calls geometry can't score) is tracked separately in
[metapowers#38](https://github.com/driftsys/metapowers/issues/38).

## Stack — open decision

Rust vs Deno/TS is **not yet settled** (see metapowers#14). The deciding factor is
the `optimize` (SVGO) subcommand; the Layer-2 geometry checks favor Rust
(`usvg`/`resvg`/`lyon`). The name `diagctl` is free on crates.io, npm, and JSR.

## License

MIT © driftsys — see [LICENSE](LICENSE).

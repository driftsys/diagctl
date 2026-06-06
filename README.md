# diagctl

A single, hermetic, cross-platform CLI that automates the **tech-diagramming**
quality gate — the Phase-2 tooling for the
[metapowers](https://github.com/driftsys/metapowers) diagramming skill family.

> **Status:** walking skeleton. `check` runs the Layer-0 viewBox + Layer-2
> aspect-ratio gate today; `optimize`/`ascii`/`freshness` are stubbed. Full
> design, scope, and acceptance criteria live in
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

## Stack

**Rust.** The deciding fork (metapowers#14) was `optimize` (SVGO) — JS-native, no
proven Rust port — and it is deferred to a stub, neutralising Rust's only weakness.
Everything load-bearing favours Rust: the Layer-2 geometry checks build on
`usvg`/`resvg` + `lyon`, and a static binary is ~2–8 MB vs `deno compile`'s ~80–100 MB.

## Build, test, run

Requires a Rust toolchain (1.95+).

```bash
cargo build            # debug build
cargo test             # run the suite (unit + binary integration tests)
cargo run -- --help    # see the subcommands
cargo run -- --version

# the one real slice today: the diagram gate
cargo run -- check path/to/diagram.svg   # exit 0 pass / 1 fail / 2 error
```

`check` runs today: **viewBox present** (Layer-0) and **aspect-ratio** (Layer-2,
long/short viewBox ratio, fails above 2.5). `optimize`, `ascii`, and `freshness` are
stubbed (exit 2) pending the next milestone.

## License

MIT © driftsys — see [LICENSE](LICENSE).

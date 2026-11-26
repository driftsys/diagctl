# diagctl

[![crates.io](https://img.shields.io/crates/v/diagctl.svg)](https://crates.io/crates/diagctl)

A single, hermetic, cross-platform CLI that automates the **tech-diagramming**
quality gate — the Phase-2 tooling for the
[metapowers](https://github.com/driftsys/metapowers) diagramming skill family.

> **Status:** `check` runs a deterministic gate of 11 checks (`html`) or 12
> checks (`markdown`) across Layers 0–2 (no vision, near-zero cost).
> `optimize`/`freshness` are stubbed. Full design, scope, and acceptance
> criteria live in
> **[metapowers#14](https://github.com/driftsys/metapowers/issues/14)**;
> remaining checks are tracked in
> **[#2](https://github.com/driftsys/diagctl/issues/2)**.

## What it does

One standalone binary (zero runtime dependencies) with these subcommands:

- **`check`** — the diagram gate. Each check prints `PASS`/`FAIL` with a layer
  tag; exit `0` clean · `1` ≥1 check failed · `2` operational error. It takes
  one or more files; with more than one, each file's path is printed before its
  results. A file that cannot be read or parsed as SVG is reported on stderr,
  the other files are still checked, and the exit code is `2`.
  `--target html|markdown` names where the SVG is displayed (default `html`).
  The targets differ in three checks: `html` runs `no-fixed-dimensions`, and
  `markdown` runs `dimensions` in its place; `min-font-size` uses an 8px
  authored floor for `html` and an 11px displayed floor for `markdown`; only
  `markdown` runs `opaque-background`.

  | Check                 | Layer | What it flags                                                                                                                                                                         |
  | --------------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
  | `viewbox-present`     | 0     | root `<svg>` missing a `viewBox`                                                                                                                                                      |
  | `no-fixed-dimensions` | 0     | `html` only: root `<svg>` with fixed `width`/`height`                                                                                                                                 |
  | `dimensions`          | 0     | `markdown` only: no valid `viewBox`, `width`/`height` missing or not px, aspect more than 1 % off the `viewBox`, or width > 800 px                                                    |
  | `contrast`            | 1     | stroke vs background below WCAG 3:1                                                                                                                                                   |
  | `min-stroke-width`    | 1     | hairline strokes below 0.5px                                                                                                                                                          |
  | `font-family`         | 1     | text with a single, non-generic family (no fallback)                                                                                                                                  |
  | `min-font-size`       | 1     | `html`: `font-size` below 8px; `markdown`: below 11px after scaling to the displayed width (at most 800 px)                                                                           |
  | `opaque-background`   | 1     | `markdown` only: no opaque `#ffffff` background in the root `<svg>` `style` attribute, and the first painted element is not an opaque `#ffffff` rectangle that covers the whole image |
  | `aspect-ratio`        | 2     | viewBox long/short ratio above 2.5                                                                                                                                                    |
  | `node-overlap`        | 2     | bounding-box overlap of unrelated nodes                                                                                                                                               |
  | `edge-crossings`      | 2     | edge–edge crossings in open whitespace                                                                                                                                                |
  | `edge-node-overlap`   | 2     | an edge routed through an unrelated node                                                                                                                                              |
  | `label-overflow`      | 2     | label text clipped at the viewBox edge                                                                                                                                                |

  Layer-2 geometry is computed from the rendered SVG via `usvg` (resolved
  geometry + transforms) with hand-rolled bezier-flattening / segment math — no
  vision, near-zero cost. `label-overflow` additionally shapes text with an
  embedded metrics font so text geometry is deterministic across platforms.

- **`optimize`** — conservative SVGO-style pass _(not yet implemented; exit 2)_.
- **`freshness`** — re-render → optimize → byte-diff _(not yet implemented;
  exit 2)_.

With `--json`, every command prints one pretty-printed JSON object on stdout,
also when there are errors (the envelope of tblctl
[AD-0005](https://github.com/driftsys/tblctl/blob/main/docs/decisions/0005-json-envelope-and-human-output.md)):

```json
{
  "schema": "diagctl/check/v1",
  "results": [
    {
      "file": "a.svg",
      "check": "viewbox-present",
      "layer": 0,
      "status": "pass",
      "detail": "root <svg> has a viewBox"
    }
  ],
  "summary": { "checks": 1, "pass": 1, "fail": 0 },
  "errors": []
}
```

- `results`: for `check`, one object per check per file, in the order of the
  text output. `status` is `pass` or `fail`.
- `summary`: for `check`, `{checks, pass, fail}`. `optimize` and `freshness`
  give empty `results` and `summary`.
- `errors`: one object per operational error, with a stable `code`, a `message`
  and, when the error is about one file, `file`. The codes are `read_failed`,
  `parse_failed` and `not_implemented`. Each error is also printed on stderr as
  `diagctl: <message>`.
- The exit codes are the same as in text mode. Argument errors (an unknown flag,
  a missing file) are printed by the argument parser as text, with exit 2 and no
  JSON.

It **reimplements** these checks in one binary rather than gluing svgo / svglint
/ xmllint / contrast tools together — so consumers get a single dependency-free
gate.

### Not yet implemented (tracked in [#2](https://github.com/driftsys/diagctl/issues/2))

- **label overflow (container variant)** — label vs its enclosing node. The
  canvas variant (`label-overflow`) now ships, using an embedded metrics font
  for deterministic geometry; container-fit is deferred pending trusted per-node
  font metrics.
- **source+SVG pair naming + freshness** — a repo/filesystem concern, distinct
  from the single-SVG checks.
- **`optimize` / `freshness`** subcommand bodies.

## Install

From the latest GitHub release (macOS, Linux; checksum-verified):

```sh
curl -fsSL https://raw.githubusercontent.com/driftsys/diagctl/main/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/driftsys/diagctl/main/install.ps1 | iex
```

`DIAGCTL_INSTALL_DIR` (default `~/.local/bin`) and `DIAGCTL_VERSION` (default
latest) override the defaults.

Manual download:

```sh
gh release download -R driftsys/diagctl -p 'diagctl-<target>*'
```

From source:

```sh
cargo install diagctl
```

## Relationship to metapowers

The Phase-1 tech-diagramming skills do this gate by hand (skill self-check +
human review). `diagctl` makes Layers 0–2 deterministic and automatable in CI —
**additive, never a blocker**. The complementary _vision_ self-check (judgment
calls geometry can't score) is tracked separately in
[metapowers#38](https://github.com/driftsys/metapowers/issues/38).

## Stack

**Rust.** The deciding fork (metapowers#14) was `optimize` (SVGO) — JS-native,
no proven Rust port — and it is deferred to a stub, neutralising Rust's only
weakness. Everything load-bearing favours Rust: the Layer-2 geometry checks
build on `usvg`/`resvg` (resolved geometry + transforms) with hand-rolled
bezier/segment math (no heavy geometry dep), and a static binary is ~2–8 MB vs
`deno compile`'s ~80–100 MB.

## Build, test, run

Requires a Rust toolchain; `rust-toolchain.toml` pins the version. Gates are
`just` recipes, and CI runs nothing else.

```bash
just                   # list recipes
just build             # full local gate (toolchain, prim, fmt, compile, test, clippy, shellcheck, install test)
just verify            # commit-message lint + full gate, before opening a PR
cargo run -- check path/to/*.svg         # exit 0 pass / 1 fail / 2 error
```

`check` runs eleven checks from the table above for `html` and twelve for
`markdown`. `optimize` and `freshness` are stubbed (exit 2) pending later
milestones.

## License

MIT © driftsys — see [LICENSE](LICENSE).

<!-- git-std:bootstrap -->

## Post-clone setup

Run `./bootstrap` after `git clone` or `git worktree add`.

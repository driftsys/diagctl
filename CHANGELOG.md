# Changelog

## 0.1.0 (2026-11-11)

### Features

- add Layer-1 min-font-size check (font-size below 8px floor) ([a857689])
- add Layer-2 label-overflow check (label bbox past viewBox edge) ([9bb5414])
- embed metrics font + svg::shaped_tree for deterministic text geometry
  ([6046146])
- Layer-0 no-fixed-dimensions check + conform in-band fixture ([a771baf])
- Layer-1 min-stroke-width + font-family checks ([6230236])
- Layer-1 contrast check (WCAG stroke vs background) ([e7724a6])
- color module (WCAG luminance + contrast ratio) ([c841d76])
- Layer-2 edge-node-overlap check ([ea6afd7])
- seg::segment_enters_rect for edge-node overlap ([1346034])
- Layer-2 edge-crossings check on the edge polyline foundation ([bf77064])
- geom::edges extracts flattened absolute edge polylines ([4c2cd22])
- geom::seg bezier flattening + segment intersection ([2f59f4b])
- Layer-2 node-overlap check on the geom foundation ([67f3878])
- geom::model classifies nodes/edges/background from usvg tree ([f26e183])
- BBox geometry primitives for Layer-2 checks ([8628338])
- CLI dispatch, check command, and stubbed subcommands ([8f38d92])
- Layer-0 viewBox-present check ([d9d0dbd])
- Layer-2 aspect-ratio check (threshold 2.5) ([e27f1d0])
- svg::load parses fixtures via usvg ([24b2c31])

### Documentation

- README reflects the 11-check gate incl. min-font-size ([763a691])
- fix stale 9-check count in README run example ([2a4d0e1])
- README reflects the 10-check gate incl. label-overflow ([6cb1f6a])
- README reflects the 9-check gate + remaining debt ([cbccbac])
- README build/test/run + lock Rust stack ([eb382b3])

[a857689]: https://github.com/driftsys/diagctl/commit/a857689
[9bb5414]: https://github.com/driftsys/diagctl/commit/9bb5414
[6046146]: https://github.com/driftsys/diagctl/commit/6046146
[a771baf]: https://github.com/driftsys/diagctl/commit/a771baf
[6230236]: https://github.com/driftsys/diagctl/commit/6230236
[e7724a6]: https://github.com/driftsys/diagctl/commit/e7724a6
[c841d76]: https://github.com/driftsys/diagctl/commit/c841d76
[ea6afd7]: https://github.com/driftsys/diagctl/commit/ea6afd7
[1346034]: https://github.com/driftsys/diagctl/commit/1346034
[bf77064]: https://github.com/driftsys/diagctl/commit/bf77064
[4c2cd22]: https://github.com/driftsys/diagctl/commit/4c2cd22
[2f59f4b]: https://github.com/driftsys/diagctl/commit/2f59f4b
[67f3878]: https://github.com/driftsys/diagctl/commit/67f3878
[f26e183]: https://github.com/driftsys/diagctl/commit/f26e183
[8628338]: https://github.com/driftsys/diagctl/commit/8628338
[8f38d92]: https://github.com/driftsys/diagctl/commit/8f38d92
[d9d0dbd]: https://github.com/driftsys/diagctl/commit/d9d0dbd
[e27f1d0]: https://github.com/driftsys/diagctl/commit/e27f1d0
[24b2c31]: https://github.com/driftsys/diagctl/commit/24b2c31
[763a691]: https://github.com/driftsys/diagctl/commit/763a691
[2a4d0e1]: https://github.com/driftsys/diagctl/commit/2a4d0e1
[6cb1f6a]: https://github.com/driftsys/diagctl/commit/6cb1f6a
[cbccbac]: https://github.com/driftsys/diagctl/commit/cbccbac
[eb382b3]: https://github.com/driftsys/diagctl/commit/eb382b3

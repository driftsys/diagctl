# diagctl edge–node overlap — design

**Date:** 2026-06-10 **Status:** approved (design locked with user 2026-06-10)
**Tracks:** [driftsys/diagctl#2](https://github.com/driftsys/diagctl/issues/2) —
milestone **2b-2** **Builds on:** 2a `geom` (nodes), 2b `geom::edges` +
`geom::seg` (PRs #4, #5)

## Goal

Add the Layer-2 **`edge-node-overlap`** check: flag an edge whose path routes
**through an unrelated node's bounding box** — a node that is not one of the
edge's two endpoint nodes. This is the last of the edge geometry checks; it
reuses the 2a/2b foundation entirely.

## The core problem & approach

A rendered SVG carries **no semantic edge→node mapping**, so "which nodes does
this edge connect" must be inferred from geometry. **Probe finding (real D2
renders):** an edge's endpoints sit ~2px from their node's border (one end
touches exactly, the other has a ~2px arrowhead gap). So an edge's two endpoint
nodes are exactly the nodes whose borders its first/last points are near.

**Approach (the only viable geometric one):** associate each edge with the nodes
near its endpoints, then flag the edge if it enters any _other_ node's bbox. The
endpoint association is what distinguishes "legitimately terminates at a node"
from "routes through an unrelated node."

## Architecture

```
src/
  geom/seg.rs            add: segment_enters_rect(a, b, rx, ry, rw, rh) -> bool (pure, float rect)
  checks/
    mod.rs               register edge_node_overlap in run_all
    edge_node_overlap.rs NEW — Layer-2 edge-through-unrelated-node check
tests/
  fixtures/
    d2-clean.svg            reuse (1 edge, 2 endpoint nodes)            -> Pass
    edge-through-node.svg    hand-authored, edge A→C routes through B    -> Fail
    edge-touches-endpoint.svg hand-authored, edge ends deep inside its
                              own nodes                                  -> Pass (exclusion)
  cli.rs                 integration: check edge-through-node.svg exit 1
```

### `seg::segment_enters_rect` (pure math, no BBox dependency)

```rust
pub fn segment_enters_rect(a: Point, b: Point, rx: f32, ry: f32, rw: f32, rh: f32) -> bool;
```

True iff segment a–b intersects the axis-aligned rect: either endpoint inside
the rect, or the segment properly crosses any of the rect's 4 sides (via
`segment_intersection`). Kept in `seg` (Point + floats only) so the module stays
BBox-free and pure-math.

### `checks::edge_node_overlap` (Layer 2)

```rust
pub const ENDPOINT_MARGIN_PX: f32 = 4.0; // ~2px probe gap + headroom
```

```text
edges = geom::edges(svg); nodes = geom::nodes(svg)
violations = 0
for each edge e (>= 2 points):
    first = e[0]; last = e[last]
    connected = { i | point in (nodes[i] expanded by ENDPOINT_MARGIN_PX)
                       for point in {first, last} }
    for each (i, node) in nodes, i not in connected:
        if any segment of e enters node.bbox  (seg::segment_enters_rect):
            violations += 1
Fail if violations > 0 else Pass; detail reports the count.
```

- `CheckResult { id: "edge-node-overlap", layer: 2, status, detail }`;
  registered in `run_all` after `edge_crossings`.
- Self-loops (both endpoints the same node) → that one node excluded; fine.
- An edge with < 2 points is skipped.

**Semantic rationale:** entering an endpoint node's bbox is expected (the edge
terminates there); entering a _non_-endpoint node is the defect. The 4px
endpoint-association margin (derived from the probe) reliably captures the
arrowhead gap without over-associating.

## Data flow

`check` → `svg::load` (unchanged) → `run_all` also calls
`edge_node_overlap::check`, reusing `geom::edges` + `geom::nodes`. Exit-code
contract unchanged (0/1/2).

## Error handling

- Operates on parsed geometry; returns a count (no fallible I/O).
- 0 nodes or 0 edges → 0 violations → Pass.
- Edges with < 2 points skipped.

## Testing (TDD, real + hand-authored, no mocks)

- **`seg::segment_enters_rect` unit tests:** segment passing through a rect →
  true; segment missing the rect → false; segment with an endpoint inside →
  true.
- **`edge_node_overlap::check` unit tests:** `d2-clean` → Pass;
  `edge-through-node` → Fail (and id `"edge-node-overlap"`);
  `edge-touches-endpoint` → Pass (endpoint exclusion proven — an edge ending
  deep inside its own nodes is not flagged).
- **Integration (`tests/cli.rs`):** `check edge-through-node.svg` → exit 1
  naming `edge-node-overlap` (authored so
  viewBox/aspect/node-overlap/edge-crossings all pass, so only edge-node-overlap
  fails).
- Fixtures: `d2-clean.svg` reused; `edge-through-node.svg` (3 nodes in a row,
  edge A→C straight through B's bbox; viewBox sized so aspect ≤ 2.5) and
  `edge-touches-endpoint.svg` (edge endpoints inside their own nodes)
  hand-authored.

## Scope discipline (YAGNI)

Ships only `edge-node-overlap`. No new deps; reuses `geom::edges`/`nodes`/`seg`.
After this milestone the edge checks are complete; remaining: 2c label overflow,
2d mechanical Layer-0/1.

## Verifiable success criteria

1. `seg::segment_enters_rect` returns true for a through-segment and an
   endpoint-inside segment, false for a missing segment (unit-tested with exact
   values).
2. `edge_node_overlap::check(d2-clean)` → Pass.
3. `edge_node_overlap::check(edge-through-node)` → Fail.
4. `edge_node_overlap::check(edge-touches-endpoint)` → Pass (endpoint exclusion
   proven).
5. `check edge-through-node.svg` → exit 1, output names `edge-node-overlap`.
6. `cargo test` green; `cargo fmt --check` + `cargo clippy -D warnings` clean;
   CI matrix
   - musl guard green.

use crate::checks::{CheckResult, Status};
use crate::geom::{self, seg, BBox};
use crate::svg::Svg;

pub const ENDPOINT_MARGIN_PX: f32 = 4.0;

pub fn check(svg: &Svg) -> CheckResult {
    let edges = geom::edges(svg);
    let nodes = geom::nodes(svg);
    let mut violations = 0usize;
    for e in &edges {
        if e.len() < 2 {
            continue;
        }
        let first = e[0];
        let last = e[e.len() - 1];
        for n in &nodes {
            // Endpoint node: the edge legitimately terminates here — not a defect.
            if point_near(first, n) || point_near(last, n) {
                continue;
            }
            let enters = e
                .windows(2)
                .any(|w| seg::segment_enters_rect(w[0], w[1], n.x, n.y, n.w, n.h));
            if enters {
                violations += 1;
            }
        }
    }
    let status = if violations == 0 {
        Status::Pass
    } else {
        Status::Fail
    };
    CheckResult {
        id: "edge-node-overlap",
        layer: 2,
        status,
        detail: format!("{violations} edge-through-unrelated-node case(s)"),
    }
}

fn point_near(p: seg::Point, n: &BBox) -> bool {
    let m = ENDPOINT_MARGIN_PX;
    p.x >= n.x - m && p.x <= n.right() + m && p.y >= n.y - m && p.y <= n.bottom() + m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    #[test]
    fn d2_clean_passes() {
        let svg = load(Path::new("tests/fixtures/d2-clean.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }

    #[test]
    fn edge_through_node_fails() {
        let svg = load(Path::new("tests/fixtures/edge-through-node.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "edge-node-overlap");
    }

    #[test]
    fn edge_into_own_endpoint_passes() {
        let svg = load(Path::new("tests/fixtures/edge-touches-endpoint.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }
}

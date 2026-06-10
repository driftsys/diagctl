use crate::checks::{CheckResult, Status};
use crate::geom::{self, seg, BBox};
use crate::svg::Svg;

pub const CROSSING_THRESHOLD: usize = 0;
pub const NODE_MARGIN_PX: f32 = 1.0;

pub fn check(svg: &Svg) -> CheckResult {
    let edges = geom::edges(svg);
    let nodes = geom::nodes(svg);
    let mut crossings = 0usize;
    for i in 0..edges.len() {
        for j in (i + 1)..edges.len() {
            crossings += count_open_crossings(&edges[i], &edges[j], &nodes);
        }
    }
    let status = if crossings > CROSSING_THRESHOLD {
        Status::Fail
    } else {
        Status::Pass
    };
    CheckResult {
        id: "edge-crossings",
        layer: 2,
        status,
        detail: format!("{crossings} open-space edge crossing(s)"),
    }
}

fn count_open_crossings(e1: &[seg::Point], e2: &[seg::Point], nodes: &[BBox]) -> usize {
    let mut n = 0;
    for a in e1.windows(2) {
        for b in e2.windows(2) {
            if let Some(p) = seg::segment_intersection(a[0], a[1], b[0], b[1]) {
                if !point_near_any_node(p, nodes) {
                    n += 1;
                }
            }
        }
    }
    n
}

fn point_near_any_node(p: seg::Point, nodes: &[BBox]) -> bool {
    let m = NODE_MARGIN_PX;
    nodes
        .iter()
        .any(|n| p.x >= n.x - m && p.x <= n.right() + m && p.y >= n.y - m && p.y <= n.bottom() + m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    #[test]
    fn cross_clean_passes() {
        let svg = load(Path::new("tests/fixtures/cross-clean.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }

    #[test]
    fn open_crossing_fails() {
        let svg = load(Path::new("tests/fixtures/crossing.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "edge-crossings");
    }

    #[test]
    fn crossing_inside_node_is_excluded() {
        let svg = load(Path::new("tests/fixtures/crossing-at-node.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }
}

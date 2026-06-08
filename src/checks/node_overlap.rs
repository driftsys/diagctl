use crate::checks::{CheckResult, Status};
use crate::geom;
use crate::svg::Svg;

pub const OVERLAP_TOLERANCE_PX: f32 = 1.0;

pub fn check(svg: &Svg) -> CheckResult {
    let nodes = geom::nodes(svg);
    let t = OVERLAP_TOLERANCE_PX / 2.0;
    let mut overlaps = 0usize;
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let (a, b) = (nodes[i], nodes[j]);
            // Skip true containment (a node inside a strictly larger container).
            if (a.contains(&b) && a.area() > b.area()) || (b.contains(&a) && b.area() > a.area()) {
                continue;
            }
            if a.inset(t).intersects(&b.inset(t)) {
                overlaps += 1;
            }
        }
    }
    let status = if overlaps == 0 {
        Status::Pass
    } else {
        Status::Fail
    };
    let detail = if overlaps == 0 {
        format!("{} node(s), no overlapping pairs", nodes.len())
    } else {
        format!("{overlaps} overlapping node pair(s) among {} nodes", nodes.len())
    };
    CheckResult {
        id: "node-overlap",
        layer: 2,
        status,
        detail,
    }
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
    fn plantuml_clean_passes() {
        let svg = load(Path::new("tests/fixtures/plantuml-clean.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }

    #[test]
    fn overlap_fixture_fails() {
        let svg = load(Path::new("tests/fixtures/overlap.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "node-overlap");
    }
}

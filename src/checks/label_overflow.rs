use crate::checks::{CheckResult, Status};
use crate::svg::{self, Svg};

/// Antialiasing / rounding headroom, in px.
pub const OVERFLOW_ABS_FLOOR_PX: f32 = 2.0;
/// Fraction of a label's own size tolerated past an edge — absorbs the bounded width
/// difference between the bundled font and the diagram's real font.
pub const OVERFLOW_SIZE_FRACTION: f32 = 0.15;

pub fn check(svg: &Svg) -> CheckResult {
    let tree = match svg::shaped_tree(svg) {
        Ok(t) => t,
        Err(_) => {
            return CheckResult {
                id: "label-overflow",
                layer: 2,
                status: Status::Pass,
                detail: "unshapeable; skipped".to_string(),
            };
        }
    };
    let size = tree.size();
    let (cw, ch) = (size.width(), size.height());

    let mut bboxes: Vec<usvg::Rect> = Vec::new();
    collect_text_bboxes(tree.root(), &mut bboxes);

    let mut overflowing = 0usize;
    let mut worst = 0.0f32;
    for b in &bboxes {
        let (x, y, w, h) = (b.x(), b.y(), b.width(), b.height());
        let tol_x = OVERFLOW_ABS_FLOOR_PX.max(OVERFLOW_SIZE_FRACTION * w);
        let tol_y = OVERFLOW_ABS_FLOOR_PX.max(OVERFLOW_SIZE_FRACTION * h);
        let h_over = (-x).max(x + w - cw).max(0.0);
        let v_over = (-y).max(y + h - ch).max(0.0);
        if h_over > tol_x || v_over > tol_y {
            overflowing += 1;
            worst = worst.max(h_over).max(v_over);
        }
    }

    let status = if overflowing == 0 {
        Status::Pass
    } else {
        Status::Fail
    };
    let detail = if bboxes.is_empty() {
        "no labels".to_string()
    } else if overflowing == 0 {
        format!("{} label(s) within canvas", bboxes.len())
    } else {
        format!("{overflowing} label(s) overflow viewBox; worst {worst:.0}px")
    };
    CheckResult {
        id: "label-overflow",
        layer: 2,
        status,
        detail,
    }
}

/// Collect the absolute bounding box of every `Text` node, recursing through groups.
/// Canvas origin is assumed `(0,0)` — the near-universal case for renderer viewBoxes.
fn collect_text_bboxes(g: &usvg::Group, out: &mut Vec<usvg::Rect>) {
    for n in g.children() {
        match n {
            usvg::Node::Group(gg) => collect_text_bboxes(gg, out),
            usvg::Node::Text(t) => out.push(t.abs_bounding_box()),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    fn run(fixture: &str) -> CheckResult {
        check(&load(Path::new(fixture)).unwrap())
    }

    #[test]
    fn label_fits_passes() {
        assert_eq!(run("tests/fixtures/label-fits.svg").status, Status::Pass);
    }

    #[test]
    fn overflowing_label_fails() {
        let r = run("tests/fixtures/label-overflow.svg");
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "label-overflow");
    }

    #[test]
    fn d2_clean_passes() {
        assert_eq!(run("tests/fixtures/d2-clean.svg").status, Status::Pass);
    }

    #[test]
    fn plantuml_clean_passes() {
        assert_eq!(
            run("tests/fixtures/plantuml-clean.svg").status,
            Status::Pass
        );
    }

    #[test]
    fn in_band_no_labels_passes() {
        assert_eq!(run("tests/fixtures/in-band.svg").status, Status::Pass);
    }
}

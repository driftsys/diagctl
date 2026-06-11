use crate::checks::{CheckResult, Status};
use crate::svg::Svg;
use usvg::{Group, Node};

pub const MIN_STROKE_WIDTH: f32 = 0.5;

pub fn check(svg: &Svg) -> CheckResult {
    let mut widths = Vec::new();
    collect_stroke_widths(svg.tree.root(), &mut widths);
    let mut thin = 0usize;
    let mut min_w = f32::INFINITY;
    for w in widths {
        if w < MIN_STROKE_WIDTH {
            thin += 1;
        }
        if w < min_w {
            min_w = w;
        }
    }
    let status = if thin == 0 {
        Status::Pass
    } else {
        Status::Fail
    };
    let detail = if min_w.is_finite() {
        format!("{thin} stroke(s) below {MIN_STROKE_WIDTH}px; thinnest {min_w:.2}px")
    } else {
        "no strokes".to_string()
    };
    CheckResult {
        id: "min-stroke-width",
        layer: 1,
        status,
        detail,
    }
}

fn collect_stroke_widths(g: &Group, out: &mut Vec<f32>) {
    for n in g.children() {
        match n {
            Node::Group(gg) => collect_stroke_widths(gg, out),
            Node::Path(p) => {
                if let Some(s) = p.stroke() {
                    out.push(s.width().get());
                }
            }
            _ => {}
        }
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
    fn thin_stroke_fails() {
        let svg = load(Path::new("tests/fixtures/thin-stroke.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "min-stroke-width");
    }
}

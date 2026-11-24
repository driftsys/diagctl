//! `opaque-background` (`--target markdown`): the first painted element is a
//! rectangle that covers the whole image, filled with the contract background
//! colour at full opacity.
//!
//! A transparent SVG shows the page colour through it, so dark text becomes
//! unreadable on a dark page (#21). Elements whose fill and stroke are fully
//! transparent paint nothing and are skipped. `usvg` turns a root
//! `background-color` into a painted rectangle, so that property counts; the
//! CSS `background` shorthand does not.

use crate::checks::{CheckResult, Status};
use crate::contract::BACKGROUND;
use crate::svg::Svg;
use usvg::tiny_skia_path::PathSegment;
use usvg::{Group, Node, Paint, Path};

/// Allowed gap between the rectangle and the image edge, in px.
const EDGE_TOLERANCE_PX: f32 = 0.5;

pub fn check(svg: &Svg) -> CheckResult {
    let (status, detail) = evaluate(&svg.tree);
    CheckResult {
        id: "opaque-background",
        layer: 1,
        status,
        detail,
    }
}

fn evaluate(tree: &usvg::Tree) -> (Status, String) {
    let fail = |d: String| (Status::Fail, d);
    let Some((path, opacity)) = first_painted(tree.root(), 1.0) else {
        return fail("nothing is painted; no background".to_string());
    };
    let Some(path) = path else {
        return fail("first painted element is not a filled rectangle".to_string());
    };
    let Some(fill) = path.fill() else {
        return fail("first painted element has no fill".to_string());
    };
    if !is_rectangle(path) {
        return fail("first painted element is not a rectangle".to_string());
    }
    let size = tree.size();
    let b = path.abs_bounding_box();
    let covers = b.left() <= EDGE_TOLERANCE_PX
        && b.top() <= EDGE_TOLERANCE_PX
        && b.right() >= size.width() - EDGE_TOLERANCE_PX
        && b.bottom() >= size.height() - EDGE_TOLERANCE_PX;
    if !covers {
        return fail(format!(
            "first painted rectangle {:.1} x {:.1} px does not cover the {:.1} x {:.1} px image",
            b.width(),
            b.height(),
            size.width(),
            size.height()
        ));
    }
    let Paint::Color(c) = fill.paint() else {
        return fail("background fill is not a solid colour".to_string());
    };
    let hex = format!("#{:02x}{:02x}{:02x}", c.red, c.green, c.blue);
    if hex != BACKGROUND {
        return fail(format!("background fill is {hex}, not {BACKGROUND}"));
    }
    let alpha = fill.opacity().get() * opacity;
    if alpha < 1.0 {
        return fail(format!("background opacity is {alpha:.2}, not 1"));
    }
    (Status::Pass, format!("opaque {BACKGROUND} background"))
}

/// The first element that paints, in paint order, with the product of the
/// opacities of its parent groups. `Some(None)` means the first painted
/// element is not a path (text or an image).
fn first_painted(g: &Group, opacity: f32) -> Option<(Option<&Path>, f32)> {
    let opacity = opacity * g.opacity().get();
    for n in g.children() {
        match n {
            Node::Group(gg) => {
                if let Some(found) = first_painted(gg, opacity) {
                    return Some(found);
                }
            }
            Node::Path(p) => {
                let fill = p.fill().is_some_and(|f| f.opacity().get() > 0.0);
                let stroke = p.stroke().is_some_and(|s| s.opacity().get() > 0.0);
                if p.is_visible() && (fill || stroke) {
                    return Some((Some(p), opacity));
                }
            }
            Node::Image(_) | Node::Text(_) => return Some((None, opacity)),
        }
    }
    None
}

/// True when every point of the path lies on a corner of its bounding box and
/// the path has only straight segments.
fn is_rectangle(p: &Path) -> bool {
    let b = p.data().bounds();
    let on_corner = |x: f32, y: f32| {
        let near = |a: f32, v: f32| (a - v).abs() <= EDGE_TOLERANCE_PX;
        (near(x, b.left()) || near(x, b.right())) && (near(y, b.top()) || near(y, b.bottom()))
    };
    p.data().segments().all(|s| match s {
        PathSegment::MoveTo(pt) | PathSegment::LineTo(pt) => on_corner(pt.x, pt.y),
        PathSegment::Close => true,
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path as FsPath;

    fn eval(body: &str) -> (Status, String) {
        let raw = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100" width="200" height="100">{body}</svg>"#
        );
        evaluate(&usvg::Tree::from_str(&raw, &usvg::Options::default()).unwrap())
    }

    #[test]
    fn rule_table() {
        // (body, expected status, text the detail must contain)
        let cases = [
            (
                r##"<rect width="100%" height="100%" fill="#ffffff"/>"##,
                Status::Pass,
                "opaque",
            ),
            (
                r##"<rect width="200" height="100" fill="#FFF"/><rect x="10" y="10" width="5" height="5" fill="#000"/>"##,
                Status::Pass,
                "opaque",
            ),
            (
                r##"<g><rect width="200" height="100" style="fill:white"/></g>"##,
                Status::Pass,
                "opaque",
            ),
            (
                r##"<rect x="-5" y="-5" width="210" height="110" fill="#ffffff"/>"##,
                Status::Pass,
                "opaque",
            ),
            ("", Status::Fail, "nothing is painted"),
            (
                r##"<rect width="200" height="100" fill="none"/>"##,
                Status::Fail,
                "nothing is painted",
            ),
            (
                r##"<rect width="200" height="100" fill="transparent"/>"##,
                Status::Fail,
                "nothing is painted",
            ),
            (
                r##"<rect width="200" height="100" fill="#ffffff" fill-opacity="0"/><rect width="200" height="100" fill="#ffffff"/>"##,
                Status::Pass,
                "opaque",
            ),
            (
                r##"<rect x="10" y="10" width="20" height="20" fill="#000"/><rect width="200" height="100" fill="#ffffff"/>"##,
                Status::Fail,
                "does not cover",
            ),
            (
                r##"<rect width="199" height="100" fill="#ffffff"/>"##,
                Status::Fail,
                "does not cover",
            ),
            (
                r##"<rect width="200" height="100" fill="#fafafa"/>"##,
                Status::Fail,
                "#fafafa, not #ffffff",
            ),
            (
                r##"<rect width="200" height="100" fill="#ffffff" fill-opacity="0.5"/>"##,
                Status::Fail,
                "opacity is 0.50",
            ),
            (
                r##"<g opacity="0.9"><rect width="200" height="100" fill="#ffffff"/></g>"##,
                Status::Fail,
                "opacity is 0.90",
            ),
            (
                r##"<rect width="200" height="100" rx="10" fill="#ffffff"/>"##,
                Status::Fail,
                "not a rectangle",
            ),
            (
                r##"<circle cx="100" cy="50" r="120" fill="#ffffff"/>"##,
                Status::Fail,
                "not a rectangle",
            ),
            (
                r##"<rect width="200" height="100" fill="none" stroke="#000"/>"##,
                Status::Fail,
                "no fill",
            ),
            (
                r##"<image width="200" height="100" href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg=="/>"##,
                Status::Fail,
                "not a filled rectangle",
            ),
        ];
        for (body, want, text) in cases {
            let (status, detail) = eval(body);
            assert_eq!(status, want, "{body}: {detail}");
            assert!(detail.contains(text), "{body}: {detail}");
        }
    }

    fn eval_root_style(style: &str) -> (Status, String) {
        let raw = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" style="{style}" viewBox="0 0 200 100" width="200" height="100"><rect x="10" y="10" width="5" height="5" fill="#000"/></svg>"##
        );
        evaluate(&usvg::Tree::from_str(&raw, &usvg::Options::default()).unwrap())
    }

    #[test]
    fn root_css_background() {
        assert_eq!(eval_root_style("background-color: #ffffff").0, Status::Pass);
        assert_eq!(
            eval_root_style("background-color: transparent").0,
            Status::Fail
        );
        assert_eq!(eval_root_style("background: #ffffff").0, Status::Fail);
    }

    #[test]
    fn renderer_fixtures() {
        let cases = [
            ("bg-d2.svg", Status::Pass),
            ("bg-d2-transparent.svg", Status::Fail),
            ("bg-drawio.svg", Status::Pass),
            ("bg-drawio-transparent.svg", Status::Fail),
            // PlantUML sets the background only in the root CSS `style`.
            ("bg-plantuml.svg", Status::Fail),
            ("md-valid.svg", Status::Pass),
        ];
        for (name, want) in cases {
            let svg = load(&FsPath::new("tests/fixtures").join(name)).unwrap();
            let r = check(&svg);
            assert_eq!(r.status, want, "{name}: {}", r.detail);
            assert_eq!((r.id, r.layer), ("opaque-background", 1));
        }
    }
}

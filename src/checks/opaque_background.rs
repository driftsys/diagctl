//! `opaque-background` (`--target markdown`): the first painted element is a
//! rectangle that covers the whole image, filled with the contract background
//! colour at full opacity.
//!
//! A transparent SVG shows the page colour through it, so dark text becomes
//! unreadable on a dark page (#21).
//!
//! - Elements that paint nothing are skipped: hidden elements, fully
//!   transparent fill and stroke, and groups with opacity 0.
//! - The rectangle must stay an axis-aligned rectangle after its transforms.
//! - A clip on the rectangle or its parent groups must itself be a rectangle
//!   that covers the image. A mask or a filter fails the check.
//! - `usvg` turns a root `background-color` into a painted rectangle, so that
//!   property counts. The CSS `background` shorthand, which PlantUML writes,
//!   does not.
//! - Text is not seen: `svg::load` has no fonts, so `usvg` drops text. Text
//!   painted before the rectangle would be hidden by it.

use crate::checks::{CheckResult, Status};
use crate::contract::BACKGROUND;
use crate::svg::Svg;
use usvg::tiny_skia_path::{PathSegment, Point};
use usvg::{Group, Node, Paint, Path, Rect, Size, Transform};

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

/// What `first_painted` finds.
enum Painted<'a> {
    Path {
        path: &'a Path,
        opacity: f32,
        /// Set when a parent group has a clip that does not cover the image,
        /// a mask or a filter.
        effect: bool,
    },
    Image,
}

fn evaluate(tree: &usvg::Tree) -> (Status, String) {
    let fail = |d: String| (Status::Fail, d);
    let size = tree.size();
    let Some(painted) = first_painted(tree.root(), 1.0, false, size) else {
        return fail("nothing is painted; no background".to_string());
    };
    let Painted::Path {
        path,
        opacity,
        effect,
    } = painted
    else {
        return fail("first painted element is an image, not a filled rectangle".to_string());
    };
    let Some(fill) = path.fill() else {
        return fail("first painted element has no fill".to_string());
    };
    let Some(b) = axis_rect(path.data(), path.abs_transform()) else {
        return fail("first painted element is not a rectangle".to_string());
    };
    if !covers(b, size) {
        return fail(format!(
            "first painted rectangle {:.1} x {:.1} px does not cover the {:.1} x {:.1} px image",
            b.width(),
            b.height(),
            size.width(),
            size.height()
        ));
    }
    if effect {
        return fail("background is clipped, masked or filtered".to_string());
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

/// The first element that paints, in paint order. `opacity` is the product of
/// the parent group opacities.
fn first_painted(g: &Group, opacity: f32, effect: bool, size: Size) -> Option<Painted<'_>> {
    let opacity = opacity * g.opacity().get();
    if opacity <= 0.0 {
        return None;
    }
    let effect = effect
        || g.mask().is_some()
        || !g.filters().is_empty()
        || g.clip_path().is_some_and(|c| !clip_covers(g, c, size));
    for n in g.children() {
        match n {
            Node::Group(gg) => {
                if let Some(found) = first_painted(gg, opacity, effect, size) {
                    return Some(found);
                }
            }
            Node::Path(p) => {
                let fill = p.fill().is_some_and(|f| f.opacity().get() > 0.0);
                let stroke = p.stroke().is_some_and(|s| s.opacity().get() > 0.0);
                if p.is_visible() && (fill || stroke) {
                    return Some(Painted::Path {
                        path: p,
                        opacity,
                        effect,
                    });
                }
            }
            Node::Image(i) => {
                if i.is_visible() {
                    return Some(Painted::Image);
                }
            }
            Node::Text(_) => {}
        }
    }
    None
}

/// True when the clip of group `g` is one rectangle that covers the image.
fn clip_covers(g: &Group, clip: &usvg::ClipPath, size: Size) -> bool {
    if clip.clip_path().is_some() {
        return false;
    }
    let ts = g.abs_transform().pre_concat(clip.transform());
    match clip.root().children() {
        [Node::Path(p)] => {
            axis_rect(p.data(), ts.pre_concat(p.abs_transform())).is_some_and(|r| covers(r, size))
        }
        _ => false,
    }
}

/// The bounds of `data` after `ts`, when the result is an axis-aligned
/// rectangle: four distinct corners joined by edges that alternate between
/// horizontal and vertical.
fn axis_rect(data: &usvg::tiny_skia_path::Path, ts: Transform) -> Option<Rect> {
    let mut pts: Vec<Point> = Vec::new();
    for seg in data.segments() {
        match seg {
            PathSegment::MoveTo(p) if pts.is_empty() => pts.push(p),
            PathSegment::LineTo(p) => pts.push(p),
            PathSegment::Close => {}
            _ => return None,
        }
    }
    ts.map_points(&mut pts);
    let near = |a: f32, b: f32| (a - b).abs() <= EDGE_TOLERANCE_PX;
    let same = |a: Point, b: Point| near(a.x, b.x) && near(a.y, b.y);
    if pts.len() == 5 && same(pts[0], pts[4]) {
        pts.pop();
    }
    if pts.len() != 4 {
        return None;
    }
    let mut vertical = Vec::with_capacity(4);
    for i in 0..4 {
        let (a, b) = (pts[i], pts[(i + 1) % 4]);
        if same(a, b) || !(near(a.x, b.x) || near(a.y, b.y)) {
            return None;
        }
        vertical.push(near(a.x, b.x));
    }
    if vertical[0] == vertical[1] || vertical[1] == vertical[2] || vertical[2] == vertical[3] {
        return None;
    }
    Rect::from_points(&pts)
}

fn covers(r: Rect, size: Size) -> bool {
    r.left() <= EDGE_TOLERANCE_PX
        && r.top() <= EDGE_TOLERANCE_PX
        && r.right() >= size.width() - EDGE_TOLERANCE_PX
        && r.bottom() >= size.height() - EDGE_TOLERANCE_PX
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
            // Elements that paint nothing are skipped.
            (
                r##"<g opacity="0"><rect width="10" height="10" fill="#000"/></g><rect width="200" height="100" fill="#ffffff"/>"##,
                Status::Pass,
                "opaque",
            ),
            (
                r##"<rect width="10" height="10" fill="#000" opacity="0"/><rect width="200" height="100" fill="#ffffff"/>"##,
                Status::Pass,
                "opaque",
            ),
            (
                r##"<rect width="200" height="100" fill="#000" visibility="hidden"/><rect width="200" height="100" fill="#ffffff"/>"##,
                Status::Pass,
                "opaque",
            ),
            (
                r##"<rect width="200" height="100" fill="none" stroke="#000" stroke-opacity="0"/><rect width="200" height="100" fill="#ffffff"/>"##,
                Status::Pass,
                "opaque",
            ),
            // Text is dropped by usvg (no fonts), and is hidden by the rect anyway.
            (
                r##"<text x="10" y="10">Hi</text><rect width="200" height="100" fill="#ffffff"/>"##,
                Status::Pass,
                "opaque",
            ),
            // A gap on any one edge fails; 0.5 px is tolerated.
            (
                r##"<rect x="5" width="195" height="100" fill="#ffffff"/>"##,
                Status::Fail,
                "does not cover",
            ),
            (
                r##"<rect y="5" width="200" height="95" fill="#ffffff"/>"##,
                Status::Fail,
                "does not cover",
            ),
            (
                r##"<rect width="200" height="95" fill="#ffffff"/>"##,
                Status::Fail,
                "does not cover",
            ),
            (
                r##"<rect x="0.3" y="0.3" width="199.4" height="99.4" fill="#ffffff"/>"##,
                Status::Pass,
                "opaque",
            ),
            // Every channel must be ff.
            (
                r##"<rect width="200" height="100" fill="#ffff00"/>"##,
                Status::Fail,
                "#ffff00, not #ffffff",
            ),
            (
                r##"<rect width="200" height="100" fill="#ff00ff"/>"##,
                Status::Fail,
                "#ff00ff, not #ffffff",
            ),
            (
                r##"<rect width="200" height="100" fill="#00ffff"/>"##,
                Status::Fail,
                "#00ffff, not #ffffff",
            ),
            // Opacity of every parent group counts.
            (
                r##"<g opacity="0.9"><g><rect width="200" height="100" fill="#ffffff"/></g></g>"##,
                Status::Fail,
                "opacity is 0.90",
            ),
            // Shapes that cover the bounding box only partly.
            (
                r##"<path d="M0 0 L200 0 L200 100 Z" fill="#ffffff"/>"##,
                Status::Fail,
                "not a rectangle",
            ),
            (
                r##"<path d="M0 0 H200 V100 Z" fill="#ffffff"/>"##,
                Status::Fail,
                "not a rectangle",
            ),
            (
                r##"<path d="M0 0 H200 H0 V100 Z" fill="#ffffff"/>"##,
                Status::Fail,
                "not a rectangle",
            ),
            (
                r##"<path d="M0 0 L200 0 L200 100 L0 100 C50 60 50 40 0 0 Z" fill="#ffffff"/>"##,
                Status::Fail,
                "not a rectangle",
            ),
            (
                r##"<rect width="200" height="100" fill="#ffffff" transform="rotate(45 100 50)"/>"##,
                Status::Fail,
                "not a rectangle",
            ),
            (
                r##"<path d="M0 0 H200 V100 H0 Z" fill="#ffffff"/>"##,
                Status::Pass,
                "opaque",
            ),
            (
                r##"<rect width="100" height="50" fill="#ffffff" transform="scale(2)"/>"##,
                Status::Pass,
                "opaque",
            ),
            // Clips, masks and filters.
            (
                r##"<clipPath id="c"><rect width="100" height="100"/></clipPath><g clip-path="url(#c)"><rect width="200" height="100" fill="#ffffff"/></g>"##,
                Status::Fail,
                "clipped, masked or filtered",
            ),
            (
                r##"<clipPath id="c"><rect width="200" height="100"/></clipPath><g clip-path="url(#c)"><rect width="200" height="100" fill="#ffffff"/></g>"##,
                Status::Pass,
                "opaque",
            ),
            (
                r##"<mask id="m"><rect width="200" height="100" fill="#fff"/></mask><g mask="url(#m)"><rect width="200" height="100" fill="#ffffff"/></g>"##,
                Status::Fail,
                "clipped, masked or filtered",
            ),
            (
                r##"<filter id="f"><feGaussianBlur stdDeviation="2"/></filter><g filter="url(#f)"><rect width="200" height="100" fill="#ffffff"/></g>"##,
                Status::Fail,
                "clipped, masked or filtered",
            ),
            // Gradient fill.
            (
                r##"<linearGradient id="g"><stop offset="0" stop-color="#fff"/><stop offset="1" stop-color="#fff"/></linearGradient><rect width="200" height="100" fill="url(#g)"/>"##,
                Status::Fail,
                "not a solid colour",
            ),
            (
                r##"<image width="200" height="100" href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg=="/>"##,
                Status::Fail,
                "is an image",
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
    fn viewbox_scaled_to_width() {
        let tree = |rect: &str| {
            let raw = format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 200" width="200" height="100">{rect}</svg>"##
            );
            evaluate(&usvg::Tree::from_str(&raw, &usvg::Options::default()).unwrap()).0
        };
        assert_eq!(
            tree(r##"<rect width="400" height="200" fill="#fff"/>"##),
            Status::Pass
        );
        assert_eq!(
            tree(r##"<rect width="300" height="200" fill="#fff"/>"##),
            Status::Fail
        );
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
        // (fixture, expected status, text the detail must contain)
        let cases = [
            ("bg-d2.svg", Status::Pass, "opaque"),
            // The transparent rect is skipped; the first shape is small.
            ("bg-d2-transparent.svg", Status::Fail, "does not cover"),
            ("bg-drawio.svg", Status::Pass, "opaque"),
            ("bg-drawio-transparent.svg", Status::Fail, "does not cover"),
            // PlantUML writes its white background only as the root CSS
            // `background` shorthand, which is not read, so no real
            // PlantUML export passes.
            ("bg-plantuml-css-only.svg", Status::Fail, "not a rectangle"),
            ("md-valid.svg", Status::Pass, "opaque"),
        ];
        for (name, want, text) in cases {
            let svg = load(&FsPath::new("tests/fixtures").join(name)).unwrap();
            let r = check(&svg);
            assert_eq!(r.status, want, "{name}: {}", r.detail);
            assert!(r.detail.contains(text), "{name}: {}", r.detail);
            assert_eq!((r.id, r.layer), ("opaque-background", 1));
        }
    }
}

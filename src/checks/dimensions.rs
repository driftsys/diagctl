//! `dimensions` (`--target markdown`): the root `<svg>` has a `viewBox`, a
//! `width` and a `height` in px with the same aspect ratio as the `viewBox`,
//! and a `width` no larger than the display column.
//!
//! A Markdown renderer shows the SVG through `<img>`. Without `width` and
//! `height` the image fills the column, so its displayed size is unknown
//! (#14, step 0).

use crate::checks::{CheckResult, Status};
use crate::contract::COLUMN_WIDTH_PX;
use crate::svg::Svg;

/// Largest allowed relative difference between the two aspect ratios.
const ASPECT_TOLERANCE: f32 = 0.01;

pub fn check(svg: &Svg) -> CheckResult {
    let (status, detail) = match roxmltree::Document::parse(&svg.raw) {
        Ok(doc) => evaluate(doc.root_element()),
        Err(_) => (Status::Pass, "unparseable; skipped".to_string()),
    };
    CheckResult {
        id: "dimensions",
        layer: 0,
        status,
        detail,
    }
}

fn evaluate(root: roxmltree::Node) -> (Status, String) {
    let fail = |d: String| (Status::Fail, d);
    let Some((vb_w, vb_h)) = root.attribute("viewBox").and_then(parse_viewbox) else {
        return fail("root <svg> has no valid viewBox".to_string());
    };
    let (Some(w), Some(h)) = (root.attribute("width"), root.attribute("height")) else {
        return fail("root <svg> needs both width and height for Markdown display".to_string());
    };
    let (Some(w), Some(h)) = (parse_px(w), parse_px(h)) else {
        return fail(format!(
            "width/height must be px numbers (got {w:?} x {h:?})"
        ));
    };
    let ratio = w / h;
    let vb_ratio = vb_w / vb_h;
    if ((ratio - vb_ratio) / vb_ratio).abs() > ASPECT_TOLERANCE {
        return fail(format!(
            "width/height aspect {ratio:.3} differs from viewBox aspect {vb_ratio:.3}"
        ));
    }
    if w > COLUMN_WIDTH_PX {
        return fail(format!(
            "width {w} px is wider than the {COLUMN_WIDTH_PX} px column"
        ));
    }
    (
        Status::Pass,
        format!("{w} x {h} px, fits the {COLUMN_WIDTH_PX} px column"),
    )
}

fn parse_viewbox(v: &str) -> Option<(f32, f32)> {
    let n: Vec<f32> = v
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    match n[..] {
        [_, _, w, h] if w > 0.0 && h > 0.0 => Some((w, h)),
        _ => None,
    }
}

fn parse_px(v: &str) -> Option<f32> {
    let v = v.trim();
    let n: f32 = v.strip_suffix("px").unwrap_or(v).trim().parse().ok()?;
    (n > 0.0).then_some(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    fn run(name: &str) -> CheckResult {
        check(&load(Path::new(&format!("tests/fixtures/{name}.svg"))).unwrap())
    }

    #[test]
    fn valid_passes() {
        assert_eq!(run("md-valid").status, Status::Pass);
    }

    #[test]
    fn missing_dimensions_fails() {
        let r = run("in-band");
        assert_eq!(r.status, Status::Fail);
        assert!(r.detail.contains("width and height"), "{}", r.detail);
    }

    #[test]
    fn width_only_fails() {
        assert_eq!(run("md-width-only").status, Status::Fail);
    }

    #[test]
    fn percent_width_fails() {
        let r = run("md-percent");
        assert_eq!(r.status, Status::Fail);
        assert!(r.detail.contains("px numbers"), "{}", r.detail);
    }

    #[test]
    fn mismatched_aspect_fails() {
        let r = run("md-aspect");
        assert_eq!(r.status, Status::Fail);
        assert!(r.detail.contains("aspect"), "{}", r.detail);
    }

    #[test]
    fn wider_than_column_fails() {
        let r = run("md-wide");
        assert_eq!(r.status, Status::Fail);
        assert!(
            r.detail.contains("wider than the 800 px column"),
            "{}",
            r.detail
        );
    }

    #[test]
    fn no_viewbox_fails() {
        assert_eq!(run("no-viewbox").status, Status::Fail);
    }

    #[test]
    fn px_suffix_parses() {
        assert_eq!(parse_px("400px"), Some(400.0));
        assert_eq!(parse_px("2em"), None);
        assert_eq!(parse_px("0"), None);
    }

    #[test]
    fn width_equal_to_column_passes() {
        let root_src = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 400" width="800" height="400"/>"#;
        let doc = roxmltree::Document::parse(root_src).unwrap();
        assert_eq!(evaluate(doc.root_element()).0, Status::Pass);
    }
}

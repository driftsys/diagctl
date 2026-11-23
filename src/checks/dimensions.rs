//! `dimensions` (`--target markdown`): the root `<svg>` has a `viewBox`, a
//! `width` and a `height` in px whose aspect ratio is within 1 % of the
//! `viewBox` aspect ratio, and a `width` no larger than the display column.
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

pub(crate) fn parse_viewbox(v: &str) -> Option<(f32, f32)> {
    let n: Vec<f32> = v
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    match n[..] {
        [_, _, w, h] if w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 => Some((w, h)),
        _ => None,
    }
}

pub(crate) fn parse_px(v: &str) -> Option<f32> {
    let v = v.trim();
    let n: f32 = v.strip_suffix("px").unwrap_or(v).trim().parse().ok()?;
    (n.is_finite() && n > 0.0).then_some(n)
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
        let r = run("md-width-only");
        assert_eq!(r.status, Status::Fail);
        assert!(r.detail.contains("width and height"), "{}", r.detail);
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
        let r = run("no-viewbox");
        assert_eq!(r.status, Status::Fail);
        assert!(r.detail.contains("no valid viewBox"), "{}", r.detail);
    }

    #[test]
    fn px_suffix_parses() {
        assert_eq!(parse_px("400px"), Some(400.0));
        assert_eq!(parse_px("2em"), None);
        assert_eq!(parse_px("0"), None);
    }

    /// Runs `evaluate` on a root `<svg>` with the given attributes.
    fn eval(attrs: &str) -> (Status, String) {
        let src = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" {attrs}/>"#);
        let doc = roxmltree::Document::parse(&src).unwrap();
        evaluate(doc.root_element())
    }

    #[test]
    fn rule_table() {
        // (attributes, expected status, text the detail must contain)
        let cases = [
            // Displayed width is limited, not the viewBox size.
            (
                r#"viewBox="0 0 1600 400" width="800" height="200""#,
                Status::Pass,
                "fits",
            ),
            (
                r#"viewBox="0 0 400 200" width="1600" height="800""#,
                Status::Fail,
                "wider than",
            ),
            // Height is not limited.
            (
                r#"viewBox="0 0 400 1200" width="400" height="1200""#,
                Status::Pass,
                "fits",
            ),
            // Aspect mismatch in both directions.
            (
                r#"viewBox="0 0 400 200" width="400" height="300""#,
                Status::Fail,
                "aspect",
            ),
            (
                r#"viewBox="0 0 400 200" width="400" height="100""#,
                Status::Fail,
                "aspect",
            ),
            // 1 % tolerance boundary: 400/201 is 0.5 % off, 400/204 is 2 % off.
            (
                r#"viewBox="0 0 400 200" width="400" height="201""#,
                Status::Pass,
                "fits",
            ),
            (
                r#"viewBox="0 0 400 200" width="400" height="204""#,
                Status::Fail,
                "aspect",
            ),
            // Presence and units.
            (
                r#"viewBox="0 0 400 200" width="400""#,
                Status::Fail,
                "width and height",
            ),
            (
                r#"viewBox="0 0 400 200" height="200""#,
                Status::Fail,
                "width and height",
            ),
            (
                r#"viewBox="0 0 400 200" width="100%" height="200""#,
                Status::Fail,
                "px numbers",
            ),
            (
                r#"viewBox="0 0 400 200" width="400" height="50%""#,
                Status::Fail,
                "px numbers",
            ),
            (
                r#"viewBox="0 0 400 200" width="-400" height="-200""#,
                Status::Fail,
                "px numbers",
            ),
            (
                r#"viewBox="0 0 400 200" width="400px" height="200px""#,
                Status::Pass,
                "fits",
            ),
            // viewBox validity.
            (
                r#"width="400" height="200""#,
                Status::Fail,
                "no valid viewBox",
            ),
            (
                r#"viewBox="0 0 0 200" width="400" height="200""#,
                Status::Fail,
                "no valid viewBox",
            ),
            (
                r#"viewBox="0 0 inf 200" width="400" height="200""#,
                Status::Fail,
                "no valid viewBox",
            ),
            (
                r#"viewBox="0 0 400" width="400" height="200""#,
                Status::Fail,
                "no valid viewBox",
            ),
            (
                r#"viewBox="0 0 400 200 9" width="400" height="200""#,
                Status::Fail,
                "no valid viewBox",
            ),
        ];
        for (attrs, status, text) in cases {
            let (got, detail) = eval(attrs);
            assert_eq!(got, status, "{attrs}: {detail}");
            assert!(detail.contains(text), "{attrs}: {detail}");
        }
    }

    #[test]
    fn width_equal_to_column_passes() {
        let root_src = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 400" width="800" height="400"/>"#;
        let doc = roxmltree::Document::parse(root_src).unwrap();
        assert_eq!(evaluate(doc.root_element()).0, Status::Pass);
    }
}

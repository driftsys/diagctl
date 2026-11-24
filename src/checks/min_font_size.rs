use crate::checks::{dimensions, CheckResult, Status};
use crate::contract::{COLUMN_WIDTH_PX, MIN_DISPLAYED_FONT_PX};
use crate::svg::Svg;

/// Authored font floor for `--target html`.
pub const MIN_FONT_SIZE: f32 = 8.0;

/// Margin for f32 rounding, so text displayed at exactly the floor passes.
const EPSILON_PX: f32 = 1e-3;

/// `--target markdown`: every px-sized text must be at least
/// `MIN_DISPLAYED_FONT_PX` after the image is scaled for display.
///
/// Font sizes are in `viewBox` units. The image is displayed
/// min(`width`, column) px wide, or the full column when `width` is absent
/// or not a positive px number, so the scale is that displayed width divided
/// by the `viewBox` width. With an absent or invalid `viewBox`, user units are
/// the `width` units. A failure lists each distinct failing size once, with
/// its count. Only a `font-size` set on the `<text>` or `<tspan>` itself is
/// read: sizes inherited from a parent or set in `<style>`, and transforms,
/// are not taken into account.
pub fn check_markdown(svg: &Svg) -> CheckResult {
    let (status, detail) = match roxmltree::Document::parse(&svg.raw) {
        Ok(doc) => evaluate_markdown(&doc),
        Err(_) => (Status::Pass, "unparseable; skipped".to_string()),
    };
    CheckResult {
        id: "min-font-size",
        layer: 1,
        status,
        detail,
    }
}

fn evaluate_markdown(doc: &roxmltree::Document) -> (Status, String) {
    let scale = display_scale(doc.root_element());
    let mut failing: Vec<f32> = Vec::new();
    let mut min_seen = f32::INFINITY;
    for px in text_sizes(doc) {
        if px * scale + EPSILON_PX < MIN_DISPLAYED_FONT_PX {
            failing.push(px);
        }
        min_seen = min_seen.min(px);
    }
    if !min_seen.is_finite() {
        return (Status::Pass, "no px-sized text".to_string());
    }
    if failing.is_empty() {
        return (
            Status::Pass,
            format!(
                "smallest {min_seen:.2}px x scale {scale:.3} = {:.2}px displayed",
                min_seen * scale
            ),
        );
    }
    failing.sort_by(f32::total_cmp);
    let mut sizes: Vec<String> = Vec::new();
    for group in failing.chunk_by(|a, b| a == b) {
        let px = group[0];
        let mut entry = format!("{px:.2}px -> {:.2}px", px * scale);
        if group.len() > 1 {
            entry.push_str(&format!(" (x{})", group.len()));
        }
        sizes.push(entry);
    }
    (
        Status::Fail,
        format!(
            "text below {MIN_DISPLAYED_FONT_PX}px displayed at scale {scale:.3}: {}",
            sizes.join(", ")
        ),
    )
}

/// Ratio of displayed px to user units for the root `<svg>`.
fn display_scale(root: roxmltree::Node) -> f32 {
    let width = root.attribute("width").and_then(dimensions::parse_px);
    let displayed = width.map_or(COLUMN_WIDTH_PX, |w| w.min(COLUMN_WIDTH_PX));
    let user_width = root
        .attribute("viewBox")
        .and_then(dimensions::parse_viewbox)
        .map(|(w, _)| w)
        .or(width)
        .unwrap_or(COLUMN_WIDTH_PX);
    displayed / user_width
}

/// Px sizes of every `<text>` and `<tspan>` that has one.
fn text_sizes<'a>(doc: &'a roxmltree::Document) -> impl Iterator<Item = f32> + 'a {
    doc.descendants()
        .filter(|n| n.is_element() && matches!(n.tag_name().name(), "text" | "tspan"))
        .filter_map(|n| font_size_of(&n).as_deref().and_then(parse_px))
}

pub fn check(svg: &Svg) -> CheckResult {
    let doc = match roxmltree::Document::parse(&svg.raw) {
        Ok(d) => d,
        Err(_) => {
            return CheckResult {
                id: "min-font-size",
                layer: 1,
                status: Status::Pass,
                detail: "unparseable; skipped".to_string(),
            };
        }
    };
    let mut tiny = 0usize;
    let mut min_seen = f32::INFINITY;
    for node in doc.descendants() {
        if !node.is_element() {
            continue;
        }
        let name = node.tag_name().name();
        if name != "text" && name != "tspan" {
            continue;
        }
        if let Some(px) = font_size_of(&node).as_deref().and_then(parse_px) {
            if px < MIN_FONT_SIZE {
                tiny += 1;
            }
            if px < min_seen {
                min_seen = px;
            }
        }
    }
    let status = if tiny == 0 {
        Status::Pass
    } else {
        Status::Fail
    };
    let detail = if min_seen.is_finite() {
        format!("{tiny} text element(s) below {MIN_FONT_SIZE}px; smallest {min_seen:.2}px")
    } else {
        "no px-sized text".to_string()
    };
    CheckResult {
        id: "min-font-size",
        layer: 1,
        status,
        detail,
    }
}

/// Read a `font-size` from the `font-size` attribute or an inline `style`.
fn font_size_of(node: &roxmltree::Node) -> Option<String> {
    if let Some(a) = node.attribute("font-size") {
        return Some(a.to_string());
    }
    let style = node.attribute("style")?;
    for decl in style.split(';') {
        let mut kv = decl.splitn(2, ':');
        let k = kv.next()?.trim();
        if k.eq_ignore_ascii_case("font-size") {
            return kv.next().map(|v| v.trim().to_string());
        }
    }
    None
}

/// Resolve a `font-size` value to user-unit pixels, accepting only unitless or
/// `px` values. Any other unit (`em`, `rem`, `%`, `pt`, `ex`, …) or non-numeric
/// value returns `None` — such text is skipped, never flagged.
fn parse_px(value: &str) -> Option<f32> {
    let v = value.trim().to_ascii_lowercase();
    let num = v.strip_suffix("px").unwrap_or(&v).trim();
    num.parse::<f32>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    #[test]
    fn parse_px_accepts_unitless_and_px() {
        assert_eq!(parse_px("6"), Some(6.0));
        assert_eq!(parse_px("14"), Some(14.0));
        assert_eq!(parse_px("12.5"), Some(12.5));
        assert_eq!(parse_px("8px"), Some(8.0));
        assert_eq!(parse_px(" 10px "), Some(10.0));
    }

    #[test]
    fn parse_px_skips_other_units_and_garbage() {
        assert_eq!(parse_px("8em"), None);
        assert_eq!(parse_px("6pt"), None);
        assert_eq!(parse_px("120%"), None);
        assert_eq!(parse_px("2ex"), None);
        assert_eq!(parse_px("abc"), None);
        assert_eq!(parse_px(""), None);
    }

    #[test]
    fn tiny_font_fails() {
        let svg = load(Path::new("tests/fixtures/tiny-font.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "min-font-size");
    }

    #[test]
    fn font_ok_passes() {
        let svg = load(Path::new("tests/fixtures/font-ok.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }

    fn md(root_attrs: &str, font: &str) -> (Status, String) {
        let raw = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" {root_attrs}><text font-size="{font}">a</text></svg>"#
        );
        evaluate_markdown(&roxmltree::Document::parse(&raw).unwrap())
    }

    #[test]
    fn markdown_scale_table() {
        // (root attributes, font, expected status)
        let cases = [
            // viewBox 1600 shown at 800: scale 0.5.
            (
                r#"viewBox="0 0 1600 400" width="800" height="200""#,
                "22",
                Status::Pass,
            ),
            (
                r#"viewBox="0 0 1600 400" width="800" height="200""#,
                "21",
                Status::Fail,
            ),
            // No width: fills the 800 px column.
            (r#"viewBox="0 0 1600 400""#, "22", Status::Pass),
            (r#"viewBox="0 0 1600 400""#, "21", Status::Fail),
            // Width wider than the column is capped at 800.
            (
                r#"viewBox="0 0 1600 400" width="1600" height="400""#,
                "21",
                Status::Fail,
            ),
            // Width smaller than the viewBox shrinks text.
            (
                r#"viewBox="0 0 400 200" width="200" height="100""#,
                "21",
                Status::Fail,
            ),
            (
                r#"viewBox="0 0 400 200" width="200" height="100""#,
                "22",
                Status::Pass,
            ),
            // Scale 1: the 11 px floor, not the 8 px html floor.
            (
                r#"viewBox="0 0 400 200" width="400" height="200""#,
                "11",
                Status::Pass,
            ),
            (
                r#"viewBox="0 0 400 200" width="400" height="200""#,
                "10.9",
                Status::Fail,
            ),
            (
                r#"viewBox="0 0 400 200" width="400" height="200""#,
                "9px",
                Status::Fail,
            ),
            // Narrow viewBox, no width: upscaled to the column.
            (r#"viewBox="0 0 400 200""#, "6", Status::Pass),
            // No viewBox: user units are width units.
            (r#"width="400" height="200""#, "10", Status::Fail),
            (r#"width="1600" height="400""#, "21", Status::Fail),
            // Width not a positive px number: fills the column.
            (r#"viewBox="0 0 1600 400" width="100%""#, "22", Status::Pass),
            (r#"viewBox="0 0 1600 400" width="100%""#, "21", Status::Fail),
            (r#"viewBox="0 0 1600 400" width="0""#, "22", Status::Pass),
            (r#"viewBox="0 0 1600 400" width="0""#, "21", Status::Fail),
            // No width and no viewBox: scale 1.
            ("", "11", Status::Pass),
            ("", "10", Status::Fail),
            // Comma-separated viewBox.
            (r#"viewBox="0,0,1600,400" width="800""#, "21", Status::Fail),
            // Displayed at exactly 11 px despite f32 rounding.
            (
                r#"viewBox="0 0 1525 400" width="800""#,
                "20.96875",
                Status::Pass,
            ),
            // Non-px font sizes are skipped.
            (r#"viewBox="0 0 400 200""#, "1em", Status::Pass),
        ];
        for (attrs, font, want) in cases {
            assert_eq!(md(attrs, font).0, want, "{attrs} font {font}");
        }
    }

    #[test]
    fn markdown_failure_reports_authored_scale_and_displayed() {
        let (status, detail) = md(r#"viewBox="0 0 1600 400" width="800" height="200""#, "14");
        assert_eq!(status, Status::Fail);
        assert_eq!(
            detail,
            "text below 11px displayed at scale 0.500: 14.00px -> 7.00px"
        );
    }

    fn md_doc(body: &str) -> (Status, String) {
        let raw = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 400" width="800" height="200">{body}</svg>"#
        );
        evaluate_markdown(&roxmltree::Document::parse(&raw).unwrap())
    }

    #[test]
    fn markdown_reports_every_failing_text() {
        let (status, detail) = md_doc(
            r#"<text font-size="30">ok</text><text font-size="16">a</text>
               <text font-size="14">b</text><text font-size="16">c</text>"#,
        );
        assert_eq!(status, Status::Fail);
        assert_eq!(
            detail,
            "text below 11px displayed at scale 0.500: 14.00px -> 7.00px, 16.00px -> 8.00px (x2)"
        );
    }

    #[test]
    fn markdown_checks_tspan_and_inline_style() {
        let (_, detail) = md_doc(r#"<text font-size="30"><tspan font-size="14">a</tspan></text>"#);
        assert!(detail.ends_with(": 14.00px -> 7.00px"), "{detail}");
        let (_, detail) = md_doc(r#"<text style="fill:#000; font-size: 12px">a</text>"#);
        assert!(detail.ends_with(": 12.00px -> 6.00px"), "{detail}");
    }

    #[test]
    fn markdown_pass_reports_smallest_displayed() {
        assert_eq!(
            md_doc(r#"<text font-size="40">a</text><text font-size="24">b</text>"#),
            (
                Status::Pass,
                "smallest 24.00px x scale 0.500 = 12.00px displayed".to_string()
            )
        );
    }

    #[test]
    fn markdown_unparseable_is_skipped() {
        let svg = load(Path::new("tests/fixtures/md-valid.svg")).unwrap();
        let svg = Svg {
            raw: "<svg".to_string(),
            ..svg
        };
        let r = check_markdown(&svg);
        assert_eq!(
            (r.status, r.detail.as_str()),
            (Status::Pass, "unparseable; skipped")
        );
    }

    #[test]
    fn markdown_no_text_passes() {
        assert_eq!(
            md(r#"viewBox="0 0 400 200""#, "x"),
            (Status::Pass, "no px-sized text".to_string())
        );
    }

    #[test]
    fn d2_clean_passes_via_skip_path() {
        let svg = load(Path::new("tests/fixtures/d2-clean.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }
}

use crate::checks::{CheckResult, Status};
use crate::svg::Svg;

pub const MIN_FONT_SIZE: f32 = 8.0;

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

    #[test]
    fn d2_clean_passes_via_skip_path() {
        let svg = load(Path::new("tests/fixtures/d2-clean.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }
}

use crate::checks::{CheckResult, Status};
use crate::svg::Svg;

const GENERICS: &[&str] = &[
    "serif",
    "sans-serif",
    "monospace",
    "cursive",
    "fantasy",
    "system-ui",
    "ui-serif",
    "ui-sans-serif",
    "ui-monospace",
    "ui-rounded",
];

pub fn check(svg: &Svg) -> CheckResult {
    let doc = match roxmltree::Document::parse(&svg.raw) {
        Ok(d) => d,
        Err(_) => {
            return CheckResult {
                id: "font-family",
                layer: 1,
                status: Status::Pass,
                detail: "unparseable; skipped".to_string(),
            };
        }
    };
    let mut bad = 0usize;
    for node in doc.descendants() {
        if !node.is_element() {
            continue;
        }
        let name = node.tag_name().name();
        if name != "text" && name != "tspan" {
            continue;
        }
        if let Some(ff) = font_family_of(&node) {
            if is_single_nongeneric(&ff) {
                bad += 1;
            }
        }
    }
    let status = if bad == 0 { Status::Pass } else { Status::Fail };
    CheckResult {
        id: "font-family",
        layer: 1,
        status,
        detail: format!("{bad} text element(s) with a single non-generic font-family"),
    }
}

fn font_family_of(node: &roxmltree::Node) -> Option<String> {
    if let Some(a) = node.attribute("font-family") {
        return Some(a.to_string());
    }
    let style = node.attribute("style")?;
    for decl in style.split(';') {
        let mut kv = decl.splitn(2, ':');
        let k = kv.next()?.trim();
        if k.eq_ignore_ascii_case("font-family") {
            return kv.next().map(|v| v.trim().to_string());
        }
    }
    None
}

fn is_single_nongeneric(ff: &str) -> bool {
    if ff.contains(',') {
        return false;
    }
    let f = ff
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .to_ascii_lowercase();
    if f.is_empty() {
        return false;
    }
    !GENERICS.contains(&f.as_str())
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
    fn single_family_fails() {
        let svg = load(Path::new("tests/fixtures/font-single.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "font-family");
    }

    #[test]
    fn fallback_chain_passes() {
        let svg = load(Path::new("tests/fixtures/font-fallback.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }
}

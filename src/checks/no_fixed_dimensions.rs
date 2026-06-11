use crate::checks::{CheckResult, Status};
use crate::svg::Svg;

pub fn check(svg: &Svg) -> CheckResult {
    let (status, detail) = match roxmltree::Document::parse(&svg.raw) {
        Ok(doc) => {
            let root = doc.root_element();
            if root.attribute("width").is_some() || root.attribute("height").is_some() {
                (
                    Status::Fail,
                    "root <svg> has a fixed width/height (use viewBox only)".to_string(),
                )
            } else {
                (
                    Status::Pass,
                    "no fixed width/height on root <svg>".to_string(),
                )
            }
        }
        Err(_) => (Status::Pass, "unparseable; skipped".to_string()),
    };
    CheckResult {
        id: "no-fixed-dimensions",
        layer: 0,
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
        // D2's root <svg> carries only viewBox; width/height sit on a nested inner svg.
        let svg = load(Path::new("tests/fixtures/d2-clean.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }

    #[test]
    fn in_band_passes() {
        let svg = load(Path::new("tests/fixtures/in-band.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }

    #[test]
    fn fixed_dims_fails() {
        let svg = load(Path::new("tests/fixtures/fixed-dims.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "no-fixed-dimensions");
    }
}

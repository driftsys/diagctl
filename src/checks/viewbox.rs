use crate::checks::{CheckResult, Status};
use crate::svg::Svg;

pub fn check(svg: &Svg) -> CheckResult {
    let present = roxmltree::Document::parse(&svg.raw)
        .map(|doc| doc.root_element().attribute("viewBox").is_some())
        .unwrap_or(false);
    let (status, detail) = if present {
        (Status::Pass, "root <svg> has a viewBox".to_string())
    } else {
        (Status::Fail, "root <svg> is missing a viewBox".to_string())
    };
    CheckResult {
        id: "viewbox-present",
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
    fn present_passes() {
        let svg = load(Path::new("tests/fixtures/in-band.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }

    #[test]
    fn absent_fails() {
        let svg = load(Path::new("tests/fixtures/no-viewbox.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "viewbox-present");
    }
}

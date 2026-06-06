use crate::checks::{CheckResult, Status};
use crate::svg::Svg;

pub const ASPECT_LIMIT: f32 = 2.5;

pub fn check(svg: &Svg) -> CheckResult {
    let size = svg.tree.size();
    let (w, h) = (size.width(), size.height());
    let (long, short) = if w >= h { (w, h) } else { (h, w) };
    let ratio = long / short;
    let status = if ratio > ASPECT_LIMIT {
        Status::Fail
    } else {
        Status::Pass
    };
    CheckResult {
        id: "aspect-ratio",
        layer: 2,
        status,
        detail: format!("viewBox long/short ratio {ratio:.2} (limit {ASPECT_LIMIT})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    #[test]
    fn in_band_passes() {
        let svg = load(Path::new("tests/fixtures/in-band.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }

    #[test]
    fn strip_fails() {
        let svg = load(Path::new("tests/fixtures/strip.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "aspect-ratio");
    }
}

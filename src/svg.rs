use std::path::Path;

pub struct Svg {
    pub raw: String,
    pub tree: usvg::Tree,
}

pub fn load(path: &Path) -> anyhow::Result<Svg> {
    let raw = std::fs::read_to_string(path)?;
    let tree = usvg::Tree::from_str(&raw, &usvg::Options::default())?;
    Ok(Svg { raw, tree })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_reads_resolved_size_from_fixture() {
        let svg = load(Path::new("tests/fixtures/in-band.svg")).unwrap();
        let size = svg.tree.size();
        assert_eq!(size.width() as i32, 400);
        assert_eq!(size.height() as i32, 300);
        assert!(svg.raw.contains("viewBox"));
    }
}

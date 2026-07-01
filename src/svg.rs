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

/// DejaVu Sans (Bitstream Vera / DejaVu license) — a single deterministic metrics font
/// embedded so text shaping is identical on every platform. Attribution:
/// assets/fonts/LICENSE-DejaVu.txt.
const BUNDLED_FONT: &[u8] = include_bytes!("../assets/fonts/DejaVuSans.ttf");
const BUNDLED_FAMILY: &str = "DejaVu Sans";

/// Re-parse the SVG with a font DB containing exactly the bundled font, with every generic
/// family pointed at it and it set as the default family. Any `font-family` therefore
/// resolves to the bundled font, yielding deterministic, platform-identical text geometry.
///
/// Used only by the `label-overflow` check; the shared `load` path keeps its empty-font tree
/// so the other checks are unaffected.
pub fn shaped_tree(svg: &Svg) -> anyhow::Result<usvg::Tree> {
    let mut opt = usvg::Options::default();
    {
        let db = opt.fontdb_mut();
        db.load_font_data(BUNDLED_FONT.to_vec());
        db.set_serif_family(BUNDLED_FAMILY);
        db.set_sans_serif_family(BUNDLED_FAMILY);
        db.set_monospace_family(BUNDLED_FAMILY);
        db.set_cursive_family(BUNDLED_FAMILY);
        db.set_fantasy_family(BUNDLED_FAMILY);
    }
    opt.font_family = BUNDLED_FAMILY.to_string();
    Ok(usvg::Tree::from_str(&svg.raw, &opt)?)
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

    #[test]
    fn shaped_tree_gives_text_nonzero_bbox() {
        let svg = load(Path::new("tests/fixtures/label-fits.svg")).unwrap();
        let tree = shaped_tree(&svg).unwrap();

        fn any_text_shaped(g: &usvg::Group) -> bool {
            g.children().iter().any(|n| match n {
                usvg::Node::Group(gg) => any_text_shaped(gg),
                usvg::Node::Text(t) => t.abs_bounding_box().width() > 0.0,
                _ => false,
            })
        }

        assert!(
            any_text_shaped(tree.root()),
            "bundled font failed to shape text to a non-zero-width bbox"
        );
    }
}

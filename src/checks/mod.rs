use crate::cli::Target;
use crate::svg::Svg;

pub mod aspect;
pub mod contrast;
pub mod dimensions;
pub mod edge_crossings;
pub mod edge_node_overlap;
pub mod font_family;
pub mod label_overflow;
pub mod min_font_size;
pub mod min_stroke_width;
pub mod no_fixed_dimensions;
pub mod node_overlap;
pub mod opaque_background;
pub mod viewbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    Fail,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub id: &'static str,
    pub layer: u8,
    pub status: Status,
    pub detail: String,
}

type CheckFn = fn(&Svg) -> CheckResult;

/// Runs every check for `target`. The targets differ in three checks:
/// `html` forbids a fixed width/height and floors the authored font at
/// 8 px; `markdown` requires a width/height, floors the displayed font at
/// 11 px and requires an opaque white background.
pub fn run_all(svg: &Svg, target: Target) -> Vec<CheckResult> {
    let (size_check, font_check): (CheckFn, CheckFn) = match target {
        Target::Html => (no_fixed_dimensions::check, min_font_size::check),
        Target::Markdown => (dimensions::check, min_font_size::check_markdown),
    };
    let mut results = vec![
        viewbox::check(svg),
        size_check(svg),
        aspect::check(svg),
        node_overlap::check(svg),
        edge_crossings::check(svg),
        edge_node_overlap::check(svg),
        label_overflow::check(svg),
        contrast::check(svg),
        min_stroke_width::check(svg),
        font_family::check(svg),
        font_check(svg),
    ];
    if target == Target::Markdown {
        results.push(opaque_background::check(svg));
    }
    results
}

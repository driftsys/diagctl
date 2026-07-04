use crate::svg::Svg;

pub mod aspect;
pub mod contrast;
pub mod edge_crossings;
pub mod edge_node_overlap;
pub mod font_family;
pub mod label_overflow;
pub mod min_font_size;
pub mod min_stroke_width;
pub mod no_fixed_dimensions;
pub mod node_overlap;
pub mod viewbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub fn run_all(svg: &Svg) -> Vec<CheckResult> {
    vec![
        viewbox::check(svg),
        no_fixed_dimensions::check(svg),
        aspect::check(svg),
        node_overlap::check(svg),
        edge_crossings::check(svg),
        edge_node_overlap::check(svg),
        label_overflow::check(svg),
        contrast::check(svg),
        min_stroke_width::check(svg),
        font_family::check(svg),
        min_font_size::check(svg),
    ]
}

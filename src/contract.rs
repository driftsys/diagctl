//! Display contract for `--target markdown`.
//!
//! pixctl has its own copy of this contract. The two repositories are
//! independent: when a value changes here, increase `VERSION` and update
//! pixctl by hand.

// The markdown checks (#19, #20, #21) will read these values. Until then
// only `VERSION` is used.
#![allow(dead_code)]

/// Contract version. Shown by `diagctl --version`.
pub const VERSION: u32 = 1;

/// Width of the Markdown content column, in CSS px.
pub const COLUMN_WIDTH_PX: f32 = 800.0;

/// Minimum font size after the image is scaled into the column, in CSS px.
pub const MIN_DISPLAYED_FONT_PX: f32 = 11.0;

/// Required opaque background colour.
pub const BACKGROUND: &str = "#ffffff";

/// Text appended to the crate version in `--version` output.
pub const VERSION_LINE: &str = concat!(env!("CARGO_PKG_VERSION"), " (display contract 1)");

#[cfg(test)]
mod tests {
    #[test]
    fn version_line_matches_version() {
        assert!(super::VERSION_LINE.ends_with(&format!("(display contract {})", super::VERSION)));
    }
}

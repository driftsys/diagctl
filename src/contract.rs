//! Display contract for `--target markdown`.
//!
//! pixctl defines the same display values independently. When a value
//! changes here, increase the contract version and update pixctl by hand.

/// Contract version, as a literal, so that `concat!` can use it.
macro_rules! contract_version {
    () => {
        "1"
    };
}

/// Contract version. Part of the `diagctl --version` output.
#[allow(dead_code)] // Read by the markdown checks (#19, #20, #21).
pub const CONTRACT_VERSION: &str = contract_version!();

/// Width of the Markdown content column, in CSS px.
#[allow(dead_code)] // Read by the markdown checks (#19, #20, #21).
pub const COLUMN_WIDTH_PX: f32 = 800.0;

/// Minimum font size after the image is scaled into the column, in CSS px.
#[allow(dead_code)] // Read by the markdown checks (#19, #20, #21).
pub const MIN_DISPLAYED_FONT_PX: f32 = 11.0;

/// Required opaque background colour.
#[allow(dead_code)] // Read by the markdown checks (#19, #20, #21).
pub const BACKGROUND: &str = "#ffffff";

/// Full `--version` string: the crate version followed by the contract version.
pub const VERSION_LINE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (display contract ",
    contract_version!(),
    ")"
);

#[cfg(test)]
mod tests {
    use super::*;

    // If this test fails because a value changed, also increase the
    // contract version and update the expected version below.
    #[test]
    fn contract_values_match_version_1() {
        assert_eq!(CONTRACT_VERSION, "1");
        assert_eq!(COLUMN_WIDTH_PX, 800.0);
        assert_eq!(MIN_DISPLAYED_FONT_PX, 11.0);
        assert_eq!(BACKGROUND, "#ffffff");
    }

    #[test]
    fn version_line_ends_with_contract_version() {
        assert!(VERSION_LINE.ends_with(&format!("(display contract {CONTRACT_VERSION})")));
    }
}

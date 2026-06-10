//! WCAG color math for the contrast check. Pure — no usvg.

fn linearize(c: u8) -> f32 {
    let s = c as f32 / 255.0;
    if s <= 0.03928 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG relative luminance of an sRGB color (0.0–1.0).
pub fn relative_luminance(r: u8, g: u8, b: u8) -> f32 {
    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

/// WCAG contrast ratio between two luminances (1.0–21.0).
pub fn contrast_ratio(l1: f32, l2: f32) -> f32 {
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_and_white_luminance() {
        assert!(relative_luminance(0, 0, 0).abs() < 1e-6);
        assert!((relative_luminance(255, 255, 255) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn black_on_white_is_21_to_1() {
        let ratio = contrast_ratio(
            relative_luminance(255, 255, 255),
            relative_luminance(0, 0, 0),
        );
        assert!((ratio - 21.0).abs() < 1e-4, "got {ratio}");
    }
}

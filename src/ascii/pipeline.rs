/// Render a horizontal box-and-arrow pipeline as aligned ASCII.
///
/// All boxes share one cell width (longest label + 2), so every box is
/// character-for-character identical in width and the three rendered lines have
/// equal length. Boxes are joined by a ` ---> ` connector on the label row.
pub fn render<S: AsRef<str>>(labels: &[S]) -> String {
    // 6-space gap on border rows aligns with the 6-char ` ---> ` connector on
    // the label row, so borders never collide with the arrow.
    const GAP: &str = "      ";
    const CONNECTOR: &str = " ---> ";
    let cell = labels
        .iter()
        .map(|l| l.as_ref().chars().count())
        .max()
        .unwrap_or(0)
        + 2;
    let border_box = format!("+{}+", "-".repeat(cell));
    let border_row = vec![border_box.as_str(); labels.len()].join(GAP);
    let label_row = labels
        .iter()
        .map(|l| format!("|{}|", center(l.as_ref(), cell)))
        .collect::<Vec<_>>()
        .join(CONNECTOR);
    format!("{border_row}\n{label_row}\n{border_row}")
}

/// Center `label` in `width` columns, biasing the extra space to the right.
fn center(label: &str, width: usize) -> String {
    let extra = width - label.chars().count();
    let left = extra / 2;
    let right = extra - left;
    format!("{}{}{}", " ".repeat(left), label, " ".repeat(right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_boxes_exact() {
        let out = render(&["Source", "Filter", "Sink"]);
        let expected = "\
+--------+      +--------+      +--------+
| Source | ---> | Filter | ---> |  Sink  |
+--------+      +--------+      +--------+";
        assert_eq!(out, expected);
    }

    #[test]
    fn single_box_has_no_connector() {
        let out = render(&["Only"]);
        let expected = "\
+------+
| Only |
+------+";
        assert_eq!(out, expected);
    }

    #[test]
    fn longest_label_sets_the_width() {
        let out = render(&["A", "Longlabel", "Cc"]);
        let expected = "\
+-----------+      +-----------+      +-----------+
|     A     | ---> | Longlabel | ---> |    Cc     |
+-----------+      +-----------+      +-----------+";
        assert_eq!(out, expected);
    }

    #[test]
    fn every_line_has_equal_width() {
        let out = render(&["A", "Longlabel", "Cc"]);
        let widths: Vec<usize> = out.lines().map(|l| l.chars().count()).collect();
        assert_eq!(widths.len(), 3);
        assert!(
            widths.iter().all(|&w| w == widths[0]),
            "line widths differ: {widths:?}"
        );
    }
}

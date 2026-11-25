use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

fn diagctl() -> Command {
    Command::cargo_bin("diagctl").unwrap()
}

#[test]
fn version_prints_crate_version() {
    diagctl()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn version_reports_display_contract_version() {
    diagctl()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("(display contract 1)\n"));
}

#[test]
fn help_lists_all_subcommands() {
    diagctl()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("check"))
        .stdout(contains("optimize"))
        .stdout(contains("freshness"));
}

#[test]
fn check_in_band_exits_0() {
    diagctl()
        .args(["check", "tests/fixtures/in-band.svg"])
        .assert()
        .code(0);
}

fn check_stdout(args: &[&str], code: i32) -> Vec<u8> {
    diagctl()
        .args(args)
        .assert()
        .code(code)
        .get_output()
        .stdout
        .clone()
}

#[test]
fn check_target_html_matches_default_output() {
    for (fixture, code) in [
        ("tests/fixtures/in-band.svg", 0),
        ("tests/fixtures/strip.svg", 1),
    ] {
        assert_eq!(
            check_stdout(&["check", "--target", "html", fixture], code),
            check_stdout(&["check", fixture], code),
            "--target html on {fixture}"
        );
    }
}

#[test]
fn check_markdown_fails_svg_without_dimensions() {
    diagctl()
        .args([
            "check",
            "--target",
            "markdown",
            "tests/fixtures/in-band.svg",
        ])
        .assert()
        .code(1)
        .stdout(contains("FAIL  [L0] dimensions"))
        .stdout(contains("no-fixed-dimensions").not());
}

#[test]
fn check_markdown_passes_sized_svg() {
    diagctl()
        .args([
            "check",
            "--target",
            "markdown",
            "tests/fixtures/md-valid.svg",
        ])
        .assert()
        .code(0)
        .stdout(contains("PASS  [L0] dimensions"));
}

/// Check output lines without the target-specific lines (size, font and
/// background).
fn lines_without_target_checks(out: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(out)
        .lines()
        .filter(|l| {
            !l.contains("dimensions")
                && !l.contains("min-font-size")
                && !l.contains("opaque-background")
        })
        .map(str::to_string)
        .collect()
}

#[test]
fn check_targets_differ_only_in_size_font_and_background_checks() {
    for fixture in ["tests/fixtures/md-valid.svg", "tests/fixtures/strip.svg"] {
        let html = diagctl().args(["check", fixture]).output().unwrap().stdout;
        let md = diagctl()
            .args(["check", "--target", "markdown", fixture])
            .output()
            .unwrap()
            .stdout;
        let html_rest = lines_without_target_checks(&html);
        // html has a size and a font line; markdown also has a background line.
        assert_eq!(
            html_rest.len() + 2,
            String::from_utf8_lossy(&html).lines().count()
        );
        assert_eq!(
            html_rest.len() + 3,
            String::from_utf8_lossy(&md).lines().count()
        );
        assert_eq!(lines_without_target_checks(&md), html_rest, "{fixture}");
    }
}

#[test]
fn check_markdown_fails_svg_wider_than_column() {
    diagctl()
        .args([
            "check",
            "--target",
            "markdown",
            "tests/fixtures/md-wide.svg",
        ])
        .assert()
        .code(1)
        .stdout(contains("FAIL  [L0] dimensions"));
}

#[test]
fn check_default_target_is_html() {
    diagctl()
        .args(["check", "--help"])
        .assert()
        .success()
        .stdout(contains("[default: html]"));
}

#[test]
fn check_unknown_target_exits_2() {
    diagctl()
        .args(["check", "--target", "pdf", "tests/fixtures/in-band.svg"])
        .assert()
        .code(2)
        .stderr(contains("invalid value 'pdf'"));
}

#[test]
fn check_strip_exits_1_naming_aspect() {
    diagctl()
        .args(["check", "tests/fixtures/strip.svg"])
        .assert()
        .code(1)
        .stdout(contains("aspect-ratio"));
}

#[test]
fn check_no_viewbox_exits_1_naming_layer0() {
    diagctl()
        .args(["check", "tests/fixtures/no-viewbox.svg"])
        .assert()
        .code(1)
        .stdout(contains("viewbox-present"));
}

#[test]
fn check_missing_file_exits_2() {
    diagctl()
        .args(["check", "tests/fixtures/does-not-exist.svg"])
        .assert()
        .code(2)
        .stderr(contains("diagctl: tests/fixtures/does-not-exist.svg: "));
}

#[test]
fn check_requires_a_file() {
    diagctl()
        .args(["check"])
        .assert()
        .code(2)
        .stderr(contains("<FILES>..."));
}

#[test]
fn check_one_file_prints_no_header() {
    let out = String::from_utf8(check_stdout(&["check", "tests/fixtures/in-band.svg"], 0)).unwrap();
    assert!(out.starts_with("PASS  [L0] viewbox-present"), "{out}");
}

#[test]
fn check_many_files_prints_a_header_per_file() {
    let out = diagctl()
        .args([
            "check",
            "tests/fixtures/in-band.svg",
            "tests/fixtures/strip.svg",
        ])
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let out = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "tests/fixtures/in-band.svg");
    assert_eq!(lines[12], "tests/fixtures/strip.svg");
    assert_eq!(lines.len(), 24, "{out}");
    assert!(lines[13].starts_with("PASS  [L0] viewbox-present"), "{out}");
}

#[test]
fn check_many_files_exit_1_when_only_checks_fail() {
    diagctl()
        .args([
            "check",
            "tests/fixtures/strip.svg",
            "tests/fixtures/in-band.svg",
        ])
        .assert()
        .code(1);
}

#[test]
fn check_unreadable_file_is_reported_and_others_still_checked() {
    let assert = diagctl()
        .args([
            "check",
            "tests/fixtures/does-not-exist.svg",
            "tests/fixtures/strip.svg",
        ])
        .assert()
        .code(2)
        .stderr(contains("diagctl: tests/fixtures/does-not-exist.svg: "));
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // The unreadable file gets no header; the readable one is checked.
    assert!(!out.contains("does-not-exist"), "{out}");
    assert!(out.starts_with("tests/fixtures/strip.svg\n"), "{out}");
    assert!(out.contains("FAIL  [L2] aspect-ratio"), "{out}");
}

#[test]
fn stub_optimize_exits_2() {
    diagctl()
        .args(["optimize", "tests/fixtures/in-band.svg"])
        .assert()
        .code(2)
        .stderr(contains("not yet implemented"));
}

#[test]
fn ascii_subcommand_is_absent() {
    diagctl()
        .args(["ascii", "tests/fixtures/in-band.svg"])
        .assert()
        .code(2)
        .stderr(contains("unrecognized subcommand 'ascii'"));
}

#[test]
fn stub_freshness_exits_2() {
    diagctl()
        .args(["freshness", "tests/fixtures/in-band.svg"])
        .assert()
        .code(2);
}

#[test]
fn check_d2_clean_exits_0() {
    diagctl()
        .args(["check", "tests/fixtures/d2-clean.svg"])
        .assert()
        .code(0)
        .stdout(contains("node-overlap"));
}

#[test]
fn check_overlap_exits_1_naming_node_overlap() {
    diagctl()
        .args(["check", "tests/fixtures/overlap.svg"])
        .assert()
        .code(1)
        .stdout(contains("node-overlap"));
}

#[test]
fn check_crossing_exits_1_naming_edge_crossings() {
    diagctl()
        .args(["check", "tests/fixtures/crossing.svg"])
        .assert()
        .code(1)
        .stdout(contains("edge-crossings"));
}

#[test]
fn check_cross_clean_exits_0() {
    diagctl()
        .args(["check", "tests/fixtures/cross-clean.svg"])
        .assert()
        .code(0)
        .stdout(contains("edge-crossings"));
}

#[test]
fn check_edge_through_node_exits_1() {
    diagctl()
        .args(["check", "tests/fixtures/edge-through-node.svg"])
        .assert()
        .code(1)
        .stdout(contains("edge-node-overlap"));
}

#[test]
fn check_low_contrast_exits_1() {
    diagctl()
        .args(["check", "tests/fixtures/low-contrast.svg"])
        .assert()
        .code(1)
        .stdout(contains("contrast"));
}

#[test]
fn check_thin_stroke_exits_1() {
    diagctl()
        .args(["check", "tests/fixtures/thin-stroke.svg"])
        .assert()
        .code(1)
        .stdout(contains("min-stroke-width"));
}

#[test]
fn check_font_single_exits_1() {
    diagctl()
        .args(["check", "tests/fixtures/font-single.svg"])
        .assert()
        .code(1)
        .stdout(contains("font-family"));
}

#[test]
fn check_fixed_dims_exits_1() {
    diagctl()
        .args(["check", "tests/fixtures/fixed-dims.svg"])
        .assert()
        .code(1)
        .stdout(contains("no-fixed-dimensions"));
}

#[test]
fn check_label_overflow_exits_1() {
    diagctl()
        .args(["check", "tests/fixtures/label-overflow.svg"])
        .assert()
        .code(1)
        .stdout(contains("label-overflow"));
}

#[test]
fn check_tiny_font_exits_1_naming_min_font_size() {
    diagctl()
        .args(["check", "tests/fixtures/tiny-font.svg"])
        .assert()
        .code(1)
        .stdout(contains("min-font-size"));
}

#[test]
fn check_markdown_fails_text_too_small_when_displayed() {
    // 14px text in a 1600-wide viewBox shown at 800 px displays at 7px.
    diagctl()
        .args([
            "check",
            "--target",
            "markdown",
            "tests/fixtures/md-small-font.svg",
        ])
        .assert()
        .code(1)
        .stdout(contains(
            "FAIL  [L1] min-font-size  text below 11px displayed at scale 0.500: 14.00px -> 7.00px",
        ));
}

#[test]
fn check_html_passes_same_font_at_authored_size() {
    diagctl()
        .args(["check", "tests/fixtures/md-small-font.svg"])
        .assert()
        .stdout(contains("PASS  [L1] min-font-size"));
}

#[test]
fn check_markdown_fails_transparent_background() {
    diagctl()
        .args([
            "check",
            "--target",
            "markdown",
            "tests/fixtures/bg-drawio-transparent.svg",
        ])
        .assert()
        .code(1)
        .stdout(contains("FAIL  [L1] opaque-background"));
}

#[test]
fn check_markdown_passes_white_background() {
    diagctl()
        .args([
            "check",
            "--target",
            "markdown",
            "tests/fixtures/md-valid.svg",
        ])
        .assert()
        .stdout(contains(
            "PASS  [L1] opaque-background  opaque #ffffff background",
        ));
}

#[test]
fn check_html_skips_background_check() {
    diagctl()
        .args(["check", "tests/fixtures/bg-drawio-transparent.svg"])
        .assert()
        .stdout(contains("opaque-background").not());
}

#[test]
fn check_markdown_passes_root_css_background() {
    // PlantUML sets its white background in the root `style` attribute.
    diagctl()
        .args([
            "check",
            "--target",
            "markdown",
            "tests/fixtures/bg-plantuml.svg",
        ])
        .assert()
        .stdout(contains(
            "PASS  [L1] opaque-background  opaque #ffffff root CSS background",
        ));
}

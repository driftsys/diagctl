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
        .stderr(contains("diagctl: tests/fixtures/does-not-exist.svg: "))
        .stderr(contains("(os error 2)"));
}

#[test]
fn check_requires_a_file() {
    diagctl()
        .args(["check"])
        .assert()
        .code(2)
        .stderr(contains("required"))
        .stderr(contains("<FILE>..."));
}

/// Splits check output into (header, result lines) blocks.
fn blocks(out: &str) -> Vec<(String, Vec<String>)> {
    let mut blocks: Vec<(String, Vec<String>)> = Vec::new();
    for line in out.lines() {
        if line.starts_with("PASS") || line.starts_with("FAIL") {
            blocks.last_mut().unwrap().1.push(line.to_string());
        } else {
            blocks.push((line.to_string(), Vec::new()));
        }
    }
    blocks
}

#[test]
fn check_one_file_prints_no_header() {
    let out = String::from_utf8(check_stdout(&["check", "tests/fixtures/in-band.svg"], 0)).unwrap();
    assert!(out.starts_with("PASS  [L0] viewbox-present"), "{out}");
}

#[test]
fn check_many_files_prints_a_header_per_file() {
    let out = String::from_utf8(check_stdout(
        &[
            "check",
            "tests/fixtures/in-band.svg",
            "tests/fixtures/strip.svg",
        ],
        1,
    ))
    .unwrap();
    let b = blocks(&out);
    assert_eq!(b.len(), 2, "{out}");
    assert_eq!(b[0].0, "tests/fixtures/in-band.svg");
    assert_eq!(b[1].0, "tests/fixtures/strip.svg");
    // Each block holds that file's own results.
    assert!(b[0].1.iter().all(|l| l.starts_with("PASS")), "{out}");
    assert!(
        b[1].1
            .iter()
            .any(|l| l.starts_with("FAIL  [L2] aspect-ratio")),
        "{out}"
    );
    assert_eq!(b[0].1.len(), b[1].1.len(), "{out}");
}

#[test]
fn check_many_files_applies_target_to_every_file() {
    let out = String::from_utf8(check_stdout(
        &[
            "check",
            "--target",
            "markdown",
            "tests/fixtures/md-valid.svg",
            "tests/fixtures/bg-drawio-transparent.svg",
        ],
        1,
    ))
    .unwrap();
    let b = blocks(&out);
    assert_eq!(b.len(), 2, "{out}");
    for (_, lines) in &b {
        assert!(lines.iter().any(|l| l.contains("] dimensions")), "{out}");
        assert!(
            lines.iter().any(|l| l.contains("opaque-background")),
            "{out}"
        );
        assert!(
            !lines.iter().any(|l| l.contains("no-fixed-dimensions")),
            "{out}"
        );
    }
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
            "tests/fixtures/malformed.svg",
        ])
        .assert()
        .code(2)
        .stderr(contains("diagctl: tests/fixtures/does-not-exist.svg: "))
        .stderr(contains("(os error 2)"))
        .stderr(contains("diagctl: tests/fixtures/malformed.svg: "));
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // Files that fail to load get no header; the readable one is checked.
    let b = blocks(&out);
    assert_eq!(b.len(), 1, "{out}");
    assert_eq!(b[0].0, "tests/fixtures/strip.svg");
    assert!(
        b[0].1
            .iter()
            .any(|l| l.starts_with("FAIL  [L2] aspect-ratio")),
        "{out}"
    );
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

/// Runs diagctl with `args`, asserts the exit code, and parses stdout as one
/// JSON document.
fn envelope(args: &[&str], code: i32) -> serde_json::Value {
    let out = check_stdout(args, code);
    serde_json::from_slice(&out).expect("stdout is one JSON document")
}

#[test]
fn check_json_is_one_pretty_object_with_schema_first() {
    let out = String::from_utf8(check_stdout(
        &["check", "--json", "tests/fixtures/in-band.svg"],
        0,
    ))
    .unwrap();
    assert!(
        out.starts_with("{\n  \"schema\": \"diagctl/check/v1\",\n"),
        "{out}"
    );
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(keys, ["schema", "results", "summary", "errors"]);
}

#[test]
fn check_json_clean_file() {
    let v = envelope(&["check", "--json", "tests/fixtures/in-band.svg"], 0);
    let results = v["results"].as_array().unwrap();
    assert_eq!(results.len(), 11, "{v}");
    for r in results {
        assert_eq!(r["file"], "tests/fixtures/in-band.svg");
        assert_eq!(r["status"], "pass");
    }
    assert_eq!(results[0]["check"], "viewbox-present");
    assert_eq!(results[0]["layer"], 0);
    assert_eq!(results[0]["detail"], "root <svg> has a viewBox");
    assert_eq!(
        v["summary"],
        serde_json::json!({"checks": 11, "pass": 11, "fail": 0})
    );
    assert_eq!(v["errors"], serde_json::json!([]));
}

#[test]
fn check_json_failing_file() {
    let v = envelope(&["check", "--json", "tests/fixtures/strip.svg"], 1);
    let results = v["results"].as_array().unwrap();
    let aspect = results
        .iter()
        .find(|r| r["check"] == "aspect-ratio")
        .unwrap();
    assert_eq!(aspect["status"], "fail");
    assert_eq!(aspect["layer"], 2);
    let failed = results.iter().filter(|r| r["status"] == "fail").count();
    assert!(failed >= 1);
    assert_eq!(v["summary"]["checks"], results.len());
    assert_eq!(v["summary"]["fail"], failed);
    assert_eq!(v["summary"]["pass"], results.len() - failed);
}

#[test]
fn check_json_applies_target() {
    let v = envelope(
        &[
            "check",
            "--json",
            "--target",
            "markdown",
            "tests/fixtures/md-valid.svg",
        ],
        0,
    );
    let checks: Vec<&str> = v["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["check"].as_str().unwrap())
        .collect();
    assert_eq!(checks.len(), 12, "{v}");
    assert!(checks.contains(&"opaque-background"), "{v}");
}

#[test]
fn check_json_reports_load_errors_with_stable_codes() {
    let assert = diagctl()
        .args([
            "check",
            "--json",
            "tests/fixtures/does-not-exist.svg",
            "tests/fixtures/in-band.svg",
            "tests/fixtures/malformed.svg",
        ])
        .assert()
        .code(2)
        .stderr(contains("diagctl: tests/fixtures/does-not-exist.svg: "))
        .stderr(contains("diagctl: tests/fixtures/malformed.svg: "));
    let v: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
    let errors = v["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 2, "{v}");
    assert_eq!(errors[0]["code"], "read_failed");
    assert_eq!(errors[0]["file"], "tests/fixtures/does-not-exist.svg");
    let message = errors[0]["message"].as_str().unwrap();
    assert!(
        message.starts_with("tests/fixtures/does-not-exist.svg: "),
        "{v}"
    );
    assert!(message.contains("(os error 2)"), "{v}");
    assert_eq!(errors[1]["code"], "parse_failed");
    assert_eq!(errors[1]["file"], "tests/fixtures/malformed.svg");
    // The readable file is still checked, and only its results are listed.
    let results = v["results"].as_array().unwrap();
    assert_eq!(results.len(), 11, "{v}");
    assert!(results
        .iter()
        .all(|r| r["file"] == "tests/fixtures/in-band.svg"));
    assert_eq!(v["summary"]["checks"], 11);
}

#[test]
fn check_json_lists_results_of_every_file() {
    let v = envelope(
        &[
            "check",
            "--json",
            "tests/fixtures/in-band.svg",
            "tests/fixtures/strip.svg",
        ],
        1,
    );
    let results = v["results"].as_array().unwrap();
    assert_eq!(results.len(), 22, "{v}");
    assert!(results[..11]
        .iter()
        .all(|r| r["file"] == "tests/fixtures/in-band.svg"));
    assert!(results[11..]
        .iter()
        .all(|r| r["file"] == "tests/fixtures/strip.svg"));
    assert_eq!(v["summary"]["checks"], 22);
}

#[test]
fn stubs_json_report_not_implemented() {
    for command in ["optimize", "freshness"] {
        let assert = diagctl()
            .args([command, "--json", "tests/fixtures/in-band.svg"])
            .assert()
            .code(2)
            .stderr(contains(format!("diagctl: {command}: not yet implemented")));
        let v: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
        assert_eq!(v["schema"], format!("diagctl/{command}/v1"));
        assert_eq!(v["results"], serde_json::json!([]));
        assert_eq!(v["summary"], serde_json::json!({}));
        let errors = v["errors"].as_array().unwrap();
        assert_eq!(errors.len(), 1, "{v}");
        assert_eq!(errors[0]["code"], "not_implemented");
        assert!(errors[0].get("file").is_none(), "{v}");
    }
}

#[test]
fn stubs_text_mode_print_nothing_on_stdout() {
    for command in ["optimize", "freshness"] {
        diagctl()
            .args([command, "tests/fixtures/in-band.svg"])
            .assert()
            .code(2)
            .stdout("")
            .stderr(contains(format!("diagctl: {command}: not yet implemented")));
    }
}

/// The check ids of text output, in order (header lines skipped).
fn text_check_ids(out: &str) -> Vec<String> {
    out.lines()
        .filter(|l| l.starts_with("PASS") || l.starts_with("FAIL"))
        .map(|l| l.split_whitespace().nth(2).unwrap().to_string())
        .collect()
}

#[test]
fn check_json_results_follow_text_order() {
    for args in [
        vec!["tests/fixtures/in-band.svg"],
        vec!["--target", "markdown", "tests/fixtures/md-valid.svg"],
        vec![
            "tests/fixtures/does-not-exist.svg",
            "tests/fixtures/strip.svg",
            "tests/fixtures/md-valid.svg",
        ],
    ] {
        let text = diagctl().arg("check").args(&args).output().unwrap();
        let json = diagctl()
            .args(["check", "--json"])
            .args(&args)
            .output()
            .unwrap();
        assert_eq!(text.status.code(), json.status.code(), "{args:?}");
        let v: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
        let json_ids: Vec<String> = v["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["check"].as_str().unwrap().to_string())
            .collect();
        let text_ids = text_check_ids(&String::from_utf8(text.stdout).unwrap());
        assert!(!text_ids.is_empty(), "{args:?}");
        assert_eq!(json_ids, text_ids, "{args:?}");
    }
}

fn keys(v: &serde_json::Value) -> Vec<&str> {
    v.as_object().unwrap().keys().map(String::as_str).collect()
}

#[test]
fn check_json_inner_key_order_and_trailing_newline() {
    let out = diagctl()
        .args([
            "check",
            "--json",
            "tests/fixtures/in-band.svg",
            "tests/fixtures/does-not-exist.svg",
        ])
        .output()
        .unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.ends_with("}\n"), "{text}");
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        keys(&v["results"][0]),
        ["file", "check", "layer", "status", "detail"]
    );
    assert_eq!(keys(&v["summary"]), ["checks", "pass", "fail"]);
    assert_eq!(keys(&v["errors"][0]), ["code", "message", "file"]);
}

#[test]
fn check_json_every_file_fails_to_load() {
    let v = envelope(
        &[
            "check",
            "--json",
            "tests/fixtures/does-not-exist.svg",
            "tests/fixtures/malformed.svg",
        ],
        2,
    );
    assert_eq!(v["results"], serde_json::json!([]));
    assert_eq!(
        v["summary"],
        serde_json::json!({"checks": 0, "pass": 0, "fail": 0})
    );
    assert_eq!(v["errors"].as_array().unwrap().len(), 2, "{v}");
}

#[test]
fn check_json_non_utf8_file_is_read_failed() {
    let v = envelope(&["check", "--json", "tests/fixtures/not-utf8.svg"], 2);
    assert_eq!(v["errors"][0]["code"], "read_failed", "{v}");
}

#[test]
fn argument_errors_print_no_json() {
    for args in [
        vec!["check", "--json"],
        vec![
            "check",
            "--json",
            "--target",
            "pdf",
            "tests/fixtures/in-band.svg",
        ],
    ] {
        diagctl().args(&args).assert().code(2).stdout("");
    }
}

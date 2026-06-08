use assert_cmd::Command;
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
fn help_lists_all_subcommands() {
    diagctl()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("check"))
        .stdout(contains("optimize"))
        .stdout(contains("ascii"))
        .stdout(contains("freshness"));
}

#[test]
fn check_in_band_exits_0() {
    diagctl()
        .args(["check", "tests/fixtures/in-band.svg"])
        .assert()
        .code(0);
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
        .code(2);
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
fn stub_ascii_exits_2() {
    diagctl()
        .args(["ascii", "tests/fixtures/in-band.svg"])
        .assert()
        .code(2);
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

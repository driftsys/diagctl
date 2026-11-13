# diagctl Walking Skeleton Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Scaffold `diagctl` (Rust) into a walking skeleton: working
`--help`/`--version`, one real `check` vertical slice (Layer-0 viewBox-present +
Layer-2 aspect-ratio) exercising usvg, stubbed `optimize`/`ascii`/`freshness`,
cross-platform CI + release.

**Architecture:** Single binary crate. `cli.rs` (clap) parses subcommands;
`main.rs` dispatches to `commands::*` returning an exit code. `commands::check`
loads the SVG via `svg::load` (usvg `Tree` + raw text) and runs an ordered
registry of checks (`checks::run_all`). Each check returns a `CheckResult`. Exit
`0` clean / `1` any check failed / `2` operational error or stub.

**Tech Stack:** Rust 1.95, `clap` (derive), `usvg` (SVG parse + resolved
geometry), `roxmltree` (raw root-attribute read), `anyhow`; dev: `assert_cmd`,
`predicates`. CI: GitHub Actions matrix; release: cargo-dist.

---

## File Structure

- `Cargo.toml` — crate metadata + deps.
- `.gitignore` — `/target`, `/wip/superpowers/` (private working memory).
- `src/main.rs` — thin: parse CLI, dispatch, map `Result<i32>` to process exit
  code. Declares modules.
- `src/cli.rs` — clap `Cli` + `Command` enum (check/optimize/ascii/freshness).
- `src/svg.rs` — `Svg { raw, tree }` + `load(path)`; the foundational usvg parse
  path.
- `src/checks/mod.rs` — `Status`, `CheckResult`, `run_all(&Svg)` registry.
- `src/checks/aspect.rs` — Layer-2 aspect-ratio check (threshold 2.5).
- `src/checks/viewbox.rs` — Layer-0 viewBox-present check.
- `src/commands/mod.rs` — re-exports the command modules.
- `src/commands/check.rs` — run registry, print results, return exit code.
- `src/commands/optimize.rs`, `ascii.rs`, `freshness.rs` — stubs (exit 2).
- `tests/fixtures/{in-band,strip,no-viewbox}.svg` — committed real SVG fixtures.
- `tests/cli.rs` — integration tests driving the compiled binary (assert_cmd).
- `.github/workflows/ci.yml` — fmt/clippy/test matrix.
- `.github/workflows/release.yml` — cross-platform release binaries.

### Type/interface contract (used across tasks — keep names stable)

```rust
// src/checks/mod.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status { Pass, Fail }

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub id: &'static str,   // "viewbox-present", "aspect-ratio"
    pub layer: u8,          // 0 or 2
    pub status: Status,
    pub detail: String,
}

pub fn run_all(svg: &crate::svg::Svg) -> Vec<CheckResult>;
```

```rust
// src/svg.rs
pub struct Svg { pub raw: String, pub tree: usvg::Tree }
pub fn load(path: &std::path::Path) -> anyhow::Result<Svg>;
```

Each command exposes `pub fn run(...) -> anyhow::Result<i32>` returning the exit
code.

> **usvg API note:** code below targets the stable
> `usvg::Tree::from_str(&str, &usvg::Options)` + `tree.size().width()/height()`
> surface. If the version cargo resolves has a slightly different signature
> (e.g. a separate `fontdb` argument), the compiler will flag it — adjust to the
> resolved version and continue. This is expected TDD drift, not a plan error.

---

## Task 1: Project init

**Files:**

- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `.gitignore`

- [ ] **Step 1: Write `Cargo.toml`**

```toml
[package]
name = "diagctl"
version = "0.1.0"
edition = "2021"
description = "Hermetic CLI that automates the tech-diagramming quality gate"
license = "MIT"
repository = "https://github.com/driftsys/diagctl"

[dependencies]
clap = { version = "4", features = ["derive"] }
usvg = "0.45"
roxmltree = "0.20"
anyhow = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

- [ ] **Step 2: Write minimal `src/main.rs`**

```rust
fn main() {}
```

- [ ] **Step 3: Write `.gitignore`**

```gitignore
/target
/wip/superpowers/
```

- [ ] **Step 4: Verify it builds**

Run: `cargo build` Expected: compiles; dependencies (clap, usvg, roxmltree,
anyhow) resolve and download. Note the exact `usvg` version cargo picks (check
`Cargo.lock`).

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs .gitignore
git commit -m "chore: cargo project init with deps"
```

---

## Task 2: SVG fixtures + `svg::load`

**Files:**

- Create: `tests/fixtures/in-band.svg`
- Create: `tests/fixtures/strip.svg`
- Create: `tests/fixtures/no-viewbox.svg`
- Create: `src/svg.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create the three fixtures**

`tests/fixtures/in-band.svg` (viewBox present, ratio 400/300 = 1.33 ≤ 2.5):

```xml
<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300" viewBox="0 0 400 300">
  <rect x="10" y="10" width="380" height="280" fill="none" stroke="black"/>
</svg>
```

`tests/fixtures/strip.svg` (viewBox present, ratio 900/200 = 4.5 > 2.5):

```xml
<svg xmlns="http://www.w3.org/2000/svg" width="900" height="200" viewBox="0 0 900 200">
  <rect x="10" y="10" width="880" height="180" fill="none" stroke="black"/>
</svg>
```

`tests/fixtures/no-viewbox.svg` (square 200×200, NO viewBox → aspect passes,
Layer-0 fails):

```xml
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
  <rect x="10" y="10" width="180" height="180" fill="none" stroke="black"/>
</svg>
```

- [ ] **Step 2: Write the failing test in `src/svg.rs`**

```rust
use std::path::Path;

pub struct Svg {
    pub raw: String,
    pub tree: usvg::Tree,
}

pub fn load(path: &Path) -> anyhow::Result<Svg> {
    unimplemented!()
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
}
```

- [ ] **Step 3: Wire the module into `src/main.rs`**

```rust
mod svg;

fn main() {}
```

- [ ] **Step 4: Run the test to verify it fails**

Run: `cargo test load_reads_resolved_size_from_fixture` Expected: FAIL — panics
at `unimplemented!()`.

- [ ] **Step 5: Implement `load`**

Replace the `unimplemented!()` body:

```rust
pub fn load(path: &Path) -> anyhow::Result<Svg> {
    let raw = std::fs::read_to_string(path)?;
    let tree = usvg::Tree::from_str(&raw, &usvg::Options::default())?;
    Ok(Svg { raw, tree })
}
```

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test load_reads_resolved_size_from_fixture` Expected: PASS. If
`width()`/`height()` resolve to 100 instead of 400/300, the resolved-size
assumption is wrong for the resolved usvg version — keep the explicit
`width`/`height` in the fixtures (already present) which forces the size;
re-run.

- [ ] **Step 7: Commit**

```bash
git add tests/fixtures src/svg.rs src/main.rs
git commit -m "feat: svg::load parses fixtures via usvg"
```

---

## Task 3: Checks registry + Layer-2 aspect-ratio

**Files:**

- Create: `src/checks/mod.rs`
- Create: `src/checks/aspect.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write `src/checks/mod.rs` with the types and a registry covering
      only aspect**

```rust
use crate::svg::Svg;

pub mod aspect;

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
    vec![aspect::check(svg)]
}
```

- [ ] **Step 2: Write the failing test in `src/checks/aspect.rs`**

```rust
use crate::checks::{CheckResult, Status};
use crate::svg::Svg;

pub const ASPECT_LIMIT: f32 = 2.5;

pub fn check(svg: &Svg) -> CheckResult {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    #[test]
    fn in_band_passes() {
        let svg = load(Path::new("tests/fixtures/in-band.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }

    #[test]
    fn strip_fails() {
        let svg = load(Path::new("tests/fixtures/strip.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "aspect-ratio");
    }
}
```

- [ ] **Step 3: Wire modules into `src/main.rs`**

```rust
mod checks;
mod svg;

fn main() {}
```

- [ ] **Step 4: Run the tests to verify they fail**

Run: `cargo test aspect` Expected: FAIL — `unimplemented!()`.

- [ ] **Step 5: Implement `check`**

Replace the `unimplemented!()` body:

```rust
pub fn check(svg: &Svg) -> CheckResult {
    let size = svg.tree.size();
    let (w, h) = (size.width(), size.height());
    let (long, short) = if w >= h { (w, h) } else { (h, w) };
    let ratio = long / short;
    let status = if ratio > ASPECT_LIMIT { Status::Fail } else { Status::Pass };
    CheckResult {
        id: "aspect-ratio",
        layer: 2,
        status,
        detail: format!("viewBox long/short ratio {ratio:.2} (limit {ASPECT_LIMIT})"),
    }
}
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test aspect` Expected: PASS (both).

- [ ] **Step 7: Commit**

```bash
git add src/checks src/main.rs
git commit -m "feat: Layer-2 aspect-ratio check (threshold 2.5)"
```

---

## Task 4: Layer-0 viewBox-present check

**Files:**

- Create: `src/checks/viewbox.rs`
- Modify: `src/checks/mod.rs`

- [ ] **Step 1: Write the failing test in `src/checks/viewbox.rs`**

```rust
use crate::checks::{CheckResult, Status};
use crate::svg::Svg;

pub fn check(svg: &Svg) -> CheckResult {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg::load;
    use std::path::Path;

    #[test]
    fn present_passes() {
        let svg = load(Path::new("tests/fixtures/in-band.svg")).unwrap();
        assert_eq!(check(&svg).status, Status::Pass);
    }

    #[test]
    fn absent_fails() {
        let svg = load(Path::new("tests/fixtures/no-viewbox.svg")).unwrap();
        let r = check(&svg);
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.id, "viewbox-present");
    }
}
```

- [ ] **Step 2: Register the module in `src/checks/mod.rs`**

Add `pub mod viewbox;` next to `pub mod aspect;`, and update `run_all`:

```rust
pub fn run_all(svg: &Svg) -> Vec<CheckResult> {
    vec![viewbox::check(svg), aspect::check(svg)]
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test viewbox` Expected: FAIL — `unimplemented!()`.

- [ ] **Step 4: Implement `check`**

Replace the `unimplemented!()` body:

```rust
pub fn check(svg: &Svg) -> CheckResult {
    let present = roxmltree::Document::parse(&svg.raw)
        .map(|doc| doc.root_element().attribute("viewBox").is_some())
        .unwrap_or(false);
    let (status, detail) = if present {
        (Status::Pass, "root <svg> has a viewBox".to_string())
    } else {
        (Status::Fail, "root <svg> is missing a viewBox".to_string())
    };
    CheckResult {
        id: "viewbox-present",
        layer: 0,
        status,
        detail,
    }
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test viewbox` Expected: PASS (both).

- [ ] **Step 6: Commit**

```bash
git add src/checks
git commit -m "feat: Layer-0 viewBox-present check"
```

---

## Task 5: CLI, check command, stubs, end-to-end integration tests

**Files:**

- Create: `src/cli.rs`
- Create: `src/commands/mod.rs`
- Create: `src/commands/check.rs`
- Create: `src/commands/optimize.rs`
- Create: `src/commands/ascii.rs`
- Create: `src/commands/freshness.rs`
- Create: `tests/cli.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write the failing integration tests in `tests/cli.rs`**

```rust
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test cli` Expected: FAIL — binary has no subcommands yet
(clap errors / wrong exit codes).

- [ ] **Step 3: Write `src/cli.rs`**

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "diagctl", version, about = "Automates the tech-diagramming quality gate")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run the diagram quality gate on an SVG file
    Check { file: PathBuf },
    /// (not yet implemented) Conservative SVGO-style optimization
    Optimize { file: PathBuf },
    /// (not yet implemented) ASCII grid render + width-aware alignment
    Ascii { file: PathBuf },
    /// (not yet implemented) Re-render and byte-diff freshness check
    Freshness { file: PathBuf },
}
```

- [ ] **Step 4: Write `src/commands/mod.rs`**

```rust
pub mod ascii;
pub mod check;
pub mod freshness;
pub mod optimize;
```

- [ ] **Step 5: Write `src/commands/check.rs`**

```rust
use crate::checks::{self, Status};
use crate::svg;
use std::path::Path;

pub fn run(file: &Path) -> anyhow::Result<i32> {
    let svg = svg::load(file)?;
    let results = checks::run_all(&svg);
    let mut any_failed = false;
    for r in &results {
        let tag = match r.status {
            Status::Pass => "PASS",
            Status::Fail => {
                any_failed = true;
                "FAIL"
            }
        };
        println!("{tag}  [L{}] {}  {}", r.layer, r.id, r.detail);
    }
    Ok(if any_failed { 1 } else { 0 })
}
```

- [ ] **Step 6: Write the three stub command modules**

`src/commands/optimize.rs`:

```rust
use std::path::Path;

pub fn run(_file: &Path) -> anyhow::Result<i32> {
    eprintln!(
        "optimize: not yet implemented (tracking: https://github.com/driftsys/metapowers/issues/14)"
    );
    Ok(2)
}
```

`src/commands/ascii.rs`:

```rust
use std::path::Path;

pub fn run(_file: &Path) -> anyhow::Result<i32> {
    eprintln!(
        "ascii: not yet implemented (tracking: https://github.com/driftsys/metapowers/issues/14)"
    );
    Ok(2)
}
```

`src/commands/freshness.rs`:

```rust
use std::path::Path;

pub fn run(_file: &Path) -> anyhow::Result<i32> {
    eprintln!(
        "freshness: not yet implemented (tracking: https://github.com/driftsys/metapowers/issues/14)"
    );
    Ok(2)
}
```

- [ ] **Step 7: Rewrite `src/main.rs` to dispatch**

```rust
mod checks;
mod cli;
mod commands;
mod svg;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Check { file } => commands::check::run(&file),
        Command::Optimize { file } => commands::optimize::run(&file),
        Command::Ascii { file } => commands::ascii::run(&file),
        Command::Freshness { file } => commands::freshness::run(&file),
    };
    match result {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("diagctl: {err:#}");
            std::process::exit(2);
        }
    }
}
```

- [ ] **Step 8: Run the full suite to verify it passes**

Run: `cargo test` Expected: PASS — all unit tests (svg, aspect, viewbox) and all
integration tests (version, help, check 0/1/1, missing-file 2, stubs 2).

- [ ] **Step 9: Commit**

```bash
git add src/cli.rs src/commands src/main.rs tests/cli.rs
git commit -m "feat: CLI dispatch, check command, and stubbed subcommands"
```

---

## Task 6: CI workflow

**Files:**

- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Write `.github/workflows/ci.yml`**

```yaml
name: ci
on:
  push:
    branches: [main]
  pull_request:
jobs:
  test:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --all
```

- [ ] **Step 2: Verify the three gates pass locally**

Run:
`cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: all clean (fmt no diff, clippy no warnings, tests pass). Fix any
`cargo fmt` diffs and clippy findings, then re-run.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: fmt + clippy + test on macOS/Linux/Windows"
```

---

## Task 7: Cross-platform release wiring

**Files:**

- Create: `.github/workflows/release.yml` (+ cargo-dist config if the tool
  initializes cleanly)
- Possibly modify: `Cargo.toml` (cargo-dist metadata)

- [ ] **Step 1: Attempt cargo-dist (preferred)**

Run: `cargo install cargo-dist --locked` then `dist init --yes` Expected:
generates `[workspace.metadata.dist]` (or `dist-workspace.toml`) and
`.github/workflows/release.yml`. If it succeeds, set the targets to
`x86_64-apple-darwin, aarch64-apple-darwin, x86_64-unknown-linux-musl, x86_64-pc-windows-msvc`,
then **skip to Step 3**.

- [ ] **Step 2: Fallback — hand-authored release workflow (use only if Step 1
      cannot install/init)**

Write `.github/workflows/release.yml`:

```yaml
name: release
on:
  push:
    tags: ["v*"]
permissions:
  contents: write
jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Install musl tools
        if: contains(matrix.target, 'musl')
        run: sudo apt-get update && sudo apt-get install -y musl-tools
      - run: cargo build --release --target ${{ matrix.target }}
      - name: Package
        shell: bash
        run: |
          ext=""
          case "${{ matrix.target }}" in *windows*) ext=".exe";; esac
          tar -czf "diagctl-${{ matrix.target }}.tar.gz" \
            -C "target/${{ matrix.target }}/release" "diagctl${ext}"
      - uses: softprops/action-gh-release@v2
        with:
          files: diagctl-*.tar.gz
```

- [ ] **Step 3: Validate the workflow YAML parses**

Run:
`python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/release.yml')); print('ok')"`
Expected: prints `ok`. (Full release only runs on tag push; YAML validity is the
local gate.)

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/release.yml Cargo.toml Cargo.lock
git commit -m "ci: cross-platform release binaries for macOS/Linux/Windows"
```

---

## Task 8: README build/test/run section

**Files:**

- Modify: `README.md`

- [ ] **Step 1: Add a "Build, test, run" section to `README.md`**

Insert before the `## License` section:

````markdown
## Build, test, run

Requires a Rust toolchain (1.95+).

```bash
cargo build            # debug build
cargo test             # run the suite (unit + binary integration tests)
cargo run -- --help    # see the subcommands
cargo run -- --version

# the one real slice today: the diagram gate
cargo run -- check path/to/diagram.svg   # exit 0 pass / 1 fail / 2 error
````

`check` runs today: **viewBox present** (Layer-0) and **aspect-ratio** (Layer-2,
long/short viewBox ratio, fails above 2.5). `optimize`, `ascii`, and `freshness`
are stubbed (exit 2) pending the next milestone.

````
- [ ] **Step 2: Update the "Status" note in `README.md`**

Change the status blockquote near the top from "not yet implemented" to reflect the skeleton:
```markdown
> **Status:** walking skeleton. `check` runs the Layer-0 viewBox + Layer-2
> aspect-ratio gate today; `optimize`/`ascii`/`freshness` are stubbed. Full
> design, scope, and acceptance criteria live in
> **[metapowers#14](https://github.com/driftsys/metapowers/issues/14)**.
````

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: README build/test/run + status"
```

---

## Final verification (before PR)

- [ ] Run `cargo test` → all green.
- [ ] Run `cargo fmt --all -- --check` → no diff.
- [ ] Run `cargo clippy --all-targets -- -D warnings` → no warnings.
- [ ] Run `cargo run -- --version` → prints `diagctl 0.1.0`.
- [ ] Run `cargo run -- check tests/fixtures/strip.svg; echo $?` → `1`.
- [ ] Run `cargo run -- check tests/fixtures/in-band.svg; echo $?` → `0`.
- [ ] Run `cargo run -- optimize tests/fixtures/in-band.svg; echo $?` → `2`.
- [ ] Push branch, open PR against `main`, report back.

---

## Self-Review

**Spec coverage:**

- `--help`/`--version` → Task 5 (integration tests) + clap in Task 5. ✓
- `check` Layer-0 viewBox-present → Task 4. ✓
- `check` Layer-2 aspect-ratio (threshold 2.5, mirrors #34) → Task 3. ✓
- Exercises usvg → Task 2 (`svg::load`) + Task 3 (aspect reads `tree.size()`). ✓
- Exit-code contract 0/1/2 → Task 5 (`check` 0/1, missing-file 2) + stubs 2. ✓
- Three fixtures → Task 2. ✓
- Stubs for optimize/ascii/freshness → Task 5. ✓
- CI matrix (fmt/clippy/test) → Task 6. ✓
- Cross-platform release (cargo-dist) → Task 7. ✓
- README build/test/run → Task 8. ✓
- TDD, no mocks, real fixtures → every check/command task writes the failing
  test first against committed SVGs. ✓

**Placeholder scan:** No "TBD"/"add error handling"/"similar to". The only
deliberate `unimplemented!()` bodies are TDD red-state stubs replaced within the
same task. The one stated uncertainty (usvg signature / resolved-size) has an
explicit, concrete resolution path. ✓

**Type consistency:** `Svg { raw, tree }`, `load(&Path) -> anyhow::Result<Svg>`,
`Status::{Pass,Fail}`, `CheckResult { id, layer, status, detail }`,
`run_all(&Svg) -> Vec<CheckResult>`,
`commands::*::run(&Path) -> anyhow::Result<i32>`, ids
`"viewbox-present"`/`"aspect-ratio"` — used identically across Tasks 2–5. ✓

```
```

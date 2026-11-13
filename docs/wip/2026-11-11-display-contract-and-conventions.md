# Display contract and output conventions — decisions

Date: 2026-11-11. Source: cross-tool review of pixctl, tblctl and diagctl.

## Decisions

1. **Display contract (#22).** Add `src/contract.rs` with constants copied by
   hand from pixctl's `src/contract.rs` (column width 800 px, minimum displayed
   font 11 px, opaque white background), plus a `CONTRACT_VERSION` that is shown
   in `--version`. Decided 2026-11-13: the whole SVG must fit the 800 px column.
   Diagrams carry no padding or frame; pixctl's 16 px padding and 1 px frame
   (766 px content) apply only to raster images. Both tools agree that the
   displayed image is at most 800 px wide. The existing `MIN_FONT_SIZE = 8.0`
   stays the `html` floor.
2. **Order of work for `--target` (#14, #19–#21).**
   1. Contract PR (decision 1).
   2. Step 0 evidence recorded on #14: an `<img>` of a width-less SVG fills the
      column in headless Chrome and a GitHub preview.
   3. `--target html|markdown` switch; default `html` keeps current behaviour.
   4. #19 `dimensions`, #20 `min-font-size` at displayed size, #21
      `opaque-background` — one PR each.
3. **Output conventions.** Decided 2026-11-13, replacing the earlier plan to
   copy pixctl's JSON array. diagctl follows tblctl's clig.dev-style convention
   (tblctl AD-0005), after the `--target` work, in separate PRs:
   - With `--json` (on every command), stdout is one pretty-printed object
     `{"schema":"diagctl/<command>/v1","results":[...],"summary":{...},"errors":[...]}`,
     printed even when there are errors. Enum values are lowercase. Each error
     is also printed on stderr as `diagctl: <message>`.
   - Human output: `PASS` green, `FAIL` red, `REVIEW` yellow, only on a TTY;
     `NO_COLOR` wins over `CLICOLOR_FORCE`. Check output ends with
     `N checks: P passed, F failed`. `--help` shows examples for each command
     and a README link.
   - `REVIEW` exits 0 and `--strict` makes it exit 1; `check` accepts multiple
     files. Exit codes stay 0/1/2.
4. **Docs layout.** Working memory lives in tracked `docs/wip/`. Remove the
   `/wip/superpowers/` ignore entry. Move the local `wip/superpowers/` notes for
   shipped work to `docs/archive/`, and the notes for open work (the ascii
   design, PR #13) to `docs/wip/`. Garden `docs/wip/` before each PR to `main`,
   or record the remaining files as accepted debt in the PR description.

## Alternatives considered

- Contract: a shared crate (no drift, but another repo and release cycle) and a
  versioned config file (no rebuild needed, but file discovery in both tools).
  Rejected for now; revisit if the values drift.
- Output: adopting the new output before `--target`. Rejected to keep the
  markdown work unblocked. pixctl's JSON array (one object per input) was
  replaced by the shared envelope object.

## Related

diagctl#14, #19, #20, #21, #22; pixctl#1.

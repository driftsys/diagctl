# Display contract and output conventions — decisions

Date: 2026-11-11. Source: cross-tool review of pixctl, tblctl and diagctl.

## Decisions

1. **Display contract (#22).** Add `src/contract.rs` with constants copied by
   hand from pixctl's `src/contract.rs` (column width 800 px, minimum displayed
   font 11 px, opaque white background), plus a `CONTRACT_VERSION` that is shown
   in `--version`. The existing `MIN_FONT_SIZE = 8.0` stays the `html` floor.
2. **Order of work for `--target` (#14, #19–#21).**
   1. Contract PR (decision 1).
   2. Step 0 evidence recorded on #14: an `<img>` of a width-less SVG fills the
      column in headless Chrome and a GitHub preview.
   3. `--target html|markdown` switch; default `html` keeps current behaviour.
   4. #19 `dimensions`, #20 `min-font-size` at displayed size, #21
      `opaque-background` — one PR each.
3. **Output conventions.** After the `--target` work, match pixctl in separate
   PRs: `PASS|FAIL|REVIEW` (`REVIEW` exits 0, `--strict` makes it exit 1),
   `--json` (an array with one object per input, lowercase enum values), and
   `check` accepting multiple files.
4. **Docs layout.** Working memory lives in tracked `docs/wip/`. Remove the
   `/wip/superpowers/` ignore entry. Move the local `wip/superpowers/` notes for
   shipped work to `docs/archive/`, and the notes for open work (the ascii
   design, PR #13) to `docs/wip/`. Garden `docs/wip/` before each PR to `main`,
   or record the remaining files as accepted debt in the PR description.

## Alternatives considered

- Contract: a shared crate (no drift, but another repo and release cycle) and a
  versioned config file (no rebuild needed, but file discovery in both tools).
  Rejected for now; revisit if the values drift.
- Output: adopting `REVIEW` before `--target`. Rejected to keep the markdown
  work unblocked.

## Related

diagctl#14, #19, #20, #21, #22; pixctl#1.

# diagctl ascii — pipeline MVP — design

**Date:** 2026-07-05 **Status:** approved (user lgtm 2026-07-05) **Tracks:**
[metapowers#14](https://github.com/driftsys/metapowers/issues/14) — the `ascii`
subcommand body (Phase-2 tooling debt). First slice of a stubbed subcommand.

The **first shape** of `diagctl ascii`. Renders a horizontal box-and-arrow
**pipeline** from AI-supplied structure into perfectly column-aligned monospace
ASCII — the tool does the character-counting the LLM cannot ("render, don't
draw"). Pipeline-only; tree / table / byte-layout are follow-on slices.

## Responsibility

Given the ordered labels of a pipeline, emit an aligned ASCII box-and-arrow
diagram. This is a **generator** (structure in → text out), distinct from
`check` (SVG inspector).

## Interface

- **Input:** labels on **stdin**, one per line. Blank lines and surrounding
  whitespace are ignored. No labels at all → operational error.
- **Output:** the diagram on **stdout**, e.g. for `Source` / `Filter` / `Sink`:

  ```text
  +--------+      +--------+      +--------+
  | Source | ---> | Filter | ---> |  Sink  |
  +--------+      +--------+      +--------+
  ```

- **Exit codes:** `0` success · `2` bad input (no labels). Matches the existing
  contract (`0` ok / `1` gate failure — N/A here / `2` operational error).
- **CLI change:** `Command::Ascii { file: PathBuf }` → `Command::Ascii` (reads
  stdin). The `file: PathBuf` was copied from the SVG subcommands and does not
  apply to a generator.

## Layout — grid-code (compute widths, emit exact characters)

- `cell = longest_label.chars().count() + 2` (one space of padding each side of
  the longest).
- Each box, `width = cell`:
  - border row: `+` + `-` × `width` + `+` (top and bottom identical).
  - label row: `|` + label **centered** in `width` + `|`. Centering:
    `left = extra / 2`, `right = extra - left` (extra space biased right,
    matching the skill's worked example `|  Sink  |`).
- Boxes joined left-to-right by a 6-char connector: `--->` on the label row, and
  6 spaces on the border rows (so borders never collide with the arrow).
- **Charset: ASCII `+ - |` only.** No Unicode box-drawing in this slice
  (follow-on).
- **Width basis: `char` count** (`str::chars().count()`), not display width.
  ASCII labels are the MVP target; wide/CJK display-width handling is a
  follow-on (`unicode-width`).

## Architecture

```text
src/cli.rs             Ascii variant: drop the file arg (now argless, reads stdin)
src/main.rs            dispatch Ascii -> commands::ascii::run()  (signature change: no path)
src/commands/ascii.rs  run(): read stdin -> parse labels -> render -> print -> exit 0/2
src/ascii/mod.rs       NEW module
src/ascii/pipeline.rs  NEW — render(labels: &[String]) -> String   (pure, unit-tested core)
```

`render` is pure and holds all the width math; the command is thin I/O around
it. No new dependencies (no serde/JSON, no unicode-width).

## Data flow & error handling

`commands::ascii::run()`:

1. Read all of stdin to a `String`.
2. Split on lines; `trim` each; drop empties → `Vec<String>`.
3. If empty → `eprintln!` a message and return `Ok(2)`.
4. Else `println!("{}", pipeline::render(&labels))` and return `Ok(0)`.

`main.rs` currently calls `commands::ascii::run(file)`; change to
`commands::ascii::run()`.

## Input format decision (recorded)

Plain newline-delimited labels, **not JSON** — zero new deps, trivially
AI-emittable. JSON becomes the shared input contract when tree/table (which need
nested structure) land; diagctl is v0.1.0 with no users, so that later migration
is cheap. Alternative (JSON now via serde_json) rejected for this slice on YAGNI
grounds.

## Testing (TDD, no mocks)

- Unit `pipeline::render`:
  1. Three boxes `["Source","Filter","Sink"]` → exact expected string (the block
     above).
  2. Single box `["Only"]` → three rows, no connector.
  3. Mixed lengths `["A","Longlabel","Cc"]` → every row has identical char
     length; width set by `Longlabel`.
  4. Property on (3): the three rendered lines are equal `.chars().count()`.
- Integration (`tests/cli.rs`, assert_cmd `.write_stdin`): 5.
  `Source\nFilter\nSink\n` → exit 0, stdout contains
  `| Source | ---> | Filter |`. 6. empty stdin → exit 2.
- Existing tests unchanged; `check`/`optimize`/`freshness` untouched.

## Verifiable success criteria

1. `pipeline::render(["Source","Filter","Sink"])` equals the exact 3-line block
   above.
2. Every line of any `render` output has an identical `chars().count()`.
3. `printf 'Source\nFilter\nSink\n' | diagctl ascii` → exit 0 + aligned diagram.
4. Empty stdin → exit 2.
5. `--help` still lists `ascii`; `optimize`/`freshness` still stubbed (exit 2).
6. `cargo test` green; fmt + clippy (`-D warnings`) clean.

## Out of scope (follow-on slices)

JSON structure input · `--unicode` box-drawing (`┌─┐│└┘`) · wide/CJK
display-width (`unicode-width`) · tree / table / byte-layout shapes · `optimize`
/ `freshness` bodies.

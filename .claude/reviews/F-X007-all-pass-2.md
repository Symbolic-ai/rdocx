# F-X007, all aspects, pass 2

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree. This is 34 files and 1,639 changed or
new line entries: 27 tracked files with 1,199 additions and 170 deletions, plus
270 lines in seven untracked files.
**Verdict**: 4 defects, 0 smells, 1 nitpick

## Defects

### D1, preserved numbering extensions can lose their namespace bindings

`crates/rdocx-oxml/src/numbering.rs:454`
`crates/rdocx-oxml/src/numbering.rs:487`

The parser captures an unknown child as raw bytes, but it does not retain the
attributes or namespace declarations from `w:numbering`. The serializer then
constructs a new root with only `xmlns:w` and `xmlns:r`. A producer extension
such as `<w15:extension/>` whose `xmlns:w15` declaration was inherited from the
original root is emitted with an unbound prefix on the next document save. The
same loss applies to `mc:Ignorable` and namespace bindings needed by raw
children below an abstract definition or level. The preservation regression at
`crates/rdocx-oxml/src/numbering.rs:947` uses only the retained `w` prefix, so it
cannot detect this invalid output. This contradicts the producer-extension
preservation contract in `docs/hld/04-opc-and-packaging.md:107`.

### D2, redefining a sparse level can move raw children before their schema predecessors

`crates/rdocx-oxml/src/numbering.rs:125`
`crates/rdocx-oxml/src/numbering.rs:641`

Raw-child boundaries are recorded against only the modeled children present
during parsing. `set_list_level` can then materialize `start`, `num_fmt`, and
`lvl_text` without shifting any of those boundaries. For example, a valid level
with `w:start`, then `w:lvlRestart`, then `w:lvlText`, but no `w:numFmt`, records
`w:lvlRestart` at boundary 1. Redefining that level adds `w:numFmt`, while the
unchanged boundary causes the writer to emit `w:lvlRestart` before `w:numFmt`.
That violates the `CT_Lvl` sequence and can make Word reject the numbering
part. The current regression has all three fields already present at
`crates/rdocx-oxml/src/numbering.rs:953`, so it does not exercise field
materialization.

### D3, a self-closing numbering root is captured as its own child

`crates/rdocx-oxml/src/numbering.rs:460`
`crates/rdocx-oxml/src/numbering.rs:495`

Every empty event at the root level is treated as an unknown child, including a
valid self-closing `<w:numbering/>`. Serializing that model creates a canonical
outer `w:numbering` and writes the captured self-closing root inside it. Since
`Document::flush_to_package` always serializes a loaded numbering model at
`crates/rdocx/src/document.rs:291`, merely saving such a package produces a
nested and invalid numbering root. Self-closing modeled `w:abstractNum`
elements are misclassified by the same branch, which also hides their IDs from
collision-safe allocation.

### D4, public PDF rustdoc still advertises a nonexistent feature

`crates/rdocx/src/document.rs:2285`
`crates/rdocx/src/document.rs:2312`

Both public PDF methods say bundled fonts depend on a `bundled-fonts` feature,
but `rdocx` exposes only `system-fonts` at
`crates/rdocx/Cargo.toml:19`. The root README was corrected to explain that
bundled deterministic fonts are always available, while these docs still send
users toward a feature Cargo cannot resolve. The documentation hardening is
therefore internally inconsistent and incomplete.

## Smells

None.

## Nitpicks

- `crates/rdocx/src/table.rs:163`, the public `set_column_width` failure list
  omits the negative-width rejection implemented at line 166.

## Verification evidence

- `cargo test -p rdocx-oxml`: 112 unit tests and 1 doc test passed.
- `cargo test -p rdocx`: 69 unit tests, 81 integration tests, 17 regression
  tests, and 2 doc tests passed.
- `python3 scripts/readme_doctests.py`: all 12 Rust examples across the six
  stable libraries compiled.
- `cargo package --locked --allow-dirty --list -p <package>` for all seven
  stable packages: each inventory contained exactly one intended README.
- `python3 scripts/hash_harness.py --check`: all 28 entries matched.
- `cargo fmt --all --check`, `git diff --check`, `python3
  scripts/prose_check.py`, and `python3 scripts/sync_agent_skills.py --check`:
  passed.
- GitHub PR 25 is merged into `sprint/s38` at `6aade64`, retains Jon Stokes as
  `@jonstokes`, and has a public valuable-fix merge note. No contributor-credit
  defect was found.

## Not found

No additional defects were found in hyperlink or hard-break semantics, table
geometry staging and overflow handling, list and paragraph level bounds, ID
allocation, stable package README inventory, README example compilation, HLD
scope, archive contents, hash stability, or PR credit evidence. No structural
rule violation, resource-bound smell, panic on the reviewed public inputs, or
nitpick beyond the one listed above was found.

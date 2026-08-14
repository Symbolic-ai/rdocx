# F-X007, all aspects, pass 16

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the current working tree and all fifteen earlier review and
remediation rounds. The reviewed state is 56 files and 8,801 changed or new
line entries: 35 tracked files with 5,770 additions and 728 deletions, plus
2,303 lines in twenty-one untracked files.
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, paragraph-property namespace context still stops at aggregate and direct-body boundaries

`crates/rdocx-oxml/src/styles.rs:58`
`crates/rdocx-oxml/src/styles.rs:59`
`crates/rdocx-oxml/src/styles.rs:62`
`crates/rdocx-oxml/src/styles.rs:105`
`crates/rdocx-oxml/src/styles.rs:108`
`crates/rdocx-oxml/src/text.rs:332`
`crates/rdocx-oxml/src/text.rs:333`
`crates/rdocx-oxml/src/text.rs:350`
`crates/rdocx-oxml/src/text.rs:353`
`crates/rdocx-oxml/src/document.rs:604`
`crates/rdocx-oxml/src/document.rs:606`
`crates/rdocx-oxml/src/document.rs:609`
`crates/rdocx-oxml/src/document.rs:610`
`crates/rdocx-oxml/src/table.rs:944`
`crates/rdocx-oxml/src/table.rs:957`
`crates/rdocx-oxml/src/header_footer.rs:42`
`crates/rdocx-oxml/src/header_footer.rs:59`
`crates/rdocx-oxml/src/footnotes.rs:140`
`crates/rdocx-oxml/src/footnotes.rs:150`
`crates/rdocx-oxml/README.md:42`
`crates/rdocx-oxml/README.md:44`
`docs/hld/10-bindings-spec.md:223`
`docs/hld/10-bindings-spec.md:227`
`docs/hld/14-development-backlog.md:1202`
`docs/hld/14-development-backlog.md:1213`

The pass-15 remediation correctly carries the real root, style, body, and
direct paragraph scope into the property projection. It does not carry that
scope across every reader covered by the same public and HLD contract. The
body computes the in-scope bindings for a table start, then calls
`CT_Tbl::from_xml` without them. A table cell later calls the public
`CT_P::from_xml`, which starts again with only `w` as a WordprocessingML
prefix. Header and footnote parts use that same context-free paragraph entry
point. The public `CT_Style::from_xml` also ignores namespace declarations on
the supplied style start and seeds its private parser with only `w`.

For example, a valid document whose root binds `q` to WordprocessingML and
whose table cell contains
`<q:p><q:pPr><q:jc q:val="center"/></q:pPr></q:p>` reaches the table and cell
parsers by local name, but the paragraph rejects `q:pPr` as non-Word because
the root binding was discarded. The resulting cell paragraph has no typed
justification. An aliased header, footnote, or direct public style parse fails
the same way. Before the contextual narrowing, these established paths parsed
the same content by local name. The README and HLD now expressly say style and
document readers retain aliased and default WordprocessingML behavior, so
fixing only aggregate styles and direct body paragraphs leaves the published
0.5 contract incomplete.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml`: all 152 unit tests and the crate README doctest
  passed. The three namespace-aware tab tests cover foreign shadows, expanded
  tab elements, and the normal 64-depth error.
- `cargo test -p rdocx`: all 69 unit, 81 integration, 17 regression, and two
  doctests passed.
- `cargo clippy -p rdocx-oxml -p rdocx --all-targets --all-features -- -D
  warnings`: passed.
- `python3 scripts/hash_harness.py --check`: all 28 entries matched.
- `python3 scripts/readme_doctests.py`: all twelve Rust examples across the six
  stable libraries compiled, and the inventory and exact snippet contracts
  passed.
- `cargo package --locked --allow-dirty --list -p <package>` for all seven
  stable packages: every inventory contains exactly one intended README.
- `cargo fmt --all --check`, `git diff --check`, `python3
  scripts/prose_check.py`, and `python3 scripts/sync_agent_skills.py --check`:
  passed.
- `gh pr view 25` confirms PR 25 remains merged into `sprint/s38` at
  `6aade64`, all three contributor commits retain Jon Stokes as author, and the
  public note credits `@jonstokes` while explaining the value and maintainer
  hardening.

## Not found

The style aggregate and direct body paragraph fixtures now use their actual
ancestor namespace scope, and foreign same-local property children do not
materialize typed state. Compatibility lookup uses the property container's
actual scope, including local rebindings, while the complete declaration
inventory is used only to reserve collision-free generated prefixes. The tab
parser rejects nesting beyond its 64-element bound with a normal error.

No further defect was found in recursive producer XML preservation, the
dedicated ignorable carrier, fixed property schema slots, tab occurrence
provenance, duplicate and out-of-range claims, the measured 10,000-occurrence
work bound, public list and table bounds, the 0.5 migration text, HLD and sprint
scope, README example quality, manifest wiring, package inventory,
deterministic hashes, or PR authorship and credit. No separate smell or
nitpick was found.

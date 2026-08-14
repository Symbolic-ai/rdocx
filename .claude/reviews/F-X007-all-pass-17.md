# F-X007, all aspects, pass 17

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the final current working tree and all sixteen earlier review
and remediation rounds. The reviewed state is 60 files and 9,263 changed or
new line entries: 38 tracked files with 6,099 additions and 753 deletions, plus
2,411 lines in twenty-two untracked files.
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the public context-free fallback promotes foreign property prefixes to WordprocessingML

`crates/rdocx-oxml/src/text.rs:332`
`crates/rdocx-oxml/src/text.rs:333`
`crates/rdocx-oxml/src/text.rs:350`
`crates/rdocx-oxml/src/text.rs:351`
`crates/rdocx-oxml/src/text.rs:355`
`crates/rdocx-oxml/src/text.rs:364`
`crates/rdocx-oxml/src/text.rs:369`
`crates/rdocx-oxml/src/text.rs:371`
`crates/rdocx-oxml/src/text.rs:630`
`crates/rdocx-oxml/src/text.rs:632`
`crates/rdocx-oxml/src/text.rs:650`
`crates/rdocx-oxml/src/text.rs:652`
`crates/rdocx-oxml/src/styles.rs:58`
`crates/rdocx-oxml/src/styles.rs:59`
`crates/rdocx-oxml/src/styles.rs:60`
`crates/rdocx-oxml/src/styles.rs:62`
`crates/rdocx-oxml/src/styles.rs:71`
`crates/rdocx-oxml/src/styles.rs:73`
`.claude/plans/F-X007-design.md:65`
`.claude/plans/F-X007-design.md:68`
`.claude/plans/F-X007-design.md:101`
`crates/rdocx-oxml/README.md:42`
`crates/rdocx-oxml/README.md:44`
`docs/hld/10-bindings-spec.md:224`
`docs/hld/10-bindings-spec.md:228`
`docs/hld/14-development-backlog.md:1213`
`docs/hld/14-development-backlog.md:1215`

The remediated aggregate readers now pass exact namespace scope, but the
public `CT_P::from_xml` entry point deliberately starts with an empty scope. At
the first same-local `pPr`, it appends that element's lexical prefix to the
Word prefix set whenever normal expanded-name resolution does not identify the
element as WordprocessingML. No namespace URI is established by that step.
`parse_scoped_ppr` then receives the invented Word prefix and can project the
foreign subtree into typed paragraph state.

For example, after a caller consumes a paragraph start whose ancestor binds
`ext` to `urn:producer`, a child
`<ext:pPr><ext:jc ext:val="right"/></ext:pPr>` has no declaration in its
captured bytes. The fallback adds `ext` as a Word prefix, and the projection
returns typed right justification instead of preserving the foreign container
as unmodelled XML. The new direct-paragraph regression contains exactly that
foreign container before a valid `q:pPr`. The later center value overwrites
the false right value, so the final assertion passes without proving the
foreign negative. Its name says it uses the paragraph start's ancestor scope,
but the public method never receives that start.

The public `CT_Style::from_xml` fallback has the same identity error. Even when
the supplied style start explicitly binds its own prefix to a foreign URI,
failure of the Word identity check unconditionally appends that prefix as
WordprocessingML. A same-prefix `pPr` subtree can then materialize typed style
properties. This contradicts the plan and HLD requirement that direct
boundaries reject foreign same-local properties. Positive alias compatibility
cannot be made namespace-safe by guessing a URI from QName spelling.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml`: all 162 unit tests and the crate README doctest
  passed. Focused direct paragraph, direct style, table-cell, header, and note
  namespace tests also passed.
- `cargo test -p rdocx`: all 69 unit, 81 integration, 17 regression, and two
  doctests passed.
- `cargo clippy -p rdocx-oxml -p rdocx --all-targets --all-features -- -D
  warnings`: passed.
- `python3 scripts/hash_harness.py --check`: all 28 entries matched on the
  final tightened tree.
- `python3 scripts/readme_doctests.py`: all twelve Rust examples across the six
  stable libraries compiled, with inventory and exact snippet contracts green.
- `cargo package --locked --allow-dirty -p rdocx-oxml`: packaged twenty files
  into an 83.6 KiB compressed archive and verified the package successfully.
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

Exact namespace scope now propagates through aggregate styles, direct body
paragraphs, tables and nested cells, headers, footers, footnotes, and endnotes.
Those context-bearing readers accept aliased and default WordprocessingML and
reject the tested foreign paragraph siblings. The final canonical `w:`
attribute fallback makes the direct default-namespace positive fixture pass.

No further defect was found in recursive producer XML preservation,
compatibility-scope resolution, collision-free prefix reservation, fixed OOXML
property slots, tab provenance and matching, the 10,000-occurrence work bound,
the 64-element tab depth bound, public list and table bounds, 0.5 migration
documentation, HLD and sprint scope, README example quality, manifest wiring,
package inventory, deterministic hashes, or PR authorship and credit. No
separate smell or nitpick was found.

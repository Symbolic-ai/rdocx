# F-X007, all aspects, pass 18

**Reviewed**: the complete feature state from sprint base `2f469e7` through
`cd1e6e1`, plus the final current working tree and all seventeen earlier review
and remediation rounds. The reviewed state is 61 files and 9,389 changed or
new line entries: 38 tracked files with 6,109 additions and 753 deletions, plus
2,527 lines in twenty-three untracked files.
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx-oxml`: all 164 unit tests and the crate README doctest
  passed. This includes the canonical, explicit-alias, default-namespace, and
  standalone foreign-only public paragraph and style cases, all contextual
  aggregate cases, the 64-level depth checks, and the 10,000-tab work gate.
- `cargo test -p rdocx`: all 69 unit, 81 integration, 17 regression, and two
  doctests passed. The custom-list, rejected-mutation, hyperlink, hard-break,
  and table-width contracts are green.
- `cargo clippy -p rdocx-oxml -p rdocx --all-targets --all-features -- -D
  warnings`: passed.
- `python3 scripts/hash_harness.py --check`: all 28 entries matched. The
  canonical styles and numbering outputs therefore remain byte-stable after
  the namespace and preservation work.
- `python3 scripts/readme_doctests.py`: all twelve Rust examples across the six
  stable libraries compiled, and its exact README inventory and snippet
  contracts passed.
- `cargo package --locked --allow-dirty -p rdocx-oxml`: packaged twenty files
  into an 85,557-byte compressed archive and verified it successfully, below
  the 10 MiB ceiling. Package inventories for all seven stable crates each
  contain exactly one intended `README.md`.
- `cargo fmt --all --check`, `git diff --check`, `python3
  scripts/prose_check.py`, and `python3 scripts/sync_agent_skills.py --check`:
  passed.
- `gh pr view 25` confirms PR 25 is merged into `sprint/s38` at `6aade64`, all
  three contributor commits retain Jon Stokes as author, and the public merge
  note credits `@jonstokes`, explains the contribution's value, and identifies
  the maintainer hardening.

## Not found

The pass-17 remediation closes the previous identity defect without replacing
it with a broader compatibility guess. The public context-free paragraph
entry point seeds only the established canonical `w` prefix
(`crates/rdocx-oxml/src/text.rs:332`), applies declarations found on each
property container (`crates/rdocx-oxml/src/text.rs:350`), and requires exact
WordprocessingML element identity before projection
(`crates/rdocx-oxml/src/text.rs:351`). Its foreign-only regression now asserts
the absence of typed properties directly
(`crates/rdocx-oxml/src/text.rs:640`). The public style entry point resolves
the supplied style start without inventing its QName identity
(`crates/rdocx-oxml/src/styles.rs:58`), and its child property check remains
expanded-name aware (`crates/rdocx-oxml/src/styles.rs:106`). The corresponding
foreign-only assertion is independent of a later valid property
(`crates/rdocx-oxml/src/styles.rs:551`).

Namespace context remains threaded through complete styles
(`crates/rdocx-oxml/src/styles.rs:329`), document body paragraphs
(`crates/rdocx-oxml/src/document.rs:592`), tables, rows, and cells
(`crates/rdocx-oxml/src/table.rs:949`), headers and footers
(`crates/rdocx-oxml/src/header_footer.rs:53`), and footnotes and endnotes
(`crates/rdocx-oxml/src/footnotes.rs:47`). Those readers accept aliased and
default WordprocessingML while their exact `is_word_element` boundaries reject
foreign same-local paragraph and property siblings.

No regression was found in the broader preservation contract. Paragraph and
run slot tables retain the OOXML sequence, including schema-final change
children (`crates/rdocx-oxml/src/numbering.rs:19` and
`crates/rdocx-oxml/src/numbering.rs:58`). Generated prefixes remain
collision-safe (`crates/rdocx-oxml/src/numbering.rs:253`), the preservation
carrier merges an expanded-name `mc:Ignorable` through actual scope
(`crates/rdocx-oxml/src/numbering.rs:319`), recursive property projection has
a normal 64-element bound (`crates/rdocx-oxml/src/numbering.rs:898`), and the
10,000-occurrence regression pins linear matching and writer work
(`crates/rdocx-oxml/src/numbering.rs:4377`). `CT_TabStop` exposes honest source
provenance while semantic equality ignores it
(`crates/rdocx-oxml/src/borders.rs:194`), and the public contextual tab parser
tracks shadows with the same bounded-depth contract
(`crates/rdocx-oxml/src/borders.rs:315`).

The public 0.5 migration is explicit in the crate README
(`crates/rdocx-oxml/README.md:24`) and the current HLD
(`docs/hld/10-bindings-spec.md:211`). HLD14 carries the complete preservation,
namespace, complexity, package, and credit gate
(`docs/hld/14-development-backlog.md:1186`). Current sprint, sprint plan, and
backlog agree on F-X007, the dependent F-X008 title, and the separately
approved 0.5.0 release boundary (`docs/sprints/CURRENT_SPRINT.md:24`,
`docs/sprints/SPRINT_PLAN.md:577`, and `docs/sprints/BACKLOG.md:283`). The seven
README purposes, examples or deprecation paths, manifest wiring, archive
inventories, deterministic hashes, and PR credit are consistent. No separate
smell or nitpick was found.

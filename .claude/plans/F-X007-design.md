# F-X007, Integrate PR 25 and stable crate documentation

**Status**: completed
**Sprint**: S38
**Size**: L
**Depends on**: none

## Problem

PR 25 adds valuable Word authoring APIs, but it was opened against the S34
tree and has no hosted checks. Review against current `main` found two concrete
merge blockers. `Table::set_column_width` leaves the table's declared width
inconsistent with its grid, and `Document::set_list_level` materializes an
empty numbering part after rejecting an unknown list identifier.

The stable package family also lacks crate-specific READMEs. The root README
documents `rdocx`, but users of the six companion crates do not get a concise
statement of purpose, a current example, or a clear deprecation path on their
crate page.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "Format-specific constructors stay at
  the format boundary" and relationship ownership.
- `docs/hld/10-bindings-spec.md`, "The rdocx wrapper" and public facade
  compatibility.
- `docs/hld/12-testing-strategy.md`, "README example correctness" and the
  workspace gate.
- `docs/hld/15-build-and-toolchain.md`, "The two release families" and README
  version inspection.

## Approach

Retain the contributor's three commits and add one bounded maintainer
correction on the PR branch. A rejected `set_list_level` call will neither
invalidate layout nor create numbering state. `set_column_width` will stage a
checked grid total, update the table width, and map physical cells through
their `gridSpan` coverage before applying synchronized cell widths.

Merge the PR into `sprint/s38`, not directly into `main`, with a GitHub merge
note that credits Jon Stokes as `@jonstokes`. Add one README per stable crate,
using the root README for the high-level `rdocx` guide and crate-local files
for the six companion packages. Extend the existing README doctest runner so
every Rust example is compile-checked against the matching packaged crate.

Preserve numbering XML by expanded namespace identity rather than prefix
spelling. Generated model QNames select a deterministic Word and relationship
prefix that does not replace a foreign source binding. Property-container
preservation uses explicit raw-XML state on the public numbering model. A
private overlay serializer applies typed `pPr` and `rPr` changes while
retaining producer attributes and child subtrees. These added fields make the
stable Rust model a breaking pre-1.0 change, so F-X008 releases the family at
`0.5.0`, not as a source-compatible `0.4.2` patch.

Repeated tab-stop preservation uses explicit provenance on each public
`CT_TabStop`. Parsed numbering tabs retain their original occurrence index,
new tabs carry no source occurrence, and semantic equality continues to
compare typed tab values rather than preservation provenance. The overlay
claims each source occurrence at most once in deterministic linear work, so an
edit and an insertion or removal cannot transfer producer state between typed
occurrences.

`CT_Tabs` exposes a namespace-aware parse path that receives the in-scope
WordprocessingML prefixes and maintains nested namespace scope with a normal
64-element depth bound. Numbering, style, body, table-cell, header, footer,
footnote, and endnote readers project complete paragraph-property subtrees through their actual namespace scope before
invoking the canonical `CT_PPr` parser. Foreign elements with a colliding local
name never materialize typed properties or tab stops. An explicit clear that must retain
producer attributes uses a dedicated foreign carrier with a non-modelled local
name. The enclosing `w:tabs` declares collision-free preservation and markup
compatibility prefixes, and merges the preservation prefix into any existing
expanded-name `mc:Ignorable` attribute resolved from the property's actual
ancestor scope. Complete document declarations reserve generated prefixes but
do not resolve producer attributes.
Prefix allocation uses the complete preserved namespace scope so generated
bindings never shadow inherited producer QName values.

## Rejected alternatives

- Merge the draft PR directly to `main`. Only `/close-sprint` may merge to
  `main`, and the two reproduced defects block acceptance.
- Fold version preparation into the contributor change. Release metadata and
  registry mutation belong to F-X008 after review.
- Copy one generic README to every package. Each package needs a distinct use
  case or deprecation path, not boilerplate.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `rejected_list_level_update_does_not_materialize_numbering` | Failed mutation leaves document state unchanged |
| regression | `table_column_width_updates_grid_table_and_spanning_cells` | Fixed geometry stays internally consistent |
| round-trip | existing PR 25 list, hyperlink, break, and table tests | New public authoring state survives save and reopen |
| integration | `python3 scripts/readme_doctests.py` | Every stable crate README Rust example compiles |
| integration | packaged README inventory | All seven stable archives carry the intended README |
| regression | `list_mutations_preserve_unmodelled_numbering_xml` | Producer numbering extensions survive list edits and additions |
| regression | list ID, level, and width boundary tests | Finite IDs allocate safely and invalid levels or widths do not mutate |
| integration | README snippet contracts | CLI flags, deterministic feature guidance, and shim dependencies match their examples |
| regression | namespace collision and rebinding fixtures | Foreign and aliased prefixes retain expanded identity and relative position |
| regression | numbering property overlay fixtures | Typed indentation and run-property edits retain producer attributes and children |
| regression | public tab parser and internal property projection fixtures | Nested namespace shadows, expanded tab elements, direct style and paragraph boundaries, table cells, headers, notes, and foreign same-local properties never materialize incorrect typed state |
| boundary | deeply nested tab namespace aliases | The parser rejects depth beyond 64 with a normal error before cumulative scope cloning can grow with input depth |
| regression | tab occurrence provenance fixtures | Same-segment edits plus insertions or removals retain producer ownership, duplicate claims are deterministic, and matching stays linear |
| round-trip | explicit tab clear with inherited namespace collisions | The dedicated carrier is `mc:Ignorable`, producer QName values retain their scope, and reparsing keeps the typed collection empty |
| migration | public numbering and tab-stop construction and equality | The `0.5.0` field additions are documented and semantic equality remains typed-value based |

The **test gate** is the merged PR's focused round-trip suite, the table and
numbering regressions, the public namespace parser and tab provenance cases,
the explicit-clear carrier round trip, and compile-checked stable crate README
examples.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Any parser or serializer. Read HLD04 and HLD06, preserve schema child order,
  and prove package-preserving round trips.
- Public API of a published crate. State the breaking pre-1.0 `0.5.0` boundary,
  document `CT_TabStop` provenance and namespace-aware tab parsing, run
  the exact stable archive dry run, and enforce the 10 MiB ceiling.
- Namespace and raw-property serialization. Use private prefix-parameterized
  writers, actual property ancestor scope, `mc:Ignorable`, and explicit
  model storage. The approved 0.5 boundary permits the named `CT_TabStop`
  field and the contextual tab parser path. Do not add a trait, feature, dependency,
  crate, module, or file.
- New files. The user explicitly requested crate READMEs. Keep one README per
  stable package and extend the existing runner rather than adding a script.
- Release scripting and version strings. README version references are
  inspected here. Actual version preparation and final approval belong to
  F-X008.

## Hash harness

Expected to be unchanged. The authoring additions must not alter existing
sample output or deterministic rendering.

## Implementation checklist

- [x] Rebase or merge current `main` into the PR acceptance result without a
  semantic conflict.
- [x] Add the two reproduced regression tests and their smallest fixes.
- [x] Cover `gridSpan` table geometry and rejected numbering mutation.
- [x] Preserve unmodelled numbering children through list mutations.
- [x] Preserve expanded namespace identity under prefix aliasing, rebinding,
  and canonical-prefix collisions.
- [x] Overlay typed numbering property changes without losing producer XML or
  changing canonical equality, and document the `0.5.0` construction change.
- [x] Enforce finite ID, nine-level, paragraph-level, and nonnegative-width boundaries.
- [x] Add and inventory documentation for all seven stable packages.
- [x] Compile every stable README Rust example against its package.
- [x] Run focused tests, full verification, archive checks, and hash harness.
- [x] Obtain a clean independent microscope review.
- [x] Write a valuable-fix merge note and credit `@jonstokes`.
- [x] Merge PR 25 into `sprint/s38` through GitHub.

## Open questions

None. The requested documentation scope is the seven-package stable family,
and the breaking pre-1.0 `0.5.0` release is isolated in F-X008.

# F-X007, Integrate PR 25 and stable crate documentation

**Status**: approved
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

The **test gate** is the merged PR's focused round-trip suite, the two named
regressions, and compile-checked stable crate README examples.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Any parser or serializer. Read HLD04 and HLD06, preserve schema child order,
  and prove package-preserving round trips.
- Public API of a published crate. State additive semver impact, run the exact
  stable archive dry run, and enforce the 10 MiB ceiling.
- New files. The user explicitly requested crate READMEs. Keep one README per
  stable package and extend the existing runner rather than adding a script.
- Release scripting and version strings. README version references are
  inspected here. Actual version preparation and final approval belong to
  F-X008.

## Hash harness

Expected to be unchanged. The authoring additions must not alter existing
sample output or deterministic rendering.

## Implementation checklist

- [ ] Rebase or merge current `main` into the PR acceptance result without a
  semantic conflict.
- [ ] Add the two reproduced regression tests and their smallest fixes.
- [ ] Cover `gridSpan` table geometry and rejected numbering mutation.
- [ ] Add and inventory documentation for all seven stable packages.
- [ ] Compile every stable README Rust example against its package.
- [ ] Run focused tests, full verification, archive checks, and hash harness.
- [ ] Obtain a clean independent microscope review.
- [ ] Write a valuable-fix merge note and credit `@jonstokes`.
- [ ] Merge PR 25 into `sprint/s38` through GitHub.

## Open questions

None. The requested documentation scope is the seven-package stable family,
and the fresh release is isolated in F-X008.

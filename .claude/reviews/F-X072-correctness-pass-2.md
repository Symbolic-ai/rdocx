# F-X072, correctness, pass 2

**Reviewed**: claim-base `43cd8a51af75f9ddfd3148236ac59dc095345c94`
through the complete working-tree delta, the approved design plan, cited HLD
sections, progress record, pass 1 review, and live Issue 65 contract. The
implementation delta is 1 file with 176 changed lines, 164 additions and 12
deletions. The untracked pass 1 review was also included.
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None. Count: 0.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 1 D1 remediation** is complete. The unsafe-prefix regression now
  constructs direct field, numbering, drawing, and raw-child paragraphs at
  `crates/rdocx-layout/src/engine.rs:9492`, exercises all four cases at
  `crates/rdocx-layout/src/engine.rs:9517`, compares warm and fresh output, and
  proves zero retained suffix hits at
  `crates/rdocx-layout/src/engine.rs:9532`. The exact focused regression passed.
- **Correctness** produced zero findings. Direct body footnote and endnote
  references are admitted only after every existing paragraph, property, and
  run safety guard at `crates/rdocx-layout/src/engine.rs:2299`. An eligible hit
  still requires the fingerprint, complete typed paragraph, exact width, and
  revision view at `crates/rdocx-layout/src/engine.rs:1903`. Therefore the
  deliberately coarse reference fingerprint at
  `crates/rdocx-layout/src/engine.rs:3914` remains only a prefilter and cannot
  alias distinct note streams or IDs.
- **Note identity and context invalidation** produced zero findings. The
  retained transaction owns complete footnote and endnote parts at
  `crates/rdocx-layout/src/engine.rs:485`, and exact equality of both parts is
  required before reuse at `crates/rdocx-layout/src/engine.rs:634`. The changed
  reference and changed note-part regression compares warm output with a fresh
  deterministic engine at `crates/rdocx-layout/src/engine.rs:9322`.
- **Scope boundaries** produced zero findings. Note-bearing header and footer
  paragraphs remain conservative at `crates/rdocx-layout/src/engine.rs:2402`.
  Every recursively visited table paragraph also remains conservative at
  `crates/rdocx-layout/src/engine.rs:2508`. The broadened predicate therefore
  applies only to the intended direct body paragraph path.
- **Unsupported content** produced zero findings. Raw paragraph children are
  rejected at `crates/rdocx-layout/src/engine.rs:2303`. Numbering, section, and
  property revision state are rejected at `crates/rdocx-layout/src/engine.rs:2315`. Run
  drawings, fields, comments, alternate drawings, raw children, and revision
  state at `crates/rdocx-layout/src/engine.rs:2334`. Encountering any such body
  paragraph still disables later reads at
  `crates/rdocx-layout/src/engine.rs:1881`.
- **Contract and public API** produced zero findings. The change broadens one
  private predicate, adds one private helper with two distinct production
  consumers, and updates tests in the existing module. It adds no public item,
  dependency, feature, module, file, trait, generic, or external oracle.
- **Panics and bounds** produced zero findings. The production delta adds no
  indexing, slicing, arithmetic, recursion, unwrap, expect, or allocation-bound
  change. Complete-key equality and the existing bounded paragraph cache remain
  authoritative. The new panic and indexing expressions are confined to
  deterministic test fixture construction at
  `crates/rdocx-layout/src/engine.rs:9254`.
- **OOXML and preservation** produced zero findings. The delta does not change
  parsing, serialization, namespace handling, schema order, raw subtree
  replay, relationships, or package paths. Raw XML remains outside the safe
  paragraph set.
- **Tests and sensitivity** produced zero findings. Footnote and endnote
  700-paragraph cases each require 699 warm hits and one rebuild at
  `crates/rdocx-layout/src/engine.rs:9294`. Reference-ID and note-part changes
  prove exact warm versus fresh behavior at
  `crates/rdocx-layout/src/engine.rs:9322`, and the related-story case proves
  exact output equality at `crates/rdocx-layout/src/engine.rs:9356`.
- **Focused evidence**: the remediated unsafe-prefix regression passed with 1
  test. All five note-reference filtered tests passed. Full
  `cargo test -p rdocx-layout --quiet` passed 219 unit tests and one doctest.
  Crate all-target, all-feature Clippy with warnings denied, workspace format
  check, and `git diff --check` all passed.

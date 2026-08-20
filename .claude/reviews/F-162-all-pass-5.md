# F-162, all, pass 5

**Reviewed**: complete working tree against `HEAD` (`6a60586`), 7 implementation files, 2,151 additions and 58 deletions, excluding review artifacts
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, multi-run sibling edits can still overlap on a shared boundary run

`crates/rdocx-oxml/src/text.rs:2466`

Marker spans solve siblings wholly contained in one run, but the multi-run path
still expands each nested field to its complete start and end runs. Two valid
sibling fields can share a physical boundary run when the first field ends and
the second begins in different children of that run. Their edit ranges then
overlap across the entire shared run. Both ranges validate against the original
source, after which the reverse splice loop at
`crates/rdocx-oxml/src/text.rs:2477` applies them as if they were disjoint. The
earlier replacement restores the old portion of the later sibling or uses an
offset invalidated by its changed result length. The saved field can remain
stale or become malformed. The same-run regression at
`crates/rdocx-oxml/src/text.rs:5404` puts both complete sibling fields in one
run and therefore takes the isolated marker-span branch. Multi-run edits also
need non-overlapping marker-level boundary fragments.

### D2, raw-only edits to a same-run nested instruction are discarded

`crates/rdocx-oxml/src/text.rs:2639`

The isolated same-run rewrite recursively updates descendant sources, then
rewrites only this field's cache and dirty state at
`crates/rdocx-oxml/src/text.rs:2650`. It never applies this nested field's own
effective instruction. If a caller changes only `nested.instruction.raw`, the
outer field can remain in the source-preserving branch because structured
identity is unchanged and nested raw text is not part of the outer raw
instruction. `nested.is_unchanged()` is false, so this helper runs, but it emits
the original instruction bytes. During `update_fields`, evaluation uses the new
raw instruction and can write its result beside the old serialized instruction.
A reopened document then evaluates a different field from the one that produced
the cache. The effective-instruction regression at
`crates/rdocx/src/field.rs:2200` covers a top-level field only, while the new
same-run regression changes caches without editing an instruction.

## Smells

None.

## Nitpicks

None.

## Pass-4 repair verification

- D1 is closed for explicit hyperlink fields. Hyperlink parsing now captures
  inter-run whitespace, comments, and processing instructions. Field-owned raw
  events move into the projected source without duplication, and the aliased
  header regression updates, saves, and preserves producer trivia.
- D2 is closed for two sibling fields wholly contained in the same physical
  run. Direct marker scanning gives them distinct spans, isolated rewriting
  keeps producer children outside each field, reverse edits retain source
  order, and the package regression saves and reopens both updated caches.

## Checks

- `cargo test -p rdocx-oxml text::tests`, passed, 68 tests.
- `cargo test -p rdocx --lib field::tests`, passed, 14 tests.
- `cargo test -p rdocx --test regression_test`, passed, 78 tests.
- `cargo clippy -p rdocx -p rdocx-oxml --all-targets --no-deps -- -D warnings`,
  passed.
- `cargo fmt --all -- --check`, passed.
- `python3 scripts/hash_harness.py --check`, passed, 49 entries unchanged.
- `python3 scripts/sync_agent_skills.py --check`, passed.
- Progress evidence for full crate tests, package dry-run, archive size, and the
  remaining repository gates was inspected.

## Not found

No additional defect was found in hyperlink trivia ownership, all-in-one-run
sibling cache updates, opaque source exclusion, package paragraph span mapping,
source identity for new or foreign replacements, F-161 traversal order,
simple-field mutation, cache and dirty policy, atomic live-state commit, layout
invalidation, update-aware save delegation, leave-alone save APIs, settings and
property preservation, namespace aliases, schema order, public binding scope,
panic safety, HLD scope, or structure. No smells or nitpicks were found.

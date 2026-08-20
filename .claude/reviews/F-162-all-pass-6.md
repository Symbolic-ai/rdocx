# F-162, all, pass 6

**Reviewed**: complete working tree against `HEAD` (`6a60586`), 7 files, 2,498 additions and 67 deletions, excluding review artifacts
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, a raw-only edit of a nested field with its own nested operand cannot serialize

`crates/rdocx-oxml/src/text.rs:2630`

The isolated rewrite recurses into the field's old source before applying that
field's new effective raw instruction. For a raw-only edit,
`crates/rdocx-oxml/src/text.rs:100` deliberately makes
`nested_fields_in_source_order` empty because the parsed raw instruction owns
the effective structure. The source scan still finds the original descendant,
so the count check at `crates/rdocx-oxml/src/text.rs:2444` returns a missing
element error. A three-level complex field triggers this by changing the middle
field's `instruction.raw` from an instruction containing a nested operand to a
plain instruction. Evaluation correctly excludes the old grandchild, but
`update_fields` fails while validating staged serialization instead of writing
the effective middle instruction and removing the grandchild. The new raw-only
regression has producer XML inside the edited field, but no typed descendant,
so it does not exercise this path.

## Smells

None.

## Nitpicks

None.

## Pass-5 repair verification

- D1 is closed for siblings that share a physical boundary run. Each
  replacement now contains only the field-marker span, reverse edits are
  disjoint, producer XML remains between the siblings, and the package
  regression saves and reopens both caches in source order.
- D2 is closed for a same-run nested field without typed descendants. Isolated
  rewriting applies the field's effective raw instruction before cache and
  dirty mutation. The focused unit and facade tests prove the serialized
  instruction, result, producer XML, dirty value, and reopened evaluation.

## Checks

- `cargo test -p rdocx-oxml text::tests`, passed, 70 tests.
- `cargo test -p rdocx --lib field::tests`, passed, 15 tests.
- `cargo test -p rdocx --test regression_test`, passed, 79 tests.
- `cargo clippy -p rdocx -p rdocx-oxml --all-targets --no-deps -- -D warnings`,
  passed.
- `cargo fmt --all -- --check`, passed.
- `python3 scripts/hash_harness.py --check`, passed, 49 entries unchanged.
- `python3 scripts/sync_agent_skills.py --check`, passed, 25 skills in sync.
- `git diff --check HEAD`, passed before this review was written.
- Progress evidence for full crate tests, package dry-run, archive size, and the
  remaining repository gates was inspected.

## Not found

No additional defect was found in shared-boundary marker spans, same-run
raw-only instruction replacement without typed descendants, hyperlink trivia,
opaque source exclusion, package paragraph span mapping, parsed source
identity, F-161 traversal order, simple-field mutation, cache and dirty policy,
atomic live-state commit, layout invalidation, update-aware save delegation,
leave-alone save APIs, settings and property preservation, namespace aliases,
schema order, public binding scope, panic safety, HLD scope, tests, or
structure. No smells or nitpicks were found.

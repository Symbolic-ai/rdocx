# F-108, all aspects, pass 2

**Reviewed**: corrected uncommitted worker diff against `ca5d1e1`, 8 files,
933 insertions and 40 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: the parsed shape-tree root id now participates in duplicate-id
  detection, every package part is scanned for relationship attributes even
  when its relationships collection is absent, and nested groups plus selected
  fallbacks use a heap-backed depth-first traversal.
- Contract: all twelve exact issue variants remain present, validation remains
  observational and deterministic, and both debug save boundaries validate
  before writing.
- Panics: production validation contains no input-dependent `unwrap`, `expect`,
  unchecked indexing, recursive traversal, or unchecked arithmetic.
- OOXML: the relationship and custom-show scans resolve namespace aliases,
  typed serialization retains fixed prefixes and schema order, and preserved
  content is inspected without mutation.
- Tests: the variant gate now covers a root-to-child shape-id collision and an
  XML relationship reference after the entire source `.rels` collection is
  removed. The focused crate suites and all 50 pinned corpus validations pass.
- Structure: no new module, crate, trait, generic abstraction, forwarding
  wrapper, feature flag, or dependency edge was introduced.

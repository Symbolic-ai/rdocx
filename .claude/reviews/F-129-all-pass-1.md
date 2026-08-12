# F-129, all, pass 1

**Reviewed**: working diff from claim base `aba870d`, 8 files, 311 changed
lines, with 287 additions and 24 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, The stale-domain error omits the documented recovery guidance
`crates/oxml-py-support/src/lib.rs:58`

The shared error ends after the structural-invalidation clause. The binding
spec says that a mismatch includes `Re-fetch it with doc.paragraphs[i].`, and
the revised ownership text says the package layer maps the shared domain error
with the same message. A stale paragraph mapped without inventing package-side
text will therefore omit the documented recovery action. The gate test checks
only that both revision numbers occur, so it passes with this incomplete
message.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: no additional wrong logic, off-by-one behavior, or unhandled
  conversion case was found.
- Contract: apart from D1, the implementation matches the approved ownership
  split, Word path inventory, revision support, and HLD impact list.
- Panics: no reachable panic, unchecked index, slice, or untrusted arithmetic
  issue was found. Revision overflow requires exhausting the private monotonic
  `u64` counter.
- OOXML: this diff adds no parser, serializer, namespace, child-order, or raw
  XML behavior.
- Tests: all five focused tests passed. Apart from the missing exact-message
  assertion described in D1, no additional test-gate gap was found.
- Structure: the approved concrete crate, single source module, value types,
  and dependency direction introduce no unjustified trait, generic, wrapper,
  dynamic dispatch, or format-specific dependency.

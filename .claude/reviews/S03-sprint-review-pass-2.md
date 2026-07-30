# S03 sprint review, pass 2

**Reviewed**: `sprint/s03` against
`4e9dbe37488196d203c1986b7cb4cbe298c4415f`, 31 files, 2,952 changed
lines, crates: `oxml-core`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Prior findings

B1 is resolved. The custom-properties parser retains non-`vt` prefixed root
namespace declarations at `crates/oxml-core/src/custom_properties.rs:74`, the
serializer replays them at `crates/oxml-core/src/custom_properties.rs:148`, and
the alternate-prefix regression verifies both the raw bytes and the binding at
`crates/oxml-core/src/custom_properties.rs:408`.

## Milestone gate

The M2 end gate is "hash harness unchanged, and `OpcPackage` opens a real
`.pptx` in a test." The first half holds through the observed 28-entry hash
check. The second half remains assigned to F-018 through F-020 in S04, so S03
does not claim the milestone complete. The sprint-level gates hold through 35
`oxml-core` tests, exact unit assertions, Word and PowerPoint application-
property round-trips, alternate-prefix raw custom-value preservation, package
verification, and the full workspace suite.

## Not found

- Interaction: the staged shared files, new units, and property models compose
  without conflicting ownership or schema-order behaviour.
- Duplication: namespace declaration retention is shared by the two current
  property consumers through one crate-private helper.
- Layering: `oxml-core` depends only on `quick-xml` and `thiserror`, with no
  dependency on either format family.
- Harness: the plans and AS_BUILT entries all declare unchanged output and all
  28 entries match.
- Gate: every S03 feature gate has direct test evidence. The later M2 package
  gate is explicitly not claimed.
- Docs: the HLD describes the staged extraction and 0.0.0 unpublished boundary.
- Dependencies: each direct dependency has a present implementation consumer.
- Surface: no public API lies outside the approved unit and property contracts.

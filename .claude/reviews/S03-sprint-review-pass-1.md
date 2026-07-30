# S03 sprint review, pass 1

**Reviewed**: `sprint/s03` against
`4e9dbe37488196d203c1986b7cb4cbe298c4415f`, 30 files, 2,876 changed
lines, crates: `oxml-core`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, raw custom values can lose their namespace binding

`crates/oxml-core/src/custom_properties.rs:141`

The custom-properties parser accepts an unsupported value under any prefix and
stores its full element bytes as `Raw`, but the serializer rebuilds the root
with only the fixed `xmlns:vt` declaration. A valid input that declares the
variant namespace as `xmlns:v` and contains `<v:ui4>` therefore writes the raw
`v:` subtree beneath a root with no `xmlns:v` binding. The application-
properties implementation already retains extra root namespace declarations,
so the two new preservation paths are inconsistent. The fix must retain or
normalize every namespace binding needed by preserved raw custom values and
add a round-trip regression using a non-`vt` prefix.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M2 end gate is "hash harness unchanged, and `OpcPackage` opens a real
`.pptx` in a test." The first half holds through the observed 28-entry hash
check. The second half is not yet due in S03 because F-018 through F-020 remain
planned for S04, so this review does not claim the milestone complete. The S03
story gates otherwise hold through 35 `oxml-core` tests, including the moved
tests, exact unit assertions, Word and PowerPoint application-property
round-trips, and unknown-subtree preservation.

## Not found

- Interaction: no conflict between the staged F-013 copies and the additive
  F-014 and F-017 implementations beyond B1.
- Duplication: the temporary Word copies are the approved staging mechanism,
  and no additional helper duplication was introduced inside `oxml-core`.
- Layering: `cargo tree -p oxml-core --edges normal` contains only `quick-xml`
  and `thiserror`, with no `rdocx-*` or `rpptx*` dependency.
- Harness: every story declared an unchanged result and all 28 entries match.
- Docs: the architecture, migration, and publishing sections describe the
  staged copy and unpublished 0.0.0 boundary.
- Dependencies: both crate dependencies have direct parser or error consumers.
- Surface: the public modules, units, `Length`, and property models match the
  approved story contracts.

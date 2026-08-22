# F-X047, all aspects, pass 3

**Reviewed**: Revised working-tree diff, 8 implementation and HLD files, 413
changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the implementation earns an unrecorded public-API risk rider

`crates/oxml-layout/src/font.rs:625`

The metrics-only resolver is a new public method on the published
`oxml-layout` crate, even though it is hidden from generated documentation.
The current plan routes only layout and text shaping. Record the additive,
hidden semver impact and the mandatory package dry-run plus archive-size
assertion from the public-API row in risk routing.

### D2, the testing HLD understates the resolved-metrics proof

`docs/hld/12-testing-strategy.md:205`

The revised regression now compares the carrier ascent and descent against
the selected font's resolved metrics, but the HLD records only font and size.
State the complete current assertion so the specification does not permit a
future test weakening that drops the metric comparison.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, contract, panics, OOXML preservation,
tests, or structure.

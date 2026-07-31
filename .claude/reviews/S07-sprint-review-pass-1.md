# S07 sprint review, pass 1

**Reviewed**: `sprint/s07` against
`8c9580e031939f17152d70755fc184b181892b88`, 21 files and 2,049 changed
lines, crates: `oxml-layout`
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

### S1, the group payload test does not prove its stated contract

`crates/oxml-layout/src/output.rs:417`

The test constructs a group with a transform, clip, opacity, and detailed
effect payload, but its assertion checks only opacity and the number of effects.
It would still pass if the transform or clip were discarded, or if the effect
values changed. Strengthen the assertion to inspect every supplied group field
and one child payload, matching the approved test plan.

## Nice-to-have

None.

## Milestone gate

The M4 gate is "hash harness unchanged. This is the milestone where that
matters most." It holds. `python3 scripts/hash_harness.py --check` regenerated
the corpus in deterministic font mode and reported all 28 entries unchanged.
The full workspace tests, both `oxml-layout` feature modes, WASM check,
documentation build, package dry-runs, archive-size check, dependency
inspection, and supply-chain gate also passed.

## Not found

No interaction, duplication, layering, harness, dependency, public-surface, or
HLD drift findings. Released `rdocx-*` source and manifests are unchanged, and
`oxml-layout` remains version 0.0.0 with publication disabled.

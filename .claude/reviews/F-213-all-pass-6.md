# F-213, all, pass 6

**Reviewed**: fully remediated working tree against `0bf3aa2`, 5
implementation files and 3,515 changed lines. The scope is 765 tracked
additions, 21 tracked deletions, and all 2,729 lines of the approved untracked
`crates/rpptx-oxml/src/timing.rs` module. The pass-1 through pass-5 review
artifacts are excluded from the implementation count
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, unsupported direct target choices do not consume the singleton

`crates/rpptx-oxml/src/timing.rs:1261`
`crates/rpptx-oxml/src/timing.rs:1317`

The condition parser consumes direct `p:tgtEl` and `p:tn` choices but ignores
the standard unsupported direct `p:rtn` choice. The target-element parser
likewise consumes only `p:spTgt` and `p:sldTgt`, while standard direct
`p:sndTgt` and `p:inkTgt` choices return no projection. A later supported
choice is therefore accepted as the typed target. For example,
`p:cond/p:rtn` followed by `p:tn` becomes an executable time-node trigger, and
`p:tgtEl/p:sndTgt` followed by `p:spTgt` becomes an executable shape target.
Both inputs contain two members of a schema choice whose maximum cardinality
is one. Unsupported direct choice members must occupy the singleton as
`TimingTarget::Unsupported`, while nested occurrences inside an unsupported
wrapper must continue to remain nonprojecting raw XML.

### D2, direct attribute-name CDATA is ignored

`crates/rpptx-oxml/src/timing.rs:2150`

`direct_text` accumulates only `Event::Text` and discards `Event::CData` in its
catch-all arm. A valid direct
`p:attrName><![CDATA[style.visibility]]></p:attrName` is consequently exposed
as no attribute name, even though CDATA and ordinary character data have the
same element-content meaning. F-214 would be unable to apply that supported
behavior after F-213 reported an incomplete typed projection. Direct CDATA
must contribute to the same trimmed text value without reading text from a
nested unsupported element.

## Smells

None. 0 smells.

## Nitpicks

None. 0 nitpicks.

## Pass-5 closure verification

- **Pass-5 D1 closed**: the morph mutator now tests direct paired and empty
  effect elements before compatibility handling, captures every other direct
  subtree without searching it, and returns an error instead of falling back
  to a recursive search when no selected morph exists
  (`crates/rpptx-oxml/src/timing.rs:1640`,
  `crates/rpptx-oxml/src/timing.rs:1675`,
  `crates/rpptx-oxml/src/timing.rs:1689`). The regression preserves the exact
  unsupported wrapper, changes the direct typed morph, and reparses the
  requested value (`crates/rpptx-oxml/tests/integration.rs:390`).

## Not found

- **Correctness and contract, 0 additional findings**: timing and transition
  projection, compatibility selection, supported mutation, and atomic commit
  behavior are consistent with the approved F-213 contract outside D1 and D2.
- **Panics, 0 findings**: parser-state `expect` calls, guarded indexing, byte
  ranges, and arithmetic remain protected by established state, counted
  matches, or validated reader positions. No reachable untrusted-input panic
  was found.
- **OOXML order and preservation, 0 additional findings**: modeled root,
  timing, behavior, and transition child order is enforced. The reconciled
  narrow layout-marker policy remains bounded, and unsupported XML remains
  byte-preserved outside the typed-boundary errors in D1 and D2.
- **Tests, 0 independent findings**: the pass-5 morph regression would fail if
  its fix were reverted and the five approved story gates remain present. D1
  and D2 identify missing contract cases rather than separate test-only
  defects.
- **Structure, 0 findings**: the approved module remains concrete and local.
  It adds no trait, generic parameter, forwarding wrapper, feature, crate, or
  dependency.
- **Harness and API scope, 0 findings**: there is no baseline change, facade
  execution API, renderer behavior, binding API, production dependency, or
  feature expansion.
- **Verification execution**: no broad test suite was run in this audit, as
  requested. The verdict is based on the complete working diff and cited
  source and test inspection.

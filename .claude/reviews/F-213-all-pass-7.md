# F-213, all, pass 7

**Reviewed**: fully remediated working tree against `0bf3aa2`, 5
implementation files and 3,552 changed lines. The scope is 797 tracked
additions, 21 tracked deletions, and all 2,734 lines of the approved untracked
`crates/rpptx-oxml/src/timing.rs` module. The pass-1 through pass-6 review
artifacts are excluded from the implementation count
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, direct attribute-name entity references are dropped

`crates/rpptx-oxml/src/timing.rs:2164`

`direct_text` now accumulates ordinary text and CDATA, but its catch-all arm
still ignores `Event::GeneralRef`. With the workspace quick-xml version, each
numeric or predefined entity reference is delivered as that separate event.
A valid direct
`p:attrName>style&#x2E;visibility</p:attrName` therefore projects as
`stylevisibility` rather than the XML-equivalent `style.visibility`. F-214 can
then apply the behavior to the wrong name or fail to apply it. Direct general
references must contribute their resolved character value while references
inside a captured unsupported child must remain excluded.

## Smells

None. 0 smells.

## Nitpicks

None. 0 nitpicks.

## Pass-6 closure verification

- **Pass-6 D1 closed**: the condition helper now maps direct `p:rtn` to an
  unsupported target that consumes the condition singleton, and the target
  helper does the same for direct `p:sndTgt` and `p:inkTgt`. Nested occurrences
  remain nonprojecting because direct children are captured before parsing
  continues (`crates/rpptx-oxml/src/timing.rs:1255`,
  `crates/rpptx-oxml/src/timing.rs:1312`,
  `crates/rpptx-oxml/src/timing.rs:1345`,
  `crates/rpptx-oxml/src/timing.rs:1364`). The regression proves unsupported
  singleton projection and rejects mixed unsupported and supported choices
  (`crates/rpptx-oxml/tests/integration.rs:240`).
- **Pass-6 D2 closed for CDATA**: direct CDATA is accumulated with ordinary
  text, while CDATA inside a captured unsupported child is not projected
  (`crates/rpptx-oxml/src/timing.rs:2147`,
  `crates/rpptx-oxml/src/timing.rs:2155`,
  `crates/rpptx-oxml/tests/integration.rs:240`). Pass-7 D1 records the distinct
  general-reference event that remains unhandled.

## Not found

- **Correctness and contract, 0 additional findings**: timing and transition
  projection, compatibility selection, supported mutation, and atomic commit
  behavior are consistent with the approved F-213 contract outside D1.
- **Panics, 0 findings**: parser-state `expect` calls, guarded indexing, byte
  ranges, and arithmetic remain protected by established state, counted
  matches, or validated reader positions. No reachable untrusted-input panic
  was found.
- **OOXML order and preservation, 0 additional findings**: modeled root,
  timing, behavior, and transition child order is enforced. The reconciled
  narrow layout-marker policy remains bounded, and unsupported XML remains
  byte-preserved outside the typed text projection in D1.
- **Tests, 0 independent findings**: the two pass-6 regressions would fail if
  their fixes were reverted and the five approved story gates remain present.
  D1 identifies a missing direct-text event case rather than a separate
  test-only defect.
- **Structure, 0 findings**: the approved module remains concrete and local.
  It adds no trait, generic parameter, forwarding wrapper, feature, crate, or
  dependency.
- **Harness and API scope, 0 findings**: there is no baseline change, facade
  execution API, renderer behavior, binding API, production dependency, or
  feature expansion.
- **Verification execution**: no broad test suite was run in this audit, as
  requested. `git diff --check` passed. The verdict is otherwise based on the
  complete working diff and cited source and test inspection.

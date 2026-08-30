# F-213, all, pass 5

**Reviewed**: fully remediated working tree against `0bf3aa2`, 5
implementation files and 3,467 changed lines. The scope is 741 tracked
additions, 21 tracked deletions, and all 2,705 lines of the approved untracked
`crates/rpptx-oxml/src/timing.rs` module. The pass-1 through pass-4 review
artifacts are excluded from the implementation count
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, direct morph mutation can rewrite an unsupported nested morph

`crates/rpptx-oxml/src/timing.rs:1674`
`crates/rpptx-oxml/src/timing.rs:2384`

The special morph rewriter inspects direct children only for an
`mc:AlternateContent` wrapper. If it does not find one, it falls back to the
generic recursive attribute search. That search accepts the first matching
expanded name at any depth. For example, a transition containing
`x:wrapper/p159:morph` before a direct `p159:morph` projects the direct morph as
the typed effect, but `set_morph_option` rewrites the nested producer-owned
morph and leaves the typed direct morph unchanged. The reparsed replacement
then commits successfully. The requested supported mutation therefore does
not take effect and an unsupported subtree loses byte identity. The direct
transition path must select the same direct modeled effect that parsing
selected, without recursively crossing an unsupported child.

## Smells

None. 0 smells.

## Nitpicks

None. 0 nitpicks.

## Pass-4 closure verification

- **Pass-4 D1 formally withdrawn**: the pinned producer corpus establishes the
  narrow read exception as part of the approved contract reconciliation. The
  implementation recognizes only an attribute-free empty layout transition,
  only when it is immediately followed by `p:hf`, and writes the modeled
  children in canonical `p:hf`, `p:transition` order
  (`crates/rpptx-oxml/src/slide_parts.rs:858`,
  `crates/rpptx-oxml/src/slide_parts.rs:893`,
  `crates/rpptx-oxml/src/slide_parts.rs:943`). Attributed, paired, and
  intervening-child forms remain rejected by focused tests
  (`crates/rpptx-oxml/tests/integration.rs:298`,
  `crates/rpptx-oxml/tests/integration.rs:323`).
- **Pass-4 D2 closed**: direct condition `p:tn` projection and target-element
  projection now use separate helpers, and `p:tgtEl` recognizes only its
  modeled shape and slide targets (`crates/rpptx-oxml/src/timing.rs:1240`,
  `crates/rpptx-oxml/src/timing.rs:1305`,
  `crates/rpptx-oxml/src/timing.rs:1337`).
- **Pass-4 D3 closed**: unsupported target descendants are captured but do not
  become typed targets or reject the retained fragment
  (`crates/rpptx-oxml/src/timing.rs:1252`,
  `crates/rpptx-oxml/src/timing.rs:1315`,
  `crates/rpptx-oxml/tests/integration.rs:219`).
- **Pass-4 D4 closed**: direct `p:to` elements and their modeled variant
  choices are accumulated through singleton checks rather than early return
  (`crates/rpptx-oxml/src/timing.rs:992`,
  `crates/rpptx-oxml/src/timing.rs:1030`,
  `crates/rpptx-oxml/tests/integration.rs:179`).
- **Pass-4 D5 closed**: attribute-name projection now selects a direct
  PresentationML `p:attrName`, excludes nested producer text, and trims direct
  formatting text (`crates/rpptx-oxml/src/timing.rs:2078`,
  `crates/rpptx-oxml/src/timing.rs:2112`,
  `crates/rpptx-oxml/tests/integration.rs:219`).
- **Pass-4 D6 closed**: selected compatibility branches enforce transition and
  effect cardinality, and duration mutation traverses only modeled timing-node
  ownership paths (`crates/rpptx-oxml/src/timing.rs:1501`,
  `crates/rpptx-oxml/src/timing.rs:1945`,
  `crates/rpptx-oxml/src/timing.rs:2141`,
  `crates/rpptx-oxml/tests/integration.rs:137`,
  `crates/rpptx-oxml/tests/integration.rs:268`). Pass-5 D1 concerns the
  separate direct morph mutation path.

## Not found

- **Correctness and contract, 0 additional findings**: typed timing and
  transition projections, singleton enforcement, order validation, and
  mutation atomicity are consistent with the approved F-213 contract outside
  D1.
- **Panics, 0 findings**: parser-state `expect` calls, the guarded layout
  `unreachable`, and range operations remain protected by established state or
  prior validation. No reachable untrusted-input panic or arithmetic defect
  was found.
- **OOXML order and preservation, 0 additional findings**: modeled root and
  transition child order is enforced, the reconciled layout marker is
  canonicalized, and unsupported XML remains raw outside the D1 morph search.
- **Tests, 0 independent findings**: focused tests cover the pass-4 closure
  cases and the reconciled layout policy. D1 identifies the missing
  unsupported-descendant morph case rather than a separate test-only defect.
- **Structure, 0 findings**: the approved module remains concrete and local.
  It adds no trait, generic parameter, forwarding wrapper, feature, crate, or
  dependency.
- **Harness and API scope, 0 findings**: there is no baseline change, facade
  execution API, renderer behavior, binding API, production dependency, or
  feature expansion.
- **Verification execution**: no broad test suite was run in this audit, as
  requested. The verdict is based on the complete working diff and cited
  source and test inspection.

# F-213, all, pass 3

**Reviewed**: fully remediated working tree against `0bf3aa2`, 5
implementation files and 2,734 changed lines. The scope is 574 tracked
additions, 18 tracked deletions, and all 2,142 lines of the approved untracked
`crates/rpptx-oxml/src/timing.rs` module. The pass-1 and pass-2 review artifacts
are excluded from the implementation count
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, an empty layout transition is kept outside the typed transition slot

`crates/rpptx-oxml/src/slide_parts.rs:934`

The layout-only empty-marker branch pushes a valid direct `p:transition` into
raw children and leaves `CT_SlideLayout.transition` as `None`. This contradicts
the approved replacement of the raw layout transition slot with an optional
typed field and loses the observable distinction between no transition and an
explicit empty transition. The branch also neither marks the transition as
seen nor advances the layout boundary. A second transition is therefore
accepted, and `p:transition` followed by the earlier `p:timing` slot is accepted
and emitted in the same schema-invalid order. Empty syntax must not change the
typed root contract or its sequence validation.

### D2, transition-level AlternateContent ignores branch capability selection

`crates/rpptx-oxml/src/timing.rs:1632`
`crates/rpptx-oxml/src/timing.rs:1684`

A direct `mc:AlternateContent` effect child is classified by recursively
searching every branch for any `p159:morph`. This path does not inspect
`mc:Choice/@Requires`, select the first supported choice, or use the fallback.
For example, a morph in an unsupported choice overrides a supported fallback
fade and is exposed to F-214 as `TransitionEffect::Morph`. Conversely, an
ordinary effect in the selected branch becomes `UnsupportedEffect` when no
morph appears anywhere. The root compatibility-wrapper path now selects a
capability-correct transition, but the effect choice within that transition
still exposes and mutates the wrong branch.

### D3, required outer timing-node children are found recursively without sequence validation

`crates/rpptx-oxml/src/timing.rs:706`
`crates/rpptx-oxml/src/timing.rs:823`

`parse_behavior` returns the first descendant `p:cBhvr`, so a behavior with
`p:to` before `p:cBhvr`, or with `p:cBhvr` hidden under producer content, still
becomes a supported set, animate, effect, or motion node. It also never checks
for a second direct common behavior. `parse_container_common` has the same
recursive first-match behavior for the required `p:cTn`, which accepts nested
or duplicate common nodes in a parallel container. The inner common-behavior
and common-time-node parsers now validate their own children, but the declared
outer model can still be typed from schema-invalid descendants and can disagree
with the retained XML.

### D4, condition target choices omit valid time-node triggers and accept invalid nesting

`crates/rpptx-oxml/src/timing.rs:1201`
`crates/rpptx-oxml/src/timing.rs:1231`

The condition parser recognizes only direct `p:tgtEl` children. It never parses
the schema-valid direct `p:tn val="..."` trigger even though the public model
declares `TimingTarget::TimeNode`. The target helper then searches recursively,
so the time-node variant is reachable only through an invalid nested shape such
as `p:tgtEl/x:wrapper/p:tn`. Repeated direct targets also overwrite the previous
projection instead of failing the condition choice singleton. F-214 would lose
valid time-node dependencies while potentially executing malformed producer
content.

### D5, an unqualified producer duration becomes the Office 2010 duration

`crates/rpptx-oxml/src/timing.rs:383`
`crates/rpptx-oxml/src/timing.rs:412`

Both transition parse paths fall back from the namespaced Office 2010
`p14:dur` attribute to an unqualified `dur`. Unqualified `dur` is not the
extension attribute and must remain unsupported producer metadata. A fragment
such as `p:transition dur="725"` is therefore exposed as `duration_ms = 725`,
violating the expanded-name boundary and giving F-214 executable timing from an
unknown attribute. Namespace aliases for the `p14` URI are already handled by
`namespaced_attribute`, so the unqualified fallback is neither needed nor
safe.

## Smells

None. 0 smells.

## Nitpicks

None. 0 nitpicks.

## Pass-2 closure verification

- **Pass-2 D1 closed at the root wrapper**: root `mc:AlternateContent`
  selection now checks every required prefix, chooses the first supported
  choice or fallback, requires a direct transition, and rewrites only that
  selected fragment. D2 covers the distinct transition-effect wrapper path.
- **Pass-2 D2 closed**: set values now require a direct `p:to` child and a
  direct PresentationML animation variant.
- **Pass-2 D3 closed inside `p:cBhvr`**: common behavior children now reject
  duplicates and backward schema order. D3 covers the still-unchecked outer
  behavior and container boundaries.
- **Pass-2 D4 closed for direct transition children**: the effect choice,
  sound action, and extension list now enforce singleton and schema order. D2
  covers capability selection inside a compatibility effect child.
- **Pass-2 D5 closed for modeled root children**: modeled children encountered
  after a transition now reject backward root order. D1 covers the layout-only
  empty-transition bypass.
- **Pass-2 D6 closed**: the corpus gate now asserts nonzero set value, effect
  parameter, and compatibility-transition counters.

## Not found

- **Panics, 0 findings**: the added indexing and internal `expect` calls remain
  guarded by established parser state or counted matches. No reachable
  untrusted-input panic or arithmetic defect was found.
- **Structure, 0 findings**: the approved concrete module adds no trait,
  generic parameter, forwarding wrapper, feature, crate, or dependency.
- **Raw preservation and mutation atomicity, 0 additional findings**:
  unsupported bytes remain the serialization source, and mutations stage and
  reparse replacement bytes before changing live state. D2 concerns which
  compatibility branch is projected, not loss of the retained wrapper.
- **Tests, 0 independent findings**: the focused remediations directly cover
  the six pass-2 cases. The five defects above identify their missing adjacent
  contract cases rather than a separate test-only failure.
- **Harness and API scope, 0 findings**: there is no baseline change, facade
  execution API, renderer behavior, binding API, production dependency, or
  feature expansion.
- **Verification execution**: no broad test suite was run in this audit, as
  requested. The verdict is based on the complete working diff and cited
  source and test inspection.

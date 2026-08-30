# F-213, all, pass 4

**Reviewed**: fully remediated working tree against `0bf3aa2`, 5
implementation files and 3,143 changed lines. The scope is 637 tracked
additions, 18 tracked deletions, and all 2,488 lines of the approved untracked
`crates/rpptx-oxml/src/timing.rs` module. The pass-1 through pass-3 review
artifacts are excluded from the implementation count
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, empty layout transitions still bypass one root sequence check

`crates/rpptx-oxml/src/slide_parts.rs:928`

The pass-3 fix now keeps an empty layout transition typed, rejects a second
transition, and rejects a following timing node. It also adds a
`legacy_layout_order` exception that accepts `p:hf` after the transition only
when that transition uses empty-element syntax. The writer then moves the
header and footer before the transition. A nonempty transition in the same
position correctly fails. Empty versus paired syntax cannot change the layout
schema, and an input must not receive a special backward-order path that
silently canonicalizes modeled children. Pass-3 D1 is therefore only partially
closed.

### D2, the condition and behavior target choices still share incompatible boundaries

`crates/rpptx-oxml/src/timing.rs:1325`
`crates/rpptx-oxml/src/timing.rs:1398`

A schema-valid direct condition `p:tn` is now typed and repeated condition
targets fail. However, `parse_target` still calls the same helper for children
of `p:tgtEl`, and that helper recognizes `p:tn`. Consequently the invalid
behavior shape `p:tgtEl/p:tn` is still exposed as
`TimingTarget::TimeNode`, even though a time-node trigger belongs directly to
the condition choice and is not a target-element choice. F-214 can therefore
receive an executable target from malformed behavior XML. The condition and
target-element choices need distinct projection helpers.

### D3, nested targets inside unsupported siblings now reject otherwise preservable XML

`crates/rpptx-oxml/src/timing.rs:1264`
`crates/rpptx-oxml/src/timing.rs:1319`

When an unmodeled direct child contains a PresentationML target at a deeper
level, both the condition and target-element parsers now return an error. The
expanded-name boundary only requires that such a descendant not become the
typed target. The approved preservation contract requires unsupported timing
siblings and their relationship-bearing subtrees to remain raw. For example,
`p:cond/x:producer/p:tn` should leave the condition target unsupported while
the retained timing bytes survive, not make the entire slide unreadable. The
new malformed-input test codifies rejection where lossless nonprojection is
required.

### D4, set value choices accept duplicate modeled children

`crates/rpptx-oxml/src/timing.rs:1009`
`crates/rpptx-oxml/src/timing.rs:1043`

`parse_set_value` returns immediately after the first direct `p:to`, and
`parse_animation_variant` returns immediately after its first direct variant.
A set with two `p:to` children, or a `p:to` containing both `p:strVal` and
`p:intVal`, therefore parses successfully and exposes only the first value
while retaining both in raw XML. This leaves the typed value inconsistent with
a schema-invalid choice and breaks the same duplicate and sequence contract
now enforced for common behavior children.

### D5, attribute-name projection ignores expanded names and formatting text

`crates/rpptx-oxml/src/timing.rs:791`
`crates/rpptx-oxml/src/timing.rs:2085`

The common behavior parser populates `attribute_name` with the first text event
anywhere in `p:attrNameLst`. It does not require a direct PresentationML
`p:attrName` owner. Leading indentation can therefore become the attribute
name, and text inside a foreign producer child can become an executable name
such as `style.visibility`. This violates the expanded-name boundary and can
make F-214 apply a behavior to the wrong property. The bounded projection must
select text from a direct modeled child and ignore formatting or unsupported
subtrees.

### D6, compatibility branches and mutation searches still cross unsupported boundaries

`crates/rpptx-oxml/src/timing.rs:1537`
`crates/rpptx-oxml/src/timing.rs:2017`
`crates/rpptx-oxml/src/timing.rs:2103`

The compatibility helpers return the first direct transition or effect from a
selected branch without checking whether the branch contains another modeled
choice. A selected branch with two transitions or two effects is accepted and
projected as the first even though markup-compatibility preprocessing would
place both at the parent and violate its singleton. Separately,
`set_node_duration` recursively searches every descendant `p:cTn`, including
ones inside a `TimingUnsupported` subtree. A request can therefore rewrite an
unsupported nested node, or fail because such a node duplicates a valid typed
id. Both paths violate the contract that schema choices are enforced and a
supported mutation rewrites only its owning modeled element while unsupported
subtrees remain byte-identical.

## Smells

None. 0 smells.

## Nitpicks

None. 0 nitpicks.

## Pass-3 closure verification

- **Pass-3 D1 partially closed**: empty layout transitions are typed and now
  participate in duplicate and timing-order checks. D1 records the remaining
  header and footer exception.
- **Pass-3 D2 base case closed**: transition-level compatibility selection now
  respects `Requires`, supported choices, and fallback, and morph mutation
  selects the supported branch. D6 records the remaining selected-branch
  cardinality and unsupported-node mutation boundary.
- **Pass-3 D3 closed**: behavior `p:cBhvr` and parallel-container `p:cTn`
  children are now direct, singleton, and ordered. D4 covers the distinct set
  value choice after the common behavior.
- **Pass-3 D4 partially closed**: valid direct condition time-node targets and
  duplicate condition choices now work as intended. D2 and D3 record the
  remaining target-element conflation and preservation regression.
- **Pass-3 D5 closed**: only the Office 2010 namespace URI supplies transition
  duration, including aliases. An unqualified producer `dur` remains raw and
  does not populate `duration_ms`.

## Not found

- **Panics, 0 findings**: internal `expect` calls and indexing remain guarded
  by established parser state, validated ranges, or counted matches. No
  reachable untrusted-input panic or arithmetic defect was found.
- **Structure, 0 findings**: the approved module remains concrete and local.
  It adds no trait, generic parameter, forwarding wrapper, feature, crate, or
  dependency.
- **Root breadth, 0 additional findings**: slides, layouts, and masters expose
  typed timing and transition fields, and their ordinary modeled child order is
  enforced outside D1.
- **Raw serialization and atomicity, 0 additional findings**: raw timing and
  transition bytes remain the serialization source, and live state changes
  only after replacement bytes reparse successfully. D3 and D6 concern target
  selection boundaries, not partial live-state updates.
- **Tests, 0 independent findings**: the focused remediations exercise the five
  pass-3 examples. D1 through D6 cite adjacent contract cases that those tests
  do not cover rather than a separate test-only failure.
- **Harness and API scope, 0 findings**: there is no baseline change, facade
  execution API, renderer behavior, binding API, production dependency, or
  feature expansion.
- **Verification execution**: no broad test suite was run in this audit, as
  requested. The verdict is based on the complete working diff and cited
  source and test inspection.

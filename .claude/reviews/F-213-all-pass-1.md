# F-213, all, pass 1

**Reviewed**: complete working tree against `0bf3aa2`, 5 implementation
files and 1,629 changed lines, including 1,371 lines in the approved untracked
`crates/rpptx-oxml/src/timing.rs` module
**Verdict**: 9 defects, 0 smells, 0 nitpicks

## Defects

### D1, set behavior values are read from the wrong schema location

`crates/rpptx-oxml/src/timing.rs:570`

`TimingSet.value` is populated from a `to` attribute on `p:set`. A conforming
set behavior stores its value in the required `p:to` animation-variant child,
for example `p:to/p:strVal/@val`. Real corpus visibility sets therefore parse
with `value == None`. F-214 cannot execute the supported set behavior even
though the original XML remains preserved.

### D2, sequence navigation triggers are not represented

`crates/rpptx-oxml/src/timing.rs:112`

`TimingSequence` exposes only the common time node, concurrency, and next
action. PresentationML stores the main sequence's previous and next triggers
in `p:prevCondLst` and `p:nextCondLst` siblings of `p:cTn`. The parser ignores
both lists. Common corpus main sequences consequently lose their `onPrev` and
`onNext` typed conditions, which violates the approved trigger model and
leaves F-214 without the events that advance the sequence.

### D3, builds cannot be linked to their timing groups

`crates/rpptx-oxml/src/timing.rs:92`
`crates/rpptx-oxml/src/timing.rs:176`

`p:bldP/@grpId` is projected on `TimingBuild`, but
`p:cTn/@grpId` is discarded from `CommonTimeNode`. The build and the timing
effect group therefore cannot be joined. The build projection also omits the
paragraph build mode, level, reverse flag, and related ordering metadata. A
vector of build entries does not recover paragraph ordering within a build.
This does not satisfy the plan's promise that build entries retain target and
paragraph ordering for the main timing tree.

### D4, transition parameters and extension duration are discarded

`crates/rpptx-oxml/src/timing.rs:282`
`crates/rpptx-oxml/src/timing.rs:340`

The effect projection keeps only a small enum selected from the element local
name. Parameters such as wipe direction, split orientation and direction, and
other effect attributes are lost from the typed model. The duration parser
looks only for an unqualified `dur` attribute, while producer XML uses the
Office 2010 `p14:dur` extension attribute. F-214 therefore cannot select and
evaluate the promised transition subset from the typed values, even when the
raw XML contains all required data.

### D5, compatibility-wrapped transitions never become typed transitions

`crates/rpptx-oxml/src/slide_parts.rs:928`
`crates/rpptx/tests/integration.rs:14`

The slide-like root parser recognizes `p:transition` only when it is a direct
root child. Modern producer decks commonly put the chosen `p:transition`, its
`p14:dur`, or its morph choice inside a root-level `mc:AlternateContent`.
Those transitions remain entirely raw and `slide.transition` is `None`. The
package test constructs the inverse nesting, an `mc:AlternateContent` inside a
direct `p:transition`, so it passes without proving the producer shape that the
feature is required to expose. The full compatibility subtree can remain the
serialization source while its selected transition is projected.

### D6, timing schema order and singleton constraints are not enforced

`crates/rpptx-oxml/src/timing.rs:432`
`crates/rpptx-oxml/src/timing.rs:737`

The root parser records whether `p:tnLst` and `p:bldLst` were seen, but it
accepts `p:bldLst` before `p:tnLst`. The common time-node parser similarly
overwrites repeated condition or child lists and does not reject modeled
children in the wrong `xsd:sequence` order. Serialization returns the retained
raw subtree, so it also emits the invalid order unchanged. This contradicts
the approved reader and writer sequence contract. The malformed-input test
checks one duplicate root list but contains no out-of-order case.

### D7, harmless namespace shadows make preserved timing XML unopenable

`crates/rpptx-oxml/src/slide_parts.rs:929`

Before storing a timing or transition subtree, the root parser rejects any
local binding of `p`, `a`, or `r` that differs from the fixed writer mapping.
The new timing types write the original raw bytes rather than synthesizing
those descendants, so a local `xmlns:a` or `xmlns:r` used only by unsupported
producer content is safe and must remain opaque. Rejecting it turns a
previously preservable slide into an open failure, contrary to the explicit
requirement to retain every unsupported namespace binding and subtree.

### D8, modeled mutations normalize unsupported attributes on the owner

`crates/rpptx-oxml/src/timing.rs:1228`

Every duration, speed, or morph mutation rebuilds the matched start tag from
decoded and normalized attribute strings. Unsupported attributes and namespace
declarations on that same element can therefore change quote style, character
reference spelling, and normalized whitespace. For example, an untouched
relationship attribute written with a character reference is emitted with its
decoded value. The preservation test places relationship attributes only on
unmatched sibling elements, so it does not exercise the owning element whose
start tag is rebuilt. This violates the byte-exact unsupported-attribute
contract for a supported mutation.

### D9, the corpus gate does not inspect the declared model or unsupported siblings

`crates/rpptx-oxml/tests/integration.rs:153`

For every corpus timing or transition, the gate asserts only that the retained
raw byte vector is nonempty. It never asserts that the timeline has typed
nodes, builds, triggers, set values, effect parameters, or compatibility
transitions. It also never inventories unsupported children before and after
serialization. A parser that returned empty typed projections while retaining
the root raw bytes would pass. The loop filters out layouts and masters, so it
does not exercise the other two roots modified by this feature. The named
round-trip gate therefore does not prove that the corpus timeline parses into
the declared model or preserves every unsupported sibling in its slot.

## Smells

None. 0 smells.

## Nitpicks

None. 0 nitpicks.

## Not found

- **Panics, 0 findings**: the internal `expect` calls reviewed are guarded by
  parser state established before the corresponding branch. No reachable
  panic, unchecked arithmetic, slice, or index defect was found for untrusted
  timing XML.
- **Structure, 0 findings**: the new module was explicitly approved. It adds
  no trait, generic parameter, wrapper-only abstraction, feature, crate, or
  dependency.
- **Root child order, 0 additional findings**: direct typed timing and
  transition fields are emitted in the existing slide, layout, and master
  schema slots. D5 covers the missing compatibility-wrapped projection and D6
  covers order inside timing XML.
- **Morph namespace selection, 0 additional findings**: morph parsing and
  mutation use the Office 2015 namespace URI rather than matching a foreign
  element by local name alone. D5 covers the unmodeled root compatibility
  shape.
- **Atomic mutation, 0 additional findings**: typed values are replaced only
  after the rewritten raw fragment reparses successfully. Failed node lookup
  and reparse leave the live value unchanged.
- **Harness and dependencies, 0 findings**: the implementation changes no
  hash baseline, production dependency, feature flag, facade execution API,
  binding API, or renderer behavior.

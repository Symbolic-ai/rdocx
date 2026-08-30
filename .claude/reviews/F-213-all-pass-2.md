# F-213, all, pass 2

**Reviewed**: fully remediated working tree against `0bf3aa2`, 5
implementation files and 2,284 changed lines, including 1,812 lines in the
approved untracked `crates/rpptx-oxml/src/timing.rs` module. The pass-1 review
artifact is excluded from the implementation count
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, AlternateContent selection ignores the Requires contract

`crates/rpptx-oxml/src/timing.rs:1255`

`selected_transition_fragment` returns the first PresentationML transition at
any depth in the compatibility wrapper. It does not inspect which
`mc:Choice/@Requires` prefixes the reader supports, select the first supported
choice, or fall back when a choice requires an unknown namespace. A choice
that requires an unsupported extension can therefore become the typed and
mutable transition instead of the fallback that this reader can interpret.
The same unrestricted descendant search can classify an unrelated root
compatibility wrapper merely because it contains a nested `p:transition`.
`set_speed` then rewrites the first transition rather than a capability-selected
branch. The raw wrapper remains intact, but the public projection and mutation
target can both be wrong.

### D2, a foreign set variant becomes a supported set value

`crates/rpptx-oxml/src/timing.rs:904`

Once `parse_set_value` has seen any descendant `p:to`, it returns the first
descendant carrying an unqualified `val` attribute without checking that the
element is an immediate PresentationML animation-variant choice. For example,
`p:to/x:producer[@val='visible']` is exposed as the supported value
`visible`. A nested `p:to` inside unrelated content is also accepted. This
violates the expanded-name model boundary and can make F-214 execute producer
content that should remain unsupported raw XML.

### D3, common behavior sequence violations are still accepted

`crates/rpptx-oxml/src/timing.rs:710`

The common behavior parser overwrites repeated `p:cTn`, `p:tgtEl`, and
`p:attrNameLst` children and does not enforce their sequence. A behavior with
the target before its common node, or with two targets, parses successfully
and is emitted unchanged even though the typed projection reflects only the
last occurrence. Since set, animate, effect, and motion all use this parser,
the pass-1 schema remediation does not yet enforce `xsd:sequence` across the
declared behavior model.

### D4, multiple transition effects silently select the last one

`crates/rpptx-oxml/src/timing.rs:1332`

The transition parser assigns `effect` and its parameters every time it sees a
direct PresentationML effect child. It does not reject a second child from the
effect choice or enforce the remaining transition child sequence. An invalid
`p:transition` containing both `p:wipe` and `p:push` is accepted, projected as
push, and serialized with both effects. This breaks the approved schema-order
contract and gives F-214 a typed value that disagrees with the retained XML.

### D5, a transition before an earlier root child is accepted and reordered

`crates/rpptx-oxml/src/slide_parts.rs:891`
`crates/rpptx-oxml/src/slide_parts.rs:971`

`capture_transition` verifies that already-seen children have not passed the
transition slot. Earlier modeled children parsed afterward still use
`boundary.max(...)` without rejecting the backward move. A slide containing
`p:transition` before `p:clrMapOvr`, or a layout containing transition before
`p:hf`, therefore parses successfully. The writer moves the earlier child in
front of the transition rather than rejecting the malformed model. F-213's
root integration must fail these out-of-order modeled children before
serialization changes their order.

### D6, three corpus remediation counters do not gate the test

`crates/rpptx-oxml/tests/integration.rs:245`

The remediated corpus walk records `set_values`, `effect_parameters`, and
`compatibility_transitions`, but the final assertions stop after builds and
unsupported nodes. The gate still passes if every corpus set loses its value,
every parameterized transition loses its parameters, or every root
compatibility transition remains raw. These are precisely the pass-1 gaps the
new counters were added to detect. Assert nonzero coverage for all three so
the named corpus gate proves the remediated declared model rather than only
printing the missing coverage in debug output.

## Smells

None. 0 smells.

## Nitpicks

None. 0 nitpicks.

## Not found

- **Pass-1 D1, base case closed**: conforming `p:to` animation variants now
  populate `TimingSet.value`, and the focused test asserts the real child
  shape. D2 covers the remaining expanded-name boundary.
- **Pass-1 D2 closed**: sequences expose and parse previous and next condition
  lists, including `onPrev` and `onNext`.
- **Pass-1 D3 closed**: common nodes retain `grpId`, and builds expose the
  target, group, paragraph mode, level, reverse, automatic advance, background,
  and UI metadata required to join the main timing tree.
- **Pass-1 D4, supported fields closed**: aliased Office 2010 duration and
  direct effect parameters are exposed. D4 covers duplicate effect choices,
  not the successful supported projection.
- **Pass-1 D5, producer shape recognized**: a root-level compatibility wrapper
  can retain its complete bytes while exposing and mutating the contained
  transition. D1 covers capability-correct branch selection.
- **Pass-1 D6, timing root and common node closed**: root lists, common-node
  children, duplicates, and backward order now fail with focused regression
  cases. D3 through D5 identify the remaining modeled sequence boundaries.
- **Pass-1 D7 closed**: timing and transition raw-backed parsing no longer
  rejects harmless local `a` or `r` namespace shadows.
- **Pass-1 D8 closed**: mutations replace only the selected attribute value in
  the original start-tag bytes. Unsupported attributes, namespace declarations,
  quote style, and character-reference spelling on the owner remain unchanged.
- **Pass-1 D9, root breadth and raw inventory closed**: the corpus gate now
  walks slides, layouts, and masters, checks raw timeline equality, inventories
  unsupported node bytes, and requires typed node, condition, and build
  coverage. D6 lists the three remaining unasserted counters.
- **Panics, 0 findings**: the added indexing and internal `expect` calls are
  guarded by explicit nonempty or established parser state. No reachable
  untrusted-input panic or arithmetic defect was found.
- **Structure, 0 findings**: the approved module remains concrete and local.
  It adds no trait, generic parameter, wrapper-only abstraction, feature,
  crate, or dependency.
- **Atomicity and preservation, 0 additional findings**: node lookup and
  transition mutations stage raw bytes and reparse before replacing live
  state. Failed mutation leaves the prior value intact. Unsupported sibling
  and relationship-bearing subtree bytes remain the serialization source.
- **Harness and API scope, 0 findings**: there is no baseline change, facade
  execution API, renderer behavior, binding API, production dependency, or
  feature expansion.

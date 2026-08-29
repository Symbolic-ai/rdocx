# F-X066, working, pass 2

**Reviewed**: complete remediated working diff against claim Base
`3ddac3a3420eda6dc25abd9c5b1dce5721725834`, 10 files, 362 insertions and
6 deletions
**Verdict**: 2 defects, 1 smell, 0 nitpicks

## Defects

### D1, namespace retention adds an unapproved public struct-literal break

`crates/rdocx-oxml/src/text.rs:434`

`CT_R` is a public struct re-exported by the published `rdocx-oxml` crate at
`crates/rdocx-oxml/src/lib.rs:41`. Adding the required public
`raw_xml_namespace_bindings` field makes every existing external `CT_R` struct
literal fail to compile until it supplies the new field. The same break is
visible inside this workspace, for example the existing literal at
`crates/rdocx/src/document.rs:2222` now needs an otherwise irrelevant empty
value. The approved plan states only the additive non-exhaustive
`RunItemRef` impact at `.claude/plans/F-X066-design.md:80`. It does not approve
another low-level pre-1.0 source break or a stable-family version change. The
namespace context must be retained without expanding the required public
`CT_R` literal shape, or the material API change must return to design rather
than ship under this plan.

### D2, equal runs can expose different public item classifications

`crates/rdocx-oxml/src/text.rs:442`

The new manual `PartialEq` implementation deliberately omits
`raw_xml_namespace_bindings`, but `RunRef::items` uses that field to choose the
public result at `crates/rdocx/src/run.rs:772`. Two `CT_R` values with identical
raw `<w:pict><v:rect o:hr="t"/></w:pict>` bytes and positions therefore compare
equal even when one binds `v` to the VML namespace and the other binds it to a
foreign namespace. The first exposes `LegacyHorizontalRule` and the second
exposes `UnsupportedXml`. Public equality cannot ignore state that changes an
observable semantic projection. Retained context needs a representation whose
semantic equality is consistent with classification while irrelevant producer
bindings do not make otherwise equal modeled runs unequal.

## Smells

### S1, every parsed run clones the complete namespace scope eagerly

`crates/rdocx-oxml/src/text.rs:575`

`from_xml_with_prefixes` clones the inherited namespace scope before it knows
whether the run contains any raw child, and `namespace_bindings` allocates two
owned strings for every binding at `crates/rdocx-oxml/src/numbering.rs:591`.
Ordinary text-only runs therefore retain a separate copy of all document-scope
bindings even though classification never reads it. This adds per-run heap and
string growth across large documents for a feature that applies only to rare
raw children. Capture only context required by a retained raw child, and avoid
repeating irrelevant bindings on every modeled run.

## Nitpicks

None.

## Not found

- Pass-1 D1 closure: the package regression now distributes `w`, `v`, and `o`
  bindings across document, paragraph, and run ancestors, and verifies the
  same exact raw item before and after save and reopen.
- Correctness beyond D2: local namespace declarations shadow inherited ones,
  and expanded-name checks still accept only Word `pict`, VML `rect`, and the
  Office `hr` attribute.
- Classifier strictness: only `t` and `true` enable the rule. Numeric, false,
  missing, duplicate, foreign, multiple-shape, visible-child, comment, and
  malformed inputs fail closed as unsupported XML.
- OOXML preservation: the synthetic scope wrapper exists only during
  classification. The stored subtree, public raw accessor, item position, and
  serialized child bytes remain unchanged.
- Parser failure behavior: namespace, attribute, decode, and XML event errors
  return the unsupported fallback without a production panic.
- Layout and rendering: the new classification still has no layout or backend
  consumer, consistent with the unchanged 49-entry hash evidence.
- Tests beyond the findings: local aliases, default namespaces, shadowing,
  negative structures, raw ordering, package reopen, affected crate suites,
  package dry runs, and the pinned Word evidence are recorded as passing.
- HLD and plan scope: no HLD file has changed before completion, no new module
  or test binary was added, and PR 57 remains read-only contribution evidence.
- Structure beyond S1: no new trait, generic parameter, dependency, feature
  flag, forwarding-only wrapper, or rendering path was introduced.

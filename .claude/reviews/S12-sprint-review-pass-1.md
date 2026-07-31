# S12 sprint review, pass 1

**Reviewed**: `sprint/s12` against
`f18ce287d6669d2686a7ff7e6a11647c8496361c`, 25 files, 3240 insertions and
49 deletions, crates: `oxml-drawing`
**Verdict**: 1 blocking, 1 should-fix, 0 nice-to-have

## Blocking

### B1, whitespace-only transform pairs are not modelled

`crates/oxml-drawing/src/color.rs:742`

F-054 accepts a colour child as an XML start event, and F-055 models it only
when the captured bytes contain an immediate end event. A schema-valid empty
transform written with formatting whitespace, such as
`<a:tint val="62000">\n</a:tint>`, produces a text event between the start and
end events. `is_explicit_empty_element` returns false, so the transform is kept
as raw XML and is absent from colour resolution. The fix must treat
whitespace-only text and non-content XML events as empty while continuing to
preserve any known transform with nested element or non-whitespace text
content verbatim.

## Should-fix

### S1, unused public map parsers widen the story surface

`crates/oxml-drawing/src/color.rs:268`

`ColorMapSlot::parse`, `ThemeColorSlot::parse`, and their two public error
variants have no caller or test in the repository. F-056 requires a concrete
map built from already parsed values, while F-069 owns `p:clrMap` and
`p:clrMapOvr` parsing. Remove this speculative parsing surface until that
consumer exists. This also removes the inaccurate description of `bg1` as an
`ST_ColorSchemeIndex` value rather than a colour-map attribute name.

## Nice-to-have

None.

## Milestone gate

The M7 end gate is: "every `a:txBody` and `a:spPr` in the deck corpus parses,
serialises and reparses to a structurally equal value." S12 is the first M7
slice, so this end-of-milestone gate is not yet due. F-063 and F-064 own those
root models.

The S12 boundary gate is otherwise evidenced: 27 active `oxml-drawing` tests
pass, including the 40 exact PowerPoint RGBA cases and dark-master resolution.
The full workspace gate passes, all 28 hashes match, the released Word theme
diff is empty, the crate is version 0.0.0 with publication disabled, and its
normal dependency graph has no `rdocx-*` or `rpptx-*` edge.

## Not found

- Interaction: no other conflict between the five integrated stories.
- Duplication: no duplicate helper or resolution path.
- Layering: no format-specific production dependency from `oxml-drawing`.
- Harness: every story declared an unchanged result and all 28 entries match.
- Docs: the one planned HLD file records the implemented 28-transform and
  partial-alpha contracts.
- Dependencies: `oxml-core` and `quick-xml` are production dependencies, and
  `oxml-opc` is limited to the development oracle.
- Gate: every S12 feature gate has direct test evidence.

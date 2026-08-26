# F-X066, Classify legacy VML horizontal rules

**Status**: approved
**Sprint**: S58
**Size**: S
**Depends on**: F-X065

## Problem

PR 57 proposes a native classification for legacy horizontal rules stored in
`w:pict`, but its submitted byte scanner relies on literal `w:`, `v:`, and
`o:` prefixes. OOXML prefixes are aliases, so that approach misses valid
namespace rebinding and can misclassify foreign namespaces using familiar
prefixes.

Raw run children currently surface through `RunItemRef::UnsupportedXml` in
`crates/rdocx/src/run.rs`. The reader needs a narrow semantic classification
that retains the exact bytes and leaves all ambiguous VML unsupported.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, expanded-name parsing and raw XML preservation.
- `docs/hld/08-rendering-spec.md`, the boundary between preserved unsupported VML and rendered VML.
- `docs/hld/10-bindings-spec.md`, native Word run inspection and non-exhaustive reader enums.
- `docs/hld/12-testing-strategy.md`, namespace-rebinding and save-reopen regressions.
- `docs/hld/14-development-backlog.md`, "F-X066, Classify legacy VML horizontal rules".

## Approach

Add `RunItemRef::LegacyHorizontalRule(LegacyHorizontalRuleRef)` and an exact raw
accessor. Recognize the shape with `quick_xml` and in-scope namespace URI
facts, not lexical prefixes. A positive item is a WordprocessingML `pict`
containing exactly one VML `rect` whose Office `hr` value is enabled, with only
whitespace otherwise. Accept the VML true forms `t` and `true`. Reject numeric
`1`, false, missing, malformed, foreign, multiple-shape, visible-child, and
ambiguous input as `UnsupportedXml`.

This is a reader-classification story only. Layout continues to ignore the raw
run child, and no renderer draws a horizontal line. The release note must not
claim visual fidelity. Thread namespace scope through existing files or classify
at the existing OXML parse boundary. Do not add a module.

Use PR 57 at commit `44498f042a2290ef40c7a6c26025f38e38e9ce2a`
as contribution evidence, then implement the hardened equivalent from the
integrated F-X065 head. Do not merge, retarget, comment on, or close the PR.

## Rejected alternatives

- Cherry-pick PR 57 unchanged. Prefix-string matching is not namespace-aware.
- Classify every VML rectangle. Only the unambiguous Office horizontal-rule marker is in scope.
- Render the rule in this story. That needs a separate layout and backend contract with a deliberate hash decision.
- Normalize or reconstruct the raw XML. Native classification must not weaken verbatim preservation.
- Add a new module or test binary. The reader and regression belong in existing run and integration files.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `legacy_horizontal_rule_classification_is_namespace_aware` | Canonical and aliased Word, VML, and Office namespaces classify |
| negative | `ambiguous_or_foreign_vml_stays_unsupported` | False or missing markers, foreign same-local names, extra shapes, visible content, comments, and malformed XML do not classify |
| preservation | `legacy_horizontal_rule_keeps_exact_raw_xml_and_item_order` | The classified item exposes original bytes in its original run position |
| package | existing `rdocx` regression binary | A source-built package saves and reopens without changing the VML subtree |
| integration | current Word corpus gate | Legacy VML documents open after the exact locked offline no-default build and produce required evidence |

The **test gate is regression**. The focused native reader and package tests,
the current Word corpus job, and `/verify --full` must pass.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serialiser**. Require expanded-name alias and foreign-prefix
  cases plus byte-exact preservation through save and reopen.
- **Public API of a published crate**. State the additive non-exhaustive enum
  impact, run the patched publish dry run, and enforce archive limits.
- **An external oracle comparison**. Use the pinned Word corpus only through
  the differential-testing contract and require nonempty current evidence.

## Hash harness

Expected unchanged at 49 of 49. Classification changes native inspection only.
No layout or rendering path consumes the item.

## Implementation checklist

- [ ] Add failing URI-aware positive and negative cases in the existing run test module.
- [ ] Add the native classification and exact raw accessor without a new module.
- [ ] Preserve unsupported fallback, item order, and package bytes.
- [ ] Run focused reader, package, corpus, publish, and risk-rider gates.
- [ ] Run microscope and `/verify --full`.
- [ ] Record PR 57 and its exact source SHA in the handoff and delivery evidence.

## Open questions

None. The approved scope is native reader classification only. Rendering a
horizontal line remains a separate story with its own output contract.

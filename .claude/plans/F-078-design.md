# F-078, relmap rewrite_rel_ids

**Status**: completed
**Sprint**: S18
**Size**: M
**Depends on**: F-067

## Problem

Preserved PresentationML payloads can contain relationship references that are
not represented by typed fields. The current crate exports no relationship-id
rewriter from `crates/rpptx-oxml/src/lib.rs:3`, while the preservation boundary
in `crates/rpptx-oxml/src/shape_tree.rs:23` retains several payloads as XML
bytes. Deep-copy code therefore cannot safely assign new relationships for
SmartArt, embedded media, OLE, or other opaque content.

The rewrite must distinguish attributes by namespace URI rather than a literal
prefix. It must also preserve every byte outside an eligible attribute value,
because reconstructing the XML would change quoting, whitespace, comments, or
processing instructions and violate the preservation contract.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "Relationship types".
- `docs/hld/06-presentationml-model.md`, "Preservation strategy" and
  "Relationship remapping".
- `docs/hld/12-testing-strategy.md`, "Unit tests" and "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-078, relmap rewrite_rel_ids".

## Approach

Add `crates/rpptx-oxml/src/relmap.rs` and export this function:

```rust
pub fn rewrite_rel_ids(
    raw: &[u8],
    map: &HashMap<String, String>,
) -> Result<Vec<u8>>;
```

The implementation validates and tokenises the XML with namespace scope while
copying the original source bytes. It records only the byte ranges of attribute
values whose expanded namespace is the Office document relationships namespace
and whose decoded value matches `rId` followed by one or more ASCII digits.
Mapped values replace those ranges with XML escaping appropriate for attribute
content. An eligible id absent from `map` remains unchanged.

Namespace aliases and nested prefix shadowing are resolved by URI. Unqualified
attributes are never relationship attributes. Attribute names such as
`r:embed`, `r:link`, `r:dm`, `r:id`, `r:lo`, `r:qs`, and `r:cs` need no special
case because the namespace and value rule covers all of them. Malformed input
returns the crate's existing XML error and never emits a partial result.

The byte-range writer preserves element spelling, namespace declarations,
attribute order, quote choice, entity spelling outside replaced values,
whitespace, text, CDATA, comments, and processing instructions exactly. It does
not parse or remodel the surrounding payload.

## Rejected alternatives

- Re-serialise every quick-xml event. That can normalise otherwise untouched
  syntax and cannot satisfy the byte-identical remainder gate.
- Match the literal `r:` prefix. OOXML readers must accept namespace aliases,
  and a nested binding can shadow that prefix.
- List only known relationship attribute local names. The HLD requires every
  relationship-namespace attribute with a matching value, including extension
  content not known today.
- Rewrite every value that looks like an id. That would corrupt ordinary
  attributes and text outside the relationships namespace.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `rewrites_relationship_attributes_and_no_other_bytes` | `r:embed`, `r:link`, and `r:dm` are all mapped while a byte-splice expectation proves every other byte is identical |
| unit | `relationship_namespace_aliases_and_shadowing_are_respected` | An alias bound to the relationships URI is rewritten and a nested prefix rebound to another URI is not |
| unit | `unmapped_and_non_numeric_relationship_values_are_unchanged` | Missing map entries, unqualified attributes, other namespaces, and values outside `^rId[0-9]+$` remain exact |
| preservation | `comments_processing_instructions_and_attribute_syntax_are_preserved` | Comments, processing instructions, whitespace, attribute order, and both quote styles survive exactly outside replacements |
| error | `malformed_preserved_xml_returns_an_error` | Invalid or truncated XML fails without panicking or returning partial bytes |
| round-trip | `every_corpus_preserved_payload_is_identity_with_an_empty_map` | Every captured opaque payload in the pinned corpus remains byte-identical when no relationship id is mapped |

The test gate is: a preserved blob containing `r:embed`, `r:link` and `r:dm`
has all three rewritten, and everything else is byte-identical.

## HLD impact

None. `docs/hld/06-presentationml-model.md` already specifies the module,
function contract, namespace rule, and byte-preservation requirement.

## Risk routing

- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Check namespace aliases and shadowing,
  malformed XML, and exact byte preservation outside rewritten values.
- A new module or file. Read the structural rules in `CLAUDE.md` and obtain
  explicit approval before adding `crates/rpptx-oxml/src/relmap.rs`.

The consolidated sprint gate adds `cargo test -p rpptx-oxml`, the required
corpus identity test, and `cargo tree -p rpptx-oxml --edges normal`.

## Hash harness

Expected to be unchanged. The helper is confined to the unpublished
PowerPoint development model and has no Word rendering path.

## Implementation checklist

- [x] Add and export the relationship remapping module.
- [x] Resolve relationship attributes by namespace URI with nested scope.
- [x] Rewrite mapped numeric relationship ids by replacing value byte ranges.
- [x] Preserve every other source byte and return errors for malformed XML.
- [x] Add focused namespace, preservation, and error tests.
- [x] Add the required pinned-corpus identity coverage.
- [x] Confirm every PowerPoint development crate remains version 0.0.0 and unpublished.
- [x] Confirm all deterministic hashes remain unchanged.

## Open questions

None. The user approved `crates/rpptx-oxml/src/relmap.rs`.

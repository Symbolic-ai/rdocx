# F-153, Content control binding

**Status**: completed
**Sprint**: S46
**Size**: M
**Depends on**: F-152

## Problem

F-152 makes content controls addressable but does not give the facade a value
mutation contract. `Document` currently resolves known typed parts individually
and keeps all other package parts opaque (`crates/rdocx/src/document.rs:156`).
There is no lookup by control tag or alias, no atomic map application, and no
way to follow `w:dataBinding` to the matching custom XML datastore while
updating the displayed content.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-153, Content control binding".
- `docs/hld/03-architecture.md`, "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", "Part naming",
  and "Package integrity".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".

## Approach

Add immutable content-control summaries and document-owned mutation methods:

```rust
impl Document {
    pub fn content_controls(&self) -> Vec<ContentControlRef<'_>>;
    pub fn content_controls_by_tag(&self, tag: &str) -> Vec<ContentControlRef<'_>>;
    pub fn content_controls_by_alias(&self, alias: &str) -> Vec<ContentControlRef<'_>>;
    pub fn set_content_control_value_by_tag(&mut self, tag: &str, value: &str) -> Result<usize>;
    pub fn set_content_control_value_by_alias(&mut self, alias: &str, value: &str) -> Result<usize>;
    pub fn bind_content_controls(&mut self, values: &std::collections::HashMap<String, String>) -> Result<usize>;
}
```

Value replacement updates the bounded plain-text display content while keeping
the control shell, formatting, and unsupported properties. Map binding matches
tag first, then alias when no tag key is present, so one control is not updated
twice.

For `w:dataBinding`, resolve `storeItemID` through each custom XML item's
properties part, apply `prefixMappings`, evaluate the approved XPath subset,
and replace the selected element text without rebuilding unrelated custom XML.
Stage all display and custom XML edits on cloned state. Commit them only after
every selected control and part has updated successfully, then invalidate the
layout once.

Proposed new source file:

```text
crates/rdocx/src/content_control.rs
```

No new crate, feature flag, trait, or generic parameter is added. The XPath
subset uses the existing pull parser unless the consolidated design decision
explicitly authorises a dependency.

## Rejected alternatives

- Implement a full XPath 1.0 engine. The story needs Word data-binding paths,
  not predicates, functions, arithmetic, or arbitrary axis evaluation.
- Replace the whole custom XML part as a string. That can alter namespaces,
  comments, processing instructions, and untouched producer content.
- Mutate matching controls as they are found. A later invalid binding would
  leave the document half updated.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `tag_precedes_alias_and_each_control_updates_once` | Map application has deterministic key precedence and returns the number of controls changed |
| regression | `a_control_map_updates_every_matching_display_value` | Nested controls at different levels receive their mapped plain text without disturbing formatting or unrelated controls |
| regression | `a_bound_custom_xml_value_updates_the_part_and_display_text_atomically` | The selected custom XML node and every bound display agree after reload, with unrelated XML bytes unchanged |
| regression | `an_invalid_binding_changes_neither_document_nor_custom_xml` | Missing store ids, unsupported paths, and ambiguous matches fail before any staged change becomes visible |

The **test gate**, from the backlog, is regression. A control set bound to a
map produces the expected text, and a bound custom XML part updates both the
part and the display text.

Tests join the existing rdocx regression binary and crate-local modules. No new
test binary is added.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`

Record document-owned control mutation, the custom XML relationship graph,
the bounded XPath contract, atomic multi-part updates, and the additive native
API.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add prefix-mapping,
  namespace-shadowing, schema-order, malformed-binding, custom XML round-trip,
  and byte-preservation tests.
- Public API of a published crate. Read HLD 10 and the structural rules. The
  lookup and mutation methods are additive and story-required. Run affected
  package dry-runs and archive size assertions.
- A new module or file. The focused facade module needs explicit approval.

## Hash harness

Expected unchanged across all 49 entries. Existing samples have no content
controls or custom XML bindings.

## Implementation checklist

- [x] Add ordered content-control traversal and lookup by tag or alias.
- [x] Add atomic plain-text value mutation and deterministic map binding.
- [x] Resolve custom XML datastores by store item id and prefix mappings.
- [x] Evaluate the approved bounded XPath subset and preserve untouched XML.
- [x] Add map, bound-part, invalid-binding, and preservation regressions.
- [x] Run focused checks plus the declared packaging rider.
- [x] Update exactly HLD 03, HLD 04, and HLD 10 at completion.

## Open questions

None. The approved XPath boundary is namespace-aware absolute child paths with
optional one-based numeric child indices and no functions, wildcards,
descendant axes, or general predicates. The focused facade module is approved.

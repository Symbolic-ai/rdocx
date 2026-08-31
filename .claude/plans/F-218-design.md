# F-218, Embedded object and macro inventory

**Status**: approved
**Sprint**: S62
**Size**: L
**Depends on**: none

## Problem

The presentation facade preserves OLE graphic-frame payloads, ActiveX package
parts, and VBA projects only as incidental package data. `GraphicDataPayload`
in `crates/rpptx-oxml/src/graphic_frame.rs` projects an OLE preview while its
raw XML remains the serialization source, but callers cannot inventory the
embedded payload, prove its identity, extract it, replace it, or remove its
owning relationship and XML reference safely. ActiveX and VBA relationships do
not have a typed facade path at all.

These payloads can contain executable content. The implementation must treat
all bytes as opaque, never invoke a decoder or host application, and make every
mutation transactional. Untouched payload bytes and signature evidence must
remain exact across ordinary presentation edits.

## Spec reference

- ECMA-376 Part 1, PresentationML embedded object and control references.
- ECMA-376 Part 2, OPC internal relationships, content types, and digital
  signature invalidation through package mutation.
- Microsoft Office relationship types for ActiveX controls, VBA projects, and
  VBA project signatures.
- `docs/hld/02-scope-and-non-goals.md`, the SmartArt and OLE scope table.
- `docs/hld/03-architecture.md`, "The dependency rule" and package facade
  seams.
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", "Part naming",
  "Package integrity", and digital signatures.
- `docs/hld/06-presentationml-model.md`, "Public facade", "Preservation
  strategy", "Relationship remapping", "Validation", and mutation signature
  handling.
- `docs/hld/10-bindings-spec.md`, published native Rust API policy.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-218, Embedded object and macro
  inventory".

## Approach

Add the known Transitional and Strict relationship constants to `oxml-opc`.
Keep relationship classification in that package layer and keep
PresentationML ownership rules in `rpptx`.

Add a private `crates/rpptx/src/embedded.rs` module and re-export concrete,
owned inspection values:

```rust
pub enum EmbeddedContentKind {
    OleObject,
    ActiveXControl,
    VbaProject,
}

pub enum EmbeddedSignatureState {
    Absent,
    Present,
    Invalidated,
}

pub enum EmbeddedMutationPolicy {
    PreserveInvalidatedSignatures,
    RemoveInvalidatedSignatures,
}

pub struct EmbeddedContentInfo {
    pub kind: EmbeddedContentKind,
    pub source_part: String,
    pub relationship_id: String,
    pub target_part: String,
    pub content_type: String,
    pub byte_len: usize,
    pub sha256: [u8; 32],
    pub signature_state: EmbeddedSignatureState,
}

impl Presentation {
    pub fn embedded_content(&self) -> Result<Vec<EmbeddedContentInfo>>;
    pub fn extract_embedded_content(
        &self,
        source_part: &str,
        relationship_id: &str,
    ) -> Result<Vec<u8>>;
    pub fn replace_embedded_content(
        &mut self,
        source_part: &str,
        relationship_id: &str,
        bytes: &[u8],
        policy: EmbeddedMutationPolicy,
    ) -> Result<EmbeddedContentInfo>;
    pub fn remove_embedded_content(
        &mut self,
        source_part: &str,
        relationship_id: &str,
        policy: EmbeddedMutationPolicy,
    ) -> Result<()>;
}
```

The pair of normalized source part and relationship id is the identity already
owned by OPC. Do not add a forwarding identifier wrapper. Inventory walks only
the normalized internal relationship graph, rejects traversal or external
targets for extraction and mutation, sorts by source part and relationship id,
and hashes the stored bytes without parsing or executing them.

OLE ownership follows the `p:graphicFrame` relationship reference and its
optional preview remains independent. ActiveX ownership follows the slide or
presentation control reference through its control-properties part to the
opaque binary payload. VBA ownership follows the presentation relationship to
the project and any project-signature relationship. Inventory reports each
logical executable payload once while retaining all relationship coordinates
needed for exact audit.

Replacement retains the existing part name, content type, owning XML node, and
relationship id. Removal deletes the owning OLE frame, ActiveX control entry,
or VBA relationship as appropriate, then deletes only candidate parts that are
unreachable from every remaining package relationship. It also removes stale
content-type overrides and relationship parts owned by deleted candidates.
Shared payloads and unrelated producer orphans remain untouched.

Every mutation clones the current staged package and affected typed model,
applies the graph and XML changes, serializes, reparses, runs package and
presentation validation, and commits only on success. The signature policy
applies to package signatures and signatures attached to the affected VBA
project. Preserving keeps the original evidence so verification reports it as
invalidated after mutation. Removing deletes only the corresponding signature
infrastructure. Untouched signatures remain byte-identical and valid.

Use the existing SHA-256 implementation already present in the workspace. Add
no new dependency, feature, trait, generic, crate, integration binary, or
binary fixture. This is additive native Rust API for the pre-1.0 `rpptx` and
`oxml-opc` crates. No Python, WASM, or CLI surface is added.

## Rejected alternatives

- Scanning file extensions would miss producer-specific names and would not
  prove relationship ownership.
- Parsing OLE Compound File Binary or VBA bytecode would expand the attack
  surface and is not required for safe inventory.
- Deleting every unreferenced embedding part would destroy producer orphans
  that the requested mutation does not own.
- Silently dropping signatures on every save would discard audit evidence and
  make ordinary edits violate preservation.
- A new identifier wrapper would only forward the existing OPC coordinates.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `embedded_inventory_reports_exact_hashes_relationships_and_signature_state` | OLE, ActiveX, and VBA entries report exact kind, source, relationship, target, content type, length, SHA-256, and signature state in stable order. |
| integration | `embedded_extract_replace_and_remove_are_atomic_and_never_execute_payloads` | Extraction is byte-exact, replacement retains identity, removal updates XML, relationships, parts, and content types together, and every failure leaves bytes unchanged. |
| regression | `ordinary_presentation_edits_preserve_every_retained_executable_payload_byte` | Shape and text edits leave OLE, ActiveX, VBA, and attached signature bytes unchanged. |
| regression | `embedded_removal_deletes_only_relationship_owned_unreachable_candidates` | Shared payloads and unrelated orphans survive while the requested logical object disappears and the deck validates after reopen. |
| regression | `embedded_mutation_policy_preserves_or_removes_invalidated_signature_evidence` | Both explicit policies affect only package and VBA signature infrastructure associated with the mutation, and no stale signature is reported valid. |
| regression | `external_and_traversal_embedded_targets_fail_closed` | Inventory diagnoses unsafe targets and extraction, replacement, and removal do not access them. |

The exact backlog **test gate is regression**: "Inventory reports exact hashes
and relationships, safe removal leaves a valid deck, and ordinary edits do not
alter retained payload bytes."

Use tracked OLE and embedded-package corpus decks where available. Construct
ActiveX, VBA, signature, shared-target, orphan, and failure fixtures in the
existing `crates/rpptx/tests/integration.rs` binary. Do not add a binary fixture
or integration binary.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Any parser or serialiser: re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add prefix-alias, fixed-prefix,
  schema-order, structural-reparse, and byte-exact retained-subtree checks.
- Crate dependency graph and cross-family uses: relationship constants remain
  in `oxml-opc`, while package ownership remains in `rpptx`. Run
  `cargo tree -p rpptx -e normal` and the shared-crate dependency-direction
  test.
- Public API of a published crate: record the additive pre-1.0 impact. Run
  publish dry runs for `oxml-opc` and `rpptx`, then assert every archive stays
  below 10 MiB.
- New module or file: explicit approval is required for
  `crates/rpptx/src/embedded.rs`. It isolates executable-content graph walking,
  signature policy, and transactional mutation from the already large facade.

## Hash harness

Expected unchanged, 49 of 49. Inventory is read-only and mutation is opt-in.
Any ordinary sample delta is unexplained and blocks integration.

## Implementation checklist

- [ ] Add relationship constants and strict internal-target classification.
- [ ] Add the approved private embedded-content module and public values.
- [ ] Inventory OLE, ActiveX, VBA, and their signature relationships exactly
  once in stable order.
- [ ] Implement byte-exact safe extraction without decoding or execution.
- [ ] Implement staged replacement and ownership-aware removal.
- [ ] Apply both explicit signature mutation policies without false validity.
- [ ] Preserve unrelated payloads, previews, raw XML, shared parts, and orphans.
- [ ] Add source-built and corpus regression cases to existing test targets.
- [ ] Run focused `oxml-opc`, `rpptx-oxml`, and `rpptx` checks plus every rider.

## Open questions

None. The private embedded-content module and explicit preserve-or-remove
signature mutation policy are approved.

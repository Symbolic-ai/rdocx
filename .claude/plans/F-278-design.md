# F-278, General simple and complex field builder

**Status**: completed
**Sprint**: S90
**Size**: L
**Depends on**: F-260

## Problem

`Field::new` constructs a simple field from raw instruction text and one plain
cached string at `crates/rdocx-oxml/src/text.rs:468`. New fields select the simple
form at `crates/rdocx-oxml/src/text.rs:7601`, unless nested operands force complex
serialization. The recursive instruction model exists at
`crates/rdocx-oxml/src/text.rs:853`, but callers lack a validated typed constructor
and an explicit complex construction surface.

Parsed fields expose ordered cached text and properties at
`crates/rdocx-oxml/src/text.rs:745`, while authored fields have no ordered
cached-run input. `Run::add_field` validates raw strings and appends a simple
field at `crates/rdocx/src/run.rs:502`. `FieldRef` exposes kind, instruction,
cached display and dirty state at `crates/rdocx/src/run.rs:277`, but no lock
state. The source model at `crates/rdocx-oxml/src/text.rs:406` has no typed lock.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-278, General simple and complex field builder".
- `docs/hld/02-scope-and-non-goals.md`, DOCX-045 in the native Word capability matrix.
- `docs/hld/03-architecture.md`, "What stays put", recursive field grammar, ordered run content and physical field source preservation.
- `docs/hld/04-opc-and-packaging.md`, "The package" and "Package integrity".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability", native field grammar and ordered run authoring.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and native Word field regression contracts.

## Approach

Reuse `Field`, `FieldInstruction`, `FieldArgument`, `FieldSwitch`, `CT_R` and
`RunContent`. Make the existing private `FieldForm` public. Keep facade
`run::FieldKind` as the existing read projection.

```rust
pub enum FieldForm {
    Simple,
    Complex,
}

impl FieldInstruction {
    pub fn new(
        name: &str,
        arguments: Vec<FieldArgument>,
        switches: Vec<FieldSwitch>,
    ) -> rdocx_oxml::Result<Self>;
}

impl Field {
    pub fn from_raw(
        instruction: &str,
        form: FieldForm,
        cached_runs: Vec<CT_R>,
    ) -> rdocx_oxml::Result<Self>;

    pub fn from_instruction(
        instruction: FieldInstruction,
        form: FieldForm,
        cached_runs: Vec<CT_R>,
    ) -> rdocx_oxml::Result<Self>;

    pub fn form(&self) -> FieldForm;
    pub fn locked(&self) -> Option<bool>;
    pub fn set_locked(&mut self, value: Option<bool>);
}

impl Run<'_> {
    pub fn add_field_value(&mut self, field: Field) -> crate::Result<()>;
}

impl FieldRef<'_> {
    pub fn locked(&self) -> Option<bool>;
}
```

`Field::new` and `Run::add_field` retain their existing signatures and behavior.
Checked constructors validate before attachment. Reject XML-invalid characters,
an absent or invalid field name, malformed instruction tokens and unbalanced
quoted instructions. Unknown valid names and switches remain authorable and
retain caches. Quoted and nested operands after otherwise unknown switches are
recognized as switch operands. Typed unknown text operands use forced quotes.
Known flag switches preserve the existing positional-operand boundary and
reject an explicit typed operand. This is a pre-1.0 projection clarification,
with original producer XML retained unchanged.

Typed construction produces one canonical instruction from validated arguments
and switches. Existing quoting and escaping prevents text operands injecting
switches, operands or field delimiters. Raw text and typed structure cannot be
competing sources of truth. Existing parsed-field mutation rules remain supported.
Explicit simple construction rejects nested instruction fields. Complex fields
serialize ordered begin, instruction, separate, cached display and end runs.
Nested arguments and switch arguments emit balanced complex sequences at their
operand positions. Otherwise unchanged parsed fields preserve encounter order.

Store authored cached content as ordered `CT_R` values. Preserve properties,
text, tabs, breaks and nested result fields through save and reopen. Keep
`cached_result` as the established projected display string. Explicit cache
string mutation retains established replacement formatting behavior. Validate
cached content before attachment, including malformed raw content and unsupported
field delimiters.

Read and write `w:fldLock` on the simple owner or complex begin owner. Distinguish
absent, explicit false and true. Existing `dirty: Option<bool>` retains the same
three states. Property-only edits retain producer XML outside the attribute.
Untouched parsed fields keep original bytes.

DOCX-045 describes checked construction and stored-cache round trips. Mark that
capability complete with layout and rendering not applicable to its construction
gate. Separate field capabilities own rendering and execution. Update the policy
inventory of completed capability owners without relaxing the live-owner rule
for any incomplete capability.

No new trait, generic, crate, source file or feature flag is required. The facade
API is additive. Low-level representation changes have intentional pre-1.0
compatibility implications that completion documents.

## Rejected alternatives

- A second grammar creates disagreement between authoring, parsing and evaluation.
- A cached-run wrapper duplicates existing `CT_R` content and properties.
- Raw XML manufacture in the facade bypasses validation and ordered preservation.
- Converting explicit simple requests to complex silently obscures selected form.
- New source modules are unnecessary because existing owners contain this logic.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `authored_simple_and_complex_fields_reopen_with_identical_semantics` | Normalized semantics, explicit form, ordered caches and formatting survive. |
| round-trip | `authored_nested_instruction_fields_preserve_operand_order` | Nested arguments and switch arguments retain balanced delimiters and order. |
| round-trip | `field_lock_and_dirty_preserve_absent_false_and_true` | Three-state toggles survive in both forms without unrelated XML changes. |
| unit | `typed_field_arguments_cannot_inject_instruction_tokens` | Quotes, backslashes, switch-looking text, whitespace and empty operands remain one operand. |
| regression | `invalid_field_construction_leaves_run_unchanged` | Invalid instructions or cached content do not mutate destination runs. |
| regression | `field_cache_runs_keep_controls_and_properties_in_order` | Text, tabs, breaks, nested result fields and properties retain order. |
| round-trip | `field_property_edits_preserve_unmodelled_xml_verbatim` | Aliased prefixes and opaque children retain exact bytes outside edited attributes. |
| regression | `legacy_add_field_retains_its_existing_simple_contract` | Existing plain cached-string API remains compatible. |
| round-trip | `unknown_typed_switch_operands_reopen_without_changing_known_flags` | Unknown text and nested switch operands retain positions while known flags stay flags. |
| regression | `equal_text_nested_replacement_writes_its_new_cache_properties` | Argument and switch operand replacement at child and grandchild depths invalidates retained source even with equal display text. |
| regression | `cached_opaque_namespaces_preserve_foreign_lookalikes_and_inherited_word_names` | Foreign lookalikes and inherited namespaces survive, while actual Word delimiters reject. |
| round-trip | `checked_cached_page_and_column_breaks_attach_and_reopen` | Typed page and column controls survive attachment and nested-operand validation. |

**Test gate**: round-trip. Every supported field shape reopens with identical
instruction semantics and ordered cached content.

Add tests to existing `text.rs` test modules and
`crates/rdocx/tests/regression_test.rs`. No new test binary. Run focused
`rdocx-oxml` and `rdocx` checks and tests with `RUSTUP_TOOLCHAIN=1.97.1` and an
isolated `CARGO_TARGET_DIR`, scoped verify and zero-finding microscope.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Any parser or serialiser. Read `04-opc-and-packaging.md` and `06-presentationml-model.md`. Verify prefix-tolerant reads, fixed-prefix new writes, schema order and byte-for-byte opaque preservation.
- Public API of a published crate. Read `10-bindings-spec.md`. Record pre-1.0 compatibility impact. Run canonical publish dry-run with local patches and assert archives below 10 MiB.
- New file. Required workflow records have explicit user creation approval. No source file, module, crate, trait or generic is planned.

## Hash harness

Expected unchanged. Existing simple construction and untouched parsed fields
retain output. Newly authored forms use new tests rather than hash fixtures.
Unexplained deltas block completion.

## Source file claims

- `crates/rdocx-oxml/src/text.rs`
- `crates/rdocx/src/run.rs`
- `crates/rdocx/src/lib.rs`, only for necessary facade re-exports
- `crates/rdocx/tests/regression_test.rs`
- `README.md`, required measured archive inventory refresh
- `crates/rdocx-oxml/README.md`, required measured archive inventory refresh
- `scripts/readme_doctests.py`, matching measured archive constants and dates
- `scripts/test_sprint_workflow.py`, completed builder capability owner inventory

The shared regression entrypoint is exclusive. F-X178 and Issue 264 are outside
this work.

## Implementation checklist

- [x] Add checked typed and raw construction to the existing recursive model.
- [x] Reuse `CT_R` for ordered cached content.
- [x] Expose explicit simple and complex form.
- [x] Implement lock access and three-state serialization.
- [x] Preserve untouched and property-edited producer XML.
- [x] Add checked run attachment and immutable lock access.
- [x] Add declared tests to existing owners.
- [x] Pass scoped verify, packaging riders and zero-finding microscope.

## Open questions

None. The user approved all six stories' workflow records and the dedicated
bibliography module. Native APIs are additive, existing outline APIs retain
their behavior, and fresh Word evidence is pinned to 16.113.2 build
16.113.26092012. INDEX and TOA use en-US source language with the captured Word
locale recorded explicitly. Technical ownership probes in the test plan must
pass before completion and may not be replaced by guessed expectations.

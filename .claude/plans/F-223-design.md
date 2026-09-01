# F-223, Modern presentation package variants

**Status**: completed
**Sprint**: S63
**Size**: M
**Depends on**: F-218

## Problem

The native presentation facade preserves opaque executable payloads and can
write an output-only ordinary slideshow, but callers cannot inspect or select
the complete modern PowerPoint package class. Macro-enabled presentations and
templates, ordinary templates, and macro-enabled slide shows therefore lack an
explicit supported boundary even though their package graphs are otherwise
readable and preservable.

Binary `.ppt` is not OPC and remains out of scope.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, native presentation package formats.
- `docs/hld/03-architecture.md`, facade ownership of package-level behavior.
- `docs/hld/04-opc-and-packaging.md`, main-part content types and opaque part
  preservation.
- `docs/hld/06-presentationml-model.md`, output-only slideshow conversion and
  executable payload ownership.
- `docs/hld/10-bindings-spec.md`, native facade public surface.
- `docs/hld/12-testing-strategy.md`, source-built package fixtures.
- `docs/hld/15-build-and-toolchain.md`, public API and package riders.

## Approach

Add the six modern PresentationML main-part content types to `oxml-opc` and one
additive native enum:

```rust
pub enum PresentationPackageClass {
    Presentation,
    MacroEnabledPresentation,
    Template,
    MacroEnabledTemplate,
    Slideshow,
    MacroEnabledSlideshow,
}

impl Presentation {
    pub fn package_class(&self) -> Result<PresentationPackageClass>;
    pub fn to_bytes_as(&self, class: PresentationPackageClass) -> Result<Vec<u8>>;
    pub fn save_as_package_class(
        &self,
        path: impl AsRef<Path>,
        class: PresentationPackageClass,
    ) -> Result<()>;
}
```

Opening accepts only those six exact main content types and retains the class
in the existing OPC content-type table. Ordinary `to_bytes` and `save` preserve
the opened class. Output-specific conversion clones the staged package and
changes only the main-part override. It never removes or rewrites VBA, OLE,
ActiveX, signatures, relationships, or unrelated content types. The existing
`save_as_show` remains and delegates to ordinary `Slideshow` output.

The implementation stays in `crates/rpptx/src/lib.rs`. No new module, file,
dependency, trait, generic, feature, binding method, or renderer behavior is
added. The native additions are additive in the pre-1.0 API and require release
review before publication.

## Rejected alternatives

- Infer class from the output extension. File extensions are caller paths and
  do not authenticate package content.
- Store a second package-class field. The existing main-part content-type
  override is already the authoritative serialized identity.
- Remove VBA when selecting an ordinary class. Output class conversion must not
  destroy preserved executable payloads.
- Add Python, WASM, or CLI methods. This story is a native package boundary.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `modern_presentation_package_variants_reopen_with_original_class_and_payloads` | PPTM, POTX, POTM, PPSX, and PPSM source-built fixtures reopen in their original class with exact VBA bytes and relationship identity. |
| conversion | `package_class_conversion_changes_only_the_main_content_type` | Every output class differs from the source only at the presentation override and leaves the live presentation unchanged. |
| regression | `ordinary_save_preserves_opened_template_and_macro_classes` | `to_bytes` and `save` retain the opened class rather than collapsing to PPTX. |
| rejection | `unknown_presentation_main_content_type_fails_closed` | An unrecognized main content type never acquires a modern package identity. |
| compatibility | `save_as_show_changes_only_the_main_content_type` | The established method remains source-compatible and output-only. |

The required gate is
`modern_presentation_package_variants_reopen_with_original_class_and_payloads`.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Package graph and executable content**: read the packaging and
  PresentationML HLD. Prove complete opaque part and relationship preservation,
  exact main-part override selection, and output-only conversion.
- **Public API**: read the bindings and build HLD. Run rustdoc with warnings
  denied, README inventories, patched publish dry-runs, and archive-size checks.
- **Security-sensitive payloads**: reuse F-218 inventory to prove VBA bytes are
  never executed, decoded, removed, or silently reclassified.

## Hash harness

Expected to remain unchanged. Package identity methods do not affect existing
PPTX serialization or rendering.

## Implementation checklist

- [x] Record the 49-entry baseline and add the five real package-class tests.
- [x] Add exact modern main-part content types and the additive native enum.
- [x] Preserve opened class on ordinary saves and add output-only conversion.
- [x] Prove exact executable payload, relationship, signature, and unrelated
  part preservation across all five new variants.
- [x] Update exactly the listed HLD files and pass the routed public-package
  riders.
- [x] Complete with a zero-defect, zero-smell microscope pass.

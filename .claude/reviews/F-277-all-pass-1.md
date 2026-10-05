# F-277, all, pass 1

**Reviewed**: Frozen working diff against `cbd0f2a276c5f25aedd3ce56b5616f203ed2da2e`, 13 files, 1342 additions and 91 deletions. Read the approved plan, cited HLD contracts, completed F-276 capture/import contract, progress notes, implementation and regression coverage. This pass changed only this review file.

**Verdict**: 4 defects, 0 smells, 0 nitpicks.

## Defects

### D1, body replacement validates an entry without its inherited namespace scope

`crates/rdocx-oxml/src/glossary.rs:281`

`replace_body_content_xml` validates the edited raw `docPart` as a standalone XML document. A valid producer entry can inherit its Word prefix and extension prefixes from `glossaryDocument` or `docParts`. Its retained raw entry does not contain those declarations. Parsing the entry earlier uses inherited Word prefixes, but the strict validation at this line does not receive the enclosing namespace scope. Updating that entry from a fragment therefore fails with an unbound prefix, even though the package and replacement body are valid. The existing fragment-update test starts with a newly created entry whose root owns its Word declaration, so it misses this case.

Retain or reconstruct the complete inherited namespace scope for validation without discarding producer XML. Add a public fragment-update regression for a producer-prefixed entry whose Word and extension declarations exist only on ancestors, including a retained prefixed body-wrapper attribute.

### D2, missing placeholder selectors are appended after later schema children

`crates/rdocx/src/building_block.rs:994`

`set_word_children` inserts every missing child except `sdtPr` at the closing boundary. Binding an existing `docPartObj` or `docPartList` containing `docPartUnique` but no gallery or category therefore produces `docPartUnique`, `docPartGallery`, `docPartCategory`. If category already exists and gallery is absent, gallery likewise lands after category. Both violate the required gallery, category, unique sequence. The current regression supplies all three children, so it exercises replacements only.

Insert missing selectors in schema order while retaining the existing discriminator, unknown siblings and attributes. Cover both selector variants with missing gallery and category and with an existing unique child. The sequence is also encoded in the [Open XML SDK Wordprocessing schema](https://raw.githubusercontent.com/dotnet/Open-XML-SDK/main/generated/DocumentFormat.OpenXml/DocumentFormat.OpenXml.Generator/DocumentFormat.OpenXml.Generator.OpenXmlGenerator/schemas_openxmlformats_org_wordprocessingml_2006_main.g.cs).

### D3, the new glossary capture path bypasses the existing block grammar check

`crates/rdocx/src/building_block.rs:468`

`crates/rdocx/src/document.rs:1097`

A caller can supply a typed body containing `BodyContent::RawXml` with a direct Word run, for example `<w:r><w:t>inline</w:t></w:r>`. Creation checks lexical XML and dependency references, but `validate_relationship_free_body` does not reject this known inline root. The body parser preserves it as raw XML. Later glossary capture calls `from_part_content`, which projects the selected bytes without the block grammar checks used by `DocumentFragment::from_range` at `crates/rdocx/src/document.rs:1179`. `package_authoritative_body_fragment` copies direct preserved nodes, so this path admits an inline run as direct glossary body content and can import it into a block destination.

Apply the same namespace-aware block grammar inventory to glossary creation and capture. Known inline Word roots and inline content controls must fail atomically, while genuinely opaque supported block extensions remain preserved. Add public regressions for a direct raw Word run and an inline SDT, proving both glossary creation/capture and destination publication obey the F-276 boundary contract.

### D4, the capability matrix still describes the previous incomplete lifecycle

`docs/hld/02-scope-and-non-goals.md:251`

DOCX-044 still marks creation and removal as `N`, mutation and save-reopen as `P`, and retains `F-277` as the owner of a partial row. This contradicts the approved modeled lifecycle and its new public create, update, remove, insert and save-reopen coverage. Updating the prose below the table does not update the machine-readable contract. Once F-277 becomes done, this row also violates the live-owner requirement checked at `scripts/test_sprint_workflow.py:7725`.

Reconcile the row with the delivered modeled capability and name its evidence. Apply the same explicit opaque-content boundary convention used by completed neighboring rows. Update the policy expected-owner inventory consistently if this row becomes complete. If an actual modeled capability remains partial, identify that specific capability and its live owner instead of leaving the completed F-277 as owner.

## Smells

0 smells.

## Nitpicks

0 nitpicks.

## Not found

- **Panics**: No additional confirmed panic or unchecked-boundary defect found in the changed code.
- **Structure**: No new speculative trait, generic parameter, dependency, feature flag or forwarding wrapper found.
- **Correctness and contract**: Reviewed snapshot checks, unique names, first creation, last removal, physical glossary relationship scope, staged publication, companion import reuse and glossary drawing-ID reservation. No additional confirmed defect beyond D1 through D4.
- **OOXML**: Reviewed raw entry retention, structural span edits, discriminator retention and namespace-complete fragment extraction. D1 through D3 record the confirmed namespace and grammar failures.
- **Tests**: Reviewed the named public gate and lifecycle, stale snapshot, relationship rejection, placeholder and fragment-update regressions. They cover meaningful new behavior, but miss the inputs described in D1 through D3. Worker-reported scoped gates and packaging results do not substitute for those missing cases.

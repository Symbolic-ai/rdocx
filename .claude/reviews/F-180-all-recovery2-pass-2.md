# F-180, all aspects, recovery2 pass 2

**Reviewed**: Entire uncommitted F-180 implementation diff, 9 files, 6,871 additions and 2,016 deletions, plus the approved plan, cited HLD sections, all prior reviews, and the complete progress record
**Verdict**: 2 defects, 1 smell, 0 nitpicks

## Defects

### D1, embedded tabs and line breaks bypass the projected-run ceiling
`crates/rdocx/src/odt.rs:490`

The scan counts each nonempty text or field content item as one projected run.
`write_odf_text` can expand one such item into any number of `text:tab` and
`text:line-break` elements, and F-179 charges each resulting inline piece as a
separate run while rebuilding the paragraph. A single source text containing
100,001 tabs therefore passes the writer's 100,000-run ceiling as one run and
fits comfortably under the XML size and node ceilings, but the package fails
its own reader boundary on reopen. The writer must charge the exact inline
pieces that its serialization makes F-179 project.

### D2, conflicting first-line and hanging indents lose one property silently
`crates/rdocx/src/odt.rs:1881`

The retained paragraph model can contain both `ind_first_line` and
`ind_hanging`, whether from malformed producer XML, style merging, or direct
model construction. Serialization always selects `ind_first_line` and omits
`ind_hanging`. Neither the validation nor
`paragraph_properties_have_unsupported` classifies this collision, so reopen
loses the hanging value without the required path-aware diagnostic. The writer
must either reject the conflicting state or diagnose the omitted property
while applying a defined projection.

## Smells

### S1, the exact loss matrix still leaves diagnostic branches unprotected
`crates/rdocx/src/odt.rs:6390`

The regression now covers applied style identities, standalone line rules,
and orphan numbering levels, but it still does not construct or assert several
promised source categories. These include hyperlink wrappers, deleted text,
page and column breaks, and retained run XML. No other focused writer test
asserts those diagnostic paths and messages. A change that silently drops one
of these categories can therefore leave the test named for unsupported
document content green. Extend the exact ordered matrix, or add equivalent
exact focused assertions for every remaining branch.

## Nitpicks

None.

## Recovery2 pass 1 findings verified

- Applied paragraph and character styles now receive exact `pStyle` and
  `rStyle` identity-loss diagnostics while their effective formatting remains
  materialized.
- A line rule without spacing and a numbering level without an id now receive
  stable property-path diagnostics. The exact ordered regression includes both
  cases.
- HLD15 now matches the actual workspace manifest. ZIP 8.1 has disabled
  defaults and enables only `deflate-flate2-zlib-rs`, with Deflate64 and clock
  timestamp support disabled.
- Every generated ZIP entry uses constant options. The determinism test asserts
  the fixed timestamp, permissions, compression, raw name, file and encryption
  state, comments, central extra data, and absence of a local extra field.

## Not found

- **Formatting and content**: Apart from D2, no additional defect was found in
  effective paragraph and run styles, exact whitespace, field fallback text,
  headings, direct numbering cancellation, list kind and level, or sibling
  preservation.
- **Tables, lists, and media**: No additional defect was found in nested-list
  emission, list continuation markers, horizontal or vertical spans,
  covered-cell placement, multiple cell paragraphs, relationship type and
  target-mode checks, supported image bytes, or exact positive image
  dimensions.
- **Packaging and determinism**: No defect was found in the stored first
  `mimetype`, fixed entry order and metadata, manifest membership, MIME
  agreement, media encounter order, repeated-write bytes, fixed namespaces,
  ODF child order, or generated XML well-formedness.
- **Bounds and panics**: Apart from D1, no reachable untrusted-input panic or
  additional mismatch was found in block, row, cell, XML node, diagnostic,
  media, entry, part, total-output, table-geometry, or style-allocation bounds.
  Writer `expect` sites rely on facts established by the immutable scan.
- **Atomicity and ownership**: Serialization does not mutate the source
  document or its retained XML. Failed staging leaves the existing destination
  unchanged and cleans the attempted sibling file.
- **API and structure**: The additive native surface remains limited to
  `OdtWriteResult`, `Document::to_odt_bytes`, and `Document::save_odt`. No new
  crate, module, source file, dependency, feature, trait, generic parameter,
  wrapper-only abstraction, Python surface, WASM surface, or CLI surface was
  added.
- **HLD and file scope**: The six changed HLD files exactly match the approved
  impact list and describe current behavior. No contradiction was found in an
  unlisted HLD file.
- **Focused verification**: All 29 selected ODT writer unit tests and the
  public writer round-trip integration test pass. `git diff --check` and the
  existing review prose check also pass before this review artifact is added.

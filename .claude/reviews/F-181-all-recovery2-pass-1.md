# F-181, all recovery2, pass 1

**Reviewed**: the complete working tree diff across 16 files with 6,180
additions and 4 deletions, including the untracked EPUB writer, all six earlier
review records, the second-recovery progress note, the approved plan, and the
cited HLD and risk contracts
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, paragraph-local namespace aliases can misattribute revision diagnostics

`crates/rdocx/src/epub.rs:1830`

The raw-marker classifier receives only namespace declarations retained on the
document root. A paragraph can validly declare one local prefix for the Word
namespace and another local prefix for a foreign namespace. Captured child
fragments do not repeat either inherited declaration, so both raw markers are
classified as unresolved. If the foreign marker precedes the typed Word marker
at the same run boundary, the fallback matching loop consumes the foreign item
against the Word revision count. The foreign item then receives no diagnostic,
while the real Word wrapper receives both its typed diagnostic and an
unmodelled-XML diagnostic. The focused test avoids this collision by declaring
the foreign prefix on the document root, so it does not prove the general
inherited-alias contract.

### D2, the PNG structural validator accepts a second IHDR chunk

`crates/rdocx/src/epub.rs:3309`

The validator requires the first chunk to be `IHDR`, checks CRCs, and requires
an `IDAT` before the terminal `IEND`, but it never rejects another `IHDR` or
enforces the remaining critical-chunk order. A valid fixture with a
CRC-correct duplicate `IHDR` inserted before `IDAT` passes both
`oxml_media::probe` and this loop, then is packaged as a supported core image.
PNG permits exactly one `IHDR`, so byte sniffing plus this check does not yet
establish the HLD's strict structural-validation claim.

### D3, a document background is dropped without a diagnostic

`crates/rdocx/src/epub.rs:463`

Diagnostic collection visits core metadata, custom metadata, body content,
and final section properties, but never visits the modeled
`CT_Document::background_xml` payload. The render input starts with a fresh
document and never copies that background. An opened DOCX with a visible
`w:background` therefore loses one retained raw source item while the EPUB
result reports no corresponding location-aware loss.

### D4, patterned or invalid table-cell shading is simplified silently

`crates/rdocx/src/epub.rs:697`

Cell-property diagnostics cover width, wrapping, direction, conditional style,
borders, and raw XML, but not `CT_TcPr::shading`. The table projection forwards
that shading to the outbound HTML helper, which consumes only a valid nonwhite
fill and ignores the pattern and foreground colour. A striped cell, a
foreground-only cell, or a cell with an invalid fill is therefore changed or
omitted without the exhaustive shading diagnostic promised by the updated HLD.

### D5, default paragraph and run style effects disappear without diagnostics

`crates/rdocx/src/epub.rs:730`

The paragraph scanner checks style loss only when the paragraph carries an
explicit `style_id`. The render-style projection retains only outline levels
and removes every run property plus all other paragraph properties. A paragraph
with no direct style id still inherits the document's default paragraph style
in Word, including alignment, spacing, fonts, and run formatting, but the EPUB
uses its fixed CSS defaults and emits no style-loss diagnostic. This is a
common visible source effect, not merely an unused style definition, and it
falls outside the promised per-item lossy-conversion reporting.

## Smells

None.

## Nitpicks

None.

## Not found

- Targeted second-recovery behavior: heading labels use bounded direct
  projected runs. Style-derived deep headings, final section properties, every
  non-basic direct-run underline variant, paragraph and run shading, and both
  preserved deleted-text losses have stable diagnostics. D1, D2, and D4 cover
  the remaining namespace, media, and shading boundaries.
- SVG and media selection: SVG and extension-only raster claims are omitted,
  body references select media, and image sources remain correlated to their
  exact attributes. Apart from D2, no active-media packaging or MIME-selection
  defect was found.
- Archive and EPUB structure: no additional defect was found in the stored
  first `mimetype`, container, OPF metadata, manifest, spine, navigation,
  stylesheet, XHTML flow structure, media deduplication, compression choices,
  fixed timestamps, or ZIP entry order.
- Lists, headings, and hyperlinks: source-ordered spine splitting, nested
  navigation, numbered headings, list identity, interruption continuation,
  nested counter resets, list-depth bounds, and absolute-URI filtering remain
  intact.
- Bounds, panics, determinism, and atomicity: no additional unchecked
  production panic, overflow, recursion, unbounded export allocation, unstable
  output choice, destination truncation, staging leak, or live source mutation
  was found.
- Public API, dependency graph, HLD scope, and structure: the additive native
  API, approved private module, existing `zip` dependency, unchanged Python,
  WASM, and CLI surfaces, and six modified HLD files match the plan. The HLD's
  namespace, media, and diagnostic claims remain premature until D1 through D5
  are resolved.
- Oracle and focused evidence: all 32 ordinary EPUB tests passed and the exact
  EPUBCheck test remained ignored because `EPUBCHECK_JAR` was not set for this
  review run. The tracked CI steps still verify the reviewed release and JAR
  digests and invoke the combined source-built oracle fixture.

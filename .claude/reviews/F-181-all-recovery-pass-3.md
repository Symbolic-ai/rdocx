# F-181, all recovery, pass 3

**Reviewed**: the complete working tree diff across 15 files with 5,348
additions and 4 deletions, including the untracked EPUB writer, all five prior
review records, the recovery progress note, and the cited HLD and risk
contracts
**Verdict**: 8 defects, 0 smells, 0 nitpicks

## Defects

### D1, heading extraction bypasses the content-control projection bound

`crates/rdocx/src/epub.rs:292`

Heading text is built with `CT_P::text`, which traverses direct runs and runs
nested inside paragraph content controls. The preflight deliberately does not
measure those content-control trees, and the render projection drops them. A
`Heading1` paragraph can therefore hold a content control with more than the
8 MiB source limit, pass preflight, then allocate and copy its full text into
the heading and navigation strings. The same dropped content also appears in
navigation while it is absent from the chapter body. This contradicts both the
before-allocation bound and the documented rule that content-control subtrees
are diagnosed and never copied into the EPUB projection.

### D2, revision de-duplication is not prefix-tolerant for inherited bindings

`crates/rdocx/src/epub.rs:1730`

The raw marker parser accepts an unresolved prefix only when its literal name
is `w`. A valid Word document may bind another prefix such as `x` to the Word
namespace on an ancestor and use `x:ins`, `x:bookmarkStart`, or another typed
marker in a paragraph. The paragraph parser recognizes that marker through its
inherited binding, but the captured marker subtree does not repeat the ancestor
declaration. `NsReader` therefore reports `Unknown(x)` here, the typed marker
does not consume its retained raw view, and the exporter emits both a typed
loss diagnostic and an unmodelled paragraph XML diagnostic for one source
item. The new foreign-namespace test proves that `x:ins` is not confused with
Word XML, but it does not cover a nonstandard prefix that is validly bound to
the Word namespace.

### D3, filename fallback can package corrupt or active media as a core image

`crates/rdocx/src/epub.rs:371`

Media eligibility uses `oxml_media::resolve`, which falls back to the part-name
extension when byte sniffing fails. A public caller can embed arbitrary bytes
with a name ending in `.png`, and this path then packages those bytes as
`image/png` with no diagnostic or validation. Malformed SVG, including active
or remotely referencing SVG, is likewise copied without checking its XML or
manifest properties. The resulting archive can fail EPUBCheck or expose active
content even though the writer reports a supported core image. The combined
oracle fixture contains only one valid PNG, so the declared conformance gate
does not exercise this path.

### D4, style-derived deep headings still become paragraphs without a diagnostic

`crates/rdocx/src/epub.rs:737`

The deep-heading check resolves direct `HeadingN` names and direct paragraph
outline levels only. A paragraph using a named style whose sole paragraph
property is `outline_lvl = 6`, `7`, or `8` receives neither the style-loss
diagnostic nor the deep-heading diagnostic. That style survives the bounded
style projection, then the outbound emitter maps its level above `h6` to `p`.
This silently loses heading semantics despite the HLD promise that reduced
heading levels are diagnosed. The focused recovery test covers a visually
formatted named style and a direct deep outline level, not this style-derived
case.

### D5, final section properties are dropped without any diagnostic

`crates/rdocx/src/epub.rs:1957`

The projection clears the body-level `sect_pr`, while diagnostic collection
only checks section properties embedded in paragraph properties. Final page
geometry, columns, header and footer references, title-page behavior,
revisions, and retained section XML can therefore all disappear without a
location-aware report. In particular, a multicolumn final section is a modeled
source item whose layout is simplified by reflow, but the returned diagnostics
claim no loss.

### D6, every non-none underline style is silently reduced to single underline

`crates/rdocx/src/epub.rs:2064`

Filtering `ST_Underline::None` fixes the visible reversal from recovery pass 2,
but every other modeled variant is forwarded unchanged to an HTML helper that
treats mere presence as `<u>`. `Words`, `Double`, `Thick`, `Dotted`, `Dash`,
`DotDash`, `DotDotDash`, and `Wave` all become one single underline, and
`scan_run_properties` reports none of those simplifications. This violates the
documented per-property diagnostic contract.

### D7, patterned paragraph and run shading is simplified without a diagnostic

`crates/rdocx/src/epub.rs:2040`

`crates/rdocx/src/epub.rs:2075`

Both projections copy the complete modeled shading value, but the outbound
CSS helper consumes only a valid nonwhite `fill`. It ignores the shading
pattern and foreground color, and can omit the property completely when the
fill is white or invalid. Diagnostic collection does not inspect paragraph or
run shading, so a striped or foreground-only shading property changes visibly
without the stable simplified-property report promised by the HLD.

### D8, preserved deleted text loses its revision diagnostic

`crates/rdocx/src/epub.rs:992`

The first match arm handles preserved `DeletedText` together with ordinary
preserved text and emits only the spacing-normalization diagnostic. It prevents
the later `DeletedText` arm from reporting that deletion semantics were
flattened. The XHTML still renders the deleted text as ordinary live text, so
the one diagnostic returned for this source item names the spacing change but
not the more consequential revision loss.

## Smells

None.

## Nitpicks

None.

## Not found

- Targeted recovery behavior: bounded cloning of direct paragraph, run,
  drawing, table, row, and cell projections is present. Hyperlink spans are
  range-checked. Numbered headings retain heading elements and anchors. Nested
  ordered counters reset when a parent advances. Explicit no-underline values
  remain non-underlined. Alternate drawings, drawing names and extents,
  preserved spacing, and column breaks have stable diagnostics. D1, D2, D4,
  D6, D7, and D8 identify the remaining boundaries around those fixes.
- Lists and hyperlinks: no additional defect was found in list identity,
  interruption continuation, standard marker CSS, nested-list XHTML ownership,
  table-cell list diagnostics, absolute-URI validation, or exact hyperlink
  source escaping.
- Archive and generated-document structure: apart from D3, no additional
  defect was found in the stored first `mimetype`, container, OPF metadata,
  manifest, spine, navigation, stylesheet, fixed timestamps, compression
  choices, media deduplication, or ZIP entry order.
- XML and XHTML safety: generated metadata, navigation, content, hyperlink,
  and alternative-text values are escaped and checked for XML 1.0 characters.
  Page breaks are lifted out of phrasing containers. No additional tag-order,
  markup-breakout, or mismatched-tag defect was found.
- Determinism and atomicity: no clock, random value, unstable output choice,
  destination truncation, staging-file leak, source mutation, or retained DOCX
  XML mutation defect was found.
- Panics: no unchecked production `unwrap`, `expect`, caller-controlled index,
  slice, recursion, or arithmetic-overflow defect was found beyond D1's
  unbounded heading projection.
- Public API, dependency graph, HLD scope, and structure: the additive native
  surface, private approved module, existing `zip` dependency, unchanged
  Python, WASM, and CLI surfaces, and six modified HLD files match the approved
  plan. The HLD's diagnostics and media-conformance claims remain premature
  until D1 through D8 are resolved.
- Focused evidence: all 29 ordinary EPUB tests passed. The combined fixture
  also passed the exact checksum-verified EPUBCheck 5.3.0 JAR. Those green
  fixtures do not exercise D1 through D8.

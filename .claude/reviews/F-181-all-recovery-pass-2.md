# F-181, all recovery, pass 2

**Reviewed**: the complete working tree diff across 14 files with 4,751
additions and 4 deletions, plus the approved plan, all four earlier reviews,
the recovery progress note, and the cited HLD and risk contracts
**Verdict**: 10 defects, 0 smells, 0 nitpicks

## Defects

### D1, preflight still clones unmeasured typed property trees
`crates/rdocx/src/epub.rs:1873`

The paragraph projection clones `CT_PPr` wholesale. The run and table
projections do the same for `CT_RPr`, table, row, and cell properties at
`crates/rdocx/src/epub.rs:1909` and `crates/rdocx/src/epub.rs:1978`.
Those values can own typed revisions with their complete captured XML and
nested typed content. They also carry caller-controlled strings such as font
names and style values. The preflight at `crates/rdocx/src/epub.rs:2731`
measures selected raw vectors but not the typed revision trees or ordinary
property strings. A document with a small displayed body and a very large
`w:pPrChange`, `w:rPrChange`, or other typed property payload therefore passes
preflight and duplicates that payload before any EPUB limit can reject it.
Rebuilding fields and drawings without their raw payload fixes the recovery
examples, but the approved before-allocation bound is still incomplete.

### D2, a public hyperlink span can bypass every projection bound
`crates/rdocx/src/epub.rs:1892`

The render projection copies `run_start` and `run_end` without checking that
they are ordered and bounded by the paragraph's run count. The reused emitter
then inserts every index in that range into a map at
`crates/rdocx-html/src/emitter.rs:218`. A caller-built `HyperlinkSpan` with one
run and `run_end = usize::MAX` passes the source checks, then performs an
effectively unbounded loop and allocation before XHTML is returned. Counting
one projected node per hyperlink does not bound the span expansion.

### D3, a numbered heading is no longer a heading in the XHTML
`crates/rdocx/src/epub.rs:1703`

The spine emitter tests for a list before it renders the source block. A
paragraph with `Heading1` and valid numbering is therefore included in the
outline and starts a chapter, but `emit_list_level` extracts only its list-item
content and places the anchor on `li`. No `h1` survives in the chapter. Numbered
headings are a normal Word construct, so this silently breaks the supported
heading semantics and makes the navigation target a list item instead of its
source heading.

### D4, nested ordered lists continue across a new parent item
`crates/rdocx/src/epub.rs:1745`

Counters are retained only by `(num_id, level)` and no deeper counter is reset
when a higher-level item advances. For one numbering instance containing
parent 1, child a, parent 2, child a, the second child list opens with
`start="2"` rather than restarting at its level definition. The ordinary
block-interruption recovery is correct for one level, but the same global map
changes the default restart semantics of nested ordered lists.

### D5, revision diagnostic de-duplication ignores namespaces
`crates/rdocx/src/epub.rs:1633`

`marker_raw_kind` strips the prefix and classifies only the local name. A
foreign preserved element such as `x:ins` at the same run boundary as a typed
Word insertion can consume the typed insertion's de-duplication count. If the
foreign element appears first, it receives no unmodelled-XML diagnostic and
the preserved raw view of the real Word insertion is diagnosed separately.
The XML-whitespace fix works, but the one-report rule is now dependent on raw
element order and an unrelated namespace.

### D6, explicit no-underline text becomes underlined
`crates/rdocx/src/epub.rs:1909`

The EPUB projection forwards `ST_Underline::None` unchanged and reports no
loss for it. The reused HTML semantics treat every present underline value as
true at `crates/rdocx-html/src/css.rs:197`, so an explicitly non-underlined run
is emitted inside `u`. This is a visible reversal of a modeled property, not a
loss that the current diagnostic can explain.

### D7, alternate drawing payloads are dropped without a diagnostic
`crates/rdocx/src/epub.rs:1913`

Every run projection replaces `alt_drawings` with an empty vector. Diagnostic
collection walks `run.content` and `run.extra_xml`, but never visits this
preserved alternate-drawing collection. A parsed run that retains an
alternate-content drawing therefore loses that source item with no
location-aware report, contrary to the dropped raw-item contract.

### D8, supported images still lose modeled drawing properties silently
`crates/rdocx/src/epub.rs:1971`

The projection clears the drawing name and raw drawing XML, while the outbound
HTML emitter does not use the retained extent to set image dimensions. The
drawing scanner at `crates/rdocx/src/epub.rs:1008` reports relationship failure
or floating placement only. A supported inline image with a modeled name,
nondefault extent, or preserved drawing child consequently succeeds without a
diagnostic for the dropped or simplified property. Preserving `descr` as
alternative text fixes the named recovery case but not the surrounding drawing
contract.

### D9, preserved Word text spacing collapses without a diagnostic
`crates/rdocx/src/epub.rs:1919`

`CT_Text::preserve_space` is copied into the projection, but the generated
XHTML emits ordinary text under CSS with no whitespace-preservation rule.
Leading, trailing, and repeated spaces marked with `xml:space="preserve"`
therefore collapse in an EPUB reader. The scanner considers only the text
bytes and emits no simplified-text diagnostic, so supported text can change
visibly while the result reports no loss.

### D10, column breaks silently become line breaks
`crates/rdocx/src/epub.rs:1922`

The projection forwards every break kind to the HTML emitter. That emitter
maps `BreakType::Column` to `br`, while the EPUB recovery only lifts page-break
`hr` elements. Diagnostic collection does not inspect line, page, or column
breaks. A modeled column break therefore becomes an ordinary line break with
no report, despite the contract that every simplified property has one stable
diagnostic.

## Smells

None.

## Nitpicks

None.

## Not found

- Recovery D2 through D10: heading-to-spine assignment and anchor lookup are
  linear. Direct deep outlines and `lvl_jc` are diagnosed. Table-cell lists are
  diagnosed. A one-level ordered list continues across an ordinary block.
  Brackets are confined to validated IP literals. XML whitespace is recognized
  during revision matching. Supported image descriptions become correlated
  alternative text. Every custom property receives a stable diagnostic. D1
  describes the remaining allocation boundary from the earlier D1.
- Archive and package structure: no additional defect was found in the stored
  first `mimetype`, container, package metadata, manifest, spine, navigation,
  stylesheet, media inventory, fixed timestamps, compression choices, or ZIP
  entry order.
- URI and XML safety: apart from D5, no additional malformed-URI, forbidden XML
  character, attribute escaping, or markup breakout defect was found.
- Determinism, atomic save, and source preservation: no clock, random value,
  unstable output iteration, destination truncation, live source mutation, or
  retained DOCX mutation defect was found.
- Panics: no unchecked production `unwrap`, `expect`, caller-controlled index,
  slice, or arithmetic-overflow defect was found beyond the unbounded range in
  D2.
- Public API, dependency graph, structure, and HLD scope: no unapproved crate,
  trait, generic, feature, binding surface, dependency, or HLD file change was
  found. The private module and six HLD files match the approved plan.
- Oracle and focused evidence: all 25 ordinary EPUB tests passed. The combined
  source-built fixture also passed the exact checksum-verified EPUBCheck 5.3.0
  JAR. Those green fixtures do not exercise D1 through D10.

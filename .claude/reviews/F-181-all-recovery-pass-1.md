# F-181, all recovery, pass 1

**Reviewed**: the complete working tree diff across 13 files with 4,164
additions and 4 deletions, plus the approved plan, all three original reviews,
the recovery progress note, and the cited HLD and risk contracts
**Verdict**: 10 defects, 0 smells, 0 nitpicks

## Defects

### D1, preflight still clones unmeasured preservation payloads

`crates/rdocx/src/epub.rs:1855`

The render projection clones every `RunContent` value in full. A parsed field
retains its complete source XML and cached formatting tree, while an inline or
anchored drawing retains its complete raw drawing XML. The preflight at
`crates/rdocx/src/epub.rs:2562` measures selected public field strings and only
counts drawing occurrences. It never measures those retained payloads. A DOCX
with a small field display or image relationship beside a very large preserved
field or drawing subtree therefore duplicates that subtree before any EPUB
limit can reject it. Removing content-control and table-grid clones fixes the
pass 3 examples, but the approved before-allocation bound is still not true.

### D2, the accepted heading limit still permits quadratic export work

`crates/rdocx/src/epub.rs:131`

Every spine item scans every heading to assign its link. The source limits
permit up to 100,000 body items and projected nodes, so 100,000 empty
`Heading1` paragraphs pass preflight and create 100,000 root spine items. This
loop then performs about ten billion range checks before serialization. The
later per-block heading searches add another quadratic path. A bounded source
can therefore hold the exporter for an impractical amount of time instead of
failing within its resource envelope.

### D3, direct deep outline levels still lose heading semantics silently

`crates/rdocx/src/epub.rs:713`

The new deep-heading diagnostic is nested under a paragraph `style_id` and
uses a helper that recognizes only `HeadingN` style names. A paragraph with a
direct `outline_lvl` of 6, 7, or 8 has no matching EPUB diagnostic. The reused
HTML emitter maps those levels above `h6` to `p`, so the paragraph loses its
heading semantics with no report. The public paragraph outline-level API can
produce this input, and the HLD promise covers reduced heading levels rather
than only built-in style names.

### D4, list marker alignment is still discarded without a diagnostic

`crates/rdocx/src/epub.rs:1941`

`CT_Lvl::lvl_jc` is a modeled marker property, but `ListInfo` carries only the
kind, start value, and CSS marker format. The diagnostic scan covers custom
marker text, paragraph and run formatting, suffixes, and raw XML, but it never
reports `lvl_jc`. A right-aligned or centered Word marker consequently becomes
the browser default with no location-aware diagnostic. This leaves the pass 3
marker-semantics recovery and the HLD simplified-property promise incomplete.

### D5, lists inside table cells lose all marker semantics

`crates/rdocx/src/epub.rs:1667`

The EPUB list reconstruction runs only when a top-level body item is a
paragraph. A table is projected as one source block. Its cell paragraphs reach
`rdocx-html` through the ordinary table path, which calls `emit_paragraph` at
`crates/rdocx-html/src/emitter.rs:535` without list detection. A resolved
decimal, Roman, letter, bullet, or unmarked list inside a cell therefore emits
plain paragraphs with no markers. Because the numbering definition resolves,
the loss scanner also emits no unresolved-list diagnostic. This contradicts
the supported lists and tables contract.

### D6, an interrupted list with the same identity restarts from its definition

`crates/rdocx/src/epub.rs:1714`

Each newly opened ordered list uses the level definition's initial `start`
value. A non-list paragraph closes the current list, and a later paragraph
with the same `num_id` opens another `ol` at that original value. Word keeps
that numbering instance active across the interruption, so a list starting at
3 with two items resumes at 5, while this EPUB resumes at 3. The distinct-list
test covers adjacent different identities but does not prove continuation of
one identity, despite the HLD promise that list identity and restart values
remain distinct.

### D7, RFC 3986 validation still allows brackets outside an IP literal

`crates/rdocx/src/epub.rs:1449`

The global character allowlist accepts `[` and `]` anywhere in the URI. The
HTTP-family branch validates brackets in the authority, but it never rejects
them in the path, query, or fragment. A target such as
`https://example.com/a[b]` is therefore emitted as supported even though raw
brackets are reserved for an IP literal and are not valid `pchar`, query, or
fragment characters under RFC 3986. Non-HTTP allowlisted schemes can also put
an invalid bracketed authority through the unchecked generic path. The pass 3
IP-literal fixes do not establish the HLD's syntactically valid absolute-URI
contract.

### D8, revision de-duplication rejects valid XML whitespace

`crates/rdocx/src/epub.rs:1593`

The raw marker classifier ends an element name only at a literal space, slash,
or closing bracket. XML also permits tab, carriage return, and line feed as
whitespace after the qualified name. A valid wrapper such as `w:ins` followed
by a newline before its attributes parses into both the typed revision and the
preserved raw view, but this classifier does not recognize the raw view. The
export then emits one unmodelled-XML diagnostic and one flattened-revision
diagnostic for the same source item, so the pass 3 one-report recovery remains
input-format dependent.

### D9, supported images silently lose modeled alternative text

`crates/rdocx/src/epub.rs:978`

The drawing scanner reports a missing or unsupported relationship and floating
placement, but it never examines the inline or anchored drawing description.
The XHTML normalizer always inserts `alt=""` at
`crates/rdocx/src/epub.rs:2160`. A supported image whose `wp:docPr/@descr`
contains meaningful alternative text is therefore packaged with empty alt text
and no diagnostic. This is a dropped modeled property and violates the HLD's
one diagnostic for every dropped or simplified property.

### D10, custom document metadata is dropped without diagnostics

`crates/rdocx/src/epub.rs:457`

Diagnostic collection handles six unused core-property fields and then walks
the body. It never visits the parsed `Document::custom_properties` collection.
A DOCX with one or more modeled custom properties therefore loses that
document metadata without any `metadata/...` diagnostic. The recovery fixes
the six fields named in pass 3, but the current HLD says unconsumed document
metadata and every dropped modeled item are reported.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 3 recovery confirmed: content-control trees and table grids are not
  cloned into render projections. Named paragraph style losses and built-in
  `Heading7` through `Heading9` reductions are diagnosed. Standard Roman and
  letter formats retain CSS list semantics, while custom marker text, marker
  run formatting, paragraph formatting, suffixes, and raw level XML are
  diagnosed. D1, D3, and D4 describe the remaining boundaries.
- Page-break XHTML: formatted-run and cached-field page breaks are lifted out
  of phrasing containers. The combined external fixture now includes a page
  break and exact EPUBCheck 5.3.0 accepts it.
- Metadata recovery: subject, description, keywords, last-modified-by,
  created, and modified each receive stable diagnostics. D10 covers the
  remaining modeled metadata collection.
- Oracle and CI gate: the tracked test job verifies both release ZIP and JAR
  digests, exports the verified JAR path, and runs the exact ignored test as a
  required step. The reviewed JAR digest was independently confirmed and all
  23 focused EPUB tests passed with that oracle included.
- Archive and package structure: no additional defect was found in the stored
  first `mimetype`, container, OPF manifest and spine, navigation document,
  stylesheet and media paths, fixed timestamps, compression choices, or ZIP
  entry order.
- Determinism, atomic save, and source preservation: no clock, random value,
  unstable output iteration, destination truncation, live document mutation,
  or retained DOCX XML mutation defect was found.
- Panics: no unchecked production `unwrap`, `expect`, caller-controlled index,
  or arithmetic-overflow defect was found.
- Public API, dependency graph, structure, and HLD scope: no unapproved crate,
  trait, generic, feature, binding surface, dependency, or HLD file change was
  found. The six modified HLD files are exactly the approved impact list. Their
  bounds, URI, list, and diagnostic claims remain premature until D1 through
  D10 are resolved.

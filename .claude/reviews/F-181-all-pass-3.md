# F-181, all, pass 3

**Reviewed**: the complete feature implementation diff across 8 files with
3,106 additions and 4 deletions, plus both prior review records and the current
progress note
**Verdict**: 8 defects, 0 smells, 0 nitpicks

## Defects

### D1, preflight still skips source trees that the renderer clones

`crates/rdocx/src/epub.rs:1602`

Every rendered body block is cloned in full before the HTML projection drops
unsupported children. The preflight does not measure paragraph content
controls, table and row content controls, cell content controls, or the table
grid. One parsed content control can recursively own a large typed tree and one
caller-built grid can contain an arbitrarily large column vector. Both are
duplicated here even though they do not survive the projection. The checks at
`crates/rdocx/src/epub.rs:2068` and
`crates/rdocx/src/epub.rs:2140` therefore still do not establish the approved
before-allocation bound.

### D2, paragraph styles and deep heading levels change without diagnostics

`crates/rdocx/src/epub.rs:1104`

The style projection retains only `outline_lvl`, so a paragraph using a named
style for spacing, indentation, shading, or run formatting loses that style's
visible effect. `scan_paragraph` does not report `style_id`. Direct Heading7,
Heading8, and Heading9 paragraphs are also forcibly rewritten to Heading6 at
`crates/rdocx/src/epub.rs:1604` without a diagnostic. These are stable modeled
losses, but the returned result claims none occurred.

### D3, numbering formats and marker text silently become browser defaults

`crates/rdocx/src/epub.rs:1175`

The projection discards `lvl_text`, level paragraph and run formatting, and
the emitter later reduces every non-bullet, non-None format to one generic
ordered list at `crates/rdocx/src/epub.rs:1697`. Upper Roman, lower Roman,
lettered, ordinal, and custom decimal markers therefore render as default
decimal markers. Custom bullet glyphs render as the browser's default bullet.
The paragraph scanner diagnoses only an unresolved numbering definition, so
these resolved but lossy list properties receive no location-aware report.

### D4, page breaks produce non-conforming EPUB XHTML

`crates/rdocx/src/epub.rs:1774`

The reused HTML emitter represents a page break, including a form feed in a
field display, as `hr` inside the current paragraph and possibly inside inline
formatting. Normalization only changes it to `<hr/>`. The resulting shape is
such as `<p>before<hr/>after</p>`, but `hr` is flow content and is not permitted
inside the phrasing-content model of `p`, `strong`, or `em`. The existing
EPUBCheck fixture has no page break, so it does not expose this validation
failure.

### D5, the absolute-URI check still accepts invalid RFC 3986 forms

`crates/rdocx/src/epub.rs:1364`

A bracketed host is accepted whenever the bracket content is nonempty and the
suffix resembles a port. Values such as `https://[not-an-ip]/` pass even though
an IP literal must be an IPv6 or IPvFuture address. The global character
allowlist also accepts multiple `@` delimiters in an authority and multiple
fragment delimiters. These malformed values are emitted as supported links
without diagnostics, contrary to the HLD's syntactically valid absolute-URI
contract.

### D6, one paragraph revision wrapper receives two diagnostics

`crates/rdocx/src/epub.rs:797`

The paragraph parser retains a revision wrapper as raw `extra_xml` and also
projects it into `revisions`. The raw loop diagnoses the wrapper as unmodelled
paragraph XML, then the loop at `crates/rdocx/src/epub.rs:816` diagnoses the
same source wrapper again as a flattened revision. Marker de-duplication covers
bookmarks and comment ranges only. This violates the approved one stable
diagnostic per unsupported or lossy source item contract.

### D7, dropped document metadata is never diagnosed

`crates/rdocx/src/epub.rs:457`

Diagnostic collection walks only body content. The package consumes title and
creator, but silently drops the modeled subject, description, keywords,
last-modified-by value, created date, and source modified date. These values
are public document metadata and are part of the lossy conversion, yet none
receives a stable location-aware diagnostic.

### D8, the pinned EPUBCheck test is not a CI gate

`crates/rdocx/src/epub.rs:2967`

The exact JAR digest and combined fixture are correct, but the only external
test is ignored. No tracked CI command sets `EPUBCHECK_JAR` or invokes this
ignored test, while `docs/hld/15-build-and-toolchain.md:581` now calls
EPUBCheck CI validation infrastructure. Ordinary workspace tests therefore
skip the story's declared regression oracle, so a future EPUB conformance
regression can pass the required checks.

## Smells

None.

## Nitpicks

None.

## Not found

- Prior-pass remediation: list identity, restart values, no-marker levels,
  capped list recursion, exact image-source replacement, referenced-only media,
  XML 1.0 character rejection, exact heading-anchor placement, and exact JAR
  hashing are present.
- Archive and package structure: no additional defect was found in the stored
  first `mimetype`, container, OPF manifest and spine, navigation tree, media
  inventory, entry order, timestamps, compression choices, or bounded ZIP
  cursor.
- Determinism: no clock, random identifier, unstable relationship order, or
  nondeterministic archive choice was found.
- Atomic save and source preservation: no destination truncation, live source
  mutation, or retained DOCX XML mutation defect was found.
- Panics: no unchecked production `unwrap`, `expect`, caller-controlled index,
  or arithmetic-overflow defect was found.
- Public API, HLD scope, and structure: no unapproved crate, trait, generic,
  feature, binding surface, or HLD file change was found.
- Focused evidence: all 18 ordinary EPUB tests passed. The exact EPUBCheck
  test remained ignored in this environment. Formatting of the diff and both
  prior review records passed their available checks.

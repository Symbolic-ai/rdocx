# F-181, all, pass 2

**Reviewed**: the complete working tree diff across 9 files with 1,913
additions and 4 deletions, including the untracked EPUB writer and pass 1 review
**Verdict**: 8 defects, 0 smells, 0 nitpicks

## Defects

### D1, the preflight still allocates unbounded source projections before rejection

`crates/rdocx/src/epub.rs:1221`

`measure_paragraph` calls `run.text()` to obtain a length. That call constructs
a new combined `String` before `add_source_bytes` can reject it, so one
oversized run duplicates its full text before the 8 MiB limit fires. The
writer also clones the complete styles and numbering trees at
`crates/rdocx/src/epub.rs:114` without measuring their retained raw XML, and
`media_items` collects every image relationship before checking the count at
`crates/rdocx/src/epub.rs:277`. A small body paired with a very large styles
part, numbering part, or relationship set can therefore cause allocation
growth outside every declared ceiling. The image-occurrence and media-byte
changes fix the exact base64 case from pass 1, but they do not yet satisfy the
approved before-allocation bound.

### D2, separate list identities and no-number levels change visible semantics

`crates/rdocx/src/epub.rs:686`

The list state carries only ordered versus unordered and the level. It omits
the paragraph's `num_id`, and `detect_list` classifies every format except
`Bullet` as ordered at `crates/rdocx/src/epub.rs:796`. Two adjacent ordered
lists with different `num_id` values are consequently merged into one `ol`,
so the second list continues numbering instead of restarting. A level whose
format is `ST_NumberFormat::None` also becomes a visibly numbered `ol`. Both
inputs are supported modeled lists, but the EPUB changes their source
semantics without a diagnostic.

### D3, rising list levels can exhaust the process stack

`crates/rdocx/src/epub.rs:697`

Each consecutive increase in the detected list level recurses into another
`emit_list_level` call, but this path has no depth parameter or check. The
64-level ceiling applies only to tables. A caller-built or malformed document
can provide many increasing `num_ilvl` values and matching definitions while
remaining under the body-item and projected-node ceilings, leading to tens of
thousands of recursive calls and a stack-overflow abort rather than a bounded
EPUB error.

### D4, global media-token replacement can rewrite supported document text

`crates/rdocx/src/epub.rs:837`

Image projection uses short synthetic data URIs and then replaces every
matching substring in the complete HTML fragment. For the first PNG, the
marker is the base64 representation of eight zero bytes. If ordinary source
text contains that same `data:image/png;base64,...` literal beside the image,
`String::replace` changes the text to the packaged image path as well as
changing the `img` attribute. The output silently corrupts supported text.
Replacement must be scoped to the source attribute or use a marker that cannot
collide with source content.

### D5, modeled losses and table-owned raw XML still receive no diagnostics

`crates/rdocx/src/epub.rs:450`

The paragraph scanner covers paragraph raw XML, run controls, hyperlinks, run
raw XML, note and comment references, and drawings. It does not inspect typed
comment ranges, bookmark markers, paragraph revisions, run revision metadata,
or fields whose live semantics are flattened to cached display. The table
scanner at `crates/rdocx/src/epub.rs:416` likewise ignores table and row raw
XML, row content controls, and unmodelled cell properties. Those constructs
are absent from the XHTML with no source-location diagnostic. This remains
contrary to the contract that every unsupported or lossy source item produces
one stable location-aware report.

### D6, relationship-driven media selection exports unreferenced content and mislocates losses

`crates/rdocx/src/epub.rs:277`

The media inventory starts from every image relationship on the document part
and never intersects that set with drawing occurrences in retained body
content. An orphan relationship, or an image referenced only inside a dropped
content control, is therefore written into the EPUB manifest and archive even
though no XHTML refers to it. This can disclose stale media and can reject an
otherwise small publication on the media ceiling. In addition,
`html_images` is populated before the core-media check at
`crates/rdocx/src/epub.rs:326`, so a drawing that references a non-core image
looks resolved to the source scanner. It receives one relationship-level
diagnostic instead of one location-aware diagnostic for each dropped source
occurrence.

### D7, generated XML accepts characters forbidden by XML 1.0

`crates/rdocx/src/epub.rs:1277`

`escape_xml` replaces only the five markup characters. The XHTML normalization
path at `crates/rdocx/src/epub.rs:866` also leaves all other source characters
unchanged. Native callers can place a NUL or another forbidden control
character in paragraph text, title, author, or a hyperlink target. The writer
then succeeds with XML that an EPUB reader must reject. The byte API needs to
reject or safely diagnose invalid XML characters before publishing any
archive.

### D8, the hyperlink allowlist does not establish a valid absolute URI

`crates/rdocx/src/epub.rs:583`

`safe_absolute_url` accepts a target solely because the substring before the
first colon names an allowed scheme. It accepts malformed values such as an
invalid percent escape or a backslash-bearing HTTP target, then emits them as
EPUB `href` values and reports no loss. The plan requires safe absolute
targets. Scheme filtering prevents script URLs, but it does not validate the
absolute URI syntax needed by the EPUB relationship and XHTML contracts.

## Smells

None.

## Nitpicks

None.

## Not found

- Prior-pass remediation: nested lists are now children of their owning list
  item, exact source blocks receive heading anchors, run raw XML and unsafe or
  internal hyperlinks are diagnosed, the exact EPUBCheck JAR SHA-256 is
  asserted, and one external fixture combines front matter, multiple roots,
  nested navigation, nested lists, and media.
- Package conformance and determinism: no additional defect was found in the
  `mimetype`, container, OPF manifest and spine, navigation, fixed metadata,
  entry order, timestamps, compression choice, or bounded ZIP cursor.
- Atomic save and source preservation: no destination truncation, source
  document mutation, or retained DOCX XML mutation defect was found.
- Public API, HLD scope, and structure: no unapproved crate, trait, generic,
  feature, binding surface, or HLD file change was found.
- CSS and attribute escaping: apart from D7 and D8, no markup breakout through
  projected style, metadata, heading, or hyperlink values was found.
- Focused evidence: all 10 ordinary EPUB tests passed. The exact external
  EPUBCheck test was not executable because `EPUBCHECK_JAR` is unset in this
  worktree environment. Formatting, focused clippy, diff checking, and prose
  checking passed.

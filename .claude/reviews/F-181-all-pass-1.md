# F-181, all, pass 1

**Reviewed**: the complete working tree diff, including the untracked EPUB
writer, across 9 files with 1,250 additions and 5 deletions
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, output limits run after unbounded clone and base64 expansion

`crates/rdocx/src/epub.rs:105`

The source check runs before `build_html_input`, but that call clones the whole
document and every relationship-resolved image before `media_items` applies the
16 MiB media ceiling. The writer then asks `rdocx-html` to inline every image
occurrence as base64 at `crates/rdocx/src/epub.rs:127` and checks the generated
XHTML size only afterward. A single allowed image relationship repeated by many
drawing runs can therefore grow an arbitrarily large intermediate `String`
before the bounded ZIP writer rejects the final publication. An oversized
single image is also cloned before its limit is checked. This does not satisfy
the approved contract that the byte path is bounded before allocation growth.

### D2, nested lists produce non-conforming XHTML

`crates/rdocx/src/epub.rs:127`

The EPUB writer embeds the outbound HTML fragment as XHTML and only normalizes
void elements afterward. The reused emitter closes each list item before it
opens the next nested list at `crates/rdocx-html/src/emitter.rs:60`, so a level
zero item followed by a level one item produces a `ul` directly inside another
`ul` instead of inside the parent `li`. That markup is well-formed XML but does
not conform to the HTML content model required for an EPUB XHTML content
document. Both the semantic test and the EPUBCheck fixture exercise only a
level zero list, so the invalid nested case remains undetected.

### D3, stable loss diagnostics omit dropped run and hyperlink content

`crates/rdocx/src/epub.rs:380`

The paragraph scanner examines modeled run content but never examines
`CT_R::extra_xml`, even though the HTML emitter does not serialize that
preserved run XML. It also never examines `CT_P::hyperlinks`. An internal
anchor, an empty or unsafe external target, or a relative external target that
loses its DOCX base is emitted as plain text or as a publication-relative link
without an EPUB diagnostic. A paragraph containing supported text beside one
of these losses therefore returns a successful publication with no record of
what was dropped or changed, contrary to the milestone and design diagnostic
contract.

### D4, tag-only anchor injection can link navigation to the wrong paragraph

`crates/rdocx/src/epub.rs:491`

Planned headings are derived only from direct `HeadingN` style ids, but the
HTML emitter can also produce heading tags from direct outline levels and from
style definitions. Anchor injection searches only for the next matching tag
name. If an outline-level or style-derived `h2` occurs before the next direct
`Heading2`, it receives that direct heading's anchor and the navigation entry
points to the wrong source paragraph. A conflicting direct outline level can
instead make injection fail entirely. The output therefore does not reliably
preserve `Document::document_outline()` links at the promised stable heading
anchors.

### D5, the EPUBCheck version check does not pin the reviewed oracle

`crates/rdocx/src/epub.rs:1167`

The ignored gate accepts any JAR whose output contains `5.3.0`. It never hashes
the JAR or the reviewed distribution ZIP, so a rebuilt, patched, or unrelated
JAR can satisfy the test. Recording the distribution digest in HLD prose does
not put the pin in the harness. This violates the differential-testing rule
and the approved plan's exact EPUBCheck 5.3.0 gate.

### D6, EPUBCheck does not validate the story's gate fixture

`crates/rdocx/src/epub.rs:1173`

The external test validates one heading, one flat list, and one image. It has
no front matter, second outline root, or nested heading. The separate outline
test at `crates/rdocx/src/epub.rs:1006` inspects strings but never passes that
publication to EPUBCheck. The declared regression gate requires one
source-built publication whose front matter, source-ordered spine, and nested
navigation match the outline and which EPUBCheck accepts. The current tests do
not establish that combined postcondition.

## Smells

None.

## Nitpicks

None.

## Not found

- Panics: no unchecked production `unwrap`, `expect`, or caller-controlled
  indexing defect was found.
- EPUB package structure: no additional container, OPF metadata, manifest,
  spine, navigation, media-type, or ZIP entry-order defect was found.
- Determinism: no clock, random identifier, unstable map iteration, timestamp,
  or compression-choice defect was found.
- Atomic save and source preservation: no destination truncation, source
  document mutation, or retained DOCX XML mutation defect was found.
- Public API and structure: no unapproved crate, trait, generic, feature,
  binding surface, or HLD-scope change was found.
- Focused evidence: all 6 ordinary EPUB tests passed. The ignored EPUBCheck
  test passed when run with the reviewed 5.3.0 distribution whose ZIP digest
  matched the HLD value. Those green runs do not resolve D1 through D6.

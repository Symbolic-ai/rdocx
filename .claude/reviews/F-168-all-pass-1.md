# F-168, all, pass 1

**Reviewed**: uncommitted `work/f-168-codex` diff, 4 files and 977 changed lines
**Verdict**: 9 defects, 0 smells, 0 nitpicks

## Defects

### D1, the setters do not put a watermark on every active page variant or section
`crates/rdocx/src/document.rs:1604`
`crates/rdocx-layout/src/paginator.rs:907`

`apply_watermark` updates only header relationships already present anywhere in
the document. When none exist, it creates one default header on the final
section only. A document with `titlePg` enabled but no first-page header is a
minimal trigger. The setter creates or updates the default watermark, then the
paginator selects `first_watermark` without a default fallback on page one, so
that page has no watermark. An earlier section with no explicit header has the
same failure when only the final section receives the newly created default
header. This breaks the approved every-page and every-section contract. The
tests cover one section with all three references already installed, so they do
not exercise either boundary.

### D2, adding a watermark reconstructs headers through a lossy unordered model
`crates/rdocx/src/document.rs:1633`
`crates/rdocx-oxml/src/header_footer.rs:273`
`crates/rdocx-oxml/src/header_footer.rs:360`

The mutation parses the complete existing header into `CT_HdrFtr` and writes it
back. That model retains root namespace declarations but not other root
attributes, and its writer emits every modeled paragraph before every raw
root child. For example, a valid header whose table precedes a paragraph is
rewritten with the paragraph first, and a producer `mc:Ignorable` attribute is
lost. A VML prefix declared only on a run ancestor can also be lost when the run
is reconstructed around its captured `w:pict`. Ordinary content therefore does
not remain ordered or verbatim as the plan requires, and schema-sensitive
header content can move when the only requested change was a watermark.

### D3, image watermark relationships are wrong for custom header part paths
`crates/rdocx/src/document.rs:1583`
`crates/rdocx/src/document.rs:1373`

`store_image_part` stores bytes under `/word/media` but returns a target with
the `/word/` prefix stripped. `set_image_watermark` attaches that same target
to every relationship-resolved header part. For a valid custom header such as
`/custom/headers/header.xml`, target `media/image1.png` resolves below
`/custom/headers`, not to the part stored below `/word/media`. Saving leaves the
new VML shape pointing at a missing image. The existing `relative_target`
helper is not used, and the relationship test covers only canonical `/word/`
header locations.

### D4, valid named VML colours render as black
`crates/rdocx-layout/src/engine.rs:1799`

The VML parser preserves `fillcolor` as a string, including valid named colours
such as `silver`, but layout passes it to the hex-only `Color::from_hex`.
Anything shorter than six bytes becomes black. A producer watermark using
`fillcolor="silver"` therefore renders black instead of silver. The parser test
uses only `#D9D9D9`, so the common named-colour path is untested.

### D5, an untrusted VML colour can panic layout
`crates/rdocx-layout/src/engine.rs:1799`

The same unchecked producer string reaches `Color::from_hex`, which slices the
first six bytes at fixed byte offsets. A valid XML attribute containing
multibyte text whose byte boundaries do not fall at offsets two or four, such
as `fillcolor="€€"`, panics during layout instead of preserving the shape as
unsupported or returning a layout error. This violates the no-panic boundary
for opened documents.

### D6, inactive even headers are rendered as active
`crates/rdocx-layout/src/paginator.rs:910`
`crates/rdocx-layout/src/paginator.rs:927`

Even-page selection is based only on physical page parity and the presence of
an even header reference. Word activates even headers through the document
settings `w:evenAndOddHeaders`. A document may retain an even reference while
that setting is absent or false, in which case Word uses the default header on
even pages. The new paginator instead renders the even header and its
watermark. The new test creates an even reference without activating the
global setting and therefore locks in the incorrect behavior.

### D7, unresolved image watermarks become empty image elements
`crates/rdocx-layout/src/engine.rs:1809`

When a projected `v:imagedata` relationship is external, missing, or points to
a missing part, `build_layout_input` has no scoped image entry. Layout silently
substitutes empty bytes, an empty content type, and the media id of empty data,
then emits a positioned image. The result neither renders the producer image
nor reports why it could not be resolved. A conservative projection must skip
the unresolved watermark with a diagnostic or return an error, rather than
claiming a typed render with fabricated empty media.

### D8, API ownership is detected by an unsafe raw-byte substring
`crates/rdocx-oxml/src/header_footer.rs:188`

Replacement treats any captured run child containing the bytes
`id="rdocx-watermark"` as the API-owned `v:shape`. An unrelated preserved
extension such as `<x:data>id="rdocx-watermark"</x:data>` is therefore deleted
and replaced wholesale by the new `w:pict`. The check does not establish the
element namespace, local name, or the location of the id attribute. This
violates the requirement to replace only the API-owned watermark and preserve
unrelated raw XML.

### D9, layout ignores the authored margin-relative positioning
`crates/rdocx-layout/src/engine.rs:1766`

Generated VML explicitly declares horizontal and vertical centering relative
to the margin rectangle, but layout centers the group in the full physical
page. The two positions differ whenever opposing margins are unequal. A
section with a 72 point left margin and a 144 point right margin shifts the
rendered watermark 36 points from the position described by the saved VML, so
the native render and Word render disagree. The golden uses symmetric margins
and cannot detect this.

## Smells

None.

## Nitpicks

None.

## Not found

No additional defects were found in generated direct-child order, canonical
header relationship id scoping, mutation staging, layout-cache invalidation,
generated text shaping determinism, z-order relative to body text, or the
repository structural rules. The focused watermark tests passed, 4 in
`rdocx` and 8 header/footer tests in `rdocx-oxml`. `git diff --check` also
passed.

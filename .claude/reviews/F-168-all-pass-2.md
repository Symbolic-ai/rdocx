# F-168, all, pass 2

**Reviewed**: uncommitted `work/f-168-codex` diff, 4 files and 1,719 changed lines
**Verdict**: 6 defects, 1 smell, 0 nitpicks

## Defects

### D1, first and even watermark fallbacks exist only in the native renderer
`crates/rdocx-layout/src/paginator.rs:908`
`crates/rdocx/src/document.rs:1643`

The remediation creates missing default header references only. The paginator
then borrows that default watermark when an active first or even header has no
watermark. Those borrowed fallbacks are not written to the DOCX package. For
the first section, `titlePg` with no first header produces a blank first header
in Word. For an enabled even-header mode with no even header in the first
section, Word likewise produces a blank even header. The new multi-section
test exercises exactly this mismatch in its final section. Native layout shows
the default watermark there, but the saved document has no first-header
watermark for Word to display. The writable and rendered every-page contract
therefore still fails.

### D2, adding explicit blank defaults destroys inherited ordinary headers
`crates/rdocx/src/document.rs:1690`

For every section missing a default reference, the setter installs one shared
watermark-only header. In Word, a missing header in a later section inherits
the same header type from the previous section. A document whose first section
has a company header and whose second section omits the default reference is a
minimal trigger. Before the setter, section two inherits the company header.
After the setter, its new explicit reference overrides that inheritance with a
blank header plus the watermark, so ordinary producer content disappears from
that section. The new section test starts with no inherited header content and
does not cover this preservation boundary.

### D3, even-header parity ignores section page-number restarts
`crates/rdocx-layout/src/paginator.rs:164`
`crates/rdocx-layout/src/paginator.rs:910`

The paginator carries physical page ordinality across sections and uses it
directly for even-header selection. Word uses the section's
`w:pgNumType/@w:start` value to determine the first page parity. That element
remains preserved raw by `CT_SectPr`, so the new selection code never sees it.
A section beginning on physical page two with `w:start="1"` must use its odd
default header, but this implementation selects the even header and watermark.
The even-header regression covers only a single section without a numbering
restart.

### D4, generated shapes reference VML templates that are never emitted
`crates/rdocx-oxml/src/header_footer.rs:86`

Authored text and image shapes set `type` to `#_x0000_t136` or
`#_x0000_t75`. A VML `type` value is a fragment reference to a `v:shapetype`
element whose path, fill, and stroke form the shape template. The generated
`w:pict` contains no matching shapetype, so both references dangle. The native
renderer ignores this attribute and the round-trip test only reparses its own
projection, which cannot prove that Word can render the authored VML. A new
document created by either setter therefore lacks the referenced template in
every generated header.

### D5, foreign same-local end tags terminate VML projection state
`crates/rdocx-oxml/src/header_footer.rs:658`

Start tags are checked by expanded namespace, but `shape` and `pict` end tags
are matched by local name alone. A supported `v:shape` containing an
extension child named `x:shape` before its `v:textpath` causes the foreign end
tag to consume the pending watermark. The real VML text path is then ignored.
A foreign `x:pict` end tag similarly clears the enclosing Word pict state.
This violates the namespace-aware conservative projection boundary. The
alias test does not include a foreign same-local nested element.

### D6, image watermarks bypass collision-safe media identity resolution
`crates/rdocx-layout/src/engine.rs:1829`

The layout creates the watermark image id directly with
`MediaId::from_bytes`, even though the scoped header image was already entered
into the layout's `MediaRegistry`. That registry compares complete bytes and
assigns deterministic alternate ids when compact hashes collide. Two distinct
images with the same compact hash therefore receive collision-safe ids for
ordinary layout, while a watermark using one of them reuses the colliding base
id. This breaks the one-registry layout contract and can make a consumer of
the backend-neutral layout identify the watermark as the other image. The
relationship-scope test checks input bytes only and never forces a media-id
collision.

## Smells

### S1, an unused low-level setter retains the lossy mutation path
`crates/rdocx-oxml/src/header_footer.rs:187`

`CT_HdrFtr::set_authored_watermark` is no longer used by the facade. It still
serializes the reduced header model before applying the byte patch, silently
ignores every error, and therefore cannot provide the producer-byte guarantee
of the package-level path. This creates a second mutation route with weaker
semantics and no current caller. It should not remain as hidden public surface.

## Nitpicks

None.

## Not found

Pass-1 defects D2 through D5 and D7 through D9 are otherwise remediated. The
facade now patches original UTF-8 header byte ranges without moving unrelated
content, resolves custom header image targets from the owning part, handles
named and malformed colours without panic, gates even references on the
settings value, diagnoses unresolved images, checks watermark ownership by
expanded element identity and id attribute, and uses the margin rectangle.
No additional defects were found in atomic staging, cache invalidation,
generated direct-child order, relationship-id scope, deterministic text
shaping, body-text z-order, or the repository structural rules beyond S1.
The focused suites passed, 11 tests in `rdocx` and 10 header/footer tests in
`rdocx-oxml`. The prose and diff checks also passed.

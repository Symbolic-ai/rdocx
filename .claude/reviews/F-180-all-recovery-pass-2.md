# F-180, all aspects, recovery pass 2

**Reviewed**: Entire uncommitted F-180 implementation diff, 9 files, 6,451 additions and 2,028 deletions, plus all original and recovery reviews and the complete progress record
**Verdict**: 8 defects, 0 smells, 0 nitpicks

## Defects

### D1, small automatic line heights lose one twip on reopen
`crates/rdocx/src/odt.rs:1791`

The writer rounds an automatic line height to ten decimal places as a
percentage, then the reader converts that percentage through
`Paragraph::set_line_spacing_multiple`, whose public contract truncates after
multiplication by 240. A retained value of 2 twips is emitted as
`0.8333333333%`. Reopening computes approximately `1.99999999992` and stores 1
twip. Values such as 119 and 359 have the same failure. The existing gate uses
360, which is exactly representable and masks the loss. This violates the
exact effective line-height round trip.

### D2, the inclusive paragraph length ceiling serializes outside the reader domain
`crates/rdocx/src/odt.rs:1968`

`validate_paragraph_projection` accepts the reader's inclusive 20,000,000
twip ceiling, but `twips_points` adds a signed one-billionth point before
serialization. The positive ceiling therefore becomes
`1000000.000000001pt`, and the negative first-line ceiling becomes
`-1000000.000000001pt`. The reader rejects either value because its absolute
point limit is exactly 1,000,000 at `crates/rdocx/src/odt.rs:4071`. Maximum
spacing, indentation, and exact line-height values can be written but cannot
pass the declared reopen boundary.

### D3, the exact maximum reader-supported image dimension is rejected
`crates/rdocx/src/odt.rs:847`

The reader accepts an inclusive `1000000pt`, which converts to exactly
12,700,000,000 EMU through the pinned truncating constructor. The writer caps
images at 12,699,999,999 EMU, so that exact supported value fails before
serialization. The recovery test labels the smaller value as the maximum and
therefore does not prove the promised exact upper boundary. The serializer
needs a boundary-safe representation for 12,700,000,000 EMU while continuing
to reject 12,700,000,001.

### D4, heading levels derived from style IDs still clamp silently
`crates/rdocx/src/odt.rs:1342`

Recovery validation rejects a retained `outline_lvl` above eight, but it does
not validate the fallback returned by `heading_level_from_style`. A public
paragraph styled `Heading10` is treated as outline level 9 and then clamped to
8 here, while `Heading0` saturates to level 0. The package succeeds without a
diagnostic and reopens with different heading semantics. The same range check
must cover the derived style path before emission.

### D5, synthesized empty table paragraphs are absent from the block budget
`crates/rdocx/src/odt.rs:1324`

When a non-continuation cell contains no paragraph, for example a retained
cell containing only an unsupported nested table or content control, the
writer emits a synthetic `text:p` here. The scan charged the table but no
paragraph for that cell. F-179 calls `bump_blocks` for the synthetic paragraph
on reopen. A document at the writer's 100,000-block ceiling can therefore add
one such cell, serialize successfully, and fail its own reader boundary at
100,001 projected blocks.

### D6, XML attribute normalization can change a caller font family
`crates/rdocx/src/odt.rs:472`

The writer validates only whether a font-family string contains XML 1.0
characters. XML attribute normalization changes tabs and line breaks to
spaces, and F-179 additionally trims the complete value at
`crates/rdocx/src/odt.rs:3896`. A public font such as `A\tB` reopens as `A B`,
and a family with boundary whitespace reopens trimmed, with no diagnostic.
The supported effective font family therefore does not survive exactly.

### D7, unsupported vertical alignment values disappear without a diagnostic
`crates/rdocx/src/odt.rs:1855`

The retained run model accepts any `w:vertAlign` value. The writer emits only
`superscript` and `subscript`, but `run_properties_have_unsupported` at
`crates/rdocx/src/odt.rs:1996` does not classify any other value as loss. A
valid explicit `baseline`, or a malformed retained producer value, is dropped
silently and reopens as an absent property. This contradicts the complete
path-aware unsupported-property diagnostic contract.

### D8, the F-179 HLD entry still declares conversion to be one-way
`docs/hld/14-development-backlog.md:1514`

The approved HLD impact includes this file, but the adjacent F-179 entry still
says ODT is a private one-way facade conversion. F-180 adds the inverse native
writer in the very next entry. The authoritative spec set therefore
contradicts the implemented and documented two-way conversion boundary.

## Smells

None.

## Nitpicks

None.

## Recovery findings verified

- Recovery pass-1 D1 is fixed for every scanned source run and for generated
  XML element nodes. D5 above is the separate synthetic-cell block path.
- Recovery pass-1 D2 is fixed for interrupted lists in the body and within a
  table cell. Subsequent lists of the same source numbering instance carry
  `text:continue-numbering="true"`.
- Recovery pass-1 D3 is fixed. Ordinary and field-projected non-ASCII Unicode
  whitespace fails at the precise run-content path rather than changing on
  reopen.
- Recovery pass-1 D4 is fixed. Only absent and exact `Internal` target modes
  reach package resolution. Exact `External` and every other retained value
  are diagnosed and omitted.
- Recovery pass-1 D5 is fixed for missing and wrong-type final header and
  footer relationships at stable typed-reference paths.
- Recovery pass-1 D6 is fixed. A vertical continuation must match a merge in
  the immediately preceding row, and an intervening overlap is rejected.
- Recovery pass-1 D7 is fixed. Every used level missing from a resolved
  numbering definition receives the stable decimal-fallback diagnostic.
- Recovery pass-1 D8 is fixed. No new numbering helper remains on the public
  facade, and the integration gate derives level kinds test-locally from the
  serialized numbering part.
- Recovery pass-1 D9 is fixed for direct and inherited `outline_lvl` values.
  D4 above is the separate style-ID-derived path.
- Recovery pass-1 S1 is fixed for the emitted fixture. The recursive check now
  tracks repeated list styles across siblings and live horizontal and vertical
  spans across table rows.

## Not found

- **Numbering and relationships**: no additional defect was found in direct
  `numId=0` cancellation, defined and undefined level-kind selection, list
  continuation attributes, image relationship type checks, exact target-mode
  handling, or final story-reference diagnostics.
- **Tables and XML semantics**: apart from D5, no additional defect was
  found in merge-overlap rejection, horizontal and vertical span emission,
  covered-cell placement, fixed namespace prefixes, ODF child order, inline
  anchoring, image-link attributes, or generated XML well-formedness.
- **Packaging, determinism, and ownership**: no defect was found in ZIP entry
  order, mimetype storage and local header, fixed ZIP metadata, manifest
  membership, media ordering, repeated-write bytes, atomic staging cleanup,
  destination preservation, or source-document mutation.
- **Panics and structure**: no reachable untrusted-input panic was found after
  checked span accumulation. No new crate, module, source file, dependency,
  trait, generic parameter, feature flag, wrapper-only abstraction, Python
  surface, WASM surface, or CLI surface was introduced.
- **API and HLD scope**: the public surface is limited to the approved
  `OdtWriteResult`, `Document::to_odt_bytes`, and `Document::save_odt`. The six
  modified HLD files match the approved impact list. D8 is the remaining
  current-state contradiction within that list. The untouched `rdocx-py`
  error classifier belongs to F-X054 and was not treated as an F-180 finding.
- **Focused verification**: all 23 ODT writer unit tests and the public writer
  round-trip integration test pass. `git diff --check` and the tracked review
  prose check pass before this review artifact is added.

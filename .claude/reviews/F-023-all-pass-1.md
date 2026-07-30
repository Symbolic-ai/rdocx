# F-023, all, pass 1

**Reviewed**: working-tree diff against claim commit `e0816ce`, 4 files, 236 lines added and 0 lines removed
**Verdict**: 3 defects, 1 smell, 0 nitpicks

## Defects

### D1, EMF and WMF return deprecated MIME aliases instead of canonical types
`crates/oxml-media/src/lib.rs:93`

`ImageFormat::Emf.content_type()` and `ImageFormat::Wmf.content_type()` return
`image/x-emf` and `image/x-wmf`. Those aliases are deprecated in favor of the
registered canonical types `image/emf` and `image/wmf`, so the public method
does not meet its documented canonical-mapping contract and the unit test pins
the deprecated values.

### D2, standard nonplaceable WMF files are not recognized
`crates/oxml-media/src/lib.rs:45`

The WMF branch recognizes only the `0x9AC6CDD7` placeable-header key. A standard
nonplaceable WMF starts directly with a `META_HEADER`, for example the valid
type, header-size, and version prefix `01 00 09 00 00 03`. That input returns
`None`, so `resolve` can select a misleading extension or default to PNG even
though the bytes are a supported WMF image.

### D3, valid SVG prologs containing comments or a document type are rejected
`crates/oxml-media/src/lib.rs:119`

After an XML declaration, the detector requires the next non-whitespace bytes
to be `<svg`. XML permits comments, processing instructions, and a document
type before the document element, so a valid SVG such as an XML declaration
followed by a comment and then `<svg>` is not sniffed. With no trustworthy SVG
extension, `resolve` reports the wrong format instead of honoring sniff-first
precedence.

## Smells

### S1, extension-second resolution has no direct regression assertion
`crates/oxml-media/src/lib.rs:188`

The resolve tests prove sniff-first precedence and the PNG default, while the
extension test calls `from_extension` directly. Removing the extension fallback
from `resolve` would leave all five tests green. Add a direct assertion that
unknown bytes with a known filename extension resolve from that extension.

## Nitpicks

None.

## Not found

No additional findings in panic safety, arithmetic or slicing safety, OOXML
schema concerns, structure, dependency isolation, publication isolation,
workspace manifest wiring, lockfile accuracy, or the JPEG canonical extension
and MIME mappings.

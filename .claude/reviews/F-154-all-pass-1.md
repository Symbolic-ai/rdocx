# F-154, all aspects, pass 1

**Reviewed**: working tree against `HEAD`, 7 files and 1,060 changed lines
**Verdict**: 9 defects, 0 smells, 0 nitpicks

## Defects

### D1, PAGEREF fields inside tables and content controls never resolve
`crates/rdocx-layout/src/engine.rs:845`

The target-id scan only visits direct body paragraphs. Layout also processes
paragraphs inside tables and body content controls, but a PAGEREF in either
location receives no target id and retains its cached display even when the
bookmark exists. The structured field contract applies wherever paragraph
layout accepts the field, so target collection must use the same recursive
paragraph traversal as layout.

### D2, PAGEREF pagination depends on the stale cached display width
`crates/rdocx-layout/src/engine.rs:672`

The pre-pagination line layout shapes the stored display whenever it is
nonempty. A producer can cache an arbitrarily long or short value, which can
move line and page breaks before the final page-number substitution. The
approved single-pagination design requires a fixed placeholder for a resolved
target and reserves the cached display for a missing target.

### D3, only one bookmark beginning at a paragraph end is carried to pagination
`crates/rdocx-layout/src/engine.rs:747`

The paragraph-end marker lookup uses `find`. Two valid bookmarks can begin at
the same run boundary, especially in an empty paragraph, and both can be
PAGEREF targets. Only the first receives a zero-width target marker, so every
other PAGEREF keeps its pre-pagination text instead of the target page.

### D4, a bookmark beginning before hidden text loses its page target
`crates/rdocx-layout/src/engine.rs:509`

Hidden runs are skipped before `push_targeted_bookmark_markers` executes at
line 561. A start marker whose run index points at a hidden run is therefore
never emitted, even though the bookmark is structurally present and can be a
valid PAGEREF target. Marker emission must not depend on the visibility of the
run after its boundary.

### D5, saving an empty REF or PAGEREF display changes it to `1`
`crates/rdocx-oxml/src/text.rs:838`

The shared field writer replaces every empty stored display with `1`. That is
a useful authored default for PAGE and NUMPAGES, but parsed REF and PAGEREF
fields are required to retain their stored fallback when a target is missing.
An empty fallback is data too, and save currently mutates it.

### D6, a foreign-namespace `fldSimple` is parsed and rewritten as WordprocessingML
`crates/rdocx-oxml/src/text.rs:622`

The parser matches only the local name for `fldSimple` and its instruction
attribute. A producer extension such as `x:fldSimple` is therefore interpreted
as a Word field and later serialized as `w:fldSimple`, losing unsupported XML.
Element and attribute recognition must use expanded names while retaining
alias-prefix support for the Word namespace.

### D7, TOC insertion can reuse an existing bookmark id
`crates/rdocx/src/document.rs:1997`

The typed TOC path still derives marker ids only from the `_TocN` suffix. If an
unrelated existing bookmark already owns id 100 and no TOC bookmark exists,
the next generated `_Toc1` also receives id 100. Correlation by id then becomes
ambiguous. TOC marker ids must be allocated from the occupied nonnegative id
set while preserving the historical sequence when it is free.

### D8, a valid maximum TOC suffix makes the public insertion API panic
`crates/rdocx/src/document.rs:2008`

`highest_toc_bookmark` accepts `_Toc2147483647`, after which the unchecked
increment overflows in debug builds. The initial `100 + toc_counter` at line
1997 can panic earlier for smaller valid suffixes, and the marker-id increment
at line 2029 has the same problem. Parsed producer input must not turn
`insert_toc` into a panic.

### D9, the shared layout model exposes a Word-specific bookmark concept
`crates/oxml-layout/src/output.rs:95`

`oxml-layout` is the format-neutral layout boundary, but `FieldKind` now names
`BookmarkStart` directly. This couples the shared output model to a DOCX
construct and expands the documented exception set. The target carrier should
use a neutral field-target name, with bookmark interpretation confined to
`rdocx-layout`.

## Smells

None.

## Nitpicks

None.

## Not found

No additional public range, atomic mutation, XML child-order, deterministic
font, hash-baseline, dependency-direction, or test-binary findings were found.

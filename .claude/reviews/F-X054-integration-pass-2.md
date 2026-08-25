# F-X054, integration, pass 2

**Reviewed**: staged squash integration against `dc9d53f`, 37 files and 4,702
changed lines with 4,597 additions and 105 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, EPUB reports marker replacement for a paragraph emitted without a marker
`crates/rdocx/src/epub.rs:824`

The producer-format branch correctly emits the paragraph outside a list, but
the remaining level diagnostics still run as if EPUB list semantics were used.
An `Other("chicago")` level with a nonstandard `lvlText` therefore reports that
the marker text was replaced by EPUB list semantics, and a retained suffix at
line 852 reports normalized marker spacing. Neither statement describes the
output because this branch emits no marker at all. The producer-format fixture
uses the standard generated marker and does not assert the complete diagnostic
set, so it cannot detect either inaccurate diagnostic.

### D2, ODT hides every producer-list loss except the format itself
`crates/rdocx/src/odt.rs:471`

The producer-format branch emits one format diagnostic, then skips both
`used_list_levels` and `ensure_list_style`. That prevents
`scan_numbering_container_losses` and `list_level_is_bullet` from seeing an
`Other(String)` level. An Other-only numbering definition with retained root,
instance, or abstract XML, or with a custom start, suffix, level text,
justification, paragraph formatting, marker formatting, or retained level XML,
is flattened while all of those existing loss diagnostics disappear. The
single-format fixture carries no additional numbering details, so it does not
exercise the hidden-loss path. This violates the ODT writer contract that every
lossy conversion identifies what it dropped and the F-X054 checklist requirement
not to hide lossy exports.

## Smells

None.

## Nitpicks

None.

## Not found

The pass-1 invented-marker defects are fixed. `detect_list` now declines
producer-defined levels, so EPUB emits their paragraph text without `ol`, `ul`,
or a counter advance. ODT excludes producer-defined paragraphs from both body
and table-cell list traversal and writes their text as ordinary paragraphs.
Those boundaries leave adjacent known lists available to resume through the
existing per-numbering counters and preserve supported paragraph siblings.
The inaccurate unresolved EPUB diagnostic is suppressed for a resolved
producer-defined level.

Modeled EPUB Decimal, Bullet, None, and Ordinal branches remain distinct in the
existing list projection. Modeled ODT Decimal and Bullet emission, and the
established diagnosed reduction of other modeled numbered formats, are
unchanged. The layout resolver still emits no marker for None or producer
formats, retains modeled decimal, Roman, letter, and ordinal formatting, and
does not advance the new producer-format branch. No fresh counter, sibling, or
table-cell traversal defect was found.

The complete EPUB filter passed 34 tests with the pinned EPUBCheck test ignored
by its environment guard. The complete ODT filter passed 41 tests. The focused
producer-defined run passed 5 tests across `rdocx`, `rdocx-layout`, and
`rdocx-oxml`. The layout style resolver passed 14 tests, and low-level OOXML
numbering passed 51 tests. The public `lib.rs` merge retains the EPUB, ODT, and
SVG result exports together with every F-X054 ordered-reader export. There are
no unmerged index entries. No additional defects were found in the staged
public exports, automatic merges, OOXML numbering round trip, panic safety,
schema order, or structural rules.

# F-X183, delta, pass 2

**Reviewed**: Supplemental pre-recording F-268a deterministic geometry delta on frozen `work/f-x183-codex`, Base and HEAD `d4b8f5458678d1640de9d2fb709655ab437ca98b`. Complete working diff contains thirteen tracked files, 886 added lines and 53 removed lines. The supplement introduces no production change beyond DELTA pass 1.

**Verdict**: 0 defects, 0 smells, 0 nitpicks within this supplemental DELTA scope. Recording the exact fourteen-rectangle geometry below is justified. Final ALL microscope and complete scoped verification remain required.

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Identity and provenance

Authenticated `/private/tmp/fx183-supplement-freeze.json`, SHA-256 `8c37857389ad950d5663413aaf2ac4bffdc5c9d11dcac5b92d873a89a7a4931a`, and complete diff `1155cdcf7d901a32740900bbc41f443005a3b1f9983d502b654ca48f9a035c86`. Independently rehashed all 35 direct bindings, matched exact status and diff, checked the detached Base clean, and repeated the frozen binding check before writing.

The original fixture builder at `crates/rdocx/tests/integration_test.rs:19938` remains byte-identical to Base, SHA-256 `64c4b73cad960a1dd2416ad10dd65b705c968c4c0fd1af56029fb9d0ff58683b`. The existing `GOLDEN_TABLE_GEOMETRY` values remain identical to Base. Read the original F-268a plan and both microscope records. The historical test name does not supply native Word provenance. Its plan explicitly describes an in-code deterministic structural golden and requires no Word GUI automation. This supplement qualifies deterministic behavior against the separately approved S90 padding contract, not fresh Word parity for this fixture.

Both genuine executions compile their respective source roots. Base passes the original assertion. Current fails that same untouched pin, with the complete observed fourteen rectangles printed. Independently checked all 217 source freshness entries in each direction against present files, accounting only for the bound temporary integration instrumentation. The identical export block has the same digest in both add/remove pairs and is removed by exact inverse. Production hashes remain identical to DELTA pass 1. No Cargo command was run by this reviewer.

## Source and package boundary

Before and after DOCX archives are byte-identical, SHA-256 `7e8bd4283275f3d6939a548d6fdcbcc59b2c5fc7b8d0878fad7c3ba23f401d4e`. Independently compared all ten ZIP member names, order and bytes. Parsed source properties confirm no selected table style, direct table or cell margins, table indent or compatibility setting. Settings are empty. No effects part or effects relationship exists. Styles contain no default table style. The nondefault `TableGrid` margins are not selected by these tables.

The fixed grid is 144/144/180 pt. The outer host columns are 234/234 pt. The nested source grid is 112.5/112.5 pt, totaling 225 pt. Source metadata and geometry therefore exclude effects fallback, built-in default exceptions, direct-cell margin overlays and authored-indent compensation from this attribution.

## Independently recomputed geometry

Read both raw geometry artifacts and checked all fourteen rectangles. All six fixed cell rectangles remain exact. Every y coordinate and height remains exact across all fourteen.

The explicit auto-width/autofit columns change from 20.34 and 242.28 pt to 9.54 and 231.48 pt. Each loses precisely 10.8 pt of implicit side padding. Their second-column origin changes from 92.34 to 81.54 pt in both rows. Autofit remains explicitly engaged and stops at its measured content maximum. This follows the zero defaults at `crates/rdocx-layout/src/table.rs:570` and line 574 and the existing measured-width consumer. It does not change the engagement predicate.

The nested host previously exposed 234 minus 10.8, or 223.2 pt. The unchanged 225 pt nested grid was consequently clamped to 223.2 pt, producing 111.6 pt columns. With zero padding, 234 pt is available and the declared 225 pt grid fits, yielding 112.5 pt columns. The nested left origin changes from 311.4 to 306 pt and the second origin from 423 to 418.5 pt. This is the existing parent content-box and grid-clamping calculation at `crates/rdocx-layout/src/table.rs:1637`, not new nested legacy-position compensation. The fixed grid and all row heights remain stable.

## Raster, content and PDF checks

Independently decoded both original 816 by 1056 RGBA rasters. Exactly 7,022 pixels change within the half-open rectangle `[96, 99, 713, 306]`. Decoded SHA-256 values are `ca09e37db6cb10dcbdb706b06b648534236467f0101a1f329310ef763012ad9a` before and `26597c32b80c6da696f696aa7b391a7d4f137fa8600798c66e39394332fb448d` after. No tolerance, conversion or resizing was applied.

Viewed both full-page rasters. Fixed shading rectangles remain stable while their text moves left. Autofit shading narrows with its intrinsic columns, and the nested shading moves and changes width as calculated. No glyph disappears. The zero-padding `ID` cell visually abuts the next cell's `A`. Raw text extraction consequently groups `IDA`, but independent PDF text-show decoding confirms the same ordered 96 font-associated glyphs in both files. This is extraction grouping at adjacent cell boundaries, not loss or an inserted character. The PDF resource fingerprint remains exact. Both PDFs retain one 612 by 792 pt page.

The existing sample after artifacts still equal the recorded complete harness manifest. The previously reviewed fourteen sample keys remain the only sample delta. This supplement adds only the declared deterministic geometry pin and does not expand that sample set.

## Not found and remaining gates

Source provenance, OOXML preservation, exact geometry attribution, glyph preservation, baseline exclusivity and regression sensitivity produced zero findings. Correctness, contract, panics, tests and structure checks within this supplemental boundary also produced zero findings. No new production policy, dependency, trait, generic or module is introduced by the supplement.

The old fixture assertion intentionally remains failing until approved recording. Preserve the six fixed rectangles and every y coordinate and height when recording the exact autofit and nested values. Final ALL microscope and scoped verification must follow. This pass does not establish absolute native Word rendering parity or feature completion.

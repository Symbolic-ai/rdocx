# F-X021, The hash harness should cover PDF output

**Status**: approved
**Sprint**: S43
**Size**: L
**Depends on**: none

Sized M at design time and revised to L during implementation, when the story
found that the PDF writer was not deterministic and the sprint decided to fix
that here rather than defer it. See "The writer was not deterministic" below.

## Problem

`scripts/hash_harness.py:26-40` records four entries per sample: three
`word/*.xml` parts and `page1.png`. Twenty-eight in total, and no PDF.

PDF is a first-class output of this workspace and comes from a different code
path from the PNG. `crates/oxml-pdf/src/writer.rs` writes glyph positions,
embedded CID font subsets, ToUnicode CMaps and Deflate-compressed content
streams. Rasterising page one exercises none of that as bytes, and it exercises
nothing at all on pages two onward.

F-X020 demonstrated the gap. Its commit records that a semver-compatible
refresh of `font-types` 0.12.2 to 0.12.3 reached the shaper and moved all seven
sample PDFs, while every PNG stayed byte-identical and the harness reported 28
of 28. The change was characterised by hand with the pinned Poppler oracle:
`pdftotext` output identical in 7 of 7, `pdfinfo` identical apart from file
size, sizes moving by single-digit bytes. Benign, and found by a person rather
than by the gate that exists to find it.

That characterisation also sets the bar for this story. A fingerprint built
from extracted text and page geometry alone would have reported green on the
one event that motivated the story, because the extracted text did not change.
The fingerprint has to reach the glyph and font-subset bytes.

## The writer was not deterministic

Recording the first fingerprint proved that `to_pdf_deterministic` was not.
Two runs of `generate_all_samples`, same machine, same binary, same input,
produced different bytes for all seven samples. Three hashed maps were iterated
to write the file, so their iteration order reached it:

- `crates/oxml-pdf/src/font.rs:70` iterated `glyph_to_unicode`, a
  `HashMap<u16, char>`, to emit the ToUnicode CMap pairs, so the CMap's lines
  came out in a different order every run. The same multiset of lines, a
  different file.
- `crates/oxml-pdf/src/writer.rs:505` and `:684` iterated `prepared_fonts` and
  `font_refs`, both `HashMap<FontId, _>`, to write the font objects and each
  page's `/Font` dictionary.
- `crates/oxml-pdf/src/writer.rs:691` iterated `image_map`, a
  `HashMap<(usize, usize), usize>`, to name a page's image XObjects.

The `pdf/pages` entry was stable across runs in all seven samples, because the
content streams and page geometry do not depend on any of those orders. The
`pdf/resources` and `pdf/bytes` entries were not stable, so neither could be
recorded until this was fixed.

This also means F-X020's by-hand characterisation was comparing against a
moving target. Its conclusion, that the dependency refresh was benign, is not
undermined, since `pdftotext` and `pdfinfo` agreed. What it could not have
known is that some of the byte movement it attributed to `font-types` was the
writer disagreeing with itself.

## Spec reference

- `docs/hld/12-testing-strategy.md`, "The hash harness", which states the 28
  entries, what each is, and the reason the harness exists. This story changes
  that count and that list.
- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate", for the neighbouring
  instrument and why this one must not depend on it: that gate needs `pdftoppm`
  and a pinned Poppler build, and the hash harness must run anywhere with the
  Python standard library alone.
- `docs/hld/08-rendering-spec.md`, "The PDF backend", which owns what the writer
  emits and already says the operator stream is compared as pixels rather than
  bytes. It gains the rule that the writer's output is reproducible.
- `docs/hld/14-development-backlog.md`, "F-X021, The hash harness should cover
  PDF output".

## Approach

`crates/rdocx/examples/generate_all_samples.rs:74-80` already writes
`samples/<name>.pdf` through `to_pdf_deterministic`. Nothing new is generated.
The harness gains a fingerprint of what is already on disk.

**The output carries no timestamp.** Inspecting `samples/invoice.pdf` confirms
the writer emits no `/CreationDate`, no `/ID`, a classic cross reference table
and no object streams. The backlog's stated worry about a creation date does not
apply to this writer, which is what makes a byte digest usable at all.

**The ordering had to be fixed first**, which is the section above. Three hashed
maps become ordered ones:

- `FontUsage::glyph_to_unicode` becomes `BTreeMap<u16, char>`, so the ToUnicode
  CMap pairs are emitted in glyph order.
- `prepared_fonts` and `font_refs` become `BTreeMap<FontId, _>`, so font objects
  and each page's `/Font` dictionary are written in font order. `FontId` gains
  `PartialOrd` and `Ord`, which is additive.
- The page's image XObject names are sorted by element index before they are
  written, rather than taken in `image_map` iteration order.

Ordered containers rather than a sort at each point of use, because the property
wanted is "this map is iterated to produce output", and a type states that once
instead of every reader having to notice it three times.

Three entries per sample, 21 new, taking the manifest from 28 to 49:

| Entry | Covers |
|---|---|
| `<sample>:pdf/pages` | Page count, each page's `/MediaBox`, and each page's **inflated** content stream, in `/Kids` order |
| `<sample>:pdf/resources` | Every other inflated stream, which is the CID font subsets, the ToUnicode CMaps and the image XObjects, in a deterministic order |
| `<sample>:pdf/bytes` | SHA-256 of the file as written |

The structural pair says **what** moved and survives a change of Deflate
implementation or level, because it hashes inflated bytes. The byte entry says
**that** something moved and cannot be evaded, including by a compression-only
change that the structural pair is deliberately blind to. F-X020's own event
moves the structural pair, since single-digit byte differences in glyph and
subset data are exactly what it hashes.

The parser is a scanner over the object syntax, not a general PDF reader,
written against the standard library:

```python
def pdf_fingerprint(data: bytes) -> dict[str, str]:
    """Page geometry, inflated content streams and inflated resource streams."""
```

It scans `N 0 obj ... endobj`, resolves `/Root` to `/Pages` to `/Kids` for page
order rather than trusting object numbering, inflates a stream when its
`/Filter` is `/FlateDecode` and hashes it raw otherwise, and raises on anything
it does not understand. Refusing to guess is the point: a harness that silently
skips an object it cannot parse reports green for the wrong reason.

`run_sample_generator` deletes `pdf` alongside `docx` and `png`, so a stale PDF
cannot be fingerprinted. A missing PDF raises rather than recording `None`. The
`None` convention at `hash_harness.py:129` means "this optional XML part is
absent by design", and a sample whose PDF failed to generate is not that.

`EXPECTED_ENTRY_COUNT` becomes `len(SAMPLES) * (len(OOXML_PARTS) + 1 + 3)`.

## Rejected alternatives

- **The byte digest alone.** The smallest possible diff and it catches strictly
  more than the structural pair, but a delta then says only "the PDF moved". The
  harness is read by whoever has to decide whether a delta is benign, and doing
  what F-X020 did by hand is the cost of dropping attribution.
- **The structural pair alone.** Quieter, and blind by construction to a change
  that is purely in compression. This sprint exists because something changed
  and nothing said so.
- **Extracted text plus page geometry.** The backlog offers this shape. It would
  have reported green on F-X020, whose extracted text was identical in 7 of 7.
- **Normalise the ordering in the harness instead of fixing the writer.** Sort
  the CMap lines and the `/Font` entries before hashing. It would have made the
  structural pair stable without touching a Rust crate, and it would have made
  the new gate blind to the defect it had just found. The byte digest cannot be
  salvaged that way at all.
- **Split the writer fix into its own F-ID and record only `pdf/pages` now.**
  Two clean stories, and a sprint that closes with a known reproducibility
  defect open and the gate half-built. The fix is three container types.
- **Rasterise the PDF and hash pixels.** That is
  `scripts/golden_png_harness.py`, which already exists. It needs `pdftoppm` and
  a pinned Poppler build, so it cannot be the gate that runs everywhere.
- **A separate PDF manifest file.** A second baseline to keep in step with the
  first, for output from the same generator run. There is one harness.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `two_identical_documents_produce_identical_deterministic_pdfs` | Two separately built documents write byte-identical PDFs. Two writes of one document cannot see this defect, because they reuse the same map instances |
| regression | `test_a_changed_content_stream_moves_the_pdf_entries_and_no_other` | Over a PDF constructed in the test, editing a content stream moves `pdf/pages` and `pdf/bytes` and leaves `pdf/resources` untouched |
| regression | `test_a_changed_resource_stream_moves_only_the_resource_entries` | The mirror case. Editing an embedded resource moves `pdf/resources` and `pdf/bytes` and leaves `pdf/pages` untouched, so the two entries attribute rather than merely detect |
| regression | `test_refingerprinting_identical_bytes_reproduces_every_entry` | The fingerprint of the same bytes twice is equal entry for entry, so a green run is a real one |
| unit | `test_recompressing_the_same_content_leaves_the_structural_entries_still` | The same content deflated at levels 1 and 9 gives identical `pdf/pages` and `pdf/resources` and a different `pdf/bytes`. The division of labour, stated as a test |
| unit | `test_a_page_geometry_change_moves_the_pages_entry` | A different `/MediaBox` moves `pdf/pages`, so the entry covers geometry and not only content |
| unit | `test_a_payload_that_looks_like_pdf_syntax_does_not_confuse_the_scanner` | A resource payload full of `endobj` and `/Root 9 0 R` fingerprints the same pages as a benign one, because the payload is taken by its declared `/Length` and `/Root` is read from the trailer |
| unit | `test_a_filter_chain_is_refused_rather_than_inflated` | `/Filter [/FlateDecode /ASCIIHexDecode]` raises about the filter rather than about a corrupt stream |
| unit | `test_an_unparseable_pdf_is_an_error_rather_than_an_absent_entry` | A file with no objects, one with no `/Root`, and a stream under a filter the scanner does not read all raise |
| unit | `test_a_missing_pdf_is_an_error_rather_than_an_absent_entry` | A sample directory with every docx and png but no pdf raises, rather than recording `None`. The generator prints a message and carries on when PDF rendering fails, so this is reachable |

**Test gate**, from the backlog: the deliberate-change regression, which is the
second row, plus the reproducibility half of the same gate, which is the fourth.
The first row is the gate for the writer fix this story absorbed, and it was
confirmed to fail against the unfixed writer.

The backlog's gate is written as "a deliberate change to the PDF writer moves
the new entries". The harness tests assert that over PDFs constructed in the
test, which is what the no-binary-fixtures rule requires. The end-to-end half is
a recorded manual demonstration in the implementation checklist: perturb one
constant in `oxml-pdf`, run `--check`, confirm the PDF entries move and the PNG
entries do not, revert. Its output goes in the AS_BUILT entry.

## HLD impact

- `docs/hld/12-testing-strategy.md`, "The hash harness". The entry count moves
  from 28 to 49, the list gains the three PDF entries per sample, and the
  section states what each covers and why the byte digest and the structural
  pair are both present.
- `docs/hld/08-rendering-spec.md`, "The PDF backend". It gains the rule that the
  writer's output is reproducible for a given layout, and the reason it is
  stated rather than assumed: three hashed maps were iterated to write the file
  and it was not.

## Risk routing

Matched rows: **Public API of a published crate**. Recorded during
implementation, when the story absorbed the writer fix. At design time the diff
touched no Rust crate and this section correctly read `none`.

- Semver impact, **additive**. `oxml_layout::FontId` gains `PartialOrd` and
  `Ord`. No signature changes, no surface is removed, and no existing caller
  stops compiling. `oxml-pdf`'s changed containers are behind `pub(crate)` and
  private locals, so they are not surface at all.
- `cargo publish --workspace --dry-run` and the `.crate` size assertion run as
  `/verify` step 10 over the integrated sprint.
- No surface no story asked for. `Ord` on `FontId` exists because the writer
  keys a `BTreeMap` on it, which exists today in this diff.

The baseline is the sprint's exclusive resource, and this is the only S43 story
permitted to move it.

## Hash harness

**Expected delta, and it is the story.** Twenty-one added entries, three per
sample, taking the manifest from 28 to 49. No existing entry changes: the
`word/*.xml` and `page1.png` digests are untouched, and the delta must report
21 `added:` lines and nothing else. Anything else in the report is a defect in
this story and not a re-record prompt.

**Observed, after the writer fix**: 21 added, 0 changed, 0 removed. The
determinism fix changes the PDF bytes and moves no PNG and no XML entry, which
is what the separation of the raster path from the writer predicts.

The re-record is `python3 scripts/hash_harness.py --update --reason "F-X021,
add the PDF fingerprint entries"`, in its own labelled commit, with the 21
added keys listed in the message.

## Implementation checklist

- [x] Record the pre-change harness state, 28 of 28
- [x] `pdf_fingerprint`, the object scanner, `/Kids` page order and the inflate
      path
- [x] `collect_hashes` emitting the three entries per sample
- [x] `run_sample_generator` deleting stale PDFs, missing PDF raising
- [x] `EXPECTED_ENTRY_COUNT` and the harness tests
- [x] Payload read by `/Length`, `/Root` read from the trailer and the filter
      read as a parsed value, per microscope pass 1
- [x] The three ordered containers in `oxml-pdf`, and the sorted image names
- [x] The writer regression, confirmed to fail against the unfixed writer
- [x] Three consecutive generator runs producing byte-identical PDFs, 7 of 7
- [x] `--update --reason`, in its own commit, delta stated as 21 added
- [x] The manual writer-perturbation demonstration, output kept for AS_BUILT
- [x] Update `12-testing-strategy.md` and `08-rendering-spec.md`
- [x] `python3 -m unittest scripts.hash_harness`, `/microscope F-X021
      --working`, `/verify`

## Open questions

None. Two went to the user and both are settled.

The fingerprint shape went to the S43 consolidated round and was settled as the
structural pair plus the byte digest, 21 new entries, which is what the approach
records. The byte digest alone was rejected for losing attribution, and the
structural pair alone for being blind by construction to a compression-only
change.

The writer non-determinism was found during implementation and went back as a
second round, because it invalidated the first answer. It was settled as fixing
the writer inside this story rather than splitting it out or normalising around
it, which is what took the story from M to L.

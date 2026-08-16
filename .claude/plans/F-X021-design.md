# F-X021, The hash harness should cover PDF output

**Status**: approved
**Sprint**: S43
**Size**: M
**Depends on**: none

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

## Spec reference

- `docs/hld/12-testing-strategy.md`, "The hash harness", which states the 28
  entries, what each is, and the reason the harness exists. This story changes
  that count and that list.
- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate", for the neighbouring
  instrument and why this one must not depend on it: that gate needs `pdftoppm`
  and a pinned Poppler build, and the hash harness must run anywhere with the
  Python standard library alone.
- `docs/hld/14-development-backlog.md`, "F-X021, The hash harness should cover
  PDF output".

## Approach

`crates/rdocx/examples/generate_all_samples.rs:74-80` already writes
`samples/<name>.pdf` through `to_pdf_deterministic`. Nothing new is generated.
The harness gains a fingerprint of what is already on disk.

**The output is genuinely deterministic.** Inspecting `samples/invoice.pdf`
confirms the writer emits no `/CreationDate`, no `/ID`, a classic cross
reference table, no object streams, and stable object numbering. The backlog's
stated worry about a creation date and unstable object ordering does not apply
to this writer, which is what makes a byte digest usable rather than brittle.

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
- **Rasterise the PDF and hash pixels.** That is
  `scripts/golden_png_harness.py`, which already exists. It needs `pdftoppm` and
  a pinned Poppler build, so it cannot be the gate that runs everywhere.
- **A separate PDF manifest file.** A second baseline to keep in step with the
  first, for output from the same generator run. There is one harness.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `a_changed_content_stream_moves_the_pdf_entries_and_no_other` | Over a PDF constructed in the test, editing a content stream moves `pdf/pages` and `pdf/bytes` and leaves the XML and PNG entries untouched |
| regression | `refingerprinting_identical_bytes_reproduces_every_entry` | The fingerprint of the same bytes twice is equal entry for entry, so a green run is a real one |
| unit | `recompressing_the_same_content_leaves_the_structural_entries_still` | The same content deflated at a different level gives an identical `pdf/pages`, and a different `pdf/bytes`. This is the division of labour between the two, stated as a test |
| unit | `a_missing_or_unparseable_pdf_is_an_error_not_an_absent_entry` | A missing file and an object the scanner does not understand both raise, rather than recording `None` or skipping |

**Test gate**, from the backlog: the first regression, plus the reproducibility
half of the same gate, which is the second row.

The backlog's gate is written as "a deliberate change to the PDF writer moves
the new entries". The tests above assert that over PDFs constructed in the test,
which is what the no-binary-fixtures rule requires. The end-to-end half is a
recorded manual demonstration in the implementation checklist: perturb one
constant in `oxml-pdf`, run `--check`, confirm the PDF entries move and the PNG
entries do not, revert. Its output goes in the AS_BUILT entry.

## HLD impact

- `docs/hld/12-testing-strategy.md`, "The hash harness". The entry count moves
  from 28 to 49, the list gains the three PDF entries per sample, and the
  section states what each covers and why the byte digest and the structural
  pair are both present.

## Risk routing

Matched rows: none.

The diff touches `scripts/hash_harness.py`, `scripts/hash_baseline.json` and one
HLD section. No Rust crate, no unit conversion, no parser in the shipped
sense, no public API, no feature flag and no new module. The one thing it does
touch is the baseline, which the sprint treats as an exclusive resource: this is
the only S43 story permitted to move it.

## Hash harness

**Expected delta, and it is the story.** Twenty-one added entries, three per
sample, taking the manifest from 28 to 49. No existing entry changes: the
`word/*.xml` and `page1.png` digests are untouched, and the delta must report
21 `added:` lines and nothing else. Anything else in the report is a defect in
this story and not a re-record prompt.

The re-record is `python3 scripts/hash_harness.py --update --reason "F-X021,
add the PDF fingerprint entries"`, in its own labelled commit, with the 21
added keys listed in the message.

## Implementation checklist

- [ ] Record the pre-change harness state, 28 of 28
- [ ] `pdf_fingerprint`, the object scanner, `/Kids` page order and the inflate
      path
- [ ] `collect_hashes` emitting the three entries per sample
- [ ] `run_sample_generator` deleting stale PDFs, missing PDF raising
- [ ] `EXPECTED_ENTRY_COUNT` and the four tests
- [ ] `--update --reason`, in its own commit, delta stated as 21 added
- [ ] The manual writer-perturbation demonstration, output kept for AS_BUILT
- [ ] Update `12-testing-strategy.md`
- [ ] `python3 -m unittest scripts.hash_harness`, `/microscope F-X021
      --working`, `/verify`

## Open questions

None. The fingerprint shape went to the S43 consolidated round and was settled
as the structural pair plus the byte digest, 21 new entries, which is what the
approach records. The byte digest alone was rejected for losing attribution,
and the structural pair alone for being blind by construction to a
compression-only change.

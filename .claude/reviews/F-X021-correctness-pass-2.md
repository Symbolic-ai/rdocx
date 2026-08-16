# F-X021, correctness, pass 2

**Reviewed**: the remediated F-X021 working diff on `work/f-x021-claude`, 8
files, 592 insertions and 64 deletions. Pass 1 raised 2 defects, 1 smell and 2
nitpicks. This pass re-reads the whole diff, not only the repairs.
**Verdict**: 0 defects, 0 smells, 1 nitpick

## Defects

None.

D1 is fixed. `scripts/hash_harness.py:150-170` locates the stream keyword with
`STREAM_RE`, which requires a newline on both sides, takes the payload by its
declared `/Length`, asserts `endstream` follows exactly there, and only then
looks for `endobj` beyond the payload. A stream whose `/Length` disagrees with
its `endstream` now raises about the disagreement instead of hashing a fragment.
`test_a_payload_that_looks_like_pdf_syntax_does_not_confuse_the_scanner` pins it
with a payload made entirely of `endobj`, `trailer`, `/Root 9 0 R` and `stream`,
which fingerprints the same pages as a benign document.

D2 is fixed. `scripts/hash_harness.py:296-303` finds the last `trailer` keyword
and searches for `/Root` from there, raising when there is no trailer at all.
The same test covers it, since its hostile payload also carries a `trailer` and
a `/Root` reference.

S1 is fixed. `scripts/hash_harness.py:186-197` reads `/Filter` as a parsed value
through `FILTER_RE` and compares it to `/FlateDecode` exactly, so a chain is
refused as a filter it does not read rather than inflated and reported as
corrupt. `test_a_filter_chain_is_refused_rather_than_inflated` pins the message.

The repairs did not move the fingerprint. `report:pdf/pages` and
`contract:pdf/pages` are identical to the values the pre-repair scanner produced
over the same files, which is what a correctness fix to a parser should look
like when the inputs were never pathological.

## Smells

None.

## Nitpicks

- `crates/oxml-pdf/src/writer.rs:695`, carried from pass 1. The image sort key
  is `(element index, image index)` where the element index alone is unique per
  page. Sorting on the pair is not wrong and the second component never
  decides anything.

The pass 1 nitpick about the content-stream test's name was resolved by the plan
rather than the test: the test plan row now claims what the test asserts, which
is that no other **PDF** entry moves. The XML and PNG entries staying still is
evidence from the real harness run, 21 added and 0 changed, and is recorded as
such in `## Hash harness`.

## Not found

- **correctness**. Re-read the scanner end to end against `samples/report.pdf`,
  57 objects, 5 pages, 15 resource streams. Page order comes from `/Kids`,
  geometry inherits through `/Parent`, content streams concatenate in
  `/Contents` order, and the resource digest is a sorted multiset so a
  renumbering alone does not move it while any byte change does.
- **contract**. Three entries per sample, the writer fix that made two of them
  recordable, both HLD sections, and the declared 21 added, 0 changed, 0 removed.
- **panics**. Every parse failure raises `PdfError`, which subclasses
  `ValueError` and is already caught by `main`, so the process exits 2 with a
  message rather than a traceback.
- **ooxml**, **structure**. Unchanged from pass 1 and re-checked. No new trait,
  generic, crate, module or file. `FontId` gains two derives and three
  containers become ordered.
- **tests**. 11 harness tests and the writer regression pass. The writer
  regression was confirmed to fail against the unfixed writer by reverting all
  three source files and re-running it. Three consecutive generator runs produce
  byte-identical PDFs across all seven samples.

## Exit condition

Zero defects, zero smells. The remaining nitpick is taste and is recorded rather
than fixed, with the reason.

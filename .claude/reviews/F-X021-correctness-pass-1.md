# F-X021, correctness, pass 1

**Reviewed**: the F-X021 working diff on `work/f-x021-claude`, 8 files, 540
insertions and 64 deletions. `scripts/hash_harness.py`,
`crates/oxml-pdf/src/{writer,font}.rs`, `crates/oxml-layout/src/output.rs`,
`crates/rdocx/tests/regression_test.rs`, both HLD sections and the plan.
**Verdict**: 2 defects, 1 smell, 2 nitpicks

## Defects

### D1, the object scanner can be cut short by its own payload
`scripts/hash_harness.py:141`

```python
end = data.find(b"endobj", match.end())
```

The search runs over the object's **stream payload**, which is Deflate-compressed
font subset, CMap and image data, so arbitrary bytes. A payload containing the
six bytes `endobj` truncates the object early. The dictionary is then read from
a fragment, the payload is read to a `rfind(b"endstream")` that now sits outside
the slice, and either the object silently hashes the wrong bytes or the scanner
raises about a file that is perfectly valid.

The odds per file are small. That is not the standard for a gate whose whole
purpose is to be believed when it reports green, and the fix costs nothing: the
dictionary carries `/Length`, which this writer emits as a direct integer, so
the payload can be taken by length and `endstream` asserted to follow. A
scanner that checks its own arithmetic is also one that raises honestly when the
assumption stops holding.

The same applies to `body.find(b"stream")` at `hash_harness.py:147`. It matches
the first occurrence of those six bytes anywhere in the dictionary text, rather
than the stream keyword that begins a payload.

### D2, `/Root` is searched for in the whole file rather than the trailer
`scripts/hash_harness.py:293`

```python
root = ROOT_RE.search(data)
```

`/Root` is a trailer key. Searching the entire file means any object dictionary
that happens to contain `/Root n 0 R`, and any stream payload that happens to
contain those bytes, is taken as the document catalog. No object this writer
emits does, so the entry is correct today, and it is correct by luck rather than
by construction. The trailer is findable: it is the last `trailer` keyword in
the file.

## Smells

### S1, the fingerprint trusts one filter and calls everything else unknown
`scripts/hash_harness.py:174-177`

```python
if b"/Filter" not in body:
    return payload
if b"/FlateDecode" not in body:
    raise PdfError(...)
```

Substring tests against the dictionary text, not parsed values. `/Filter
[/FlateDecode /ASCIIHexDecode]`, a chain, satisfies the second test and would be
inflated as though it were Flate alone, which fails inside `zlib` and is
reported as "did not inflate" rather than "this is a filter chain I do not
read". The writer emits no chains today. The message a future reader gets is
what makes this a smell rather than a nitpick: it points at the data when the
scanner is what is wrong.

## Nitpicks

- `scripts/hash_harness.py:406`, `test_a_changed_content_stream_moves_the_pdf_
  entries_and_no_other` asserts that no other **PDF** entry moves. The XML and
  PNG entries not moving is shown by the real harness run, 21 added and 0
  changed, and not by this test. The name reads wider than the assertions.
- `crates/oxml-pdf/src/writer.rs:695`, the comment explains why the image names
  are sorted, and the sort key is `(element index, image index)` when the
  element index alone is unique per page. Harmless, and marginally more than
  the property being asserted.

## Not found

- **correctness**, in the writer fix. `FontId` ordering is total over a `u32`
  newtype. `BTreeMap` preserves the existing `insert`, `get` and `is_empty`
  semantics at every call site. `glyph_to_unicode` is only inserted into and
  iterated. Three consecutive generator runs produce byte-identical PDFs across
  all seven samples, and the regression fails against the unfixed writer, which
  was confirmed by reverting the three files and re-running it.
- **contract**. The story does what the revised plan describes: three entries per
  sample, the writer fix that made two of them recordable, and the two HLD
  sections. The harness delta is exactly the declared 21 added, 0 changed, 0
  removed.
- **panics**. `decoded_stream` and the tree walk raise `PdfError` rather than
  indexing blindly, except through the defects above. `objects[current]` in
  `media_box_of` and `objects[int(...)]` in `pdf_fingerprint` can raise
  `KeyError` on a dangling reference rather than `PdfError`, which is still a
  loud failure and not a silent one.
- **ooxml**. No parser or serialiser in the OOXML sense is touched.
- **structure**. No new trait, generic, crate, module or file. `PdfError`
  subclasses `ValueError`, which `main` already catches, so the existing exit
  code 2 path covers it. Three container types changed and one derive added.
- **tests**, in part. Nine harness tests and the writer regression pass. The
  gate rows were checked against reverted code. D1, D2 and S1 are about inputs
  no test constructs, which is the point of raising them here.

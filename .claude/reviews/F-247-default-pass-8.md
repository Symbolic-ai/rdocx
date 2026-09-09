# F-247, default, pass 8

**Reviewed**: uncommitted working tree diff, 24 files, 8,742 changed lines
**Verdict**: 1 defect, 1 smell, 0 nitpicks

## Defects

### D1, unrendered standard numbering now makes EPUB export fail

`crates/rdocx-html/src/emitter.rs:170`

The HTML classifier now correctly refuses to turn newly typed unrendered
formats into ordered HTML lists. EPUB still classifies every typed format other
than bullet and `none` as an ordered list at
`crates/rdocx/src/epub.rs:2624`. For a normal paragraph using `chicago`, EPUB
therefore enters its list emitter, asks this HTML emitter for the item fragment,
receives `<p>item</p>`, and then requires an `<li>` at
`crates/rdocx/src/epub.rs:2253`. `Document::to_epub_bytes()` returns
`list paragraph projection produced no list item` instead of exporting the text
without an invented marker. The HTML change and EPUB list classification must
agree on the unrendered standard set.

## Smells

### S1, the regression does not prove that unrendered formats have no marker

`crates/rdocx/tests/regression_test.rs:6017`

The new producer-defined and Chicago cases assert only that HTML does not use
`<ol>` and Markdown does not contain `1. item`. A regression that converts
either case to a bullet list would emit `<ul>` and `- item`, and both assertions
would still pass even though the rendering contract forbids any invented
marker. Assert the expected paragraph or plain-text output, or explicitly
exclude both ordered and unordered list forms. The test should also exercise
Chicago through EPUB so D1 cannot recur at the downstream HTML consumer.

## Nitpicks

None.

## Pass-7 state rechecked

- The regression now covers both `producerFormat` and `chicago` at
  `crates/rdocx/tests/regression_test.rs:5988` and verifies exact package
  round-trip for each value.
- Its RTF expectation distinguishes the producer-defined diagnostic from the
  typed Chicago path at `crates/rdocx/tests/regression_test.rs:6019`.
- HTML and Markdown retain the pre-F-247 decimal, Roman, letter, ordinal,
  bullet, absent-format, and `none` branches at
  `crates/rdocx-html/src/emitter.rs:158` and
  `crates/rdocx-html/src/markdown.rs:89`.
- Keeping `none` in the HTML list branch preserves EPUB's no-marker projection.
  `epub_preserves_list_identity_and_no_number_levels` passed and still checks
  `<ul class="no-marker">` at `crates/rdocx/src/epub.rs:4487`.

## Checks run

- `git diff --check`, passed.
- `cargo fmt --all --check`, passed.
- `cargo test -p rdocx-html`, 20 tests passed including doctests.
- `cargo test -p rdocx --test regression_test unrendered_number_formats_survive_without_decimal_coercion`,
  1 passed with an isolated target directory.
- `cargo test -p rdocx --lib epub_preserves_list_identity_and_no_number_levels`,
  1 passed with an isolated target directory.
- `cargo test -p rdocx --lib epub_does_not_invent_markers_for_producer_defined_numbering`,
  1 passed with an isolated target directory.
- `cargo test -p rdocx --lib all_public_numbering_level_properties_survive_reopen`,
  1 passed with an isolated target directory.
- `cargo test -p rdocx --lib rtf_writer_preserves_all_typed_numbering_formats_without_inventing_markers`,
  1 passed with an isolated target directory.

## Not found

No additional correctness, contract, panic, OOXML child-order, namespace,
preservation, public API, equality, RTF, ODT, atomicity, structure, or nitpick
findings were found.

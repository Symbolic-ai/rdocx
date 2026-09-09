# F-247, default, pass 9

**Reviewed**: uncommitted working tree diff, 24 files, 8,828 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass-8 findings rechecked

- EPUB list detection now routes through `epub_list_semantics` at
  `crates/rdocx/src/epub.rs:2633`. Its supported set matches the HTML and
  Markdown classifiers at `crates/rdocx-html/src/emitter.rs:158` and
  `crates/rdocx-html/src/markdown.rs:89`.
- A typed unrendered standard now returns no EPUB list semantics at
  `crates/rdocx/src/epub.rs:2656`, so Chicago is emitted as a paragraph rather
  than entering a list path that requires an HTML `<li>`.
- The EPUB diagnostic scanner distinguishes producer-defined formats from
  typed unrendered standards at `crates/rdocx/src/epub.rs:813`. It reports the
  standard-format, start, marker, and suffix losses without also calling the
  definition unresolved.
- The Chicago EPUB regression at `crates/rdocx/src/epub.rs:4574` proves that
  export succeeds, preserves the text as a paragraph, emits no ordered or
  unordered list, and records the standard-format diagnostic.
- The package regression covers both `producerFormat` and `chicago` at
  `crates/rdocx/tests/regression_test.rs:5988`. Its HTML assertions exclude
  both `<ol>` and `<ul>`, and its Markdown assertions exclude both ordered and
  bullet markers at `crates/rdocx/tests/regression_test.rs:6017`.

## Preserved behavior rechecked

- `none` remains in the HTML and Markdown legacy list branches and in EPUB's
  dedicated no-marker branch. The EPUB regression still verifies
  `<ul class="no-marker">` and the retained list item at
  `crates/rdocx/src/epub.rs:4502`.
- Producer-defined numbering still becomes a plain EPUB paragraph with its
  producer-specific diagnostic and without list semantics at
  `crates/rdocx/src/epub.rs:4507`.
- Decimal, Roman, letter, ordinal, bullet, and absent-format behavior remains
  supported in the HTML, Markdown, and EPUB classifiers.
- Exact package round-trip and the different RTF diagnostic expectations for
  producer-defined and Chicago values remain asserted at
  `crates/rdocx/tests/regression_test.rs:6023` and
  `crates/rdocx/tests/regression_test.rs:6031`.

## Checks run

- `git diff --check`, passed.
- `cargo fmt --all --check`, passed.
- `cargo test -p rdocx-html`, 20 tests passed including doctests.
- `cargo test -p rdocx --test regression_test unrendered_number_formats_survive_without_decimal_coercion`,
  1 passed with an isolated target directory.
- `cargo test -p rdocx --lib epub_does_not_coerce_unrendered_standard_numbering`,
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

No correctness, contract, panic, OOXML child-order, namespace, preservation,
public API, equality, render, HTML, Markdown, EPUB, RTF, ODT, atomicity,
test-sensitivity, structure, smell, or nitpick findings were found.

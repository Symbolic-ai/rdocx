# F-247, default, pass 7

**Reviewed**: uncommitted working tree diff, 22 files, 8,618 changed lines
**Verdict**: 0 high, 1 medium, 0 low

## High

None.

## Medium

### M1, the producer fixture replacement drops end-to-end coverage for unformatted standard formats

`crates/rdocx/tests/regression_test.rs:5991`

Changing the fixture from `chicago` to `producerFormat` correctly makes this
test exercise the producer-defined branch named by the test. It also removes
the only end-to-end assertion in this regression suite that a modeled standard
format outside the implemented decimal, letter, and Roman families reaches
HTML and Markdown without an invented marker. The exhaustive test at
`crates/rdocx/src/document.rs:21566` proves that Chicago survives package
round-trip, but it does not render the reopened value. The focused layout test
at `crates/rdocx-layout/src/style_resolver.rs:578` also uses only a
producer-defined value. A regression that starts formatting Chicago or another
currently unformatted standard as decimal can therefore violate the contract
at `docs/hld/08-rendering-spec.md:758` without failing either test. Keep the
corrected producer-defined fixture, and add a separate modeled standard fixture
that asserts the no-invented-marker behavior through the text export boundary.

## Low

None.

## Earlier findings rechecked

- Standard `none` remains a supported RTF no-marker value at
  `crates/rdocx/tests/integration_test.rs:4282`.
- Producer-defined RTF values remain separated from standard typed values at
  `crates/rdocx/src/rtf.rs:1238` and retain the expected diagnostic.
- Update validation and instance removal continue to use the supported Word
  story owner set at `crates/rdocx/src/document.rs:9142`.
- Producer payload preservation, sparse level identity, typed attribute
  preflight, semantic `ListLevel` equality, and atomic graph mutation remain
  intact.

## Checks run

- `git diff --check`, passed.
- `cargo fmt --all --check`, passed.
- `cargo test -p rdocx --test regression_test producer_defined_number_formats_survive_save_and_reopen`,
  1 passed with an isolated target directory.
- `cargo test -p rdocx --test integration_test rtf_writer_preserves_none_numbering_without_coercion`,
  1 passed with an isolated target directory.
- `cargo test -p rdocx --lib rtf_writer_preserves_all_typed_numbering_formats_without_inventing_markers`,
  1 passed with an isolated target directory.
- `cargo test -p rdocx --lib all_public_numbering_level_properties_survive_reopen`,
  1 passed with an isolated target directory.

## Not found

No high-severity correctness, contract, panic, OOXML child-order, namespace,
preservation, public API, equality, render, RTF, ODT, EPUB, atomicity, or
structure findings were found. No low-severity findings were found.

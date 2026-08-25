# F-X054, all, fourth recovery pass 1

**Reviewed**: uncommitted working diff, 20 files, 3,616 changed lines with
3,531 additions and 85 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

The declaration error-classification defect is fixed. Namespace attribute
iteration errors convert through `OxmlError` at
`crates/rdocx/src/document.rs:336`, invalid UTF-8 declaration names convert
through the same variant at `crates/rdocx/src/document.rs:341`, and decoded
namespace value failures convert through it at
`crates/rdocx/src/document.rs:344`. The facade conversion at
`crates/rdocx/src/error.rs:10` retains `Error::Oxml`, and the Python binding
maps that exact variant to `XmlError` at `crates/rdocx-py/src/lib.rs:72`.

The native declaration regression at `crates/rdocx/src/document.rs:6342`
accepted ordinary and entity-escaped declarations, decoded `urn:a&amp;b` to
`urn:a&b`, and classified `xmlns:x="urn:&bad;"` as `Error::Oxml`. The Python
regression at `crates/rdocx-py/tests/test_shared.py:143` exercised the same
valid and malformed values through a source-built package. The distinct named
package, malformed event, and stale-handle boundaries at
`crates/rdocx-py/tests/test_shared.py:129`,
`crates/rdocx-py/tests/test_shared.py:136`, and
`crates/rdocx-py/tests/test_shared.py:166` retained `PackageError`, `XmlError`,
and `StaleElementError` respectively.

The focused native declaration test passed. All 37 Python binding tests passed
after rebuilding the exact Python 3.12.9 extension with maturin 1.13.3. All
168 `rdocx` regression tests passed, including namespace owner replay, Word
alias correlation, intermediate raw shadows, exact marker cardinality, ordered
body, cell, paragraph, hyperlink, and run facts, raw subtree identity, unknown
numbering preservation, and fail-closed visible text decoding.

No additional findings were found in namespace preservation, logical owner
matching, serializer-prefix collision handling, direct child ordering,
complete typed variant projection, legacy flattened accessors, panic safety,
OOXML child order, public enum exhaustiveness, documentation, dependency
structure, test naming, or the repository structural rules.

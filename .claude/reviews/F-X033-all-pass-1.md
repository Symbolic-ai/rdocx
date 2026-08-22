# F-X033, all, pass 1

**Reviewed**: integrated PR 36 range, 3 files and 86 changed lines, plus the uncommitted maintainer integration test, 1 file and 62 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, self-closing modeled body elements are reported as unsupported XML

`crates/rdocx-oxml/src/document.rs:744`
`crates/rdocx/src/document.rs:955`
`crates/rdocx/tests/integration_test.rs:68`

The body parser recognizes paragraphs, tables, content controls, and section
properties only on `Event::Start`. Its `Event::Empty` arm appends every element
other than `body` as `BodyContent::RawXml`. A valid self-closing `<w:p/>`
therefore appears through the new public iterator as `UnsupportedXml` instead
of `Paragraph`. A self-closing `<w:sectPr/>` also becomes an extra
`UnsupportedXml` item, while the semantically equivalent paired
`<w:sectPr></w:sectPr>` is parsed into `sect_pr` and stays outside
`body_items()`. The public result consequently changes with an XML lexical
choice rather than the body model. The opened-package gate uses nonempty
modeled items and deliberately or accidentally uses the paired section form,
so it remains green and does not expose either misclassification.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in the borrowed public lifetime, direct source-order
projection, raw-byte borrowing, accessor panic safety, public re-export, or
native-only additive surface. The enum and iterator are documented at
`crates/rdocx/src/document.rs:53` and `crates/rdocx/src/document.rs:941`, and
the crate-root export is at `crates/rdocx/src/lib.rs:40`. No Python, WASM, CLI,
manifest, dependency, or package-content change is present in the reviewed
range.

The unchanged recursive accessors still delegate to the content-control
collectors at `crates/rdocx-oxml/src/document.rs:578` and
`crates/rdocx-oxml/src/document.rs:599`. Their existing nested control
regression remains at `crates/rdocx/tests/regression_test.rs:2688`. Raw body
children are still written without modification at
`crates/rdocx-oxml/src/document.rs:763`.

The contributor unit gate at `crates/rdocx/src/document.rs:4789`, the public
opened-package gate at `crates/rdocx/tests/integration_test.rs:55`, and the
recursive-accessor regression all passed locally. The public gate would fail
to compile if the API were reverted, and it covers parsing plus public
traversal for the four intended variants in exact order. It does not cover the
self-closing forms in D1.

GitHub PR 36 is merged with Pedro Assumpcao's original commit
`79390535acba0a116b25ac986b863bdb941c8f15` retained under merge commit
`92951e71474383b48ce7ede194be4d0f34729488`. The local reconciliation merge
retains that merge record, and its feature patch has the same stable patch id
as the GitHub PR range. The recorded contributor and merge evidence is also
listed at `.claude/scratch/F-X033-progress.md:5`. Fresh current-base CI run
`32516942671` completed successfully before the merge, including the test,
documentation, clippy, MSRV, no-default-features, output-stability, binding,
WASM, and final CI-gate jobs.

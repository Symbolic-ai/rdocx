# F-136, all, pass 4

**Reviewed**: the complete working diff from `4b128371`, 26 implementation and contract files with 3,048 changed lines, plus all prior reviews and current progress evidence
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Resolved findings

Pass-3 D1 is resolved. Root slide and layout collections now capture a revision
path when fetched at `crates/rpptx-py/src/presentation.rs:55`. Their handles and
iterators validate that path at `crates/rpptx-py/src/slide.rs:29`,
`crates/rpptx-py/src/slide.rs:65`, `crates/rpptx-py/src/slide.rs:195`, and
`crates/rpptx-py/src/slide.rs:275`. Whole-text setters bump without rewriting
the receiver revision at `crates/rpptx-py/src/shape.rs:103`,
`crates/rpptx-py/src/text.rs:70`, and `crates/rpptx-py/src/text.rs:150`.
Structural append methods likewise leave their collection or frame receiver at
the captured pre-write revision and return only the new current handle. The
documented examples perform the corresponding minimal public re-fetches, for
example after text and paragraph replacement at
`crates/rpptx-py/tests/test_documented_examples.py:60` and after picture append
at `crates/rpptx-py/tests/test_documented_examples.py:115`.

An independent fresh-wheel probe held every public root and nested handle,
collection, and iterator across an unrelated slide append. Layout, slide,
shape, placeholder, text, paragraph, run, font, table, column, and cell views
all raised `StaleElementError`. The layout, slide, shape, paragraph, run, and
column iterator families also failed on their next access. Separate probes confirmed
that `add_slide`, `add_textbox`, `Shape.text`, `TextFrame.text`,
`Paragraph.text`, and `add_paragraph` stale the mutating receiver, while each
newly returned handle is current. The permanent regression covers root views
and structural receivers at
`crates/rpptx-py/tests/test_documented_examples.py:251` and
`crates/rpptx-py/tests/test_documented_examples.py:350`.

Pass-3 D2 is resolved. One shared formatter walks the concrete `ContentPath`
and emits every slide, repeated shape, row, cell, paragraph, and run index at
`crates/rpptx-py/src/lib.rs:43`. Each handle adds only its public property or
collection suffix through the common validator at
`crates/rpptx-py/src/lib.rs:69`. The three-deep nested group regression asserts
the complete exact strings for nested shapes, text frames, paragraph and run
collections, paragraphs, runs, and fonts at
`crates/rpptx-py/tests/test_documented_examples.py:396`. The top-level matrix
also pins exact concrete layout, slide, table, column, and cell paths at
`crates/rpptx-py/tests/test_documented_examples.py:305`.

## Earlier findings and sensitivity

All pass-1 and pass-2 remediations remain intact. Placeholder index omission
maps to zero, whole-text replacement stales descendants exactly once, and all
seven examples run in both writer and reader directions. Preset value 51 maps
to `homePlate` at `crates/rpptx-py/src/shape.rs:338`. The direct writer contract
at `crates/rpptx-py/tests/test_documented_examples.py:691` remains sensitive to
a one-sided preset mutation. An independent mutation routed rpptx PENTAGON
through CHEVRON. Both readers and the common normalized record still agreed on
that file, but the direct oracle-inspected writer contract rejected it.

The revised HLD strict-global rule at `docs/hld/10-bindings-spec.md:171`, the
two-writer gate at `docs/hld/12-testing-strategy.md:361`, and the bounded backlog
gate at `docs/hld/14-development-backlog.md:1041` match the implementation and
the revised example bodies. The HLD impact remains exactly the six approved
files.

## Independent checks

A fresh cp39-abi3 build installed into the disposable review environment and
the complete Python suite passed 10 tests with exact
`python-pptx==1.0.2`. `cargo test -p oxml-py-support` passed 6 tests. The focused
rpptx facade regression passed. `cargo check -p rpptx-py --all-targets` and
focused all-feature clippy with warnings denied passed. Formatting, prose,
generated-skill drift, and diff hygiene checks passed. The review-created
extension was moved out of the worker, and no generated extension, Python
bytecode, or test-cache artifact remains.

No fresh correctness, contract, panic, OOXML boundary, facade layering, PyO3
ownership, lazy indexing, slicing, iteration, revision, failed-mutation,
unit-conversion, oracle-version, release metadata, dependency-direction,
package-layout, HLD-impact, scope, formatting, or artifact finding was found.

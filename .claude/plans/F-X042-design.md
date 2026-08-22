# F-X042, Prove headers and footers in PDF output

**Status**: approved
**Sprint**: S52
**Size**: S
**Depends on**: F-168, F-X032

## Problem

The integration suite proves header and footer model round trips at
`crates/rdocx/tests/integration_test.rs:1187`, while facade unit tests exercise
layout selection in `crates/rdocx/src/document.rs:8456` and
`crates/rdocx/src/document.rs:8749`. There is no single public regression that
authors, saves, reopens, lays out, and renders all default, first, even,
inherited, and multi-section variants to deterministic PDF.

Issue 15 therefore remains unclosed at the output boundary even though the
individual model and layout paths exist.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "Relationship types" and "Package
  integrity".
- `docs/hld/08-rendering-spec.md`, "Performance" and "Word watermarks".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The golden-PNG
  gate".

## Approach

Add one module to the existing `rdocx` integration entrypoint, not a new test
binary. Construct the package in code through public authoring APIs plus the
minimum readable package XML needed for custom even and inherited variants.
Save, reopen with `Document::from_bytes`, call `layout`, and render through
`to_pdf_deterministic`.

Extract text from each `PageFrame` and from the final PDF content stream to
assert page-by-page default, first, even, inherited, blank-variant, and
multi-section selection. Also reopen the saved package through `OpcPackage` to
prove unrelated parts, content types, and relationships survive. If the test
exposes a remaining production drop, fix only the relationship resolution,
selection, placement, or backend path that loses the content.

## Rejected alternatives

- Another model-only test would not exercise relationship resolution,
  pagination, or PDF emission.
- A committed DOCX fixture would violate the repository's no-binary-fixture
  rule and hide the package structure under test.
- A screenshot comparison is weaker than exact text and placement assertions
  for this non-visual selection bug.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `authored_reopened_headers_and_footers_reach_pdf` | Public author, save, reopen, layout, and deterministic PDF paths place default, first, even, inherited, and multi-section header and footer text on exactly the intended pages. |
| regression | `blank_first_and_even_variants_do_not_borrow_defaults` | Selected blank variants suppress defaults in both page frames and PDF text. |
| round-trip | `header_footer_pdf_fixture_preserves_unrelated_package_state` | Unrelated parts, content types, relationships, and unmodelled XML survive the save and reopen used by the output proof. |

The test gate is **integration**. A readable in-code package passes through the
public `Document` facade and produces the expected header and footer text on
each applicable page in both `WordLayoutResult` and deterministic PDF output.
Blank first or even variants do not borrow defaults, inherited variants remain
visible, unrelated package parts survive, and the hash harness is unchanged.

## HLD impact

None. Current intent already specifies variant selection and backend behavior.
If the regression exposes a behavior gap, revise the plan before changing HLD.

## Risk routing

- Layout and pagination: re-read `docs/hld/08-rendering-spec.md`. Run the PDF
  regression and every render check with deterministic font mode.
- Any parser or serialiser: re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. The in-code package must round-trip
  relationships and unrelated XML without changing schema order.

## Hash harness

Expected to be unchanged. This is test-only unless the public regression
reveals an in-scope production drop.

## Implementation checklist

- [ ] Add the case to the existing `rdocx` integration binary.
- [ ] Construct all header and footer variants and unrelated package state in
      readable source.
- [ ] Save, reopen, layout, and render through public facade methods.
- [ ] Assert exact page-frame and PDF text selection and placement.
- [ ] Fix only an exposed production drop, if any, and revise HLD impact first.
- [ ] Run focused integration, deterministic PDF, and hash checks.

## Open questions

None. The story is an end-to-end evidence gap with an explicit fallback to the
smallest exposed production fix.

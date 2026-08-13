# F-136, all, pass 3

**Reviewed**: the complete current working diff from `4b128371`, the revised approved plan and HLD impact, both prior microscope reviews, progress notes, and focused gates
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, strict global invalidation still has receiver and root-collection exceptions
`crates/rpptx-py/src/shape.rs:126`

The owner pyclass fields and owner-based validation branches from pass 2 are
gone, but structural text setters still bump the global revision and then
overwrite the existing receiver's captured revision. `Shape.text` does this at
`crates/rpptx-py/src/shape.rs:126`, `TextFrame.text` at
`crates/rpptx-py/src/text.rs:88`, and `Paragraph.text` at
`crates/rpptx-py/src/text.rs:171`. The regression explicitly requires each
pre-write receiver to remain readable at
`crates/rpptx-py/tests/test_documented_examples.py:327`. A held root slide
collection is another exception because `PySlideCollection` has no
`ContentPath` at `crates/rpptx-py/src/slide.rs:158` and reads the current length
without revision validation at `crates/rpptx-py/src/slide.rs:180`. With the fresh
wheel, both `shape.text` after assigning `shape.text` and `len(slides)` after
`slides.add_slide(...)` succeeded. This contradicts the revised HLD statement
that strict global invalidation stales every pre-write handle and collection at
`docs/hld/10-bindings-spec.md:177`, as well as the construction-time capture
rule at `docs/hld/10-bindings-spec.md:59`. The documented workflows also rely on
the receiver exception, for example assigning `p.text` and then using `p.level`
without a public re-fetch at
`crates/rpptx-py/tests/test_documented_examples.py:62`. Pass-2 D2 therefore
removed the owner mechanism but did not implement the revised strict contract.

### D2, nested shape handles still report an incomplete public recovery path
`crates/rpptx-py/src/shape.rs:92`

`PathSeg::Shape` is intentionally repeatable, and the binding publicly exposes
nested collections through `Shape.shapes` at
`crates/rpptx-py/src/shape.rs:100`. Nevertheless, every shape validates with the
single-level recovery path `prs.slides[i].shapes[j]` at
`crates/rpptx-py/src/shape.rs:92`, and every shape collection uses the slide-root
path `prs.slides[i].shapes` at `crates/rpptx-py/src/shape.rs:202`. Text and table
handles beneath a nested shape likewise hard-code one shape step, for example
`crates/rpptx-py/src/text.rs:64` and `crates/rpptx-py/src/table.rs:61`. A handle
returned from `prs.slides[i].shapes[j].shapes[k]` therefore receives guidance
that re-fetches its parent group or the slide-root collection, not that handle.
The exact-message matrix constructs only immediate slide shapes at
`crates/rpptx-py/tests/test_documented_examples.py:276`, so its assertions at
`crates/rpptx-py/tests/test_documented_examples.py:296` cannot detect this
truncation. Pass-2 D3 is resolved for top-level handles but not for the approved
repeatable nested-shape path surface.

## Smells

None.

## Nitpicks

None.

## Resolved findings and evidence

Pass-1 D1 remains resolved: omitted placeholder indices use the OOXML default
of zero. Pass-1 D2's descendant-retargeting failure remains resolved: each
whole-text replacement bumps once and stale descendants fail. Pass-1 D3's
seven-example and reverse-writer coverage remains present.

Pass-2 D1 is resolved. The gate now directly compares normalized writer output
and an oracle-inspected geometry, placeholder, shape-kind, and preset contract
at `crates/rpptx-py/tests/test_documented_examples.py:593`. The preset mapping is
`51 => "homePlate"` at `crates/rpptx-py/src/shape.rs:362`, matching pinned
python-pptx 1.0.2. An independent one-sided mutation changed rpptx's PENTAGON
input to the CHEVRON value. Both per-writer reader comparisons and the common
normalized records still passed, while the direct writer contract assertion
failed as required. This demonstrates sensitivity to the exact preset-51
semantics rather than merely to readable output.

The fresh cp39-abi3 extension built successfully and the complete focused
Python suite passed 9 tests with exact `python-pptx==1.0.2`. Independently,
`cargo test -p oxml-py-support` passed 6 tests, the focused rpptx facade
regression passed, and `cargo check -p rpptx-py --all-targets`,
`cargo fmt --all --check`, `python3 scripts/prose_check.py`, and
`git diff --check` passed. No owner fields, owner-refresh validation branches,
validation mismatch conditions, generated extension, Python bytecode, or test
cache artifacts remain in the worker. No fresh OOXML default-index, facade
totality, panic, PyO3 ownership, unit conversion, oracle pin, dependency,
release metadata, HLD file-list, formatting, prose, or artifact findings were
found beyond the two contract defects above.

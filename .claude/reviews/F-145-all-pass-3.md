# F-145, all, pass 3

**Reviewed**: uncommitted `work/f-145-codex` implementation, 5 files and 364
changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the public equality contract has no valid HLD completion path

`.claude/plans/F-145-design.md:73`

The revised implementation adds public `PartialEq` and `Eq` behavior to
`ShapeRef`, but the HLD impact work list still starts at the rendering spec and
omits `docs/hld/06-presentationml-model.md`, whose public-facade section owns and
already describes `ShapeRef`. Completion updates exactly the files in this
list, so it cannot document the new identity semantics in the owning spec. The
same plan also still says to add no new facade API at
`.claude/plans/F-145-design.md:84`, contradicting both the approved approach and
the following public-API risk rider. Revise the plan to add HLD06 and remove the
stale no-new-facade assertion before completion.

## Smells

None.

## Nitpicks

None.

## Pass 2 re-evaluation

- **D1, field-only titles are still emitted twice**: resolved. `ShapeRef`
  equality uses exact borrowed `ShapeTreeChild` identity, so the title handle
  compares equal to its traversal handle without depending on exposed text-run
  or property addresses. The valid field-only fixture now produces the title
  once while retaining every distinct body and grouped shape.

## Not found

- **Public API semantics**: `std::ptr::eq` gives reflexive identity equality for
  two handles to the same underlying shape and keeps distinct shape nodes
  unequal regardless of equal content. The implementation adds trait behavior
  to the existing concrete handle and introduces no trait, type, generic,
  dependency, module, or file.
- **Outline behavior**: direct title comparison suppresses only the exact title
  shape. Ordinary shape paragraphs, row-major unspanned table cells, recursive
  group children, and selected alternate-content fallback children retain
  document order. Empty paragraphs are omitted, levels control indentation,
  and line-break controls remain normalized.
- **Thumbnail behavior**: derived deterministic DPI produces a 320-pixel-wide
  slide-one PNG, preserves a nonstandard aspect ratio, and passes dimensions
  through the existing pixel-budget validation before rasterization.
- **Tests and sensitivity**: all 14 CLI integration tests passed with the
  50-deck pinned corpus. The field-only exact-output regression distinguishes
  same-node title equality from distinct body shapes, and the progress record
  documents a bypass mutation that failed this test before byte-identical
  restoration. The `rpptx` suite passed 19 unit and 86 integration tests with 7
  ignored when run with a writable isolated `uv` cache.
- **Panics, OOXML, and structure**: production paths add no `unwrap`, `expect`,
  indexing, raw PresentationML access, serializer mutation, or schema-order
  change. Test-only package surgery is bounded by fixture assertions and stays
  in the existing integration binary.
- **Hygiene**: CLI clippy passed with warnings denied. Prose validation,
  generated-skill drift, and `git diff --check` also passed.

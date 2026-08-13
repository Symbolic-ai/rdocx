# F-145, all, pass 4

**Reviewed**: uncommitted `work/f-145-codex` implementation, 5 files and 368
changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 3 re-evaluation

- **D1, the public equality contract has no valid HLD completion path**:
  resolved. The plan now cites `docs/hld/06-presentationml-model.md`, includes it
  in the exact HLD impact work list, and no longer contradicts the approved
  public `ShapeRef` trait addition in the CLI risk rider.

## Prior-pass re-evaluation

- **Pass 2 D1, field-only titles are emitted twice**: remains resolved. Exact
  borrowed-node equality suppresses the field-only title traversal handle and
  retains distinct body and grouped shapes.
- **Pass 1 D1, collapsed placeholder-index title suppression**: remains
  resolved. Title suppression no longer depends on the non-total placeholder
  index.
- **Pass 1 D2, DrawingML line-break control output**: remains resolved.
  Carriage returns, line feeds, and U+000B are normalized before title and
  paragraph output.

## Not found

- **Contract and HLD impact**: the implementation matches the revised approved
  contract. The impact list now covers the owning presentation facade spec,
  rendering, CLI bindings, testing, backlog, and package mechanism documents.
- **Public API semantics**: `ShapeRef` equality is reflexive exact node identity
  through `std::ptr::eq`. Equal-content distinct shapes remain unequal, while
  separate borrowed handles for the same shape compare equal. No unrequested
  public type or method was added.
- **Thumbnail correctness and resources**: deterministic derived DPI produces
  exactly 320 pixels in width, preserves nonstandard aspect ratio, and uses the
  existing checked raster-dimension budget before allocation. Empty decks,
  invalid dimensions, raster failure, and output I/O errors propagate.
- **Outline correctness**: the title is printed once, every non-title textual
  paragraph is visited in recursive z-order, table continuations are skipped,
  empty paragraphs are omitted, and paragraph levels determine indentation.
- **Tests and sensitivity**: the source and test implementation are unchanged
  from pass 3, where all 14 CLI integration tests passed with the 50-deck
  corpus and the `rpptx` suite passed 19 unit and 86 integration tests with 7
  ignored. The exact-output tests distinguish same-node title identity,
  distinct text shapes, fixed thumbnail width, aspect ratio, recursive order,
  line-break normalization, and paragraph-level indentation. Recorded
  mutations failed before byte-identical restoration.
- **Panics, OOXML, and structure**: production code adds no panic path, raw XML
  access, parser or serializer mutation, schema-order change, new trait,
  generic, dependency, module, test binary, or file. Fixture-only XML surgery
  remains bounded by assertions in the existing integration binary.
- **Hygiene**: prose validation, generated-skill drift, and `git diff --check`
  passed on the pass 4 tree.

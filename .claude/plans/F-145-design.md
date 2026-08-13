# F-145, rpptx-cli thumbnail and outline

**Status**: completed
**Sprint**: S36
**Size**: M
**Depends on**: F-144

## Problem

The base presentation CLI does not provide the two presentation-specific
surfaces named by HLD10. Content-management systems need a predictable slide
one thumbnail, while indexing and language-model consumers need a stable title
and bullet-tree outline without parsing PresentationML.

The phrase "fixed size" is not currently quantified, and the outline boundary
must be precise enough that tests can distinguish the title from ordinary text
and preserve nested paragraph levels.

## Spec reference

- `docs/hld/06-presentationml-model.md`, "Public facade".
- `docs/hld/08-rendering-spec.md`, deterministic presentation rendering.
- `docs/hld/10-bindings-spec.md`, "CLIs".
- `docs/hld/12-testing-strategy.md`, CLI rendering gates.
- `docs/hld/14-development-backlog.md`, "F-145, rpptx-cli thumbnail and outline".
- `docs/hld/15-build-and-toolchain.md`, CLI package gates.

## Approach

Extend the F-144 executable with two commands in its existing source and
integration-test files.

- `thumbnail <file> [-o path]` renders slide one with deterministic fonts at
  exactly 320 pixels wide and preserves the presentation aspect ratio. The
  implementation derives the DPI from the slide width and uses the shared
  default output-path helper when `-o` is absent.
- `outline <file>` prints each slide title followed by every non-title textual
  paragraph in recursive shape z-order. Paragraph level controls indentation.
  Empty text is omitted and grouped shapes retain their document order.

Use the `rpptx` facade accessors for slide title, recursive shapes, text frames,
paragraphs, and paragraph levels. Add `PartialEq` and `Eq` for `ShapeRef` using
underlying shape identity so field-only and property-free title shapes can be
suppressed exactly during recursive traversal. Do not add placeholder APIs or
read raw PresentationML. F-144's one integration binary remains the sole test
entrypoint.

## Rejected alternatives

- Use a fixed DPI. That does not produce a fixed pixel width across different
  slide dimensions.
- Stretch to a fixed width and height. It distorts nonstandard aspect ratios.
- Add a placeholder-type facade API. Exact `ShapeRef` identity is the smaller
  surface and avoids format-specific placeholder assumptions.
- Create another test file. The F-144 integration entrypoint already owns this
  executable.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `thumbnail_and_outline_match_the_presentation_contract` | Thumbnail is a valid 320-pixel-wide PNG of slide one and outline contains its title and bullet tree |
| integration | nonstandard slide aspect ratio | Thumbnail height is proportional and pixels are not stretched |
| integration | nested and grouped text outline | Recursive z-order and paragraph-level indentation are stable, with no duplicated title |
| regression | field-only title identity | A title containing only a field is emitted once and remains distinguishable from other text shapes |
| regression | default output contract | Omitted thumbnail output uses the shared path helper and explicit output wins |

Sensitivity changes the fixed width and drops paragraph-level indentation
independently. The exact gate must fail before byte-identical restoration and a
green rerun.

## HLD impact

- `docs/hld/06-presentationml-model.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Layout and rendering. Use deterministic fonts for every assertion, run the
  presentation render gates, and require unchanged hash and golden baselines.
- Public behavior of the published CLI. Preserve every F-144 command, run the
  publication dry run and archive-size check.
- Public API of published `rpptx`. `PartialEq` and `Eq` on `ShapeRef` are
  additive and compare underlying shape identity. Run the facade tests,
  publication dry run, archive-size check, and a field-only identity mutation.

## Hash harness

Expected unchanged. The commands consume presentation rendering and do not
alter Word samples or renderer defaults.

## Implementation checklist

- [x] Add thumbnail and outline to the existing CLI source files.
- [x] Implement proportional 320-pixel slide-one rendering.
- [x] Implement title and recursive bullet-tree output through the facade.
- [x] Add exact `ShapeRef` identity and field-only title coverage.
- [x] Extend the existing integration binary with gate and edge cases.
- [x] Run deterministic rendering, publication, and hash riders.

## Open questions

None. The exact thumbnail contract is approved at 320 pixels wide with
proportional height. Additive `PartialEq` and `Eq` for `ShapeRef` are approved
as the smallest total identity seam for field-only title suppression. No
additional new file is required.

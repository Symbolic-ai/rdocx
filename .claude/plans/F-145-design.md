# F-145, rpptx-cli thumbnail and outline

**Status**: approved
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

Use only existing `rpptx` facade accessors for slide title, recursive shapes,
text frames, paragraphs, and paragraph levels. Do not add placeholder APIs or
read raw PresentationML. F-144's one integration binary remains the sole test
entrypoint.

## Rejected alternatives

- Use a fixed DPI. That does not produce a fixed pixel width across different
  slide dimensions.
- Stretch to a fixed width and height. It distorts nonstandard aspect ratios.
- Add a placeholder-type facade API. Existing title and text accessors provide
  the required outline without broadening the public surface.
- Create another test file. The F-144 integration entrypoint already owns this
  executable.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `thumbnail_and_outline_match_the_presentation_contract` | Thumbnail is a valid 320-pixel-wide PNG of slide one and outline contains its title and bullet tree |
| integration | nonstandard slide aspect ratio | Thumbnail height is proportional and pixels are not stretched |
| integration | nested and grouped text outline | Recursive z-order and paragraph-level indentation are stable, with no duplicated title |
| regression | default output contract | Omitted thumbnail output uses the shared path helper and explicit output wins |

Sensitivity changes the fixed width and drops paragraph-level indentation
independently. The exact gate must fail before byte-identical restoration and a
green rerun.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Layout and rendering. Use deterministic fonts for every assertion, run the
  presentation render gates, and require unchanged hash and golden baselines.
- Public behavior of the published CLI. Preserve every F-144 command, run the
  publication dry run and archive-size check, and add no new facade API.

## Hash harness

Expected unchanged. The commands consume presentation rendering and do not
alter Word samples or renderer defaults.

## Implementation checklist

- [ ] Add thumbnail and outline to the existing CLI source files.
- [ ] Implement proportional 320-pixel slide-one rendering.
- [ ] Implement title and recursive bullet-tree output through the facade.
- [ ] Extend the existing integration binary with gate and edge cases.
- [ ] Run deterministic rendering, publication, and hash riders.

## Open questions

None. The exact thumbnail contract is approved at 320 pixels wide with
proportional height. No additional new file or public facade API is required.

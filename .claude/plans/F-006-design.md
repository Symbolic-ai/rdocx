# F-006, Fix the JPEG standalone-marker walk

**Status**: approved
**Sprint**: S01
**Size**: S
**Depends on**: none

## Problem

`jpeg_dimensions` reads a two-byte length for every marker at
`crates/rdocx-pdf/src/image.rs:58`. JPEG restart markers and other standalone
markers have no length field, so the current walk interprets following bytes as
a length and can skip past the real SOF marker or index truncated data.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "Media".
- `docs/hld/13-risks-and-open-questions.md`, "The JPEG marker walk".
- `docs/hld/12-testing-strategy.md`, "New tests the extracted crates need",
  the `oxml-media` truncation-loop requirement.

## Approach

Teach the existing JPEG walk which markers are standalone, including SOI, EOI,
TEM, and RST0 through RST7. Advance those markers by their marker width without
reading a length. For length-bearing markers, validate that the length exists,
is at least two, and remains within the input before advancing. Treat fill
bytes safely and return `None` for malformed or truncated data.

Keep the implementation in `crates/rdocx-pdf/src/image.rs` and extend its
existing unit-test module. Construct JPEG bytes in code, including an RST marker
before SOF and every truncation of that input.

## Rejected alternatives

- Pulling in a JPEG decoder was rejected because the renderer needs only
  dimensions and already passes compressed JPEG bytes through unchanged.
- Moving the parser to a new module was rejected because F-006 has one caller
  and the planned `oxml-media` extraction is a later story.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `jpeg_restart_marker_before_sof_preserves_dimensions` | A JPEG containing RST before SOF reports its encoded width and height. |
| unit | `every_truncated_jpeg_header_returns_without_panicking` | Every prefix of the constructed JPEG returns normally, with only complete inputs yielding dimensions. |

The **test gate** is `jpeg_restart_marker_before_sof_preserves_dimensions`,
plus the required truncation loop proving no prefix panics.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/13-risks-and-open-questions.md`

## Risk routing

none

## Hash harness

Expected to be unchanged. The sample set does not place standalone markers
before a JPEG SOF segment.

## Implementation checklist

- [ ] Classify standalone JPEG markers in the existing marker walk.
- [ ] Validate every length-bearing segment before indexing or advancing.
- [ ] Add the RST-before-SOF regression fixture in code.
- [ ] Add the full truncation loop and run the rdocx-pdf unit tests.

## Open questions

None.

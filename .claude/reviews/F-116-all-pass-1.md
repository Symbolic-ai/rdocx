# F-116, all, pass 1

**Reviewed**: complete working tree diff, 3 files, 678 changed lines, comprising 672 insertions and 6 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, pending viewer rows record slide counts that were never observed

`crates/rpptx/tests/integration.rs:96`

Both the pending Keynote row at line 96 and the pending Google Slides row at
line 105 store `slide_count: 10`. Their observations state that Keynote timed
out before a count was available and that the Google import has not happened.
The evidence test then asserts ten for every row at line 232, including those
pending rows. This turns the expected count into recorded evidence even when no
viewer produced it. Model the count as unavailable for a pending observation
and require ten only for a clean verdict.

### D2, the evidence gates accept a clean verdict with an explicitly pending observation

`crates/rpptx/tests/integration.rs:201`

The ignored Google Slides gate checks only that the row's verdict is `Clean`.
The evidence test at line 233 requires only a nonempty observation, so the
current text saying the signed-in import, count, conversion check, and export
remain pending already satisfies that field. Supplying any date and build and
flipping the verdict makes the Google portion of both gates green without
replacing the pending observation with a clean-open record. The same schema
also permits a clean Keynote row to retain its timeout observation. A clean row
must carry an observation that positively records the required operation, not
merely a nonempty string.

### D3, the ordinary-save test never calls the ordinary save API

`crates/rpptx/tests/integration.rs:166`

`ten_slide_write_api_deck_saves_as_presentation_and_show` obtains the ordinary
package through `to_bytes()`, while only the slideshow half calls a path-based
save method. The stable `.pptx` candidate at line 125 is likewise written with
`fs::write`. This misses the design contract at
`.claude/plans/F-116-design.md:33` to exercise ordinary save in the combined
deck. A regression confined to `Presentation::save` would leave both the
package test and all viewer checks green.

### D4, unignored tests race through one fixed candidate path

`crates/rpptx/tests/integration.rs:125`

`ten_slide_write_api_deck_validates_and_reopens` writes and hashes the fixed
candidate path here. The unignored evidence test independently calls
`write_f116_candidate` at line 227, which truncates and rewrites that same path
at lines 4350 through 4355. Rust runs tests concurrently by default. Either
hash process can therefore read while the other test has truncated or only
partly rewritten the file, producing a nondeterministic SHA failure even though
both intended byte strings are identical. The automatic tests need unique
temporary paths or synchronization around the reviewed candidate.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: no further findings. The collection sequence produces the
  declared final order of ten slides, including the duplicated slide 08, and
  the recorded temporary candidate independently hashes to
  `d36da6e8849eabd4487d2572baea19c3716ee7d0fe03aaa4714a28ce3c41de4f`.
- Contract: no further findings. The builder calls the F-107 through F-115
  story surfaces for slide creation, validation, mutation, constructors,
  pictures, text, tables, collection edits, properties, and slideshow output.
  No production API or durable binary fixture was added.
- Panics: zero production findings. The changed Rust is test-only code over a
  generated trusted fixture. Its assertions and unwraps do not create a new
  panic path for library callers.
- OOXML: zero findings. The generated ordinary and slideshow packages check
  their main content types, validate cleanly, reopen through the facade, and
  retain the representative relationship graph. No production parser or
  serializer changed.
- Tests: no further findings. The final title sequence, unique slide ids,
  picture relationship scopes, one deduplicated media part, constructor kinds,
  table presence, hidden state, background, core properties, and slideshow
  content type are asserted. The corrected one-pixel PNG has a valid zlib
  stream for one RGB scanline and a matching IDAT CRC, so it is a legitimate
  test-only prerequisite for the LibreOffice import.
- Structure: zero findings. The diff adds no file, module, test binary, trait,
  generic, wrapper, feature, dependency, or production abstraction.
- External evidence: no further findings. PowerPoint and LibreOffice are
  recorded clean at the pinned versions and builds. Keynote and Google Slides
  remain explicitly pending human-action evidence, and HLD 12 accurately says
  the acceptance gate is incomplete until both are observed cleanly.

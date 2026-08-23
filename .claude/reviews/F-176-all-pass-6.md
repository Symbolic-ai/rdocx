# F-176, all, pass 6

**Reviewed**: the complete worker diff from base `d73dc2b`, including the untracked RTF module, across 8 implementation and test files with 3,719 added lines and 0 removed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, a group boundary can still split a UTF-16 surrogate pair
`crates/rdocx/src/rtf.rs:993`

`open_group` checks starred destinations, Unicode fallback width, and then
flushes the parent state before creating a child, but it never rejects a
pending high surrogate. The child state copies that pending surrogate at
`crates/rdocx/src/rtf.rs:380`, so `{\rtf1\uc0\u-10179{\u-8704}\u-8704}` lets
the nested group consume the copied high surrogate and emit the emoji, then
lets the parent consume the same low surrogate and emit it again. A group
boundary is the same malformed interruption class already guarded for control
symbols, control words, binary controls, text, and close braces, so this leaves
the surrogate-pair parser bounds incomplete.

### D2, the differential record still does not bind caps, small-caps, or hidden text to the correct run
`crates/rdocx/tests/integration_test.rs:98`

`WordRtfRunRecord` records text, font, size, emphasis, colour, highlight,
position, vertical alignment, and image presence, but has no field for all
caps, small caps, or hidden text. The only oracle check for those three
properties is the document-wide XML marker scan at
`crates/rdocx/tests/integration_test.rs:225`, with expected global markers at
`crates/rdocx/tests/integration_test.rs:384`. The source contains
`{\caps C}{\scaps M}{\v V}`, but a projection that writes `w:caps`,
`w:smallCaps`, or `w:vanish` on the wrong run, or leaves one active for later
runs, still satisfies the captured per-run records. That means the named
differential gate is not a structural comparison for this part of the
implemented Word-written run formatting surface.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Prior defect verification

- Pass 1 D1 to D14 were rechecked against the current diff. The differential
  fixture, default font handling, legacy code pages, Unicode alternate
  destination grammar, malformed numeric controls, root marker placement,
  paragraph formatting, table projection, table-cell numbering, list fallback
  diagnostics, picture ordering and ownership, picture transform diagnostics,
  path input bound, and cloned-buffer bound are fixed or superseded by the
  current stricter paths.
- Pass 2 D1 to D14 were rechecked. Breaks and tabs project as OOXML run
  content, line spacing distinguishes at-least, exact, and automatic modes,
  Unicode alternate destinations require exactly two branches with `\*\ud` in
  the second branch, destinations must begin groups, font charset handling was
  tightened, picture-after-table order is preserved, row-boundary differences
  diagnose, picture parameters are required, list levels and lookup references
  fail closed, collection bounds exist, and control symbols cannot interrupt
  surrogate pairs.
- Pass 3 D1 to D9 were rechecked. List override levels finalize after `\ls`,
  Word special-character controls emit text, non-`u` control words reject
  pending surrogate pairs, malformed table row cardinality fails, font charset
  controls require known values, dropped list properties diagnose,
  `\levelnfcn` remains authoritative, Unicode list markers are not decoded as
  ANSI bytes again, and root character-set declarations are restricted to the
  header.
- Pass 4 D1 to D7 were rechecked. Symbol bytes use a dedicated decoder, root
  character-set declarations fail after header tables, stray cell boundaries
  fail, `\ls0` clears numbering, binary controls cannot interrupt surrogate
  pairs, page geometry controls diagnose, and the oracle surface was expanded.
  D2 above is the remaining gap in that expanded oracle surface.
- Pass 5 D1 is fixed. The former visible no-op controls now emit stable
  diagnostics from the document-formatting diagnostic arm at
  `crates/rdocx/src/rtf.rs:1577`, and the focused test asserts the exact
  offsets and messages at `crates/rdocx/src/rtf.rs:2768`.
- Pass 5 D2 is fixed. `finish_picture` now copies the current paragraph state
  before appending the image at `crates/rdocx/src/rtf.rs:2181`, and the focused
  EOF picture-only paragraph test asserts alignment, indents, spacing, line
  spacing, and image preservation at `crates/rdocx/src/rtf.rs:2810`.

## Not found

- No additional public API expansion was found. The additive byte and path RTF
  APIs match the design plan shape.
- No new trait, generic parameter, feature flag, or crate was introduced. The
  approved private `rtf` module and direct `encoding_rs` dependency remain the
  structural changes.
- Existing OOXML parsers and serializers were not changed, so no new
  unmodelled-XML preservation or schema child-order issue was found in this
  diff.
- Parser bounds cover input bytes, group depth, lookup entries, diagnostics,
  blocks, runs, table cells, picture bytes, and retained output bytes. Apart
  from D1, no additional parser-bound or malformed-input defect was found.
- Diagnostics were re-audited across the former no-op arm. Apart from D2's
  test-gate limitation, no additional silent visible-control drop was found.

## Checks

- Diff check: `git diff --check d73dc2b` passed.
- Prose check: `python3 scripts/prose_check.py .claude/reviews/F-176-all-pass-6.md` passed.

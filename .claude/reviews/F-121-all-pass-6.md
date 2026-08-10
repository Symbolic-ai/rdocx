# F-121, all, pass 6

**Reviewed**: working diff from claim base `7e2794b`, 1 source file and 1,827 changed lines
**Verdict**: 0 defects, 1 smell, 0 nitpicks

## Defects

None.

## Smells

### S1, the CRLF shortcut can still consume a valid first pixel byte

`crates/rpptx-chart/src/lib.rs:8369`

The P6 parser consumes two bytes whenever the required separator is `0x0d`
and the next byte is `0x0a`. P6 permits one whitespace separator followed by
arbitrary binary samples, so a valid buffer whose separator is carriage return
and whose first sample is line feed is indistinguishable here from the added
CRLF extension. The parser drops that first sample and then panics at the pixel
length assertion. The new regressions cover LF separators followed by each
whitespace-valued sample and a CRLF separator followed by space, but not this
remaining combination. Use the already known raster length to distinguish a
two-byte CRLF header ending from a one-byte carriage-return separator followed
by an LF-valued sample, and add the latter regression.

## Nitpicks

None.

## Pass 5 remediation

- Pass 5 S1 is fixed for the ordinary LF header emitted by the pinned Poppler
  gate. Space, line feed, carriage return, and tab first-sample bytes remain in
  the pixel slice.
- The parser consumes only the header delimiter after a non-CR maximum-value
  token, so it no longer skips an arbitrary run of binary whitespace samples.
- A separate regression exercises the accepted CRLF header form followed by a
  space-valued first sample.

## Not found

- Correctness: no wrong enum mapping, range check, default, boolean handling,
  plot-axis resolution, reciprocal-axis validation, or repeated-item match was
  found.
- Contract: supported single-family plot areas own one typed plot and their
  axes, unsupported and combination choices remain opaque, and no F-125 native
  geometry scope was taken.
- Panics in production: no production panic, unchecked index, slice, or
  arithmetic overflow on untrusted ChartML input was found.
- OOXML: no namespace-alias, fixed-prefix, modelled-child sequence,
  repeated-child reconciliation, unsupported-plot preservation, extension
  preservation, or unknown-attribute defect was found.
- Tests beyond S1: malformed supported plots, duplicate modelled children,
  unresolved axes, colliding series identities, ordinary edits and reorders,
  axis reordering, family replacement, exact corpus coverage, and the zero-MAE
  viewer comparison are exercised.
- Structure: no new crate, file, module, dependency, trait, generic parameter,
  feature flag, forwarding wrapper, or unnecessary dynamic dispatch was found.

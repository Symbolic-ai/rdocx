# F-121, all, pass 2

**Reviewed**: working diff from claim base `7e2794b`, 1 file and 1,514 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, raw content between repeated plot children moves after the repeated run

`crates/rpptx-chart/src/lib.rs:4406`

Every bar series advances to the same raw boundary 3, and every bar axis id
advances to the same raw boundary 7. The line parser uses the equivalent fixed
boundaries. The writer then emits every series before boundary 3 and both axis
ids before boundary 7 at `crates/rpptx-chart/src/lib.rs:4668` and
`crates/rpptx-chart/src/lib.rs:4700`. A comment, processing instruction, or
whitespace between two `c:ser` children therefore moves after the second
series. The same movement occurs between the two `c:axId` children. This
violates the approved requirement that comments and whitespace retain their
ordered schema slots. Repeated modeled children need per-item raw boundaries,
as the existing cache point and plot-area axis code already use.

## Smells

None.

## Nitpicks

None.

## Pass 1 remediation

- D1 is fixed. Supported single-family plot selection now depends on the plot
  root at `crates/rpptx-chart/src/lib.rs:3995`, then typed axis validation runs
  unconditionally at `crates/rpptx-chart/src/lib.rs:4037`.
- D2 is fixed. The viewer threshold is exactly zero at
  `crates/rpptx-chart/src/lib.rs:6078`, and the SHA-bound comparison asserts it
  at `crates/rpptx-chart/src/lib.rs:7977`.
- D3 is fixed. The negative matrix includes an isolated duplicate grouping and
  unresolved axis references at `crates/rpptx-chart/src/lib.rs:7747`, plus the
  one-axis plot-area case at `crates/rpptx-chart/src/lib.rs:7757`.
- D4 is fixed. The corpus gate counts typed and opaque plots separately at
  `crates/rpptx-chart/src/lib.rs:7842` and asserts 11 typed bars, 2 typed lines,
  and one preserved bar-line combination at
  `crates/rpptx-chart/src/lib.rs:7899`.

The required-corpus `cargo test -p rpptx-chart` run passed all 33 tests. It
verified the pinned 50-deck corpus, the exact typed and opaque plot counts,
LibreOffice 26.2.5.2, Poppler 26.01.0, and zero normalized RGB MAE for both
SHA-bound representative candidates.

## Not found

- Correctness beyond D1: no wrong enum mapping, default, range check, boolean
  handling, axis resolution, or reciprocal-axis validation defect was found.
- Contract beyond D1: the public plot boundary owns plots and axis references,
  unsupported and combination choices remain opaque, and no F-125 native
  geometry scope was taken.
- Panics: no production panic, unchecked index, slice, or arithmetic overflow
  on untrusted ChartML input was found.
- OOXML beyond D1: no namespace-alias, fixed-prefix, modeled-child sequence,
  unsupported-plot preservation, extension preservation, or unknown-attribute
  retention defect was found.
- Tests beyond D1: malformed supported plots, duplicate children, unresolved
  axes, mutation preservation, exact corpus coverage, and the zero-MAE viewer
  gate are exercised. No test defect beyond the missing repeated-child raw-slot
  case was found.
- Structure: no new crate, file, module, dependency, trait, generic parameter,
  feature flag, forwarding wrapper, or unnecessary dynamic dispatch was found.

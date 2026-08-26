# F-X064, Accept whole-valued decimal table measurements

**Status**: approved
**Sprint**: S58
**Size**: S
**Depends on**: F-X059

## Problem

PR 55 reports Word-produced table measurements such as `9345.0`. The current
table width and default cell-margin parsers in
`crates/rdocx-oxml/src/table.rs` accept only integer lexical forms. One path
also converts an invalid typed table width to zero, which turns malformed or
unsupported input into a plausible layout value.

`CT_TblWidth.w` represents a wider OOXML union than the current integer field
can express. This story hardens the submitted outcome without claiming support
for percentage or universal-measure arms that need a lossless model.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, WordprocessingML namespace handling,
  typed projection, raw preservation, and schema-order serialization.
- `docs/hld/12-testing-strategy.md`, Word corpus and parser round-trip evidence.
- `docs/hld/14-development-backlog.md`, "F-X064, Accept whole-valued decimal table measurements".

## Approach

Use one exact string parser for the existing signed integer projection. Keep
integer lexical forms. Accept a decimal form only when it has a nonempty
fractional part and every fractional digit is zero, then checked-parse the
integer portion into `i32`. Do not use floating-point conversion.

Apply the parser to `CT_TblWidth` and default cell-margin numeric widths.
Missing `w:w` keeps its existing default. Fractional decimals, exponent forms,
empty fractional parts, overflow, malformed input, percentages, and universal
measures return an explicit parse error. They never become zero. Serialization
continues to emit the canonical integer form represented by the public model.

Use PR 55 at commit `056d48fdf23f35e3538ef3d6ff78cf9e3863e3a5`
as contribution evidence, then implement the hardened equivalent from the
integrated sprint head. Do not merge, retarget, comment on, or close the PR.

## Rejected alternatives

- Cherry-pick PR 55 unchanged. It does not define the unsupported union arms or malformed-input policy.
- Parse with `f64`. Large integers lose precision and non-finite forms become another case to reject.
- Preserve malformed-to-zero fallback. It hides invalid input as valid layout data.
- Model every percentage and universal-measure arm here. That is a larger public data-model change than the submitted compatibility fix.
- Add a new module or test binary. The parser and its tests belong in the existing table file.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `whole_valued_decimal_table_measurements_parse_exactly` | Integer, `9345.0`, and multiple-zero fractions parse for table width, cell width, table indent, and default cell margins |
| parser | `table_measurement_attributes_are_namespace_aware` | Aliased Word attributes parse and foreign same-local attributes do not |
| negative | `unsupported_or_malformed_table_measurements_fail` | Fractional, exponent, empty fraction, overflow, percent, unit, and malformed forms return errors rather than zero |
| round trip | `whole_valued_decimal_table_measurements_serialize_canonically` | Parsed values save as canonical integers in valid child order |
| integration | current Word corpus gate | Word documents open and render after the exact locked offline no-default build |

The **test gate is regression**. The focused parser and round-trip tests, the
current Word corpus job, and `/verify --full` must pass.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Unit conversion, `Twips`, `Emu`, `Points`, `Inches`**. Preserve the existing
  represented integer value exactly and require the hash harness to remain
  unchanged.
- **Any parser or serialiser**. Require prefix-tolerant Word namespace tests,
  fixed-prefix schema-order output, and byte-preserving coverage for unmodelled
  content.
- **An external oracle comparison**. Use the pinned Word corpus and accepted
  LibreOffice oracle only through the existing differential-testing contract.

## Hash harness

Expected unchanged at 49 of 49. PR 55's output-stability job was green, and
the accepted compatibility forms project to the same integer values already
rendered by the engine.

## Implementation checklist

- [ ] Add failing exact lexical-form and namespace tests in the existing table test module.
- [ ] Replace malformed-to-zero parsing with the shared checked string parser.
- [ ] Apply the parser to table widths and default cell margins.
- [ ] Preserve canonical serialization and unmodelled XML.
- [ ] Run focused parser tests, current Word corpus evidence, and risk riders.
- [ ] Run microscope and `/verify --full`.
- [ ] Record PR 55 and its exact source SHA in the handoff and delivery evidence.

## Open questions

None. The approved scope adds whole-valued decimal producer tolerance to the
existing integer projection. Other valid union arms remain explicit unsupported
input until a lossless model is designed.

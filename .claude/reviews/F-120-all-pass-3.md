# F-120, all, pass 3

**Reviewed**: `git diff --working` against claim commit `696d464`, one tracked
file with 1,769 changed lines, comprising 1,739 additions and 30 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, a fresh axis exposes a kind mutation that can never serialize

`crates/rpptx-chart/src/lib.rs:2372`

`Axis::new()` now stores the initial public `kind` as `parsed_kind`, and
`validate()` rejects every later difference. A caller can therefore create an
axis with no preserved type-specific content, assign a different value to its
public `kind` field, and receive an error from `to_xml()` even though that
relabel is schema-safe. The remediation should distinguish parsed axes whose
opaque tails require a stable root from newly constructed axes, or make the
root kind genuinely immutable instead of exposing a public mutation that the
writer always rejects.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-2 D1 is remediated. A constructed axis now compares equal to its
serialize-reparse result, and a parsed axis cannot move an incompatible opaque
tail under another root. No other correctness, contract, panic-safety, OOXML
namespace, schema-order, raw-preservation, graph-validation, test, or
structural finding was found.

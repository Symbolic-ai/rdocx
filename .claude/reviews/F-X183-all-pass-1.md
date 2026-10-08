# F-X183, all, pass 1

**Reviewed**: Frozen working diff on `work/f-x183-codex`, Base and HEAD `d4b8f5458678d1640de9d2fb709655ab437ca98b`. Sixteen tracked files, 911 added lines and 72 removed lines, plus three immutable DELTA review records.

**Verdict**: 1 defect, 0 smells, 0 nitpicks. D1 blocks completion. The separately reviewed baseline deltas remain justified and are not the cause of this finding.

## Defects

### D1, a leading accepted row without projected cells disables legacy compensation

`crates/rdocx-layout/src/table.rs:921`

The new compensation obtains the first cell only from `rows.first()`. The accepted traversal can legally produce an initial row with no projected cells while a later row contains the first laid-out cell. In that case the entire compensation block is skipped. This violates the approved first-accepted-cell contract for eligible legacy tables.

A concrete source is a top-level LTR, nonfloating table in compatibility mode 14 with explicit zero dxa indent, a one-column grid, an initial cell-level control with empty content and a following ordinary cell:

```xml
<w:tbl xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:tblPr>
    <w:tblW w:w="1440" w:type="dxa"/>
    <w:tblInd w:w="0" w:type="dxa"/>
    <w:tblLayout w:type="fixed"/>
    <w:tblCellMar><w:left w:w="108" w:type="dxa"/></w:tblCellMar>
  </w:tblPr>
  <w:tblGrid><w:gridCol w:w="1440"/></w:tblGrid>
  <w:tr>
    <w:sdt><w:sdtPr><w:id w:val="17"/></w:sdtPr>
      <w:sdtContent></w:sdtContent>
    </w:sdt>
  </w:tr>
  <w:tr><w:tc><w:p><w:r><w:t>Left</w:t></w:r></w:p></w:tc></w:tr>
</w:tbl>
```

This is not a speculative malformed zero-cell row. Microsoft's public Open XML SDK schema permits a cell-level SDT in the row content group, and cell SDT content is optional and may be empty. See the `w:CT_Row/w:tr`, `w:CT_SdtCell/w:sdt` and `w:CT_SdtContentCell/w:sdtContent` definitions in the [official schema metadata](https://raw.githubusercontent.com/dotnet/Open-XML-SDK/main/data/schemas/schemas_openxmlformats_org_wordprocessingml_2006_main.json), around lines 15125, 16932 and 35354.

The existing parser retains this typed SDT at `crates/rdocx-oxml/src/table.rs:2309`. Its content starts empty and the explicit empty content parses successfully at `crates/rdocx-oxml/src/content_control.rs:1027`. The row carries no deletion or move-away marker, so `accepted_table_rows` at `crates/rdocx-layout/src/table.rs:54` retains it. `layout_row_cells` at line 84 projects zero cells for that row, and `rows.push` at line 823 retains the empty laid-out row. The second row supplies a real cell with resolved left margin 5.4 pt. At line 921, looking only in row zero returns `None` despite that real accepted cell.

For this input the returned table indent stays zero instead of the contracted -5.4 pt. With a 72 pt page margin, the real cell's text starts at 77.4 rather than 72 pt. Right/end alignment similarly misses its contracted +5.4 pt compensation. Removing the empty leading control row changes placement despite leaving the first actual cell and its properties unchanged. These offsets are derived directly from the frozen code path and approved contract, not asserted as a new Word measurement. No Cargo reproduction or native capture was run during review.

The new matrix does not exercise this valid projected-row shape. Remediation must preserve source rows and control ownership while making first-cell selection obey accepted traversal. Review and verification must follow remediation separately.

## Smells

None.

## Nitpicks

None.

## Identity, contract and source review

Authenticated `/private/tmp/fx183-final-freeze.json`, SHA-256 `33b0df2f999d994453d825c5f3258e1d5ef5ab81323fd95f53fed5c1121d9a91`, and binary diff `48c5066fc2a988132a591b8f662bfaac4521b58ed9ae0ea62321d514def158f2`. Independently rehashed all 63 direct bindings, including sixteen tracked files, three DELTA reviews, forty proofs and four retained archives. Branch, HEAD, status and complete diff matched the freeze. Rechecked all bindings and diff immediately before writing. Read CLAUDE, WORKFLOW, microscope, approved plan, latest progress context, HLD04/08 and the applicable testing, API and unit contracts. F-X183 is in progress in both shared records.

All production hashes remain identical to DELTA pass 1. Side-margin absence is zero in intrinsic and final layout. Direct-cell overlays retain explicit zero. Base-first style resolution and direct alignment remain intact. The fixed built-in TableNormal exception changes only its effective base left edge under the exact qualified identity and metadata. Derived and direct overlays still win. Ordinary custom defaults retain authored values. No broader built-in recognition, right-padding exception, arbitrary effects merge or absolute native metric equivalence is claimed.

The effects consumer reads the actual main-part relationship, requires a sole internal target and guarded complete Word style-owner projection, and accepts only a sole self-contained default without an ID collision. It uses an owned layout style clone. Missing, malformed, external, duplicate and unprojected catalogues contribute no fallback. Save paths retain authored main styles, effects XML, settings, relationships and opaque parts. Namespace aliases and relocated main parts have focused preservation coverage. No parser or serializer source was changed.

Compatibility mode parsing uses the existing settings path. The concrete required public field is populated by all compiled callers, documented as an intentional pre-1.0 struct-literal break and included in retained-engine compatibility. Nested recursion supplies false explicitly, and floating and bidi controls remain excluded from compensation. Apart from D1, these source paths agree with the approved bounded contract.

## Native and baseline evidence

Independently rehashed all 491 unique native evidence bindings from the bound authentication report with zero errors. Minimal, official Grid, margin-cascade, right-margin, first-left, built-in, custom-default and effects observations remain separately qualified. Their relative geometry does not establish absolute native Arial versus deterministic bundled-font pixel parity.

All three DELTA records retain their frozen hashes. Their reviewed sample and geometry changes were recorded without widening tolerance or changing builders. The hash baseline changes exactly fourteen entries and retains 35. The golden manifest changes only invoice and quote RGBA identities. Dense, F266c, F268a fourteen-rectangle geometry and the explicit-autofit positive width arm retain the reviewed values and unchanged controls. No contributor baseline or proprietary oracle implementation was adopted.

## Tests, publication and policy

Read the actual bound gate logs. Layout passes 319 unit tests and one doctest. Facade passes 500 unit tests with six ignored and 360 integration tests with eight ignored. The original full regression attempt passes 869 with seven ignored and fails only the subsequently approved old width pin. The corrected exact target then passes. This is combined evidence, not a newly executed full 870-test pass. Facade doctests pass two, CLI passes three unit and 58 integration tests. The named reversion gate, inherited/direct controls, intrinsic overlay test and retained-engine/floating control have concrete fail-before or pass-after evidence. D1 remains uncovered.

Scoped Clippy, formatting, docs with warnings denied, Python binding checks and both WASM targets pass. Hash49 and golden7 pass. Workflow tests pass 140 with two skips in 100.570 seconds, including fresh README examples and archive inventory. Prose reports zero violations and all 26 generated adapters are in sync. Full workspace, no-default-features and dependency-deny runs are explicitly deferred or unearned scoped riders. They are not claimed here. Final integrated sprint verification remains due. No test was rerun by this reviewer.

Independently opened all four retained measured and actual publish-dry-run archives. Both rdocx-layout archives measure exactly `(310873, 1681341, 15)` and both facade archives `(1525647, 8444832, 36)` under the existing VCS metadata normalization. These match the README rows and policy constants. All ten archived layout source members and all 25 facade source/test members match frozen files. Each archive is below 10 MiB. Actual publisher `package/tmp-crate` provenance is separately retained and bound. Patched publication dry runs perform verification and explicitly abort upload. The existing 64-byte compressed-size policy is unchanged and is not needed to excuse these retained measurements.

## Aspect results and handback

- **Correctness**: D1. No other correctness finding.
- **Contract**: D1 is the accepted-traversal gap. No additional contract finding.
- **Panics**: zero findings. New production option/error paths are guarded, and no new untrusted unwrap or indexing is introduced.
- **OOXML**: zero findings. Render-only style consumption preserves source ownership and schema order. D1 concerns geometry selection, not dropped XML.
- **Tests**: D1's missing leading-empty-projection control is part of that defect. No separate test finding. Reviewed recording and combined scoped evidence are reported at their actual limits.
- **Structure**: zero findings. No new production file, module, crate, dependency, trait or generic is introduced. Existing-file concrete control flow is used.

This pass ends here. D1 must be remediated in a separate implementation phase, followed by a fresh review of the new freeze and affected verification. No source, baseline, plan, HLD, progress or queue file was modified by this review. Only this review record was written.

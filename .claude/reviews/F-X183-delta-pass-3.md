# F-X183, delta, pass 3

**Reviewed**: Frozen supplemental structural-width delta on `work/f-x183-codex`, Base and HEAD `d4b8f5458678d1640de9d2fb709655ab437ca98b`. Complete working diff contains thirteen tracked files, 901 added lines and 63 removed lines. This supplement introduces no production change beyond DELTA pass 1.

**Verdict**: 0 defects, 0 smells, 0 nitpicks within DELTA3 scope. Recording the positive-arm widths `[9.54, 204.39]` is justified. The negative-arm grid must remain `[72.0, 360.0]`. Final ALL microscope and complete scoped verification remain required.

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Frozen provenance

Independently authenticated `/private/tmp/fx183-width-freeze.json`, SHA-256 `a3ce438b620c1ce2222f8ec56d86a72d553a53afaccd1423e86197dba67481ab`, and complete diff `57396d678ea3f04d1498b0d435eee62bf8286cb87e1cbf2c0c8dee62183ae04d`. Rehashed all 25 direct bindings and matched branch, HEAD, exact status and binary diff. The detached Base is clean. Independently checked all 217 current-source hashes and 217 Base-source hashes in the two content-preserving freshness receipts. Both targeted execution logs show compilation from their respective source roots. No Cargo command was executed by this reviewer.

The original test, `shaded_table`, and the combined `lay_out_with` and `lay_out` spans remain byte-identical to Base and match the frozen span digests. The only `layout_input` difference is the required `legacy_table_positioning: false` field. That explicitly disables legacy compensation. Every production source hash remains identical to DELTA pass 1. The old positive-arm pin remains unchanged before this review.

## Concrete source and attribution

`crates/rdocx/tests/regression_test.rs:40431` constructs a two-column `CT_Tbl` with grid `[1440, 7200]` twips and two cells containing `ID` and `A considerably longer second column heading`. The unchanged helper at line 40123 authors grid, width/layout, shading and text without table or cell margins, selected table style or indent. `lay_out_with` at line 40093 uses deterministic fonts and `CT_Styles::new_default`. The default implementation at `crates/rdocx-oxml/src/styles.rs:799` supplies paragraph styles, not a default table style or horizontal table padding.

The negative arm sets auto width and absent layout, then asserts `[72.0, 360.0]`. The positive arm changes only the layout mode to explicit autofit, rounds the measured widths to two decimals, and asserts the old `[20.34, 215.19]` at `crates/rdocx/tests/regression_test.rs:40461`. Base passes. Current reaches and passes the unchanged negative assertion, then fails exactly at the old positive assertion with `[9.54, 204.39]`. The logs therefore demonstrate the distinction rather than assuming the engagement predicate from the final failure alone.

The explicit-autofit predicate at `crates/rdocx-layout/src/table.rs:1397` remains unchanged. Horizontal intrinsic margins at line 1450 now default to zero and retain direct-edge inheritance. Each actual positive column is exactly 10.8 pt narrower: 20.34 minus 9.54 and 215.19 minus 204.39. These cells contain no authored margins, so the difference is precisely removal of the old 5.4 pt padding on both sides, not the new direct-cell overlay behavior.

The original F-268a pass 2 review's D2 closure explicitly describes adding horizontal margins once to both measurements and re-pinning this same corpus-shaped guard alongside its golden. F-X183's approved zero-fallback contract removes that previously unqualified padding. The explicit predicate and declared negative grid remain intact. No package, effects style consumer, built-in TableNormal exception or native application is involved in this direct structural test.

## Boundaries and remaining gates

The previously reviewed fourteen sample hash changes remain the only sample delta. DELTA3 requests only the two positive-arm scalar values. No PDF, raster or new native capture is needed or claimed for this direct width observation. The rounding precision and existing assertion strength must remain unchanged when recording.

Source freshness, regression sensitivity, exact width attribution, baseline exclusivity and preservation of the negative control produced zero findings. Correctness, contract, panics, OOXML, tests and structure checks within this bounded structural attribution also produced zero findings. This is not native Word parity, a final all-aspect review, scoped verification or feature completion. Recording and final gates remain separate workflow steps.

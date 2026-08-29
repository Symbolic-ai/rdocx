# F-198, correctness, pass 4

**Reviewed**: reconstructed working-tree diff against
`bc478f8a06d37268d06cd41598037df1d91b0611`, 17 tracked implementation, HLD,
and baseline files with 942 additions and 24 deletions, plus 3 restored prior
review records with 124 lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the F-X062 engine acceptance workload was weakened below its approved boundary
`crates/rdocx-layout/src/engine.rs:8876`
`crates/rdocx-layout/src/engine.rs:8922`
`crates/rdocx-layout/src/engine.rs:8979`
`crates/rdocx-layout/src/engine.rs:9016`
`.claude/plans/F-X062-design.md:67`
`docs/hld/12-testing-strategy.md:414`

F-198 lowers all four related-story engine cases from 700 paragraphs to 640 so
the expanded retained run-property state remains below the unchanged restart
budget. That masks the production regression instead of preserving the
completed F-X062 contract. The named engine test and the current HLD require a
700-paragraph workload through both the engine and facade. The separate facade
case still uses 700 paragraphs, but it cannot replace the named engine checks
for note-clean restart, endnote completion, header and footer eligibility, and
changed-story invalidation. Restore the 700-paragraph engine boundary and make
the exact retained state fit, or stop for an approved contract change.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 3 D1 through D4 are closed. The self-closing settings rewrite retains the
root QName and attributes, adds or validates the fixed Word namespace binding,
and emits a matching end tag. Settings authoring reuses an existing resolved
target or chooses an unoccupied numbered target across parts, relationship
owners, and content-type overrides without overwriting the conventional part.
Malformed retained `w:lang` content uses the current modeled occurrence, stays
after the modeled child, and is byte-stable on repeated serialization. HLD 10
accurately states the intentional pre-1.0 full-literal source breaks from the
new public `CT_RPr` and `LayoutInput` fields as current compatibility reality.

No additional correctness, contract, panic or error-path, OOXML namespace or
schema-order, raw-preservation, public compatibility, test, or structural
findings were found. Paragraph suppression, inherited and mixed run language,
generated-field exclusion, note and table layout, drawing reflow, conditional
hyphen source spans, warm-versus-fresh invalidation, F-X063 font-elided context
matching, and F-X066 run preservation remain coherent. The declared hash delta
is limited to the five `feature_showcase` keys, deterministic golden and pinned
Writer evidence explain it, current packaging resolves registry
`oxml-layout@0.7.0`, and the immutable historical regression remains isolated
at `oxml-layout@0.6.0`.

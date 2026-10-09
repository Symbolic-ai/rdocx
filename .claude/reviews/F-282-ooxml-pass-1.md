# F-282, ooxml, pass 1

**Reviewed**: Frozen working source in `work/f-282-codex` at HEAD31524cc698be5eeaf0850aef0d7d913b1ec5a17f, with immutable claim base70e0b11fd6f2c0db35c6627fd9f50ca035bc30de. The F-282 working production delta comprises six files, 17363 added lines and 51 removed lines, including the new 16077-line bibliography module. The existing regression entrypoint adds335027 lines. The unchanged document facade was also inspected as the staging and physical ownership dependency. This aspect reviews qualified XML, source CRUD, scanner and structured cache ownership. In-progress worker HLD edits, formatter catalogue expansion, collation, final scoped verification and final ALL acceptance are excluded.
**Contract**: Canonical `.claude/plans/F-282-design.md` at79588295, the user-approved bounded S90 delivery. Remaining catalogue work is F-X192 in S91 and F-283 is carried. Read CLAUDE.md, WORKFLOW.md, microscope, the cited architecture field and cache sections, OPC relationship and integrity sections, and native facade stability.
**Verdict**: 2 defects, 0 smells, 0 nitpicks.

## Defects

### D1, first paragraph reset discards unknown Word namespace attributes

`crates/rdocx/src/bibliography.rs:1056`
`crates/rdocx/src/bibliography.rs:1094`

The metadata guard rejects attributes only when their namespace differs from WordprocessingML. It never checks their local names against the attributes owned by each recognized formatting particle. For an otherwise supported bibliography owner whose first paragraph contains `<w:ind w:left="360" w:producer="keep"/>`, the unknown `w:producer` attribute passes the guard. The whole `ind` element is then replaced by the generated particle, silently deleting that unmodeled attribute. The same problem applies to an absent generated counterpart, where the whole old particle is removed, and to recognized run property children through the recursive merge. Foreign namespace attribute refusal does not cover this case.

The approved contract requires namespace-qualified opaque producer data to survive or cause an atomic refusal. Owning a formatting element does not establish ownership of every possible attribute in its namespace. Preserve unowned attributes or reject their replacement atomically, with exact raw XML and revision controls for both the paragraph and recursive run property paths. Existing foreign attribute controls at `crates/rdocx/tests/regression_test.rs:67810` do not exercise unknown local names in the owned namespace.

### D2, source member rename assumes no whitespace before the closing bracket

`crates/rdocx/src/bibliography.rs:1502`

The closing name replacement range is calculated from the end of the complete element, assuming the closing name directly precedes `>`. XML permits whitespace between the name and that bracket. An imported `<b:Title>old</b:Title >` parses and projects normally, but replacing that property's identity with Year selects the closing byte span `:Title ` rather than `b:Title`. Together with the opening edit, this produces `<b:Year>old</bb:Year>`. The staged parser correctly prevents publication, but a valid supported source edit consequently fails solely because of retained legal XML spelling. Contributor role renames use the same helper.

Use the actual closing QName source span rather than deriving it from the full element end. Preserve trailing XML whitespace and exercise scalar and contributor role identity changes with legal closing-tag spacing, including namespace aliases. This is a static byte-range reproduction of the frozen helper, not a new Cargo runtime result. No source was modified during the review.

## Smells

None found in this aspect.

## Nitpicks

None recorded.

## Not found

- Qualified source identity: no additional defect found in expanded-name source and contributor selection, foreign lookalike exclusion, source locale identity, repeated property retention or duplicate expanded attribute refusal.
- Source graph and transactions: no additional defect found in normalized internal custom XML targets, unique collection and store ownership, staged publication, scalar opaque metadata refusal, exact source no-op replacement or citation reference deletion refusal.
- Instruction edits: no additional defect found in owned l/f/m token ranges, qualified instruction attribute discovery, character-reference byte maps, unowned spelling retention or malformed token refusal.
- Structured cache ownership: no additional defect found in physical story offset binding, outside prefix and suffix retention, simple owner attribute protection, lock retention, namespace closure, generated IEEE row group preservation or checked overlapping edits.
- Schema and opaque content: no additional defect found in the explicit property sequence, foreign property particle retention, paragraph history and section preservation, or scanner comment and PI handling. D1 and D2 remain the blockers identified above.
- Provenance: no proprietary XSL import or production execution was found in the inspected source. No catalogue or renderer parity claim is made by this aspect.

## Frozen source authentication

The seven source bindings below independently matched the checkpoint receipt before inspection and again immediately before writing this report. Receipt SHA256 is3bd0f8be5be9c81c959c1fdefeae5e35a4d940b193cfeb57490b8569e58929a3. No Cargo, Git mutation, UI operation, GitHub lookup or source write ran during this review. Development test receipts do not substitute for final scoped verification or ALL review.

| Worker path | SHA256 |
|---|---|
| crates/rdocx-layout/src/engine.rs | 90f328bb37575d257ae0b34fbeb1ef3ebdf6539f4c1dafe71ad4db84e3db9e6e |
| crates/rdocx-oxml/src/text.rs | cc6bb3ec2b64f2fec3aa06b55820386daa8ca42fd5de51020b53bd97bcd8f354 |
| crates/rdocx/src/field.rs | 63db78f9e205c56f2982c46a5999092c0041e00204256455669d7b6da07001d7 |
| crates/rdocx/src/lib.rs | 2071060310142958b0fb1276ee967bf69840d039c85afd19dc0209e28a4d32a0 |
| crates/rdocx/src/bibliography.rs | 31485a1f2061125662459770e3ff6654a524c29be1ecb9791d7508991d1fd767 |
| crates/rdocx/tests/regression_test.rs | 884ee40a35a01e8f991e722e967cc9c3949a0917c68466baf77144e2271ca3d4 |
| crates/rdocx/src/document.rs | 4057ef506a747776554fc57832eabc7daadc516f8ab8618b5af2961bb08910be |

This pass ends with the report. Remediation belongs to the implementing phase, followed by a new independent pass.

# F-280, microscope, pass 1

**Reviewed**: The frozen working diff on `work/f-280-codex` at Base and HEAD `cf75a087eeb00ad63f9883cd044a12e41066461d`, 29 tracked files, +10749/-1498 lines. Binary diff SHA-256 `3b3824a2cafb45ec86235bda1c66e666c3148d75e23d141fd5784a6aa3abc2b4`. All six default aspects were checked against the approved F-280 plan, cited HLD, workflow, risk routing and differential-testing guidance.

**Verdict**: 4 defects, 0 smells, 0 nitpicks. Completion is blocked pending remediation and another microscope pass.

This is an independent static review. No Cargo or UI commands were run and no implementation files were changed. Existing final scoped test logs and the compiling semantic reversion receipt were inspected as worker evidence, not represented as independently rerun verification. Genuine native captures qualify their measured owners and switches only. They do not prove the untested branches below.

## Defects

### D1, rendered same-paragraph REF position ignores the field's run boundary

`crates/rdocx-layout/src/engine.rs:10154`
`crates/rdocx-layout/src/engine.rs:9248`
`crates/rdocx/src/field.rs:12164`
`.claude/plans/F-280-design.md:481`
`crates/rdocx/tests/regression_test.rs:51274`

The renderer computes above or below from target and referring paragraph node IDs. Equal paragraph IDs always produce above. A `REF target \p` before a later target in that same paragraph therefore paints above, while the facade correctly resolves below from accepted run boundaries. A field inside its target also paints above instead of retaining its cache with the facade's diagnostic. The layout call passes only the paragraph source, so the corrected facade boundary comparison cannot reach this path. The named same-paragraph regression checks only an after-target pure evaluation and does not exercise before-target, containment, materialized reopen or rendered output. Carry the qualified accepted field and target boundaries into layout and cover these discriminators through both APIs and actual paint.

### D2, ordinary REF cannot resolve uniquely owned related-story targets

`crates/rdocx/src/field.rs:11552`
`crates/rdocx/src/comments.rs:760`
`crates/rdocx/src/field.rs:12092`
`crates/rdocx-layout/src/engine.rs:10465`
`.claude/plans/F-280-design.md:484`

Ordinary REF target lookup still starts from `Document::bookmarks()`, whose documented implementation inventories only main-body paragraphs. Layout likewise builds bookmark text with `visit_document_paragraphs`. A uniquely owned literal bookmark in a header, footer, note or selected text box therefore reports missing and retains OLD when a body REF should resolve its text. Caption authoring accepts related story owners and returns their bookmark names, but those names cannot be consumed by ordinary REF. The REF-f-specific story-range lookup does not repair the ordinary branch. Use the approved physical owner inventory for target qualification without inventing cross-story relative positions or a new sequence counter policy. Add related-target text and number-context consumers, rather than only asserting that caption bookmarks were allocated.

### D3, cross-story position requests silently replace caches with target text

`crates/rdocx/src/field.rs:12104`
`crates/rdocx/src/field.rs:12138`
`crates/rdocx-layout/src/engine.rs:10128`
`.claude/plans/F-280-design.md:483`

A header REF to an existing body bookmark with \p reaches the ordinary branch, skips position because `story != "main"`, and returns the bookmark text. Layout similarly returns bookmark text when either owner is outside the main story. The approved contract requires unavailable relative position to retain the stored display and report a diagnostic. For example, a header `REF BodyTarget \p` with cache OLD is materialized and painted as BodyTarget's literal text, with no unavailable-position diagnostic. Combined number and position requests also silently drop the position request. Preserve the complete field cache when requested position cannot be qualified, and prove this with cross-owner pure evaluation, update, reopen and deterministic rendering.

### D4, malformed producer SEQ bypasses instruction validation and advances the snapshot

`crates/rdocx/src/field.rs:11815`
`crates/rdocx/src/field.rs:11961`
`crates/rdocx/src/field.rs:14895`
`crates/rdocx-layout/src/engine.rs:1771`
`crates/rdocx-layout/src/engine.rs:1810`
`crates/rdocx-layout/src/engine.rs:1887`
`crates/rdocx-oxml/src/text.rs:12551`

The new snapshot SEQ branch bypasses `evaluate_instruction`, including the existing raw quote-balance validation. The collector checks parsed arguments and switch operands but never checks raw quoting. The preserving lexer deliberately returns a final token even when a quote remains open. Consequently an imported `SEQ Figure \* "ARABIC` with cache OLD is accepted as ARABIC, resolves to 1 and advances the shared counter, so the next valid `SEQ Figure` resolves to 2. It should retain OLD with the existing unclosed-quoting diagnostic and contribute no accepted sequence event. Validate producer instruction shape before snapshot mutation and before either facade or renderer consumes it. Add malformed producer controls followed by a valid increment to prove both fallback and counter isolation. Do not reapply the old one-argument validator unchanged, since the separately measured optional bookmark operand is valid.

## Smells

Zero smells.

## Nitpicks

Zero nitpicks.

## Not found

- **correctness**: Four defects above. No additional independently substantiated defect was found in copied note/comment closure, selected raw owner publication, sequence source identity, furniture placement or numbering delimiter handling.
- **contract**: D1 through D4 cover the remaining target, position and malformed-input gaps. No additional contract finding was substantiated.
- **panics**: Zero findings. Reviewed introduced indexing, slicing, checked arithmetic, staged publication and invariant-bound expects. No cited input-triggerable panic was established.
- **ooxml**: Zero findings. Reviewed raw complex/simple cache boundaries, namespace-qualified companion extraction, optional relationship closure, paragraph/durable identity remapping, selected/fallback ownership, whitespace preservation and child placement. No independent schema or unmodelled-XML loss finding was established.
- **tests**: D1 through D4 require missing branch discriminators. The existing named differential gate has a compiling semantic reversion that fails at runtime, and extensive owner fixtures exist. Neither substitutes for the cases identified above.
- **structure**: Zero findings. No new production module, crate, speculative trait, generic parameter or forwarding wrapper was introduced. The concrete shared snapshot has actual facade and layout consumers. The large implementation remains in the approved existing files.

The reviewed scoped log reports 122 shared-layout units, 500 facade units, 360 integrations, 839 regressions, 316 Word-layout units and 626 XML units passing, with explicit existing ignores. The worker records unchanged 49-entry deterministic hashes and verified package dry runs. Those receipts are compatible with the uncovered static branches and do not change this verdict. Integrated sprint verification remains the orchestration command's responsibility.

# F-279, implementation, pass 1

**Reviewed**: Frozen working diff on `work/f-279-codex`, based on `114ff99bc5f75c3bda1859f5f3c4d0cd5ba38a06`. Fifteen files, 4065 additions and 386 deletions. The diff SHA-256 is `e3b5f086234caa1426c316fb0af8213a8728b1e00cded63e3d4d4faa05091098`, matching `/private/tmp/f279-initial-review-diff.patch`.

**Verdict**: 8 defects, 0 smells, 0 nitpicks. Completion is blocked.

The approved plan, current progress record, workflow and microscope command were read before assessment. The cited scope, architecture, packaging, rendering, bindings and testing specifications were checked. This is an independent static review. No Cargo command, Word operation, implementation edit or remediation was performed. Previously reported checks are worker evidence, not new review executions.

## Defects

### D1, the approved rich text-box body is absent

`crates/rdocx-layout/src/engine.rs:8707`
`crates/rdocx-layout/src/engine.rs:506`
`crates/rdocx-oxml/src/drawing.rs:334`

Known contract blocker. Layout and registration still visit only `shape.text`, a vector of paragraphs. The approved `CT_Shape` rich-body rider requires an authoritative optional `CT_Body` projection, with table and modeled control traversal, source identity and mutation. A supported text box containing a table or block control cannot have its complete physical field inventory laid out and updated through this path. The paragraph-only fallback is required for compatibility but cannot substitute for the approved rich execution path.

### D2, continuous groups discard later members' pagination policy

`crates/rdocx-layout/src/paginator.rs:462`
`crates/rdocx-layout/src/paginator.rs:466`
`crates/rdocx-layout/src/paginator.rs:472`
`crates/rdocx-layout/src/paginator.rs:525`

Known correctness blocker. Matching continuous sections are concatenated and paginated with only the first member's page-number start, furniture, title-page state and footnote placement policy. Every member's snapshot then copies the resulting page's displayed number. When a later continuous section continues onto another physical page with different furniture or a page restart, that member's policy never enters pagination. Field ownership can name the later section while PAGE and PAGEREF values still come from the first member's numbering. Fresh Word evidence must determine shared-page boundary behavior, but there is no branch capable of applying the later member's policy on subsequent pages.

### D3, notes on a shared continuous page receive guessed section ownership

`crates/rdocx-layout/src/engine.rs:2998`
`crates/rdocx-layout/src/engine.rs:3005`
`crates/rdocx-layout/src/engine.rs:3011`
`crates/rdocx/src/field.rs:10610`

Known contract blocker. `owner_section` resolves document paragraphs and document-owned text boxes only. Footnotes, endnotes and related-story boxes fall back to the first section record for the page. On a page occupied by multiple continuous sections, that fallback is a guessed owner. The updater then writes SECTION and SECTIONPAGES from it and marks the result clean. The approved plan requires measured note ownership and retained caches with diagnostics when ownership is ambiguous. The pending continuous-note oracle cases must settle ownership before these fields are published as resolved.

### D4, cached-result children inherit unrelated outer-field pages

`crates/rdocx-layout/src/engine.rs:3030`
`crates/rdocx-layout/src/engine.rs:3039`
`crates/rdocx/src/field.rs:10579`
`crates/rdocx/tests/regression_test.rs:47807`

Additional correctness defect. Every outer field occurrence synthesizes placements for every descendant, copying the outer occurrence's page and section. Cached-result children have physical display positions and need their own placements. For example, an UNKNOWN complex field with cached text on page 1, a cached page break, and a nested PAGE on page 2 receives synthetic child placements on both pages. The updater selects page 1 and writes `1` into the page-2 child. A vanished child can similarly receive a placement from visible outer text. The current nested identity regression uses a single-page cache, so it cannot distinguish this implementation from correct child placement. Add simple and complex outer/child cases across page and column breaks, plus an invisible child whose cache must remain.

### D5, text-box cache staging can select a different physical owner

`crates/rdocx-layout/src/engine.rs:474`
`crates/rdocx/src/field.rs:1046`
`crates/rdocx/src/field.rs:1062`
`crates/rdocx/src/document.rs:8172`

Additional correctness defect. Layout's owner list contains only registered anchored typed shapes, while `StoryId::owner_index` counts the physical text-box owners found by story discovery. Sorting the filtered layout paths does not make those inventories equivalent. An earlier selected inline text box or other physical owner omitted by anchored registration shifts the ordinals. The earlier box then receives the later anchored box's placement, and the later box has no corresponding entry. The wrong cache can be written successfully because patching validates the selected paragraph's field source, not its correspondence to the layout owner. Join physical identities directly and retain unregistered owners without consuming another owner's placement. Include an omitted owner before a supported anchored owner and verify both source-specific caches.

### D6, nested simple-cache display segments omit child text

`crates/rdocx-oxml/src/text.rs:2769`
`crates/rdocx-oxml/src/text.rs:2820`
`crates/rdocx-oxml/src/text.rs:2826`
`crates/rdocx-oxml/src/text.rs:1009`

Additional correctness and OOXML projection defect. The initial simple-field pass records segments from direct result runs and skips nested simple fields. The second pass replaces `cached_result` and `original_cached_result` with the complete nested display but leaves the original `cached_segments` unchanged. `cached_display_segments` consequently returns incomplete segments for an unchanged outer simple field. An UNKNOWN simple field containing `prefix `, a nested simple PAGE cached as `OLD`, and ` suffix` exposes the correct aggregate `prefix OLD suffix` while layout paints only `prefix  suffix`. With complex cached children the original segments can also include instruction-run text rather than only the child's display. The ordered projection must supply complete display segments with child properties. Assert actual segment text and deterministic rendered text before and after edits for all four outer/child wire-form combinations.

### D7, a later continuous member's section-end notes are skipped

`crates/rdocx-layout/src/paginator.rs:457`
`crates/rdocx-layout/src/paginator.rs:482`
`crates/rdocx-layout/src/engine.rs:2884`

Additional correctness defect, distinct from furniture selection. Grouping prevents a merge after a member with section-end notes, but permits that member to become the last member of a group. Placement checks only the first member's `endnotes_at_section_end`. If section 1 has document-end notes and continuous section 2 requests section-end notes, the combined group skips section 2's placement. With a following section, those notes can be deferred to the document end rather than section 2's end. Preserve each member's note boundary when forming and placing continuous groups. Cover a section-end final member followed by another section, including references in both members.

### D8, section number-format lookup excludes section-ending control paragraphs

`crates/rdocx/src/field.rs:10708`
`crates/rdocx/src/field.rs:10719`
`crates/rdocx-layout/src/engine.rs:2317`
`crates/rdocx-layout/src/engine.rs:2438`

Additional correctness defect. Layout expands modeled body controls into paragraph and table items and recognizes a section-ending paragraph there. `section_page_format` separately enumerates only direct body paragraphs and the final body section. Their section indices therefore disagree when a modeled control owns a section-ending paragraph. A control-owned Roman section followed by a decimal final section gets the final section's decimal format at index zero, and missing indices silently become decimal. PAGE and target-owned PAGEREF caches are formatted with the wrong policy. Resolve format from the same section inventory that produced placement, rather than a narrower second traversal. Add a control-owned section break with distinct numbering formats and cross-section PAGEREF.

## Smells

None found.

## Nitpicks

None found.

## Completion evidence still required

`crates/rdocx/tests/regression_test.rs:49165` currently moves one existing field paragraph into a one-row, one-cell related table, and another into a control. `crates/rdocx/tests/regression_test.rs:49230` checks path lengths. This establishes bounded source identity, but does not establish larger-row wrapping, exact-height clipping, multirow continuation or multipage note-table flow. These acknowledged rich-story rider cases remain required.

`crates/rdocx/tests/regression_test.rs:48486` replays six authenticated cache cases. The continuous case is bounded to the previously captured simple shared page and does not settle D2 or D3. `crates/rdocx/tests/regression_test.rs:49129` provides the 252-field format matrix. Neither substitutes for the pending continuous furniture, restart and note ownership evidence.

The approved HLD work list remains unexecuted at `.claude/plans/F-279-design.md:187`. Existing `docs/hld/08-rendering-spec.md:1872` still specifies top-level-only field identity and `docs/hld/10-bindings-spec.md:1723` still describes three report counts. Final scoped checks, archive and binding riders, final hash evidence after the default document-end note behavior change, HLD completion and another independent microscope pass remain completion barriers. These are not assertions that the worker has claimed completion.

## Not found

- Panics: no additional independently established panic defect in the inspected changed paths. No runtime panic or hostile-input campaign was executed.
- Structure: no new trait, generic parameter, crate, module, feature flag or source file. The source changes stay within approved file claims.
- Atomic publication: the public updater stages a candidate, applies body and related-story patches to it, and publishes only after successful completion. No additional receiver-publication defect was found.
- Bindings: the Python report adds defaulted constructor counters and both getters while retaining the existing constructor arguments. No additional binding compatibility defect was found by static inspection.
- OOXML outside D6: no independently established additional child-order or opaque-source preservation defect. Related-story edits use checked physical field spans and stage parsed replacement parts.

The frozen diff hash was rechecked immediately before writing this review and still matched the recorded scope. Review ends here. Remediation belongs to a separate implementation phase.

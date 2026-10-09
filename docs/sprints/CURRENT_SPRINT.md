# Current Sprint, S90

**Milestone**: M24, modern DOCX authoring completeness.

**Goal**: provide a complete field construction surface and deterministic
field results across stories, then compose captions, cross-references, indexes,
bounded native citations and bibliography results. Build on S89's note and range
contracts while preserving producer XML and saved field caches.
Issue264 is excluded and Issue281 remains open. F-X178 is carried for separate work. F-282 covers
the measured native source-authoring and formatter subset. F-X192 owns the
remaining full catalogue work in S91. F-283 is carried to S91.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for DOCX-045 through DOCX-050 and the remaining field and navigation capability owners.
- `docs/hld/03-architecture.md`, for recursive field grammar, immutable evaluation, physical story ownership and the pagination boundary.
- `docs/hld/04-opc-and-packaging.md`, for relationship closure and preservation when authoring fields and bibliography parts.
- `docs/hld/08-rendering-spec.md`, for deterministic pagination, page targets and field result placement.
- `docs/hld/12-testing-strategy.md`, for round-trip, regression and pinned Word differential gates.
- `docs/hld/14-development-backlog.md`, F-278 through F-283, for the six contracts, dependencies and test gates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-278 | General simple and complex field builder | L | done | - |
| F-279 | Pagination field materialization across stories | L | done | - |
| F-280 | Captions, sequences, and complete cross-references | M | done | - |
| F-281 | Indexes and tables of figures and authorities | L | done | - |
| F-282 | Citations and bibliography authoring | L | done | - |
| F-283 | Complete numbering-aware navigation fields | L | pending | - |
| F-X178 | Clearable direct run formatting setters | S | pending | - |
| F-X179 | Correct multi-paragraph comment threads from PR 271 | S | done | - |
| F-X180 | Correct cell nil and none border precedence | S | done | - |
| F-X181 | Ignore page and column breaks inside table cells | S | done | - |
| F-X182 | Honor direct table alignment | M | done | - |
| F-X183 | Correct table margins and legacy positioning | M | done | - |
| F-X184 | Safe comment ownership during content removal | L | done | - |
| F-X185 | Expose comment anchor text and story location | M | done | - |
| F-X186 | Move comment anchors without losing threads | M | done | - |
| F-X187 | Scoped paragraph and cell text replacement | M | done | - |
| F-X188 | Preserve comment ownership when replacing or removing whole stories | L | done | - |
| F-X190 | Preserve cached text in multi-run complex field story snapshots | M | done | - |
| F-X191 | Ignore namespace declarations in numbering reader completeness | S | done | - |
| F-X189 | Prepare Word 0.16.0 and PowerPoint 0.14.0 families | L | done | - |

## Sequencing note

Rows are listed in dependency order, not implementation waves. F-278 supplies
the field substrate. F-279 consumes the completed S89 note policy and F-280
consumes S89 range markers. Both depend on F-278. F-281 follows F-278 through
F-280, while F-282 needs only F-278. F-283 follows F-279 through F-282 and the
completed numbering foundation in S91. Shared source and test files determine which
otherwise independent stories can run together after design.
F-X178 remains pending in the backlog and carried in the S90 run state.
It has no implementation wave in this sprint, honoring the Issue 264 exclusion.

F-X179 adopts PR 271 against all of Issue 270, including multiline text and
last-paragraph thread metadata. It has no feature dependency. Issue 264 and
F-X178 remain outside this intake. GitHub closure waits for sprint close.

The user added Issues 272 and 273 with PRs 274 and 275 during S90.
F-X180 and F-X181 validate each full issue before accepting its contribution.
Their source and regression files overlap F-282 and F-283. Pause F-282 at a
saved checkpoint, run these independent fixes in separate waves, then resume
the bounded F-282 contract. F-283 is carried to S91. Source and Cargo ownership never overlap. Reconcile both
approved contracts at integration and check table geometry and bibliography
output. Issue 264 remains excluded.


Newly opened Issues 277 and 276 add F-X182 and F-X183. Run direct alignment
before legacy positioning, with F-X182 completed at a scoped dependency
checkpoint before F-X183 starts. Keep F-282 paused through these exclusive
source and hash-baseline waves, then resume its bounded S90 contract. F-283 is carried to S91. Each rendering
delta is separately declared and reviewed. Issue 264 remains excluded.
PR 279 supplies F-X182. PR 280 is stacked on it and supplies F-X183, including new Issue 278 for unset side margins. Validate both complete issue sets before contributor disposition.

Issue [281](https://github.com/tensorbee/rdocx/issues/281), raised by `hadim`,
adds roadmap feedback to this sprint intake. Record its unified CLI/Python
workflow, template filling, preservation-safe edits and rendering needs in the
existing F-184 decision at S95. The [maintainer response](https://github.com/tensorbee/rdocx/issues/281#issuecomment-6058792996)
answers scheduling and staged-delivery questions and requests concrete workflows
and preservation needs. Keep the discussion open for the F-184 decision. S90 delivers the documented
assessment, while spreadsheet implementation retains its affirmative decision
barrier.

The [reporter follow-up](https://github.com/tensorbee/rdocx/issues/281#issuecomment-6063901190)
adds concrete Google Sheets review requirements: byte-preserve unmodeled
comment metadata, validation, conditional formatting and names, read typed
cells and formula caches, edit with styles and atomic output, and keep shared
formulas intact. Explicit stale-cache handling is required. The first-cut
alternatives are recalculate-on-load or a bounded basic evaluator. Unified
installation and API shape, Rust performance and safety, and later rendering
are the differentiators. Chart authoring is not requested. HLD14 and the S95
plan retain these inputs without starting conditional spreadsheet work in S90
or changing the reviewed release boundary. Keep Issue281 open for F-184.

The [maintainer commitment](https://github.com/tensorbee/rdocx/issues/281#issuecomment-6064533994)
confirms preservation tests and the review workflow first, then reading and
styled edits with explicit stale-cache handling. A bounded evaluator and
rendering follow, with chart authoring deferred. Pivot refresh, Power Query
and scripting remain longer-term goals. F-184 must reconcile the scheduled
stories and any proposed reader/editor distribution boundary with that order
before approving implementation. S90 records this direction without claiming
that the conditional spreadsheet programme or an earlier release is approved.

Issues [282](https://github.com/tensorbee/rdocx/issues/282), [283](https://github.com/tensorbee/rdocx/issues/283), [284](https://github.com/tensorbee/rdocx/issues/284) and [285](https://github.com/tensorbee/rdocx/issues/285), reported by `hadim`, add F-X184 through F-X187. The intake was read against canonical 0a775842a8fd12f088bf4f2b3d0a049ddc3e976b. All four were opened on 2026-10-08 at 13:35 UTC and have no matching contribution PR at intake. Subsequent [PR 286](https://github.com/tensorbee/rdocx/pull/286), by `hadim`, targets Issue 285 at head `08361f6af99110cbae71bf9f0498a12feadc01db` and is included in F-X187 contribution assessment. Its patch must satisfy the approved scope and integrated verification before acceptance or closure. Their approved plans retain the complete issue criteria. Issue numbers 282 and 283 are distinct from existing feature IDs F-282 and F-283.

Pause F-282 only at an explicit saved external checkpoint. Run F-X184, F-X185, F-X186 and F-X187 in exclusive waves 11, 12, 13 and 14, respectively. F-X184 uses completed F-X179 and F-271. Complete each formal dependency before starting its consumer. F-X187 has no formal dependency but follows F-X186 because document, comment, binding and existing test entrypoints overlap. These waves exclusively own shared source, Cargo execution and HLD edits. Resume the revised measured-subset F-282 contract afterward. Remaining bibliography work is F-X192 in S91, and F-283 is carried to S91. Draft batch planning does not waive implementation barriers or the final integrated full gate. Issue 264 and F-X178 remain excluded. GitHub closure waits for verified sprint close and complete issue acceptance.



Subsequent [PR 287](https://github.com/tensorbee/rdocx/pull/287), by Hadrien
Mary (`hadim`), targets Issues 282, 283 and 284 at immutable head
`e4b216934eb3abc58ea1a42d72a902f90aa8e120`, based on
`20888b7a2c636e322ad23dc611f494beaac1c09b`. It is included in F-X184 through
F-X186 contribution assessment. Existing F-X184 acceptance retains atomic
partial-cut and fragment-detach refusal. Assess reusable anchor projection,
typed binding and movement changes against the full F-X185 and F-X186 plans,
without replacing the completed ownership protections or accepting the PR's
stated note and header removal gaps. The stacked F-X179 contribution is
already integrated. Read the exact contribution and preserve contributor
attribution, with PR reconciliation only after complete integrated acceptance
at verified sprint close. No additional F-ID or scope reduction is introduced.

Issues [288](https://github.com/tensorbee/rdocx/issues/288) and
[289](https://github.com/tensorbee/rdocx/issues/289), reported by `hadim` on
2026-10-08 at 16:18 UTC, are included in S90 acceptance assessment. Issue288
covers note removal, all header and footer replacement variants, shared story
references and building-block removal with complete comment thread and
companion preservation or atomic refusal. Compare every criterion with the
completed F-X184 implementation and add any missing implementation and tests
through the feature lifecycle before acceptance. Issue289 requires nested
block-control paragraph snapshots to carry accepted text and XML, including
comment anchor endpoints and other checked two-segment path consumers. Assess
it under F-X185's existing checked paragraph snapshot contract. Nested
paragraph enumeration is optional in that issue and does not change the
ordinary story inventory contract. Neither issue is closed from similarity to
an existing fix. Exact integrated evidence and full issue acceptance remain
required. The seven observed PR heads remain unchanged. Issue264 remains
excluded.

F-X188 owns exclusive wave15 after F-X187 and before F-282
resumes under its revised measured-subset contract. It completes every Issue288 criterion with the approved new design,
retaining completed F-X184 protections. Issue289 is part of F-X185 acceptance.

The user approved F-X189 to prepare Word0.16.0 and shared/PowerPoint0.14.0
after every included implementation story completes. Both prior unified
versions are already published at S88. Exclusive wave18 prepares exact
carriers, reviewed family notes, contributor credit and fresh package/binding
evidence. The final reviewed sprint SHA must pass the hosted build-only
rehearsal before close. Publication follows close through separate final
exact-main-SHA approvals, shared family first where Word pins require it.
Issue264 remains excluded and Issue281 remains open for its roadmap decision.

Issue [292](https://github.com/tensorbee/rdocx/issues/292), reported by
`hadim`, extends F-X188's building-block ownership contract to creation,
fragment update, extraction and insertion with isolated comments ownership.
PR290 and updated PR287 are assessed contributions, not automatic acceptance.
The user corrected the exclusion: Issue [281](https://github.com/tensorbee/rdocx/issues/281)
remains open for F-184, while Issue [291](https://github.com/tensorbee/rdocx/issues/291)
is included. Plan and verify its multi-run complex-field snapshot correction
through F-X190 in exclusive wave16 after F-X188 and before resuming F-282. Updated PR287 and PR290
stacks require selective acceptance against each issue. Issue264 remains
excluded. All source, Cargo and shared test execution remains exclusive.

S90 contribution intake is frozen by the user at PR293. Do not search for
or add later issues or pull requests to this sprint. Finish the existing
measured bibliography, numbering reader and release preparation scope.
Remaining catalogue work and unfinished navigation move to S91.
Issues264 and281 remain open.

PR [293](https://github.com/tensorbee/rdocx/pull/293), contributed by Pedro
Assumpcao (`pedroassumpcao`) at head
`fd112a7ac3333709f62746b7065c5cc1b4be0eed`, adds F-X191. The user approved
its workflow records. Exclusive wave17 corrects the namespace-only numbering
reader flag while preserving declarations, foreign attributes, opaque children
and the separate authoring completeness contract. It has no formal dependency,
but shared source and Cargo ownership remain exclusive. Complete F-X191 before
F-X189 in wave18. Assess only the reader fix and focused tests, preserving
current archive policy rather than adopting contributor platform overrides.
Issue264 remains excluded and Issue281 stays open. PR disposition waits for
complete integrated acceptance at verified sprint close.

The user moved remaining bibliography work to F-X192 in S91 and explicitly
carried F-283 to S91 to publish completed S90 work. F-282 must satisfy its
revised measured-subset design, actual admission and atomic refusal contracts,
independent review and scoped gates. Its current 210-of-223 dense APA checkpoint
is not full catalogue parity. F-X189 depends on completed revised F-282 and
F-X191, not the carried S91 stories. Final integrated verification, sprint
review, package checks and exact-SHA publication approval remain required.

## Definition of done for this sprint

- Simple, complex and nested fields reopen with identical instruction semantics and ordered cached content.
- Pagination field caches across body, headers, footers, notes and text boxes match the pinned Word page and section values.
- Captions and cross-references match Word before and after insertion and renumbering.
- Index, figure and authority tables retain the ordered entries and page targets produced by a pinned Word update.
- Admitted citation and bibliography controls match pinned Word identifiers, ordering, rich display and package round trips. Unfinished standard refresh errors are atomic, while noncatalogue retention reports preserve caches. Full catalogue parity is deferred to F-X192.
- The integrated full verification and sprint review pass over delivered scope, including source-built field caches and page targets. Any intentional hash delta is declared and reviewed.

- Cell border precedence and table-cell break handling satisfy Issues 272 and 273 against the pinned Word oracle.

- Direct table alignment, absent cell margins and compatibility-mode positioning satisfy Issues 277, 278 and 276 against pinned Word controls.

- Issue 281 has a documented roadmap assessment and a disposition linked to F-184.

- Issues 282 through 285 satisfy comment ownership, typed anchor discovery, identity-preserving moves and scoped replacement criteria with atomic refusal and complete source preservation controls.

- Issues288 and289 satisfy whole-story comment closure and complete checked nested paragraph snapshots.

- Issue292 satisfies documented building-block comment isolation without unanchored main comments. Issue291 bulk field snapshots match direct field text across supported owners and namespace contexts. Issues264 and281 remain open.

- Both approved new release families have exact versions, reviewed notes and contributor inventories, verified packages and installed Python evidence, plus a current reviewed-SHA hosted build-only rehearsal before close. Publication remains separately approved after close.

- PR293 namespace-only numbering inspection reports modeled levels correctly while preserving real unmodeled content and saved XML.

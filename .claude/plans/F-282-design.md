# F-282, Citations and bibliography authoring

**Status**: completed
**Sprint**: S90
**Size**: L
**Depends on**: F-278

## Problem

The native facade needs source authoring and staged citation and bibliography
cache updates without discarding producer XML. The user originally selected
full Word catalogue parity, then explicitly moved its unfinished portion to
F-X192 in S91 so completed S90 work can publish. F-283 is also carried to S91.
This revised contract delivers the current measured native foundation and
formatter subset. It does not claim full catalogue parity.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, DOCX-049 capability boundary.
- `docs/hld/03-architecture.md`, "What stays put", recursive field grammar, typed stories and staged cache updates.
- `docs/hld/04-opc-and-packaging.md`, "Relationship types" and "Package integrity", custom XML ownership and preservation.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability", published native Rust surface and distinct binding scope.
- `docs/hld/12-testing-strategy.md`, "The Word corpus", source-built differential evidence and deterministic harness policy.
- `docs/hld/14-development-backlog.md`, F-282 measured delivery and F-X192 remaining catalogue contract.
- `docs/hld/15-build-and-toolchain.md`, "Packaging" and "Publishing", current archive and package gates.

## Approach

Deliver the reviewed current native Rust source model, source inspection and
CRUD, style/options, field insertion and staged update APIs in the explicitly
approved bibliography.rs module. Preserve all seventeen source kinds, sixteen
contributor roles, property members and twelve pinned style identities as
source-authoring metadata. Metadata coverage is distinct from formatting
coverage. Do not claim dedicated bibliography Python or WASM APIs where the
actual wrappers do not expose them.

Formatting remains bounded by the actual admission predicates and measured
consumer operations. The frozen checkpoint has 210 of 223 dense APA locale
grammars and 1340 passing APA tests, with no skips or ignored tests. Those
counts describe earned configurations rather than arbitrary sparse, plural,
corporate or Unicode combinations. Retain every already earned source-script,
sparse/date, repeated-property and source-kind control. The remaining thirteen
selectors and broader catalogue dimensions belong to F-X192. Do not implement
new locale/style consumers merely to complete S90.

The eleven non-APA bibliography branches have concrete lean Book materializers,
currently admitted at numeric1033 with Author-only contributors and a small
Title/ShortTitle/Year/City/Publisher/ReferenceOrder property set. Their additional
materializer boundary requires one personal contributor and nonempty ASCII
Last/First/Title/Year/City/Publisher values. Review exact accepted grammar and
prove its intended generalization or add an honest atomic restriction through
normal remediation. Exported style identity does not imply full style support.
Citation consumers have separately earned locale/kind and modifier evidence.
Do not transfer bibliography assertions to contextual citation results.

Choose the existing fail-atomic incomplete-standard policy for this release.
A recognized unfinished branch returns an explicit error and aborts the whole
staged refresh, including earlier candidate edits. It is not partial refresh
success. Noncatalogue/unrecognized paths retain the owner cache with report
diagnostics. Locked and protected producer topologies have their own existing
retained/error boundaries. Document each policy, test mixed eligible/unfinished
transactions and prove complete byte and revision preservation on refusal.
Do not convert errors to retention reports without a separately reviewed
behavioral change and actual regression evidence.

The source and application-default locale resolver selects actual existing
consumers. It does not provide a guessed en-US fallback. Preserve raw source
LCIDs, field selector spelling and standard source-root style metadata.
Non-ASCII collection sort keys still refuse rather than approximate Word.
Do not advertise arbitrary Unicode bibliography ordering. The unresolved Word
comparator and any ICU dependency/data implementation move to F-X192. No new
collation dependency is required or authorized by this bounded delivery.

Retain recursive instruction grammar, owned l/f/m token replacement and exact
unowned switch spans. Source mutations preserve namespace scope, foreign
lookalikes, opaque children, repeated producer members and unrelated package
parts. No-op replacement is byte-identical. Duplicate or ambiguous identities,
wrong graph edges, referenced deletion, stale paths and malformed particles
fail without partial publication. Equivalent XML namespace prefixes produce
equal library results while retaining imported raw XML and documenting Word's
measured prefix-sensitive discrepancy.

Structured bibliography caches retain actual paragraph/table ownership,
IEEE grid/cell/label structure, rich CT_R properties and physical begin/separate/
end boundaries. Preserve first/interior paragraph formatting and outside
content under the measured ownership rules. Keep actual final raw oracle
partitions. Only already documented exact pagination comparison pairs may be
projected, with complete raw expectations and guards retained. No generic
coalescing, property stripping or font coverage heuristic. Historical recovered
caches without completed F9 provenance remain separate preservation evidence.
Normal save leaves published caches alone.

The existing layout font resolver's public visibility change retains its
algorithm and requires effective already-cascaded properties. Its bibliography
caller and existing layout callers share that concrete implementation. No new
trait, generic, wrapper, runtime flag, crate or production file is introduced.

## Rejected alternatives

- Completing full catalogue parity in S90 contradicts the user's explicit deferral.
- Calling source/style enum coverage complete formatter support conceals admission boundaries.
- Silently falling back to en-US or approximate collation fabricates results.
- Treating unfinished standard errors as retained reports misstates atomic behavior.
- Dropping existing tests or weakening rich expectations to publish destroys earned evidence.
- Copying, executing or redistributing proprietary Word XSL is outside this implementation.

## Test plan

**Test gate**: differential.
`bibliography_apa_rich_gurmukhi_gujarati_odia_1094_full_seventeen_kind_owner_matches_word`
is the named executable gate in the existing regression entrypoint. Public source-built
admitted citation and bibliography controls match pinned Word identities,
ordering, readable text, rich output and source/package round trips. All1340
existing `bibliography_apa_` controls, including
`bibliography_apa_mixed_seventeen_kind_collection_matches_actual_word`, and the
admitted non-APA controls remain
mandatory supporting evidence. Full catalogue parity is F-X192's gate, not a
renamed or weakened S90 assertion. The historical start-contract smoke test
`full_bibliography_catalogue_matches_pinned_word` checks source authoring only
and does not certify formatter catalogue parity or this release boundary.

| Category | Gate | Asserts |
|---|---|---|
| differential | current measured APA locale/source controls | All existing 1340 APA tests plus their exact current source-bound oracles, including full17 owners and source scripts. No unilateral skips or ignored obligations |
| differential | admitted non-APA Book and citation controls | Actual style-specific grammar, selection, numeric assignment, modifiers and rich structure against independently earned outputs |
| regression | `bibliography_unfinished_later_owner_rolls_back_eligible_refresh` | Whole update errors atomically, preserving every cache, source part, package byte and document revision |
| regression | noncatalogue and protected owner handling | Retained owner/report diagnostics remain distinct from recognized unfinished errors |
| round-trip | source metadata and physical cache preservation | Qualified XML, unmodeled data, IDs, relationships, first/interior boundaries, historical raw cache domains and no-op source replacement |
| integration | native public source and field transactions | Create/read/replace/remove/options/insertion/update across supported physical stories, explicit defaults, dangling references and atomic refusal |
| regression | scanner and structured-result compatibility | Existing F-X188 comment ownership, F-X190 namespace-lifetime field snapshots, ordinary field grammar and raw comment/PI preservation remain valid |
| unit | namespace/schema/font consumer controls | Expanded qualified identity, schema particle order and unchanged concrete font-slot/theme behavior |

Primary oracle is Microsoft Word for Mac16.113.2 build16.113.26092012. Preserve
actual style, LCID, input fingerprints, UI update/save/reopen receipts and raw
qualified records. Native phase normalization is qualified rather than erased.
No PDF or full renderer parity is inferred from cache text/properties.

Stable development receipt is
`/private/tmp/S90-F282-resume-20261009/rich-gurmukhi-gujarati-odia-checkpoint-receipt.json`,
SHA2563bd0f8be5be9c81c959c1fdefeae5e35a4d940b193cfeb57490b8569e58929a3.
The named1094 gate genuinely fails in that receipt's compiled before log
against retained formatter source e821236591d914fa149034de5b65ef2d1e6ccfe5ba90d792870d2131da481ee4
and retained compiled test source257f28b1c14e932ce9694875c36d2bd95fa161d853596102f7240d7af6847e04.
That qualified earlier formatter checkpoint already exposes the native APIs,
so this is a runtime grammar refusal, not a missing-API compilation failure
or a claimed test of immutable claim Base. The current1340 suite includes the
same exact named gate and must pass on the reconciled delivery prefix.
Its 46 bindings and the authored-source backup manifest
c5bcdb8c341a460bad86264eed393f057948ff56b55549cfed8456e4d9a871a8 were
independently checked. These are development receipts, not the final scoped or
publication gate. Authenticate source after F-X191 integration, reconcile both
approved plans and rerun affected checks. Extend existing test entrypoints,
without a new binary or binary fixture. Run scoped native tests, actual-diff
risk riders and independent zero-finding ALL microscope before preparation.
Full integrated verification and sprint review follow the final merged result.

## Exact finite pagination comparison guards

The exhaustive original APA223 full-bibliography scan binds a finite registry
of seventeen adjacent equal-property page splits. It includes the two cases
above and the additional measured entries below. Registry index SHA256
137cbb3844944b518968aca0f4c7d9671030d9a0aa2c426eec5114745ae4e156 and
raw registry SHA256
27c33a3d3dcee2b13e8fa36555cc7e8033c4ae5e9a62828f994e4f596b44dca7 in
/private/tmp/S90-F282-APA6-full223-pagebreak-marker-registry retain actual
source tags, owner and body paths, exact raw run properties, attributes,
xml:space and PDF anchors. All seventeen have an independently verified actual
page transition, identical adjacent run properties and attributes, and the
specific child shapes [rPr,t] and [rPr,lastRenderedPageBreak,t].

| Numeric locale | Full owner | Source tag | Body block | Actual PDF pages |
|---|---|---|---|---|
| 1033 | 161 | F282K15 | 341 | 18 to 19 |
| 1058 | 611 | F282K03 | 1310 | 70 to 71 |
| 1063 | 701 | F282K15 | 1511 | 81 to 82 |
| 1071 | 845 | F282K15 | 1823 | 98 to 99 |
| 1076 | 935 | F282K03 | 2012 | 108 to 109 |
| 1079 | 989 | F282K17 | 2136 | 115 to 116 |
| 1099 | 1349 | F282K15 | 2915 | 161 to 162 |
| 1100 | 1367 | F282K12 | 2960 | 164 to 165 |
| 1101 | 1385 | F282K17 | 2994 | 166 to 167 |
| 1113 | 1601 | F282K03 | 3455 | 194 to 195 |
| 1115 | 1637 | F282K15 | 3539 | 199 to 200 |
| 1118 | 1691 | F282K03 | 3650 | 205 to 206 |
| 1143 | 2087 | F282K15 | 4514 | 252 to 253 |
| 2115 | 2501 | F282K15 | 5411 | 301 to 302 |
| 2128 | 2555 | F282K15 | 5528 | 308 to 309 |
| 2163 | 2681 | F282K15 | 5801 | 324 to 325 |
| 20490 | 3959 | F282K03 | 8564 | 473 to 474 |

Only these registry-bound comparisons may remove the identified page-cache
marker and join those two adjacent runs after checking their exact raw shape.
Keep immutable raw arrays beside each comparison projection, preserve exact
spaces and assert actual paragraph context separately. This is a finite
extension of the two qualified comparisons above. The other 316 markers,
including 17 empty owned paragraphs, acquire no new qualification. The 75
original long-window PDF nonmatches remain explicit, with separate exact short
source-context anchors supplying proof only for four identified split cases.
Do not apply a global marker or coalescing rule, infer locale aliases, alter
imported caches, synthesize formatter markers or claim Rust pagination parity.
No new runtime file, type, dependency or fixture binary is needed.

Only the new1125 F282K03 split in completed paragraph22, runs14 and15, earns
an additional finite comparison projection. Raw proof2dbdb96e binds exact
text Cambridge, England, followed by United Kingdom: Publisher03, equal raw
rPr and empty run attributes, child shapes [rPr,t] and
[rPr,lastRenderedPageBreak,t], preserved trailing space on the first span,
no xml:space on the second, and cumulative character offset191. Retain both
raw runs and the marker before joining this pair for comparison. Assert the
unprojected paragraph properties independently. This extends only the finite
comparison registry, with no formatter pagination marker or general coalescing.

The finite comparison registry additionally admits only own1093
BookSection K03, completed document722dcb0f paragraph22, runs30 and31.
Root raw proof4fd27d26 and worker proof0d79f467 bind the actual marker.
Before joining the comparison pair, assert 38 raw runs, empty run
attributes, exact text `পৃ. ` and `21-29). `, equal Vrinda font properties,
hintcs, noProof, cs, own bn-IN language, no RTL, the exact child shapes
with one actual lastRenderedPageBreak and character offset172. Retain
raw expected XML and full17 unprojected paragraph properties. No other
pair is admitted and no formatter marker is synthesized. This earns no
PDF page-transition or renderer parity claim.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/15-build-and-toolchain.md`

Describe current native authoring, formatting admissions, staged ownership and
refusal behavior. Keep DOCX-049 partial with F-X192 owning the remaining
catalogue boundary. Do not claim new dedicated bindings or unimplemented
collation dependencies. Preserve published font-resolver input requirements.

## Risk routing

- Parser/serializer: packaging and PresentationML preservation/schema rules, exact qualified and opaque XML round trips, scanner compatibility.
- Published native API: additive pre-1.0 source/field types and font resolver visibility, public Rust docs, current package dry runs and below10MiB archive assertions.
- Shared layout font API: existing concrete slot/theme regressions, effective-property caller checks and unchanged deterministic hash harness.
- External oracle: pinned Word build, exact source/rich provenance and qualified native lifecycle differences.
- Compatibility: existing Python runtime/typing and WASM checks over changed shared Rust behavior, without claiming dedicated bibliography wrappers.
- New file/module: bibliography.rs and workflow records were explicitly user-approved. No additional production file, trait, generic, crate, feature flag or dependency.

The future-only collation dependency/data rider is deferred with F-X192.
Every rider earned by the actual shipped diff still runs. Release family
preparation and publication retain their independent gates and SHA approval.

## Hash harness

Expected unchanged. Existing samples do not invoke bibliography source authoring
or updates. All49 entries must match. No baseline movement is allocated.

## Implementation checklist

- [x] Complete F-278 foundation and preserve the current authored checkpoint.
- [x] Record explicit user deferral to F-X192 and carry F-283 to S91.
- [x] Reconcile the F-X191 integrated prefix and revised plan into the worker.
- [x] Review actual admissions and document the precise bounded public behavior.
- [x] Remediate only review/gate defects under this contract, retaining existing assertions.
- [x] Update all seven named HLD targets to current intent.
- [x] Pass current scoped/differential/compatibility/package/hash riders.
- [x] Earn independent zero-finding ALL microscope and validated handoff.

## Open questions

None blocking. The user explicitly requested the remaining F-282 story in S91,
confirmed F-283 carry and asked for parallel agents to finish S90 faster.
New GitHub intake is frozen at PR293. Issues264 and281 remain open.

# F-X191, Ignore namespace declarations in numbering reader completeness

**Status**: completed
**Sprint**: S90
**Size**: S
**Depends on**: none

## Problem

PR293 reports a supported native numbering level as unmodeled when its only
retained extras are namespace declarations. Canonical
`crates/rdocx/src/document.rs:20549` combines extra XML and every retained
attribute on the instance, abstract definition and level. The model correctly
retains namespace declarations at `crates/rdocx-oxml/src/numbering.rs:564`.
Declarations provide XML scope and are not unsupported numbering semantics.

The user explicitly authorized the design, review, progress and handoff
records. The story is registered before approval. Review and handoff records
are written only when their lifecycle steps have earned them.

## Spec reference

- `docs/hld/10-bindings-spec.md`, "Two supporting decisions", native Word reader projections and unmodeled-content facts at lines109 through118.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability", native numbering projections and the separate checked authoring surface at lines818 through864.
- `docs/hld/04-opc-and-packaging.md`, "The package", numbering graph transactions and retained producer parts at lines521 through538.
- `docs/hld/14-development-backlog.md`, "F-X191, Ignore namespace declarations in numbering reader completeness (S)", registered regression gate.

## Approach

Correct only `Document::numbering_level`'s reader boolean. Keep all three
existing `extra_xml` predicates. Chain the three existing
`extra_attributes` collections and report a real extra attribute when its
lexical name is neither exactly `xmlns` nor starts with `xmlns:`. Never strip
namespace declarations or infer modeled semantics from a namespace URI.
Keep same-local foreign attributes such as `producer:xmlns` and `xmlnsLike`
flagged. Keep a Word-looking alias rebound to a foreign namespace flagged.

No new public signature, type, trait, generic, module, source file or
serialization helper is needed. Update the existing method documentation to
state namespace declaration retention separately from this reader fact.
Extend tests in the existing `document.rs` unit-test module, reusing the
existing numbering XML replacement fixture. Preserve modeled metadata and raw
pPr/rPr or typed-leaf overlays excluded by the existing reader contract.
Do not change `numbering_level_has_unmodeled`, instance/definition authoring
completeness, raw overlays, parser storage or writer output.

This is an intentional public pre-1.0 behavioral correction with no new API
or binding surface. Label its implementation commit and expected boolean
change separately from any package evidence updates. It must not alter
rendered output, numbering values, IDs, namespace qualification or the hash
baseline.

## Intake and sequencing

Read-only intake SHA81b115184bb7903ef41ba759073e9719ef9b879aa75f24fb0b09760af431994b
binds the production PR review and latest-head supplement. Latest PR head is
fd112a7ac3333709f62746b7065c5cc1b4be0eed. Its document.rs142 additions and
3 deletions remain the same native reader fix and tests as the original head.
Credit Pedro Assumpcao, username pedroassumpcao, and PR293 in contribution and
release records. Upstream acceptance claims are not local compiled evidence.

Canonical base31524cc6 is a planning observation, not the future claim Base.
Claim this separate story from the actual canonical tip in exclusive wave17
after the active F282 writer has an explicit safe checkpoint.
There is no formal feature prerequisite. Scheduling protects shared document
and test ownership without introducing a silent F283 dependency or scope
expansion. Complete and integrate F-X191 before F-X189 starts. F-X191 is a formal F-X189 prerequisite in the story, release plan and shared
workflow records. F-X189 is scheduled in exclusive wave18. Issue264 stays excluded and Issue281 stays open.

Do not adopt PR293 README stale sizes, date, M3 platform override or the
ARCHIVE_MEASUREMENT_PLATFORMS script change. Reconcile only actual current
archives from final local source on the real measured host through existing
canonical archive measurement and validation procedures. Preserve all
numerical, member-inventory and compression-tolerance policies.

## Rejected alternatives

- Dropping declarations from extra_attributes damages namespace and opaque XML preservation.
- Ignoring every attribute or every name containing xmlns hides real producer semantics.
- Broadening the reader flag to raw property overlays changes an established independent contract.
- Altering richer authoring completeness helpers confuses reporting with mutation admission.
- Folding this change into F283 widens an approved unrelated story.
- Importing contributor archive observations provides stale package evidence and an unnecessary platform mechanism.

## Test plan

**Test gate**: regression.
`numbering_level_namespace_declarations_do_not_report_unmodeled_properties`
must compile and actually fail on the exact claimed Base through the existing
API, then pass the repaired implementation. A compile failure or test not
executed is not genuine before evidence.

| Category | Test | Asserts |
|---|---|---|
| regression | `numbering_level_namespace_declarations_do_not_report_unmodeled_properties` | Three owners times four declaration forms: repeated Word prefix, arbitrary Word alias, default producer namespace and unused producer prefix. Reader false before and after save/reopen, namespace metadata retained |
| preservation | declaration storage and saved namespace controls | Instance, definition and level retained collections remain populated, saved XML resolves declarations, Bullet/start/suffix/level facts unchanged |
| regression | `numbering_level_unknown_attributes_and_xml_remain_unmodeled_after_reopen` | Real producer:flag, producer:xmlns and xmlnsLike attributes stay flagged independently of raw children, all three owners tested, attributes survive save/reopen |
| namespace | foreign rebound Word-looking alias control | Retained foreign namespace attributes remain flagged and byte-preserved, no qualified identity relaxation |
| preservation | actual raw child controls | Actual extra_xml on each of the three owners independently stays flagged and survives save/reopen, opaque subtree payload retained |
| compatibility | `numbering_level_unmodeled_fact_contract_is_limited_to_extras` | Existing modeled metadata and raw property/leaf-overlay reader outcomes unchanged, richer authoring facts unchanged |
| native | existing complete public reader and level controls | Public level properties, extreme levels and paragraph/marker presentation retain values and no-overflow behavior |

Use the existing Rust test module, not a new test binary or production file.
Retain original compiled-before source/logs and bind current focused evidence.
Run affected native tests and prescribed scoped check, fmt, Clippy, rustdoc,
prose and adapter gates. Run unchanged49-entry hash harness and actual current
archive/dry-run checks. Independent `/microscope F-X191 --working ALL` must
reach zero valid findings on immutable final source/evidence before prepare.
`/verify --scoped F-X191` is the completion floor. Full integrated sprint
verification and final release approval remain separate.

## HLD impact

- `docs/hld/10-bindings-spec.md`

Clarify current intended native NumberingLevel reader completeness in "Two
supporting decisions" and "Native Word facade stability". Namespace
metadata remains retained without itself signaling unmodeled numbering
semantics. Actual extra attributes/children remain flagged. Richer authoring
admission and all binding surfaces retain their existing contracts. Write
current intent, not PR history. HLD04 is required reading but unchanged.

## Risk routing

- Parser or serializer classification: read HLD04 and HLD06 before editing. Preserve qualified identity, schema order and exact opaque child payloads. Test declaration, foreign attribute and raw child retention with saved/reopened packages. No parser/writer modification is planned.
- Public API of published crate: read HLD10 and structural rules. State the pre-1.0 boolean correction, no added signature. Actual `cargo publish --dry-run` and archive size/source inventory checks must pass for current source. Retain below10MiB compressed bound, exact member bytes/count and existing64-byte compression tolerance.
- No dependency, layout, external oracle, binding, release-script or new runtime construct trigger. Workflow record creation is explicitly user-authorized. Current archive documentation reconciliation does not authorize a release tag or publication.

## Hash harness

All49 existing entries are expected byte-identical. Namespace completeness is
an inspection-only fact and must not change rendered or generated outputs.
Any unexplained delta blocks completion. No baseline write is planned.

## Implementation checklist

- [x] Register exact SizeS/no-dependency/regression story and approve the design.
- [x] Add F-X191 as a formal F-X189 prerequisite in wave17 before wave18 preparation.
- [x] Record exact claimed Base and genuine compiled twelve-case before failure.
- [x] Adopt only the local reader predicate/docs and focused existing-entrypoint tests with contributor provenance.
- [x] Verify all namespace, foreign attribute, rebound alias, raw child and compatibility controls.
- [x] Reconcile current archive evidence without upstream platform override or stale values.
- [x] Complete HLD10 current intent, scoped/risk gates and independent zero-ALL microscope.
- [x] Prepare the normal worker handoff for integration before release preparation.

Integration and shared delivery totals remain owned by the integrator. The
worker completion state records the reviewed source, passed scoped gates and
validated handoff, without claiming that integration has happened.

## Open questions

None blocking. User authorized workflow records. Root chose the independent
story ID, size, test gate, single HLD impact and exclusion of contributor
archive-platform overrides. Wave17 is exclusive and precedes F-X189 wave18. The exact claim Base is
recorded at claim time, not inferred from this readiness snapshot.

## Read-only source bindings

- `crates/rdocx/src/document.rs`: `4057ef506a747776554fc57832eabc7daadc516f8ab8618b5af2961bb08910be`
- `crates/rdocx-oxml/src/numbering.rs`: `335be828bc866edfa40c32d32a8be2f184382ba4149ac47e1eb48ee3800d88be`
- `docs/hld/10-bindings-spec.md`: `359259a186eb2692f79e013d3dae31ba696cd493f41bbf4d2283d770bbd1d8bf`
- `docs/hld/04-opc-and-packaging.md`: `a514f00390dd905d3665d3456b7b5d15467af8db41aa3123c0e8393809e26eab`

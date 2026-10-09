# F-X190, Preserve cached text in multi-run complex field story snapshots

**Status**: completed
**Sprint**: S90
**Size**: M
**Depends on**: none

## Problem

[Issue291](https://github.com/tensorbee/rdocx/issues/291), reported by Hadrien
Mary (`hadim`), demonstrates cached PAGE text7 through direct story reads but
empty text through bulk snapshots and Python story_items. In the current
source, `crates/rdocx/src/document.rs:6673` namespace-closes a complex field's
sequence of sibling runs as one root. Only the first run receives inherited
bindings. Later result runs can lose their qualified identity. The physical
scan range is established at `document.rs:8453`. Local ancestor translation
at `document.rs:6664` can also subtract an enclosing field offset that lies
before an isolated nested excerpt. These explanations require compiled proof.

## Spec reference

- `docs/hld/03-architecture.md`, "Facade conventions", checked story items and source-owned projections.
- `docs/hld/04-opc-and-packaging.md`, "The package", namespace-qualified identity and retained producer XML.
- `docs/hld/10-bindings-spec.md`, "Python API shape" and "The invalidation problem, handled loudly", owned snapshots and read-only revisions.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "Binding tests" and "The hash harness", cached result parity, source preservation and actual binding runtime.
- `docs/hld/14-development-backlog.md`, "F-X190, Preserve cached text in multi-run complex field story snapshots (M)", the named acceptance gate.

## Intake and sequencing

The user's correction includes Issue291 and leaves Issue281 open for F-184.
Issue264 remains excluded. Previously approved permission covers this story's
design, review, progress and handoff records. No new production file, module,
crate, dependency, trait or generic is authorized or needed.

The full issue, latest PR287 body and empty discussion/review-comment arrays
were read. Contribution head is `39945370bbd18fc45ee0a1626e5bc8e9f73560da`.
Assess semantic commit `65365c4b2de49956f73c09bb326190e336db9bc8`
selectively and credit `hadim`. The newer PR290 stack inherits this fix, so do
not double count it or adopt unrelated comment policy changes. Upstream
archive measurements are not current package evidence.

Readiness evidence is `/private/tmp/S90-F-X190-reincluded291-design-readiness.md`,
SHA256 `b5d6e3c40c3be20b7770f9022de0e6fe6c3c7ee10e0eb1df5b679b0af5d9e592`.
There is no formal prerequisite. Shared document and test files require
exclusive wave16 after F-X188 completes and before F-282 resumes. Claim from
the actual integrated prefix. Release preparation moves to wave17 and depends
on this story's completion. No source or Cargo work overlaps those waves.

## Approach

Repair existing native bulk snapshots, direct story reads and Python
StoryItem.text. Preserve one valid namespace context across all sibling runs
of an isolated complex field. Reuse the existing qualified field scanner and
bounded namespace envelope machinery. Translate full and scan positions into
that envelope exactly. Enclosing ancestors outside the excerpt have no local
offset and must not underflow or redirect nested field/link reads. Establish
their discovery-only role before omitting them from an isolated projection.

Public field discovery retains its existing typed-admission boundary. A raw
hyperlink wrapper contained inside a complex cached-result span is not admitted
as a public field by the current source-equality proof. Keep that shape as an
opaque discovery and exact-byte preservation control. Real link span translation
inside a complex excerpt is proved in the existing private unit module with
qualified source bytes and distinct neighboring links. Public controls cover
surrounding links, fields contained in hyperlinks and non-complex links. The
admitted run-only nested fields independently prove actual ancestor behavior.
This clarification does not relax typed discovery or introduce a public API.

Keep the single-root path unchanged. Preserve nested instruction versus
result visibility, same-run and sibling fields, accepted and deleted text,
literal tabs and breaks, aliases and authoritative foreign shadows. Do not
replace field parsing with unqualified text concatenation or paragraph
accepted_text. Returned field XML, ContentLocations, fingerprints, ordering,
direct-body indices and source package bytes remain unchanged.

The issue explicitly identifies story_item_snapshot as an upstream PR287
addition. Canonical has no such public method. This repair adds no new public
snapshot API. Compare the existing bulk and direct routes and Python reads.
Keep X185's distinct checked story_range_paragraph_snapshot contract and X187
direct-control paragraph identity intact. No forwarding wrapper over the
entire bulk inventory or second location authority is introduced.

## Rejected alternatives

- Closing namespace scope on the first run leaves later siblings unbound.
- Concatenating every text element leaks instructions, deleted content and foreign nodes.
- Relaxing global namespace qualification risks unrelated story identities.
- Translating ancestor offsets outside an excerpt can underflow or select the wrong field.
- Adding the upstream one-location API solely to reproduce the issue obscures the regression in existing APIs.
- Word UI capture or a new field evaluator does not test exposure of already stored cached text.

## Test plan

**Test gate**: regression.
`complex_field_story_snapshots_preserve_cached_text` fails on the exact
claimed Base using existing API calls, then passes native and rebuilt Python
cached-result controls. A missing new API is not before evidence.

| Category | Test | Asserts |
|---|---|---|
| regression | `complex_field_story_snapshots_preserve_cached_text` | Five-run PAGE cached7 equals direct text, same-run equivalent and bulk/Python projections |
| native | existing field snapshot entrypoint controls | Body, header, footer, block control, cell and textbox owners supported by the existing inventory retain text, XML, ordering and exact locations |
| nested | nested field and link excerpt controls | IF instruction PAGE, REF result PAGE, sibling fields and real nested-result hyperlinks have correct text and link coordinates without panic |
| namespace | alias/default/local/shadow controls | Namespace closure preserves qualification, local bindings, foreign opaque bytes and surrounding-text isolation |
| text | cached result grammar controls | Empty and multi-run cache, escaped and CDATA text, tabs, breaks, tracked insertion and deletion retain established semantics |
| Python | existing test_core.py reporter reproduction | Current built extension exposes correct field text in body/control/header/footer, preserves source bytes and leaves held handles valid on reads |
| preservation | repeat/save/reopen comparisons | Source parts, unrelated package members, field XML, fingerprints and direct-child metadata remain exact |

Extend existing Rust and Python entrypoints only. Before remediation, retain
any actual nested ancestor panic separately from the primary exact-Base
empty-text failure. A source prediction is not a recorded runtime failure.
Run focused tests, all affected native tests, actual current Python core,
pinned strict typing/stub agreement, all-target checks, Clippy, rustdoc,
both WASM graphs and every scoped risk rider. No Word UI is needed.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Parser or serializer: read HLD04 and HLD06. Preserve child order, qualified identity and exact unmodeled XML. Namespace and byte comparisons exercise the isolated reader projection.
- Published crate: read HLD10 and CLAUDE structural rules. No public signature changes. Record the intentional returned-text correction, remeasure affected archives and README inventories, verify actual locally patched publication dry runs and the10MiB ceiling.
- Python binding behavior: read HLD10. Rebuild and authenticate the current extension, run actual runtime tests and pinned strict typing/stub agreement separately. Check both WASM graphs. Native test binaries exclude extension binding crates.
- Scoped verification includes affected all-target checks, complete affected native suites, all-feature denied-warning Clippy, fmt, denied-warning docs, hash49, README, workflow, prose and adapter checks. Full workspace verification and the union of riders remain due on the final integrated sprint.
- Workflow files were approved. No new production file/module/dependency/trait/generic. Source, Cargo and hash execution remain exclusive, with no baseline movement.

## Hash harness

Expected unchanged: all49 deterministic entries and rendered resources. The
intentional behavior change exposes stored cached field text through bulk
snapshots. It does not alter document field caches or serialized source. Any
unexpected output delta blocks completion until explained and reviewed.

## Implementation checklist

- [x] Read issue, contribution provenance, current source and risk references, and approve design with existing workflow-record permission.
- [x] Claim exclusive wave16 from the integrated prefix after F-X188 releases source and Cargo.
- [x] Capture genuine compiled exact-Base regression using existing APIs.
- [x] Repair complex-field namespace scope and nested local coordinate mapping without changing the global scanner or public surface.
- [x] Pass native, current Python, source-preservation and scoped risk controls.
- [x] Update exactly the HLD impact list and obtain an independent zero-finding microscope.
- [x] Prepare feature-local implementation and evidence for integrator handoff.

Independent ALL pass2 reports zero defects, smells and nitpicks. The named
gate passes again during prepare. Validated handoff94ddc0ff records CodeHead
49f84796 and was consumed by conflict-free integration126255dc. All12 integrated
feature files equal CodeHead. Delivery acceptance is recorded in AS_BUILT and
SPRINT_TRACKER. Full integrated sprint verification and review remain due.

## Open questions

None requiring new user input. Implement the smallest repair of existing
published routes. Issue281 remains open and Issue264 excluded. Preserve all
approved ownership policies and the full bibliography catalogue commitment.

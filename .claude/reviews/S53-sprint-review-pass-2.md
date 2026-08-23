# S53 sprint review, pass 2

**Reviewed**: `sprint/s53` at `573b5e8832d600dd53c06583132876f835051236`
against merge base `72f3384fc3b97aa7e4f31e0ec642c6b70bd69c59`, 126 files and
14,138 changed lines, crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-oxml`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`, and
`rpptx-wasm`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M17 gate is: "an encrypted document opens with its password, a signed
document verifies, and a rendered PDF passes a PDF/UA structure check"
(`docs/hld/14-development-backlog.md:1385`).

The technical gate holds on the integrated result. The encrypted-package oracle
is exercised by `word_agile_document_opens_only_with_its_password`
(`crates/oxml-opc/src/encryption.rs:1465`). Signature creation reopens the exact
serialized package and proves complete cryptographic coverage in
`signed_package_verifies_with_complete_coverage`
(`crates/oxml-opc/src/signature.rs:1747`). Word for Mac 16.104 recognized the
signature and protected the document. This is not represented as Windows
certificate-trust evidence. The tagged structure regression is at
`crates/oxml-pdf/src/writer.rs:2124`, and the pinned veraPDF 1.30.2 check covers
PDF/A-2b, PDF/A-3b, and PDF/UA at `crates/oxml-pdf/src/writer.rs:2324`.

The sprint is not ready to close because its release gates are intentionally
real. F-X049 remains reviewed and in-progress with no publication or
notification claimed. F-X050 remains blocked until incubating 0.5.0 is
published and verified. The clean preparation review does not replace either
separate final approval.

## Not found

- `pass 1 B1`: resolved. The latest recorded full verification covers exact
  reviewed HEAD `573b5e8832d600dd53c06583132876f835051236`
  (`.claude/scratch/S53-run.json:104`). It includes the full gate and the union
  of plan riders. The reconciled result is exactly 14 changed `pdf/bytes` and
  `pdf/pages` entries, with PNG, resources, and selected OOXML unchanged.
- `pass 1 B2`: resolved. The sprint contract now places every release-bound
  notification after successful publication and release-body verification, and
  before the release F-ID completes (`docs/sprints/CURRENT_SPRINT.md:96`). This
  matches the canonical release order and keeps preparation mutation-free.
- `interaction`: no conflict was found between tagged structure and PDF/A
  conformance, recursive table semantics and tagged layout, or staged signing
  and redaction package ownership.
- `duplication`: no sprint-local helper or policy was independently reimplemented
  under competing names.
- `layering`: `cargo metadata --no-deps` reports no `oxml-*` dependency on an
  `rdocx-*` or `rpptx-*` crate.
- `harness`: the current 49-entry check passes. The integrated baseline changes
  exactly the 14 declared PDF bytes and pages entries, and F-X048 owns two final
  values inside that set. PNG, PDF resources, and selected OOXML entries remain
  unchanged.
- `gate`: the named technical tests, pinned validators, and recorded external
  observations support the M17 gate without claiming publication or Windows
  certificate trust.
- `docs`: the plan-listed HLD sections describe the integrated package,
  rendering, binding, testing, asset, and release-preparation state.
- `deps`: the sprint adds no third-party dependency. The ICC profile and legal
  file are crate-local assets, and the 0.5.0 changes are internal version pins.
- `surface`: each added public type and method belongs to the approved signing,
  semantic-layout, PDF/A, redaction, or dense-form contract. Python, WASM, and
  CLI binding surfaces remain unchanged where the plans require isolation.
- `release preparation`: the incubating family is prepared at 0.5.0, the stable
  family remains at 0.8.0, `rpptx-wasm` remains unpublished, and the changelog
  separates shared hardened-equivalent credit from stable-only outcomes.
- `delivery ledgers`: the five non-release stories are completed consistently.
  F-X049 remains reviewed in run state and in-progress in both trackers.
  F-X050 remains pending or blocked on real incubating publication, with no
  premature `AS_BUILT.md` entry.
- `F-X051 identity and counts`: the new cross-cutting F-ID is unique, pending,
  size M, and scheduled for S54 in both the backlog and sprint plan
  (`docs/sprints/BACKLOG.md:454`). The cross-cutting and repository totals count
  it once, while the one archived cross-cutting story remains excluded from the
  status subtotals (`docs/sprints/BACKLOG.md:39`).
- `F-X051 contract`: the HLD records the completed F-X043 dependency, exact
  embedded-family priority, alias and fallback behavior, cache invalidation,
  warm-cold equality, both WASM targets, and an unchanged hash gate
  (`docs/hld/14-development-backlog.md:2625`). S54 keeps it independent of the
  RTF work (`docs/sprints/SPRINT_PLAN.md:956`).
- `F-X051 evidence boundary`: Issue 44 and PR 45 are queued for S54 rather than
  folded into either S53 release. The story requires authenticated
  `@emptinessform` credit only in the next release that actually contains the
  behavior (`docs/hld/14-development-backlog.md:2636`). It claims no current
  implementation, release, closure, comment, or publication.

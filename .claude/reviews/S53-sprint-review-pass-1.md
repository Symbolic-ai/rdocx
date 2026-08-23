# S53 sprint review, pass 1

**Reviewed**: `sprint/s53` at `120d1da103d0e7de97103e155c6bcad30a4366f6`
against merge base `72f3384fc3b97aa7e4f31e0ec642c6b70bd69c59`, 125 files and
14,004 changed lines, crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-oxml`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`, and
`rpptx-wasm`
**Verdict**: 2 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, full verification does not cover the reviewed HEAD

`.claude/scratch/S53-run.json:98`

The only recorded passing full verification covers `fa6b473`, while this pass
reviews `120d1da` after the sprint-ledger commit. The release contract requires
the latest full verification to pass at the current HEAD
(`.claude/commands/release.md:61`). The current evidence therefore cannot
authorize the `rpptx-v0.5.0` approval boundary. Commit the review and any
remediation, rerun the complete `/verify --full` gate and the union of sprint
riders at the resulting release candidate HEAD, and record that exact SHA and
the reconciled 14-entry PDF delta before release preflight.

### B2, the sprint contract puts contributor notifications before publication

`docs/sprints/CURRENT_SPRINT.md:96`

The definition of done requires every release-bound maintainer comment before
publication. The canonical workflow requires comments only after publication
and release-body verification (`.claude/WORKFLOW.md:254`), and the reviewed
F-X049 plan refuses notification during preparation
(`.claude/plans/F-X049-design.md:59`). Following the sprint wording would make
an external claim about a release that does not yet exist. Change the definition
of done to require the comments after successful publication and before the
release F-ID completes.

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
signature and protected the document, but this is not represented as Windows
certificate-trust evidence. The tagged structure regression is at
`crates/oxml-pdf/src/writer.rs:2124`, and the pinned veraPDF 1.30.2 check covers
PDF/A-2b, PDF/A-3b, and PDF/UA at `crates/oxml-pdf/src/writer.rs:2324`.

The sprint is not ready to close. F-X049 is prepared and remains in-progress,
with no publication or notification claimed. F-X050 remains blocked until the
incubating 0.5.0 graph is published and verified. Those are real release gates,
not substitutes for the technical M17 evidence.

## Not found

- `interaction`: no conflict was found between tagged structure and PDF/A
  conformance, recursive table semantics and tagged layout, or staged signing
  and redaction package ownership.
- `duplication`: no sprint-local helper or policy was independently reimplemented
  under competing names.
- `layering`: `cargo metadata --no-deps` reports no `oxml-*` dependency on an
  `rdocx-*` or `rpptx-*` crate.
- `harness`: the integrated baseline changes exactly the 14 declared
  `pdf/bytes` and `pdf/pages` entries. F-X048 owns two final values inside that
  set. PNG, PDF resources, and selected OOXML entries remain unchanged, and the
  current 49-entry harness check passes.
- `gate`: apart from B1's stale exact-HEAD record, the named technical tests and
  external observations support the M17 gate without claiming publication or
  Windows certificate trust.
- `docs`: apart from B2, the plan-listed HLD sections describe the integrated
  package, rendering, binding, testing, asset, and release-preparation state.
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

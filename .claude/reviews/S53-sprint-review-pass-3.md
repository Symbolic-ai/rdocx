# S53 sprint review, pass 3

**Reviewed**: `sprint/s53` at `ac4280ed879f19d437b0703b9924c8352d39ef29`
against merge base `72f3384fc3b97aa7e4f31e0ec642c6b70bd69c59`, 138 files and
14,623 changed lines, crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`, `rdocx-opc`,
`rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`, `rpptx`, `rpptx-chart`,
`rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-py`, `rpptx-render`, and
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

The incubating release gate is also complete. The 15-package 0.5.0 family was
published and verified from reviewed SHA `343388e19bce21b3d83f17e8cc0e5418861a94cb`
(`docs/sprints/AS_BUILT.md:8622`). The stable release gate remains intentionally
incomplete. F-X050 is prepared and reviewed, but no `v0.9.0` tag, crates.io
version, GitHub release, or post-release notification exists. This clean review
does not replace the separate immediate approval required by `/release v0.9.0`.

## Not found

- `prior findings`: pass 1 B1 remains resolved. The latest recorded full
  verification covers exact reviewed HEAD
  `ac4280ed879f19d437b0703b9924c8352d39ef29`
  (`.claude/scratch/S53-run.json:137`). Pass 1 B2 remains resolved by the
  post-publication, pre-completion notification order
  (`docs/sprints/CURRENT_SPRINT.md:96`). The clean pass 2 conclusions and the
  S54 F-X051 planning record are unchanged.
- `verification`: the exact-HEAD full gate and all union riders passed with the
  reconciled 14-entry PDF-only baseline delta. The current 49-entry harness
  matches. PNG, PDF resources, and selected OOXML entries remain unchanged.
- `interaction`: no conflict was found between tagged structure and PDF/A
  conformance, recursive table semantics and tagged layout, staged signing and
  redaction package ownership, or the two separately versioned release trains.
- `duplication`: no sprint-local helper or release policy was independently
  reimplemented under competing names.
- `layering`: `cargo metadata --no-deps` reports no `oxml-*` dependency on an
  `rdocx-*` or `rpptx-*` crate.
- `harness`: F-X050 changes version and release metadata only. It adds no
  baseline entry and moves none of the 14 reviewed PDF bytes and pages values.
- `gate`: the named technical tests, pinned validators, exact-head verification,
  and recorded external observations support the M17 gate without claiming
  Windows certificate trust or stable publication.
- `docs`: the four F-X050 HLD files distinguish prepared workspace 0.9.0 from
  published stable 0.8.0, and distinguish both from published incubating 0.5.0.
  They grant no binding, WASM, Python, npm, or cross-family publication
  authority.
- `deps`: the sprint adds no third-party dependency. The F-X050 diff changes
  versions and release carriers without changing dependency direction.
- `surface`: F-X050 adds no Rust, Python, WASM, CLI, or package API. Every
  earlier public addition remains tied to its approved feature contract.
- `stable authority`: the preparation selects exactly `rdocx-opc`,
  `rdocx-oxml`, `rdocx-layout`, `rdocx-html`, `rdocx-pdf`, `rdocx`, and
  `rdocx-cli` for crates.io in dependency order
  (`.github/workflows/publish.yml:55`). The metadata regression proves the
  eleven inherited 0.9.0 carriers, nine pins, unpublished binding and WASM
  members, and exact seven-package allowlist
  (`scripts/test_sprint_workflow.py:3962`).
- `incubating isolation`: all sixteen explicit incubating manifests and pins
  remain at the published 0.5.0 boundary. `rpptx-wasm` remains unpublished,
  and the stable workflow contains no incubating publish command.
- `package dry run`: the clean integrated no-flag command produced exactly the
  22-package union. Every archive is below 10 MiB. `oxml-layout` contains all
  20 TTF files and three legal files, `rdocx-layout` contains no duplicate TTF,
  `oxml-pdf` contains the ICC profile and licence, and `rpptx` contains
  `assets/default.pptx`.
- `contribution inventory`: authenticated GitHub evidence confirms Issues 15
  and 23 belong to `@mantissaman`, while Issues 39 and 42 and PRs 40, 41, and
  43 belong to `@emptinessform`. All seven direct links occur twice in the
  rendered notes, and the notes distinguish direct Issue 15 and Issue 23
  outcomes from the five hardened-equivalent outcomes
  (`CHANGELOG.md:29`).
- `PR 43 evidence`: authenticated GitHub state reports PR 43 closed and
  unmerged. Its closure comment identifies exact reviewed F-X048 commit
  `fa9c0c9b3326a2b522f3ee3cc3e5c39429189973` and says it was addressed rather
  than merged. The prepared notification preserves that classification.
- `release boundary`: F-X050 remains reviewed in run state and in-progress in
  both delivery trackers (`docs/sprints/CURRENT_SPRINT.md:44`). Its plan leaves
  the real `/release v0.9.0` approval and post-publication verification steps
  unchecked (`.claude/plans/F-X050-design.md:141`). Local and remote tag checks,
  the GitHub release lookup, and all seven crates.io version lookups confirm
  that stable 0.9.0 has not been published.
- `delivery ledgers`: F-X049 has one completed tracker row and one `AS_BUILT.md`
  entry with publication and notification evidence. F-X050 has neither
  completion record yet, which is correct until its real release gate succeeds.

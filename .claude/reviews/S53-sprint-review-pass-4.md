# S53 sprint review, pass 4

**Reviewed**: `sprint/s53` at `89190b5b29e19ec62fd26c91130d758ad5d72bc4`
against merge base `72f3384fc3b97aa7e4f31e0ec642c6b70bd69c59`, 139 files and
14,807 changed lines, crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`, `rdocx-opc`,
`rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`, `rpptx`, `rpptx-chart`,
`rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-py`, `rpptx-render`, and
`rpptx-wasm`
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have

## Extension rationale

Pass 3 was clean, so the normal three-pass review loop had already stopped.
This fourth pass is an explicitly authorized extension of the bound. It exists
solely because `/run-sprint` step 9 required another bounded sprint review after
the real `v0.9.0` publication gate and F-X050 delivery-record finalization
changed the sprint HEAD. It is not a confirmation pass over the unchanged pass
3 state.

## Blocking

None.

## Should-fix

### S1, the repository summary still describes stable 0.9.0 as unpublished

`CLAUDE.md:14`

The current repository description says the exact stable family is on
crates.io at 0.8.0 and that workspace 0.9.0 is only prepared for the next
stable release. That now contradicts the finalized versioning authority, which
records the exact seven-package 0.9.0 publication and reviewed tag SHA
(`docs/hld/03-architecture.md:466`). This can send a future agent down the
already completed release path. Update the summary to identify 0.9.0 as the
published stable boundary while retaining the separate incubating 0.5.0 train.

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
certificate-trust evidence. Tagged output proves `/StructTreeRoot` ownership
(`crates/oxml-pdf/src/writer.rs:2169`), and the pinned veraPDF 1.30.2 check
covers PDF/A-2b, PDF/A-3b, and PDF/UA (`crates/oxml-pdf/src/writer.rs:2323`).

Both real publication gates also hold. The completed incubating 0.5.0 evidence
remains recorded in the prior F-X049 entry. For stable 0.9.0, the remote
annotated tag dereferences to reviewed release SHA
`e27e519c94c90cd5be340fe5bf8e431cf542ac51`. Actions run `32658680024`
completed successfully at that SHA. Its stable allowlist step succeeded, its
incubating allowlist step was skipped, and its GitHub Release job succeeded.
All seven selected 0.9.0 registry entries are non-yanked and owned by
`mantissaman`. The GitHub release body is byte-identical to the committed
render, and all seven reviewed release-bound comments exist at the exact URLs
recorded in the completion evidence (`docs/sprints/AS_BUILT.md:8680`).

## Not found

- `prior findings`: pass 1 B1 remains resolved for the publication boundary.
  Full verification and the union riders passed at exact reviewed release SHA
  `e27e519c94c90cd5be340fe5bf8e431cf542ac51`, with the reconciled 14-entry
  PDF-only delta recorded in run state. The post-publication commit changes only
  the completed plan, delivery records, and plan-listed HLD files, which is the
  affected-check path required by `/run-sprint` step 9. Pass 1 B2 remains
  resolved by the post-publication, pre-completion notification order
  (`docs/sprints/CURRENT_SPRINT.md:96`).
- `pass 2 conclusions`: the clean integrated conclusions remain valid. The S54
  F-X051 plan entry and its Issue 44 and PR 45 scope have not changed since pass
  2, and no S53 delivery or release claim was added for them.
- `interaction`: no conflict was found between tagged structure and PDF/A
  conformance, recursive table semantics and tagged layout, staged signing and
  redaction package ownership, or the separately published incubating and
  stable release trains. Publication finalization changes no source code.
- `duplication`: no sprint-local helper or release policy was independently
  reimplemented under competing names.
- `layering`: `cargo metadata --no-deps` reports no `oxml-*` dependency on an
  `rdocx-*` or `rpptx-*` crate.
- `harness`: the integrated baseline changes exactly the 14 declared
  `pdf/bytes` and `pdf/pages` entries. PNG, PDF resources, and selected OOXML
  remain unchanged. Neither stable preparation, pass 3 recording, publication,
  nor delivery finalization changes the baseline.
- `gate`: the named technical tests, pinned validator, exact-release-SHA full
  verification, successful real publication workflow, registry observations,
  and recorded external observations support the complete M17 and release
  gates without claiming Windows certificate trust or binding publication.
- `HLD docs`: apart from S1 outside the HLD set, the F-X050 plan lists exactly
  four HLD files and finalization changes exactly those four. Their versioning,
  binding, testing, and release-process sections describe published stable
  0.9.0, published incubating 0.5.0, the reviewed SHAs, and continuing
  publication isolation.
- `deps`: the sprint adds no third-party dependency. Publication finalization
  changes no manifest or lockfile, and the release keeps the reviewed dependency
  graph and package ordering.
- `surface`: publication finalization adds no Rust, Python, WASM, CLI, or package
  API. Every earlier public addition remains tied to its approved feature
  contract.
- `stable authority`: the workflow selects exactly `rdocx-opc`, `rdocx-oxml`,
  `rdocx-layout`, `rdocx-html`, `rdocx-pdf`, `rdocx`, and `rdocx-cli` in
  dependency order (`.github/workflows/publish.yml:55`). The real run published
  that step and skipped the incubating step.
- `registry and bindings`: each selected crate resolves at 0.9.0 as non-yanked
  with `mantissaman` as owner. `oxml-py-support`, `rdocx-py`, `rpptx-py`,
  `rdocx-wasm`, and `rpptx-wasm` remain manifest-ineligible and have no 0.9.0
  crates.io record. `rpptx-wasm` remains at unpublished 0.5.0.
- `release body and notifications`: direct byte comparison of the GitHub
  release body and canonical render succeeds. The seven distinct comment URLs
  in `AS_BUILT.md` resolve to the intended Issues 15, 23, 39, and 42 and PRs
  40, 41, and 43. Their bodies name `v0.9.0`, retain direct versus
  hardened-equivalent classification, provide the reviewed credit, and cite the
  exact F-X048 SHA for PR 43 (`docs/sprints/AS_BUILT.md:8704`).
- `delivery ledgers`: F-X050 is completed once in its plan, backlog, current
  sprint, tracker, and `AS_BUILT.md`. The run state reports all seven S53
  features completed. The published SHA, run, package count, binding boundary,
  contribution inventory, notification URLs, and 14-entry harness evidence
  agree across those records.

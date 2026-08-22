# S52 sprint review, pass 2

**Reviewed**: `sprint/s52` at `f243a777b7bfbc904f02489e741d2a5445281127`
against merge base `6a97f93cfb89b9478b04ae05a4d26db24c2b938a`, 87 files and
15,090 changed lines, crates: `oxml-chart`, `oxml-layout`, `oxml-opc`,
`oxml-pdf`, `rdocx-layout`, `rdocx`, `rpptx-render`, `rpptx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Pass-one remediation

- **B1 resolved**: direct references contribute manifest roots only after their
  digest succeeds, and recursive manifests are reached only through further
  digest-valid same-document references at
  `crates/oxml-opc/src/signature.rs:271`. The mutation regression at
  `crates/oxml-opc/src/signature.rs:1571` appends a digest-correct unsigned
  manifest and proves that it cannot cover the new part.
- **B2 resolved**: the parser retains processing instructions at
  `crates/oxml-opc/src/signature.rs:784`, and exclusive canonicalization emits
  them in XML child order at `crates/oxml-opc/src/signature.rs:957`. The
  regression at `crates/oxml-opc/src/signature.rs:1268` proves exact canonical
  bytes and proves that changing only the instruction content invalidates the
  referenced-object digest.
- **B3 resolved**: password-key parsing no longer imposes its own key size on
  `encryptedKeyValue` at `crates/oxml-opc/src/encryption.rs:194`. Parent-level
  validation uses the independent `keyData` size at
  `crates/oxml-opc/src/encryption.rs:693`. The matrix at
  `crates/oxml-opc/src/encryption.rs:1430` passes all nine data-key and
  password-key size combinations across SHA-1, SHA-256, SHA-384, and SHA-512.
- **S1 resolved**: the backlog total now records 228 done, 32 pending, and one
  archived story across 261 F-IDs at `docs/sprints/BACKLOG.md:40`.

The feature-enabled `oxml-opc` suite passes 42 tests, including all three code
remediation regressions. The `rdocx-layout` suite passes 152 tests, including
the combined paragraph, table, header and footer, restart, substituted-page,
font-remap, provenance, and warm-versus-cold paths.

## Milestone gate

> An encrypted document opens with its password, a signed document verifies,
> and a rendered PDF passes a PDF/UA structure check.

The two S52 legs now hold. The source-encoded Word oracle and
`word_agile_document_opens_only_with_its_password` prove authenticated reading,
and the recorded Word 16.104 observation proves that output from this library
opens only with the correct password. Signature verification now combines the
valid-signature and changed-part regressions with authenticated manifest
reachability and mutation-sensitive processing-instruction coverage. The
feature-enabled package suite passes all 42 tests at this reviewed SHA.

The end-of-M17 gate does not yet hold because the PDF/UA leg belongs to the
pending F-173 work in S53 at `docs/hld/14-development-backlog.md:1417`. This is
the planned milestone boundary, not an S52 defect. The final full verification
still needs to run over the review-record HEAD before sprint close, as required
by the sprint workflow.

The deterministic harness remains consistent with the approved F-X041 change.
All 49 hashes match the recorded baseline, including the reviewed 26-entry
delta, and all seven page-one golden pixel buffers match at 150 DPI. The XML
fingerprints remain unchanged.

## Not found

- **Interaction**: no defect was found across the exact-context engine transfer,
  bundled fallback, paragraph, table, header and footer, restart,
  substituted-page, font canonicalization, provenance, or empty-paragraph
  paths. The cache partitions total the declared 4,224 entries and 64 MiB at
  `crates/rdocx-layout/src/engine.rs:579`, and warm outputs remain complete
  equals of cold outputs.
- **Duplication**: no materially duplicated sprint helper was found. The
  follow-up stories extend the existing engine, cache, paginator, font, and
  output constructs.
- **Layering**: no `oxml-*` crate gained an `rdocx-*` or `rpptx-*` dependency.
- **Harness**: no unexplained deterministic or golden-pixel delta was found.
- **Gate**: every S52 definition-of-done item has named automated or recorded
  manual evidence. The remaining PDF/UA milestone leg is explicitly assigned
  to S53.
- **Docs**: the HLD now describes authenticated manifest reachability,
  processing-instruction canonicalization, and independent Agile key sizes at
  `docs/hld/04-opc-and-packaging.md:319` and
  `docs/hld/04-opc-and-packaging.md:361`. No implementation contradiction was
  found outside a plan's declared HLD impact.
- **Dependencies**: every new dependency has a named Agile-encryption or
  digital-signature consumer. Both graphs remain default-off, and the ordinary
  `oxml-opc` graph contains neither cryptographic feature graph.
- **Public surface**: the native security APIs, shared immutable payloads,
  bundled-fallback entry point, and checked transfer methods are required by
  their stories. The low-level transfer validates complete retained context at
  `crates/rdocx-layout/src/engine.rs:678`. No unchecked document engine setter
  was introduced.
- **PR 40 and PR 41 replacement safety**: S52 retains shared font and page
  payloads, caller-font bundled fallback, exact paragraph lookup, transactional
  table and header/footer reuse, bounded restart and substituted-page reuse,
  and attributed empty paragraphs. It excludes the draft raw engine setter,
  hash-authoritative equality, immediate cache publication, and unbounded
  retained maps.
- **Ledgers**: all 12 S52 stories are done in the current sprint and backlog,
  have completed plans and clean feature reviews, and have exactly one
  `AS_BUILT.md` entry and one `SPRINT_TRACKER.md` row.

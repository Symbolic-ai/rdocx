# S52 sprint review, pass 1

**Reviewed**: `sprint/s52` at `1666deee578a20c6572622b88ca2d06c0ced5c48`
against merge base `6a97f93cfb89b9478b04ae05a4d26db24c2b938a`, 86 files and
14,787 changed lines, crates: `oxml-chart`, `oxml-layout`, `oxml-opc`,
`oxml-pdf`, `rdocx-layout`, `rdocx`, `rpptx-render`, `rpptx`
**Verdict**: 3 blocking, 1 should-fix, 0 nice-to-have

## Blocking

### B1, unsigned manifests can manufacture complete signature coverage

`crates/oxml-opc/src/signature.rs:291`

The verifier authenticates the direct `SignedInfo` references, then separately
walks every `Manifest` descendant anywhere under the signature root and credits
all of those references toward cryptographic validity and package coverage. An
attacker can append an unsigned `Object` containing a second `Manifest` to a
valid partial signature. Correct digests in that unsigned manifest make the
missing parts and relationships appear covered without changing the signed
`Object` or `SignedInfo`, so the report can claim both cryptographic validity
and complete coverage for declarations that the signer never authenticated.
The verifier must derive manifests only from the authenticated same-document
reference graph. A regression must append an unsigned manifest to a valid
partial signature and prove that it can never complete coverage.

### B2, exclusive canonicalization drops processing instructions

`crates/oxml-opc/src/signature.rs:763`

The XML parser discards every processing instruction, but exclusive XML
canonicalization without comments retains processing instructions. A
processing instruction inside an authenticated same-document object therefore
does not contribute to the digest calculated here. Changing only that node can
leave the report cryptographically valid even though the signed XML changed,
and conforming signatures that include one can be rejected. The parsed form
and canonicalizer must retain processing instructions with their canonical
placement and escaping. A mutation-sensitive regression must cover a
processing instruction inside a referenced object.

### B3, Agile parsing binds the encrypted package key to the wrong key size

`crates/oxml-opc/src/encryption.rs:212`

`encryptedKeyValue` carries the intermediate package key whose length comes
from the parent `keyData`, while the password key encryptor's `keyBits` controls
the key used to encrypt that value. These are independent Agile parameters.
The early check instead requires the encrypted value to have the password key
encryptor's key length. The later parent-level check correctly requires the
`keyData` key length, so only descriptors where both sizes happen to match can
pass. The test builder also assigns one input to both sizes at
`crates/oxml-opc/src/encryption.rs:1960`. Valid mixed AES-128, AES-192, and
AES-256 descriptors are rejected, contrary to the declared complete read
profile. Validation must use the parent `keyData` size only, with a matrix that
varies the data and password-encryptor key sizes independently.

## Should-fix

### S1, the backlog total undercounts completed stories

`docs/sprints/BACKLOG.md:40`

The generated total reports 227 done stories. The milestone summary rows and
the completed backlog entries both total 228. With 261 stories, 32 pending,
and one archived story, the done count must be 228. Regenerate the summary so
the sprint ledger is internally consistent.

## Nice-to-have

None.

## Milestone gate

> An encrypted document opens with its password, a signed document verifies,
> and a rendered PDF passes a PDF/UA structure check.

The M17 end gate does not yet hold at this reviewed SHA. The Word oracle and
`word_agile_document_opens_only_with_its_password` provide evidence for the
password-opening leg, although B3 leaves the declared read profile incomplete.
The signature leg is blocked by B1 and B2 because the current success report
does not reliably prove what was signed. The PDF/UA leg is assigned to the
pending tagged-PDF work in S53 and has not been demonstrated in S52. This is
consistent with M17 spanning S52 and S53, but the S52 sprint gate remains
blocked until the three blocking findings are resolved and final verification
runs over the remediated ledger HEAD.

The deterministic harness declaration is accurate. The approved F-X041 delta
is exactly 26 of the 49 entries: five page-one PNG hashes and all three PDF
fingerprints for seven samples. All 21 XML hashes are unchanged. The separate
golden-pixel harness confirms all seven page-one buffers at 150 DPI.

## Not found

- **Interaction**: no defect was found in the combined restart, paragraph,
  table, header and footer, substituted-page, fallback-font, or empty-paragraph
  paths. Their exact-context checks, transactional publication, and shared
  memory limits remain coherent when enabled together.
- **Duplication**: no materially duplicated sprint helper was found.
- **Layering**: no `oxml-*` crate gained an `rdocx-*` or `rpptx-*` dependency.
- **Harness**: apart from the declared 26-entry F-X041 delta, no unexplained
  deterministic or golden-pixel change was found.
- **Docs**: the plans' HLD impact union is reflected in the updated current
  specifications. No contradiction was found outside the ledger count in S1.
- **Dependencies and feature isolation**: every new dependency has a named
  security consumer. The Agile and signature graphs remain default-off, and
  neither graph enters the Word or PowerPoint WASM builds.
- **Public surface**: the new native security APIs and checked layout-transfer
  surfaces are called for by their stories. No unchecked raw engine setter or
  unrelated public API was introduced.
- **PR 40 and PR 41 replacement safety**: the integrated stories retain the
  intended shared payloads and bounded reuse while excluding unchecked engine
  access, hash-authoritative equality, and unbounded caches.
- **Ledger coverage**: all 12 S52 stories have matching tracker and as-built
  entries. No missing or duplicate S52 story record was found.

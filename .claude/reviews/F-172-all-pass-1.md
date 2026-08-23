# F-172, all, pass 1

**Reviewed**: uncommitted F-172 working tree against `HEAD`, 11 files and 623 changed lines
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, the generated package Object omits required OPC signature properties
`crates/oxml-opc/src/signature.rs:208`

The package-specific `ds:Object` is serialized with only a `ds:Manifest`.
ECMA-376 Part 2 requires that object to contain the Manifest followed by
`ds:SignatureProperties`, including `ds:SignatureProperty` and the OPC
`SignatureTime`. The shared verifier accepts the smaller self-generated shape,
so local cryptographic success does not make the XML an OPC-conforming package
signature. Microsoft Word can open the ZIP while refusing to report this
signature as valid, which is the distinction the oracle gate was meant to
detect.

### D2, signing a package with an existing origin creates a second origin
`crates/oxml-opc/src/signature.rs:185`

`allocate_origin_part` treats an existing Digital Signature Origin part as an
ordinary name collision and allocates `origin1.sigs`, after which creation adds
another package-level origin relationship. OPC permits no more than one
Digital Signature Origin part. The existing origin must be reused for another
signature, or the operation must fail before mutation. The later verifier call
also selects only the newly created report at
`crates/oxml-opc/src/signature.rs:234`, so an invalid pre-existing signature or
invalid multiple-origin graph does not prevent the candidate from being
committed.

### D3, relationship sources are not validated before signing
`crates/oxml-opc/src/signature.rs:283`

The manifest walks every `part_rels` entry without checking that its source is
an existing normalized package part. A loaded package containing
`/word/_rels/missing.xml.rels` but no `/word/missing.xml` can therefore have the
orphan relationship item authenticated and receive a successful complete
coverage report as long as its targets exist. An empty source key reaches
`relationship_part_name` and slices an empty string at
`crates/oxml-opc/src/signature.rs:363`, which panics instead of returning the
required fail-closed error. This contradicts the documented rejection of
dangling graph entries.

### D4, the round-trip and Word gates do not validate their claimed boundaries
`crates/oxml-opc/src/signature.rs:1619`

`signed_package_verifies_with_complete_coverage` signs and verifies the same
in-memory `OpcPackage`. It never serializes, reopens, and verifies the emitted
DOCX, so it is not a round-trip test under the repository taxonomy and cannot
catch writer or reload regressions. The ignored Word test at
`crates/oxml-opc/src/signature.rs:1659` only repeats the local assertions and
optionally writes bytes. It does not obtain a Word signature verdict. The
progress record confirms that Word opened the file but the visible signature
result was not observed at `.claude/scratch/F-172-progress.md:7`, while the
approved test contract requires Word to report a valid signature at
`.claude/plans/F-172-design.md:74`.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in RSA-SHA256 algorithm selection, strict PKCS#8 and
X.509 DER parsing, key and certificate matching, cloned package and document
commit atomicity, deterministic part and relationship ordering, relationship
transform ID order, present XML Signature child order, certificate trust
separation, default-off dependency isolation, native facade gating, WASM and
binding surface isolation, public API packaging, HLD impact scope, or the
repository structure rules.

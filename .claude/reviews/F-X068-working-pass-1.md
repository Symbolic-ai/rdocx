# F-X068, working, pass 1

**Reviewed**: the complete 47-file working diff against claim Base `1ff6add`,
262 insertions and 157 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: all 15 publishable shared and PowerPoint packages, the
  preparation-only WASM carrier, workspace pins, lock records, READMEs, source
  assertions, and CI literals move coherently to 0.8.0 while stable carriers
  remain at 0.11.0.
- Contract: the diff stays within the approved F-X068 recovery scope, retains
  the immutable v0.11.0 evidence, credits no unrelated issues or pull requests,
  and grants no publication authority before the separate release approval.
- Panics: no runtime parsing or untrusted-input path changes in this metadata
  story.
- OOXML: no parser, serializer, namespace, schema-order, or raw-subtree path
  changes.
- Tests: the named carrier, isolation, notes, metadata, package-order, and
  workflow tests prove the preparation boundary. Mutation controls reject a
  missing or false stable-only registry condition and authority value. The
  registry-only 0.8.0 consumer is correctly deferred until publication.
- Structure: no new file, module, trait, generic, wrapper, feature flag, crate,
  public API, or dependency was introduced. The stable-only preflight remains
  one explicit workflow step before packaging.
- Assets and supply chain: all 22 patched workspace archives verify below 10
  MiB. The shared archive retains the complete font, licence, notice, and
  subset-provenance inventory without duplicating fonts in the stable layout
  crate.
- HLD discipline: exactly the five plan-listed HLD files describe the current
  prepared 0.8.0 state and keep external publication assigned to `/release`.

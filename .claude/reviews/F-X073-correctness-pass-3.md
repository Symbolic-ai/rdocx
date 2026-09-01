# F-X073, correctness, pass 3

**Reviewed**: claim base `ef34d1f31eb9334d993bf333762079682633319f`
through the complete current working-tree delta, the approved revised plan,
cited HLD sections, progress record, pass 1 and pass 2 reviews, and untracked
state. The tracked delta is 7 files with 858 changed lines, 691 additions and
167 deletions. The two untracked prior reviews add 236 reviewed lines.
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, aliased duplicate bookmark identity attributes still pass the exact-raw guard
`crates/rdocx-layout/src/engine.rs:2490`
`crates/rdocx-layout/src/engine.rs:2512`
`crates/rdocx-oxml/src/text.rs:4829`
`crates/rdocx-layout/src/engine.rs:11224`

The remediation reparses the raw element and compares the resulting marker,
but the underlying bookmark projection returns the first attribute whose
prefix is bound to the Word namespace. It does not count attributes by expanded
name. A raw element such as `<w:bookmarkStart xmlns:x="http://schemas.openxmlformats.org/wordprocessingml/2006/main" w:id="7" x:id="8" w:name="target"/>`
therefore yields one typed marker with ID 7 in both the source parse and the
guard parse. The guard sees a required ID and matching marker and accepts the
raw entry even though `w:id` and `x:id` are duplicate expanded-name identity
attributes. The same bypass applies to two Word-qualified `name` attributes.
This leaves the pass 2 requirement to reject duplicate identity attributes
unmet and lets namespace-invalid raw content publish restart state instead of
failing closed. Count the Word-expanded `id` and `name` attributes during the
complete empty-element validation, require exactly one `id`, require exactly
one `name` for a start and zero for an end, and add same-URI alias duplicates to
the adversarial regression. The current test proves one ordinary alias only.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 2 D1, complete root shape** produced no finding beyond D1. Leading and
  trailing whitespace are trimmed, the candidate must be one lexical empty
  element, and reparsing must yield exactly one paragraph raw entry and one
  bookmark marker at `crates/rdocx-layout/src/engine.rs:2471`. Nested content,
  trailing roots, foreign elements, malformed XML, missing or invalid IDs,
  missing start names, and unexpected end names fail closed.
- **Namespace aliases and shadows** produced no finding beyond D1. A locally
  declared alias bound to the Word namespace is accepted by expanded identity,
  while a local fixed-prefix or alias binding to a foreign URI produces no
  bookmark projection and is rejected. An inherited alias absent from the
  retained raw subtree is conservatively rejected rather than guessed.
- **Pass 2 D2, raw and typed correspondence** produced zero findings. Each raw
  entry must align with one typed marker, the same run boundary, and the exact
  count of preceding raw children at that boundary at
  `crates/rdocx-layout/src/engine.rs:2453`. Reordered or stale `raw_before`
  values fail closed.
- **Field safety and page-pair retention** produced zero findings. Every source
  field, including one whose rendered `field_kind` is `None`, prevents
  checkpoint pagination at `crates/rdocx-layout/src/engine.rs:1409`. Supported
  substitution pairs remain eligible for the complete restart record.
- **Aggregate accounting** produced zero findings. Candidate admission uses
  checked arithmetic across current and pending paragraph, table, header or
  footer state plus the complete candidate at
  `crates/rdocx-layout/src/engine.rs:2180`. The 5,216-entry and 64 MiB limits,
  component caps, complete retained payload accounting, and overflow rejection
  remain intact.
- **Ordinary prose and split fallback** produced zero findings. Multi-line
  prose, headings, `keepNext`, and `keepLines` reach finalized block-boundary
  checkpoints. Any paragraph split is recorded at
  `crates/rdocx-layout/src/paginator.rs:2167`, disables the restart candidate,
  and causes full pagination before publication.
- **Exact restart identity and transactionality** produced zero findings.
  Complete serialized body identity, exact retained context, note ordering,
  font trace, provenance mode, page substitution inputs, and suffix equality
  remain authoritative. Failed or over-budget candidates publish no partial
  restart state.
- **Panics and bounds** produced zero findings. The production delta adds no
  unguarded input indexing, recursion, or unchecked arithmetic. The raw slice
  bounds derive from enumeration, and retained-byte calculations saturate to a
  rejected candidate.
- **OOXML preservation and schema order** produced no serializer, namespace
  replay, schema-order, relationship, or package finding. D1 is a fail-closed
  cache classification defect. It does not alter the retained raw bytes.
- **Public API and structure** produced zero findings. No public API,
  dependency, feature, module, file, trait, generic, binding, or package graph
  changed. The private helpers remain beside the existing source-safety
  predicates.
- **HLD and hash expectations** produced zero findings. HLD 08, 12, 14, and 15
  remain aligned with aggregate admission, complete boundaries, field-pair
  retention, and the retired independent restart byte ceiling. The
  deterministic hash harness matched all 49 entries unchanged.
- **Focused evidence**: the unsafe-content regression, ordinary-prose cases,
  and aggregate-budget cases passed. Full `rdocx-layout` passed 225 unit tests
  and one doctest. All-target, all-feature `rdocx-layout` Clippy, both WASM
  facade target checks, workspace formatting, and diff hygiene passed. The
  bookmark regression has no duplicate expanded-name mutation and therefore
  does not detect D1.

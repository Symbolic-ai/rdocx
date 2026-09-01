# F-X073, correctness, pass 2

**Reviewed**: claim base `ef34d1f31eb9334d993bf333762079682633319f`
through the complete current working-tree delta, the approved revised plan,
cited HLD sections, progress record, pass 1 review, and untracked state. The
tracked delta is 7 files with 794 changed lines, 627 additions and 167
deletions. The untracked pass 1 review was also included.
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, bookmark root validation ignores nested or trailing raw content
`crates/rdocx-layout/src/engine.rs:2464`
`crates/rdocx-layout/src/engine.rs:2488`
`crates/rdocx-layout/src/engine.rs:11158`

The new validator extracts the root name and selected start-tag attributes, but
it never validates the remainder of the raw entry. A parsed
`<w:bookmarkStart w:id="7"><w:unknown/></w:bookmarkStart>` therefore matches
the typed start marker and ID and is accepted as represented. A public raw
entry containing a valid empty bookmark followed by a second foreign element
also passes. The Word paragraph parser can produce the first case directly
because it creates the typed marker before preserving the complete element at
`crates/rdocx-oxml/src/text.rs:2326`. Missing required IDs also pass when both
the raw attribute lookup and typed marker ID are `None`. These cases retain
unrepresented raw content despite the fail-closed contract. Parse exactly one
complete bookmark element, require its legal required identity, and reject
nested content, trailing bytes, duplicate identity attributes, or another root.
The remediation test currently covers only a foreign root and cannot detect
these bypasses.

### D2, bookmark correspondence does not validate raw ordering identity
`crates/rdocx-layout/src/engine.rs:2453`
`crates/rdocx-layout/src/engine.rs:5496`
`crates/rdocx-oxml/src/text.rs:416`

The correspondence check zips raw entries with typed markers and validates the
run boundary, root kind, ID, and name, but omits `BookmarkMarker::raw_before`.
That value is the retained count of raw children preceding the marker and is
used to decide where the zero-width target marker enters the inline stream. A
public paragraph can contain two otherwise exact bookmark pairs at one run
boundary, reorder both public vectors together, and leave their private
`raw_before` values stale. Every implemented comparison still passes although
typed marker order no longer corresponds to preserved raw order. This violates
the requested stale-sidecar fail-closed boundary. Validate each marker's raw
ordinal within its run boundary, and add reordered and stale-ordinal
regressions.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 1 D1 remediation** is complete. Checkpoint eligibility now scans every
  body paragraph's typed source for `RunContent::Field` at
  `crates/rdocx-layout/src/engine.rs:1409`, independently of rendered
  `field_kind`. PAGE, DATE, unresolved REF, and unsupported field regressions
  all require zero checkpoints at
  `crates/rdocx-layout/src/engine.rs:11128`.
- **Field page-pair retention** produced zero findings. Field source remains
  eligible for the complete restart record while being ineligible for
  checkpoint pagination. The existing PAGE case proves the restart record and
  substitution pairs remain present with an empty checkpoint vector at
  `crates/rdocx-layout/src/engine.rs:12004`.
- **Pass 1 D2 partial remediation** correctly rejects the original cardinality
  bypass. A raw entry must now pair with one typed marker, use its run boundary,
  carry the expected start or end type, and match its ID and name at
  `crates/rdocx-layout/src/engine.rs:2453`. Foreign prefixed roots and all alias
  roots fail closed through the literal standard-root comparison at
  `crates/rdocx-layout/src/engine.rs:2477`. A local `w` shadow is rejected by
  the namespace comparison at `crates/rdocx-layout/src/engine.rs:2482`. D1 and
  D2 are the remaining complete-root and stale-order gaps.
- **Aggregate accounting** produced zero findings. Checked admission includes
  current and pending paragraph, table, header or footer entries and bytes plus
  the complete candidate at `crates/rdocx-layout/src/engine.rs:2180`. The 5,216
  entry and 64 MiB limits remain authoritative and arithmetic overflow rejects
  the record.
- **Split fallback** produced zero findings. The paginator records every actual
  paragraph split at `crates/rdocx-layout/src/paginator.rs:2167`. The engine
  then clears record eligibility, reruns complete full pagination, and
  publishes no checkpoint state at
  `crates/rdocx-layout/src/engine.rs:1519`.
- **Ordinary-prose correctness and exact identity** produced zero findings.
  Multi-line prose, headings, `keepNext`, and `keepLines` reach only complete
  finalized block boundaries. Restart reuse still requires exact context,
  ordered note references, font trace, source mode, serialized body identity,
  and suffix equality at `crates/rdocx-layout/src/engine.rs:1420`.
- **Panics and bounds** produced zero findings. The remediations add no
  untrusted indexing, slicing, recursion, or unchecked arithmetic in
  production. Aggregate addition is checked, retained-byte calculation
  saturates to rejection, and the 1,024 aligned restart-slot cap is unchanged.
- **OOXML and preservation** produced no parser, serializer, namespace-write,
  schema-order, relationship, or package finding. D1 and D2 concern whether a
  retained raw subtree is safe for cache admission. They do not change its
  preserved bytes.
- **Public API and structure** produced zero findings. No public surface,
  dependency, feature, module, file, trait, generic, binding, or package graph
  changed. The new helpers are private and keep source classification beside
  the existing cache predicates.
- **HLD and hash expectations** produced zero findings. HLD 08, 12, 14, and 15
  remain aligned with aggregate admission, complete boundaries, field-pair
  retention, and the absence of a restart byte partition. The deterministic
  hash harness matched all 49 entries unchanged.
- **Focused evidence**: the unsafe-content, substituted-page retention,
  fallback, three ordinary-prose, and two aggregate regressions passed. Full
  `rdocx-layout` passed 225 unit tests and one doctest. The `rdocx` library
  passed 326 tests with three ignored, and its regression binary passed 180
  tests with one ignored. All-target, all-feature `rdocx-layout` Clippy, both
  WASM target checks, workspace formatting, prose, and diff hygiene passed.
  The exact ignored release-mode 1,000-page performance regression passed with
  deterministic fonts and one test thread.

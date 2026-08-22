# F-175, Redaction

**Status**: approved
**Sprint**: S53
**Size**: M
**Depends on**: F-147, F-149

## Problem

`Document` owns typed body, comment, revision, core-property, and custom-property
state alongside the raw OPC package at `crates/rdocx/src/document.rs:65`.
Saving flushes several typed parts directly into the live package at
`crates/rdocx/src/document.rs:796`. There is no operation that removes sensitive
text across all of those representations as one atomic mutation.

Charts add a second recoverability path. The package stores both ChartML caches
and an embedded workbook, created from one source at
`crates/rdocx/src/document.rs:929`. Replacing only visible body text or only a
chart cache leaves the original value recoverable from comments, revisions,
metadata, raw XML, or SpreadsheetML inside the embedded package.

## Spec reference

- ECMA-376 Part 1, WordprocessingML text, comments, revisions, core and custom
  properties, ChartML caches, and embedded SpreadsheetML packages.
- `docs/hld/04-opc-and-packaging.md`, "Package integrity".
- `docs/hld/09-charts-spec.md`, "Cached values are not optional" and
  "Authoring API".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and the raw-package
  regression requirements.

## Approach

Add a native `Document::redact_text` operation that applies one non-empty
selector and returns a `RedactionReport` with per-surface replacement counts.
The operation clones complete typed and package state with
`clone_for_staging`, flushes the candidate, and performs all edits on that
candidate. The live document is replaced only after the candidate serializes,
reopens, validates, and passes a raw ZIP scan.

Traverse modeled Word text in body, tables, nested controls, headers, footers,
footnotes, endnotes, comments, and every accepted or rejected revision branch.
Redact core and custom string properties. For preserved XML parts, use one
expanded-name XML text rewriter that copies unaffected byte ranges verbatim,
changes only text and attribute values in the approved sensitive surfaces, and
fails closed on malformed XML. Do not parse unrelated elements into the object
model.

For every chart relationship, redact string and numeric text in ChartML caches
and follow the internal package relationship to an embedded workbook. Open the
workbook as bounded OPC, redact shared strings, inline strings, and matching
cell values, then serialize it back into the staged outer package. External
workbook relationships are rejected because their bytes are outside the
document's atomic boundary.

The final scan examines every inflated ZIP entry, including nested workbook ZIP
entries, for both UTF-8 and UTF-16LE forms of the selector. Any remaining trace
fails the operation and preserves the original document. Python, WASM, and CLI
surfaces remain unchanged.

## Rejected alternatives

- Drawing an opaque rectangle hides pixels but leaves all source text
  recoverable.
- Replacing only modeled `w:t` nodes misses comments, deleted revisions,
  metadata, ChartML caches, and workbooks.
- Blind byte replacement can corrupt XML escaping, UTF-16 content, numeric
  chart caches, and ZIP structure.
- Mutating one part at a time cannot provide the required all-or-nothing
  boundary.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `redaction_rewrites_only_approved_xml_text_and_attributes` | Expanded-name parsing is prefix tolerant, escaping is valid, unmodelled siblings remain byte-identical, and malformed XML fails closed. |
| regression | `redaction_removes_body_comments_revisions_and_metadata_traces` | Visible, inserted, deleted, commented, core, custom, header, footer, note, table, and control occurrences are absent after reopen. |
| regression | `redaction_removes_chart_cache_and_embedded_workbook_traces` | Chart string and numeric caches plus shared, inline, and cell workbook values no longer contain the selector. |
| regression | `redaction_failure_is_atomic` | Invalid selectors, malformed parts, external workbooks, nested ZIP limits, serialization, and post-scan failures leave package bytes, typed state, and caches unchanged. |
| round-trip | `redacted_package_preserves_unrelated_parts_and_relationships` | Unrelated modeled and unmodelled parts, relationship targets, content types, and child order remain byte-identical. |
| regression | `raw_zip_scan_finds_no_redacted_value` | Neither the outer package nor nested workbook contains UTF-8 or UTF-16LE forms of the sensitive value. |

The test gate is **regression**. Redacted text is absent from every part of the
saved package, checked by scanning the raw ZIP rather than the model.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/09-charts-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Any parser or serialiser: re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add prefix-alias, schema-order,
  malformed-input, nested-package, and byte-preservation tests.
- Public API of a published crate: record the additive native semver impact,
  run dry-run packaging for `rdocx`, and assert archive size.
- WASM bindings: run both WASM checks and prove the native redaction surface
  does not expand Python, WASM, or CLI APIs.
- New module and file: explicit approval is required for
  `crates/rdocx/src/redaction.rs`. It keeps one complete atomic operation out
  of the already 9,700 line document module.

## Hash harness

Expected to be unchanged. Generated samples do not invoke redaction.

## Implementation checklist

- [ ] Define the smallest native selector and report types.
- [ ] Stage all typed and raw package mutation on a complete clone.
- [ ] Traverse every Word story, comments, revisions, and properties.
- [ ] Redact ChartML caches and relationship-resolved embedded workbooks.
- [ ] Preserve unrelated raw XML and package parts byte for byte.
- [ ] Reopen, validate, and raw-scan outer and nested packages before commit.
- [ ] Add atomic failure, round-trip, WASM, packaging, and harness checks.

## Open questions

None. The first public API accepts one non-empty exact literal, without regex.
The new internal file `crates/rdocx/src/redaction.rs` is approved for the
staged traversal, package rewrite, and post-scan logic.

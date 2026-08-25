# F-181, all recovery2, pass 3

**Reviewed**: the complete working tree diff across 18 files with 6,851
additions and 4 deletions, including the untracked EPUB writer, all eight prior
review records, the second-recovery progress note, the approved plan, and the
cited HLD and risk contracts
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, style-derived deep headings receive a false reduction diagnostic

`crates/rdocx/src/epub.rs:773`

`projected_heading_level` detects a named style whose `outline_lvl` is 6 or
higher and reports that the heading was reduced to level 6. The projection
clamp at `crates/rdocx/src/epub.rs:2252` sees only direct paragraph properties,
so it does not change this style-derived level. The projected style still
resolves to level 7 or higher in `rdocx-html`, which emits a paragraph rather
than an `h6`. The existing focused fixture exposes the mismatch by producing
only two `h6` elements for its three diagnosed deep headings. A caller is told
that the heading level survived in reduced form when its heading semantics were
actually dropped. The writer must either clamp the style-derived projection to
level 6 or report the loss accurately.

### D2, malformed PNG chunk type codes pass structural validation

`crates/rdocx/src/epub.rs:3508`

The fallback chunk branch rejects an unknown chunk only when its first type
byte is uppercase. PNG requires all four type bytes to be ASCII letters and
requires the reserved third byte to be uppercase. A CRC-correct ancillary
chunk such as `teXt`, or one containing a non-letter such as `t0XT`, can be
inserted after `IHDR` and before `IDAT`. `oxml_media::probe` ignores it and this
validator accepts it, so the bytes are packaged as supported `image/png`
without a diagnostic. That contradicts the HLD and test contract that media is
structurally validated before packaging.

## Smells

None.

## Nitpicks

None.

## Not found

- Targeted pass 2 remediation: parser-derived raw ordinals correlate a
  paragraph-local Word alias even when it shadows a foreign document-root
  binding. Foreign siblings remain unconsumed and diagnosed. Indexed PNG
  palettes are capped at `2^bit_depth`. Visible document-default and effective
  default-style paragraph effects are diagnosed per affected unstyled
  paragraph, active run defaults are conditional on projected text, and
  revision, change, and raw-only defaults remain quiet. HTTP user information
  accepts only the RFC 3986 user-information character set, with global percent
  escape validation, while valid IPv6 and IPvFuture literals remain accepted.
- Archive and EPUB structure: apart from D2's media boundary, no additional
  defect was found in the stored first `mimetype`, container, package metadata,
  manifest, spine, navigation, stylesheet, XHTML flow structure, media
  deduplication, fixed timestamps, compression choices, or ZIP entry order.
- Loss diagnostics and source semantics: apart from D1, no additional defect
  was found in metadata, styles, defaults, numbering, revisions, fields,
  hyperlinks, tables, drawings, shading, underline, spacing, or break
  diagnostics. Supported siblings remain in source order.
- Bounds, panics, determinism, atomicity, and preservation: no additional
  unchecked production panic, overflow, recursion escape, unbounded export
  allocation, unstable output choice, destination truncation, staging leak,
  live document mutation, or retained DOCX XML mutation was found.
- Public API, dependency graph, HLD scope, and structure: the additive native
  API, approved private module, existing `zip` dependency, unchanged Python,
  WASM, and CLI surfaces, and six modified HLD files match the approved plan.
- Focused evidence: all 33 ordinary EPUB tests passed. The combined
  source-built publication also passed the exact checksum-verified EPUBCheck
  5.3.0 JAR. Those green fixtures do not exercise D1's diagnostic-to-output
  mismatch or D2's malformed ancillary chunk types.

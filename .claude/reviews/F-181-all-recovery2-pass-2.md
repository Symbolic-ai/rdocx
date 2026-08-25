# F-181, all recovery2, pass 2

**Reviewed**: the complete working tree diff across 17 files with 6,559
additions and 4 deletions, including the untracked EPUB writer, all seven prior
review records, the second-recovery progress note, the approved plan, and the
cited HLD and risk contracts
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, a paragraph-local Word alias cannot shadow a foreign root binding

`crates/rdocx/src/epub.rs:963`

Raw revision correlation passes only the document-root namespace declarations
to `marker_raw_kind`. If the root binds `x` to a foreign namespace and a
paragraph validly shadows `x` to the Word namespace, the paragraph parser
creates the typed revision from the local binding but the captured `x:ins`
fragment has no declaration of its own. The raw classifier resolves its
unknown prefix against the stale foreign root binding at
`crates/rdocx/src/epub.rs:1896` and refuses to correlate it. The exporter then
reports both the typed revision loss and an unmodelled-XML loss for one source
wrapper. The new alias fixture covers local declarations when the root has no
same-prefix binding, but not this valid namespace-shadowing case.

### D2, indexed PNG palettes can exceed their bit-depth capacity

`crates/rdocx/src/epub.rs:3397`

The `PLTE` state check accepts any nonempty palette through 256 entries. For an
indexed-colour image, PNG limits the palette to `2^bit_depth` entries. A 1-bit
indexed PNG with three CRC-correct palette entries therefore passes this
validator, the header probe, IDAT ordering, and terminal IEND checks, then is
packaged as supported media. This contradicts the HLD claim that critical PNG
chunk counts are legal. The recovery tests cover duplicate IHDR and late PLTE,
but not palette cardinality.

### D3, visible document defaults disappear without a diagnostic

`crates/rdocx/src/epub.rs:1365`

`render_styles` starts from `CT_Styles::new`, so it drops the modeled
`doc_defaults` paragraph and run properties. Diagnostic detection starts only
from the named default paragraph style at
`crates/rdocx/src/epub.rs:1481`. A document whose `w:docDefaults` makes
unstyled text bold or adds paragraph spacing consequently loses that visible
effect in EPUB without the promised default-style diagnostic. The focused test
changes the named default style and does not exercise document defaults.

### D4, revision-only default styles produce visible-formatting noise

`crates/rdocx/src/epub.rs:1486`

The visible-effect test compares the complete paragraph and run property values
with `Default`. A default paragraph style whose only run state is
`rPrChange`, revision markers, or retained revision XML therefore counts as
visible formatting even though its active run formatting is empty. Every
unstyled paragraph then receives a `default paragraph style formatting was
dropped` diagnostic. That violates the HLD rule that inert defaults stay quiet
and also duplicates one style-level preservation item across unrelated source
paragraphs.

### D5, brackets in HTTP user information pass the URI validator

`crates/rdocx/src/epub.rs:1774`

The HTTP-family branch removes user information before it checks the host for
brackets. A target such as `https://us[er]@example.com/` has one `@`, leaves a
plain `example.com` host, and is accepted. Raw square brackets are not permitted
in RFC 3986 user information, so the writer emits a syntactically invalid
absolute URI without a loss diagnostic. Existing malformed-URI tests cover
brackets in the host suffix, path, query, fragment, and `mailto`, but not the
user-information component.

## Smells

None.

## Nitpicks

None.

## Not found

- Targeted second-recovery behavior: exact raw ordinals correlate local Word
  and foreign aliases when no root binding shadows them. Duplicate IHDR, late
  PLTE, noncontiguous IDAT, repeated PLTE, and nonterminal IEND are rejected.
  D1 and D2 identify the remaining namespace-shadow and PNG-count boundaries.
- Background and shading recovery: document backgrounds are bounded and
  diagnosed. Patterned, foreground, and non-hex paragraph, run, and table-cell
  shading are diagnosed, while valid fill colours survive the bounded
  projection. D3 and D4 cover the remaining default-style boundary.
- Archive and EPUB structure: no additional defect was found in the stored
  first `mimetype`, container, package metadata, manifest, spine, navigation,
  stylesheet, XHTML flow structure, media deduplication, fixed timestamps,
  compression choices, or ZIP entry order.
- Lists, headings, images, and loss reporting: source-ordered spine splitting,
  nested navigation, numbered headings, list identity and counters, bounded
  nesting, image-source correlation, alternative text, page-break lifting, and
  the previously named modeled and raw loss diagnostics remain intact.
- Bounds, panics, determinism, and atomicity: no additional unchecked
  production panic, overflow, recursive depth escape, unbounded export
  allocation, unstable output choice, destination truncation, staging leak, or
  live source mutation was found.
- Public API, dependency graph, HLD scope, and structure: the additive native
  API, approved private module, existing `zip` dependency, unchanged Python,
  WASM, and CLI surfaces, and six modified HLD files match the approved plan.
- Oracle and focused evidence: all 33 ordinary EPUB tests passed. The combined
  source-built fixture also passed the exact checksum-verified EPUBCheck 5.3.0
  JAR. Those green fixtures do not exercise D1 through D5.

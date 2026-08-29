# F-199, working, pass 3

**Reviewed**: working diff against
`4225fb60fa5c14301c25c759c185c667b179c698`, 13 feature files with 1,653
tracked insertions and 92 tracked deletions, plus 122 lines of oracle licence
and provenance text and one 15,344-byte oracle font fixture
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-2 D1, source reconstruction and failure handling: the checked
  `MultilingualTextSegment` constructor preserves every rich shaping field
  while changing only the embedded source at
  `crates/rdocx-layout/src/engine.rs:3292`. Both line items and retained reflow
  items use that path, errors propagate instead of exposing a partially
  reconstructed segment, and all owned heading, table, header, and footer
  callers propagate the result. The focused heading regression at
  `crates/rdocx-layout/src/engine.rs:9166` and the complete story-source
  regression at `crates/rdocx-layout/src/engine.rs:10891` are green, including
  the source-free header and footer boundary.
- Pass-2 D2, hybrid bidi and hyphenation: the final logical line contents now
  contribute both legacy and rich text to one paragraph-wide bidi resolution
  at `crates/oxml-layout/src/line.rs:622`. Each completed line applies UAX 9 L1
  through `reordered_levels` before L2 visual reordering at
  `crates/oxml-layout/src/line.rs:675`. The byte cursor spans wraps, generated
  conditional hyphens, and forced-break lines without changing source spans.
  The RTL-first Arabic and hyphenatable English regression at
  `crates/rdocx-layout/src/engine.rs:7355` is green, as are the existing
  wrapped internal-whitespace L1 regression and conditional-hyphen gates.
- Pass-2 D3, oracle locking: macOS holds one fixed POSIX advisory lock across
  the complete CoreText registration and Writer conversion lifetime at
  `scripts/docx_ssim_harness.py:273`. Acquisition is nonblocking with a bounded
  240-second monotonic timeout, the `finally` path releases the lock and closes
  its descriptor, and non-macOS runs avoid CoreText and the POSIX lock. The
  child-process overlap, timeout, and error-release regression at
  `scripts/docx_ssim_harness.py:872` is green.
- Prior pass-1 D1 through D4: conditional hyphenation remains attached to
  eligible Latin spans, mixed text nodes retain their direct, bidirectional,
  and East Asian effective languages, eligible stored and resolved field text
  retains language and spacing metadata, and drawing reflow retains the
  reviewed Word 0.8em rich baseline. No regression of those closures was found.
- Correctness and compatibility: Word consumes the existing shared rich types,
  keeps paragraph-wide logical ordering and line-local visual ordering, and
  preserves F-X058 cluster, offset, source, and backend contracts. The feature
  adds no public type, field, entrypoint, dependency, module, or product font.
- Panics and errors: source slicing uses scalar boundaries and checked source
  arithmetic, rich reconstruction uses the shared validating constructor, and
  mismatched shaping or bidi cardinality returns a layout error. No new
  untrusted-input panic or unchecked rich-run access was found.
- OOXML and provenance: the diff does not alter namespace classification,
  schema order, raw run preservation, or the F-X066 raw-position sidecar. Rich
  body, table, note, header, footer, field, and cache paths retain their exact
  source ownership.
- Oracle determinism, licence, and packages: the isolated evidence remains
  green on four of four pages with raw scores from 0.972241230 to 0.997558968.
  The oracle source and output hashes match the recorded bytes, the copied OFL
  is byte-identical to the approved Noto licence, the three-file inventory is
  exact, and the fixture stays outside published crate archives.
- Tests, HLD, and structure: the focused pass-2 remediation tests and the
  20-test harness self-test are green. Recorded affected suites, portability,
  package, supply-chain, and 49-of-49 hash gates are green. Exactly the five
  plan-listed HLD files changed, and no forwarding-only wrapper or unnecessary
  public abstraction was introduced.

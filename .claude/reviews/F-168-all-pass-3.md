# F-168, all, pass 3

**Reviewed**: uncommitted `work/f-168-codex` diff, 4 files and 2,188 changed lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, native layout still falls back to the default variant when Word selects a blank first or even header
`crates/rdocx-layout/src/paginator.rs:925`
`crates/rdocx-layout/src/paginator.rs:944`
`crates/rdocx-layout/src/paginator.rs:969`

The watermark selector borrows the default watermark whenever the selected
first or even header has none. The ordinary even header and footer selectors
also use an empty block list as a signal to borrow the default variant. Word
does not use content emptiness for this decision. An explicit empty variant is
still the selected variant, and a missing first-section variant is blank rather
than the default variant. The new setter exposes this mismatch when a document
has a nonempty default company header, enables even headers, and has no initial
even reference. The setter correctly creates a watermark-only even header, so
Word shows that watermark without the company header. Native layout sees the
empty even block list and adds the default company header. A producer document
with a default watermark and an explicit blank first header likewise gains a
watermark in native layout that Word does not display. Materialized same-type
inheritance now handles actual missing references, so these default fallbacks
should not remain.

### D2, an entity-encoded false setting enables even headers
`crates/rdocx/src/document.rs:4171`

`word_on_off_value` reads the raw attribute bytes instead of the XML-decoded
value. A valid setting such as `w:val="&#48;"` has the decoded value `0`, so Word
disables even headers. This reader compares the literal bytes `&#48;` with
`0`, `false`, and `off`, then returns true. The layout consequently loads and
selects even headers that are inactive in the document. The setting regression
only covers an unescaped `w:val="0"` and does not exercise this valid XML form.

### D3, the section page-number reader ignores expanded names and XML attribute decoding
`crates/rdocx-layout/src/engine.rs:1662`
`crates/rdocx-layout/src/engine.rs:1713`

The raw scanner recognizes `pgNumType` and `start` by local name alone, then
parses the undecoded attribute bytes directly. A preserved foreign element such
as `<x:pgNumType x:start="2"/>` therefore changes even-header parity even though
it is not WordprocessingML. Conversely, a valid
`<w:pgNumType w:start="&#49;"/>` is ignored because the raw entity bytes do not
parse as an integer. The helper also searches descendants inside each preserved
raw child rather than requiring the schema-level `w:pgNumType` child. Any of
these cases can select the wrong default or even header for a restarted
section. The new restart regression covers only canonical prefixes and a
literal integer.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 2 remediation correctly materializes package-visible first and even
watermark headers, preserves inherited ordinary defaults, uses separate
logical parity across section restarts, emits or removes VML template
references without leaving a dangling reference, keeps foreign same-local end
tags out of the VML state machine, and obtains image ids from the collision-safe
media registry. The earlier raw-byte ownership, complete-header preservation,
custom target normalization, malformed colour diagnostic, margin centering,
atomic staging, cache invalidation, deterministic shaping, z-order, schema
child order, and structural-rule concerns are also resolved.

The focused suites passed with 14 `rdocx` watermark tests, 11
`rdocx-oxml` header and footer tests, and 1 `rdocx-layout` watermark test.
`python3 scripts/prose_check.py` and `git diff --check` also passed.

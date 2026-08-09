# F-115, all, pass 2

**Reviewed**: complete revised working tree diff, 7 files, 737 changed lines, comprising 725 insertions and 12 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Prior findings

- Pass-1 D1 is resolved at `crates/rpptx-oxml/src/slide_parts.rs:87` and
  `crates/rpptx-oxml/src/slide_parts.rs:363`. Existing direct backgrounds now
  replace only the selected fill byte range. The exact regression at
  `crates/rpptx-oxml/tests/integration.rs:114` proves that container and
  property attributes, comments, effects, extension children, and surrounding
  raw bytes remain identical.
- Pass-1 D2 is resolved at `crates/rpptx/src/lib.rs:451`. An occupied
  conventional core-properties part now returns a concrete error before the
  package is cloned or an output path is written. The regression at
  `crates/rpptx/tests/integration.rs:173` covers both `to_bytes` and `save`,
  retains an existing output file, and confirms the source package bytes.
- Pass-1 D3 is resolved at `crates/rpptx/tests/integration.rs:59` and
  `crates/rpptx/tests/integration.rs:149`. The facade gate inspects the exact
  requested background fill after package serialization, and missing-core
  creation proves the part exists with the requested subject. The direct
  background regression now compares the complete retained subtree.

## Not found

- Correctness: zero findings. Missing `show` remains visible, explicit values
  retain the required inverse semantics, slide-size validation completes
  before mutation, and background replacement changes the typed rendering only
  after replacement serialization succeeds.
- Contract: zero findings. The public surface matches the approved plan.
  Core properties follow their package relationship, immutable access remains
  clean, missing metadata uses the conventional part with collision failure,
  and slideshow mode remains output-only.
- Panics: zero findings. The private background fill range comes only from
  writer lengths or parser event boundaries over the same byte buffer. The
  scanner selects a DrawingML fill only at the immediate `p:bgPr` depth, and
  every error path returns before mutating the background, package, or output
  file.
- OOXML: zero findings. Root `show` reads both boolean spellings and writes
  fixed numeric values. New backgrounds occupy the schema slot before
  `p:spTree`. Existing direct backgrounds preserve all bytes outside the fill,
  while theme references and unsupported choices remain untouched.
- Tests: zero findings. The four focused pass-2 regressions passed. The
  invalid-size test compares complete package bytes, background replacement
  compares the full raw subtree, missing-core collision covers `to_bytes` and
  `save`, and slideshow output is compared structurally except for the main
  content type.
- Structure: zero findings. No file, module, trait, generic, feature,
  dependency, forwarding wrapper, or unjustified dynamic dispatch was added.

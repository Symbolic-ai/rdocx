# F-003, correctness, pass 1

**Reviewed**: F-003 working diff, 4 files with 251 additions and 0 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

No script-safety or check-mode mutation defect was found. Check mode has no
baseline write path, removes every expected DOCX and PNG before invoking the
generator, and returns distinct nonzero results for output deltas and
operational errors. An independent check reproduced all entries while the
baseline SHA-256 remained
`9a3c64d61df793b9d8f7203df9cb966fb67201518b4f7fc0f2e68d276aaaca8f`.

No collection or baseline-accuracy defect was found. The seven names match the
seven generator entries, each contributes three OOXML states and one PNG state,
and the checked-in manifest has 28 sorted entries. Independent recomputation
matched every value. The absent `invoice:word/numbering.xml` entry is preserved
explicitly as JSON `null` rather than omitted.

No deterministic-rendering defect was found. The sample generator renders page
index zero at exactly 150 dpi through
`Document::render_page_to_png_deterministic()` before writing each PNG.

No update-gate defect was found. Update mode validates and trims a nonempty
reason before running the generator or opening the baseline. The unit test and
an independent blank-reason invocation both prove refusal without creating or
rewriting a baseline.

No ordering or comparison-report defect was found. Manifest keys and each
added, removed and changed group are sorted, and the comparison unit test pins
the precise diagnostics for all three classes.

No stale-output gap was found for hashed artifacts. Every expected DOCX and PNG
is deleted before generation. A missing, failed or partial generator output is
therefore reported rather than satisfied by an earlier run.

No gap was found in the deliberate writer-injection gate. The progress evidence
records that a single XML whitespace injection left the structural round-trip
test green while check mode reported all seven `word/document.xml` digests as
changed and exited 1. The reverse patch restored a green 28-entry check, and no
writer diff remains in the reviewed tree.

No test-scope defect was found. The two unit tests cover added, removed and
changed comparisons plus reason-gated update refusal, while the generated
baseline check exercises sample generation, ZIP-part hashing, explicit absence
and deterministic PNG hashing end to end.

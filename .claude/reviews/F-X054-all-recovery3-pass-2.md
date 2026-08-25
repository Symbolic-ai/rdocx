# F-X054, all, third recovery pass 2

**Reviewed**: uncommitted working diff, 15 files, 3,463 changed lines with
3,394 additions and 69 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

The two third-recovery pass 1 defects are fixed. Raw-dependent local aliases
bound to WordprocessingML correlate retained producer XML with modeled siblings
that use the same alias. Logical owner snapshots compare Word and XML names by
resolved namespace, while retained foreign names preserve their lexical
structure. The source-built regression performs a modified save and reopen and
keeps the exact raw producer bytes on the correct paragraph owner.

Intermediate namespace declarations now shadow owner declarations by scope
provenance, including when the prefix and URI are identical. A retained raw
descendant below that intermediate declaration is self-contained and permits a
safe edit. A direct retained use of the owner-local foreign `wp` binding still
fails closed.

No additional findings were found in body, cell, paragraph, hyperlink, or run
item ordering, complete typed variant projection, exact raw bytes, modeled
unsupported facts, namespace resolution and replay, producer-defined numbering
preservation, layout and exporter marker suppression, fail-closed ordinary or
deleted text decoding, Python error classification, legacy flattened
accessors, public enum exhaustiveness, OOXML child order, panic safety, public
documentation, dependency structure, test naming, or the repository
structural rules.

All 168 `rdocx` regression tests passed. The two namespace recovery regressions
and the ordered-reader save and reopen gate also passed independently. Prose
checking and `git diff --check` passed.

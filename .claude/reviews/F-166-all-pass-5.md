# F-166, all, pass 5

**Reviewed**: Uncommitted working diff, 4 files and 1,855 changed lines, with
1,810 additions and 45 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-4 D1 is resolved. The sectioned-merge dependency check now scans the
complete serialized main document for WordprocessingML `headerReference` and
`footerReference` elements, including references inside typed block content
controls and preserved raw block wrappers. The matching regression test proves
that a varying field in a header referenced only by a nested section is
rejected.

Relationship resolution: no finding. Reference IDs are selected by the
expanded office-document relationship namespace. Foreign same-local-name and
unbound attributes are ignored. A reference is followed only when its
relationship has the matching header or footer type, is internal, resolves to
an existing package part, and has not already selected that physical part.
External relationships, missing relationships, missing targets, and duplicate
references do not cause a panic or a duplicate scan.

Correctness and contract: no finding. Separate outputs remain record ordered,
sectioned output retains schema-final section properties, absent merge values
use the merge-local empty policy, and record-varying non-body dependencies are
rejected before candidate assembly.

Ordinary API behavior: no finding. The full-document relationship scan is used
only by sectioned mail merge. Existing `evaluate_fields` and `update_fields`
continue to use their original typed header and footer discovery and do not
gain nested-story behavior.

Scanner correctness and OOXML: no finding. Element and attribute expanded
names honor in-scope declarations and shadowing. Simple and complex field
instructions decode entities once, complex stacks preserve begin, separate,
and end state, and changed identity references are escaped on write. Unrelated
producer XML is not reserialized.

Identity uniqueness: no finding. Body identities and typed or raw `REF`,
`PAGEREF`, and hyperlink references share decoded remap keys. Existing
unresolved generated-name candidates are reserved before checked allocation.

Footnote handling: no finding. Clean relationship-resolved footnote parts stay
source-backed and are patched in place. Complete typed serialization remains
limited to explicitly dirty footnotes.

Panics: none found. Empty input is rejected before candidate indexing, XML
spans are bounded, and identity allocation uses checked arithmetic.

Tests: no finding. `cargo test -p rdocx --test regression_test` passed all 93
tests, including the nested section-reference regression.

Structure and scope: no finding. The merge-only relationship scanner and the
shared expanded-attribute helper remain concrete and local. No new trait,
generic, module, forwarding wrapper, feature flag, or speculative abstraction
was introduced.

Verification also passed `cargo check -p rdocx --all-targets`, `git diff
--check`, and `python3 scripts/prose_check.py`.

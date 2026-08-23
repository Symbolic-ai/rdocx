# F-X051, all, pass 3

**Reviewed**: uncommitted working diff, 4 files, 790 insertions and 40 deletions, 830 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, explicit alias state has no configured bound

`crates/oxml-layout/src/font.rs:403`

`set_caller_aliases` clones every supplied entry into both `explicit_aliases`
and `explicit_alias_map` without an entry or retained-byte ceiling. The reusable
context then copies the complete slice again at
`crates/rdocx-layout/src/engine.rs:492`. A caller can therefore make retained
engine state and per-context copying grow without limit by supplying a large
alias slice. This violates the approved implementation checklist's explicit
requirement to bound alias and resolution state. The existing bound test covers
only the resolution cache and does not exercise alias storage.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 2 D1: constructor caller-label and embedded-family metadata are restored
  with `base_db`, and the focused constructor regressions pass.
- Pass 2 D2: caller candidate selection now uses `fontdb::Database::query`, and
  the focused weight, stretch, and style regression passes.
- Correctness: no additional resolution-order, caller-byte selection, or
  fallback defect was found.
- Cache identity: exact ordered aliases participate in retained-work context and
  checked transfer compatibility. Changed aliases invalidate dependent caches,
  while incompatible transfer preserves both engines.
- API contract: the approved default, option-taking, and checked-transfer
  alias-aware facade methods are present. Existing strict and bundled-fallback
  signatures remain unchanged.
- Panics: no new production panic, unchecked indexing, slicing, or arithmetic
  hazard was found.
- Tests: the focused label, face-selection, cache-context, warm and cold, and
  public regression tests pass. No issue beyond the missing alias-bound coverage
  was found.
- OOXML: the diff does not parse or serialize XML, and no schema-order,
  namespace, whitespace, or unmodelled-subtree issue was found.
- Structure: no unjustified trait, generic, wrapper, feature flag, crate,
  module, or file was introduced.

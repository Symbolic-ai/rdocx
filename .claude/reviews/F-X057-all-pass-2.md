# F-X057, all, pass 2

**Reviewed**: `sprint/s56` working tree, 26 files and 502 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, contributor identities are not bound to their credited records

`scripts/test_sprint_workflow.py:4003`
`CHANGELOG.md:92`
`CHANGELOG.md:99`

The release-note truth contract checks only that each contributor handle occurs
at least once. Swapping `@emptinessform` and `@pedroassumpcao` in the two credit
paragraphs still passes the helper and all four focused v0.10.1 tests, while it
credits each contributor for the other person's reports and proposals. That is
a false exact contribution inventory in a publication artifact. Bind each
authenticated handle to its reviewed issue and pull-request set, and add a
rejection mutation that swaps the two attributions.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass-1 D1, current-reality HLD**: resolved. The F-X055 section now records
  the immutable two-package v0.10.0 result, the failed `rdocx-layout`
  verification, the absent GitHub release, and the F-X056 and F-X057 recovery
  ownership without promising the impossible seven-package outcome.
- **Pass-1 D2, direct-landing and partial-inventory truth**: the helper now
  checks the exact recovery, compatibility, and direct-classification
  paragraphs. Dedicated tests reject the two fact-reversing mutations from
  pass 1.
- **Version carriers**: the workspace version, stable pins, lock records,
  binding metadata, WASM literals, CI literal, README requirements, and
  publication flags remain coherent at 0.10.1.
- **Dependency graph and registry proof**: the shared family remains at 0.6.0,
  no prohibited reverse family edge was introduced, and the clean
  `rdocx-layout` package proof leaves `oxml-layout` unpatched.
- **Workflow authority and release safety**: tag selection, approval, failure
  propagation, registry ownership checks, release body comparison,
  notifications, closure authority, and tag authority remain intact.
- **Bindings and package assets**: Python and WASM crates retain publication
  isolation, and no font, licence, ICC, template, or archive path changed.
- **Panics, OOXML, surface, and structure**: no production parsing, arithmetic,
  indexing, XML ordering, namespace, preservation, public API, dependency,
  trait, generic, wrapper, crate, module, or feature behavior changed.

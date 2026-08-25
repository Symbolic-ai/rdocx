# F-X057, all, pass 1

**Reviewed**: `sprint/s56` working tree, 26 files and 334 changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the current-reality HLD still promises the failed v0.10.0 release

`docs/hld/14-development-backlog.md:3182`
`docs/hld/14-development-backlog.md:3218`

The F-X055 section still says to publish all seven stable packages at 0.10.0
and defines success as all seven packages, the GitHub release, and contribution
comments existing at that version. That outcome is now impossible because the
immutable attempt published only `rdocx-opc` and `rdocx-oxml`. The surrounding
current-reality HLD correctly records the partial attempt and the v0.10.1
recovery, so this section contradicts both the repository state and F-X057's
contract not to pretend v0.10.0 completed. Rewrite the archived F-X055 section
to describe its actual partial result and its handoff to F-X056 and F-X057.

### D2, the release-note gate accepts reversed contribution and recovery facts

`scripts/test_sprint_workflow.py:4175`
`scripts/test_sprint_workflow.py:4240`
`CHANGELOG.md:108`

Both v0.10.1 tests use positive substring checks for `hardened equivalent` and
`landed directly`. Changing the reviewed sentence from `No named external
patch landed directly` to an affirmative direct-landing claim preserves the
checked substring and passes both tests. The recovery checks have the same
weakness. For example, appending another package after the checked
`only rdocx-opc and rdocx-oxml` fragment would preserve the assertion while
falsifying the partial-publication inventory. The design plan requires recovery
claims and contribution classifications to remain mutation-tested. Add an
exact release-note contract and rejection tests for fact-reversing mutations
before these notes can gate publication and external notifications.

## Smells

None.

## Nitpicks

None.

## Not found

- **Version carriers**: the workspace version, nine stable pins, eleven
  inherited lock records, Python metadata, WASM literals, CI literal, seven
  stable README requirements, and publication flags agree at 0.10.1.
- **Dependency graph and layering**: the shared family remains at 0.6.0, the
  stable family remains at 0.10.1, and no reverse `oxml-*` to `rdocx-*` or
  `rpptx-*` edge was introduced.
- **Registry proof**: the new dry run verifies `rdocx-layout` with a fresh Cargo
  home, patches only the unpublished stable `rdocx-oxml` edge, and leaves
  `oxml-layout` to resolve from crates.io.
- **Release-note content and live inventory**: all nine selected records occur
  in the rendered section, contributor identities match the authenticated
  authors, PRs 47 through 52 remain open for their authorized post-release
  closure, and the v0.10.0 partial-publication account matches registry and tag
  evidence. The defect above is in mutation enforcement, not the current prose.
- **Workflow authority and release safety**: the diff does not weaken tag
  selection, approval, failure propagation, registry ownership checks, release
  body comparison, notification recording, closure authority, or tag authority.
- **Binding and publication isolation**: Python and WASM carriers move with the
  shared workspace version without gaining crates.io publication eligibility.
- **Package assets and hashes**: no package asset path or source output changed,
  the prepared archive inventory retains the required font, legal, ICC, and
  template assets, and the 49-entry hash harness remains unchanged.
- **Panics, OOXML, and structure**: no production parsing, indexing, arithmetic,
  OOXML ordering, namespace, preservation, trait, generic, wrapper, crate,
  module, feature, or dependency behavior changed.

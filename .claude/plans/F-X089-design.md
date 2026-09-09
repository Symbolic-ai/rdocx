# F-X089, Capability-led README family

**Status**: completed
**Sprint**: S71
**Size**: L
**Depends on**: F-242, F-X009

## Problem

The root README opens with a classification legend and a large table containing
six `partial` rows before it explains why a user should choose the toolkit
(`README.md:18`). Its comparison section reports only what alternatives claim
and does not show the integrated capability advantage that matters to a Rust
user (`README.md:173`). Product status and sprint links receive their own
prominent section (`README.md:186`). The result is accurate but reads like an
internal readiness report rather than the front page of a mature product.

The 26 crate-local READMEs meet the F-X009 inventory contract, but most repeat
the same minimal `Use it when`, `Relationship`, and `Example` skeleton. They do
not foreground the results, capabilities, or technical differentiators that
make each package useful. That undersells both the complete toolkit and the
specialist crates at their crates.io entry points.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, the modern DOCX capability matrix and
  the difference between authoring, reading, rendering, and preservation.
- `docs/hld/03-architecture.md`, workspace families, dependency direction, and
  ownership boundaries.
- `docs/hld/10-bindings-spec.md`, native, CLI, Python, and WASM public surfaces.
- `docs/hld/12-testing-strategy.md`, README compilation and claim validation.
- `docs/hld/14-development-backlog.md`, "F-X089, Capability-led README family".
- `docs/hld/15-build-and-toolchain.md`, docs job and packaged README contract.

## Approach

Rewrite the root README around an outcome-first product story. Lead with the
single-document-object workflow, then show a compact capability table covering
DOCX creation and editing, round-trip preservation, native layout, PDF and PNG
rendering, HTML and Markdown export, CLI, Python, and browser surfaces. Follow
with checked examples and an evidence-backed comparison that makes the breadth
of the integrated Rust toolkit visible. Keep detailed limitations one click
away in the canonical matrix and include a short honest-boundaries section
later in the page. Do not lead with `partial`, `unsupported`, the backlog, or
the active sprint.

Use a dated comparison scope and official sources. A comparison may say that an
alternative does not document a capability only when the approved official
evidence set has been reviewed for that surface. Exclude download counts,
prices, binary sizes, memory use, cold-start timing, popularity, and broad
market superlatives unless a reproducible repository measurement supports them.
Position rdocx strongly as an integrated native Rust document toolkit and make
any uniqueness claim no broader than the reviewed alternatives and date.

Give every crate-local README an outcome-led introduction, three to six
implemented highlights, direct-use guidance, its relationship to adjacent
packages, installation or invocation instructions, and one checked example.
Use family-specific structure for shared `oxml-*`, Word `rdocx-*`, Presentation
`rpptx-*`, command-line, Python, WASM, compatibility-shim, and internal support
packages. Do not duplicate the root comparison table in every crate. Link back
to the root product story where that context helps. Preserve accurate
deprecation, incubation, and publication facts below the value proposition.

Extend the existing README validator rather than create a second checker. Make
the capability-led section contract, all-README local links, official comparison
evidence, package metadata, snippets, compiled examples, and byte-identical
archive contents mutation-sensitive.

## Rejected alternatives

- Restore the old README byte for byte. It contains volatile and unsupported
  footprint, performance, price, and popularity claims.
- Keep the status matrix as the opening section. It is useful specification
  evidence but it is not the product's value proposition.
- Put a competitor table in every crate README. That creates repetitive pages
  and multiplies evidence maintenance without helping specialist consumers.
- Remove honest boundary links. Strong positioning remains credible only when
  readers can inspect the exact supported surface.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_root_readme_leads_with_proven_capabilities` | The front page opens with outcomes and implemented highlights, keeps internal status links below the product story, and carries the scoped comparison. |
| regression | `test_crate_readmes_present_capabilities_and_audience` | All 26 crate pages contain an outcome, highlights, direct-use guidance, relationship, and checked example appropriate to their family. |
| regression | `test_all_readme_local_links_resolve` | Every local README path and anchor stays within the repository and resolves. |
| regression | approved comparison evidence mutation matrix | Every external comparison row and scoped uniqueness statement is backed by the reviewed official source allowlist and rejects volatile claims. |
| integration | `python3 scripts/readme_doctests.py` | All 23 Rust examples compile, non-Rust snippets match public surfaces, versions match package metadata, and the exact 27-package README inventory passes. |
| package | README archive inventory | All 22 publishable archives contain exactly one byte-identical declared README. |
| external | `python3 scripts/readme_doctests.py --check-official-links` | Every approved comparison source resolves during implementation review. |

The **test gate is regression**. The capability-led narrative and scoped
comparison are mutation-sensitive, all examples and snippets validate, all
local links resolve, versions match metadata, and publishable archives carry
the exact reviewed README bytes.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **A new trait, generic parameter, crate, module or file**. The user explicitly
  approved this workflow-required design plan. Production code, tests, and
  documentation checks stay in existing files.

Additional focused checks review only official project and product sources,
date the comparison boundary, run the focused link resolver, and exercise the
existing patched package dry-run plus exact 22-archive README inventory. No
package version or publication authority changes.

## Hash harness

Expected unchanged across all 49 entries. README prose and its validation do not
change generated documents or renders.

## Implementation checklist

- [x] Replace the root status-first opening with an outcome and capability-led product story.
- [x] Add a strong, dated, evidence-backed comparison without volatile claims.
- [x] Keep exact support boundaries available without presenting the README as a sprint tracker.
- [x] Rewrite all 26 crate-local READMEs using family-appropriate capability-led structures.
- [x] Compile all Rust examples and validate CLI, Python, and JavaScript snippets.
- [x] Validate local links across all 27 package READMEs.
- [x] Make narrative, comparison, inventory, metadata, and archive checks mutation-sensitive.
- [x] Run focused official-link checks, package dry-runs, the full gate, and the unchanged hash harness.
- [x] Update exactly the listed HLD files.

## Open questions

None. The user explicitly requested strong capability and comparison messaging
for the root README and similar treatment for every crate README. Comparisons
remain scoped to reviewed evidence, while exact limitations remain linked rather
than leading the page.

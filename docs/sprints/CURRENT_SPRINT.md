# Current Sprint, S60

**Milestone**: M21 Presentation depth.

**Goal**: turn preserved timing XML into deterministic, bounded behavior. The
sprint adds a typed animation and transition model, then evaluates supported
timelines at explicit timestamps without changing ordinary static rendering.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the M21 decision that supersedes
  opaque-only timing while keeping ordinary static rendering independent of
  timeline execution.
- `docs/hld/03-architecture.md`, for the separate PresentationML model,
  inheritance resolver, and renderer layers that the timeline path must cross
  without reversing dependencies.
- `docs/hld/06-presentationml-model.md`, for the current opaque `p:timing` and
  `p:transition` preservation boundary, schema-order rules, and full-corpus
  round-trip contract.
- `docs/hld/08-rendering-spec.md`, for the frozen resolved-slide and page-frame
  seams that timeline evaluation must reuse without changing static output.
- `docs/hld/12-testing-strategy.md`, for deterministic golden output, the
  pinned Presentation corpus, and PowerPoint differential evidence.
- `docs/hld/14-development-backlog.md`, for the F-213 and F-214 acceptance
  contracts, their dependency, and the M21 representative-deck gate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-213 | Animation and transition timing model | L | done | - |
| F-214 | Timeline evaluation and transition rendering | L | done | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-213 owns the typed timing and transition model and must land before F-214 can
evaluate it. F-214 then projects supported timeline state through the existing
resolved-slide and renderer boundaries. The dependency is strict, so the
second story starts only after the first story is integrated and verified.

## Definition of done for this sprint

- Timing nodes, sequences, parallel groups, triggers, supported effects,
  motion paths, transitions, and morph metadata survive mutation, save, and
  reopen in schema order.
- Unsupported timing extensions and siblings remain relationship-complete and
  byte-preserved through the corpus round-trip gate.
- Explicit timestamps produce deterministic frame states whose supported
  entrance, exit, emphasis, motion, transition, and bounded morph behavior
  matches the pinned PowerPoint oracle at the declared tolerances.
- Ordinary static rendering remains unchanged, including all 49 deterministic
  harness entries.
- Full verification passes with every deterministic hash explained and all
  package, portability, documentation, and supply-chain gates green.

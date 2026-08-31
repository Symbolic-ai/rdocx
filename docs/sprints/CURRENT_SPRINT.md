# Current Sprint, S61

**Milestone**: M21 Presentation depth.

**Goal**: complete the media timeline from package relationships to output.
The sprint adds relationship-safe audio and video mutation, deterministic poster
and playback state, and bounded animated GIF and video export. Unsupported
codecs stay packaged, extractable, visible through fallbacks, and diagnosed.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the bounded audio and video policy,
  poster-frame static behavior, and explicit diagnostics for reduced fidelity.
- `docs/hld/03-architecture.md`, for the leaf `oxml-media` boundary and the
  model, layout, renderer, and facade separation used by timeline execution.
- `docs/hld/04-opc-and-packaging.md`, for content-sniffed media identity,
  relationship ownership, content types, deduplication, and atomic package
  mutation.
- `docs/hld/06-presentationml-model.md`, for opaque media preservation,
  relationship-id remapping inside retained XML, and the typed timing model
  that carries playback triggers.
- `docs/hld/08-rendering-spec.md`, for format-neutral page frames, resolved
  media fallbacks, timeline sampling, and the requirement that static entry
  points remain independent.
- `docs/hld/10-bindings-spec.md`, for additive native Rust timeline surfaces
  and the rule that unsupported behavior does not acquire an implicit binding.
- `docs/hld/12-testing-strategy.md`, for the pinned presentation corpus,
  deterministic fonts, source-built oracles, exact artifact identity, and
  declared geometry and pixel boundaries.
- `docs/hld/14-development-backlog.md`, for the F-215, F-216, and F-227
  acceptance gates, their dependency chain, and the still-open M21
  representative-deck gate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-215 | Audio and video package model | L | done | - |
| F-216 | Media poster and playback rendering | M | done | - |
| F-227 | Animated GIF and video export | L | in-progress | codex |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-215 owns media parts, relationships, settings, and playback triggers. F-216
depends on that model and the completed F-214 timeline path before it can
resolve poster frames and synchronized playback state. F-227 then samples the
completed F-214 and F-216 behavior into bounded animated outputs.

## Definition of done for this sprint

- Linked and embedded audio and video survive add, replace, extract, remove,
  save, and reopen with exact relationships, settings, poster ownership, and
  unsupported payload bytes.
- Static poster frames and timestamped playback events match source-built
  golden fixtures without silently decoding an unsupported codec.
- Animated GIF and the approved video backend produce the reviewed frame
  hashes, timestamps, loop behavior, duration, and dimensions on two machines.
- Media failures retain a visible deterministic fallback and stable diagnostic
  rather than dropping content or changing unrelated timeline siblings.
- Full verification passes with every deterministic hash explained, all
  package archives below 10 MiB, and the bounded sprint review clean.

# F-059, a:custGeom

**Status**: approved
**Sprint**: S13
**Size**: M
**Depends on**: F-058

## Problem

F-058 supplies an in-memory guide and path evaluator, but the crate still needs
the XML boundary for `a:custGeom`. No existing source parses adjust lists,
guide lists, text rectangles, or path lists, and the repository currently
contains no deck corpus fixture for the backlog's corpus-shape gate.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Geometry" and "Preservation".
- `docs/hld/08-rendering-spec.md`, "Preset geometry".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-059, a:custGeom".

## Approach

Extend the approved `geometry.rs` module with `CT_CustomGeometry2D` and the
DrawingML structures for `a:avLst`, `a:gdLst`, `a:rect`, `a:pathLst`, and path
commands. Parse local names without relying on prefixes, serialise with the
fixed `a:` prefix, and use `OrderedRawChildren` at every modelled parent so
unknown siblings and subtrees return at the same schema boundary.

Parsed formula strings become the F-058 guide representation. Evaluation uses
the custom geometry's declared path dimensions, guides, adjust values, and
text rectangle, then produces the four-command local evaluated path form. The
test fixture is inline XML, not a committed binary deck.

## Rejected alternatives

- Add a second custom-geometry module. Guide representation, XML model, and
  evaluation form one cohesive geometry boundary and would otherwise require
  readers to follow forwarding types.
- Parse unknown geometry extensions into partial values. Unmodelled XML is
  preserved verbatim instead.
- Add a binary `.pptx` fixture. Repository policy keeps fixtures in code until
  the separately fetched deck corpus exists.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `corpus_custom_geometry_round_trips_and_evaluates_to_a_closed_path` | The backlog test gate for adjust lists, guide lists, text rectangle, path lists, and a final close command |
| regression | `custom_geometry_reads_any_prefix_and_writes_fixed_a_prefix_in_schema_order` | Prefix tolerance and canonical child order |
| regression | `unknown_custom_geometry_children_round_trip_byte_for_byte_in_place` | Raw subtrees survive at their original boundaries |
| regression | `malformed_custom_geometry_returns_an_error_without_panicking` | Missing attributes, bad formulas, and premature EOF are rejected safely |

The test gate is
`corpus_custom_geometry_round_trips_and_evaluates_to_a_closed_path`.

## HLD impact

None. The mechanism and corpus policy are already specified.

## Risk routing

- Any parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Extra checks prove fixed child order,
  prefix-tolerant reads, fixed `a:` writes, and byte-for-byte preservation of
  unmodelled subtrees.

## Hash harness

Expected to be unchanged. The custom geometry model is unpublished and has no
current Word consumer.

## Implementation checklist

- [ ] Add failing inline custom-geometry round-trip, evaluation, raw, and malformed-input tests.
- [ ] Parse and serialise adjust lists, guide lists, text rectangles, and path lists.
- [ ] Convert parsed formula and path values into the F-058 evaluator inputs.
- [ ] Preserve unknown XML at exact schema boundaries and run focused checks.

## Open questions

None. The inline schema-valid custom geometry fixture is approved for F-059,
and the fetched corpus gate remains at the M7 boundary.

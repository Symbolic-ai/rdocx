# F-089, Resolve the preset geometry licensing question

**Status**: approved
**Sprint**: S22
**Size**: S
**Depends on**: none

## Problem

The renderer specification chooses a generated table for all DrawingML preset
geometries, but the provenance gate is still open at
`docs/hld/13-risks-and-open-questions.md:5`. LibreOffice is already rejected
because its table is MPL-2.0, while the ECMA-376 electronic addendum has not
yet been classified for redistribution in this MIT OR Apache-2.0 workspace.
F-090 cannot safely vendor or transform those definitions until this decision
is durable.

The official ECMA-376 fifth-edition Part 1 archive contains
`OfficeOpenXML-DrawingMLGeometries.zip`, whose
`presetShapeDefinitions.xml` has exactly 187 definitions. Ecma's software
policy defines XML data sets as software and makes software incorporated in an
Ecma standard available under its three-clause BSD licence. That permissive
licence is compatible with this workspace when its notice is retained.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Preset geometry".
- `docs/hld/13-risks-and-open-questions.md`, "Q1, preset shape definition
  provenance".
- `docs/hld/15-build-and-toolchain.md`, "Packaging" and "Publishing".

## Approach

Close Q1 by recording that F-090 may use only the official ECMA-376 fifth
edition Part 1 electronic addendum, not a LibreOffice or other implementation
table. Record the exact archive path, upstream URL, 187-definition count,
source SHA-256, Ecma software-policy basis, BSD notice-retention requirement,
and the absence of any change to the repository's MIT OR Apache-2.0 licence.

Update the rendering mechanism to name that source as the chosen input. Keep
the specification-text derivation path as the fallback if the official file or
its required notice cannot be reproduced exactly. This story changes tracked
documentation only and does not vendor the XML or generate code.

## Rejected alternatives

- Use LibreOffice's preset table. MPL-2.0 file-level copyleft is outside the
  repository's approved licensing model.
- Treat the ECMA archive as ordinary specification text. Ecma separately
  classifies XML data sets as software, so the software licence and notice are
  the relevant terms.
- Copy the definitions without a retained BSD notice. That would omit an
  explicit redistribution condition.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `preset_geometry_provenance_is_recorded` | The HLD names the official archive, licence basis, checksum, and notice requirement |
| regression | `libreoffice_preset_table_remains_rejected` | The HLD still rejects MPL-2.0 implementation data |

The backlog test gate is the written decision recorded in the HLD with its
licence basis.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/13-risks-and-open-questions.md`

## Risk routing

none. This is a documentation decision and changes no parser, public API,
dependency, generated artifact, or output.

## Hash harness

Expected to be unchanged. This story changes no executable code or fixture.

## Implementation checklist

- [ ] Record the official fifth-edition Part 1 archive and inner file.
- [ ] Record the 187-definition count and source SHA-256.
- [ ] Record the Ecma software-policy and BSD notice basis.
- [ ] Retain the LibreOffice rejection and specification-text fallback.
- [ ] Run prose and link-oriented documentation checks.

## Open questions

None. The official archive contents and Ecma software policy settle the source,
and the required notice removes the remaining redistribution ambiguity.

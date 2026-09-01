# F-222, ODP read and write

**Status**: completed
**Sprint**: S63
**Size**: L
**Depends on**: F-214, F-215, F-217, F-220

## Problem

The native presentation facade owns editable PresentationML packages, resolved
rendering, notes, charts, media, animation metadata, and supported SmartArt,
but it cannot cross the OpenDocument Presentation boundary. Callers currently
need a second library or an external office process to read or write ODP, which
loses the facade's bounded diagnostics and makes supported fidelity unclear.

ODP is a ZIP package rather than OPC. Reusing `OpcPackage` would invent
relationships and content types that ODF does not have. Retaining a second
public ODP object model would also create two competing presentation models.
The conversion must instead validate a bounded ODF package and project the
declared subset directly to and from the existing `Presentation` owner.

## Spec reference

- `docs/hld/03-architecture.md`, the format-facade ownership rules and the ODT
  conversion precedent.
- `docs/hld/04-opc-and-packaging.md`, bounded non-OPC ZIP validation,
  deterministic package writing, and atomic publication.
- `docs/hld/06-presentationml-model.md`, slide, text, table, image, chart,
  notes, media, animation, and SmartArt ownership.
- `docs/hld/08-rendering-spec.md`, deterministic presentation rendering and
  unsupported-content diagnostics.
- `docs/hld/10-bindings-spec.md`, native facade additions and binding parity.
- `docs/hld/12-testing-strategy.md`, source-built fixtures and pinned external
  differential records.
- `docs/hld/15-build-and-toolchain.md`, pinned LibreOffice and packaging
  requirements.

## Approach

Add one private `crates/rpptx/src/odp.rs` module. This is the smallest readable
boundary for a two-way ZIP parser and writer and is approved by the user's
instruction to proceed with F-222. The module uses the existing workspace
`zip`, `quick-xml`, `sha2`, and media dependencies. It does not enter
`oxml-opc`, add a crate, or retain an ODP object graph.

Expose additive native Rust types and methods:

```rust
pub struct OdpDiagnostic {
    pub path: String,
    pub message: String,
}

pub struct OdpReadResult {
    pub presentation: Presentation,
    pub diagnostics: Vec<OdpDiagnostic>,
}

pub struct OdpWriteResult {
    pub bytes: Vec<u8>,
    pub diagnostics: Vec<OdpDiagnostic>,
}

impl Presentation {
    pub fn from_odp_bytes(bytes: &[u8]) -> Result<OdpReadResult>;
    pub fn from_odp_bytes_with_limits(
        bytes: &[u8],
        limits: PackageReadLimits,
    ) -> Result<OdpReadResult>;
    pub fn open_odp<P: AsRef<Path>>(path: P) -> Result<OdpReadResult>;
    pub fn to_odp_bytes(&self) -> Result<OdpWriteResult>;
    pub fn save_odp<P: AsRef<Path>>(&self, path: P) -> Result<Vec<OdpDiagnostic>>;
}
```

The reader indexes the complete ZIP before XML parsing. It rejects unsafe or
duplicate paths, encryption, unsupported compression, invalid first-entry
`mimetype`, missing required XML, and configured expansion-limit violations.
It resolves ODF content by expanded namespace and projects ordered pages,
ordinary geometry, text, tables, embedded images, speaker notes, and slide
names into a fresh `Presentation`. Charts, transitions, media, animation, and
SmartArt do not have a safe editable cross-format mapping in this story. They
receive stable source-path diagnostics instead of being guessed or silently
dropped.

The writer walks the existing presentation and its package-owned resources. It
emits deterministic ODF 1.3 with stored first `mimetype`, fixed-prefix XML,
stable XML, deterministic ZIP metadata, source-ordered images, and an exact
manifest. It materializes the supported slide, shape, text, table, image, and
notes subset. Lossy chart, transition, media, animation, and SmartArt content
receives stable model-path diagnostics. Output is bounded, source-neutral,
byte-identical across repeated writes, and path saves stage completely before
portable replacement.

Python, WASM, CLI, `rpptx-layout`, and `rpptx-render` gain no ODP entry point.
The public change is additive in the pre-1.0 native `rpptx` facade and requires
release review before publication.

## Rejected alternatives

- Reuse `OpcPackage`. ODP has no OPC relationships or content types.
- Add a public ODP model. It would duplicate the presentation owner and expand
  the supported surface beyond the story.
- Shell out to LibreOffice in production. External tools are oracle-only and
  cannot provide deterministic bounded library behavior.
- Put the conversion in `rpptx/src/lib.rs`. The two-way package boundary is
  large enough that doing so would make the facade harder to audit.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `odp_reader_and_writer_match_pinned_libreoffice_both_directions` | Source-built ODP and PPTX conversions match the pinned LibreOffice structural and render records in both directions. |
| integration | `odp_round_trip_preserves_supported_presentation_content` | Slides, geometry, text, tables, images, and notes survive ODP write and read. |
| package | `odp_packages_are_bounded_deterministic_and_manifest_complete` | First stored mimetype, exact manifest, fixed metadata, duplicate and unsafe path rejection, caller limits, and repeated bytes are stable. |
| diagnostics | `odp_lossy_diagnostics_are_stable_bounded_and_location_aware` | Unsupported safe content reports stable ordered paths and the diagnostic ceiling fails closed. |
| mutation | `failed_odp_read_and_save_never_publish_partial_state` | Read failures publish no presentation and save failures do not truncate an existing destination. |

The required test gate is
`odp_reader_and_writer_match_pinned_libreoffice_both_directions`.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Parser and serialiser**: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add expanded-name parsing, fixed-prefix
  schema-order writing, bounded ZIP tests, and byte-preservation checks at the
  PresentationML boundary.
- **Layout, text, and rendering**: read `docs/hld/08-rendering-spec.md`. Use
  deterministic fonts for every Rust baseline and declare any hash delta.
- **Public API**: read `docs/hld/10-bindings-spec.md` and the structural rules.
  State additive pre-1.0 impact, run rustdoc with warnings denied, patched
  publish dry-runs, and the 10 MiB archive assertion.
- **New module**: read the structural rules. The existing implementations are
  the ODP reader and ODP writer in the same private format boundary. No trait,
  generic, wrapper, feature, or additional file is introduced.
- **External oracle**: apply `.claude/skills/differential-testing.md`. Pin the
  exact LibreOffice build, use source-built inputs, compare normalized semantic
  and render outputs, and prove sensitivity.

## Hash harness

Expected to remain unchanged. ODP methods do not change existing PPTX or
rendering entry points.

## Implementation checklist

- [x] Add real failing integration stubs and record the 49-entry baseline.
- [x] Implement bounded ODP archive indexing, namespace-aware XML projection,
  stable diagnostics, and fresh Presentation construction.
- [x] Implement deterministic ODF 1.3 serialization, media and manifest
  ownership, output bounds, and atomic path publication.
- [x] Cover slides, geometry, text, tables, images, and notes in both
  directions, with stable diagnostics for the declared lossy categories.
- [x] Run the pinned two-direction LibreOffice differential and sensitivity
  checks.
- [x] Update exactly the listed HLD files, run all routed riders, and complete
  with a zero-defect, zero-smell microscope pass.

## Open questions

None. The story fixes the native-only surface, non-OPC ownership, supported
content categories, stable diagnostics, and the pinned LibreOffice gate. The
existing ODT boundary supplies the repository conventions for limits,
deterministic ZIP output, and atomic path replacement.

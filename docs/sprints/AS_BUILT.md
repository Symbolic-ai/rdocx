# As Built

Append-only completion log. One entry per F-ID, written by `/complete-feature`
at the moment of completion, describing what was actually built rather than what
was planned.

Entries are never edited after the fact. When a later story changes something
recorded here, the later story gets its own entry. The design intent lives in
`docs/hld/`, the plan lives in `.claude/plans/F-XXX-design.md`, and this file is
the record of what happened.

Newest entries at the bottom.

## Entry template

```markdown
### F-XXX, Short title

**Sprint.** SNN
**Completed.** YYYY-MM-DD
**Size.** S | M | L, estimated N days, actual N days

**What was built.** One paragraph. What exists now that did not before, in terms
a reader who has not seen the diff can follow.

**Non-obvious choices.** Anything a future reader would otherwise have to
reverse-engineer from the code, and the reason for it. Rejected alternatives
belong here, not in a comment.

**Deviations from the design plan.** What changed between
`.claude/plans/F-XXX-design.md` and the implementation, and why. "None" is a
valid and common answer.

**Spec sections touched.** The `docs/hld/` sections this story implements or
contradicts. If it contradicts one, say which and confirm the spec was updated.

**Tests.** The test gate from `docs/hld/14-development-backlog.md`, plus any
others added. Name them.

**Hash harness.** Unchanged, or the expected delta and its justification.
Mandatory for every story in M1 through M6.

**Notes for future sessions.** Anything that will not be obvious in three
months. Traps found, assumptions made, follow-up worth filing.
```

## Entries

### F-001, Deterministic font mode

**Sprint.** S01
**Completed.** 2026-07-29
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `FontManager`, the layout engine, and `Document` now expose
an explicit rendering path that loads checked-in bundled fonts and never
discovers host fonts. The existing `bundled-fonts` feature is default-on for
the current `rdocx` consumer.

**Non-obvious choices.** Determinism is explicit rather than ambient. Normal
library rendering keeps its system-font path, while deterministic rendering
returns a clear error when bundled fonts are disabled.

**Deviations from the design plan.** The plan was revised to correct the
manifest's missing default declaration after implementation discovery showed
that the code and HLD already described bundled fonts as default-on. Microscope
pass 1 also strengthened the golden gate to inspect the actual resolved font
bytes rather than compare two calls under one environment.

**Spec sections touched.** `docs/hld/15-build-and-toolchain.md`, "Deterministic
rendering" and "Feature flags".

**Tests.** `deterministic_font_manager_uses_only_bundled_fonts`,
`deterministic_font_manager_requires_bundled_fonts`, and
`deterministic_render_is_independent_of_system_fonts`.

**Hash harness.** Unchanged. F-003 recorded the first baseline after this path
was integrated.

**Notes for future sessions.** The end-to-end test verifies every font buffer
used by the inspected layout belongs to the checked-in bundled set.

### F-002, rust-toolchain.toml

**Sprint.** S01
**Completed.** 2026-07-29
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The repository now selects Rust 1.97.1 with `rustfmt`,
`clippy`, and `wasm32-unknown-unknown` through `rust-toolchain.toml`.

**Non-obvious choices.** The workspace and CI MSRV declarations remain 1.93.
The development toolchain and the compatibility floor answer different
questions.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/15-build-and-toolchain.md`, "Toolchain
pinning".

**Tests.** `rustup show active-toolchain`, installed component and target
inspection, and confirmation of every 1.93 MSRV declaration.

**Hash harness.** Unchanged.

**Notes for future sessions.** Rustup may synchronize channel metadata before
reporting the repository override, even when the toolchain is installed.

### F-003, Output-stability hash harness

**Sprint.** S01
**Completed.** 2026-07-29
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `scripts/hash_harness.py` regenerates seven samples and
compares SHA-256 values for three OOXML parts and deterministic page-one PNG
output at 150 dpi. Check mode is read-only, while update mode requires a
non-empty reason.

**Non-obvious choices.** Missing optional parts are recorded as JSON `null`
rather than omitted. The baseline has 28 sorted entries, so additions,
removals, and byte changes are reported separately.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/12-testing-strategy.md`, "The hash
harness".

**Tests.** Python comparison and reason-refusal unit tests,
`python3 scripts/hash_harness.py --check`, and a temporary writer whitespace
injection that left the structural round-trip test green while changing all
seven `document.xml` digests.

**Hash harness.** Expected initial delta. Added 28 entries with reason
`F-003 initial deterministic baseline`. Manifest SHA-256 is
`9a3c64d61df793b9d8f7203df9cb966fb67201518b4f7fc0f2e68d276aaaca8f`.

**Notes for future sessions.** `invoice` has no `word/numbering.xml`, which is
the single explicit null entry in the initial baseline.

### F-004, Caladea licence and the false OFL claim

**Sprint.** S01
**Completed.** 2026-07-29
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The `rdocx-layout` package now carries the Apache-2.0
licence and Caladea notice beside the four TTFs. Bundled-font documentation
names the correct licence per family, and a test enforces licence coverage.

**Non-obvious choices.** Attribution files live under the crate's `fonts/`
directory so they are included in the published archive with the assets they
cover.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/13-risks-and-open-questions.md`, "Known
defects being carried", and `docs/hld/15-build-and-toolchain.md`, "Packaging".

**Tests.** `every_bundled_font_family_has_a_licence_file`, the full
`rdocx-layout` suite, upstream TTF provenance checks, and the package file list.

**Hash harness.** Unchanged.

**Notes for future sessions.** The checked-in Caladea files match the
`crosextrafonts-20130214` source archive, and the notice fields match embedded
TTF metadata.

### F-005, Fix the image counter

**Sprint.** S01
**Completed.** 2026-07-29
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Imported media names now seed allocation from the greatest
positive numeric suffix rather than the part count. Allocation avoids existing
suffixes and remains collision-free at the finite `usize` boundary.

**Non-obvious choices.** When the greatest suffix cannot be incremented, the
counter wraps to one and skips occupied suffixes until it finds a free name.
Ordinary packages still allocate exactly maximum plus one.

**Deviations from the design plan.** Microscope passes 1 and 2 exposed overflow
and overwrite cases at `usize::MAX`. The plan was clarified to add checked
wrapping and occupied-suffix skipping without adding a media-namer abstraction.

**Spec sections touched.** `docs/hld/04-opc-and-packaging.md`, "Part naming",
and `docs/hld/13-risks-and-open-questions.md`, "Known defects being carried".

**Tests.** `next_image_name_uses_the_highest_existing_index_not_the_part_count`,
`malformed_media_names_do_not_change_the_highest_image_index`,
`occupied_max_image_suffix_wraps_to_a_free_low_number`, and
`max_minus_one_allocates_max_then_wraps_safely`.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Suffix parsing is extension-independent and
reads only consecutive ASCII digits immediately after `image`.

### F-006, Fix the JPEG standalone-marker walk

**Sprint.** S01
**Completed.** 2026-07-29
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The JPEG dimension walk now handles SOI, TEM, and restart
markers without reading nonexistent lengths, validates every length-bearing
segment, tolerates marker fill bytes, and terminates at EOI.

**Non-obvious choices.** The parser remains a small header walk because PDF
output passes JPEG bytes through unchanged and needs only dimensions.

**Deviations from the design plan.** Microscope pass 1 found that EOI was being
skipped like a restart marker. Pass 2 verified immediate termination and the
new trailing-data regression.

**Spec sections touched.** `docs/hld/04-opc-and-packaging.md`, "Media", and
`docs/hld/13-risks-and-open-questions.md`, "Known defects being carried".

**Tests.** `jpeg_restart_marker_before_sof_preserves_dimensions`,
`every_truncated_jpeg_header_returns_without_panicking`, and
`jpeg_bytes_after_eoi_cannot_supply_dimensions`.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** SOF still has to appear before EOI. Trailing
bytes after a completed JPEG cannot supply dimensions.

### F-007, Resolve core properties through the relationship

**Sprint.** S02
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Document metadata now resolves through the package-level
core-properties relationship, preserves a custom part target across load and
save, and creates the conventional target only when the relationship is
missing. `rdocx-opc` exposes the standard relationship type publicly.

**Non-obvious choices.** The facade retains a private copy of the stable
relationship URI so the `rdocx 0.3.0` package can still verify against the
published `rdocx-opc 0.3.0` dependency before both move to 0.4.1.

**Deviations from the design plan.** The full packaging gate exposed the
published-dependency compatibility issue after workspace tests passed. An
independent microscope pass approved the private URI because both integration
gates cross-check it against the public constant.

**Spec sections touched.** `docs/hld/04-opc-and-packaging.md`, "Relationship
types" and "Part naming".

**Tests.** `core_properties_at_relationship_target_round_trip_in_place`,
`metadata_round_trip`, focused rdocx and OPC suites, and the clean package
dry-run.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** A non-standard target is authoritative. Saving
must not create an orphaned `/docProps/core.xml` part.

### F-008, Non-consuming setter twins

**Sprint.** S02
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** All 61 consuming builders across `Paragraph`, `Run`,
`Table`, `Row`, and `Cell` now delegate to non-consuming `set_*` twins.

**Non-obvious choices.** Action builders receive literal `set_*` names as the
story required. Existing builder names and chaining behavior remain unchanged.

**Deviations from the design plan.** The backlog's paragraph-level bold gate
was corrected to obtain a `Run`, where bold formatting belongs. Integration
with F-007 required retaining two independent additions to the shared test
file, followed by a clean microscope pass.

**Spec sections touched.** `docs/hld/03-architecture.md`, "Facade conventions",
`docs/hld/10-bindings-spec.md`, "Two supporting decisions", and
`docs/hld/14-development-backlog.md`, "F-008, Non-consuming setter twins (M)".

**Tests.** `non_consuming_setters_mutate_borrowed_wrappers` and
`non_consuming_setters_match_consuming_builders`, plus all 68 integrated rdocx
integration tests.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep mutation bodies in the in-place setters so
builder and binding behavior remain single-sourced.

### F-009, Cache the layout result

**Sprint.** S02
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `Document` caches normal and deterministic layout results
in separate thread-safe slots, exposes cloned page layout access, and clears
both caches across direct mutations and mutable-accessor paths.

**Non-obvious choices.** `Mutex<Option<Arc<LayoutResult>>>` preserves the
`Document: Send + Sync` binding contract. Caller-supplied font layouts remain
uncached because their inputs are not part of a stable document cache key.

**Deviations from the design plan.** None. The approved plan had already
replaced the backlog's thread-local `RefCell<Option<Rc<_>>>` proposal.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`, "Performance",
`docs/hld/10-bindings-spec.md`, "Two supporting decisions",
`docs/hld/13-risks-and-open-questions.md`, "Known defects being carried", and
`docs/hld/14-development-backlog.md`, "F-009, Cache the layout result (M)".

**Tests.** `rendering_all_pages_performs_one_layout`,
`document_mutation_invalidates_cached_layout`,
`mutable_accessor_invalidates_cached_layout`,
`font_modes_use_isolated_layout_caches`, and
`document_remains_send_and_sync`.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Any new mutable accessor must invalidate both
layout modes before returning the borrow.

### F-010, Reserve crate names

**Sprint.** S02
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Fourteen approved `oxml-*` and `rpptx*` names were
published as dependency-free `0.0.0` placeholders and verified as owned by
`mantissaman`.

**Non-obvious choices.** Python and wasm binding names were excluded because
their documented distribution channels are PyPI and npm. Publications ran
sequentially through crates.io's rolling new-crate rate limit.

**Deviations from the design plan.** None. The registry required repeated
cooldown windows, and the workflow stopped after every HTTP 429 before
resuming at the exact rejected name.

**Spec sections touched.** `docs/hld/13-risks-and-open-questions.md`,
"Q2, PyPI name availability", and `docs/hld/15-build-and-toolchain.md`,
"Publishing".

**Tests.** Exact `cargo info <name>@0.0.0` and owner checks for all fourteen
names, package inspection, publish dry-runs, and archive-size checks.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** The placeholders reserve names only. They expose
no implementation API and do not change any existing `rdocx 0.3.0` crate.

### F-011, Pin unit truncation behaviour

**Sprint.** S02
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Fractional positive and negative tests now pin truncation
toward zero for every float constructor on `Length`, `Twips`, and `Emu`.

**Non-obvious choices.** The vectors cross the half-unit boundary so temporary
rounding mutations fail while the existing production casts remain unchanged.

**Deviations from the design plan.** Microscope pass 1 corrected one invalid
HLD heading citation. Pass 2 found no defects or smells.

**Spec sections touched.** `docs/hld/11-migration-plan.md`, "Preserve
behaviour, do not improve it", and `docs/hld/12-testing-strategy.md`, "New
tests the extracted crates need".

**Tests.** `length_float_constructors_truncate_toward_zero`,
`twips_float_constructors_truncate_toward_zero`, and
`emu_float_constructors_truncate_toward_zero`, including temporary rounding
mutations that made every gate fail.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** A change from casts to rounding is a behavior
change even when whole-unit conversion tests continue to pass.

### F-012, Tag v0.4.1

**Sprint.** S02
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The workspace was published as seven lockstep rdocx crates
at 0.4.1 from the reviewed S02 SHA. A dedicated `/release` command now owns
`v*` tags and publication, while the tag workflow verifies the deterministic
hash baseline and publishes only the approved rdocx allowlist.

**Non-obvious choices.** The published `v0.4.0` mainline was merged into S02
before release, preserving its contract changes and retargeting the planned
0.3.1 release to 0.4.1. The fourteen `oxml-*` and `rpptx*` placeholders remain
at 0.0.0 until PowerPoint development is complete.

**Deviations from the design plan.** The original plan targeted 0.3.1 before
the separate 0.4.0 release appeared. The reconciled plan and release evidence
target 0.4.1. The publication workflow retains deliberate registry-index waits
because real publication is explicitly allowlisted instead of workspace-wide.

**Spec sections touched.** `docs/hld/11-migration-plan.md`, release boundary,
`docs/hld/13-risks-and-open-questions.md`, release risks,
`docs/hld/14-development-backlog.md`, M1 gate and F-012, and
`docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".

**Tests.** `/verify --full` passed at
`6e02a4b6417c9bb0c245237bdf8168dd06310c39`. The package dry-run produced
exactly seven archives below 10 MiB, including all 20 TTFs and required licence
files in `rdocx-layout`. GitHub Actions run 30522998328 passed, every exact
`cargo info <crate>@0.4.1` lookup succeeded, all owners were `mantissaman`, and
the GitHub release tag peeled to the reviewed SHA.

**Hash harness.** Unchanged. All 28 entries matched locally and on the Linux
publication runner.

**Notes for future sessions.** The release workflow must remain restricted to
the seven rdocx crates until PowerPoint development is complete. After S02 is
merged, forward-merge `main` into `feature/release-0.5.0` before that release
branch continues.

### F-013, Create oxml-core

**Sprint.** S03
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** A new private `oxml-core` workspace crate now owns staged
copies of the format-neutral error, units, raw XML, XML text, core properties,
and `Length` implementations. It also provides shared namespace-aware XML
helpers and public XML text handling with focused event coverage.

**Non-obvious choices.** The crate remains at 0.0.0 with `publish = false`.
The existing Word implementations stay in place until F-015 and F-016 can
switch the facades without putting an unpublished dependency into a published
rdocx package.

**Deviations from the design plan.** None. The approved plan already specified
the staged copy and delayed facade switch.

**Spec sections touched.** `docs/hld/03-architecture.md`, "Three families, one
workspace", `docs/hld/11-migration-plan.md`, "The facade trick" and "Order of
operations", and `docs/hld/15-build-and-toolchain.md`, "Publishing".

**Tests.** The moved unit, raw XML, core properties, and XML text tests,
`xml_text_handles_cdata_mixed_nested_and_general_refs`, workspace compilation,
package verification, and the dependency-tree direction check.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Do not publish this crate or connect published
rdocx packages to it until the PowerPoint development publication boundary is
explicitly lifted.

### F-014, New unit types

**Sprint.** S03
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml_core::units` now exposes `Centipoints`, `Angle`, and
`Percent1000` with the exact OOXML storage scales. `Length::mm` adds direct
millimetre construction through 36,000 EMUs per millimetre.

**Non-obvious choices.** Float conversions retain Rust cast semantics and
truncate positive and negative fractional values toward zero. No generic unit
abstraction or `Length::to_mm` accessor was added.

**Deviations from the design plan.** None.

**Spec sections touched.** None. The existing glossary and DrawingML model
already specify the implemented types and scales.

**Tests.** `centipoints_round_trip_points`, `angle_round_trip_degrees`,
`percent1000_round_trip_percent`,
`new_unit_float_constructors_truncate_toward_zero`, and
`length_millimetres_round_trip`.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Preserve the pinned truncation rule when these
types gain format consumers.

### F-017, App and custom properties

**Sprint.** S03
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-core` now provides a shared application-properties
union for Word and PowerPoint plus typed custom properties for text, signed
integers, floating point values, Booleans, file times, and empty values.
Parsers preserve child order and retain unsupported property subtrees as raw
XML.

**Non-obvious choices.** Parsed application children replay their encountered
order, while constructed values use canonical schema order. Unsupported custom
value types remain raw XML instead of being coerced to strings.

**Deviations from the design plan.** Microscope passes 1 and 2 found malformed
root acceptance and inconsistent self-closing typed values. Both were fixed
with regression tests before the clean pass 3.

**Spec sections touched.** None. The existing scope, architecture, and testing
documents already describe the shared model.

**Tests.** `word_app_properties_round_trip_without_presentation_fields`,
`powerpoint_app_properties_round_trip_without_word_fields`,
`unknown_app_property_subtree_is_preserved_verbatim`,
`custom_property_value_types_round_trip`,
`unknown_custom_property_value_is_preserved_verbatim`, and malformed-root and
self-closing-value regressions.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep unknown XML preservation and schema child
order intact when adding further extended-property variants.

### F-018, Create oxml-opc

**Sprint.** S04
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** A new private `oxml-opc` workspace crate now owns a staged
copy of the format-neutral OPC package, content-type, relationship, and error
implementation. Generic `OpcPackage::new`, `OpcPackage::with_main_part`, and
`ContentTypes::minimal` replace the DOCX-specific public constructors in the
new crate.

**Non-obvious choices.** The crate remains at 0.0.0 with `publish = false` and
depends only on `quick-xml`, `thiserror`, and `zip`. The existing `rdocx-opc`
implementation and every released rdocx consumer remain untouched until the
real shared crates have an approved publication path.

**Deviations from the design plan.** None.

**Spec sections touched.** None. The existing architecture, OPC, migration,
testing, and build specifications already describe the staged extraction.

**Tests.** Eleven moved OPC tests rebuilt around private DOCX helpers,
`minimal_content_types_contain_only_universal_defaults`,
`with_main_part_resolves_and_round_trips`, independent dependency inspection,
local package verification, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep `rdocx-opc` and released consumers on the
published implementation until the deferred F-022 cutover is packageable.

### F-019, PresentationML relationship and content types

**Sprint.** S04
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `oxml-opc` now exposes package, shared-property, and
PresentationML relationship constants plus universal, shared-property, and
PresentationML content-type constants. The generic minimal constructor reuses
the universal values.

**Non-obvious choices.** Word-specific MIME constants remain outside this
story. The table-driven gate lists every public value and asserts uniqueness,
the correct relationship namespace, the `application` MIME top-level type,
and the absence of whitespace or extra slashes.

**Deviations from the design plan.** Microscope pass 1 found that the initial
MIME-shape assertion accepted arbitrary top-level types and extra slashes. The
gate was tightened before clean pass 2.

**Spec sections touched.** None. The existing OPC and PresentationML
specifications already enumerate the required constants.

**Tests.** `relationship_and_content_type_constants_are_unique_and_well_formed`,
all-target compilation, a 12,155-byte integrated local package, dependency
inspection, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Add future package constants to the exhaustive
table so a new value cannot bypass namespace and MIME-shape classification.

### F-020, oxml-opc reads a pptx

**Sprint.** S04
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** A code-built PowerPoint package fixture writes and reopens
a presentation, slide, and slide-layout graph entirely in memory. It proves
main-part discovery, relationship round-tripping, normalized package keys, and
parent-directory layout resolution.

**Non-obvious choices.** The fixture lives in the existing `package.rs` test
module. It adds no binary fixture, integration-test target, dependency,
production API, or production-code change.

**Deviations from the design plan.** Sprint review pass 1 required a second
fixture built directly as a valid PresentationML ZIP, independent of
`OpcPackage::write_to`, so the M2 real-package gate does not rely on a
self-round-trip.

**Spec sections touched.** None. The existing OPC and testing specifications
already require this exact package graph.

**Tests.** `pptx_package_resolves_main_slide_and_layout_parts`,
`presentation_layout_target_resolves_one_directory_up`,
`independently_built_pptx_opens_and_resolves_relationships`, all integrated
`oxml-opc` tests, and the integrated full gate. The original two named tests
were observed failing for their intended reasons before completion.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** OPC navigation is now proven against both Word
and PowerPoint package shapes before any PresentationML parser is introduced.
The close workflow preserves the S32.2 target for publication-bound cutovers
instead of forcing them through every intervening sprint.

### F-021, Zip-slip hardening tests

**Sprint.** S04
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The `oxml-opc` reader now normalizes ZIP entry names before
classifying package metadata and parts. Root-escaping traversal clamps to the
package root, and absolute entries become canonical leading-slash part names.

**Non-obvious choices.** Normalization uses OPC package-path algebra rather
than host filesystem canonicalization, and no archive entry is extracted to
the filesystem. The released `rdocx-opc` reader remains unchanged.

**Deviations from the design plan.** None.

**Spec sections touched.** None. The existing OPC and testing specifications
already require canonical package names and both hostile cases.

**Tests.** `zip_entry_that_escapes_root_is_clamped_to_root`,
`absolute_zip_entry_is_normalized_to_package_root`, the opaque package
round-trip, deterministic save, all integrated OPC parser tests, and the full
gate. Both hostile cases were observed failing before the normalization fix.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Preserve package-path normalization before raw
metadata classification when the reader gains further ZIP validation.

### F-023, oxml-media format sniffing

**Sprint.** S05
**Completed.** 2026-07-30
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The new dependency-free `oxml-media` crate classifies PNG,
JPEG, GIF, BMP, TIFF, WebP, SVG, EMF, and WMF from bytes. It exposes canonical
extension and content-type mappings plus sniff-first resolution with extension
fallback and a PNG default.

**Non-obvious choices.** The staged crate remains at version 0.0.0 with
publication disabled. Released rdocx consumers still use their existing image
paths until the deferred cutover after PowerPoint development.

**Deviations from the design plan.** Microscope review tightened SVG prolog
handling and standard WMF recognition before the final clean pass.

**Spec sections touched.** None. The existing media and migration contracts
already specify the staged crate and sniffing precedence.

**Tests.** Magic-byte coverage for every format, canonical mappings,
misleading-extension precedence, unknown-image fallback, every-prefix safety,
the dependency and package riders, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep format detection byte-led when the released
consumers move to `oxml-media`.

### F-024, Image probing and DPI

**Sprint.** S05
**Completed.** 2026-07-30
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `oxml-media` now probes pixel dimensions, optional DPI,
bit depth, channel count, and alpha metadata from bounded PNG, JPEG, GIF, BMP,
and WebP headers.

**Non-obvious choices.** All fixtures are constructed in code. Unsupported,
inconsistent, and truncated headers return `None`, and every indexed read is
bounds checked without adding a decoder dependency.

**Deviations from the design plan.** Microscope review found and drove fixes
for BMP mask placement and alpha classification plus WebP frame, profile,
canvas, RIFF-padding, and parity validation before clean pass 4.

**Spec sections touched.** None. The implementation follows the existing
binary-header parsing contract.

**Tests.** PNG `pHYs`, JFIF density units, EXIF before progressive SOF, GIF,
BMP, all WebP layouts, every-prefix truncation, 22 integrated media tests, and
the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Preserve bounded parsing and the truncation
loops whenever another header layout is accepted.

### F-025, MediaNamer

**Sprint.** S05
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `MediaNamer` scans exact directory and stem matches for
positive numeric suffixes, allocates after the maximum, and wraps safely after
`usize::MAX` without emitting zero or reusing an occupied name.

**Non-obvious choices.** The caller's extension is preserved verbatim. The
allocator retains occupied suffixes so integer wrap can search safely from one.

**Deviations from the design plan.** Microscope pass 1 found that root package
parts were not recognized. The fix normalizes the empty root split back to `/`
and adds root, trailing-slash, and non-PNG regression coverage.

**Spec sections touched.** None. The existing media contract already requires
maximum-suffix allocation.

**Tests.** All four F-005 sentence-named regressions, root and directory
normalization, caller extension handling, 22 integrated media tests, and the
integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** The released rdocx allocator remains active
until the deferred consumer cutover.

### F-026, native_size with explicit DPI

**Sprint.** S05
**Completed.** 2026-07-30
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `ImageInfo::native_size` returns dependency-free
`NativeSize` values with explicit width and height EMU fields. Each axis uses
finite positive declared DPI when available and otherwise uses the caller's
finite positive default.

**Non-obvious choices.** Pixel conversion uses 914400 EMU per inch and
truncates toward zero. Invalid effective DPI or dimensions outside the `i64`
range return `None`.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/03-architecture.md`,
`docs/hld/04-opc-and-packaging.md`, and
`docs/hld/14-development-backlog.md` now state the dependency-free return type.

**Tests.** Declared-DPI precedence, independent per-axis fallback, fractional
truncation, invalid input, dependency and package riders, 22 integrated media
tests, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** The later consumer cutover must supply its own
default DPI and convert no units outside this API.

### F-029, Create oxml-layout

**Sprint.** S06
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The unpublished `oxml-layout` crate now stages
format-neutral layout output, font management, bundled deterministic fonts,
and layout errors without changing a released consumer.

**Non-obvious choices.** Bundled fonts are always available for deterministic
rendering. The default `system-fonts` feature controls only host font discovery,
and the no-default path keeps the same bundled archive.

**Deviations from the design plan.** Sprint review pass 1 found that two HLD
passages still described an older bundled-fonts-off no-default path. The
implementation followed the intended boundary, and the stale current-intent
wording was corrected during sprint remediation.

**Spec sections touched.** `docs/hld/12-testing-strategy.md` and
`docs/hld/15-build-and-toolchain.md` now state that the no-default path disables
system discovery while retaining bundled deterministic fonts.

**Tests.** Default and no-default font-manager tests, empty-font error handling,
bundled-font licence coverage, dependency isolation, archive contents and size,
released-crate isolation, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep deterministic fonts inside `oxml-layout`
when consumers migrate. System discovery remains an optional capability, not a
determinism requirement.

### F-030, Decouple line.rs

**Sprint.** S06
**Completed.** 2026-07-31
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `oxml-layout` now has a greedy line breaker with owned
layout types for alignment, tabs, leaders, underline, and spacing. Explicit
wrapping control can retain width overflow while forced line, page, and column
breaks continue to split content.

**Non-obvious choices.** Tab positions and exact or minimum spacing are stored
in points. Multiple spacing stores a factor, and the staged boundary contains
neither twips nor stringly line rules.

**Deviations from the design plan.** Microscope pass 1 strengthened the copied
tests to use deterministic bundled fonts and made the leader regression prove
real glyph shaping. Production behavior remained on the approved design.

**Spec sections touched.** None. The existing migration and rendering contracts
already define the owned line-breaking boundary and deferred rdocx converter.

**Tests.** All 11 copied compatibility tests, four owned spacing and wrapping
regressions, deterministic leader shaping, both feature modes, dependency and
package riders, released-line-breaker isolation, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** The later rdocx cutover must perform schema-unit
conversion before constructing these types. Do not move Word-specific enums
back across this boundary.

### F-031, Transform

**Sprint.** S06
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-layout` now exports a concrete six-coefficient affine
transform with rotation about a point, self-first composition, point
application, exact identity checks, and four-corner rectangle bounds.

**Non-obvious choices.** `self.then(next)` applies `self` first and `next`
second, matching the point equations and PDF `cm` concatenation order. Identity
comparison is exact so small intentional transforms are never discarded.

**Deviations from the design plan.** Microscope pass 1 strengthened the
composition gate with fully nonzero matrices and replaced an exact quarter-turn
bounds case with a negative 30-degree rotation. Production algebra was already
correct.

**Spec sections touched.** None. The existing rendering and testing contracts
already specify the matrix representation and composition order.

**Tests.** Identity neutrality, fractional and positive rotation, fully nonzero
hand-computed PDF composition, negative-rotation four-corner bounds, exact
identity, package and dependency riders, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Convert DrawingML rotation units to degrees
before this boundary, and preserve self-first composition when group transforms
begin consuming the type.

### F-032, Path and PathCommand

**Sprint.** S07
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-layout` now exports backend-neutral path commands,
fill rules, conservative bounds, and constructors for rectangles, rounded
rectangles, and four-cubic ellipses.

**Non-obvious choices.** Bounds include cubic control points without solving
curve extrema. Rounded rectangles use one circular radius clamped from zero to
half the shorter side.

**Deviations from the design plan.** None.

**Spec sections touched.** None. The existing rendering contract already
defines the path representation and conservative bounds.

**Tests.** Ellipse bounds within the control hull, cubic control-point bounds,
empty paths, fill-rule independence, closure, and radius clamping.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Backend path conversion must preserve command
order and choose the fill operator from `FillRule`.

### F-033, Paint and Stroke

**Sprint.** S07
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-layout` now exports solid, linear, radial, and tiled
paint plus gradient stops, line caps, line joins, and arbitrary dash arrays for
strokes.

**Non-obvious choices.** Only a one-stop gradient is normalized at
construction, becoming solid paint. Empty and multi-stop gradients remain
unchanged for the later backend normalization stage.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/14-development-backlog.md` now records
F-036 as an explicit dependency because tiled paint stores `MediaId`.

**Tests.** Single-stop linear and radial degradation, multi-stop preservation,
stroke defaults, tile media identity, package, dependency, and feature-mode
checks.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Sorting, clamping, and duplicate-stop precedence
remain backend construction work and must not be moved silently into this
model.

### F-034, Path and Group arms

**Sprint.** S07
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The staged positioned-element model now carries painted
paths and nested groups with transforms, clips, opacity, effects, and children.
Page backgrounds, layout diagnostics, and constructors for the two
non-exhaustive result structs are also available.

**Non-obvious choices.** `Diagnostic` initially carries one message.
`PositionedElement` and `Effect` are non-exhaustive enums, while `PageFrame`
and `LayoutResult` are non-exhaustive structs with neutral constructor defaults.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/08-rendering-spec.md` and
`docs/hld/14-development-backlog.md` now state the exact non-exhaustive targets,
minimal diagnostic shape, constructor contract, and unpublished staging
boundary.

**Tests.** Path and group payload preservation, transform direction, neutral
page and result defaults, external constructor doctests, and the integrated
full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Group transforms map child-local coordinates
into their parent. Preserve that direction in every recursive backend.

### F-035, The walk helper

**Sprint.** S07
**Completed.** 2026-07-31
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `oxml-layout::walk` visits every non-group element once in
depth-first document order while carrying its accumulated child-to-page
transform.

**Non-obvious choices.** Group containers are not yielded. With self-first
composition, each group transform is composed before the accumulated parent
transform.

**Deviations from the design plan.** None.

**Spec sections touched.** None. The existing recursion-hazard contract already
defines the helper and its consumers.

**Tests.** Three-deep traversal with three ordered leaves, hand-computed nested
transform order, group exclusion, and root identity.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Font, image, and link collection passes must use
this helper when the PDF backend migrates.

### F-036, MediaId

**Sprint.** S07
**Completed.** 2026-07-31
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Staged output and line image values now use a stable
content-addressed `MediaId` derived from raw bytes instead of relationship-local
embed identifiers.

**Non-obvious choices.** The compact handle uses fixed 64-bit FNV-1a and is
documented as a renderer key rather than a collision-free content guarantee.
This story adds no media store.

**Deviations from the design plan.** None.

**Spec sections touched.** None. The existing rendering contract already
defines the content-addressed handle and replacement boundary.

**Tests.** Equal bytes collapse to one set key, different fixtures differ,
staged output images use the handle, and line conversion preserves it.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Relationship resolution must happen before
constructing this renderer key. Part-local relationship names must not cross
the shared layout boundary.

### F-037, Create oxml-pdf

**Sprint.** S08
**Completed.** 2026-07-31
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The workspace now contains an unpublished `oxml-pdf`
backend at version 0.0.0. It consumes `oxml-layout` and `oxml-media`, keeps the
eight moved backend gates, and removes the copied image-header parsers.

**Non-obvious choices.** The staged crate has no dependency on an `rdocx-*` or
`rpptx-*` crate. It remains excluded from publication while the PowerPoint
development line is incomplete.

**Deviations from the design plan.** A normal staged package archive was built,
but extracted verification resolves the unpublished 0.0.0 dependencies from
crates.io placeholder packages. The reviewed package rider therefore used
`cargo package --no-verify`. The 19.6 KiB archive stayed below the 10 MiB gate,
and no package was published.

**Spec sections touched.** `docs/hld/03-architecture.md` and
`docs/hld/08-rendering-spec.md` now record the staged backend boundary and its
dependency direction.

**Tests.** Fifteen staged backend tests, including the eight moved gates,
dependency-tree inspection, archive inspection and size, and the integrated
full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep `oxml-pdf` unpublished and independent of
released format crates until the shared publication and cutover sprint.

### F-038, Golden-PNG harness

**Sprint.** S08
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `Document::to_pdf_deterministic` exposes the bundled-font
PDF path, and the golden harness rasterises page one of all seven samples at
150 DPI before comparing decoded RGBA dimensions and SHA-256 digests exactly.

**Non-obvious choices.** The manifest records `pdftoppm version 26.01.0` and
contains digests rather than binary fixtures. Comparison has no tolerance.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/12-testing-strategy.md` and
`docs/hld/15-build-and-toolchain.md` now define the deterministic facade,
rasterizer identity, manifest contents, and exact pixel gate.

**Tests.** Four harness unit tests, seven exact sample comparisons, a synthetic
one-pixel `proposal` failure that names only that sample, facade coverage,
package checks, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Update the golden manifest only for a declared,
reviewed rendering change with a non-empty reason. Continue comparing decoded
pixels exactly.

### F-039, Global CTM flip

**Sprint.** S08
**Completed.** 2026-07-31
**Size.** L, estimated 4 days, actual 1 day

**What was built.** Both PDF writers now emit one page-level
`q 1 0 0 -1 0 H cm`. Text and images cancel that outer reflection locally,
while lines and rectangles use top-left coordinates and link annotations stay
outside the content transform.

**Non-obvious choices.** The correct image operator is
`[w 0 0 -h x y+h]`. Omitting the added height moves images outside the page
under the outer CTM.

**Deviations from the design plan.** The original operator used `y`, which was
corrected to `y+h` before integration. Poppler 26.01.0 then produced exactly
four one-pixel vertical antialias swaps at x 112, two in `invoice` and two in
`quote`. The user approved that exact delta, the manifest changed once with an
F-039 reason, and all seven buffers now compare exactly.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`,
`docs/hld/12-testing-strategy.md`, `docs/hld/13-risks-and-open-questions.md`,
and `docs/hld/14-development-backlog.md` now record the corrected operator,
mirrored released-backend exception, and exact rendering evidence.

**Tests.** Page CTM, top-left geometry, upright text and images, unchanged
annotation coordinates, exact pre-update four-pixel evidence, seven exact
post-update buffers, one-pixel injection rejection, and the integrated full
gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Preserve the same page and image matrices in
both PDF writers until the F-046 cutover removes the released duplicate. Do not
introduce a pixel tolerance.

### F-040, Group rendering

**Sprint.** S09
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The staged PDF writer recursively emits nested groups with
balanced graphics-state saves and restores, local matrices, optional clipping,
shared opacity states, and children in document order.

**Non-obvious choices.** Group clips reuse F-041's private geometry emitter and
group opacity reuses F-044's document-wide alpha registry. Effects remain
staged, and the raster group path remains owned by F-045.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`,
`docs/hld/12-testing-strategy.md`, and `docs/hld/14-development-backlog.md` now
describe recursive group emission and its exact ordering.

**Tests.** `three_deep_groups_balance_graphics_state`, transform ordering,
non-zero and even-odd clipping, shared group opacity, staged effects, the exact
seven-sample golden gate, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Content emission must preserve group boundaries.
Flattening with `walk` is correct for collection passes but would lose clip and
opacity scope here.

### F-041, Path rendering

**Sprint.** S09
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Staged PDF paths now emit move, line, cubic, and close
geometry plus solid fill and stroke state. Paint selection covers `f`, `f*`,
`S`, `B`, and `B*` with balanced graphics state.

**Non-obvious choices.** The private geometry emitter also serves group clips.
Gradient and tile components remain staged, while a supported solid component
still renders when the other paint component is not yet supported.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`,
`docs/hld/12-testing-strategy.md`, and `docs/hld/14-development-backlog.md` now
state the supported geometry, stroke state, and paint operators.

**Tests.** Fill-only, stroke-only, combined, even-odd, command-order, cap, join,
miter, dash, staged-component, exact seven-sample golden, and integrated full
gates.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** F-043 owns gradient resources. Preserve the
single geometry emitter so visible paths and group clips cannot diverge.

### F-042, Rewrite the three collection passes on walk

**Sprint.** S09
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Font usage, image registration, and every link annotation
pass now traverse nested leaves through `walk`. Image resources, annotations,
and recursive emission share depth-first leaf ordinals, and grouped link
rectangles apply the accumulated transform before PDF page conversion.

**Non-obvious choices.** A private per-page emission state carries the resource
maps and leaf ordinal through recursion. It keeps identity aligned without
public model fields, pointer keys, or parallel traversal implementations.

**Deviations from the design plan.** None. Microscope pass 1 strengthened the
grouped-link test to prove the page `/Annots` reference as well as the
transformed annotation dictionary.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`,
`docs/hld/12-testing-strategy.md`, `docs/hld/13-risks-and-open-questions.md`,
and `docs/hld/14-development-backlog.md` now close the R3 mitigation with the
implemented traversal and identity contract.

**Tests.** Nested font subsetting, nested XObject registration and use, nested
transformed link annotations, depth-first leaf identity, top-level stability,
the exact seven-sample golden gate, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Any new resource collection pass must use the
same depth-first leaf contract as recursive content emission.

### F-044, ExtGState alpha

**Sprint.** S09
**Completed.** 2026-07-31
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The staged PDF writer allocates one document-wide
`/ExtGState` for each normalized non-opaque alpha value, writes matching `CA`
and `ca`, and exposes only the states each page uses. Text, lines, rectangles,
solid paths, and group opacity share the registry.

**Non-obvious choices.** Keys use the serialized `f32` value, finite inputs are
clamped, negative zero is normalized, and non-finite values remain opaque. A
path with different fill and stroke alpha repeats its geometry so each paint
operation can select the correct state.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`,
`docs/hld/12-testing-strategy.md`, `docs/hld/13-risks-and-open-questions.md`,
and `docs/hld/14-development-backlog.md` now describe shared PDF alpha and its
deterministic raster gate.

**Tests.** Equal-state reuse, distinct states, matching `CA` and `ca`, opaque
content, shared path alpha, differing fill and stroke alpha, exact midpoint
raster compositing, the exact seven-sample golden gate, and the integrated full
gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Reuse this private registry for any new PDF
primitive with alpha. Do not allocate graphics states per element.

### F-043, Gradient shading dictionaries

**Sprint.** S10
**Completed.** 2026-07-31
**Size.** L, estimated 4 days, actual 1 day

**What was built.** The staged PDF writer now emits deterministic type 2
shading patterns, axial and radial shadings, and type 3 stitching functions
over interval type 2 functions. Page-local pattern resources and fill or stroke
operators address each gradient occurrence by stable depth-first identity.

**Non-obvious choices.** Pattern matrices compose the accumulated group
transform with the global page flip so paint and path geometry share top-left
coordinates. Stops are clamped and sorted, the last repeated offset wins, and
stop alpha uses the documented opaque DeviceRGB fallback over white.

**Deviations from the design plan.** Microscope pass 1 corrected the pattern
matrix to include the global page flip. Pass 2 strengthened the page-local
resource test so it fails if pattern names leak between pages.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`,
`docs/hld/12-testing-strategy.md`, and `docs/hld/14-development-backlog.md` now
describe the implemented PDF gradient resource graph, normalization, matrices,
operators, and exact sampled gate.

**Tests.** Linear and radial resource structure, stop normalization, mixed
solid paint, gradient stroke operators, page-local resources, and exact rotated
gradient samples under Poppler 26.01.0. The integrated 57-test `oxml-pdf`
suite, seven-buffer golden gate, and full workspace gate also pass.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep PDF and raster gradient stop normalization
aligned. The integrated gate completed normally after a worker-only duplicate
hash rerun had stalled in the macOS loader without changing files or evidence.

### F-045, Rasteriser groups, paths, gradients, dashes, and background

**Sprint.** S10
**Completed.** 2026-07-31
**Size.** L, estimated 4 days, actual 1 day

**What was built.** The staged tiny-skia backend now recursively renders group
transforms and intersecting clips, composites group opacity once per subtree,
draws backend-neutral paths with solid or gradient paint, honours line and path
dashes, and paints supported page backgrounds.

**Non-obvious choices.** Scoped opacity uses a scratch pixmap so overlapping
children are not attenuated individually. Non-extended gradient domains receive
an explicit mask, while tile paint and group effects remain outside the S10
contract.

**Deviations from the design plan.** Microscope pass 1 added explicit raster
stop normalization with clamping, stable sorting, and last-repeated-offset
semantics to match F-043. Pass 2 was clean.

**Spec sections touched.** `docs/hld/08-rendering-spec.md`,
`docs/hld/12-testing-strategy.md`, and `docs/hld/14-development-backlog.md` now
describe recursive raster state, paint translation, dashes, backgrounds, and
the deterministic sampled gates.

**Tests.** Twelve raster tests cover the rotated-rectangle and dashed-line
gates, nested transform order, clip intersection, group opacity, fill rules,
linear and radial gradients, gradient domains and normalization, path dashes,
and page backgrounds. The integrated 57-test `oxml-pdf` suite, exact golden
gate, deliberate one-pixel rejection, and full workspace gate also pass.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Preserve recursive draw order and scoped state.
Flattening groups would lose clip and opacity semantics even though `walk` is
correct for collection passes.

### F-052, Create oxml-drawing and namespace constants

**Sprint.** S12
**Completed.** 2026-07-31
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The workspace now contains the unpublished `oxml-drawing`
crate with the DrawingML, relationships, and package namespace constants used
by later model stories.

**Non-obvious choices.** The crate remains at version 0.0.0 with publication
disabled. It started without dependencies so the format-neutral boundary was
established before parsers were added.

**Deviations from the design plan.** None.

**Spec sections touched.** No HLD file changed. The implementation follows the
existing crate boundary in `docs/hld/03-architecture.md` and development
publication policy in `docs/hld/15-build-and-toolchain.md`.

**Tests.** Namespace URI assertions, workspace membership and publication
state checks, package inspection, dependency inspection, and the integrated
full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep every `oxml-*` production edge
format-neutral. The final S12 graph contains no `rdocx-*` or `rpptx-*` edge.

### F-053, OrderedRawChildren

**Sprint.** S12
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `OrderedRawChildren` stores unmodelled XML subtrees at
caller-defined schema boundaries so modelled children can be written in schema
order without moving or dropping unknown siblings.

**Non-obvious choices.** The helper is concrete and schema-boundary based. It
does not own a generic parser policy or hide which child sequence the caller
implements.

**Deviations from the design plan.** None.

**Spec sections touched.** No HLD file changed. The helper implements the
existing child-order and verbatim-preservation contracts in
`docs/hld/05-drawingml-model.md`.

**Tests.** Raw children before, between, and after modelled children, multiple
children at one boundary, byte-for-byte nested subtree preservation, and the
integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Owning parsers still decide their schema
boundaries. Do not append unknown children at the end of a parent.

### F-054, Colour choices

**Sprint.** S12
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `ColorChoice` now models sRGB, scheme, system, and preset
DrawingML colours with validated RGB values, prefix-tolerant parsing,
fixed-prefix writing, and ordered raw-child preservation.

**Non-obvious choices.** System colours preserve `lastClr` as their portable
fallback. Unknown child subtrees remain byte-for-byte raw rather than becoming
partially modelled data.

**Deviations from the design plan.** Microscope pass 1 corrected the shared
`OxmlError` conversion so parser errors retain their source contract. Pass 2
was clean.

**Spec sections touched.** No HLD file changed. The four choices and
preservation behaviour were already specified in
`docs/hld/05-drawingml-model.md`.

**Tests.** All four colour forms parse and round-trip, malformed RGB and system
fallback values fail, unknown nested children retain their exact bytes, and
the integrated full gate passes.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Parse only children the shared renderer
consumes. Preserve every other subtree at its original boundary.

### F-055, The colour transform stack

**Sprint.** S12
**Completed.** 2026-07-31
**Size.** L, estimated 4 days, actual 1 day

**What was built.** All 28 DrawingML colour transforms parse, serialize, and
apply in document order. A readable 40-case table records exact RGBA sampled
from PowerPoint 16.104 build 16.104.25121423.

**Non-obvious choices.** The oracle starts from a PowerPoint-authored native
shape shell, validates the transformed deck without repair, and captures each
shape through the native clipboard PNG payload. This replaced the PowerPoint
`save as picture` command, which returned success without creating a file on
the pinned build.
Linear-light, HSL, alpha, and PNG quantization rules are kept explicit.

**Deviations from the design plan.** The approved plan was revised before
completion to record the clipboard transport. Microscope pass 1 found that
explicit empty start and end transform pairs were preserved raw instead of
modelled. The parser now models those pairs while preserving unexpected
nonempty transform content verbatim. Pass 2 was clean.

**Spec sections touched.** `docs/hld/05-drawingml-model.md`, "Colour, the part
everyone gets wrong", now lists all 28 transforms and the exact resolution and
partial-alpha boundary rules.

**Tests.** The 40 exact PowerPoint cases, all 28 XML mappings, transform order,
linear-gamma conversion, partial alpha, explicit empty pairs, unexpected
nested content, raw-child order, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Expected oracle values are evidence, not values
generated from the Rust formulas. Re-run the ignored generator only with the
pinned PowerPoint build and an explicit native shell.

### F-056, Colour map resolution

**Sprint.** S12
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** A concrete 12-slot `ColorMap` provides the standard Office
mapping, selective layout or slide overrides, and resolution in map, theme,
then transform order. Direct RGB, system, and preset choices bypass the map.

**Non-obvious choices.** Semantic and theme slots are validated enums. The
resolver takes a concrete lookup slice instead of a trait or generic, and a
missing system name uses its `lastClr` fallback.

**Deviations from the design plan.** Microscope pass 1 strengthened all 12
default mappings, all 11 untouched override slots, exact direct-colour results,
and the system fallback path. Pass 2 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
three-stage resolution contract in `docs/hld/05-drawingml-model.md` and leaves
`p:clrMap` parsing to F-069.

**Tests.** Standard mapping, selective override composition, the exact dark
master inversion gate with transforms, direct-colour bypass and system
fallback, plus the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** PresentationML parsers should construct this
format-neutral value. They must not move `p:` parsing into `oxml-drawing`.

### F-057, a:xfrm

**Sprint.** S13
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-drawing` now models DrawingML transforms with shape
and child coordinate rectangles, rotation, flips, prefix-tolerant parsing,
fixed-prefix writing, ordered raw-child preservation, and finite affine
composition.

**Non-obvious choices.** Child coordinates map into the parent rectangle before
rotation and flips are composed. Zero child extents return a typed error rather
than producing non-finite matrix coefficients.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
transform contract in `docs/hld/05-drawingml-model.md`.

**Tests.** `nested_group_transform_composes_to_the_hand_computed_matrix`,
prefix and schema-order writing, raw-child preservation, zero-extent rejection,
and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep DrawingML coordinate conversion in this
format model. Renderer transforms remain backend-neutral values.

### F-058, Guide evaluator

**Sprint.** S13
**Completed.** 2026-07-31
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `oxml-drawing` now parses and evaluates all 17 DrawingML
guide formula tokens from an owned seeded environment, applies adjust-value
overrides in declaration order, evaluates local path commands, and lowers
clockwise arcs to finite cubic Beziers in segments no larger than 90 degrees.

**Non-obvious choices.** Office interoperability defines `mod` as a Euclidean
norm and applies `sqrt` to the absolute input. Multi-turn sweeps are valid and
bounded by a segment-count guard rather than rejected at one full turn.

**Deviations from the design plan.** The approved plan was corrected before
implementation to use the standard 17 formula tokens. Microscope pass 1 found
that valid multi-turn arcs were rejected. A 450-degree regression fixed the
defect, and pass 2 was clean.

**Spec sections touched.** `docs/hld/05-drawingml-model.md`, "Geometry", now
records the standard formula set, owned environment, Office deviations, and
arc-lowering semantics.

**Tests.** `hand_written_custom_geometry_guides_produce_expected_path_coordinates`,
all formula operations, Office deviations, invalid math, finite arc endpoints,
multi-turn sweeps, unit-angle conversion, truncation riders, and the integrated
full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep formula operands owned because custom
geometry supplies document data. Reject every non-finite intermediate before a
renderer sees it.

### F-059, a:custGeom

**Sprint.** S13
**Completed.** 2026-07-31
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-drawing` now parses, writes, and evaluates
`a:custGeom` adjust lists, guide lists, text rectangles, path lists, and path
commands. Reads tolerate arbitrary prefixes, writes use the fixed `a:` prefix,
and unknown subtrees remain byte-for-byte at their schema boundaries.

**Non-obvious choices.** The approved schema-valid custom geometry fixture is
inline because the repository has no fetched deck corpus. The real corpus gate
remains at the M7 boundary.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
geometry and preservation contracts in `docs/hld/05-drawingml-model.md`.

**Tests.** `corpus_custom_geometry_round_trips_and_evaluates_to_a_closed_path`,
prefix and child-order writing, raw-child preservation, malformed-input
handling, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Replace or supplement the inline fixture when
the separately fetched M7 deck corpus is available. Do not commit a binary deck
only to duplicate the same XML boundary test.

### F-060, Fills

**Sprint.** S13
**Completed.** 2026-07-31
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `oxml-drawing` now models no fill, solid fill, linear and
path gradients, pattern fill, and stretched or tiled picture fill with source
rectangles. Gradient stops retain document order, reads tolerate arbitrary
prefixes, and writers emit fixed-prefix schema order.

**Non-obvious choices.** Picture fills retain relationship identifiers as
owned strings without introducing an OPC dependency. Modelled leaf elements
also retain ordered raw children so nested extensions survive round trips.

**Deviations from the design plan.** Microscope pass 1 found that a pattern
colour wrapper containing only an extension was dropped. Pass 2 found the same
class of loss in modelled leaf elements. Both were fixed with focused
regressions, and pass 3 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
fill module and preservation contracts in `docs/hld/05-drawingml-model.md`.

**Tests.** `every_fill_form_round_trips_and_gradient_stops_keep_document_order`,
prefix and schema-order writing, nested raw preservation, malformed-value
rejection, released Word theme isolation, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Resolve picture relationships and media bytes in
the PresentationML package layer. Do not add an OPC dependency to this crate.

### F-061, Lines

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-drawing` now models `a:ln` width, fill, preset and
custom dashes, cap and join choices, and head and tail ends. Reads accept any
prefix, writes use fixed `a:` prefixes, and unsupported children retain their
schema boundaries.

**Non-obvious choices.** Every preset dash token maps to a concrete dash array
without a fallback. The wire model retains schema values while the mapping
provides the later renderer boundary.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
line and preservation contracts in `docs/hld/05-drawingml-model.md`.

**Tests.** `every_preset_line_dash_value_maps_to_a_dash_array`, complete line
round trips, schema-order writing, raw-child preservation, malformed-value
handling, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep dash interpretation in the renderer seam.
The XML layer should continue to preserve the declared line vocabulary.

### F-062, Effects

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `oxml-drawing` now models effect lists and outer shadows,
including geometry and colour, while retaining unsupported effects such as
glow as raw XML at their exact positions.

**Non-obvious choices.** The effect list models only the values needed by the
current renderer contract. Unsupported effect subtrees remain authoritative
wire data rather than partial models.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
effect and preservation contracts in `docs/hld/05-drawingml-model.md`.

**Tests.** `a_shape_with_glow_round_trips_with_glow_intact_as_raw_xml`, outer
shadow round trips, schema-order output, malformed-value handling, and the
integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Add another typed effect only when a current
consumer needs it. Raw preservation already protects unsupported effects.

### F-063, Shape properties and style references

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-drawing` now composes transforms, geometry, fills,
lines, and effects through `a:spPr`, and models line, fill, effect, and font
style references with optional colour choices.

**Non-obvious choices.** Style index zero is valid. Fill indices greater than
1000 select the background-fill list with an offset of 1000, so index 1001
resolves to background fill style 1.

**Deviations from the design plan.** Microscope pass 1 found that zero indices
and colourless style references were rejected. Both schema-valid cases were
added, and pass 2 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
shape composition and style-matrix rules in `docs/hld/05-drawingml-model.md`.

**Tests.** `fill_ref_1001_resolves_to_background_fill_style_1`, all four style
reference forms, zero and colourless references, shape schema order,
malformed-input handling, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** The format-neutral reference model reuses
`ColorChoice`. It does not alter the released Word theme path.

### F-064a, Text body properties and shell

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-drawing` now owns a typed text-body shell and body
properties for insets, anchoring, wrapping, vertical direction, autofit, and
schema-ordered raw children.

**Non-obvious choices.** The first child kept later list styles and paragraphs
opaque until their dependent stories arrived. This preserved a usable shell
without anticipating their models.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
text-body and autofit contracts in `docs/hld/05-drawingml-model.md`.

**Tests.** `every_body_property_autofit_form_round_trips_in_schema_order`,
prefix handling, raw-child boundaries, malformed attributes, and the
integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Autofit remains a wire-model choice. Layout
policy belongs to the later renderer milestone.

### F-064b, Text paragraphs and runs

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `oxml-drawing` now models paragraphs, paragraph and run
properties, regular runs, fields, line breaks, text spans, fonts, hyperlinks,
spacing values, and ordered unsupported XML.

**Non-obvious choices.** Significant leading or trailing text is emitted with
`xml:space="preserve"`. Qualified relationship and whitespace attributes are
matched exactly so hostile prefixes cannot masquerade as `r:id` or
`xml:space`.

**Deviations from the design plan.** Microscope pass 1 found that local-name
fallback could interpret hostile qualified attributes as modelled attributes.
Exact matching and regressions fixed the issue, and pass 2 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
text and preservation contracts in `docs/hld/05-drawingml-model.md`.

**Tests.** `leading_and_trailing_text_whitespace_survives_via_xml_space_preserve`,
paragraph content order, property units, hyperlink attributes, hostile
prefixes, malformed input, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Relationship resolution remains outside this
crate. The parser stores relationship identifiers without adding an OPC edge.

### F-064c, Text bullets

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Paragraph properties now carry character, automatic, and
explicit no-bullet choices with optional font, point or percentage size, and
colour components in DrawingML schema order.

**Non-obvious choices.** All 41 automatic-numbering tokens are explicit. The
wire model retains literal bullet characters, while Wingdings conversion stays
with the later renderer work.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
bullet and preservation contracts in `docs/hld/05-drawingml-model.md`.

**Tests.** `every_modelled_bullet_form_round_trips_in_schema_order`, every
numbering token, optional component order, raw preservation, malformed values,
and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep font-symbol conversion out of structural
round trips so original bullet codepoints remain recoverable.

### F-064d, Nine-level list styles

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `a:lstStyle` now has nine explicit optional paragraph
property slots. The text-body model composes typed body properties, list
styles, and paragraphs into one structural round trip.

**Non-obvious choices.** Fixed slots make invalid list-level numbers
unrepresentable. Unsupported list-style siblings remain raw at their captured
boundaries, and modelled levels write in ascending schema order.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
nine-level text style chain in `docs/hld/07-inheritance-and-resolution.md`.

**Tests.** `schema_valid_text_body_using_all_nine_list_levels_round_trips_structurally`,
ascending level order, raw sibling preservation, invalid levels, and the
integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** The inline schema-valid fixture covers this
sprint. The fetched deck corpus remains required at the M7 boundary.

### F-064, DrawingML text model

**Sprint.** S14
**Completed.** 2026-08-01
**Size.** XL, estimated 0 days after split, actual 1 day

**What was built.** The umbrella closed after F-064a through F-064d delivered
the complete staged DrawingML text hierarchy for body properties, list styles,
paragraphs, runs, fields, breaks, whitespace, and bullets.

**Non-obvious choices.** The XL story carries no duplicate implementation.
Its four child stories own the natural schema boundaries and their individual
evidence, while this entry records the integrated contract.

**Deviations from the design plan.** The approved split used inline
schema-valid XML for S14. The fetched external deck corpus remains the M7
boundary gate, as planned.

**Spec sections touched.** `docs/hld/14-development-backlog.md` already records
the four-child split and the parent closure rule. No further prose change was
required.

**Tests.** The complete nine-level text-body structural round trip,
`leading_and_trailing_text_whitespace_survives_via_xml_space_preserve`, every
child test gate, and the integrated full workspace gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Run the separately fetched PowerPoint deck
corpus before closing M7. Do not publish the PowerPoint development crates
before PowerPoint development is complete.

### F-065, Theme read and write

**Sprint.** S15
**Completed.** 2026-08-01
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `oxml-drawing` now reads and writes complete DrawingML
themes with twelve colour slots, major and minor font collections, supplemental
script fonts, and typed fill, line, effect, and background-fill style lists.
`office_default()` supplies the standard Aptos-era Office theme.

**Non-obvious choices.** Unsupported attributes and children remain raw at
their schema boundaries. The canonical default is pinned to PowerPoint 16.104,
plist build 16.104.25121423, and AppleScript build 1214. The generated theme
opened in that build without repair.

**Deviations from the design plan.** Microscope pass 1 found that modelled XML
attributes were not decoded before canonical rewriting and that private writer
helpers had unjustified single-use generics. Entity decoding, its regression,
and concrete writer helpers fixed both findings. Pass 2 was clean.

**Spec sections touched.** `docs/hld/12-testing-strategy.md`, "The deck
corpus", and `docs/hld/14-development-backlog.md`, the M7 gate and F-065 and
F-067 entries. The external corpus gate now runs at S16 entry after F-067
creates its harness.

**Tests.** `powerpoint_office_theme_round_trips_structurally`,
`office_default_theme_is_accepted_by_powerpoint`, schema-order and fixed-prefix
writing, four format-style lists, raw-child preservation, entity decoding,
malformed input, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** F-067 must execute the carried M7 `a:txBody` and
`a:spPr` corpus gate before M8 model work. Keep development crates unpublished.

### F-066, The rdocx Theme adapter

**Sprint.** S15
**Completed.** 2026-08-01
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `oxml-drawing` now implements
`From<&CT_OfficeStyleSheet>` for the stable `rdocx_oxml::theme::Theme`,
projecting twelve concrete colour slots and the two Latin font families.

**Non-obvious choices.** The dependency runs only from unpublished
`oxml-drawing` to released `rdocx-oxml`. The adapter ignores shared fields the
legacy type cannot represent, prefers a system colour's `lastClr`, and retains
the legacy symbolic fallback when no resolved value exists.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
single dependency exception in `docs/hld/03-architecture.md` and the frozen
Word tint and shade contract in `docs/hld/05-drawingml-model.md`.

**Tests.** `shared_theme_adapter_matches_the_legacy_theme_projection`,
`shared_theme_adapter_does_not_project_unresolved_colour_forms`, the legacy
`tint_shade_modifiers` regression, both dependency trees, the released
`rdocx-oxml` package dry-run, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Do not reverse the dependency or install the
shared parser into the active Word path before the separately reviewed shared
crate publication and cutover work.

### F-067, Create rpptx-oxml and the corpus harness

**Sprint.** S16
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The workspace now contains unpublished `rpptx-oxml` at
version 0.0.0, PresentationML namespace constants, a pinned 50-deck public
corpus manifest and fetcher, opaque OPC round trips, and the carried M7
DrawingML corpus gate.

**Non-obvious choices.** Byte identity means equality of every decompressed
package part after canonical OPC save. ZIP metadata and compression are not
model state. The corpus remains in ignored storage and every fetch is checked
against its pinned SHA-256 value.

**Deviations from the design plan.** The corpus exposed boundary whitespace,
empty hyperlink relationship ids, and an empty custom-geometry path list.
Their canonical parser states and focused regressions were added before the
carried M7 gate passed.

**Spec sections touched.** `docs/hld/03-architecture.md`,
`docs/hld/12-testing-strategy.md`, and
`docs/hld/13-risks-and-open-questions.md` now record the implemented crate,
corpus source, gate, and publication boundary.

**Tests.** `corpus_manifest_is_complete_and_verified`,
`all_corpus_decks_round_trip_opaquely`,
`carried_m7_drawingml_gate_passes_for_the_corpus`, all 50 pinned decks, 6,898
text bodies, 8,643 shape-property elements, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep every PowerPoint development crate at
version 0.0.0 with publication disabled until PowerPoint development is
complete. Do not commit the fetched corpus binaries.

### F-068, presentation.xml

**Sprint.** S16
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `rpptx-oxml` now models the presentation root, slide and
notes sizes, ordered slide and master identifiers, and the default text style.
It validates slide-id bounds and uniqueness while preserving unsupported XML
at its schema boundaries.

**Non-obvious choices.** Relationship identifiers remain strings for the OPC
layer to resolve. Reads use namespace URIs and tolerate alternate prefixes,
while writes use fixed PresentationML, DrawingML, and relationship prefixes.
Canonical-prefix collisions are rejected rather than changing the meaning of
preserved raw XML.

**Deviations from the design plan.** Microscope passes found local-name-only
matching, qualified id ambiguity, and nested canonical-prefix rebinding. URI
aware element matching, qualified relationship attributes, collision checks,
and focused regressions fixed them. Pass 3 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
presentation and preservation contracts in
`docs/hld/06-presentationml-model.md`.

**Tests.** `every_corpus_presentation_part_round_trips_structurally`, all 50
presentation roots, slide-id validation, alternate-prefix writing,
zero-slide templates, malformed input, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep relationship target resolution out of the
XML model and continue rejecting namespace rebinding that would corrupt raw
payload semantics.

### F-069, Slide, layout and master parts

**Sprint.** S16
**Completed.** 2026-08-01
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `rpptx-oxml` now has distinct schema-ordered models for
slides, layouts, masters, common slide data, master text styles, colour maps,
and colour-map overrides. Unsupported timing, transition, extension, and
producer-specific XML remains at ordered raw boundaries.

**Non-obvious choices.** The three roots use concrete types because their
schema sequences differ. Colour maps reuse the DrawingML value model but keep
their PresentationML element ownership. OPC relationship cardinality remains
an integration concern outside the XML structs.

**Deviations from the design plan.** None. Microscope pass 1 was clean. F-070
subsequently replaced the deliberately raw shape-tree boundary with its typed
model.

**Spec sections touched.** No HLD file changed. The implementation follows the
part, colour-map, and preservation contracts in
`docs/hld/06-presentationml-model.md`.

**Tests.** `every_corpus_slide_layout_and_master_round_trips_structurally`,
`corpus_part_relationship_counts_are_valid`, schema-order and colour-map
fixtures, 421 slides, 766 layouts, 76 masters, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep required package relationships in the OPC
layer and preserve unsupported root children in their captured schema slots.

### F-070, The shape tree

**Sprint.** S16
**Completed.** 2026-08-01
**Size.** L, estimated 4 days, actual 1 day

**What was built.** Common slide data now owns a typed, schema-ordered shape
tree with required non-visual group properties, DrawingML group properties,
and recursive group shapes. All six child variants retain document z-order.

**Non-obvious choices.** Only group shapes recurse. Shapes, pictures, graphic
frames, connectors, and alternate content own their captured XML bytes until
their named later stories model them. Group properties expose the existing
DrawingML transform while preserving unsupported children.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
shape-tree and preservation contracts in
`docs/hld/06-presentationml-model.md`.

**Tests.** `nested_group_shape_tree_round_trips_with_tree_shape_preserved`,
`every_corpus_shape_tree_round_trips_structurally`, all six child variants,
required child order, 1,263 trees, 63 recursive groups, and the integrated full
gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** F-071 through F-074 own the opaque payload
variants. Preserve their current XML until each later story replaces one
boundary with an approved typed model.

### F-071, Placeholders

**Sprint.** S17
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `rpptx-oxml` now models placeholder-bearing partial shapes
inside the ordered shape tree. Placeholder keys retain optional indices,
default an absent type to body, and implement index priority plus the title and
body equivalence classes.

**Non-obvious choices.** An optional `u32` preserves the distinction between a
missing index and index zero. Matching compares indices only when both sides
provide one, then falls back to effective placeholder types. Unrelated shape
content remains in ordered raw slots.

**Deviations from the design plan.** Microscope passes found local-name-only
matching, qualified identifier ambiguity, and nested canonical-prefix
rebinding. URI-aware element matching, qualified relationship attributes,
prefix-collision checks, and focused regressions fixed them. Pass 4 was clean.

**Spec sections touched.** `docs/hld/06-presentationml-model.md` now records
the presence-sensitive placeholder key used by the matching contract.

**Tests.** Index and type matching, absent-type defaulting, both equivalence
classes, opaque preservation, nested group shapes, all 50 corpus decks, and
the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep placeholder matching presence-sensitive
and preserve unsupported shape content at its schema boundary.

### F-072, Pictures

**Sprint.** S17
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `rpptx-oxml` now models pictures in root and recursive
shape trees, including non-visual properties, optional placeholders, embedded
and linked image relationships, source-rectangle crops, shape properties,
style, and extensions.

**Non-obvious choices.** Existing DrawingML blip-fill and shape-property types
gained concrete root-aware writers so picture-owned element names remain
schema-correct without adding forwarding wrappers. Relationship identifiers
stay strings for the OPC layer to resolve.

**Deviations from the design plan.** None. Microscope pass 2 was clean after
the first pass findings were remediated.

**Spec sections touched.** No HLD file changed. The implementation follows the
picture, prefix, ordering, and preservation contracts in
`docs/hld/05-drawingml-model.md` and `docs/hld/06-presentationml-model.md`.

**Tests.** Cropped picture round-trip, qualified relationships, alternate read
prefixes, fixed write prefixes, required child order, opaque alternate-content
preservation, 240 corpus pictures, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Resolve image targets in the OPC layer and keep
unsupported blip choices verbatim.

### F-073, Graphic frames

**Sprint.** S17
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `rpptx-oxml` now models graphic frames and dispatches exact
graphic-data URIs to typed tables or opaque chart, SmartArt, OLE, and unknown
payloads. Root and recursive shape-tree arms use the typed frame model.

**Non-obvious choices.** Only the table branch is parsed because F-074 owns its
model. Every other payload remains opaque until its named story. The existing
DrawingML transform gained a concrete `p:xfrm` writer while retaining its
DrawingML path.

**Deviations from the design plan.** None. Microscope pass 2 was clean after
the first pass findings were remediated.

**Spec sections touched.** No HLD file changed. The implementation follows the
graphic-frame dispatch and preservation contracts in
`docs/hld/06-presentationml-model.md`.

**Tests.** Exact URI dispatch, required child order, fixed prefixes,
root-aware transforms, opaque payload preservation, all 86 corpus frames with
all four required kinds observed, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Add typed payload branches only in their owning
stories and retain unknown graphic data verbatim.

### F-074, DrawingML tables

**Sprint.** S17
**Completed.** 2026-08-01
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `oxml-drawing` now models table properties, style and
banding flags, grid columns, rows, cells, text bodies, merge origins, spans,
and horizontal and vertical continuations. Unsupported table and cell content
remains at ordered raw boundaries.

**Non-obvious choices.** Ambiguous edits to preserved grid metadata return a
typed error instead of silently attaching metadata to the wrong column. Cell
text reuses the existing concrete text-body model.

**Deviations from the design plan.** Microscope passes found defects in grid
metadata edits and preservation boundaries. The model now rejects ambiguous
mutations and has focused regressions. Pass 5 was clean.

**Spec sections touched.** `docs/hld/05-drawingml-model.md` now records the
table model, merge semantics, and preservation contract.

**Tests.** Merged-cell origins and continuations, banding and style flags,
schema order, alternate prefixes, opaque preservation, all 26 corpus tables
with 724 cells, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep grid metadata aligned with column identity
and reject edits that cannot preserve that identity unambiguously.

### F-075, Connectors

**Sprint.** S18
**Completed.** 2026-08-01
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `rpptx-oxml` now models connectors in root and recursive
shape trees, including optional typed start and end connections, required
non-visual and shape properties, and ordered preservation of unsupported
content.

**Non-obvious choices.** Each endpoint keeps its required unqualified shape id
and connection-site index. Unsupported locks, style, extensions, attributes,
and children remain at their original schema boundaries.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** `docs/hld/06-presentationml-model.md` now records
the typed connector arm and its optional endpoint contract.

**Tests.** Endpoint round-trip, namespace aliases, fixed prefixes, required
order, qualified-attribute rejection, raw preservation, 85 corpus connectors
with 30 starts, 28 ends, and 6 nested connectors, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep routing and unsupported non-visual content
opaque until a named story owns those boundaries.

### F-076, mc:AlternateContent

**Sprint.** S18
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Shape trees now expose ordered typed members from the
immediate `mc:Fallback` branch while retaining the complete alternate-content
subtree as the only serialisation source.

**Non-obvious choices.** Choices are not evaluated, an absent fallback returns
no selection, and an empty fallback remains distinct from an absent fallback.
Namespace URI resolution identifies the MC fallback instead of its prefix.

**Deviations from the design plan.** None. Microscope pass 2 was clean after
the first pass corrected stale HLD wording.

**Spec sections touched.** `docs/hld/06-presentationml-model.md` now records
fallback-only selection, opaque choices, absent-fallback behaviour, and
raw-only serialisation.

**Tests.** Fallback selection, no-fallback and duplicate-fallback cases,
namespace aliases, recursive order, exact raw preservation, all 21 corpus
alternate-content subtrees, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Add choice evaluation only with an explicit
capability policy. Never serialise this model from the selected fallback.

### F-077, Notes slides and notes master

**Sprint.** S18
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `rpptx-oxml` now models notes slides and notes masters,
types shape text bodies with the existing DrawingML text model, and extracts
speaker notes from effective body placeholders only.

**Non-obvious choices.** Plain text retains run, field, explicit-break, and
paragraph order. Slide images, numbers, dates, footers, headers, and master
prompt text do not enter speaker-note output.

**Deviations from the design plan.** Design review against the authoritative
schema corrected `p:notesStyle` from required to optional. The model, writer,
HLD, and regression gate use the optional contract. Microscope pass 2 was
clean.

**Spec sections touched.** `docs/hld/06-presentationml-model.md` now records
the notes-root sequences, relationship cardinalities, optional notes style,
and body-placeholder extraction contract.

**Tests.** Text extraction order, placeholder filtering, schema order,
optional notes style, raw preservation, relationship completeness, all 210
corpus notes slides and 24 notes masters with 72 nonempty bodies, and the
integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Treat the typed text body as the single source
of truth and keep notes-master prompt text outside speaker-note extraction.

### F-078, relmap rewrite_rel_ids

**Sprint.** S18
**Completed.** 2026-08-01
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `rpptx-oxml` now rewrites mapped numeric relationship ids
inside preserved XML by replacing only eligible relationship-namespace
attribute value ranges.

**Non-obvious choices.** Namespace URI scope, including aliases and nested
shadowing, decides eligibility. Element spelling, declarations, attribute
order, quote choice, comments, processing instructions, and every untouched
byte remain unchanged.

**Deviations from the design plan.** None. Design review corrected one HLD
section citation before implementation, and microscope pass 1 was clean.

**Spec sections touched.** No HLD file changed. The implementation follows the
relationship-remapping contract in `docs/hld/06-presentationml-model.md`.

**Tests.** Embed, link, and diagram relationship rewriting, aliases and
shadowing, unmapped and nonnumeric values, exact surrounding-byte preservation,
malformed XML, empty-map identity across preserved payloads in all 50 corpus
decks, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Use this byte-splice helper before deep-copying
opaque payloads. Do not reconstruct preserved XML through an event writer.

### F-079, The rpptx read facade

**Sprint.** S19
**Completed.** 2026-08-02
**Size.** L, estimated 4 days, actual 1 day

**What was built.** The new unpublished `rpptx` crate opens presentations from
paths or bytes, resolves ordered slides and notes through OPC relationships,
and exposes safe borrowed slide and recursive shape handles with text, table
text, and speaker-note access. It also saves facade-owned modelled parts through
the deterministic package writer.

**Non-obvious choices.** Immediate shape iteration preserves z-order. Groups
and selected alternate-content fallbacks expose children explicitly. Indexed
access returns `Option`, missing notes remain distinct from empty notes, and
the public facade does not expose schema-layer `CT_*` values.

**Deviations from the design plan.** Microscope pass 1 found package-root
relationship targets with dot segments were resolved incorrectly and found a
forwarding-only helper. Both were corrected, and pass 2 was clean.

**Spec sections touched.** `docs/hld/06-presentationml-model.md` records the
read facade and relationship-resolved ownership model. `docs/hld/12-testing-strategy.md`
records the normalized python-pptx differential gate.

**Tests.** Eight facade integration tests cover ordered slides and notes, total
indexed access, all six shape kinds, recursive text, contextual graph errors,
deterministic reopen, package-root dot segments, and workspace publication
metadata. `dump_deck_matches_python_pptx_1_0_2_for_the_corpus` matched the
pinned oracle across all 50 decks. The integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep `rpptx` at version 0.0.0 with
`publish = false` until PowerPoint development is complete. Preserve the
normalized facade boundary instead of leaking lower-level model types.

### F-080, Modelled round-trip gate

**Sprint.** S19
**Completed.** 2026-08-02
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The 50-deck gate now parses, serialises, reparses, and
structurally compares all seven modelled PresentationML and theme roots. It
builds an exact expected package, checks canonical bytes for rewritten parts,
preserves original bytes for unmodelled parts, and verifies the facade read
surface after save and reopen.

**Non-obvious choices.** `CT_StyleMatrix.name` is optional because accepted
producer themes omit `a:fmtScheme/@name`. Canonical `a:blip` output declares
the relationship namespace locally whenever it writes `r:embed` or `r:link`,
so independently serialised fills remain namespace-valid.

**Deviations from the design plan.** The approved plan was extended when the
first corpus run exposed an absent format-scheme name and the first native
PowerPoint run exposed an undeclared relationship prefix. Both compatibility
repairs received focused regressions. Microscope pass 3 was clean.

**Spec sections touched.** `docs/hld/05-drawingml-model.md` records optional
format-scheme names and self-contained blip namespaces.
`docs/hld/06-presentationml-model.md` and
`docs/hld/12-testing-strategy.md` record the structural and exact-byte
round-trip boundary.

**Tests.** The required 50-deck corpus passed all 11 `rpptx` tests, including
seven-root structural equality, exact expected packages, facade reopen, and
the pinned python-pptx differential. Both compatibility regressions passed.
A Codex-operated native run opened and closed all 50 generated decks without
repair in PowerPoint 16.104 build 16.104.25121423. The output digest was
`19609644c12923fad63939656fc54681c667efa2e066fbd2a080bb717aa037fc`.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep lexical byte equality for unmodelled parts
and exact expected canonical bytes for rewritten roots. XML well-formedness is
not a substitute for the native PowerPoint acceptance gate.

### F-081, ResolveCtx skeleton and placeholder chain

**Sprint.** S20
**Completed.** 2026-08-02
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The new unpublished `rpptx-layout` crate owns a concrete
per-slide `ResolveCtx` with theme, colour-map, presentation text defaults,
master, layout, and slide inputs. It resolves an ordinary slide placeholder
through nested groups and selected fallback branches to its layout and master
counterparts.

**Non-obvious choices.** Matching uses the existing index-first and
type-fallback rule. The master hop uses the matched layout placeholder key, and
a missing layout match terminates the chain instead of skipping a level.

**Deviations from the design plan.** Microscope pass 1 found that the chain
method had the wrong visibility. It was corrected to the crate boundary, and
pass 2 was clean.

**Spec sections touched.** No HLD file changed. The existing architecture and
inheritance HLD already assigned this boundary to `rpptx-layout`.

**Tests.** Four focused tests cover the complete two-hop gate, layout-key
master matching, recursive group and fallback lookup, and missing or
non-placeholder shapes. The integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Build each context once per slide and preserve
the exact hierarchy order when adding later resolver consumers.

### F-082, Effective transform and body properties

**Sprint.** S20
**Completed.** 2026-08-02
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Ordinary `p:spPr` is typed as DrawingML shape properties.
The resolver returns the first slide, layout, or master transform and merges
body properties per field over exact OOXML inset, anchor, wrap, direction, and
autofit defaults.

**Non-obvious choices.** Transform results are owned clones so hierarchy
lifetimes do not leak through the API. Body values overlay master, layout, and
slide in that order, retaining unrelated inherited fields.

**Deviations from the design plan.** A canonical-prefix assertion was updated
when typed `p:spPr` began using the fixed `p:` root. The first corpus run also
exposed excessive debug-stack pressure from storing shape properties by value.
F-084 completed the reviewed boxed-storage remediation while preserving field
access semantics and proving the normal-stack corpus gate.

**Spec sections touched.** `docs/hld/06-presentationml-model.md` records typed
ordinary-shape properties. `docs/hld/07-inheritance-and-resolution.md` records
owned transform precedence and the exact per-field body cascade.

**Tests.** Transform precedence, exact defaults, per-field body overlay,
ordinary-shape schema order and raw preservation, all 68 PresentationML parser
tests, and the normal-stack 50-deck structural corpus gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep the exact integer EMU defaults and do not
replace the property-level body cascade with whole-value selection.

### F-083, The seven-step list style merge

**Sprint.** S20
**Completed.** 2026-08-02
**Size.** L, estimated 4 days, actual 1 day

**What was built.** DrawingML list styles now type `a:defPPr`, and
`rpptx-layout` resolves all nine levels through presentation defaults, the
selected master style, master and layout placeholders, shape list style,
paragraph properties, and run properties.

**Non-obvious choices.** The cache stores only the first four sources by
optional placeholder key. Shape formatting is applied after cloning the cached
prefix, so two shapes occupying one placeholder cannot leak direct formatting.
Bullet components and nested character properties merge independently.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** `docs/hld/05-drawingml-model.md` records typed
default paragraph properties. `docs/hld/07-inheritance-and-resolution.md`
records level selection, merge granularity, and prefix-only cache semantics.

**Tests.** Eleven resolver tests cover the named seven-source gate, all nine
levels, `defPPr`, property retention, bullet components, atomic fill and font
slots, raw-action exclusion, and cache isolation. DrawingML parser and required
corpus structural tests also passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Apply shape-owned style after cache cloning and
keep paragraph level selection separate from inherited formatting.

### F-084, Format scheme reference resolution

**Sprint.** S20
**Completed.** 2026-08-02
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Ordinary shapes now type `p:style` in schema order. The
resolver selects one-based fill, line, and effect format entries, applies the
background-fill rule above 1000, substitutes modelled placeholder colours, and
layers explicit shape properties over theme values.

**Non-obvious choices.** Index zero means no referenced entry, while a positive
out-of-range value is an error. Opaque effect DAGs replace referenced effects
atomically and are rejected when they retain unresolved `phClr`. Font
references retain their `major`, `minor`, or `none` collection selector rather
than being treated as numeric indices.

**Deviations from the design plan.** The plan was revised after the integrated
F-082 parser overflowed on a real slide master at normal stack size. Boxing
shape properties and private shape style removed the large transient values
without requiring a stack-size workaround. Microscope pass 1 then found opaque
effect DAGs bypassed unresolved-colour checks. Three regressions fixed that
defect, and microscope pass 2 was clean. Sprint review pass 1 then found that
the raw scanners matched local names without resolving namespaces. Two focused
regressions made foreign producer extensions inert.

**Spec sections touched.** `docs/hld/06-presentationml-model.md` records typed
shape properties and style with bounded storage. `docs/hld/07-inheritance-and-resolution.md`
records numeric format-list rules, font collection selection, placeholder
colour substitution, explicit overlays, and malformed-reference errors.

**Tests.** Thirty-five integrated resolver tests include the named fill gate,
background fills, zero and out-of-range indices, transform order, explicit
overlays, modelled and opaque effects, unresolved placeholder rejection, and
same-named foreign effect extensions. All 68 parser tests, the 40-case exact
colour table, and the normal-stack 50-deck corpus gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Never claim an opaque effect with `phClr` is
concrete. Type it, substitute it, or return a resolver error.

### F-085, Typeface resolution

**Sprint.** S20
**Completed.** 2026-08-02
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `ResolveCtx::resolve_typeface` maps major and minor Latin,
East Asian, and complex-script tokens to concrete theme faces and applies
supplemental per-script overrides.

**Non-obvious choices.** Script is an explicit optional input because the
current run model does not perform script segmentation. The first matching
supplemental entry wins in document order, missing matches fall back to the
token-specific base face, and ordinary or unknown typefaces pass through.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** `docs/hld/07-inheritance-and-resolution.md` records
the explicit script input, all six tokens, override order, fallback, and
pass-through behavior.

**Tests.** Six focused tests cover the named minor Latin gate, all major and
minor aliases, supplemental overrides, missing-script fallback, pass-through,
and duplicate-script document order. The integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep Unicode script segmentation in the text
shaping layer and pass the resulting ISO 15924 tag into this resolver.

### F-086, Draw order and the flattener

**Sprint.** S21
**Completed.** 2026-08-02
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `ResolveCtx::flatten` now emits the effective background,
allowed master artwork, allowed layout artwork, and slide shape-tree leaves in
final source order. It walks nested groups and selected fallback content while
retaining each leaf's source. Slide and layout `showMasterSp`, ordinary
placeholder suppression, typed header-footer controls, and occupied latent
placeholders are applied before the renderer boundary.

**Non-obvious choices.** Background selection keeps the producing part and
effective master colour map. The layout visibility flag controls only the
master pass, while the slide flag controls only the layout pass. Ordinary
master and layout placeholders remain templates rather than drawable content.

**Deviations from the design plan.** Microscope pass 1 found that the tests did
not isolate the two visibility controls and did not prove all four background
fallback sources. Both coverage gaps were corrected, and pass 2 was clean.

**Spec sections touched.** `docs/hld/06-presentationml-model.md` records typed
visibility and header-footer inputs. `docs/hld/07-inheritance-and-resolution.md`
records the borrowed flattened view, final draw order, and suppression policy.

**Tests.** Forty-one resolver tests, 68 PresentationML integration tests, all
50 pinned corpus decks, exact-colour checks, and the integrated full gate
passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep source order and suppression in the
flattener. Renderers must not reconstruct the slide, layout, and master passes.

### F-087, ResolvedSlide contract

**Sprint.** S21
**Completed.** 2026-08-02
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `rpptx-layout` now exposes an owned `ResolvedSlide`
boundary with point geometry, accumulated group transforms, concrete paint,
lines, shadows, text, tables, unsupported categories, and diagnostics. The
contract contains no PresentationML or DrawingML model types and retains
visible bounds fallbacks when content cannot yet be represented exactly.

**Non-obvious choices.** Custom paths scale in their own declared coordinate
spaces. Unsupported gradient geometry, media relationships, charts, SmartArt,
OLE, and pending presets remain explicit instead of being approximated as
concrete output. Character and automatic-number bullets retain independently
inherited font, colour, size, and choice values.

**Deviations from the design plan.** The plan added a test-only `oxml-opc`
dependency so both named gates could traverse all 50 real decks. Microscope
pass 1 found six contract and corpus defects, and pass 2 found group-composition
and automatic-number bullet defects. All were remediated. Pass 3 was clean.
Sprint review pass 1 then found that the named corpus gate accepted contextual
resolver errors. The strict gate exposed 20 affected slides. Preset black and
white now resolve concretely, and invalid custom geometry retains a diagnosed
bounds fallback. All corpus slides now produce an owned contract.

**Spec sections touched.** `docs/hld/07-inheritance-and-resolution.md` freezes
the owned output and fallback boundary. `docs/hld/08-rendering-spec.md` records
the accumulated group transform supplied to renderers.

**Tests.** Fifty-seven resolver tests, two independent 50-deck gates, exact
colour checks, dependency-direction riders, publication dry-run, and the
integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Renderers consume only the owned contract.
Relationship-to-media resolution and pending geometry evaluation must fill the
documented gaps without exposing source-model types.

### F-088, Visual differential tests

**Sprint.** S21
**Completed.** 2026-08-02
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The existing `rpptx` integration binary now assembles full
resolver inputs through package relationships and emits normalized visual
records with ordered shape kinds, bounds, concrete shape and run paint, text,
unsupported categories, and diagnostics. It compares shared fields with pinned
python-pptx 1.0.2, proves exact cyan inheritance, prompt suppression, draw
order, and master artwork multiplicity, and records the one-time native
PowerPoint acceptance.

**Non-obvious choices.** Python supplies only mutually observable structure.
Rust separately asserts effective latent-placeholder visibility because
python-pptx exposes raw collections. Automated gates skip when the ignored
external corpus is absent and optional, but fail when the corpus is required.
The manual acceptance record remains available without the corpus files.

**Deviations from the design plan.** The first native review exposed inherited
date and slide-number fields that PowerPoint hid and an explicit slide footer
that the resolver dropped. The private flattener was repaired to require an
inherited header-footer container, preserve occupied slide latent content, and
match latent types across level-specific indices. Microscope pass 1 found two
evidence gaps, pass 2 was clean after remediation, and integrator review found
the clean-clone corpus issue. The compact repair received a clean pass 3.

**Spec sections touched.** `docs/hld/07-inheritance-and-resolution.md` records
source-sensitive latent visibility and the executable visual differential.
`docs/hld/12-testing-strategy.md` records the selected decks, pinned oracle,
external-corpus policy, and native acceptance evidence.

**Tests.** Sixteen `rpptx` tests, 57 resolver tests, all 50 pinned decks, the
40-case exact PowerPoint colour table, optional and required corpus modes, and
the integrated full gate passed. Microsoft PowerPoint 16.104 build
16.104.25121423 opened and exported all four selected originals without repair
or clipping. Native master artwork, backgrounds, exact cyan, prompt
suppression, and footer visibility matched the remediated evidence.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep native visibility policy in the flattener,
keep the Python comparison structural, and preserve explicit unsupported
diagnostics until modelled background paint and media resolution land.

### F-089, Resolve the preset geometry licensing question

**Sprint.** S22
**Completed.** 2026-08-02
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The renderer specification and risk record now identify
the official ECMA-376 fifth-edition Part 1 electronic addendum as the permitted
source for preset geometry definitions. They record the inner archive path,
187-definition count, exact SHA-256, Ecma software-policy basis, and retained
BSD three-clause notice requirement.

**Non-obvious choices.** The decision permits only the official Ecma data set.
The MPL-2.0 LibreOffice implementation table remains rejected, and derivation
from specification text remains the fallback if the official file or notice
cannot be reproduced exactly.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** `docs/hld/08-rendering-spec.md` records the chosen
source and generator input. `docs/hld/13-risks-and-open-questions.md` closes the
provenance question with its licensing evidence.

**Tests.** `preset_geometry_provenance_is_recorded`,
`libreoffice_preset_table_remains_rejected`, repository prose checks, and the
integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Retain the vendored Ecma notice with the source
XML and generated table. Do not substitute an implementation-owned table.

### F-090, Preset table generator

**Sprint.** S22
**Completed.** 2026-08-02
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `tools/gen-presets/` now vendors the permitted Ecma XML and
licence notice and generates a checked-in Rust lookup table offline. Check mode
verifies the exact source hash, byte-identical regeneration, all 187 direct
definitions, 186 unique preset names, and coverage of every preset used by the
50-deck corpus.

**Non-obvious choices.** The official source repeats `upDownArrow` twice with
byte-identical XML. The generator rejects conflicting duplicate names and
deduplicates only this identical pair, leaving a deterministic 186-key table.

**Deviations from the design plan.** Source inspection corrected the planned
187 unique names to 187 direct definitions and 186 unique names. Microscope
pass 1 also found that the corpus scan accepted foreign-namespace elements and
that duplicate comparison passed through XML reserialization. Both checks now
operate on namespace-qualified input and exact source bytes. Pass 2 was clean.

**Spec sections touched.** None. F-089 had already recorded the source and the
rendering specification already required the offline checked-in mechanism.

**Tests.** `generator_reproduces_checked_in_table`,
`generated_table_covers_every_corpus_preset`,
`source_has_187_direct_definitions`, `generated_lookup_has_known_and_unknown_cases`,
the 50-deck scan covering 2,141 uses and 26 corpus names, and the integrated
full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Regenerate only from the pinned XML after its
hash check. A repeated name with different source bytes is a hard failure.

### F-091, Preset evaluation and fallback

**Sprint.** S22
**Completed.** 2026-08-02
**Size.** M, estimated 2 days, actual 1 day

**What was built.** DrawingML preset geometry now models the preset name,
adjustment guides, and ordered raw children while preserving unknown XML.
Known presets use the generated definitions and shared custom-geometry guide
engine to produce backend-neutral paths and text rectangles. Unknown presets
retain shape bounds and text and emit a diagnostic naming the preset.

**Non-obvious choices.** Preset definitions are parsed through the existing
custom-geometry path instead of gaining a second evaluator. Shape-level
adjustments override generated defaults, and custom geometry retains schema
choice precedence when both forms are encountered.

**Deviations from the design plan.** Microscope pass 1 found that the corpus
gate could pass without proving evaluation. The strengthened gate exposed the
standard `wd12` and `hd10` fractional guides, which are now seeded by the
existing evaluator. Pass 2 was clean.

**Spec sections touched.** None. The implementation follows the existing
DrawingML parsing, resolver output, and preset fallback contracts.

**Tests.** `preset_geometry_round_trips_with_unknown_children_verbatim`,
`rectangle_preset_evaluates_to_expected_bounds_and_text_rect`,
`preset_adjustments_override_generated_defaults`,
`unknown_preset_keeps_bounds_text_and_diagnostic`, the non-vacuous 50-deck
corpus gate across 921 preset inputs, and the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep preset and custom geometry on the shared
guide engine. Unknown names must remain visible and diagnosed.

### F-092, rpptx-render skeleton and RenderInput

**Sprint.** S22
**Completed.** 2026-08-02
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The workspace now contains an unpublished `rpptx-render`
crate. Its `RenderInput` consumes owned `ResolvedSlide` values, content-addressed
media, fonts, and metadata. Upstream `SlideBundle` assembly carries slide,
layout, master, parsed theme, notes, visibility, and three explicitly scoped
relationship maps.

**Non-obvious choices.** Relationship lookup always names slide, layout, or
master scope, so identical relationship IDs cannot alias. Media keys derive
from bytes and deduplicate shared content. Raw PresentationML stays upstream of
the rendering boundary.

**Deviations from the design plan.** The implementation added the direct
inward `oxml-drawing` dependency required by `SlideBundle`'s concrete
`CT_OfficeStyleSheet` field. The dependency-direction rider confirms that no
reverse `oxml-*` to `rpptx-*` edge was introduced. Microscope pass 1 was clean.

**Spec sections touched.** `docs/hld/08-rendering-spec.md` now separates raw
assembly through `SlideBundle` from renderer consumption through `RenderInput`.

**Tests.** `same_relationship_id_resolves_independently_in_all_three_scopes`,
`equal_media_bytes_deduplicate_to_one_media_entry`,
`missing_relationship_reports_scope_and_id`,
`render_input_contains_only_resolved_slides`,
`rpptx_render_dependency_direction_is_one_way`, the publication dry-run, and
the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep raw OOXML in `SlideBundle` assembly and
feed renderers only the frozen owned resolver contract. All PowerPoint crates
remain version 0.0.0 with publication disabled until development is complete.

### F-093, Shape geometry, fills and lines

**Sprint.** S23
**Completed.** 2026-08-03
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `rpptx-render` now lowers resolved slides into ordered page
frames and shape geometry into backend-neutral paths. Solid fills, gradients,
outlines, diagnostics, metadata, and page order cross the renderer boundary.
An otherwise unpainted bounds fallback receives a deterministic 1 point black
outline so unsupported geometry remains visible.

**Non-obvious choices.** Shape paths stay in local coordinates beneath one
group transform. This keeps geometry, paint, and later text on one transform
path and avoids rewriting gradient coordinates into page space.

**Deviations from the design plan.** The sampled-pixel gate exposed an existing
`oxml-pdf` raster defect that applied an accumulated group transform twice to
gradient coordinates. The backend now keeps shader coordinates local, with a
focused regression test. Microscope pass 1 was clean.

**Spec sections touched.** None. The implementation follows the existing
page-frame and path-lowering contract.

**Tests.** `solid_gradient_and_outlined_shapes_rasterise_at_sampled_pixels`,
`preset_and_custom_geometry_lower_to_ordered_paths`,
`bounds_fallback_emits_a_visible_black_outline`,
`layout_slide_rejects_an_out_of_range_index`,
`layout_presentation_preserves_page_order_and_diagnostics`,
`translated_group_gradient_uses_local_coordinates_exactly_once`, and the
integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep gradient coordinates local to the same
group as their paths. The visible fallback outline is part of the approved
renderer contract.

### F-094, Rotation, flips and groups

**Sprint.** S23
**Completed.** 2026-08-03
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Resolved shapes now compose local rotation, centre-based
horizontal and vertical flips, bounds translation, and the accumulated parent
group transform into one backend-neutral group transform.

**Non-obvious choices.** The exact order is child rotation, flips,
translation, then parent transform. Geometry, gradients, outlines, and later
content remain beneath the same group so every visual component shares the
same placement.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** None. The implementation follows the documented
DrawingML composition and shared group boundary.

**Tests.** `rotated_shape_corners_match_hand_computed_coordinates`,
`horizontal_and_vertical_flips_are_about_the_shape_centre`,
`nested_group_transform_applies_child_before_parent`,
`rotated_gradient_and_outline_share_the_shape_transform`, and the integrated
full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Use the shared group transform for every new
shape-local element. Do not pre-transform individual path coordinates.

### F-095, Arrowheads

**Sprint.** S23
**Completed.** 2026-08-03
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The resolved shape contract now carries source-neutral
line-end kind, width, and length values. The renderer derives stable endpoint
tangents and lowers triangle, stealth, diamond, oval, and arrow ends into
closed filled paths using the resolved line paint.

**Non-obvious choices.** Missing dimensions use DrawingML medium defaults.
Small, medium, and large dimensions are 2, 3, and 5 times the stroke width.
Degenerate segments omit their decoration without producing invalid geometry.

**Deviations from the design plan.** Microscope pass 1 found that structural
path checks did not prove rendered output. A deterministic raster assertion
was added, and pass 2 was clean.

**Spec sections touched.** `docs/hld/07-inheritance-and-resolution.md` and
`docs/hld/08-rendering-spec.md` now include the approved neutral line-end
contract and filled-path lowering.

**Tests.** `line_end_resolution_keeps_kind_width_and_length`,
`triangular_tail_end_emits_an_extra_filled_path`,
`head_end_uses_the_reversed_start_tangent`,
`all_supported_line_end_kinds_produce_finite_geometry`,
`zero_length_segment_omits_arrowhead_without_panicking`, deterministic raster
evidence, and the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Arrowheads remain presentation-side geometry,
not a shared stroke or backend primitive.

### F-096, Pictures with crop and tile

**Sprint.** S23
**Completed.** 2026-08-03
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Slide, layout, and master relationship identifiers now map
to content-addressed media in separate source scopes. Resolved picture content
carries neutral stretch or tile placement, and the renderer lowers crop,
alignment, translation, scale, flip, DPI, and rotation policy into clipped
shared image elements with bounded row-major tiling.

**Non-obvious choices.** Missing tile values normalize to zero translation,
100 percent scale, no flip, top-left alignment, and rotation with the shape.
Declared DPI wins over embedded DPI, which wins over 96 DPI. External linked
media remains unsupported without network access.

**Deviations from the design plan.** Microscope pass 1 found that
`rotateWithShape=false` did not cover every stretch and tile branch. The fix
made coverage, clipping, flip phase, and rotation policy explicit. Pass 2 was
clean.

**Spec sections touched.** `docs/hld/07-inheritance-and-resolution.md` and
`docs/hld/08-rendering-spec.md` now describe source-scoped media and neutral
picture placement.

**Tests.** `cropped_picture_renders_only_its_crop_region`,
`same_relationship_id_resolves_to_distinct_media_in_each_source_scope`,
`picture_model_resolves_to_neutral_stretch_and_tile_placement`,
`crop_lowers_to_clipped_source_image_geometry`,
`tile_picture_repeats_media_in_row_major_order_inside_shape_clip`,
`tile_dpi_prefers_declared_then_embedded_then_96`,
`equal_picture_bytes_reuse_one_media_id_across_elements`,
`missing_external_media_and_empty_crop_are_contextual`, and the integrated
full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Resolve relationships to `MediaId` before the
renderer boundary. Keep repeated image lowering bounded even for malformed
placement values.

### F-097, Backgrounds

**Sprint.** S23
**Completed.** 2026-08-03
**Size.** S, estimated 1 day, actual 1 day

**What was built.** PresentationML backgrounds now retain their complete raw
subtree as the sole serialization source while exposing a typed rendering
projection for `p:bgPr` and `p:bgRef`. The resolver applies slide, layout,
master, then theme precedence, resolves style references and `phClr`, and the
renderer assigns concrete paint to the page background before shape content.

**Non-obvious choices.** The typed projection is read-only and never becomes a
second writer. Unsupported paint records a specific diagnostic, while the raw
subtree remains byte-for-byte preserved in its schema position.

**Deviations from the design plan.** Microscope pass 1 strengthened the exact
theme transform-order assertion. A later lint exposed an oversized fill enum
variant, which was boxed without changing the serialized contract. Microscope
pass 3 was clean.

**Spec sections touched.** `docs/hld/06-presentationml-model.md` and
`docs/hld/07-inheritance-and-resolution.md` now describe the preserving
background projection and concrete resolution path.

**Tests.** `background_projection_preserves_the_source_subtree_verbatim`,
`background_precedence_is_slide_layout_master_then_theme`,
`master_gradient_background_renders_when_slide_and_layout_omit_one`,
`background_reference_resolves_phclr_through_the_master_colour_map`,
`background_is_not_duplicated_in_page_elements`, and the integrated full gate
passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep the captured background subtree as the
only serialization source and the typed form as a rendering projection only.

### F-098a, Text content box

**Sprint.** S24
**Completed.** 2026-08-05
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Shape text now starts from the evaluated preset or custom
geometry text rectangle, falls back to local shape bounds when needed, applies
all four resolved insets, and clamps malformed negative extents to zero.

**Non-obvious choices.** The content box remains in shape-local coordinates so
the existing shape group transform places paths and text through one boundary.
Missing geometry uses a visible bounds fallback instead of dropping text.

**Deviations from the design plan.** Microscope pass 1 found that the fallback
regression omitted the diagnosed bounds-fallback case. The test was expanded,
and pass 2 was clean.

**Spec sections touched.** None. The implementation follows the existing text
rectangle and body-inset contract.

**Tests.** `preset_text_rectangle_minus_unequal_insets_produces_the_computed_content_box`,
`missing_text_rectangle_falls_back_to_local_shape_bounds`,
`insets_larger_than_the_text_rectangle_do_not_create_negative_extents`, and the
integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep the box local and finite before shaping.
Do not clip overflowing text at the shape boundary.

### F-098b, Paragraph inline resolution

**Sprint.** S24
**Completed.** 2026-08-05
**Size.** L, estimated 4 days, actual 1 day

**What was built.** Resolved paragraphs now become shaped inline segments with
concrete size, fill, style, typeface, fields, and explicit break boundaries.
One font manager and shaping cache are reused across presentation layout.

**Non-obvious choices.** Typeface selection follows the script actually present
in the text, then uses the resolved Latin, East Asian, or complex-script slot.
Visible 18 point black sans-serif defaults cover missing resolved styling.

**Deviations from the design plan.** Microscope pass 1 found that a populated
Latin slot incorrectly overrode script-specific faces. Text-driven slot
selection and regressions fixed it, and pass 2 was clean.

**Spec sections touched.** None. The frozen resolver and shared shaping
boundaries already describe the implemented ownership.

**Tests.** `resolved_runs_emit_glyph_items_with_concrete_style_and_break_boundaries`,
`script_specific_text_selects_its_resolved_concrete_typeface`,
`repeated_resolved_runs_reuse_one_shaped_cache_entry`,
`text_shaping_failures_return_a_render_error_without_panicking`, and the
integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Select a concrete typeface before shaping and
keep shaping failures contextual instead of substituting silent empty output.

### F-098c, Line stacking

**Sprint.** S24
**Completed.** 2026-08-05
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Paragraph margins, hanging indents, wrapping, explicit
breaks, point and percentage spacing, horizontal alignment, and stacked
baselines now lower into positioned text and marker items without clipping.

**Non-obvious choices.** Text and markers share one baseline emitter. Percentage
line spacing uses the effective first-run size, while justified final lines
retain their ordinary alignment.

**Deviations from the design plan.** Microscope pass 1 corrected percentage
spacing that used natural metrics. Pass 2 added proof for justified lines and
production draw order. Pass 3 was clean.

**Spec sections touched.** None. The line-breaking and paragraph-order
contracts already cover the implementation.

**Tests.** `paragraphs_stack_wrapped_lines_with_spacing_and_alignment`,
`wrap_none_breaks_only_at_explicit_line_breaks`,
`percentage_line_spacing_uses_effective_first_run_font_size`,
`shape_text_stays_above_the_path_and_overflows_without_a_clip`, and the
integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep line breaking and baseline emission shared
between body text and markers so later decorations cannot drift vertically.

### F-098d, Text anchoring

**Sprint.** S24
**Completed.** 2026-08-05
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Complete text blocks now use top, centre, bottom, justified,
or distributed vertical placement inside the content box, then render above the
shape path within the existing group transform.

**Non-obvious choices.** Overflow remains visible. Justified anchoring allocates
spare height between line boxes, while distributed anchoring uses equal line
gaps with half a gap before the first line and after the last.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** `docs/hld/08-rendering-spec.md` now defines the exact
justified and distributed anchoring policy.

**Tests.** `bottom_center_text_in_an_inset_box_lands_at_the_computed_baseline`,
`top_center_and_bottom_anchors_use_zero_half_and_full_spare_height`,
`justified_and_distributed_anchors_allocate_positive_spare_height`,
`overflowing_anchored_text_remains_visible_without_a_clip`, and the integrated
full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Measure the whole unanchored block first, then
apply one vertical placement policy without adding a clip.

### F-098, Shape text layout

**Sprint.** S24
**Completed.** 2026-08-05
**Size.** XL, estimated 0 days, actual 1 day

**What was built.** The umbrella closes the integrated content-box, inline
resolution, line-stacking, and anchoring pipeline delivered by F-098a through
F-098d. Shape text now follows the frozen resolved-slide boundary end to end.

**Non-obvious choices.** The umbrella has no independent source diff. Its gate
is the combined child evidence and deterministic bottom-centre baseline.

**Deviations from the design plan.** None. Every child completed its own plan,
review, tests, and delivery record before the parent closed.

**Spec sections touched.** `docs/hld/14-development-backlog.md` defines the four
implemented ownership boundaries and the combined parent gate.

**Tests.** F-098a through F-098d focused gates, the deterministic
`bottom_center_text_in_an_inset_box_lands_at_the_computed_baseline` regression,
and the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Extend the existing private text module and
shared layout primitives. Do not introduce a second presentation text model.

### F-099, Bullets

**Sprint.** S24
**Completed.** 2026-08-05
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Character and automatic bullets now render with independent
size, colour, and typeface styling. Automatic counters are scoped per body and
level across eight common formats, and Wingdings F0B7 maps to a visible Unicode
bullet.

**Non-obvious choices.** Scheme changes, character bullets, no-bullet
paragraphs, and shallower levels reset the relevant sequence. Unsupported
schemes retain a visible Arabic-period marker, and the hanging slot stays fixed
even when a marker is wider than it.

**Deviations from the design plan.** Microscope pass 1 found that long markers
expanded the fixed hanging slot. The slot was fixed and an oversized-marker
regression added. Pass 2 was clean.

**Spec sections touched.** `docs/hld/08-rendering-spec.md` now records the eight
formats, counter resets, visible fallback, and Wingdings mapping.

**Tests.** `wingdings_f0b7_bullet_renders_as_a_visible_unicode_glyph`,
`automatic_bullets_increment_and_reset_by_level`,
`eight_common_auto_number_schemes_format_exact_markers`,
`wide_auto_number_marker_keeps_text_on_the_paragraph_margin`, and the integrated
full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep numbering state inside one resolved body and
keep marker measurement independent from the fixed text margin.

### F-100, Autofit

**Sprint.** S24
**Completed.** 2026-08-05
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Stored normal-autofit font scale and line-spacing reduction
now apply verbatim. Bare normal autofit selects from a deterministic 2.5 percent
ladder down to 25 percent, while no-autofit and shape-autofit keep visible
overflow behavior.

**Non-obvious choices.** Only extra leading is reduced, never the font metrics.
Each ladder attempt measures inside paragraph margins and reuses shaping work
within the calculation.

**Deviations from the design plan.** None. Microscope pass 1 was clean and added
no remediation.

**Spec sections touched.** `docs/hld/08-rendering-spec.md` now defines the
25 percent floor, line-spacing floor, and visible smallest-candidate overflow.

**Tests.** `stored_font_scale_renders_at_exactly_sixty_two_point_five_percent`,
`stored_line_spacing_reduction_reduces_only_extra_leading`,
`bare_normal_autofit_uses_quantised_two_point_five_percent_steps`,
`bare_normal_autofit_keeps_the_twenty_five_percent_floor_visible`, and the
integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep the ladder deterministic and avoid clipping
when even its smallest candidate cannot fit.

### F-101, Vertical text

**Sprint.** S24
**Completed.** 2026-08-05
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Vertical and vertical-270 text reuse horizontal layout in a
transposed content box, then apply opposite centre-preserving quarter turns.
East Asian vertical and other unsupported variants remain visible through
documented rotations and stable diagnostics.

**Non-obvious choices.** The resolver owns fallback diagnostics, while the
renderer receives only concrete direction and applies the affine transform to
one grouped text block.

**Deviations from the design plan.** Microscope pass 1 found an exact renderer
mapping coverage gap. The regression was added, and pass 2 was clean.

**Spec sections touched.** `docs/hld/02-scope-and-non-goals.md` and
`docs/hld/08-rendering-spec.md` now define direction mapping and visible
diagnosed fallbacks.

**Tests.** `vertical_text_uses_a_transposed_box_and_rotated_group`,
`vertical_270_uses_the_opposite_quarter_turn`,
`east_asian_vertical_text_degrades_to_rotated_with_a_diagnostic`,
`other_vertical_variants_remain_visible_with_diagnostics`, and the integrated
full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep vertical fallbacks visible and diagnosed.
Do not add a separate vertical shaping pipeline.

### F-102, Table rendering

**Sprint.** S25
**Completed.** 2026-08-08
**Size.** L, estimated 4 days, actual 1 day

**What was built.** DrawingML table styles, cell fills, margins, borders, and
text styles now parse and preserve unsupported XML, resolve through concrete
table-region precedence, and lower to fills, text, and one physical stroke per
border segment. Two-dimensional merges render only from their origin while
using the correct covered-cell outer edges.

**Non-obvious choices.** Table text style enters the character cascade before
explicit paragraph and run formatting. Corner regions require their matching
row and column options, adjacent borders use one deterministic conflict policy,
and unsupported table paint or cell autofit stays visible through stable
diagnostics.

**Deviations from the design plan.** Microscope passes exposed explicit text
overrides applied in the wrong order, unequal right-to-left column sizing,
merged-edge sourcing, inside-border selection, option-independent corners, and
missing unsupported-paint diagnostics. Each was corrected with a distinguishing
regression before the clean third pass.

**Spec sections touched.** `docs/hld/05-drawingml-model.md`,
`docs/hld/06-presentationml-model.md`,
`docs/hld/07-inheritance-and-resolution.md`,
`docs/hld/08-rendering-spec.md`, and `docs/hld/12-testing-strategy.md`.

**Tests.** `table_style_and_cell_properties_preserve_unmodelled_xml_byte_for_byte`,
`table_style_regions_resolve_in_documented_precedence`,
`table_cell_autofit_is_ignored_and_records_a_diagnostic`,
`banded_merged_table_renders_correct_fills_without_duplicated_borders`,
`merged_continuation_cells_do_not_render_fill_border_or_text_twice`,
`table_cell_margins_place_text_in_the_fixed_content_box`, and the integrated
full workspace gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Resolve logical cell ownership separately from
physical edge ownership. A merge origin owns content, but its far border can
come from the last covered cell.

### F-103, Hyperlinks, fields and diagnostics

**Sprint.** S25
**Completed.** 2026-08-08
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Direct external run hyperlinks now resolve in the producing
slide, layout, or master relationship scope and emit transformed URI link
annotations. Typed and effective slide-number fields substitute the one-based
page number before shaping, while broken or unsupported actions retain visible
text and add stable diagnostics.

**Non-obvious choices.** Hyperlink actions remain direct run state rather than
joining the inheritance cascade. The resolver freezes only the resolved URI,
and the existing text-segment emitter supplies annotation bounds after the
normal recursive transform walk.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** `docs/hld/07-inheritance-and-resolution.md` and
`docs/hld/08-rendering-spec.md`.

**Tests.** `slide_number_field_renders_current_page_and_hyperlink_emits_annotation`,
`same_relationship_id_resolves_hyperlink_in_its_shape_source_scope`,
`missing_hyperlink_relationship_keeps_text_and_records_diagnostic`,
`untyped_slide_number_placeholder_uses_the_current_page_number`,
`grouped_hyperlink_annotation_keeps_transformed_run_bounds`, and the integrated
full workspace gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep package relationship objects upstream of
the frozen renderer contract. Unsupported actions should lose only the
annotation, never their visible text.

### F-104, SSIM fidelity harness

**Sprint.** S25
**Completed.** 2026-08-08
**Size.** L, estimated 4 days, actual 1 day

**What was built.** A deterministic whole-presentation entry point, a concrete
50-deck renderer, and a version-pinned LibreOffice SSIM harness now produce and
retain per-slide fidelity evidence at 150 dpi. CI enforces complete rendering,
records the SSIM trend, uploads the detailed evidence even on failure, and keeps
native PowerPoint review as the hard manual fidelity gate. Evidence-ranked
renderer corrections raised the integrated trend to 30 of 421 slides at or
above 0.95 with median SSIM 0.622465 and zero dropped bounded shapes.

**Non-obvious choices.** SSIM against LibreOffice is a trend rather than a
conformance threshold. Native PowerPoint 16.104 versus the same LibreOffice
oracle reached zero of 34 representative slides at 0.95, with median
0.650406194, so completeness and native review remain the hard gates. The
normal renderer still discovers system fonts, while the evidence entry point
uses bundled fonts exclusively.

**Deviations from the design plan.** Native calibration superseded the initial
hard interpretation of 0.95 SSIM on 80 percent of slides. Microscope required
durable CI evidence uploads and clearer acceptance sources. Completion also
removed an out-of-scope mirrored Word layout change after it produced an
undeclared hash delta.

**Spec sections touched.** `docs/hld/02-scope-and-non-goals.md`,
`docs/hld/07-inheritance-and-resolution.md`,
`docs/hld/08-rendering-spec.md`, `docs/hld/12-testing-strategy.md`,
`docs/hld/14-development-backlog.md`, and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** `test_all_corpus_slides_render_without_panic_or_dropped_shape`,
`test_corpus_render_fidelity_records_ssim_trend`, the seven local metric and
oracle-contract self-tests, exact tool-version assertions, the accepted M10
native PowerPoint spot-check, and the integrated 50-deck gate passed for all
421 slides.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Integrated evidence is retained at
`/private/tmp/s25-integrated-fidelity-escalated-20260808`. On macOS, sandboxed
headless LibreOffice can abort during AppKit application registration before it
opens a deck. The same exact command and deck succeed outside that boundary.

### F-105, Bundled default.pptx

**Sprint.** S26
**Completed.** 2026-08-08
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The unpublished `rpptx` facade now owns a crate-local
zero-slide PowerPoint template and exposes `Presentation::new()` when the
default-on `default-template` feature is enabled. The 16:9 template contains
one master, eleven layouts, a full theme, notes infrastructure, and table
styles.

**Non-obvious choices.** The template is loaded through the normal parser and
serialized through the deterministic package writer. Shipping a reviewed
binary template avoids thousands of lines of write-only theme and layout
construction code.

**Deviations from the design plan.** None. The native PowerPoint gate and both
feature modes passed as planned.

**Spec sections touched.** `docs/hld/02-scope-and-non-goals.md`,
`docs/hld/03-architecture.md`, `docs/hld/06-presentationml-model.md`, and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** `new_presentation_uses_the_bundled_zero_slide_template`,
`bundled_template_has_the_documented_part_graph`, default and no-default
feature checks, package-list inspection, native PowerPoint no-repair
acceptance, and the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep the binary asset inside the `rpptx` crate
and retain its recorded source and licence evidence when replacing it.

### F-106, ShapeIdAllocator and MediaStore

**Sprint.** S26
**Completed.** 2026-08-08
**Size.** M, estimated 2 days, actual 1 day

**What was built.** PresentationML shape trees now expose typed non-visual ids
and allocate fresh ids across root shapes, nested groups, and selected
alternate-content fallbacks. The facade-owned media store deduplicates equal
bytes by content hash and allocates collision-free package part names.

**Non-obvious choices.** Hash equality is confirmed with the original bytes so
a hash collision cannot alias different media. Parsed non-visual properties
remain backed by their preserved raw XML, and allocation starts at id 2 because
the shape-tree root owns id 1 in generated decks.

**Deviations from the design plan.** Microscope pass 1 found that insertion
order could change which duplicate media part was reused. Sorting existing
part names made reuse deterministic, and pass 2 was clean.

**Spec sections touched.** `docs/hld/03-architecture.md`,
`docs/hld/04-opc-and-packaging.md`, and
`docs/hld/06-presentationml-model.md`.

**Tests.** `shape_id_allocator_scans_nested_groups_and_alternate_content`,
`shape_id_allocator_starts_at_two_and_skips_sparse_ids`,
`typed_non_visual_ids_preserve_original_shape_xml`,
`equal_media_bytes_inserted_twice_reuse_one_part`,
`media_store_compares_bytes_inside_a_hash_bucket`,
`media_store_allocates_after_the_highest_existing_suffix`, and the integrated
full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep `MediaStore` private to the facade and keep
the allocator scan aligned with every recursive shape-tree container.

### F-107, add_slide

**Sprint.** S26
**Completed.** 2026-08-08
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `Presentation::add_slide()` now selects a resolved layout,
synthesizes a minimal slide with its non-latent placeholders, assigns unique
shape and slide ids, creates the relative layout and presentation
relationships, registers the content type, and appends the slide to the owned
read model.

**Non-obvious choices.** Placeholder type and `idx` are copied without cloning
the layout XML, which avoids hidden relationship identifiers. Date, footer,
and slide-number placeholders remain latent, while every synthesized text body
contains its required paragraph.

**Deviations from the design plan.** Microscope passes strengthened sparse
part-name allocation and the test oracle for placeholder inheritance. The
native three-slide deck then opened in PowerPoint 16.104 without repair.

**Spec sections touched.** `docs/hld/01-glossary.md`,
`docs/hld/04-opc-and-packaging.md`, `docs/hld/06-presentationml-model.md`, and
`docs/hld/13-risks-and-open-questions.md`.

**Tests.** `three_added_slides_have_unique_ids_and_reopen`,
`add_slide_allocates_after_the_highest_existing_part_suffix`,
`add_slide_synthesizes_only_non_latent_layout_placeholders`,
`synthesized_slide_uses_schema_order_and_one_relative_layout_relationship`,
`synthesized_text_bodies_always_contain_a_paragraph`,
`add_slide_rejects_an_unknown_layout_index_without_mutation`, native
PowerPoint acceptance, and the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Continue using the nine-step package mutation
order and allocate from observed maxima so sparse producer packages are never
overwritten.

### F-108, validate()

**Sprint.** S26
**Completed.** 2026-08-08
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `Presentation::validate()` now returns all twelve exact
package and PresentationML issue variants in deterministic order. It checks
slide and shape ids, text bodies, placeholders, content types, relationships,
media reachability, custom shows, layouts, and themes. `save()` writes the same
deterministic bytes as `to_bytes()`, and both debug save boundaries assert a
clean validation result before writing.

**Non-obvious choices.** Semantic slide-id and empty-text checks are deferred
from parsers so corrupted decks remain inspectable. Namespace-aware XML scans
are observational, uppercase media extensions use case-insensitive default
lookup, and shape traversal uses an explicit heap stack to keep validation
non-panicking for deeply nested trees.

**Deviations from the design plan.** Microscope pass 1 found a hard-coded root
shape id, skipped XML parts whose relationship collection was entirely absent,
and recursive traversal that could overflow the stack. All three were repaired
with distinguishing regressions, and pass 2 was clean.

**Spec sections touched.** `docs/hld/04-opc-and-packaging.md`,
`docs/hld/06-presentationml-model.md`, and
`docs/hld/12-testing-strategy.md`.

**Tests.** `every_validation_issue_variant_detects_its_corrupted_deck`,
`validate_collects_all_issues_in_deterministic_order`,
`all_pinned_corpus_decks_validate_cleanly`,
`debug_save_boundaries_assert_on_invalid_presentations`,
`save_writes_the_same_bytes_as_to_bytes`,
`validation_xml_scan_is_prefix_tolerant_and_non_mutating`, and the integrated
full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep validation observational and preserve
corrupt input long enough to report every issue. Relationship scans must cover
parts even when their entire `.rels` collection is absent.

### F-109, Shape mutation facade

**Sprint.** S27
**Completed.** 2026-08-08
**Size.** L, estimated 4 days, actual 1 day

**What was built.** The presentation facade now exposes borrowed mutable slide
and recursive shape handles. Supported shapes can update position, size,
rotation, name, fill, line, and preset adjustments, and every change survives
save and reload without replacing the owning shape tree.

**Non-obvious choices.** Mutable access follows the typed shape-tree projection
and deliberately leaves selected `AlternateContent` fallback children
read-only. Setters update the narrow typed field while preserving unmodelled
attributes, children, sibling order, and non-visual ids.

**Deviations from the design plan.** Microscope pass 1 found that creating an
absent group transform could place it after preserved group properties. The
repair shifted the raw boundary and strengthened nested-group coverage to prove
two sibling ids and their order remain unchanged. Pass 2 was clean.

**Spec sections touched.** `docs/hld/06-presentationml-model.md`.

**Tests.** `shape_mutation_setters_survive_save_and_reload`,
`shape_mutation_preserves_unmodelled_xml_and_schema_order`,
`shape_mutation_handles_nested_group_children`,
`alternate_content_fallback_is_not_mutable`,
`shape_mutation_indices_and_kinds_are_total`,
`preset_adjustment_setter_inserts_and_replaces_named_values`,
`shape_name_mutation_escapes_xml_and_preserves_children`, and the integrated
full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep behavior-bearing raw-boundary repair beside
the typed mutation that creates an absent schema child. Do not expose mutable
handles into compatibility branches that remain serialized from preserved XML.

### F-110, Shape constructors

**Sprint.** S27
**Completed.** 2026-08-08
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Mutable slides can now append text boxes, preset shapes,
connectors, and empty groups at the top of z-order. Each constructor allocates
a tree-wide unique id, emits a canonical schema-ordered shell, and returns a
borrowed handle to the exact appended shape.

**Non-obvious choices.** The id allocator scans typed members, preserved raw
members, and every markup-compatibility branch using namespace-resolved
`cNvPr` matching. Connector bounds normalize every endpoint direction into
nonnegative extents and flips, and preset names are checked against all 187
pinned ECMA definitions before mutation.

**Deviations from the design plan.** Microscope pass 1 found append ordering
after a trailing extension, opaque-id collisions, unvalidated preset strings,
and incomplete reopen assertions. The repairs added a preservation-aware append
path, a complete raw scan, preset validation, and full geometry assertions.
Pass 2 was clean.

**Spec sections touched.** `docs/hld/06-presentationml-model.md`.

**Tests.** `all_shape_constructors_open_in_powerpoint_without_repair`,
`ordinary_shape_and_textbox_constructors_emit_canonical_shells`,
`connector_constructor_normalizes_every_direction`,
`empty_group_constructor_has_required_children`,
`four_appended_shapes_have_unique_ids_and_reopen`,
`constructor_names_are_deterministic_from_allocated_ids`, pinned PowerPoint
16.104 build 16.104.25121423 acceptance, and the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Append through `CT_ShapeTree::append_child` so
schema-final preserved content remains final. Keep the allocator scan aligned
with every typed and opaque shape-tree container.

### F-111, add_picture

**Sprint.** S27
**Completed.** 2026-08-08
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `Presentation::add_picture` now inserts a picture into a
chosen slide, using exact 72-DPI native dimensions when both axes are omitted
and truncating aspect-ratio inference when one axis is supplied. Equal image
bytes share one media part while each slide retains its own relationship scope.

**Non-obvious choices.** Image bytes determine the stored extension and MIME
type even when the supplied filename is misleading. Package, media, and
relationship changes are staged in clones so every fallible operation finishes
before the live presentation commits an atomic mutation.

**Deviations from the design plan.** None. Microscope pass 1 was clean. The
pinned python-pptx 1.0.2 comparison produced the same picture kind and 12,700 by
12,700 EMU bounds, and PowerPoint 16.104 build 16.104.25121423 opened the deck
without repair.

**Spec sections touched.** `docs/hld/04-opc-and-packaging.md`,
`docs/hld/06-presentationml-model.md`, and
`docs/hld/14-development-backlog.md`.

**Tests.** `picture_without_explicit_size_uses_native_dimensions`,
`picture_constructor_round_trips_in_schema_order`,
`picture_one_dimension_preserves_aspect_ratio_with_truncation`,
`duplicate_picture_bytes_share_one_media_part_across_slides`,
`picture_sniffs_bytes_when_extension_is_misleading`,
`invalid_picture_input_does_not_mutate_the_presentation`,
`picture_native_size_matches_python_pptx_1_0_2`,
`added_picture_validates_and_opens_without_repair`, and the integrated full gate
passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep media deduplication package-wide and image
relationship reuse slide-scoped. Preserve the staged commit boundary whenever
new fallible picture options are added.

### F-112, Text frame mutation

**Sprint.** S27
**Completed.** 2026-08-08
**Size.** L, estimated 4 days, actual 1 day

**What was built.** Ordinary shapes now expose borrowed text-frame, paragraph,
and run handles. Callers can replace or clear text, append paragraphs and runs,
and set typed paragraph, character, Latin font, and bullet properties while
retaining the required nonempty paragraph invariant.

**Non-obvious choices.** Replacing placeholder text preserves placeholder type
and `idx`, body properties, list style, and paragraph-level state. The mutation
surface keeps fields, breaks, raw compatibility content, whitespace intent, and
schema order unless the caller replaces the owning typed value.

**Deviations from the design plan.** Microscope pass 1 found that creating an
absent paragraph property node could leave preserved boundary-0 content before
it. The repair moved that raw boundary behind the new property and added a
complete markup-compatibility run substitution regression. Pass 2 was clean.

**Spec sections touched.** `docs/hld/05-drawingml-model.md` and
`docs/hld/06-presentationml-model.md`.

**Tests.** `setting_text_on_placeholder_round_trips_and_renders`,
`clearing_text_preserves_required_paragraph`,
`paragraph_run_font_and_bullet_properties_round_trip`,
`text_mutation_preserves_placeholder_identity`,
`text_mutation_preserves_unmodelled_xml_and_schema_order`,
`text_mutation_indices_and_shape_kinds_are_total`,
`text_frame_handles_append_paragraphs_and_runs_in_order`, and the integrated
full gate passed with deterministic fonts.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep the minimum one-paragraph invariant in the
DrawingML model and keep raw-boundary shifts local to the property insertion
that changes schema order.

### F-113, Table facade

**Sprint.** S28
**Completed.** 2026-08-09
**Size.** L, estimated 4 days, actual 1 day

**What was built.** Mutable slides can now append tables and expose borrowed
table and cell handles. Callers can edit cell text, fill, margins, banding,
column widths, and rectangular merges, then split a merged origin back into
the original grid while retaining valid table structure.

**Non-obvious choices.** Merge moves source-cell content into the origin in
row-major order, matching pinned python-pptx 1.0.2 semantics. Width updates use
checked EMU arithmetic and keep the table grid synchronized with the graphic
frame extent. Unsupported XML remains at its original schema boundary.

**Deviations from the design plan.** Microscope pass 1 found missing required
paragraphs after content migration, missing constructor validation, and a
preservation test that did not prove byte identity. All three were repaired,
and pass 2 was clean.

**Spec sections touched.** `docs/hld/05-drawingml-model.md` and
`docs/hld/06-presentationml-model.md`.

**Tests.** `merge_then_split_restores_the_original_grid`,
`add_table_round_trips_cells_formatting_banding_and_widths`,
`table_mutation_rejects_invalid_ranges_without_partial_changes`,
`table_mutation_preserves_unmodelled_xml_and_schema_order`,
`table_graphic_frame_constructor_writes_the_canonical_shell`, the pinned
python-pptx 1.0.2 differential gate, and the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep cell-grid mutation transactional. A merged
source must retain a schema-valid text body even after its content moves to the
origin.

### F-114, remove_slide, move_slide, duplicate_slide

**Sprint.** S28
**Completed.** 2026-08-09
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `Presentation` can remove, move, and duplicate slides while
keeping the slide-id list, relationships, parts, content types, notes, custom
shows, media, shape ids, and connector endpoints consistent. Duplicated images
resolve through the new slide's own relationship scope.

**Non-obvious choices.** Duplication remaps typed and preserved relationship
ids, creates fresh notes and back relationships, and allocates fresh shape ids
through compatibility content. Removal prunes only candidate media that is no
longer reachable from any package relationship.

**Deviations from the design plan.** Microscope pass 1 found package-root media
reachability, notes normalization, and compatibility shape-id gaps. Pass 2
found a remaining nonnumeric preserved notes reference. All four defects were
repaired, and pass 3 was clean.

**Spec sections touched.** `docs/hld/04-opc-and-packaging.md` and
`docs/hld/06-presentationml-model.md`.

**Tests.** `duplicated_slides_images_resolve_to_the_new_slides_own_relationships`,
`remove_slide_removes_its_part_relationship_notes_and_custom_show_entries`,
`move_slide_reorders_the_slide_id_list_without_rewriting_relationships`,
`duplicate_slide_rewrites_typed_and_preserved_relationship_ids_without_other_byte_changes`,
`slide_id_list_raw_children_follow_surviving_ids_after_collection_edits`,
`every_corpus_preserved_payload_is_identity_with_an_empty_map`, and the
integrated full gate passed against all 50 pinned decks.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Treat slide collection edits as package-graph
transactions. Relationship ids are scoped by producer part, and media pruning
must include package-root reachability.

### F-115, Slide and presentation properties

**Sprint.** S28
**Completed.** 2026-08-09
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The facade now reads and writes slide size, hidden state,
direct slide backgrounds, and core properties. It can also save presentation
bytes as a slideshow package while preserving a valid part graph and changing
only the main content type.

**Non-obvious choices.** Hidden state follows inverse `p:sld/@show` semantics.
Direct background edits preserve theme references and raw producer XML. Core
properties are created only when the package owns or lacks the conventional
part, avoiding unrelated-part replacement.

**Deviations from the design plan.** Microscope pass 1 found direct-background
replacement losing preserved XML, unsafe reuse of an unowned conventional core
part, and incomplete value and preservation assertions. All three were
repaired, and pass 2 was clean.

**Spec sections touched.** `docs/hld/04-opc-and-packaging.md` and
`docs/hld/06-presentationml-model.md`.

**Tests.** `slide_and_presentation_properties_round_trip`,
`slide_size_mutation_preserves_kind_and_unmodelled_xml`,
`hidden_flag_uses_inverse_show_semantics`,
`background_set_and_clear_preserve_theme_references_and_raw_xml`,
`core_properties_are_loaded_lazily_and_written_with_valid_graph`,
`save_as_show_changes_only_the_main_content_type`, and the integrated full gate
passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep slideshow conversion output-only and keep
core-property creation guarded by relationship ownership rather than a
conventional path alone.

### F-116, Cross-viewer acceptance

**Sprint.** S28
**Completed.** 2026-08-09
**Size.** M, estimated 2 days, actual 1 day

**What was built.** One deterministic ten-slide deck exercises every F-107
through F-115 write feature, validates cleanly, reopens as `.pptx` and `.ppsx`,
and binds four-viewer evidence to SHA-256
`d36da6e8849eabd4487d2572baea19c3716ee7d0fe03aaa4714a28ce3c41de4f`.

**Non-obvious choices.** The ignored gate reruns automatable PowerPoint and
LibreOffice checks, then validates all four tracked evidence rows. Keynote is
user-confirmed human-action evidence because its UI import is not reliably
scriptable. Google Slides is identified by acceptance date and Chrome build,
without retaining the private imported-document URL.

**Deviations from the design plan.** Review hardened the evidence schema
against unobserved counts, pending or blank clean records, vacuous pending
coverage, and shared temporary paths. It also aligned the gate and plan with
the supported Keynote human-action path. The test-only one-pixel PNG was
replaced with a valid precomputed fixture after LibreOffice rejected its IDAT
stream. Microscope pass 8 was clean.

**Spec sections touched.** `docs/hld/12-testing-strategy.md`.

**Tests.** `ten_slide_write_api_deck_validates_and_reopens`,
`ten_slide_write_api_deck_saves_as_presentation_and_show`,
`cross_viewer_acceptance_evidence_is_complete_and_bound_to_one_artifact`,
`generated_ten_slide_write_api_deck_opens_clean_in_all_four_viewers`, pinned
PowerPoint 16.104, Keynote 14.4, Google Slides on 2026-08-09 through Chrome
151.0.7922.76, LibreOffice 26.2.5.2 acceptance, and the integrated full gate
passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep every viewer row bound to the same frozen
artifact SHA. Never promote a pending row without a positive observed open or
import, exact slide count, clean conversion result, and close or export result.

### F-117, oxml-sml workbook writer

**Sprint.** S29
**Completed.** 2026-08-10
**Size.** L, estimated 4 days, actual 1 day

**What was built.** The workspace now contains an unpublished `oxml-sml`
crate that writes one-sheet `.xlsx` packages with string and numeric columns,
shared strings, number formats, defined ranges, deterministic relationships,
and the complete minimal OPC part graph.

**Non-obvious choices.** The API is column-oriented and validates all lengths,
formula ranges, finite numbers, sheet names, shared-string counts, and XML
string escapes before package construction. Shared-string indexes are stable,
and workbook output is byte-identical for the same input.

**Deviations from the design plan.** Microscope review added complete
SpreadsheetML escaping for attribute normalization and reserved sequences,
row-limit validation, aggregate shared-string overflow protection, and an
executable viewer-artifact binding. The approved crate boundary did not
change.

**Spec sections touched.** `docs/hld/09-charts-spec.md` and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** `workbook_package_has_the_minimal_editable_part_graph`,
`formula_ranges_quote_sheet_names_and_track_column_lengths`,
`spreadsheet_strings_escape_xml_and_reserved_sequences_exactly`,
`viewer_gate_candidate_is_bound_to_recorded_sha`, and the integrated full
gate. Excel 16.104 and LibreOffice Calc 26.2.5.2 opened the same artifact
without repair at SHA-256
`8f8d12aa4ebe94f86c8164fd251cdb23845f985090be0fb6c77242aaa0fba329`.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep `oxml-sml` deliberately smaller than a
spreadsheet library. Chart authoring should consume its deterministic workbook
package and formula ranges rather than adding workbook-reading or calculation
features here.

### F-118, ChartML core types

**Sprint.** S29
**Completed.** 2026-08-10
**Size.** L, estimated 4 days, actual 1 day

**What was built.** The workspace now contains an unpublished `rpptx-chart`
crate with typed ChartML space, chart, plot-area, title, legend, flags, shape
properties, and text properties. It reads aliases by namespace URI, writes
fixed `c`, `a`, and `r` prefixes in schema order, and preserves unsupported
ChartML at stable schema boundaries.

**Non-obvious choices.** The core plot shells remain intentionally opaque for
later plot and axis stories. DrawingML text parsing accepts the caller-owned
`c:txPr` root while reusing the existing concrete text-body implementation.
Corpus preservation evidence retains parent path, schema boundary, sibling
order, and exact bytes.

**Deviations from the design plan.** Microscope review hardened schema-slot
stability after public edits, comments and processing instructions, scalar
extension data, nested namespace handling, first-parse preservation evidence,
and trailing root validation. The approved core-only type boundary remained
unchanged.

**Spec sections touched.** `docs/hld/09-charts-spec.md` and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** `chart_space_reads_aliases_and_writes_fixed_prefixes_in_schema_order`,
`core_chart_shells_preserve_unmodelled_children_byte_for_byte`,
`malformed_core_chart_values_return_errors_without_panicking`,
`every_corpus_chart_part_round_trips_structurally`, and the integrated full
gate. The required corpus gate verified 26 chart parts across 9 of 50 pinned
decks.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep plot kinds, axes, and series attached to
the stable raw schema seams until their owning F-IDs type them. Namespace
resolution must remain URI-aware even when output prefixes are canonical.

### F-119, Series and data references

**Sprint.** S29
**Completed.** 2026-08-10
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `rpptx-chart` now models series indexes and order, names,
string and numeric references, category and value data, bubble sizes, formulae,
format codes, and literal caches. Constructors derive cache counts and point
indexes from one value vector, while corpus parsing retains valid sparse
producer caches and unsupported series payloads.

**Non-obvious choices.** Series projection resolves mixed and inherited
prefixes by namespace URI. Preserved payloads use stable schema slots so public
edits cannot reorder markers, labels, error bars, shapes, or extensions. Typed
fixed-prefix rewrites reject conflicting local bindings, and cache edits
reconcile preserved point boundaries without dropping schema-final payloads.

**Deviations from the design plan.** The corpus demonstrated that a logical
point count may exceed the number of cached points when all retained indexes
remain in range, so the plan and HLD were aligned with valid sparse caches.
Nine microscope passes hardened namespace propagation, duplicate wrapper
detection, public-edit behavior, cache resizing, and schema-slot stability
before the independent review became clean.

**Spec sections touched.** `docs/hld/09-charts-spec.md`.

**Tests.** `series_formula_and_cache_are_consistent_with_one_source`,
`string_and_numeric_references_write_fixed_prefixes_in_schema_order`,
`malformed_series_and_cache_values_return_errors_without_panicking`,
`series_preserves_unmodelled_children_byte_for_byte`,
`public_series_edits_do_not_duplicate_or_drop_preserved_payloads`,
`every_corpus_series_round_trips_structurally`, and the integrated full gate.
The required corpus gate verified 66 series across all 26 chart parts in the
50 pinned decks.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Authoring code should supply formulae and value
vectors once and let the ChartML writer derive cache metadata. Plot stories
must preserve the stable series schema slots and URI-aware namespace rules.

### F-120, Axes

**Sprint.** S30
**Completed.** 2026-08-10
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `rpptx-chart` now models category, value, date, and series
axes with scaling, gridlines, titles, number formats, ticks, shape and text
properties, positions, and reciprocal cross-axis references. Plot areas expose
validated typed axes while preserving unsupported children and producer
markup in schema order.

**Non-obvious choices.** Axis identifiers accept the producer-compatible range
from signed 32-bit minimum through unsigned 32-bit maximum because the corpus
contains negative PowerPoint identifiers. Parsed root-family provenance blocks
unsafe relabelling when opaque family-specific content exists, while newly
constructed axes remain freely editable.

**Deviations from the design plan.** Five microscope passes hardened parsed
axis relabelling, constructed versus parsed provenance, structural equality,
and lexical identifier preservation. The approved API and HLD impact did not
change.

**Spec sections touched.** `docs/hld/09-charts-spec.md`.

**Tests.** `axis_id_pairs_are_reciprocal`,
`all_axis_forms_write_fixed_prefixes_in_schema_order`,
`malformed_axis_values_return_errors_without_panicking`,
`axes_preserve_unmodelled_children_byte_for_byte`,
`every_corpus_axis_round_trips_structurally`, and the integrated full gate.
The required corpus gate verified 40 axes across 26 chart parts in all 50
pinned decks.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep axes owned by the plot area and plots
limited to identifier references. Preserve unchanged identifier lexemes for
round-trip output, but compare and validate their normalized numeric values.

### F-123, Data labels and number formats

**Sprint.** S30
**Completed.** 2026-08-10
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Series now carry typed collection-level data labels with
number format, position, separator, and visibility flags. The shared
`NumberFormat` value projects `General`, fixed decimal, and percentage forms
deterministically while preserving unsupported valid producer codes for
round-trip output.

**Non-obvious choices.** Cache source formatting and label formatting remain
separate XML states. Individual point labels, leader lines, shape properties,
text properties, and extensions remain ordered raw payloads. Native glyph
placement remains outside this model boundary.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** `docs/hld/09-charts-spec.md`.

**Tests.** `data_labels_write_fixed_prefixes_in_schema_order`,
`common_number_formats_project_cached_values_deterministically`,
`malformed_data_labels_and_number_formats_return_errors_without_panicking`,
`data_labels_preserve_point_overrides_and_extensions_byte_for_byte`,
`every_corpus_data_label_collection_round_trips_structurally`,
`percentage_formatted_label_renders_with_correct_text`, and the integrated
full gate. The corpus gate verified 34 label collections and 35 axis number
formats. LibreOffice 26.2.5.2 and Poppler 26.01.0 extracted `25%` from candidate
SHA-256 `4ba02faa8e4cff6cefa7a7dc73fc0eb0c08d62d180f83fa0d3fd56a7e4136242`.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Treat `format_value` as a deliberately small
renderer projection, not an Excel format-language engine. F-126 owns label
placement and should consume this typed state without reparsing ChartML.

### F-121, Bar and line plots

**Sprint.** S30
**Completed.** 2026-08-10
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Plot areas now own typed single-family bar and line plots
with validated properties, series, data labels, and exactly two references to
the plot-area axis set. Unsupported 3-D plots and combination choices remain
opaque and byte-preserved.

**Non-obvious choices.** Mutable repeated series and axis references reconcile
preserved raw boundaries through stable exact and positional matching. Parsed
bar and line families cannot be relabelled while incompatible preserved
payload remains. Viewer acceptance compares decoded pixels with a zero-error
threshold rather than adding native chart geometry.

**Deviations from the design plan.** Eight microscope passes hardened malformed
single-family validation, exact viewer equality, typed versus opaque corpus
counts, repeated-child raw boundaries, mutable identity reconciliation,
family replacement, and the binary PPM parser.

**Spec sections touched.** `docs/hld/09-charts-spec.md`.

**Tests.** `bar_and_line_plots_round_trip_and_render`,
`bar_and_line_plots_write_fixed_prefixes_in_schema_order`,
`malformed_bar_and_line_plots_return_errors_without_panicking`,
`unsupported_and_combo_plots_remain_byte_preserved`,
`public_plot_edits_preserve_axes_and_unselected_payloads`,
`every_corpus_bar_and_line_plot_round_trips_structurally`, and the integrated
full gate. The corpus gate verified 11 typed bar plots, 2 typed line plots, and
one preserved bar-line combination. Pinned original and candidate renders had
normalized RGB mean absolute error `0.00000000` for both families.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** A plot owns axis references, not axis objects.
Keep combination choices opaque until a dedicated story can model and validate
the entire choice without partially rewriting it.

### F-122, Pie, doughnut, area, scatter and radar plots

**Sprint.** S30
**Completed.** 2026-08-10
**Size.** L, estimated 4 days, actual 1 day

**What was built.** The typed plot boundary now covers all seven v1 families.
Pie and doughnut plots are axis-free, area and radar plots use paired axes, and
scatter plots map existing numeric category and value caches to `c:xVal` and
`c:yVal`. Unsupported bubble, stock, surface, `ofPie`, and 3-D choices remain
opaque.

**Non-obvious choices.** Scatter wrapper provenance remains private on the
shared public `Series` value, so standalone and plot-level round trips retain
the correct wrappers. Typed families reject both public and preserved bubble
payload. Family-specific raw boundaries remain stable when optional typed
children are inserted or removed.

**Deviations from the design plan.** Three microscope passes hardened optional
child insertion order, standalone scatter wrapper preservation, bubble-only
payload rejection, the malformed-input matrix, and live raw-boundary updates.
The corpus gap for four families remained as designed and is covered by inline
fixtures plus pinned viewer candidates.

**Spec sections touched.** `docs/hld/09-charts-spec.md`.

**Tests.** `remaining_v1_plots_round_trip_and_render`,
`remaining_plot_families_write_fixed_prefixes_in_schema_order`,
`scatter_series_map_numeric_categories_and_values_to_x_and_y`,
`malformed_remaining_plots_return_errors_without_panicking`,
`unsupported_plot_families_and_children_remain_byte_preserved`,
`every_supported_corpus_plot_round_trips_structurally`, and the integrated full
gate. The corpus supplied one typed pie plot. SHA-bound candidates for pie,
doughnut, area, scatter, and radar exceeded the 1,000 nonblank-pixel threshold
with counts of 309,502, 233,915, 308,569, 9,865, and 7,161.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep one public series type and retain scatter
wrapper provenance privately. F-125 owns native geometry for every plot family
and should consume these typed values rather than duplicating ChartML parsing.

### F-124, add_chart

**Sprint.** S31
**Completed.** 2026-08-10
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `Presentation::add_chart` now validates one `ChartData`
value and atomically writes the typed ChartML part, editable workbook, slide
and chart relationships, content-type overrides, and canonical graphic frame.
All seven supported two-dimensional chart families share this authoring path.

**Non-obvious choices.** Chart and workbook part suffixes advance independently
from their greatest occupied positive suffix. Workbook cells and ChartML caches
are derived from the same source value. Every fallible package and slide change
is staged before the live presentation is updated.

**Deviations from the design plan.** None. Three microscope passes hardened
independent part numbering, nonpositive extent rollback, relationship-id
rollover, exact cache-to-cell mapping, and the native PowerPoint gate.

**Spec sections touched.** `docs/hld/09-charts-spec.md`.

**Tests.** `add_chart_writes_complete_relationship_graph`,
`add_chart_uses_collision_free_part_numbers`,
`add_chart_caches_and_workbook_share_one_source`,
`add_chart_rejects_invalid_data_without_mutation`,
`authored_chart_graphic_frame_round_trips`, and the integrated full gate.
`authored_chart_enters_renderer_deterministically` parses the ChartML produced
by the owning facade and proves finite, nonempty, repeatable paths and labels.
Microsoft PowerPoint 16.104, build 16.104.25121423, opened candidate SHA-256
`e6e9f7eef1c774d0414c5d0c3f1202da1a28635b5d089e15455b7adc3f66cb00`
without repair. Edit Data showed the authored Category, Revenue, and Cost
values for North, South, and West.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep package mutation on the owning presentation
facade. Do not split the workbook, ChartML, relationships, and slide frame
across separate public operations.

### F-125, Chart rendering: geometry

**Sprint.** S31
**Completed.** 2026-08-10
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `rpptx-chart` now lowers all seven supported plot families
to finite backend-neutral paths and markers inside stable chart-local bounds.
The geometry covers clustered and stacked bars, line and area blank policies,
pie and doughnut wedges, matched scatter points, and radar polygons.

**Non-obvious choices.** Sparse caches retain logical indexes instead of being
densified. Gap, Zero, and Span are projected without allocating an arbitrary
declared count. Domain normalization scales before subtraction so opposite
finite extremes cannot create nonfinite geometry.

**Deviations from the design plan.** Three microscope passes hardened sparse
blank handling, aggregate overflow, nonfinite scatter categories, and adjacent
finite-coordinate cases. The approved API and HLD impact did not change.

**Spec sections touched.** `docs/hld/03-architecture.md` and
`docs/hld/09-charts-spec.md`.

**Tests.** `bar_chart_rasterises_at_computed_positions`,
`bar_geometry_handles_direction_grouping_gap_and_overlap`,
`line_scatter_and_radar_emit_paths_and_markers`,
`pie_doughnut_and_area_emit_closed_paths`,
`sparse_cache_indexes_preserve_slots_and_scatter_pairing`,
`finite_extremes_never_produce_nonfinite_geometry`, and the integrated full
gate. Deterministic raster checks passed with the new
`rpptx-chart` to `oxml-layout` dependency pointing inward.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep plot geometry independent of package
resolution and final theme colours. F-127 and F-128 own those boundaries.

### F-126, Chart rendering: axes, gridlines and labels

**Sprint.** S31
**Completed.** 2026-08-10
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `render_chart` now adds nice-number value axes, category
axes, gridlines, tick marks, deterministic glyph labels, point-level data
labels, and legends around the shared plot geometry. Radar charts use radial
spokes, perimeter labels, and concentric value grids.

**Non-obvious choices.** Annotation expansion is bounded at 16,384 logical
slots. Label anchors derive from clipped family geometry and retain direction
for degenerate bars and zero-radius radar points. Parsed point overrides affect
rendering while their original raw `c:dLbl` subtrees remain the sole
serialization source.

**Deviations from the design plan.** Seven microscope passes hardened scale
selection, explicit and reversed bounds, sparse joins, effective number
formats, short geometry, radar annotations, and exact direction and margin
coverage. The approved public surface and HLD impact did not change.

**Spec sections touched.** `docs/hld/09-charts-spec.md`.

**Tests.** `zero_to_one_hundred_axis_uses_expected_ticks`,
`nice_number_ticks_cover_unpinned_extents`,
`axes_gridlines_and_tick_marks_follow_model_state`,
`labels_and_legend_shape_with_deterministic_fonts`,
`inside_and_outside_label_positions_follow_family_geometry`,
`radar_annotations_use_spokes_perimeter_labels_and_radial_gridlines`,
`point_label_overrides_render_without_changing_preserved_xml`,
`labelled_chart_raster_is_deterministic`, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Preserve the shared plot rectangle and z-order
of gridlines, clipped plot, axes, ticks, legend swatches, and text. Keep every
text run on the caller-provided deterministic `FontManager`.

### F-127, Chart colour resolution

**Sprint.** S32
**Completed.** 2026-08-10
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Presentation charts now resolve direct series fill and
line paint before a mapped accent1 through accent6 cycle. The effective theme,
colour map, theme-slot transforms, series transforms, and alpha flow through
every geometry family, marker, data label, and legend swatch.

**Non-obvious choices.** Direct `a:noFill` remains transparent. Filled area and
radar paths compose their established 55 percent policy with resolved alpha.
Only the selected concrete theme slot is required, so unrelated theme entries
cannot block a direct series colour.

**Deviations from the design plan.** None. Three microscope passes hardened
transparent filled plots, unused theme slots, and the order of theme-slot and
series transforms.

**Spec sections touched.** `docs/hld/09-charts-spec.md`.

**Tests.** `unstyled_four_series_use_accent_one_through_four`,
`direct_series_solid_colour_overrides_theme_accent`,
`series_accent_cycle_repeats_after_six`,
`series_colours_honor_colour_map_and_transform_order`,
`unsupported_direct_series_paint_is_contextual`,
`resolved_chart_palette_raster_is_deterministic`,
`authored_chart_enters_renderer_deterministically`, and the integrated full
gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep presentation chart colour resolution on
the exact DrawingML pipeline. The deliberately naive Word tint and shade helper
remains outside this path.

### F-128, Preserved chart fallback

**Sprint.** S32
**Completed.** 2026-08-10
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Presentation package rendering now resolves chart
relationships in slide, layout, and master scope. Supported charts enter the
ordinary renderer as frozen backend-neutral groups. Unsupported charts use a
compatible immediate cached picture or a labelled placeholder with a stable
diagnostic.

**Non-obvious choices.** AlternateContent and ChartML raw bytes remain the only
serialization source. Cached preview admission matches sniffed MIME and backend
capabilities, accepts only 8-bit three-component JPEG, and caps encoded,
decoded, and PNG inflation storage at 16 MiB before decoding. Integration tests
and the corpus driver call the same package-rendering function.

**Deviations from the design plan.** Seven microscope passes hardened missing,
malformed, corrupt, mismatched, sparse, oversized, grayscale, and CMYK cached
previews. They also tightened schema-positioned chart projection, immutable raw
payload access, source-scoped resolution, and production-path test coverage.
The approved routing and HLD scope did not change.

**Spec sections touched.** `docs/hld/03-architecture.md`,
`docs/hld/06-presentationml-model.md`,
`docs/hld/07-inheritance-and-resolution.md`,
`docs/hld/08-rendering-spec.md`, and `docs/hld/09-charts-spec.md`.

**Tests.** `three_dimensional_chart_uses_cached_image_and_diagnostic`,
`authored_chart_relationship_enters_presentation_renderer`,
`same_chart_relationship_id_is_scoped_to_its_source_part`,
`unsupported_chart_without_preview_keeps_labelled_bounds`,
`missing_or_external_chart_relationship_is_contextual`,
`chart_choice_and_picture_fallback_remain_byte_preserved`,
`non_chart_choice_with_descendant_chart_uri_remains_opaque`,
`supported_and_fallback_charts_render_deterministically`, and the integrated
full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Treat cached chart previews as untrusted package
media. Keep admission bounded and aligned with every backend before allowing a
preview to suppress the visible labelled fallback.

### F-047, Packaging include and size gate

**Sprint.** S32.1
**Completed.** 2026-08-11
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `oxml-layout` is now a publication candidate with an
explicit package boundary containing its source, all 20 bundled TTFs, three
family licence files, and the Caladea notice. CI packages the crate, verifies
the archive, compares its exact inventory, and rejects archives above 10 MiB.

**Non-obvious choices.** The gate compares the package list to an exact sorted
inventory instead of checking only globs. This makes a missing legal file and
an accidental extra asset equally visible.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/15-build-and-toolchain.md`, "Packaging"
and "CI job matrix".

**Tests.** `cargo package -p oxml-layout --list`, verified workspace packaging,
the exact inventory assertion for 20 TTFs and four legal files, the 3,596,626
byte archive-size assertion, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep `NOTICE-Caladea` beside the full Apache
licence. The archive ceiling applies to the compressed `.crate`, while the
inventory check proves the required uncompressed assets are present.

### F-048, Automate split-family release preparation

**Sprint.** S32.1
**Completed.** 2026-08-11
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Cargo release metadata now prepares the inherited stable
rdocx train and the explicit incubating shared and PowerPoint train as separate
version groups with `v{{version}}` and `rpptx-v{{version}}` tag templates.
Preparation consolidates each version change while publication, tags, pushes,
and README replacements remain disabled.

**Non-obvious choices.** Stable packages use cargo-release's effective
`workspace` group because they inherit the root version. The 12 incubating
packages use the named `incubating` group because their versions are explicit.

**Deviations from the design plan.** Microscope pass 1 corrected the stable
metadata to cargo-release's effective `workspace` group. The approved family
boundary and external-action restrictions did not change.

**Spec sections touched.** `docs/hld/15-build-and-toolchain.md`, "Release
process".

**Tests.** Cargo-release 1.1.3 configuration assertions, the workflow
regression suite, a disposable stable preparation from 0.4.1 to 0.4.2, a
disposable incubating preparation from 0.0.0 to 0.1.0, manifest and lockfile
diff inspection, `cargo metadata --no-deps`, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** `/release` remains the sole authority for real
release tags and publication. Cargo-release prepares reviewed version commits
only.

### F-049, Extend publish.yml to the extracted workspace

**Sprint.** S32.1
**Completed.** 2026-08-11
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The publication workflow now accepts stable `v*` and
incubating `rpptx-v*` tags, preflights the exact 19-package publishable union,
and routes each namespace to its own explicit dependency-ordered allowlist.
The release command validates and creates only the requested family tag after
the reviewed-SHA and separate final-approval gates.

**Non-obvious choices.** The workspace dry run patches all 19 internal
dependencies to reviewed local sources. Cargo otherwise rewrites packaged path
dependencies to crates.io, where the reserved incubating 0.0.0 packages expose
no API. The patches verify the source graph without entering any archive or
weakening archive verification.

**Deviations from the design plan.** Microscope pass 1 added the missing
incubating `/release` authority and stronger exact workflow mutations. The
integrated packaging gate then disproved Cargo's assumed automatic local
staging, so the plan was corrected to require the exact local patch set and a
third clean microscope pass.

**Spec sections touched.** `docs/hld/11-migration-plan.md`, "Release tooling",
and `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".

**Tests.** Twenty-one workflow regression tests including swapped predicates,
extra and missing packages, a missing local patch, `continue-on-error`, and
successful fallback mutations. The locally patched
`cargo publish --workspace --dry-run` verified all 19 candidates without an
upload, every archive remained below 10 MiB, and the integrated full gate
passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep local patches on the dry-run preflight
only. Real dependency-ordered publish commands stay bare and wait for each
registry layer before publishing its consumers.

### F-050, CI matrix additions

**Sprint.** S32.1
**Completed.** 2026-08-11
**Size.** S, estimated 1 day, actual 1 day

**What was built.** CI now runs the `oxml-layout` no-default-features path, the
supported `rdocx-wasm` target check, and separate prose and generated-skill
drift checks. Every workspace all-feature test, lint, docs, and MSRV command
excludes the two Python extension packages.

**Non-obvious choices.** `rpptx-wasm` remains absent until F-138. The PyO3
exclusions are carried on every all-feature job because extension-module test
binaries cannot link against host Python symbols on Linux.

**Deviations from the design plan.** Microscope pass 1 found missing binding
exclusions on the clippy and docs jobs. The remediation made the exclusion
contract uniform across all all-feature jobs.

**Spec sections touched.** `docs/hld/12-testing-strategy.md`, "CI matrix", and
`docs/hld/15-build-and-toolchain.md`, "CI job matrix".

**Tests.** Current and MSRV workspace checks, exact CI command inspection,
`cargo test -p oxml-layout --no-default-features`,
`cargo check --target wasm32-unknown-unknown -p rdocx-wasm`, prose and skill
sync checks, and the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Add `rpptx-wasm` to the target job only when
F-138 creates that package. Keep the binding exclusions synchronized across
every new all-feature job.

### F-X005, Tag rpptx-v0.1.2

**Sprint.** S32.2
**Completed.** 2026-08-11
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The complete 12-crate incubating family is published at
0.1.2 under the `rpptx-v0.1.2` tag. Every selected manifest has non-empty
release metadata, and the publication workflow runs a self-contained metadata
regression before its archive checks and dependency-ordered crates.io uploads.
The matching GitHub release targets the reviewed sprint commit.

**Non-obvious choices.** The immutable `rpptx-v0.1.0` tag remains the partial
publication that contains only `oxml-core` 0.1.0. The immutable
`rpptx-v0.1.1` tag remains the CI-only failed recovery. A new 0.1.2 family was
required because release tags and published registry versions are never moved
or overwritten.

**Deviations from the design plan.** None. The approved 0.1.2 recovery ran as
designed after a fresh full verification, clean sprint review, and separate
final approval.

**Spec sections touched.** `docs/hld/03-architecture.md`, "Version trains",
`docs/hld/14-development-backlog.md`, "F-X005, Tag rpptx-v0.1.2", and
`docs/hld/15-build-and-toolchain.md`, "Packaging" and "Release process".

**Tests.** The targeted 0.1.2 metadata regression, all workflow regressions,
`cargo metadata --no-deps`, the exact patched 19-package publication dry run,
archive size and bundled asset assertions, supply-chain checks, the full
workspace gate, and all 28 output hashes passed. GitHub Actions run 31496676517
published all 12 packages and created the release. Independent `cargo info`
and owner checks confirmed every 0.1.2 registry entry under `mantissaman`, and
the annotated GitHub tag resolved to commit
`27a8bb8aa494759568d40bf66c167c214e759500`.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Released rdocx consumers may now cut over to
the 0.1.2 shared crates without local registry patches. The stable rdocx family
was not published by this tag.

### F-015, rdocx-oxml becomes a facade

**Sprint.** S32.2
**Completed.** 2026-08-11
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `rdocx-oxml` now re-exports the shared `oxml-core` XML,
raw XML, unit, and property implementations while preserving the established
Word-facing module paths. Five duplicate source files were removed without
call-site changes.

**Non-obvious choices.** The namespace facade retains the Word-specific
constants beside shared namespace helpers. This preserves the public surface
without moving Word vocabulary into the format-neutral crate.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/11-migration-plan.md`, "Consumer
cutovers", and `docs/hld/14-development-backlog.md`, "F-015".

**Tests.** Focused `rdocx-oxml` tests, dependency direction and package checks,
the integrated workspace gate, and all 28 output hashes passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Add format-neutral XML primitives to
`oxml-core`. Keep Word namespace vocabulary in the facade.

### F-016, Length re-export

**Sprint.** S32.2
**Completed.** 2026-08-11
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `rdocx::Length` is now the shared `oxml_core::Length` type.
The duplicate Word implementation was deleted, and every existing constructor,
accessor, conversion, and caller continues through the retained public path.

**Non-obvious choices.** The crate re-exports the type directly instead of
wrapping it. This preserves type identity and keeps the conversion behavior in
one implementation.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/11-migration-plan.md`, "Consumer
cutovers".

**Tests.** Shared unit conversion tests, focused rdocx checks, package
verification, the integrated workspace gate, and all 28 output hashes passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Use `oxml_core::Length` internally and retain
`rdocx::Length` as the compatibility import path.

### F-022, rdocx-opc deprecation shim

**Sprint.** S32.2
**Completed.** 2026-08-11
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `rdocx-opc` is now a deprecated exact re-export shim over
`oxml-opc`. The rdocx library, CLI, and WASM consumer use the shared crate
directly, construct Word packages explicitly, and expose the shared OPC error
type through the high-level error surface.

**Non-obvious choices.** Word-specific new-document setup remains in rdocx.
The shared OPC crate owns generic package mechanics and does not gain a reverse
dependency on a document format.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/10-bindings-spec.md`, "WebAssembly",
`docs/hld/11-migration-plan.md`, "Consumer cutovers", and
`docs/hld/15-build-and-toolchain.md`, "Published crate graph".

**Tests.** Shared error identity, new-document graph, CLI and WASM checks,
package verification, dependency direction, the integrated workspace gate,
and all 28 output hashes passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** New consumers should depend on `oxml-opc`.
Retain `rdocx-opc` only for compatibility through the next stable transition.

### F-027, rdocx adopts oxml-media

**Sprint.** S32.2
**Completed.** 2026-08-11
**Size.** M, estimated 2 days, actual 1 day

**What was built.** rdocx now uses `oxml-media` for byte-first image format
detection, collision-safe media naming, MIME resolution, and downstream media
extraction. Duplicate local media helpers were removed. Mislabelled JPEG bytes
now produce a JPEG part, content type, and relationship target regardless of
the supplied filename extension.

**Non-obvious choices.** When loaded content-type defaults conflict with the
sniffed bytes, rdocx writes a per-part override. This changes only the new media
part and preserves the existing package default for other parts.

**Deviations from the design plan.** Microscope pass 1 found that replacing a
loaded package default could relabel unrelated existing parts. The remediation
used a per-part override and added a loaded-package regression before pass 2
returned clean.

**Spec sections touched.** `docs/hld/03-architecture.md`, "Crate boundaries",
`docs/hld/04-opc-and-packaging.md`, "Media", `docs/hld/11-migration-plan.md`,
"Consumer cutovers", and `docs/hld/14-development-backlog.md`, "F-027".

**Tests.** The exact mislabelled-JPEG package regression, loaded-package
content-type preservation, naming regressions, package verification,
dependency direction, the integrated workspace gate, and all 28 output hashes
passed.

**Hash harness.** Unchanged. All 28 integrated entries match. The intentional
metadata behavior is covered by the focused package regression because the
harness does not inspect that part name or content type.

**Notes for future sessions.** Resolve media metadata from bytes first. Treat
caller extensions as hints and preserve unrelated loaded-package defaults.

### F-028, add_picture_auto

**Sprint.** S32.2
**Completed.** 2026-08-11
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The rdocx write API now provides `add_picture_auto`, which
probes image metadata and inserts the native EMU dimensions using a 72 DPI
caller default. Invalid or unsupported image data returns a typed error before
any document mutation.

**Non-obvious choices.** The method computes dimensions first and delegates a
successful insertion to the existing explicit-size path. This keeps numbering,
relationships, and drawing construction in one implementation.

**Deviations from the design plan.** None.

**Spec sections touched.** `docs/hld/04-opc-and-packaging.md`, "Native image
sizing", and `docs/hld/14-development-backlog.md`, "F-028".

**Tests.** Exact 72 DPI extent and round-trip checks, declared and fallback DPI
cases, atomic failure coverage, explicit-size regressions, package verification,
the integrated workspace gate, and all 28 output hashes passed.

**Hash harness.** Unchanged. Existing samples continue to use explicit sizes.

**Notes for future sessions.** Keep the convenience API additive and preserve
the probe-before-mutation boundary.

### F-046, rdocx layout and PDF cutover

**Sprint.** S32.2
**Completed.** 2026-08-11
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `rdocx-layout` retains the Word flow model while converting
its result into shared `oxml-layout` pages, elements, fonts, media IDs, and
diagnostics. `rdocx-pdf` is now an exact deprecated shim over `oxml-pdf`, and
the high-level render path uses the shared backend. Duplicate neutral layout,
font, media, and PDF backend sources and bundled font assets were removed.

**Non-obvious choices.** A concrete conversion function is the only boundary
between Word flow layout and shared output. It preserves the pre-cutover Word
glyph slices and line-height behavior while using shared output types and
renderers.

**Deviations from the design plan.** Initial integration exposed four Word PNG
deltas. The converter was corrected to preserve the established wrap and line
height semantics, returning all 28 hashes to baseline. Microscope pass 1 then
found an empty-image omission regression, which was restored before pass 2
returned clean. Sprint review pass 4 found that distinct image bytes could
overwrite each other when their compact `MediaId` values collided. Collision
resolution now assigns deterministic alternate IDs, and a forced-collision
regression covers both inline and anchored images. Sprint review pass 5 then
found repeated registry construction for each image occurrence. The final path
builds the relationship and media maps once per layout and reuses them through
paragraphs, tables, headers, footers, footnotes, shapes, and pagination.
Sprint review pass 6 found that the lower-level public entry points could not
share that private result. `MediaRegistry` is now their common public argument,
so direct callers retain both collision-resolved IDs and their image bytes.

**Spec sections touched.** `docs/hld/03-architecture.md`, "Rendering
boundaries", `docs/hld/08-rendering-spec.md`, "Word conversion boundary",
`docs/hld/11-migration-plan.md`, "Consumer cutovers", and
`docs/hld/15-build-and-toolchain.md`, "Packaging".

**Tests.** Exact conversion cases, layout and PDF suites, both no-default
layout paths, WASM, dependency direction, archive inventory, the integrated
workspace gate, and all 28 output hashes passed.

**Hash harness.** Unchanged. All 28 integrated entries match after preserving
the established Word conversion semantics.

**Notes for future sessions.** Keep format-specific flow logic in
`rdocx-layout`. Add neutral output and backend behavior to the shared crates.

### F-051, CHANGELOG and migration notes

**Sprint.** S32.2
**Completed.** 2026-08-11
**Size.** S, estimated 1 day, actual 1 day

**What was built.** A root `CHANGELOG.md` now documents the Unreleased stable
rdocx cutover, the published shared 0.1.2 family, retained facades, deprecated
shims, breaking surfaces, and automatic picture sizing. The README crate table
names the shared replacements and links the migration notes.

**Non-obvious choices.** One migration table covers every moved or deprecated
crate. This keeps version and compatibility guidance in a single durable
artifact.

**Deviations from the design plan.** Sprint review pass 4 found three retained
`rdocx-layout` breaking changes missing from the migration notes. The completed
table now documents the shared `MediaRegistry` argument on lower-level layout
and pagination, `AnchoredContent::Image` media ID, and `ParagraphBlock::jc`
alignment type. Sprint review pass 7 corrected those function references to
their retained `engine`, `table`, and `paginator` module paths.

**Spec sections touched.** None. The documentation reflects the completed HLD
contract without changing system intent.

**Tests.** Exact migration-path and version assertions, rustdoc with warnings
denied, prose checks, the integrated workspace gate, and all 28 output hashes
passed.

**Hash harness.** Unchanged. Documentation does not affect generated output.

**Notes for future sessions.** Keep the stable rdocx train under Unreleased
until its own release workflow runs. Do not imply that `rpptx-v0.1.2` published
the stable family.

### F-129, oxml-py-support

**Sprint.** S33
**Completed.** 2026-08-12
**Size.** M, estimated 2 days, actual 1 day

**What was built.** A new unpublished `oxml-py-support` crate provides ordered
Word `ContentPath` and `PathSeg` values, `RevisionCounter`, a concrete Rust
`StaleElementError`, and canonical positive and negative `Length` conversions
delegated to `oxml-core`.

**Non-obvious choices.** The shared crate owns stale-domain classification but
accepts caller-supplied recovery guidance. Package-specific wording and Python
exception inheritance remain in the consuming binding. Presentation path
variants remain deferred until F-136 has a concrete consumer.

**Deviations from the design plan.** The approved plan was revised to include
`docs/hld/03-architecture.md` after its crate summary assigned the `Length`
pyclass incorrectly. Microscope pass 1 added caller-owned recovery guidance.
Pass 2 added `docs/hld/15-build-and-toolchain.md`, release metadata, and the
updated workspace-version package count.

**Spec sections touched.** `docs/hld/03-architecture.md`, "Three families, one
workspace", `docs/hld/10-bindings-spec.md`, "The chosen design", "The
invalidation problem, handled loudly", and "Python API shape",
`docs/hld/14-development-backlog.md`, "F-129, oxml-py-support" and "F-132,
Python enums, units and exceptions", and `docs/hld/15-build-and-toolchain.md`,
"Release process".

**Tests.** `stale_path_reports_both_revisions`, current-revision acceptance,
revision bumping, ordered Word paths, positive and negative Length truncation,
release-family metadata, focused crate checks, and the integrated full gate
passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep this crate format-neutral. Package bindings
own recovery paths and Python exception classes. Add presentation path variants
only with F-136's concrete consumer.

### F-130, rdocx-py core

**Sprint.** S33
**Completed.** 2026-08-12
**Size.** L, estimated 4 days, actual 1 day

**What was built.** A new unpublished mixed `rdocx-py` package exposes
`Document()`, `Document(path)`, lazy paragraph and run collections, path-only
handles, Python indexing, slicing and iteration, structural mutation, byte
round trips, and named stale-handle failures.

**Non-obvious choices.** `PyDocument` owns the Rust document while every child
handle stores only a Python document reference and an F-129 content path.
Immutable run reads use total facade accessors and preserve both layout caches.
Revision counters advance only after successful structural mutations.

**Deviations from the design plan.** The approved plan was revised to include
release-family metadata, its count regression, and
`docs/hld/15-build-and-toolchain.md`. Microscope pass 1 added the documented
optional path constructor and immutable cache-preserving run accessors. F-130
kept only the temporary stale exception bridge required by its gate, and F-132
replaced it with the final hierarchy. The consolidated gate upgraded PyO3 to
the first fixed 0.29.0 release after two RustSec advisories blocked completion.

**Spec sections touched.** `docs/hld/03-architecture.md`, "Facade conventions",
`docs/hld/10-bindings-spec.md`, "The chosen design" and "The invalidation
problem, handled loudly", and `docs/hld/15-build-and-toolchain.md`, "Release
process".

**Tests.** `stale_paragraph_after_structural_removal_raises_named_error`,
`constructor_accepts_an_optional_input_path`,
`immutable_run_accessors_preserve_cached_layout`, lazy collection coverage,
total facade accessors, byte round trips, 31 installed-package tests, and the
integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep lazy handles path-only and re-resolve them
on every operation. Read-only binding access must stay on immutable facade
methods so it cannot invalidate layout caches.

### F-131, rdocx-py formatting and tables

**Sprint.** S33
**Completed.** 2026-08-12
**Size.** L, estimated 4 days, actual 1 day

**What was built.** Path-only font and paragraph-format subhandles expose the
bounded S33 formatting inventory with tri-state clearing. Lazy table, row,
cell, and nested paragraph handles expose table and cell formatting through
total public facade accessors.

**Non-obvious choices.** Binding-only underline variants use a bounded
integer-code facade API so the published exhaustive Rust `UnderlineStyle` enum
remains compatible. Signed Python indentation clearing is separate from the
established Rust helper. Cell text replacement invalidates nested handles with
exactly one revision bump.

**Deviations from the design plan.** There was no approved scope or HLD
deviation. Microscope remediation restored exhaustive underline and legacy
indent semantics, added single-bump cell invalidation and full tri-state
clearing coverage, made unrepresentable table justification and automatic font
colour read as `None`, proved rejected underline values do not mutate state,
and supplied complete recovery paths for stale nested paragraphs.

**Spec sections touched.** `docs/hld/03-architecture.md`, "Facade conventions",
`docs/hld/10-bindings-spec.md`, "Python API shape", and
`docs/hld/14-development-backlog.md`, "F-131, rdocx-py formatting and tables".

**Tests.** `unset_run_bold_is_none`, `none_clears_direct_formatting`,
`facade_table_and_tristate_accessors_are_total`,
`established_underline_enum_and_first_line_indent_remain_compatible`,
`cell_text_replacement_invalidates_nested_run_and_font`, table reopen tests,
the installed binding suite, and the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Preserve public exhaustive enums and use checked
binding-only integer accessors for a wider Python literal set. Structural
replacement must bump the revision exactly once, and nested stale errors must
name the complete recovery path.

### F-132, Python enums, units and exceptions

**Sprint.** S33
**Completed.** 2026-08-12
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Pure-Python immutable `Length` integer subclasses,
`RGBColor`, four bounded `IntEnum` families, compatibility import paths,
top-level exports, and the public `RdocxError` hierarchy now ship in the mixed
package. Rust package, XML, stale, and layout failures map to their exact
concrete Python classes.

**Non-obvious choices.** Pure-Python value types preserve the Python 3.9
limited ABI while native conversion helpers retain truncation toward zero. A
direct `oxml-layout` dependency exists only under dev dependencies so a private
Rust test can construct the concrete layout failure and prove exact mapping.

**Deviations from the design plan.** There was no product or HLD scope
deviation. Microscope remediation added exact `LayoutError` mapping sensitivity
and its inward, test-only `oxml-layout` dependency, then documented the added
dependency-graph rider in the plan.

**Spec sections touched.** `docs/hld/10-bindings-spec.md`, "Python API shape",
and `docs/hld/14-development-backlog.md`, "F-132, Python enums, units and
exceptions".

**Tests.** `alignment_center_and_inches_match_python_contract`,
`length_is_an_int_with_unit_properties`, fractional truncation, exact enum
values and docs, exception hierarchy,
`layout_error_maps_to_the_exact_public_layout_error_class`, installed abi3
package tests, dependency-direction checks, and the integrated full gate
passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep Python value and enum types outside the
native ABI boundary. Preserve the direct `oxml-layout` edge as dev-only unless
a separately designed production dependency requires it.

### F-133, rdocx-py rendering with allow_threads

**Sprint.** S33
**Completed.** 2026-08-12
**Size.** S, estimated 1 day, actual 1 day

**What was built.** `Document.to_pdf`, `to_bytes`, `render_page_to_png`, and
`render_all_pages` convert Python arguments before releasing the GIL, perform
only Rust-owned work while detached, then rebuild Python bytes, lists, and
mapped exceptions after reattachment.

**Non-obvious choices.** The concurrency gate uses independent uncached
nontrivial documents and compares equivalent serial and parallel work. It
validates complete PDFs and extracted semantics through exactly pinned Poppler
26.01.0 instead of treating cache timing or byte identity as the oracle.

**Deviations from the design plan.** An approved plan revision added the
pinned Poppler differential-test oracle without changing implementation or HLD
scope. Review remediation added semantic equivalence and completeness checks,
progress-sensitive GIL gates for all four methods, exact version parsing for
both Poppler tools, and rejection of suffixed unreviewed versions.

**Spec sections touched.** None. The implementation fulfills the existing
binding threading contract without changing architectural intent.

**Tests.** `four_concurrent_to_pdf_calls_are_faster_than_serial`,
`poppler_pdf_oracle_is_available_at_reviewed_version`,
`poppler_version_pin_rejects_unreviewed_suffix`, the additional
`releases_gil_for_python_worker` gates, result and error mapping tests, 31
installed binding tests, and the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Every blocking native Python method needs a
progress-sensitive GIL-release regression. Keep correctness comparisons
outside timing assertions, require complete outputs, and pin external semantic
oracles to an exact reviewed version.

### F-134, Type stubs and py.typed

**Sprint.** S34
**Completed.** 2026-08-13
**Size.** M, estimated 2 days, actual 1 day

**What was built.** Both mixed Python packages now ship hand-written native
extension stubs and zero-byte `py.typed` markers. Strict consumer programs and
live `stubtest` coverage describe lazy handles, collections, units, enums,
optional values, and factory-only construction exactly.

**Non-obvious choices.** Python 3.12 runs exact `mypy==2.3.0` because that mypy
release no longer supports Python 3.9. The generated extensions retain the
cp39-abi3 floor and were installed separately before the typing gates.

**Deviations from the design plan.** Review remediation narrowed rpptx shape
and length types, included every inline-typed module in strict checking, and
made all non-root native handles statically non-constructible.

**Spec sections touched.** `docs/hld/10-bindings-spec.md`, Packaging,
`docs/hld/12-testing-strategy.md`, Python bindings, and
`docs/hld/14-development-backlog.md`, F-134.

**Tests.** Fresh rdocx and rpptx wheels contained the expected stubs and
markers. Strict mypy passed seven rdocx and six rpptx sources. Stubtest passed
six rdocx and five rpptx modules. The integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Change a native signature and its hand-written
stub in the same story, then run both strict installed-wheel checks and live
stubtest.

### F-135, python-docx parity suite

**Sprint.** S34
**Completed.** 2026-08-13
**Size.** M, estimated 2 days, actual 1 day

**What was built.** A tagged `python-docx==1.2.0` parity suite covers seventeen
documented examples within the delivered rdocx surface. Both libraries author
documents, both libraries read both outputs, and normalized paragraphs, runs,
formatting, tables, cells, units, and enums must agree.

**Non-obvious choices.** The held-row Quickstart example performs one declared
public re-fetch after a structural cell write. This preserves strict global
revision invalidation while keeping the remaining sixteen example bodies as
namespace-only substitutions.

**Deviations from the design plan.** Review expanded the manifest from sixteen
to seventeen tagged examples, distinguished relative line spacing from length
spacing, and moved table-style coverage into both saved writer paths.

**Spec sections touched.** `docs/hld/02-scope-and-non-goals.md`,
`docs/hld/10-bindings-spec.md`, `docs/hld/12-testing-strategy.md`, and
`docs/hld/14-development-backlog.md`.

**Tests.** The documented-example gate, bidirectional saved round trip, oracle
pin, manifest, line-spacing, and table-style mutation gates passed against a
fresh installed cp39-abi3 wheel. The integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep the parity manifest bounded and tied to a
tagged upstream source. Compare public structure, never package bytes or XML.

### F-136, rpptx-py

**Sprint.** S34
**Completed.** 2026-08-13
**Size.** L, estimated 4 days, actual 1 day

**What was built.** The unpublished `rpptx-py` mixed package exposes lazy
presentation, slide, shape, text, and table handles over the Rust facade. It
ships Python-compatible lengths, the bounded shape enum, mirrored exceptions,
and seven Getting Started examples with two-way `python-pptx==1.0.2`
structural comparison.

**Non-obvious choices.** Every handle and collection is path-only and carries
one captured global revision. Successful structural mutation invalidates all
previous views, including the mutating receiver. Recovery messages are derived
from the concrete repeated-shape path.

**Deviations from the design plan.** The documented examples gained the
minimal required public re-fetches after structural writes. Review also fixed
the omitted placeholder-index default, shape value 51 compatibility, complete
writer-drift sensitivity, and nested recovery paths.

**Spec sections touched.** `docs/hld/03-architecture.md`,
`docs/hld/06-presentationml-model.md`, `docs/hld/10-bindings-spec.md`,
`docs/hld/12-testing-strategy.md`, `docs/hld/14-development-backlog.md`, and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** Ten installed binding tests, six shared path tests, 103 rpptx tests,
the seven-example bidirectional differential, exhaustive stale-view probes,
WASM isolation, dependency trees, and the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Do not add owner references or revision
bypasses to preserve source compatibility. Re-fetch explicitly after every
structural write.

### F-137, wheels.yml

**Sprint.** S34
**Completed.** 2026-08-13
**Size.** M, estimated 2 days, actual 1 day

**What was built.** A pinned GitHub Actions workflow builds both distributions
as cp39-abi3 wheels for six approved platform targets, plus one source
distribution per package. It validates compatible wheels in fresh
environments and reserves PyPI trusted publication for successful `py-v*` tag
runs in the `pypi` environment.

**Non-obvious choices.** Only the final publication job receives
`id-token: write`. The reviewed workflow has a raw-byte SHA-256 attestation in
addition to structural semantic tests, so any unreviewed workflow-byte change
fails closed before its release graph can be trusted.

**Deviations from the design plan.** Review hardened the workflow contract
against 155 non-vacuous matrix, execution, permission, action-pin, artifact,
trigger, and publication mutations. Native wheels and source distributions
were built locally, while the first real hosted cross-platform run remains
future GitHub evidence as planned.

**Spec sections touched.** `docs/hld/10-bindings-spec.md`,
`docs/hld/12-testing-strategy.md`, `docs/hld/14-development-backlog.md`, and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** Workflow contract and mutation tests, two native wheels, two source
distributions, clean imports, strict typing, stubtest, archive inventory, and
the integrated full gate passed. No tag, dispatch, or publication occurred.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Any intentional workflow edit must update the
semantic contract and its reviewed raw-byte digest in the same reviewed story.

### F-138, PR-time Python job

**Sprint.** S34
**Completed.** 2026-08-13
**Size.** S, estimated 1 day, actual 1 day

**What was built.** Pull requests now run one fail-fast-disabled two-package
matrix that creates a fresh environment, builds each extension with maturin,
installs exact test and oracle dependencies, and runs the complete package
pytest directory.

**Non-obvious choices.** The job uses exact immutable action commits, root
`contents: read` permission, no OIDC authority, and direct failure propagation.
Its Python 3.12.9 runtime supports the exact pytest 9.1.1 pin while exercising
the cp39-abi3 extensions.

**Deviations from the design plan.** Review added structural trigger,
permission, action-input, step-order, and failure-suppression checks. An
inherited prose violation in an F-137 review artifact was fixed separately
before the integrated gate.

**Spec sections touched.** `docs/hld/12-testing-strategy.md`, Python bindings,
and `docs/hld/15-build-and-toolchain.md`, CI job matrix.

**Tests.** Twenty-eight workflow regressions, thirty-three installed rdocx
tests, ten installed rpptx tests, a real failing-test propagation mutation,
WASM and dependency isolation, and the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep the pull-request job least privilege and
make any new package test failure propagate without a conditional or fallback.

### F-139, Rewrite rdocx-wasm

**Sprint.** S35
**Completed.** 2026-08-13
**Size.** L, estimated 4 days, actual 1 day

**What was built.** `WasmDocument` now owns one real `rdocx::Document` and
delegates its established JavaScript surface to the facade. DOCX byte round
trips retain the complete package, ordered text comes through the additive
facade getter, and generated Node tests exercise the actual JavaScript byte
boundary. The native Word and presentation paths retain system-font discovery
through explicit feature forwarding, while the WASM graph disables it and
keeps bundled fonts available.

**Non-obvious choices.** Workspace rendering dependencies are defaults-off so
each concrete consumer selects its graph. Native consumers opt into
`system-fonts`, while `rdocx-wasm` does not. The wrapper keeps the existing
JavaScript names, maps concrete facade errors to string-valued `JsValue`s, and
does not maintain a second package model or add byte aliases.

**Deviations from the design plan.** Microscope remediation added exact
generated-JavaScript reflection, restored presentation system-font defaults,
and strengthened the root workspace manifest sensitivity. The approved facade
ownership, public surface, and HLD impact did not change.

**Spec sections touched.** `docs/hld/03-architecture.md`,
`docs/hld/10-bindings-spec.md`, `docs/hld/12-testing-strategy.md`,
`docs/hld/13-risks-and-open-questions.md`,
`docs/hld/14-development-backlog.md`, and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** `document_with_images_headers_and_numbering_round_trips_every_part_intact`,
`document_text_preserves_body_and_table_order`,
`wasm_round_trip_preserves_the_complete_package_in_node`, native and WASM
feature-contract mutations, no-default gates, package and publication checks,
and the integrated full gate at `fecfd0a` passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep the WASM package as a facade consumer.
Feature isolation depends on defaults-off workspace edges and explicit native
opt-ins, while bundled font bytes remain unconditional.

### F-140, wasm CI job

**Sprint.** S35
**Completed.** 2026-08-13
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The pull-request WASM job now target-checks both facade
wrappers with the locked graph and runs both inline Node suites. It installs
exact Node 24.11.1 and wasm-pack 0.15.0, and a structured workflow contract
enforces package coverage, step order, immutable action pins, least privilege,
and ordinary failure propagation.

**Non-obvious choices.** The operative setup-node pin is the reviewed v6.5.0
commit. Both Node suites run as separate commands without conditions,
`continue-on-error`, fallback success, or listing-only substitutions. The
incubating cargo-release preparation group includes unpublished `rpptx-wasm`,
while the crates.io allowlist remains limited to published packages.

**Deviations from the design plan.** Microscope remediation corrected the
setup-node release label in the testing HLD and added independent provenance
sensitivity. The approved two-package workflow and HLD scope did not change.

**Spec sections touched.** `docs/hld/10-bindings-spec.md`,
`docs/hld/12-testing-strategy.md`, `docs/hld/14-development-backlog.md`, and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** `test_wasm_pr_job_checks_both_targets_and_runs_node_tests`,
`test_wasm_pr_job_rejects_skipped_or_weakened_gates`, setup-node provenance
and release-family mutations, locked wasm32 checks, both complete Node suites,
a real propagated Node failure, and the integrated full gate at `fecfd0a`
passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep action annotations aligned with their
immutable commits. Any new WASM package must receive an executable Node gate,
not only a target check.

### F-141, to_pdf in the browser

**Sprint.** S35
**Completed.** 2026-08-13
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `WasmDocument.toPdf` now returns bytes directly from the
normal `rdocx::Document::to_pdf` facade. A generated-JavaScript Node regression
adds text through the public binding, calls `toPdf` reflectively, and requires
a complete PDF with a Type 0 font, an embedded TrueType stream, and bundled
Carlito.

**Non-obvious choices.** There is no deterministic alias or WASM-only renderer.
The wrapper's defaults-off graph makes the normal facade path browser-safe by
excluding host discovery while retaining unconditional bundled fonts.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** `docs/hld/10-bindings-spec.md`,
`docs/hld/12-testing-strategy.md`, and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** `to_pdf_in_node_returns_a_complete_pdf_with_an_embedded_bundled_font`
proved the generated `toPdf` name, `Uint8Array` boundary, `%PDF-` through
`%%EOF`, `/Subtype /Type0`, `/FontFile2`, and Carlito. The two-test Node suite,
feature-isolation contract, font and rendering riders, mutation sensitivity,
and the integrated full gate at `fecfd0a` passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Browser PDF behavior belongs to the normal
facade method under the WASM feature graph. Do not fork layout or PDF assembly
inside the binding.

### F-142, rpptx-wasm

**Sprint.** S35
**Completed.** 2026-08-13
**Size.** M, estimated 2 days, actual 1 day

**What was built.** The workspace now contains an unpublished `rpptx-wasm`
package backed by one real `rpptx::Presentation`. Its bounded default profile
constructs, opens, serializes, counts, and mutates presentations without the
renderer. The opt-in `render` profile adds only `toPdf`. Package-to-render
assembly moved from the corpus example into the owning facade, and the example
now delegates to that single deterministic path.

**Non-obvious choices.** Native `rpptx` defaults retain template, rendering,
and system fonts, while the wrapper selects a bundled-template-only facade and
adds deterministic rendering explicitly. The exact optimized normal-default
artifact is 519,060 decimal gzip bytes, below the 1,000,000-byte gate. The
size check binds reviewed wasm-pack, wasm-opt, and deterministic gzip arguments
to the freshly built artifact.

**Deviations from the design plan.** Microscope remediation bound the size gate
to the current artifact, hardened normal-default and render feature contracts,
and strengthened facade-to-example parity and rendering completeness. The
approved profiles, facade boundary, and HLD impact did not change.

**Spec sections touched.** `docs/hld/03-architecture.md`,
`docs/hld/08-rendering-spec.md`, `docs/hld/10-bindings-spec.md`,
`docs/hld/12-testing-strategy.md`, and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** `default_profile_is_under_one_megabyte_and_round_trips_a_deck`,
`wasm_presentation_uses_the_real_facade_in_node`,
`render_profile_returns_a_complete_pdf`, facade-to-example render parity,
default and render dependency graphs, exact 519,060-byte size and mutation
sensitivity, publication riders, and the integrated full gate at `fecfd0a`
passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep rendering optional for the wrapper and
package interpretation in the facade. Re-run the exact optimized size gate
whenever the normal-default dependency graph changes.

### F-143, oxml-cli-support

**Sprint.** S36
**Completed.** 2026-08-13
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The publishable, currently unpublished
`oxml-cli-support` crate owns bounded
one-based range parsing, default output-path construction, and the schema-one
JSON envelope shared by both command-line tools. `rdocx-cli` now uses the
shared output-path and JSON helpers without changing its command surface.

**Non-obvious choices.** Range parsing charges requested expansion work before
deduplication and accepts exactly 100,000 requested values. This prevents large
or overlapping ranges from amplifying memory or CPU work while retaining
sorted and deduplicated results.

**Deviations from the design plan.** Review added the explicit materialization
and cumulative-work bounds, the exact accepted boundary, and full compatibility
coverage for every existing rdocx inspect field and default conversion path.

**Spec sections touched.** `docs/hld/03-architecture.md`,
`docs/hld/10-bindings-spec.md`, and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** Seven shared helper tests, two rdocx-cli compatibility tests,
oversized and overlapping range mutations, the 21-package publication dry run,
dependency-direction checks, and the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep format-neutral command plumbing in this
crate and charge range work before materializing user input.

### F-144, rpptx-cli

**Sprint.** S36
**Completed.** 2026-08-13
**Size.** L, estimated 4 days, actual 1 day

**What was built.** The publishable, currently unpublished `rpptx-cli` binary
provides `inspect`,
`text`, `convert`, `diff`, `replace`, `validate`, and `render`. It consumes the
real presentation facade and shared CLI support, preserves package content and
run formatting during replacement, and uses deterministic rendering for PDF
and PNG output.

**Non-obvious choices.** Raster commands reject more than 8,000,000 pixels per
page. PNG conversion preflights every page and then renders, writes, and drops
one encoded page at a time. Text diff retains its established LCS behavior but
rejects matrices above 1,000,000 cells before allocation.

**Deviations from the design plan.** Review added complete core metadata to
plain inspect output, zero-slide PNG failure, bounded DPI and diff resources,
streaming multi-slide PNG output, and verified corpus provision for both clean
CI jobs.

**Spec sections touched.** `docs/hld/03-architecture.md`,
`docs/hld/06-presentationml-model.md`, `docs/hld/10-bindings-spec.md`,
`docs/hld/12-testing-strategy.md`, and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** Fourteen command integrations, all 50 pinned corpus decks, the
same-run, cross-run, grouped, and table replacement matrix, resource-boundary
mutations, deterministic rendering checks, workflow regressions, and the
integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep OOXML ownership in the facade. New CLI
operations must remain bounded before allocating or collecting output.

### F-145, rpptx-cli thumbnail and outline

**Sprint.** S36
**Completed.** 2026-08-13
**Size.** M, estimated 2 days, actual 1 day

**What was built.** `rpptx thumbnail` renders slide one to a deterministic PNG
at exactly 320 pixels wide with proportional height. `rpptx outline` prints
each slide title once and recursively emits textual paragraphs in shape order
with two spaces per paragraph level.

**Non-obvious choices.** Outline title suppression compares the actual
`ShapeRef` node identity through additive `PartialEq` and `Eq` implementations.
This remains total for field-only titles and does not depend on collapsed
placeholder indexes. Paragraph line breaks normalize to printable spaces.

**Deviations from the design plan.** Review exposed unindexed placeholder and
field-only title cases. The user approved the bounded equality trait addition,
and the owning facade HLD was added to the exact work list before completion.

**Spec sections touched.** `docs/hld/06-presentationml-model.md`,
`docs/hld/08-rendering-spec.md`, `docs/hld/10-bindings-spec.md`,
`docs/hld/12-testing-strategy.md`, `docs/hld/14-development-backlog.md`, and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** Fourteen CLI integrations cover portrait aspect ratio, output-path
precedence, grouped text, paragraph levels, embedded breaks, unindexed
placeholders, field-only titles, all 50 corpus decks, targeted mutations, and
the integrated full gate.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Compare facade handles by their defined node
identity when exact shape suppression is required. Do not infer identity from
placeholder indexes or text.

### F-146, npm publication

**Sprint.** S36
**Completed.** 2026-08-13
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The CI WASM job can build local bundler packages, run
`npm pack`, and install `@tensorbee/rdocx-wasm` and
`@tensorbee/rpptx-wasm` into separate fresh consumers. The packages contain
their scoped metadata, WebAssembly binary, JavaScript glue, and public type
declarations. No registry publication path or authority was added.

**Non-obvious choices.** Both manifests use wasm-opt 125 with `-Oz`,
`--enable-bulk-memory`, and `--enable-nontrapping-float-to-int`. CI downloads
the reviewed official Binaryen asset and verifies its SHA-256 before use.
Fresh installs disable scripts, audits, and funding calls.

**Deviations from the design plan.** Actual Rust output required the approved
third wasm-opt feature flag. The user also approved installing exact wasm-opt
125 because wasm-pack otherwise falls back to a different bundled version.

**Spec sections touched.** `docs/hld/10-bindings-spec.md`,
`docs/hld/12-testing-strategy.md`, `docs/hld/14-development-backlog.md`, and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** Local bundler builds, two scoped tarballs, separate fresh installs,
package inventories, installed imports, both locked WASM checks, both Node
suites, 36 workflow regressions, dependency isolation, and the integrated full
gate passed. No package was published.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Treat real npm publication as a separate
reviewed story with explicit authority. Keep this path local and install-only.

### F-X001, rdocx-cli tests

**Sprint.** S36
**Completed.** 2026-08-13
**Size.** M, estimated 2 days, actual 1 day

**What was built.** One integration binary invokes all seven `rdocx-cli`
commands through `std::process::Command` with isolated in-code fixtures. The
text command now uses facade document-order text, and both render branches use
the bundled-font deterministic facade.

**Non-obvious choices.** The tests bind visible font output byte-for-byte to
the deterministic renderer for both selected-page and all-page paths. Their
temporary workspaces combine process identity with a local counter.

**Deviations from the design plan.** Review exposed interleaved body-order and
system-font rendering defects in the product. The approved plan was revised to
include those bounded command fixes and exactly three HLD files.

**Spec sections touched.** `docs/hld/10-bindings-spec.md`,
`docs/hld/12-testing-strategy.md`, and
`docs/hld/14-development-backlog.md`.

**Tests.** Seven command integrations, misspelled-command and false-validation
mutations, interleaved paragraph and table text, deterministic selected and
all-page rendering mutations, golden PNG checks, and the integrated full gate
passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep one command integration entrypoint and
bind rendering tests to the deterministic facade rather than host coincidence.

### F-X002, README example correctness

**Sprint.** S36
**Completed.** 2026-08-13
**Size.** S, estimated 1 day, actual 1 day

**What was built.** All six root README Rust examples compile as `no_run`
rustdoc tests. The table example uses the real indexed row and cell APIs. One
canonical Python runner obtains the exact locked rdocx rlib through Cargo JSON
and invokes rustdoc with warnings denied.

**Non-obvious choices.** README remains the only snippet source. The CI docs
job and canonical full verification call the same runner, which prevents drift
without duplicating examples into crate documentation.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** `docs/hld/12-testing-strategy.md`,
`docs/hld/14-development-backlog.md`, and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** The runner compiled six examples, a disposable `rows()` and
`cells()` mutation failed with E0599, output scans remained clean, CI and
generated-adapter contracts passed, and the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Keep README as the snippet source and discover
the actual locked rlib rather than assuming a target filename.

### F-X003, Deduplicate the sample generators

**Sprint.** S36
**Completed.** 2026-08-13
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The obsolete `generate_samples` example was deleted.
`generate_all_samples` is now the single source for every document and output
consumed by the hash and golden-image harnesses.

**Non-obvious choices.** This is a behavior-neutral deletion. The surviving
generator was not rewritten, and no baseline was recorded or moved.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** None.

**Tests.** All 28 deterministic hashes, all seven golden PNG buffers, every
example compile, the canonical seven-sample inventory, a missing-contract
mutation, repository invocation search, and the integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Add sample artifacts only through
`generate_all_samples` and prove the full deterministic inventory.

### F-X004, Fix the shared temp path in the test suite

**Sprint.** S36
**Completed.** 2026-08-13
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The rdocx file round-trip integration uses an output name
containing the test process ID, so concurrent test processes do not share one
fixed path.

**Non-obvious choices.** The regression asserts the exact process identity in
the filename. No production helper or new dependency was introduced for a
single test-isolation correction.

**Deviations from the design plan.** None. Microscope pass 1 was clean.

**Spec sections touched.** None.

**Tests.** The exact test failed under the former fixed-name mutation, two
concurrent invocations passed, the complete rdocx suite passed, and the
integrated full gate passed.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Test files that can be created concurrently
must include process identity or use an isolated workspace.

### F-X006, Tag the expanded rpptx family

**Sprint.** S37
**Completed.** 2026-08-14
**Size.** S, estimated 1 day, actual 1 day

**What was built.** The complete 14-package shared and PowerPoint family is
published on crates.io at 0.1.3 through `rpptx-v0.1.3`. The release adds
`oxml-cli-support` and `rpptx-cli` to the earlier 12-package family while
keeping unpublished `rpptx-wasm` in the 0.1.3 local preparation group.

**Non-obvious choices.** The release used the smallest fresh patch version
above immutable 0.1.2. The annotated tag peels to reviewed sprint SHA
`805680ab8a6dadd4d4247471a81cbb21b88a3196`. The workflow published only the
14-package incubating allowlist and created the matching GitHub release. No
npm package was published. The user gave separate final approval at that
reviewed SHA immediately before the sprint branch push, annotated tag
creation, tag push, and publication workflow.

**Deviations from the design plan.** Full verification exposed stale release
prose that required font assets in both `rdocx-layout` and `oxml-layout`.
The reviewed correction names `oxml-layout` as the sole owner of 20 TTF files
and four legal files, forbids duplication in `rdocx-layout`, and retains the
`rpptx` default presentation asset check.

**Spec sections touched.** `docs/hld/03-architecture.md`,
`docs/hld/14-development-backlog.md`, and
`docs/hld/15-build-and-toolchain.md`.

**Tests.** Full verification passed at the reviewed release SHA. GitHub Actions
run `31762653847` completed successfully for tag `rpptx-v0.1.3`, including the
incubating publication and GitHub release jobs. All 14 packages resolve from
crates.io at 0.1.3, and every owner check reports `mantissaman (Atul Sharma)`.
The remote annotated tag peels to the reviewed SHA.

**Hash harness.** Unchanged. All 28 integrated entries match.

**Notes for future sessions.** Preserve the immutable 0.1.2 and 0.1.3 tags.
Future Rust-family publication must use a fresh version and a separately
approved `/release` invocation. npm publication remains unauthorized.

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

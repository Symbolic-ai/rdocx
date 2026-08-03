# Sprint Tracker

Velocity log. One row per completed F-ID, appended by `/complete-feature`, plus
a per-sprint summary appended by `/close-sprint`.

Estimates come from `docs/hld/14-development-backlog.md`. Actuals are recorded
so the velocity assumption can be corrected against reality rather than
defended.

`S = 1d`, `M = 2-3d`, `L = 4-5d`, `XL = split me`.

## Per-sprint summary

| Sprint | Milestone | Planned | Done | Carried | Est. days | Actual days | Notes |
|--------|-----------|---------|------|---------|-----------|-------------|-------|
| S01 | M1 | 6 | 6 | 0 | 10 | 6 | Completed with no carries |
| S02 | M1 | 6 | 6 | 0 | 8 | 6 | Completed M1 and published rdocx 0.4.1 |
| S03 | M2 | 5 | 3 | 2 | 8 | 3 | F-015 and F-016 carried to S04 to keep rdocx 0.5.0 independent of unpublished oxml-core |
| S04 | M2 | 7 | 4 | 3 | 9 | 4 | F-015, F-016, and F-022 carried to S32.2 so development crates remain unpublished until PowerPoint is complete |
| S05 | M3 | 4 | 4 | 0 | 8 | 4 | Completed isolated unpublished oxml-media staging, with F-027 and F-028 remaining planned for S32.2 |
| S06 | M4 | 3 | 3 | 0 | 8 | 3 | Completed unpublished oxml-layout staging, with M4 continuing in S07 |
| S07 | M4 | 5 | 5 | 0 | 8 | 5 | Completed M4 in unpublished oxml-layout with all 28 hashes unchanged |
| S08 | M5 | 3 | 3 | 0 | 7 | 3 | Staged unpublished oxml-pdf, installed the exact golden gate, and completed the global CTM rewrite |
| S09 | M5 | 4 | 4 | 0 | 7 | 4 | Completed nested groups, solid paths, transform-aware collection, and reusable alpha in unpublished oxml-pdf |
| S10 | M5 | 2 | 2 | 0 | 8 | 2 | Completed M5 with PDF gradients and recursive raster groups, paths, clips, gradients, dashes, and backgrounds |
| S11 | M6 | 0 | 0 | 0 | 0 | 1 | Confirmed the staged extraction boundary with no publication, consumer cutover, or implementation F-IDs |
| S12 | M7 | 5 | 5 | 0 | 11 | 5 | Completed the first M7 DrawingML slice with exact PowerPoint colour evidence and no publication |
| S13 | M7 | 4 | 4 | 0 | 12 | 4 | Completed transforms, custom geometry, and fills in unpublished oxml-drawing with all 28 hashes unchanged |
| S14 | M7 | 8 | 8 | 0 | 14 | 8 | Completed lines, effects, shape properties, style references, and the split text model with all 28 hashes unchanged and no publication |
| S15 | M7 | 2 | 2 | 0 | 5 | 2 | Completed themes and the stable Word adapter with pinned PowerPoint acceptance, all 28 hashes unchanged, and no publication. The external corpus boundary runs with F-067 at S16 entry |
| S16 | M8 | 4 | 4 | 0 | 12 | 4 | Established the pinned 50-deck corpus and modelled core PresentationML parts and recursive shape trees with all 28 hashes unchanged and no publication |
| S17 | M8 | 4 | 4 | 0 | 10 | 4 | Completed placeholders, pictures, graphic-frame dispatch, and DrawingML tables against all 50 pinned decks with all 28 hashes unchanged and no publication |
| S18 | M8 | 4 | 4 | 0 | 7 | 4 | Completed connectors, alternate-content fallback selection, notes parts, and relationship-id rewriting against all 50 pinned decks with all 28 hashes unchanged and no publication |
| S19 | M8 | 2 | 2 | 0 | 6 | 2 | Completed the rpptx read facade and modelled 50-deck gate with native PowerPoint acceptance, all 28 hashes unchanged, and no publication |
| S20 | M9 | 5 | 5 | 0 | 11 | 5 | Completed placeholder, transform, body, text-style, format-scheme, and typeface inheritance with all 28 hashes unchanged and no publication |
| S21 | M9 | 3 | 3 | 0 | 8 | 3 | Completed M9 with the frozen ResolvedSlide contract, strict all-slide corpus resolution, native PowerPoint acceptance, all 28 hashes unchanged, and no publication |
| S22 | M10 | 4 | 4 | 0 | 9 | 4 | Completed preset provenance, generation, evaluation, fallback, and the unpublished renderer input boundary with all 28 hashes unchanged and no publication |

## Completed features

| F-ID | Sprint | Size | Est. days | Actual days | Completed | Notes |
|------|--------|------|-----------|-------------|-----------|-------|
| F-001 | S01 | M | 2 | 1 | 2026-07-29 | Deterministic bundled-font path |
| F-002 | S01 | S | 1 | 1 | 2026-07-29 | Rust 1.97.1 toolchain pin |
| F-003 | S01 | L | 4 | 1 | 2026-07-29 | Initial 28-entry hash baseline |
| F-004 | S01 | S | 1 | 1 | 2026-07-29 | Caladea licence and notice |
| F-005 | S01 | S | 1 | 1 | 2026-07-29 | Collision-safe image suffix allocation |
| F-006 | S01 | S | 1 | 1 | 2026-07-29 | Safe JPEG standalone-marker walk |
| F-007 | S02 | S | 1 | 1 | 2026-07-30 | Relationship-based core properties |
| F-008 | S02 | M | 2 | 1 | 2026-07-30 | 61 non-consuming setter twins |
| F-009 | S02 | M | 2 | 1 | 2026-07-30 | Thread-safe two-mode layout cache |
| F-010 | S02 | S | 1 | 1 | 2026-07-30 | Fourteen crates.io names reserved |
| F-011 | S02 | S | 1 | 1 | 2026-07-30 | Unit truncation behavior pinned |
| F-012 | S02 | S | 1 | 1 | 2026-07-30 | Published and tagged rdocx 0.4.1 |
| F-013 | S03 | M | 2 | 1 | 2026-07-30 | Unpublished shared OOXML core |
| F-014 | S03 | M | 2 | 1 | 2026-07-30 | Shared schema unit types |
| F-017 | S03 | M | 2 | 1 | 2026-07-30 | Shared app and custom properties |
| F-018 | S04 | M | 2 | 1 | 2026-07-30 | Unpublished format-neutral OPC package |
| F-019 | S04 | S | 1 | 1 | 2026-07-30 | PresentationML package constants |
| F-020 | S04 | M | 2 | 1 | 2026-07-30 | Code-built PowerPoint OPC proof |
| F-021 | S04 | S | 1 | 1 | 2026-07-30 | Canonical ZIP entry normalization |
| F-023 | S05 | M | 2 | 1 | 2026-07-30 | Dependency-free image format sniffing |
| F-024 | S05 | L | 4 | 1 | 2026-07-30 | Safe image metadata and DPI probing |
| F-025 | S05 | S | 1 | 1 | 2026-07-30 | Collision-free shared media naming |
| F-026 | S05 | S | 1 | 1 | 2026-07-30 | Dependency-free native EMU sizing |
| F-029 | S06 | M | 2 | 1 | 2026-07-31 | Unpublished layout output and font staging |
| F-030 | S06 | L | 4 | 1 | 2026-07-31 | Owned format-neutral line-breaking boundary |
| F-031 | S06 | M | 2 | 1 | 2026-07-31 | Six-coefficient affine transforms |
| F-032 | S07 | M | 2 | 1 | 2026-07-31 | Backend-neutral path geometry |
| F-033 | S07 | M | 2 | 1 | 2026-07-31 | Gradient, tile, and stroke paint model |
| F-034 | S07 | M | 2 | 1 | 2026-07-31 | Nested group and path output arms |
| F-035 | S07 | S | 1 | 1 | 2026-07-31 | Transform-aware nested leaf traversal |
| F-036 | S07 | S | 1 | 1 | 2026-07-31 | Content-addressed staged image keys |
| F-037 | S08 | S | 1 | 1 | 2026-07-31 | Unpublished shared PDF backend staging |
| F-038 | S08 | M | 2 | 1 | 2026-07-31 | Exact deterministic golden-PNG gate |
| F-039 | S08 | L | 4 | 1 | 2026-07-31 | Global page CTM with reviewed pixel delta |
| F-040 | S09 | M | 2 | 1 | 2026-07-31 | Recursive PDF group graphics states |
| F-041 | S09 | M | 2 | 1 | 2026-07-31 | Solid PDF path geometry and paint |
| F-042 | S09 | M | 2 | 1 | Nested font, image, and link collection |
| F-044 | S09 | S | 1 | 1 | Reused PDF ExtGState alpha resources |
| F-043 | S10 | L | 4 | 1 | 2026-07-31 | Deterministic PDF gradient resource graphs |
| F-045 | S10 | L | 4 | 1 | 2026-07-31 | Recursive raster groups, paths, gradients, and dashes |
| F-052 | S12 | S | 1 | 1 | 2026-07-31 | Unpublished DrawingML crate and namespace constants |
| F-053 | S12 | M | 2 | 1 | 2026-07-31 | Schema-boundary raw child ordering |
| F-054 | S12 | M | 2 | 1 | 2026-07-31 | Four DrawingML colour choices with raw preservation |
| F-055 | S12 | L | 4 | 1 | 2026-07-31 | Exact PowerPoint colour transform stack |
| F-056 | S12 | M | 2 | 1 | 2026-07-31 | Master colour-map resolution before theme lookup |
| F-057 | S13 | M | 2 | 1 | 2026-07-31 | DrawingML transforms and exact nested composition |
| F-058 | S13 | L | 4 | 1 | 2026-07-31 | Guide formulas, path evaluation, and arc lowering |
| F-059 | S13 | M | 2 | 1 | 2026-07-31 | Custom geometry XML model and evaluation |
| F-060 | S13 | L | 4 | 1 | 2026-07-31 | DrawingML fill families with raw preservation |
| F-061 | S14 | M | 2 | 1 | 2026-08-01 | DrawingML line properties and preset dash mapping |
| F-062 | S14 | S | 1 | 1 | 2026-08-01 | Outer shadows with unsupported effect preservation |
| F-063 | S14 | M | 2 | 1 | 2026-08-01 | Shape properties and four style-reference forms |
| F-064a | S14 | M | 2 | 1 | 2026-08-01 | Text body properties and typed shell |
| F-064b | S14 | L | 4 | 1 | 2026-08-01 | Paragraphs, runs, fields, breaks, and whitespace |
| F-064c | S14 | S | 1 | 1 | 2026-08-01 | Character, automatic, and no-bullet forms |
| F-064d | S14 | M | 2 | 1 | 2026-08-01 | Fixed nine-level list styles |
| F-064 | S14 | XL | 0 | 1 | 2026-08-01 | Umbrella closed after four child stories and integrated gates |
| F-065 | S15 | L | 4 | 1 | 2026-08-01 | Complete DrawingML theme and pinned PowerPoint default |
| F-066 | S15 | S | 1 | 1 | 2026-08-01 | Stable Word theme projection through the documented edge |
| F-067 | S16 | M | 2 | 1 | 2026-08-01 | Unpublished PresentationML crate and pinned 50-deck corpus harness |
| F-068 | S16 | M | 2 | 1 | 2026-08-01 | Presentation root, sizes, identifiers, and default text style |
| F-069 | S16 | L | 4 | 1 | 2026-08-01 | Slide, layout, master, colour-map, and text-style models |
| F-070 | S16 | L | 4 | 1 | 2026-08-01 | Recursive ordered shape tree with opaque child payloads |
| F-071 | S17 | M | 2 | 1 | 2026-08-01 | Presence-sensitive placeholder keys and typed partial shapes |
| F-072 | S17 | M | 2 | 1 | 2026-08-01 | Typed pictures with crops, relationships, and placeholders |
| F-073 | S17 | M | 2 | 1 | 2026-08-01 | Graphic-frame URI dispatch with typed tables and opaque payloads |
| F-074 | S17 | L | 4 | 1 | 2026-08-01 | DrawingML tables with merges, banding, and preserved content |
| F-075 | S18 | S | 1 | 1 | 2026-08-01 | Typed connectors with optional start and end connections |
| F-076 | S18 | M | 2 | 1 | 2026-08-01 | Raw-preserving alternate content with typed fallback selection |
| F-077 | S18 | M | 2 | 1 | Notes parts and body-placeholder speaker-note extraction |
| F-078 | S18 | M | 2 | 1 | Namespace-aware relationship-id rewriting in preserved XML |
| F-079 | S19 | L | 4 | 1 | 2026-08-02 | Unpublished relationship-resolved rpptx read facade |
| F-080 | S19 | M | 2 | 1 | 2026-08-02 | Seven-root 50-deck modelled round-trip and native PowerPoint gate |
| F-081 | S20 | M | 2 | 1 | 2026-08-02 | Unpublished resolver context and recursive placeholder chain |
| F-082 | S20 | M | 2 | 1 | 2026-08-02 | Typed ordinary-shape properties plus transform and body inheritance |
| F-083 | S20 | L | 4 | 1 | 2026-08-02 | Seven-source, nine-level text-property cascade with safe caching |
| F-084 | S20 | M | 2 | 1 | 2026-08-02 | Typed shape styles and format-scheme resolution with placeholder colours |
| F-085 | S20 | S | 1 | 1 | 2026-08-02 | Major and minor theme-token typeface resolution |
| F-086 | S21 | L | 4 | 1 | 2026-08-02 | Final draw-order flattener with inherited-shape suppression |
| F-087 | S21 | M | 2 | 1 | 2026-08-02 | Frozen owned ResolvedSlide contract with concrete renderer values |
| F-088 | S21 | M | 2 | 1 | 2026-08-02 | Pinned visual differential and native PowerPoint acceptance gates |
| F-089 | S22 | S | 1 | 1 | 2026-08-02 | Licensed ECMA preset geometry provenance decision |
| F-090 | S22 | L | 4 | 1 | 2026-08-02 | Reproducible complete preset geometry table generator |
| F-091 | S22 | M | 2 | 1 | 2026-08-02 | Known preset evaluation and diagnosed bounds fallback |
| F-092 | S22 | M | 2 | 1 | 2026-08-02 | Unpublished scoped relationship and RenderInput boundary |
| F-093 | S23 | L | 4 | 1 | 2026-08-03 | Shape paths with solid, gradient, outline, and visible fallback paint |
| F-094 | S23 | M | 2 | 1 | 2026-08-03 | Exact rotation, centre flip, translation, and parent transform composition |
| F-095 | S23 | S | 1 | 1 | 2026-08-03 | Source-neutral resolved line ends lowered to filled paths |
| F-096 | S23 | M | 2 | 1 | 2026-08-03 | Source-scoped cropped, stretched, and tiled picture rendering |
| F-097 | S23 | S | 1 | 1 | 2026-08-03 | Preserving explicit background projection and concrete paint resolution |

## Velocity

Recalculated at each sprint close. The backlog assumes about 2 stories per week
sustained, and the whole plan is sized at roughly 390 developer-days. If the
first three sprints diverge from that by more than 30 percent, replan rather
than absorb it.

Stories per week is completed stories divided by actual days, multiplied by
five working days.

| Window | Stories | Days | Stories/week |
|--------|---------|------|--------------|
| S01 | 6 | 6 | 5.00 |
| S02 | 6 | 6 | 5.00 |
| S03 | 3 | 3 | 5.00 |
| S04 | 4 | 4 | 5.00 |
| S05 | 4 | 4 | 5.00 |
| S06 | 3 | 3 | 5.00 |
| S07 | 5 | 5 | 5.00 |
| S08 | 3 | 3 | 5.00 |
| S09 | 4 | 4 | 5.00 |
| S10 | 2 | 2 | 5.00 |
| S11 | 0 | 1 | 0.00 |
| S12 | 5 | 5 | 5.00 |
| S13 | 4 | 4 | 5.00 |
| S14 | 8 | 8 | 5.00 |
| S15 | 2 | 2 | 5.00 |
| S16 | 4 | 4 | 5.00 |
| S17 | 4 | 4 | 5.00 |
| S18 | 4 | 4 | 5.00 |
| S19 | 2 | 2 | 5.00 |
| S20 | 5 | 5 | 5.00 |
| S21 | 3 | 3 | 5.00 |
| S22 | 4 | 4 | 5.00 |

## Escalation record

Logged when an escalation trigger from `.claude/WORKFLOW.md` fires, with what
was done about it. Empty is the expected state.

| Date | Trigger | F-ID or sprint | Response |
|------|---------|----------------|----------|
| 2026-07-30 | Three-sprint velocity variance exceeded 30 percent | S01 to S03 | Reforecast 366 remaining estimated days to 45 to 50 active weeks, retain dependency-defined boundaries, and recalibrate after S06 |
| 2026-07-31 | Sprint estimate variance exceeded 30 percent | S05 | Record 4 actual days against 8 estimated, retain the 45 to 50 active week reforecast, and recalibrate after S06 |
| 2026-07-31 | Sprint estimate variance exceeded 30 percent | S06 | Reforecast 124 remaining stories at the observed five stories per active week to about 25 active weeks, while retaining dependency-defined sprint boundaries |
| 2026-07-31 | Sprint estimate variance exceeded 30 percent | S07 | Record 5 actual days against 8 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |
| 2026-07-31 | Sprint estimate variance exceeded 30 percent | S08 | Record 3 actual days against 7 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |
| 2026-07-31 | Sprint estimate variance exceeded 30 percent | S09 | Record 4 actual days against 7 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |
| 2026-07-31 | Sprint estimate variance exceeded 30 percent | S10 | Record 2 actual days against 8 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |
| 2026-07-31 | Sprint estimate variance exceeded 30 percent | S12 | Record 5 actual days against 11 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |
| 2026-08-01 | Sprint estimate variance exceeded 30 percent | S13 | Record 4 actual days against 12 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |
| 2026-08-01 | Sprint estimate variance exceeded 30 percent | S14 | Record 8 actual days against 14 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |
| 2026-08-01 | Sprint estimate variance exceeded 30 percent | S15 | Record 2 actual days against 5 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |
| 2026-08-01 | Sprint estimate variance exceeded 30 percent | S16 | Record 4 actual days against 12 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |
| 2026-08-01 | Sprint estimate variance exceeded 30 percent | S17 | Record 4 actual days against 10 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |
| 2026-08-02 | Sprint estimate variance exceeded 30 percent | S18 | Record 4 actual days against 7 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |
| 2026-08-02 | Sprint estimate variance exceeded 30 percent | S19 | Record 2 actual days against 6 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |
| 2026-08-02 | Sprint estimate variance exceeded 30 percent | S20 | Record 5 actual days against 11 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |
| 2026-08-02 | Sprint estimate variance exceeded 30 percent | S21 | Record 3 actual days against 8 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |
| 2026-08-03 | Sprint estimate variance exceeded 30 percent | S22 | Record 4 actual days against 9 estimated and retain the about 25 active week reforecast with dependency-defined sprint boundaries |

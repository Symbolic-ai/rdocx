# 11, Migration plan

How the `oxml-*` crates are extracted without breaking a shipped library.

Covers milestones M1 through M6. Shared implementations live in their neutral
crates, while the released Word family retains only format-specific models and
small compatibility shims. Each extraction step kept `cargo test --workspace`
green and independently revertible.

## The safety net comes first

The migration touches **unit conversion** and **text-shaping input types**. Both
change output silently rather than failing to compile. The 64 in-memory
round-trip tests prove structure survives, not that bytes are identical, so they
cannot catch this class of defect.

**M1 therefore builds an output-stability harness before anything moves.**
`crates/rdocx/examples/generate_all_samples.rs` already exercises nearly every
API. For each sample, record a digest of the flushed `document.xml`,
`styles.xml` and `numbering.xml`, plus the page-one PNG at 150 dpi. Re-run after
every step, and treat any delta as a defect until it is explained.

**Deterministic font mode is a prerequisite for that harness**, not an
optimisation. System fonts differ by platform, so a digest recorded on one
machine would not match one recorded on another. The harness and the SSIM gate
use `oxml_layout::FontManager::new_deterministic()` and render from bundled
fonts only, with system loading bypassed. The shared layout
`--no-default-features` path proves the same isolation used by WASM.

## The facade trick

The namespace helpers have call sites throughout `rdocx-oxml`. Migrating them
individually would be a large, risky, reviewer-hostile diff.

`rdocx-oxml` is therefore a facade over the published `oxml-core` 0.1.2 crate,
and **not one call site changes**:

```rust
// crates/rdocx-oxml/src/lib.rs
pub use oxml_core::{core_properties, error, raw_xml, units};
pub use error::{OxmlError, Result};
pub(crate) use oxml_core::xml_text;

// crates/rdocx-oxml/src/namespace.rs keeps W_NS and W_PREFIX, and adds:
pub use oxml_core::xml::{matches_local_name, R_NS, MC_NS};
```

The shared implementation is consumed only after its exact 0.1.2 release is
resolvable from crates.io. The acceptance check is mechanical: the crate-local
diff shows only `lib.rs`, `namespace.rs` and `Cargo.toml` modified, plus five
deletions. `Cargo.lock` records the one-way dependency edge. The `rdocx` facade
uses the same pattern for `Length`: it directly depends on `oxml-core`,
re-exports `oxml_core::Length`, and keeps every existing caller unchanged.

This is what makes the bulk of the extraction low-risk, and it is worth stating
plainly: most of this migration is a re-export block.

## Order of operations

| Step | Crate | Note |
|---|---|---|
| 1 | `oxml-core` staging | Copy the shared implementation, add its new types, and leave released rdocx consumers unchanged |
| 2 | `oxml-opc` staging | Copy the package implementation, generalise constructors, add PowerPoint constants, and prove the isolated crate |
| 3 | `oxml-media` staging | Build the shared media implementation without changing released consumers |
| 4 | `oxml-layout` staging | Copy the output types, font manager, and bundled fonts behind an isolated crate boundary |
| 5 | **`line.rs` decoupling** | Its own PR and review inside the staged layout implementation |
| 6 | `PositionedElement` extension | Add `Transform`, `Path`, `Paint`, `Group`, `walk`, and `#[non_exhaustive]` |
| 7 | `oxml-pdf` staging | Copy the backend, rewrite the three collection passes on `walk`, and add the new arms |
| 8 | PowerPoint implementation | Finish and review the shared-crate publication plan before any released rdocx package consumes real development code |
| 9 | `rdocx-oxml` and `Length` cutover | Apply the re-export block above and delete the staged duplicates |
| 10 | `rdocx-opc` cutover | Install the deprecated shim, flip direct consumers, and change `rdocx::Error::Opc` to the shared type |
| 11 | Media and layout cutover | Released rdocx media handling uses published `oxml-media`. The layout type cutover follows separately |
| 12 | `rdocx-pdf` cutover | Install `pub use oxml_pdf::*` after the shared backend is publishable |

Staging steps keep every released rdocx package on its published dependency
graph. Cutover steps begin only after PowerPoint development is complete and
the real shared crates have an approved publication path. This preserves the
full package dry-run gate without publishing development crates early, and
each step remains independently revertable.

## The Word conversion boundary

`oxml-layout` owns line breaking and its parameters. The concrete functions in
`crates/rdocx-layout/src/convert.rs` translate the retained Word flow values at
the engine boundary.

| Word input | Shared layout value |
|---|---|
| `CT_TabStop` | `TabStop { pos_pt: f64, align: TabAlign, leader: Option<TabLeader> }` |
| `ST_Jc` | `Align { Start, Center, End, Justify, Distribute }` |
| `ST_TabJc` | `TabAlign { Left, Center, Right, Decimal, Bar }` |
| `ST_Underline` | `Underline { Single, Double, Thick, Dotted, Dash, Wave, ... }` |
| `line_spacing: Option<Twips>` plus `line_rule: Option<String>` | `LineSpacing { Single, Multiple(f64), Exact(f64), AtLeast(f64) }` |

Tab positions become points rather than twips, because the layout engine already
works in points everywhere else. Replacing the stringly-typed `line_rule` with a
proper enum is a strict improvement.

The converter uses concrete functions rather than a trait. It also preserves
the pre-cutover glyph slices at Unicode wrap opportunities and restores Word's
automatic line-height formula after shared line breaking. Those compatibility
steps keep the 28-entry hash harness byte-identical while slide text retains
its shared point-size spacing semantics.

## Preserve behaviour, do not improve it

Three things are deliberately left wrong during the move, because correcting
them mid-extraction would produce hash deltas indistinguishable from migration
bugs:

- **Unit truncation.** Float constructors truncate toward zero with `as i64` or
  `as i32`. Positive and negative fractional tests pin every `Length`, `Twips`
  and `Emu` constructor. A rounding change shifts every twip, which shifts
  layout, which moves the regression tests' output.
- **`apply_tint_shade`.** Keep Word's 0-255 convention and its naive sRGB
  interpolation, byte for byte. `oxml-drawing` adds spec-correct functions
  alongside under different names.
- **Everything else that looks improvable.** File it as a story, do not fold it
  into a move.

The exception is behaviour that is a **defect**, which is fixed in M1 as its own
commit with a reviewed hash delta: the image counter, the JPEG marker walk, and
core-property resolution.

One intentional package-structure difference is isolated in M3: rdocx content
types and media part extensions are sniffed from magic bytes, so a mislabelled
`.png` that is really a JPEG gets a `.jpeg` part and `image/jpeg`. A focused
package regression pins the part name, content type, and relationship target.
The 28-entry hash harness does not include those fields, so it remains
unchanged.

## What happens to the published crates

All seven released rdocx crates remain published at the immutable 0.4.1
boundary. The workspace prepares the same seven-package crates.io family at
the breaking pre-1.0 0.5.0 boundary, but no 0.5.0 tag, GitHub release, or
registry version exists until `/release v0.5.0` receives its separate final
approval at the reviewed SHA. The eleven-package shared-version preparation
group also carries unpublished `rdocx-wasm`, `rdocx-py`, `rpptx-py`, and
`oxml-py-support` at 0.5.0 without adding them to crates.io publication. The
complete 14-package `oxml-*` and `rpptx*` crates.io family remains published at
the immutable 0.1.3 boundary. Released rdocx consumers depend on those
registry-backed shared crates.

| Crate | Fate |
|---|---|
| `rdocx-opc` | Deprecation shim in the approved cutover release, then stop publishing. Prior versions stay on crates.io forever |
| `rdocx-pdf` | Same, over published `oxml-pdf` |
| `rdocx-oxml` | **Stays a real crate permanently.** It keeps ~8,700 lines of WordprocessingML |
| `rdocx-layout` | Stays. Keeps the flow model |
| `rdocx`, `rdocx-cli`, `rdocx-html` | Names unaffected |

**Do not yank anything.** Yanking is for broken or insecure releases. It breaks
fresh resolution for existing users and does not remove the crate.

The `rdocx-opc` shim re-exports `oxml_opc` exactly and carries the package
description `deprecated: moved to oxml-opc`. That string is what appears on
crates.io search results and docs.rs, and it is the only whole-crate
deprecation signal Cargo surfaces. Retained paths such as
`rdocx_opc::OpcPackage` are type-identical to the shared type. The removed
Word-specific `OpcPackage::new_docx` and `ContentTypes::new_docx` constructors
are an intentional breaking change.

A shim is cheap insurance specifically for `rdocx-oxml`, because rdocx's public
API currently **leaks** its types (`CT_PPr`, `CT_SectPr`, `VMerge`, `Twips`)
without re-exporting them, so a downstream user may depend on it directly.

## Repository and link impact

The repository keeps the name `tensorbee/rdocx`, so **no existing link is
affected at all**. crates.io indexes by crate name, docs.rs builds from the
uploaded tarball, and no redirect is involved.

The rdocx cutover is a breaking release regardless of its assigned version.
`Error::Opc` wraps `oxml_opc::OpcError`, `Error::Layout` wraps
`oxml_layout::LayoutError`, and the removed public `rdocx_layout::line` module
is replaced by shared root types. `PositionedElement` is also
`#[non_exhaustive]`.

## Release tooling

The unsafe `scripts/release.sh` is gone. Version changes are prepared as
reviewable F-ID commits with targeted manifest and lockfile edits. `/release`
then tags the exact fully verified commit after a separate final approval. It
accepts exactly one stable `vX.Y.Z` or incubating `rpptx-vX.Y.Z` release and is
the sole authority for both namespaces and crates.io publication.
`/close-sprint` owns `sNN` tags and `/spec-bump` owns local `spec-v*` tags.

`publish.yml` routes the namespaces to disjoint dependency-ordered allowlists.
A stable tag publishes exactly `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`,
`rdocx-html`, `rdocx-pdf`, `rdocx`, and `rdocx-cli`. An incubating tag publishes
exactly `oxml-core`, `oxml-opc`, `oxml-media`, `oxml-layout`, `oxml-drawing`,
`oxml-pdf`, `oxml-sml`, `oxml-cli-support`, `rpptx-oxml`, `rpptx-chart`,
`rpptx-layout`, `rpptx-render`, `rpptx`, and `rpptx-cli`. The stable
shared-version group is prepared locally at 0.5.0 while the seven stable
registry releases remain at 0.4.1. Preparation does not authorize the pending
tag, GitHub release, or registry publication.

Before either real allowlist, the workflow reproduces the deterministic hash
baseline and verifies the full publishable workspace with a dry run. Each real
publish command keeps archive verification enabled, propagates authentication,
network, compilation and duplicate-version failures, and waits for the registry
between dependency layers. `/release` validates the requested family's exact
package and version set at the clean reviewed SHA, requires full verification
and a clean sprint review, obtains a separate final approval, pushes only the
requested tag, and verifies crates.io ownership plus the matching GitHub
release.

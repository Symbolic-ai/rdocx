---
description: Trace what a proposed change would touch, before committing to it.
---

# /impact <target>

Answer "what breaks if I change this?" before the change is made. The target is
a type, a function, a crate, a part name or an F-ID.

Read-only. Produces an assessment, not a diff.

## Steps

1. **Resolve the target.** A symbol, a file, a crate, or an F-ID whose design
   plan names the surface it touches.

2. **Find the direct users.** Search the workspace for every reference. Report
   the count per crate, because a change confined to one crate is a different
   proposition from one crossing three.

3. **Classify the surface.**

   | Class | Consequence |
   |---|---|
   | Crate-private | Free to change |
   | Workspace-internal | Mechanical, one commit |
   | Published API of a released crate | **Semver decision.** Name the crate and its version |
   | Leaked type | Worse than it looks, see below |
   | Serialised to disk | Round-trip and corpus consequences |

   **Leaked types are the trap here.** `rdocx`'s public API exposes
   `rdocx-oxml` and `rdocx-layout` types (`CT_PPr`, `CT_SectPr`, `VMerge`,
   `Twips`, `FontFile`) without re-exporting them. A change to one of those is a
   breaking change to `rdocx` even though nothing in `rdocx` mentions it.

4. **Check the render path.** Does the target feed layout, shaping, colour
   resolution or PDF emission? If so, **the hash harness will move**, and that
   is a fact about the change rather than a risk of it. Say so, and say whether
   the delta is expected to be explicable.

5. **Check the corpus path.** Does it affect parsing or serialisation of a part
   type? If so the round-trip gate applies, and for pptx the "opens without
   repair" gate applies too.

6. **Check the spec.** Which `docs/hld/` sections describe the current
   behaviour. Those are the `## HLD impact` list for whatever story does this.

7. **Report** as: blast radius, semver verdict, harness and corpus consequences,
   the HLD files to update, and a suggested story size.

## Worked shape

```
Target: oxml_layout::PositionedElement

Blast radius
  oxml-pdf         12 sites (writer.rs 8, raster.rs 3, font.rs 1)
  rdocx-layout     31 construction sites
  rpptx-render     future consumer

Surface class
  Published API of oxml-layout. Also reachable through rdocx's public API.

Semver
  Adding a variant is breaking unless the enum is #[non_exhaustive].
  Take that once, at the 0.3.0 cut.

Harness
  No delta expected if existing variants are untouched. Verify, do not assume.

HLD impact
  docs/hld/08-rendering-spec.md, "Extending PositionedElement"

Suggested size
  M, if additive. L if any existing variant changes shape.
```

## Refused situations

- **Making the change.** This command assesses. Use `/design` to plan it and
  `/start-feature` to begin it.
- **Reporting a blast radius without having searched.** Grep, then report.

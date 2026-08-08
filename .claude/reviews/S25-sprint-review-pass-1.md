# S25 sprint review, pass 1

**Reviewed**: `sprint/s25` against `0140bdad3a93837c1ec0eec52305082998baed64`,
43 files, 10,196 changed lines, crates: `oxml-drawing`, `oxml-layout`,
`oxml-pdf`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`, and `rpptx`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, the M10 milestone gate still requires the missed SSIM trend target

`docs/hld/14-development-backlog.md:707`

The milestone gate still says that the SSIM harness meets its target across the
corpus. The accepted F-104 contract in the same file makes 0.95 SSIM on 80
percent a recorded trend, and the integrated gate reports 30 of 421 slides at
or above 0.95 with `target met false`. The hard automatic gate and native
PowerPoint review passed, but the milestone cannot satisfy this stale sentence.
Replace it with the current hard completeness, retained trend, and native
review contract before closure.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The backlog currently says, "the SSIM harness meets its target across the
corpus." It does not hold as written. The integrated 50-deck run rendered all
421 slides with zero dropped bounded shapes and recorded 30 slides at or above
0.95 SSIM, which is 7.126 percent and below the 80 percent trend reference.
The accepted native PowerPoint review is recorded, and all 28 deterministic
hashes remain unchanged.

## Not found

- `interaction`: table cell text threads the source-scoped hyperlink map and
  one-based page number through the same resolved and rendered text path.
- `duplication`: the sprint reuses the existing text layout, link annotation,
  media, paint, and raster paths rather than adding competing helpers.
- `layering`: no new `oxml-*` dependency points at an `rdocx-*` or `rpptx-*`
  crate. The documented `oxml-drawing` to `rdocx-oxml` exception is unchanged.
- `harness`: every plan and completion entry declares the 28-entry Word harness
  unchanged, and the integrated hash gate confirms that result.
- `deps`: `zune-jpeg` has the concrete `oxml-pdf` raster JPEG decoder as its
  current consumer. The new `rpptx` dependencies are development-only consumers
  of the whole-deck render example.
- `surface`: the deterministic presentation entry point, scoped hyperlink map,
  and resolved table values are each required and consumed by the approved
  renderer or fidelity paths.
- `gate`: no missing implementation or test evidence was found beyond B1.
- `docs`: no other contradicted HLD section was found.

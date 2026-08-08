# S25 sprint review, pass 2

**Reviewed**: `sprint/s25` at `dc282b69601f3aba99446ec38809d530dcad068e`
against `0140bdad3a93837c1ec0eec52305082998baed64`, 44 files, 10,257
changed lines, crates: `oxml-drawing`, `oxml-layout`, `oxml-pdf`,
`rpptx-layout`, `rpptx-oxml`, `rpptx-render`, and `rpptx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The backlog gate now requires the pinned 50-deck harness to render every slide
without panic, missing output, dimension mismatch, or a dropped bounded shape,
to retain the 0.95 SSIM on 80 percent trend result, and to have an accepted
native PowerPoint representative review.

The integrated gate rendered 50 decks and 421 slides with zero dropped bounded
shapes. It retained per-slide evidence and recorded 30 of 421 slides at or above
0.95 SSIM, or 7.126 percent, with median 0.622465. The missed trend is recorded
without weakening completeness. The accepted Microsoft PowerPoint 16.104
representative review and its raster hashes are recorded in the testing HLD.
The full workspace verification passed, all PowerPoint development crates remain
at version 0.0.0 with publication disabled, and all 28 deterministic hashes
remain unchanged. The milestone gate holds.

## Not found

- `interaction`: F-102 table text uses the same resolved run model, hyperlink
  lookup, page-number substitution, shaping, and annotation emission completed
  by F-103. F-104 exercises the combined renderer over every corpus slide.
- `duplication`: no competing table text, hyperlink annotation, media, paint,
  JPEG, deterministic-font, or raster path was introduced.
- `layering`: no new forbidden dependency from an `oxml-*` crate to an
  `rdocx-*` or `rpptx-*` crate exists.
- `harness`: F-102, F-103, and F-104 all declare the Word hash harness
  unchanged, their completion records agree, and the integrated check confirms
  all 28 entries.
- `gate`: the named table, hyperlink, field, completeness, metric, exact-version,
  and native review evidence covers the sprint and milestone contracts.
- `docs`: pass 1 B1 is resolved at
  `docs/hld/14-development-backlog.md:707`. No other contradicted section or
  stale delivery status was found.
- `deps`: pinned `zune-jpeg` is consumed by the concrete `oxml-pdf` raster JPEG
  path. The new `rpptx` dependencies are development-only inputs to the
  whole-deck renderer.
- `surface`: each new public resolver or renderer value has a current approved
  consumer, and no speculative trait, generic, wrapper, crate, module, or
  feature flag was added.

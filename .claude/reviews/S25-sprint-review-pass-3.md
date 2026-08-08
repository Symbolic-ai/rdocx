# S25 sprint review, pass 3

**Reviewed**: `sprint/s25` against
`0140bdad3a93837c1ec0eec52305082998baed64`, 45 files, 10,316 changed
lines, crates: `oxml-drawing`, `oxml-layout`, `oxml-pdf`, `rpptx-layout`,
`rpptx-oxml`, `rpptx-render`, and `rpptx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M10 gate holds. The integrated 50-deck harness rendered all 421 slides with
zero dropped bounded shapes and retained the full comparison evidence. It
recorded 30 slides at or above 0.95 SSIM, or 7.126 percent, as a reviewed trend
rather than a false automatic pass. The accepted Microsoft PowerPoint 16.104
representative review is recorded. Every PowerPoint development crate remains
at version 0.0.0 with publication disabled, the full workspace gate passed, and
all 28 deterministic hashes remain unchanged.

## Not found

- `interaction`: the combined table, hyperlink, field, diagnostic, and fidelity
  paths share one resolver and renderer pipeline without conflicting ownership
  or lost state.
- `duplication`: no second text, link, media, paint, JPEG, font, or raster path
  exists in the sprint delta.
- `layering`: dependency direction remains valid, including every changed
  `oxml-*` manifest.
- `harness`: plan declarations, completion records, and integrated hash evidence
  all agree on an unchanged 28-entry baseline.
- `gate`: the named focused tests, full workspace verification, complete corpus
  run, retained SSIM evidence, and native review cover the final contract.
- `docs`: the M10 milestone wording now matches the accepted F-104 gate. The
  backlog, current sprint, plans, completion log, and tracker agree on all three
  completed F-IDs.
- `deps`: every new dependency has a concrete current consumer and remains on
  the approved unpublished development path.
- `surface`: no unrequested public API, trait, generic, wrapper, crate, module,
  or feature flag was introduced.
- `preflight`: completed owners use the canonical `-` marker. The only expected
  preflight refusals before this review is committed are exact-HEAD review and
  verification records.

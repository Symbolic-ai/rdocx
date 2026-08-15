# S37 sprint review, pass 1

**Reviewed**: `sprint/s37` at
`370c7d6d00c5790ddb558918b510fbbe9ee010c4` against merge base
`6cb41a282f52aae2396bd619ec3b2a25e0f7a1a1`, 37 files and 514 changed lines.
Crates: `oxml-cli-support`, `oxml-core`, `oxml-drawing`, `oxml-layout`,
`oxml-media`, `oxml-opc`, `oxml-pdf`, `oxml-sml`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`, and
`rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The F-X006 backlog gate is: "all 14 incubating packages resolve from crates.io
at 0.1.3 with the expected owner, and the GitHub release targets the reviewed
sprint SHA."

The external gate does not hold yet, by design. The crates.io API returns 404
for all 14 version 0.1.3 candidates, and `rpptx-v0.1.3` is absent locally and
from `origin`. This is the required fresh-version state before the separately
approved `/release rpptx-v0.1.3` invocation. No final approval, tag, registry
publication, or GitHub release is inferred by this review. S37 must remain open
and F-X006 must remain in progress until the watched release succeeds and the
external gate is verified.

The release-readiness portion holds:

- Sprint state records `/verify --full` passing at the exact reviewed HEAD with
  all 28 hashes unchanged. An independent hash check also reports 28 matches.
- Cargo metadata and the lockfile expose exactly 14 publishable candidates and
  one additional unpublished preparation member, `rpptx-wasm`, all at 0.1.3.
  All 14 workspace pins are 0.1.3, and the stable family remains 0.4.1.
- Five focused release, metadata, allowlist, workflow, and authority
  regressions pass. The publish workflow retains the exact dependency-ordered
  14-package allowlist and its preflight names the 0.1.3 gate.
- The recorded full gate includes the exact 21-package locally patched dry run
  and the 10 MiB ceiling. Independent package inventories contain 20 TTFs and
  four required legal files in `oxml-layout`, no duplicate font assets in
  `rdocx-layout`, and `assets/default.pptx` in `rpptx`.
- The local and remote immutable `rpptx-v0.1.2` tag agree. The original 12
  crates.io packages resolve at 0.1.2, while `oxml-cli-support` and `rpptx-cli`
  remain absent at that version.
- Both scoped npm package lookups remain absent. The sprint diff adds no npm
  credential, OIDC, registry, publish, tag, or release authority.
- F-X006 is `reviewed` in sprint state and remains `in-progress` in both
  delivery trackers. F-143, F-144, and F-145 are complete. The working tree was
  clean at the reviewed SHA.

## Not found

- Interaction: the version preparation, HLD state, release preflight, and
  archive-ownership amendment agree at sprint scope.
- Duplication: no duplicate helper or package contract. `oxml-layout` is the
  sole bundled-font asset owner and `rdocx-layout` contains no copied inventory.
- Layering: no dependency edge changed, and no forbidden cross-family edge was
  introduced.
- Harness: the design declares no delta, sprint verification records
  `unchanged`, and an independent check reports all 28 entries matching.
- Gate: no local release-readiness gap. The remaining registry and GitHub gate
  is explicitly deferred to the separately approved release phase.
- Docs: exactly HLD 03, HLD 14, and HLD 15 changed as listed by the plan. They
  describe prepared 0.1.3 state, pending publication, immutable 0.1.2 history,
  the 15 prepared versus 14 publishable boundary, and corrected archive
  ownership consistently.
- Dependencies: no dependency was added or removed.
- Surface: no public Rust API or rendering behaviour changed.

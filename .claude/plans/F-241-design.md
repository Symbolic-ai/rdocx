# F-241, Public authoring conformance harness

**Status**: approved
**Sprint**: S70
**Size**: L
**Depends on**: F-240

## Problem

The current Word differential harness owns the pinned render oracle and public
corpus, but it does not prove that a fixture starts from `Document::new()` and
uses only the public `rdocx` facade (`scripts/docx_ssim_harness.py:34`). It also
does not compare from-scratch package ownership, save-reopen state, unsupported
diagnostics, or a confidential local corpus.

The M23 contract requires one public synthetic gate and one ignored required
private mode. Missing private inputs must be an explicit optional skip or a
required-mode failure, never an accidental weakening
(`docs/hld/12-testing-strategy.md:1314`). Combining those concerns inside the
existing 1,200-line render harness would obscure which code owns authoring,
privacy, package, and visual acceptance.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, deterministic package ownership,
  relationships, content types, schema order, and unmodeled preservation.
- `docs/hld/10-bindings-spec.md`, the public native facade and binding boundary.
- `docs/hld/12-testing-strategy.md`, "The private from-scratch DOCX conformance
  corpus" and "The Word render fidelity gate".
- `docs/hld/14-development-backlog.md`, "F-241, Public authoring conformance
  harness".
- `docs/hld/15-build-and-toolchain.md`, deterministic tools, CI, and private
  evidence boundaries.
- `.claude/skills/differential-testing.md`, pinned structural and render oracle
  rules.

## Approach

Create `scripts/docx_authoring_conformance.py` as the single owner of the new
gate. Provide `--self-test`, `--public`, optional-private default behavior, and
`--private-required` modes. Public mode compiles a temporary consumer with only
the local `rdocx` dependency, starts with `Document::new()`, and rejects base
DOCX input, raw XML injection, `rdocx-oxml`, and direct `oxml-*` dependencies.

Run ordered stages for generation provenance, normalized part and relationship
graphs, content types, schema order, save-reopen modeled assertions, exact
unknown-subtree preservation, repeated deterministic output, bundled-font
rendering, and explicit unsupported-content diagnostics. Compare structural
trees rather than producer ZIP or XML bytes.

Private mode reads exactly anonymous P1 through P5 plus an ignored local
manifest and ignored evidence directory. It never executes commands from the
manifest and never prints private text, names, XML, media, or hashes. Optional
mode reports a clear skip when inputs are absent. Required mode fails on
missing, extra, changed, or incomplete private evidence. Scan tracked and
staged paths for private artifacts without echoing sensitive values.

Render at 150 DPI in deterministic bundled-font mode. Store per-case reviewed
geometry and SSIM thresholds in the ignored manifest because the five layouts
have different valid tolerances. Pin and assert every external tool version.
Extend the existing `rdocx` integration binary and
`scripts/test_sprint_workflow.py`, and add public CI mode without creating a new
Rust test binary.

## Rejected alternatives

- Extend `docx_ssim_harness.py` with unrelated package and privacy ownership.
  That file already has a distinct render-oracle purpose.
- Commit private fixtures, hashes, renders, or manifests. They are customer
  data or identifying derivatives.
- Compare package bytes with Word or LibreOffice output. Prefix, ordering, and
  ZIP metadata differences are not the modeled contract.
- Use one unexplained global pixel threshold. Each private case needs a reviewed
  structural and visual expectation.
- Let a missing private directory pass required mode. That would weaken the
  acceptance gate silently.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `sanitized_public_authoring_fixture_passes_every_conformance_stage` | A source-built public fixture passes package, reopen, deterministic render, preservation, and diagnostic stages. |
| regression | `public_fixture_rejects_base_package_raw_xml_or_private_oxml_dependency` | The generator boundary cannot bypass public `rdocx` authoring. |
| round-trip | public fixture save and reopen | Modeled state and an unmodeled probe survive with schema-correct ownership. |
| golden | repeated deterministic bundled-font render | DOCX structure and rendered output are stable at the declared 150 DPI thresholds. |
| regression | optional skip and required-private failure matrix | Missing, extra, changed, or incomplete corpus evidence is reported without disclosure. |
| regression | `tracked_or_staged_private_artifacts_are_rejected_without_disclosure` | Private inputs and identifying derivatives cannot enter tracked history. |

The **test gate is differential**. A sanitized representative fixture proves
every gate locally, while missing private inputs report a clear skip rather
than weakening required private-corpus mode.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Layout, pagination, line breaking, and text shaping**. Render only with
  deterministic bundled fonts at 150 DPI and never record a system-font
  baseline.
- **Any parser or serializer**. Keep reads prefix-tolerant, validate schema
  order, and prove unmodeled subtree preservation byte for byte.
- **A new module or file**. Create exactly one focused script only after the
  consolidated design approval.
- **External oracle comparison**. Pin each oracle version, compare structural
  trees rather than bytes, and keep reviewed per-case visual thresholds.

## Hash harness

Expected unchanged across all 49 entries. The new harness validates separate
source-built fixtures and does not alter shipped rendering behavior.

## Implementation checklist

- [ ] Add failing public-boundary, package, reopen, privacy, and mode tests.
- [ ] Create the one focused conformance script after explicit approval.
- [ ] Build the public fixture through a temporary `rdocx`-only consumer.
- [ ] Add structural, deterministic, diagnostic, and privacy gate stages.
- [ ] Add optional and required ignored-corpus modes for anonymous P1 through P5.
- [ ] Pin external tools and record private per-case thresholds outside git.
- [ ] Add public mode to CI and preserve private mode as local required evidence.
- [ ] Run focused tests, full verification, hash checks, and routed riders.
- [ ] Update exactly the listed HLD files.

## Open questions

None. The user approved the focused `scripts/docx_authoring_conformance.py`
file and 150 DPI rendering with per-case reviewed geometry and SSIM thresholds
stored in the ignored manifest.

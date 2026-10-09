# F-X191, ALL, pass 1

**Reviewed**: Frozen working diff on `work/f-x191-codex`, exact Base and HEAD `49249290bc9efb68192cbeda3437e361b9042368`. Four files, 205 insertions and 9 deletions. Diff SHA `c6bd2b14d981c4010722029ca84e17375d637eb9dda068fd9c0b3ac4e9816812`.
**Verdict**: 0 defects, 0 smells, 0 nitpicks.

## Contract and scope

Read CLAUDE, WORKFLOW, microscope command, approved F-X191 plan, current sprint, backlog and progress. F-X191 is in-progress and owned by codex. There is no prior F-X191 review. Read the cited HLD10 reader and authoring contracts, HLD04 package transaction contract and HLD06 namespace and preservation principles.

The production change at `crates/rdocx/src/document.rs:20548` preserves all three extra XML predicates and excludes only lexical XML declaration names from the three retained attribute collections. Actual producer attributes, same-local lookalikes and foreign rebound aliases remain reported. No parser, serializer, namespace storage, numbering value, public signature or binding surface changes. The richer authoring helpers at `crates/rdocx/src/document.rs:30134` and instance and definition admission facts remain unchanged.

The README and script delta updates only the actual current rdocx archive tuple. No upstream platform override, member policy, size bound or compression tolerance change was imported. HLD10 describes current reader intent and preserves the distinct authoring admission contract.

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: zero findings. The three-owner predicate is local and retains all existing extra XML checks.
- Contract: zero findings. The reader fact changes exactly as approved, with no mutation admission expansion.
- Panics: zero findings. Production adds no indexing, slicing, arithmetic or fallible unchecked operation. Test unwraps address constructed fixtures.
- OOXML: zero findings. Retained declaration storage and writer output are unchanged. Tests cover imported and saved/reopened declaration retention, actual producer attributes, rebound foreign qualification and byte-exact opaque children on all three owners.
- Tests: zero findings. The primary regression compiles and executes against exact Base production, gathers all twelve owner/declaration combinations before asserting and proves failure on both imported and reopened documents. Repair evidence executes all seven public reader and compatibility controls.
- Structure: zero findings. No trait, generic, dependency, production file, module, wrapper or speculative abstraction is introduced.
- HLD and documentation: zero findings. Both approved HLD10 sections and method documentation match the behavior. Archive documentation uses the existing canonical measurement policy.
- Archive policy: zero findings. The available current archive has 1,583,630 compressed bytes, 8,813,125 canonically normalized member bytes and 36 members. Its packaged document.rs equals frozen source. The exact member policy, below10MiB bound and 64-byte compression tolerance remain unchanged.

## Evidence and source authentication

Binder `/private/tmp/fx191-review-source-freeze/binder.json`, SHA `81e93a616c7f15ae5afb065c7395eb9b22dcbde53f2d3414b68532ea62693efb`, binds sixteen resolved worker source and record paths and five evidence paths. All live and captured hashes authenticated at entry. All sixteen live source and record hashes authenticated again at review exit. The working diff matched its bound SHA. No reviewed source changed during this pass.

Frozen document.rs SHA `3b8eec9fc3980a9dc743be2faef39d1126c819be102a61b58a4f6ae798a59620`.

The retained before source has SHA `ccc00d86cabc589a058b229ee79e0582ddb647847395068d7c97f2cd1d16ad06`. Independent comparison confirms its complete non-test production equals exact Base. Before log SHA `2d93f7c8d59f569fe4fd03ee7148c71aca83b6aa838343a20f741b0c5a655f79` records a completed compilation and actual twelve-case imported and reopened boolean failure, not a compile error or missing test. Focused after log SHA `4dc2a4c6d618d4b4c51f3b250f76c360ba1e241db436eb1d0e33acdf077ce095` records seven executed tests passing.

Current archive SHA `7baa16f796171366d4e4500ef2f4d9411d7044d9cb1693b8e3909756a9405713` independently matches the canonical normalized measurement tuple. This is the available measured archive observation, not a substitute for the remaining prescribed package and publication dry-run gates.

## Completion qualification

This pass audits immutable implementation and available source-bound evidence. Scoped runtime and risk gates remain pending at review time. The worker must finish and authenticate prescribed scoped checks, rustdoc, Clippy, formatting, prose, adapter, unchanged49-entry harness, archive checks and publication dry run before prepared handoff. Full integrated sprint verification, sprint review and separate final release approval remain required. Zero review findings do not claim those gates have completed.

No remediation or writes outside this sole review record were performed. Control returns to the orchestrator.

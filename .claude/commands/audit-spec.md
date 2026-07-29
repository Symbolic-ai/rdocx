---
description: Scan for drift between the docs/hld/ spec set and the code that exists. Reports, changes nothing.
---

# /audit-spec [--section NN]

Find where the spec set and the workspace disagree. **This command changes no
file.** It produces a report someone then acts on.

Run it before a milestone boundary, before `/spec-bump`, after a long absence,
and whenever `/sync-status` comes back clean but something still feels stale.
`/sync-status` checks the trackers against each other. This checks the spec
against the code.

## Steps

1. **Build the spec inventory.** Walk `docs/hld/00-vision.md` through
   `15-build-and-toolchain.md` and index every checkable claim:
   - Crate names, and the dependency edges `03-architecture.md` draws.
   - Type and function names cited by `04` through `10`.
   - Feature flags and their defaults, from `15-build-and-toolchain.md`.
   - Story IDs, sizes and test gates, from `14-development-backlog.md`.
   - Named test categories and harnesses, from `12-testing-strategy.md`.
   - Bundled asset claims, fonts and their licences.

2. **Build the code inventory.**

   ```bash
   cargo metadata --no-deps --format-version 1
   ```

   Plus the things metadata does not answer: `[features]` blocks across every
   `Cargo.toml`, `pub` items in each crate's `lib.rs`, the files under
   `crates/rdocx-layout/fonts/` and `crates/rpptx/assets/`, and the integration
   test binaries under each `tests/`.

3. **Diff into four buckets.**

   | Bucket | Meaning |
   |---|---|
   | Missing | The spec names it, the code has no such thing |
   | Undocumented | The code has it, no spec section mentions it |
   | Shape divergence | Both have it, with different shapes |
   | Contradiction | Two spec documents disagree with each other |

4. **Run the checks that are specific to this workspace.** These are the ones
   worth the command existing:

   - **Dependency direction.** Every `oxml-*` crate's dependency list against
     the rule in `03-architecture.md`. A path from `oxml-*` into `rdocx-*` or
     `rpptx-*`, other than the documented `oxml-drawing -> rdocx-oxml` `Theme`
     adapter, is a **contradiction**, not a note.
   - **Backlog integrity.** Every `### F-XXX` in `14-development-backlog.md`
     appears in `docs/sprints/BACKLOG.md` and the reverse. Sizes and test gates
     match. Each `Depends on` names a story that exists.
   - **Feature flags.** Every flag in a `Cargo.toml` has a named consumer, and
     `15-build-and-toolchain.md` describes every flag that exists.
   - **Bundled assets.** Every font family under `crates/rdocx-layout/fonts/`
     has a licence file, and the licence the code claims is the real one. Every
     asset `rpptx` loads lives under the crate directory, since one outside it
     is absent from the published tarball.
   - **Carried defects.** Each entry in `CLAUDE.md` under "Known defects being
     carried" either still exists at the cited location, or has been fixed and
     the entry is stale. Both are findings.
   - **Deliberate wrongness.** Each entry under "Things that are deliberately
     wrong" still matches the code, and the document that explains why still
     says so.

5. **Report.** Group by bucket, and within a bucket by spec document. Every
   finding carries a `path:line` on both sides where both exist.

   ```markdown
   ## Drift report, <date>

   ### Contradiction
   - `docs/hld/03-architecture.md:41` forbids `oxml-* -> rpptx-*`, but
     `crates/oxml-layout/Cargo.toml:19` depends on `rpptx-oxml`.

   ### Missing in code
   - `docs/hld/06-presentationml-model.md:88` names `CT_SlideMaster`, absent
     from `crates/rpptx-oxml/`. Scheduled by F-071, which is `pending`.

   ### Undocumented in spec
   - `crates/oxml-core/src/raw_xml.rs:14`, `capture_element_ns`, unmentioned.

   ### Shape divergence
   - `docs/hld/15-build-and-toolchain.md:52` lists `bundled-fonts` as
     default-off. `crates/rdocx-layout/Cargo.toml:24` has it default-on.

   ### Checked and clean
   Named, so a reader knows what was covered.
   ```

6. **Triage each finding** in the report itself. Undocumented is not
   automatically drift, and missing is not automatically a bug:

   | Finding | Usual action |
   |---|---|
   | Missing, and a `pending` story owns it | Expected. Name the story and move on |
   | Missing, and no story owns it | Real gap. Needs a backlog entry |
   | Undocumented, and it is a real feature | The spec needs updating |
   | Undocumented, and it is leftover scaffolding | Delete the code |
   | Shape divergence | The spec is usually the contract and usually right. Decide which is wrong before changing either |
   | Contradiction | Stop. Resolve before implementing anything that touches it |

7. **Exit clean or not.** Zero drift, or a count by bucket. `/spec-bump` reads
   this result.

## Limitations, which the report must state

- Static only. It cannot see that the code does X while claiming X.
- Spec prose is ambiguous in places, so the match is heuristic. A finding is a
  prompt to look, not a proof.
- It does not check the rendered output against anything. That is the hash
  harness and the differential corpus.

## Refused situations

- **Editing any file.** This reports. `/realign-docs` and `/spec-bump` act.
- **Reporting a finding without a citation on the side that exists.** Delete it
  rather than softening it.
- **Manufacturing findings.** Zero in a bucket is a valid and common result.
  Say so by name.

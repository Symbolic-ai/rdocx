---
description: The gate. Runs formatting, lints, tests, the hash harness and the prose rules.
---

# /verify [--fast | --scoped F-XXX | --full]

There are three deliberately different verification scopes:

- `--fast` is the inner loop. It runs formatting, changed-crate linting, and
  changed-crate tests.
- `--scoped F-XXX` is the feature-completion gate. It runs the fast checks,
  the hash and repository-policy checks, and the exact risk riders declared by
  that feature's approved design plan.
- `--full` is the integrated sprint gate. It runs everything across the
  workspace and is required by `/close-sprint`.

Do not silently widen `--scoped` to `--full`. Feature completion proves the
feature contract. Sprint closure proves that the integrated result works as a
whole.
Run only the focused external oracle cases named by the approved story plan
under `--scoped`. Run the union of applicable oracle riders at the integrated
`--full` gate. A dependency-prefix checkpoint confirms scoped evidence and focused integration
checks. The full gate runs on the final integrated sprint result.

## Steps

1. **Format.** `cargo fmt --all --check`.

2. **Lint.** For `--fast` and `--scoped`, run `cargo clippy -p <crate>
   --all-targets --all-features -- -D warnings` for each changed crate. For
   `--full`, run `cargo clippy --workspace --all-targets --all-features
   --exclude rdocx-py --exclude rpptx-py -- -D warnings`.

3. **Test the changed crates.** `cargo test -p <crate>` for each crate touched.
   For `--scoped`, determine the set from `git diff --name-only` against the
   feature's starting commit in `.claude/scratch/<F-ID>-progress.md`, or the
   handoff `Base` for a worker. Do not use the sprint base because that reruns
   every earlier story in the sprint. For `rdocx-py` and `rpptx-py`, run the
   isolated Python suite declared by the design plan instead. Their
   `pyo3/extension-module` configuration cannot link a Rust test binary.

4. **Test the workspace.**
   `cargo test --workspace --all-features --exclude rdocx-py --exclude rpptx-py`.
   `--full` only.

5. **The hash harness.** `python3 scripts/hash_harness.py --check`. Skipped by
   `--fast` and required by `--scoped` and `--full`.

   **Mandatory for every story in M1 through M6.** An unexplained delta fails
   the gate. An expected delta must be declared in the design plan's
   `## Hash harness` section, and the reported delta must match what was
   declared. A delta that is real but undeclared is a failure, not a prompt to
   update the baseline.

6. **The repository-policy checks.** Skipped by `--fast` and required by
   `--scoped` and `--full`.

   Run `python3 scripts/prose_check.py` over tracked Markdown and the commit
   message. No em-dash, no en-dash, no prose semicolon.

   Then `python3 scripts/sync_agent_skills.py --check`. The `.agents/skills/`
   adapters are generated from `.claude/commands/` and `.claude/skills/`, and
   drift means Codex and Claude are following different rules while believing
   they agree. Regenerate with the same script and commit the result.

   Then `python3 -m unittest scripts.test_sprint_workflow`. This holds the
   release family preflights that `.github/workflows/wheels.yml` invokes by
   name as the publication gate, and the assertions over the pinned CI
   toolchains. A failure means a version carrier moved without its assertion
   moving with it. The fix is the carrier or the assertion, never deleting the
   test.

   Without this step the preflights run for the first time on a tag, after the
   sprint is closed. S42 is the demonstration: F-X022 moved every version
   carrier under `crates/`, passed the entire local gate, and left the
   incubating preflight and the `ci.yml` WASM literal asserting the old
   version. The whole module takes about four seconds.

7. **The no-default-features path.**
   `cargo test -p oxml-layout --no-default-features`. This is the only thing
   that exercises system font discovery being off while bundled fonts remain
   available. Run it under `--full`, or under `--scoped` when the design
   plan's risk routing requires it. Skip it under `--fast`.

8. **The wasm targets.**
   `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`.
   Run under `--full`, or under `--scoped` when the design plan's risk routing
   requires it. Skip under `--fast`.

9. **Docs.** `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D warnings`,
   then `python3 scripts/readme_doctests.py`. Run under `--full`, or under
   `--scoped` when the design plan's risk routing requires it. Skip under
   `--fast`.

10. **Packaging.** Run the workspace dry run with every publishable internal
    crate patched to its reviewed local source:

    ```bash
    cargo publish --workspace --dry-run \
      --config 'patch.crates-io.oxml-core.path="crates/oxml-core"' \
      --config 'patch.crates-io.oxml-drawing.path="crates/oxml-drawing"' \
      --config 'patch.crates-io.oxml-layout.path="crates/oxml-layout"' \
      --config 'patch.crates-io.oxml-media.path="crates/oxml-media"' \
      --config 'patch.crates-io.oxml-opc.path="crates/oxml-opc"' \
      --config 'patch.crates-io.oxml-pdf.path="crates/oxml-pdf"' \
      --config 'patch.crates-io.oxml-sml.path="crates/oxml-sml"' \
      --config 'patch.crates-io.oxml-cli-support.path="crates/oxml-cli-support"' \
      --config 'patch.crates-io.oxml-chart.path="crates/oxml-chart"' \
      --config 'patch.crates-io.rdocx.path="crates/rdocx"' \
      --config 'patch.crates-io.rdocx-cli.path="crates/rdocx-cli"' \
      --config 'patch.crates-io.rdocx-html.path="crates/rdocx-html"' \
      --config 'patch.crates-io.rdocx-layout.path="crates/rdocx-layout"' \
      --config 'patch.crates-io.rdocx-opc.path="crates/rdocx-opc"' \
      --config 'patch.crates-io.rdocx-oxml.path="crates/rdocx-oxml"' \
      --config 'patch.crates-io.rdocx-pdf.path="crates/rdocx-pdf"' \
      --config 'patch.crates-io.rpptx.path="crates/rpptx"' \
      --config 'patch.crates-io.rpptx-cli.path="crates/rpptx-cli"' \
      --config 'patch.crates-io.rpptx-chart.path="crates/rpptx-chart"' \
      --config 'patch.crates-io.rpptx-layout.path="crates/rpptx-layout"' \
      --config 'patch.crates-io.rpptx-oxml.path="crates/rpptx-oxml"' \
      --config 'patch.crates-io.rpptx-render.path="crates/rpptx-render"'
    ```

    Cargo rewrites packaged path dependencies to the registry. The local
    patches keep archive verification on this exact reviewed source graph,
    including versions that have not yet been published. They do not enter
    any generated archive. Then assert every generated archive is below the
    crates.io 10 MiB limit:

    ```bash
    oversized=$(find target/package -name '*.crate' -size +10485760c -print)
    if [ -n "$oversized" ]; then
      echo "package exceeds 10 MiB: $oversized" >&2
      exit 1
    fi
    ```

    Run under `--full`, or under `--scoped` when the design plan's risk routing
    requires it. For an uncommitted `--scoped` feature, add `--allow-dirty` to
    the command above so Cargo verifies the actual working source. Record that
    flag in the feature evidence. The `--full` gate on the clean integrated
    tree must use the command above without `--allow-dirty`. Skip packaging
    under `--fast`. This is what `--no-verify` used to hide.

11. **Supply chain.** `cargo deny check`. Run under `--full`, or under
    `--scoped` when the design plan's risk routing requires it. Skip under
    `--fast`.

## Reporting

Report each step as pass or fail with its command. On failure, show the actual
output rather than summarising it, and stop at the first failing step unless
the failures are independent.

**Never report a pass you did not observe.** If a step was skipped, say it was
skipped and why.

## Refused situations

- **`--fast` as the gate for `/complete-feature`.** Run `/verify --scoped
  F-XXX` instead.
- **`--scoped` without the feature's declared risk riders.** The plan defines
  the scope. Omitting a matched rider is a failed gate.
- **Updating the hash baseline to make step 5 pass.** The baseline changes only
  through a declared, reviewed delta.

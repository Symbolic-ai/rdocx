---
description: The gate. Runs formatting, lints, tests, the hash harness and the prose rules.
---

# /verify [--fast] [--full]

The gate that must pass before `/complete-feature`. `--fast` runs steps 1 to 4
against changed crates only, for the inner loop. `--full` runs everything across
the workspace, which is what `/close-sprint` requires.

## Steps

1. **Format.** `cargo fmt --all --check`.

2. **Lint.** `cargo clippy --workspace --all-targets --all-features -D warnings`,
   with `--exclude rdocx-py --exclude rpptx-py`.

3. **Test the changed crates.** `cargo test -p <crate>` for each crate touched.
   Determine the set from `git diff --name-only` against the sprint base.

4. **Test the workspace.**
   `cargo test --workspace --all-features --exclude rdocx-py --exclude rpptx-py`.
   Skipped by `--fast`.

5. **The hash harness.** `python3 scripts/hash_harness.py --check`.

   **Mandatory for every story in M1 through M6.** An unexplained delta fails
   the gate. An expected delta must be declared in the design plan's
   `## Hash harness` section, and the reported delta must match what was
   declared. A delta that is real but undeclared is a failure, not a prompt to
   update the baseline.

6. **The prose rules.** `python3 scripts/prose_check.py` over tracked Markdown
   and the commit message. No em-dash, no en-dash, no prose semicolon.

   Then `python3 scripts/sync_agent_skills.py --check`. The `.agents/skills/`
   adapters are generated from `.claude/commands/` and `.claude/skills/`, and
   drift means Codex and Claude are following different rules while believing
   they agree. Regenerate with the same script and commit the result.

7. **The no-default-features path.**
   `cargo test -p oxml-layout --no-default-features`. This is the only thing
   that exercises bundled fonts being off.

8. **The wasm targets.**
   `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`.
   Skipped by `--fast`.

9. **Docs.** `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D warnings`.
   Skipped by `--fast`.

10. **Packaging.** `cargo publish --dry-run` plus the `.crate` size assertion.
    `--full` only. This is what `--no-verify` used to hide.

11. **Supply chain.** `cargo deny check`. `--full` only.

## Reporting

Report each step as pass or fail with its command. On failure, show the actual
output rather than summarising it, and stop at the first failing step unless
the failures are independent.

**Never report a pass you did not observe.** If a step was skipped, say it was
skipped and why.

## Refused situations

- **`--fast` as the gate for `/complete-feature`.** It is the inner loop only.
- **Updating the hash baseline to make step 5 pass.** The baseline changes only
  through a declared, reviewed delta.

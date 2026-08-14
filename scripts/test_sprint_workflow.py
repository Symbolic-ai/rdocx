from __future__ import annotations

import argparse
import contextlib
import hashlib
import io
import json
import os
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import sprint_workflow as workflow


class SprintWorkflowTests(unittest.TestCase):
    def yaml_block(self, source: str, header: str) -> str:
        lines = source.splitlines()
        matches = [index for index, line in enumerate(lines) if line == header]
        self.assertEqual(len(matches), 1, header)
        start = matches[0]
        indentation = len(header) - len(header.lstrip())
        end = len(lines)
        for index in range(start + 1, len(lines)):
            line = lines[index]
            if line.strip() and len(line) - len(line.lstrip()) <= indentation:
                end = index
                break
        return "\n".join(lines[start:end]) + "\n"

    def yaml_step(self, job: str, name: str) -> str:
        return self.yaml_block(job, f"      - name: {name}")

    def yaml_direct_lines(self, block: str, indentation: int) -> tuple[str, ...]:
        return tuple(
            line.strip().split(" #", 1)[0].rstrip()
            for line in block.splitlines()[1:]
            if line.strip()
            and not line.lstrip().startswith("#")
            and len(line) - len(line.lstrip()) == indentation
        )

    def operative_lines(self, block: str) -> tuple[str, ...]:
        return tuple(
            line.strip().split(" #", 1)[0].rstrip()
            for line in block.splitlines()
            if line.strip() and not line.lstrip().startswith("#")
        )

    def yaml_steps(self, job: str) -> tuple[str, ...]:
        steps = self.yaml_block(job, "    steps:")
        lines = steps.splitlines()[1:]
        starts = tuple(
            index
            for index, line in enumerate(lines)
            if line.strip().startswith("- ")
            and not line.lstrip().startswith("#")
            and len(line) - len(line.lstrip()) == 6
        )
        self.assertTrue(starts)
        return tuple(
            "\n".join(lines[start:end]) + "\n"
            for start, end in zip(starts, starts[1:] + (len(lines),))
        )

    def yaml_step_identity(self, step: str, position: int) -> str:
        header = self.operative_lines(step)[0]
        if header.startswith("- name: "):
            return header.removeprefix("- name: ")
        if header.startswith("- id: "):
            return "id:" + header.removeprefix("- id: ")
        return f"step:{position}"

    def yaml_step_actions(self, step: str) -> tuple[str, ...]:
        actions = []
        for line in step.splitlines():
            indentation = len(line) - len(line.lstrip())
            stripped = line.strip()
            if indentation == 6 and stripped.startswith("- uses: "):
                value = stripped.removeprefix("- uses: ")
            elif indentation == 8 and stripped.startswith("uses: "):
                value = stripped.removeprefix("uses: ")
            else:
                continue
            actions.append(value.split(" #", 1)[0].rstrip())
        return tuple(actions)

    def yaml_run_lines(self, step: str) -> tuple[str, ...]:
        run = self.yaml_block(step, "        run: |")
        return self.operative_lines(run)[1:]

    def assert_no_success_short_circuit(self, lines: tuple[str, ...]) -> None:
        for line in lines:
            tokens = tuple(
                token.strip("'\"()")
                for token in line.replace(";", " ")
                .replace("&&", " ")
                .replace("||", " ")
                .split()
            )
            self.assertNotIn("true", tokens, line)
            for index, token in enumerate(tokens[:-1]):
                self.assertFalse(
                    token in ("exit", "return") and tokens[index + 1] == "0",
                    line,
                )

    def assert_python_pr_job_contract(self, ci: str) -> None:
        triggers = self.yaml_block(ci, "on:")
        trigger_keys = tuple(
            line.split(":", 1)[0]
            for line in self.yaml_direct_lines(triggers, 2)
        )
        self.assertEqual(trigger_keys, ("push", "pull_request", "schedule"))
        pull_request = self.yaml_block(triggers, "  pull_request:")
        self.assertEqual(self.yaml_direct_lines(pull_request, 4), ())

        root_permissions = self.yaml_block(ci, "permissions:")
        self.assertEqual(
            self.yaml_direct_lines(root_permissions, 2),
            ("contents: read",),
        )
        operative_ci = self.operative_lines(ci)
        self.assertFalse(any("id-token:" in line for line in operative_ci))
        self.assertFalse(any("write-all" in line for line in operative_ci))
        self.assertFalse(any("PYTEST_ADDOPTS" in line for line in operative_ci))

        job = self.yaml_block(ci, "  python-bindings:")
        direct = self.yaml_direct_lines(job, 4)
        self.assertEqual(
            direct,
            (
                "name: Python bindings (${{ matrix.package.distribution }})",
                "runs-on: macos-26",
                "strategy:",
                "steps:",
            ),
        )
        self.assertFalse(
            any("continue-on-error:" in line for line in self.operative_lines(job))
        )

        strategy = self.yaml_block(job, "    strategy:")
        self.assertEqual(
            self.yaml_direct_lines(strategy, 6),
            ("fail-fast: false", "matrix:"),
        )
        matrix = self.yaml_block(strategy, "      matrix:")
        self.assertEqual(self.yaml_direct_lines(matrix, 8), ("package:",))
        package = self.yaml_block(matrix, "        package:")
        self.assertEqual(
            self.yaml_direct_lines(package, 10),
            (
                "- { distribution: rdocx, crate: rdocx-py, "
                'oracle: "python-docx==1.2.0" }',
                "- { distribution: rpptx, crate: rpptx-py, "
                'oracle: "python-pptx==1.0.2" }',
            ),
        )

        steps = self.yaml_steps(job)
        identities = tuple(
            self.yaml_step_identity(step, position)
            for position, step in enumerate(steps)
        )
        required_order = (
            "step:0",
            "step:1",
            "step:2",
            "Set up Python 3.12",
            "Install pinned Poppler",
            "Create isolated binding environment",
            "Build Python extension",
            "Run full Python binding suite",
        )
        self.assertEqual(identities, required_order)

        action_contract = (
            (
                steps[0],
                "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd",
            ),
            (
                steps[1],
                "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c",
            ),
            (
                steps[2],
                "Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4",
            ),
        )
        for action_step, expected_action in action_contract:
            self.assertEqual(self.yaml_step_actions(action_step), (expected_action,))
            self.assertEqual(self.yaml_direct_lines(action_step, 8), ())

        setup = self.yaml_step(job, "Set up Python 3.12")
        self.assertEqual(
            self.yaml_step_actions(setup),
            ("actions/setup-python@a309ff8b426b58ec0e2a45f0f869d46889d02405",),
        )
        self.assertEqual(
            self.yaml_direct_lines(setup, 8),
            (
                "uses: actions/setup-python@"
                "a309ff8b426b58ec0e2a45f0f869d46889d02405",
                "with:",
            ),
        )
        setup_with = self.yaml_block(setup, "        with:")
        self.assertEqual(
            self.yaml_direct_lines(setup_with, 10),
            ('python-version: "3.12.9"',),
        )

        poppler = self.yaml_step(job, "Install pinned Poppler")
        self.assertEqual(self.yaml_run_lines(poppler), ("brew install poppler",))

        environment = self.yaml_step(job, "Create isolated binding environment")
        self.assertEqual(
            self.yaml_run_lines(environment),
            (
                'binding_venv="${RUNNER_TEMP}/${{ matrix.package.distribution }}-venv"',
                'python -m venv "$binding_venv"',
                'binding_python="$binding_venv/bin/python"',
                '"$binding_python" -m pip install \\',
                'maturin==1.13.3 \\',
                'pytest==9.1.1 \\',
                '"${{ matrix.package.oracle }}"',
            ),
        )

        build = self.yaml_step(job, "Build Python extension")
        build_lines = self.yaml_run_lines(build)
        self.assertEqual(
            build_lines,
            (
                'binding_venv="${RUNNER_TEMP}/${{ matrix.package.distribution }}-venv"',
                'binding_python="$binding_venv/bin/python"',
                'VIRTUAL_ENV="$binding_venv" \\',
                '"$binding_python" -m maturin develop --locked \\',
                '--manifest-path "crates/${{ matrix.package.crate }}/Cargo.toml"',
            ),
        )

        tests = self.yaml_step(job, "Run full Python binding suite")
        self.assertEqual(
            self.yaml_direct_lines(tests, 8),
            ("shell: bash", "run: |"),
        )
        test_lines = self.yaml_run_lines(tests)
        self.assertEqual(
            test_lines,
            (
                'binding_venv="${RUNNER_TEMP}/${{ matrix.package.distribution }}-venv"',
                'binding_python="$binding_venv/bin/python"',
                '"$binding_python" -m pytest "crates/${{ matrix.package.crate }}/tests"',
            ),
        )
        self.assert_no_success_short_circuit(build_lines + test_lines)
        self.assertNotIn("|| true", job)
        self.assertNotIn("set +e", job)

        for rust_job_name in ("test", "clippy", "doc", "msrv"):
            rust_job = self.yaml_block(ci, f"  {rust_job_name}:")
            self.assertIn("--all-features", rust_job, rust_job_name)
            self.assertIn("--exclude rdocx-py", rust_job, rust_job_name)
            self.assertIn("--exclude rpptx-py", rust_job, rust_job_name)

    def test_python_pr_job_builds_both_extensions_before_pytest(self) -> None:
        ci = (workflow.REPO / ".github/workflows/ci.yml").read_text(
            encoding="utf-8"
        )
        self.assert_python_pr_job_contract(ci)

    def test_workspace_test_jobs_fetch_the_pinned_presentation_corpus(self) -> None:
        ci = (workflow.REPO / ".github/workflows/ci.yml").read_text(
            encoding="utf-8"
        )
        for job_name in ("test", "msrv"):
            with self.subTest(job=job_name):
                job = self.yaml_block(ci, f"  {job_name}:")
                fetch = self.yaml_step(job, "Fetch pinned presentation corpus")
                self.assertEqual(
                    self.yaml_direct_lines(fetch, 8),
                    ("run: python3 scripts/fetch_pptx_corpus.py",),
                )
                test_steps = tuple(
                    step
                    for step in self.yaml_steps(job)
                    if "cargo test --workspace" in step
                )
                self.assertEqual(len(test_steps), 1)
                self.assertLess(job.index(fetch), job.index(test_steps[0]))
                self.assertNotIn("continue-on-error", fetch)

    def test_python_pr_job_rejects_failure_swallowing_and_incomplete_cells(
        self,
    ) -> None:
        ci = (workflow.REPO / ".github/workflows/ci.yml").read_text(
            encoding="utf-8"
        )
        self.assert_python_pr_job_contract(ci)
        mutations = {
            "missing-pull-request-trigger": ci.replace(
                "  pull_request:\n", "", 1
            ),
            "commented-pull-request-trigger": ci.replace(
                "  pull_request:\n", "  # pull_request:\n", 1
            ),
            "root-contents-write": ci.replace(
                "  contents: read\n", "  contents: write\n", 1
            ),
            "root-contents-write-with-required-comment": ci.replace(
                "  contents: read\n",
                "  contents: write # contents: read\n",
                1,
            ),
            "root-id-token-write": ci.replace(
                "  contents: read\n",
                "  contents: read\n  id-token: write\n",
                1,
            ),
            "job-if-false": ci.replace(
                "    name: Python bindings (${{ matrix.package.distribution }})\n",
                "    name: Python bindings (${{ matrix.package.distribution }})\n"
                "    if: false\n",
                1,
            ),
            "job-if-true": ci.replace(
                "    name: Python bindings (${{ matrix.package.distribution }})\n",
                "    name: Python bindings (${{ matrix.package.distribution }})\n"
                "    if: true\n",
                1,
            ),
            "job-pytest-collect-only-environment": ci.replace(
                "    runs-on: macos-26\n",
                "    runs-on: macos-26\n"
                "    env:\n"
                "      PYTEST_ADDOPTS: --collect-only\n",
                1,
            ),
            "root-pytest-collect-only-environment": ci.replace(
                "env:\n  CARGO_TERM_COLOR: always\n",
                "env:\n"
                "  CARGO_TERM_COLOR: always\n"
                "  PYTEST_ADDOPTS: --collect-only\n",
                1,
            ),
            "missing-rpptx-cell": ci.replace(
                '          - { distribution: rpptx, crate: rpptx-py, oracle: "python-pptx==1.0.2" }\n',
                "",
                1,
            ),
            "cancel-other-package-on-failure": ci.replace(
                "fail-fast: false", "fail-fast: true", 1
            ),
            "unversioned-pytest": ci.replace("pytest==9.1.1", "pytest", 1),
            "wrong-python-version": ci.replace(
                'python-version: "3.12.9"', 'python-version: "3.13"', 1
            ),
            "wrong-rdocx-oracle": ci.replace(
                "python-docx==1.2.0", "python-docx==1.1.2", 1
            ),
            "missing-develop": ci.replace(
                "maturin develop --locked", "maturin --version", 1
            ),
            "single-test-file": ci.replace(
                '"crates/${{ matrix.package.crate }}/tests"',
                '"crates/${{ matrix.package.crate }}/tests/test_core.py"',
                1,
            ),
            "continue-on-error": ci.replace(
                "      - name: Run full Python binding suite\n",
                "      - name: Run full Python binding suite\n        continue-on-error: true\n",
                1,
            ),
            "continue-on-error-false": ci.replace(
                "      - name: Run full Python binding suite\n",
                "      - name: Run full Python binding suite\n"
                "        continue-on-error: false\n",
                1,
            ),
            "pytest-if-false": ci.replace(
                "      - name: Run full Python binding suite\n",
                "      - name: Run full Python binding suite\n"
                "        if: false\n",
                1,
            ),
            "pytest-if-true": ci.replace(
                "      - name: Run full Python binding suite\n",
                "      - name: Run full Python binding suite\n"
                "        if: true\n",
                1,
            ),
            "pytest-step-environment": ci.replace(
                "      - name: Run full Python binding suite\n",
                "      - name: Run full Python binding suite\n"
                "        env:\n"
                "          PYTEST_ADDOPTS: --collect-only\n",
                1,
            ),
            "successful-pytest-fallback": ci.replace(
                '"$binding_python" -m pytest "crates/${{ matrix.package.crate }}/tests"',
                '"$binding_python" -m pytest "crates/${{ matrix.package.crate }}/tests" || true',
                1,
            ),
            "wrong-checkout-sha": ci.replace(
                "de0fac2e4500dabe0009e67214ff5f5447ce83dd",
                "0000000000000000000000000000000000000000",
                1,
            ),
            "wrong-checkout-sha-with-required-comment": ci.replace(
                "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2",
                "actions/checkout@0000000000000000000000000000000000000000 "
                "# v6.0.2 de0fac2e4500dabe0009e67214ff5f5447ce83dd",
                1,
            ),
            "checkout-ref-input": ci.replace(
                "      - uses: actions/checkout@"
                "de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2\n",
                "      - uses: actions/checkout@"
                "de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2\n"
                "        with:\n"
                "          ref: main\n",
                1,
            ),
            "wrong-rust-toolchain-sha": ci.replace(
                "4360b52568e2003a75bf9bc1d59f33a8e3fc893c",
                "0000000000000000000000000000000000000000",
                1,
            ),
            "rust-toolchain-input": ci.replace(
                "      - uses: dtolnay/rust-toolchain@"
                "4360b52568e2003a75bf9bc1d59f33a8e3fc893c # stable\n",
                "      - uses: dtolnay/rust-toolchain@"
                "4360b52568e2003a75bf9bc1d59f33a8e3fc893c # stable\n"
                "        with:\n"
                "          toolchain: nightly\n",
                1,
            ),
            "wrong-rust-cache-sha": ci.replace(
                "c19371144df3bb44fab255c43d04cbc2ab54d1c4",
                "0000000000000000000000000000000000000000",
                1,
            ),
            "rust-cache-input": ci.replace(
                "      - uses: Swatinem/rust-cache@"
                "c19371144df3bb44fab255c43d04cbc2ab54d1c4 # v2.9.1\n",
                "      - uses: Swatinem/rust-cache@"
                "c19371144df3bb44fab255c43d04cbc2ab54d1c4 # v2.9.1\n"
                "        with:\n"
                "          workspaces: crates/rdocx-py\n",
                1,
            ),
            "wrong-setup-python-sha": ci.replace(
                "a309ff8b426b58ec0e2a45f0f869d46889d02405",
                "0000000000000000000000000000000000000000",
                1,
            ),
            "wrong-setup-python-sha-with-required-comment": ci.replace(
                "actions/setup-python@"
                "a309ff8b426b58ec0e2a45f0f869d46889d02405 # v6.2.0",
                "actions/setup-python@"
                "0000000000000000000000000000000000000000 "
                "# v6.2.0 a309ff8b426b58ec0e2a45f0f869d46889d02405",
                1,
            ),
            "setup-python-extra-input": ci.replace(
                '          python-version: "3.12.9"\n',
                '          python-version: "3.12.9"\n'
                '          architecture: "x64"\n',
                1,
            ),
            "setup-python-comment-smuggled-version": ci.replace(
                '          python-version: "3.12.9"\n',
                '          python-version: "3.13" # python-version: "3.12.9"\n',
                1,
            ),
            "missing-rpptx-exclusion": ci.replace(
                "--exclude rdocx-py --exclude rpptx-py",
                "--exclude rdocx-py",
                1,
            ),
        }
        for name, mutated in mutations.items():
            self.assertNotEqual(mutated, ci, name)
            with self.subTest(name=name), self.assertRaises(AssertionError):
                self.assert_python_pr_job_contract(mutated)

    def assert_wasm_pr_job_contract(self, ci: str) -> None:
        triggers = self.yaml_block(ci, "on:")
        trigger_keys = tuple(
            line.split(":", 1)[0]
            for line in self.yaml_direct_lines(triggers, 2)
        )
        self.assertEqual(trigger_keys, ("push", "pull_request", "schedule"))
        pull_request = self.yaml_block(triggers, "  pull_request:")
        self.assertEqual(self.yaml_direct_lines(pull_request, 4), ())

        root_permissions = self.yaml_block(ci, "permissions:")
        self.assertEqual(
            self.yaml_direct_lines(root_permissions, 2),
            ("contents: read",),
        )
        operative_ci = self.operative_lines(ci)
        self.assertFalse(any("id-token:" in line for line in operative_ci))
        self.assertFalse(any("write-all" in line for line in operative_ci))

        job = self.yaml_block(ci, "  wasm:")
        self.assertEqual(
            self.yaml_direct_lines(job, 4),
            ("name: WASM", "runs-on: ubuntu-latest", "steps:"),
        )
        self.assertFalse(
            any("continue-on-error:" in line for line in self.operative_lines(job))
        )

        steps = self.yaml_steps(job)
        identities = tuple(
            self.yaml_step_identity(step, position)
            for position, step in enumerate(steps)
        )
        self.assertEqual(
            identities,
            (
                "step:0",
                "step:1",
                "step:2",
                "Set up Node 24.11.1",
                "Install wasm-pack 0.15.0",
                "Install wasm-opt 125",
                "Check WASM targets",
                "Run WASM Node tests",
                "Build and install local WASM packages",
            ),
        )

        action_contract = (
            (
                steps[0],
                "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd",
            ),
            (
                steps[1],
                "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c",
            ),
            (
                steps[2],
                "Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4",
            ),
        )
        for action_step, expected_action in action_contract:
            self.assertEqual(self.yaml_step_actions(action_step), (expected_action,))

        rust_inputs = self.yaml_block(steps[1], "        with:")
        self.assertEqual(
            self.yaml_direct_lines(rust_inputs, 10),
            ("targets: wasm32-unknown-unknown",),
        )
        self.assertEqual(self.yaml_direct_lines(steps[0], 8), ())
        self.assertEqual(self.yaml_direct_lines(steps[2], 8), ())

        node = self.yaml_step(job, "Set up Node 24.11.1")
        self.assertEqual(
            self.yaml_step_actions(node),
            (
                "actions/setup-node@"
                "249970729cb0ef3589644e2896645e5dc5ba9c38",
            ),
        )
        self.assertEqual(
            self.yaml_direct_lines(node, 8),
            (
                "uses: actions/setup-node@"
                "249970729cb0ef3589644e2896645e5dc5ba9c38",
                "with:",
            ),
        )
        node_inputs = self.yaml_block(node, "        with:")
        self.assertEqual(
            self.yaml_direct_lines(node_inputs, 10),
            ('node-version: "24.11.1"',),
        )

        install = self.yaml_step(job, "Install wasm-pack 0.15.0")
        install_optimizer = self.yaml_step(job, "Install wasm-opt 125")
        checks = self.yaml_step(job, "Check WASM targets")
        node_tests = self.yaml_step(job, "Run WASM Node tests")
        packages = self.yaml_step(job, "Build and install local WASM packages")
        for command_step in (install, install_optimizer, checks, node_tests, packages):
            self.assertEqual(
                self.yaml_direct_lines(command_step, 8),
                ("shell: bash", "run: |"),
            )
        install_lines = self.yaml_run_lines(install)
        optimizer_lines = self.yaml_run_lines(install_optimizer)
        check_lines = self.yaml_run_lines(checks)
        node_test_lines = self.yaml_run_lines(node_tests)
        self.assertEqual(
            install_lines,
            ("cargo install wasm-pack --version 0.15.0 --locked",),
        )
        self.assertEqual(
            optimizer_lines,
            (
                'binaryen_archive="${RUNNER_TEMP}/binaryen-version_125-x86_64-linux.tar.gz"',
                'binaryen_root="${RUNNER_TEMP}/binaryen-version_125"',
                "curl --fail --location --silent --show-error "
                '"https://github.com/WebAssembly/binaryen/releases/download/'
                'version_125/binaryen-version_125-x86_64-linux.tar.gz" '
                '--output "$binaryen_archive"',
                'echo "7c3bc16599c8274a04d34a504fe4be2047884f900e0e2da2f6fb9cd667183be4  '
                '$binaryen_archive" | sha256sum --check',
                'mkdir -p "$binaryen_root"',
                'tar --extract --gzip --file "$binaryen_archive" --directory '
                '"$binaryen_root" --strip-components=1',
                'echo "$binaryen_root/bin" >> "$GITHUB_PATH"',
                '"$binaryen_root/bin/wasm-opt" --version | grep --fixed-strings '
                '--line-regexp "wasm-opt version 125"',
            ),
        )
        self.assertEqual(
            check_lines,
            (
                "cargo check --locked --target wasm32-unknown-unknown -p rdocx-wasm",
                "cargo check --locked --target wasm32-unknown-unknown -p rpptx-wasm",
            ),
        )
        self.assertEqual(
            node_test_lines,
            (
                "wasm-pack test --node crates/rdocx-wasm",
                "wasm-pack test --node crates/rpptx-wasm",
            ),
        )
        self.assert_no_success_short_circuit(
            install_lines + optimizer_lines + check_lines + node_test_lines
        )
        package_lines = self.yaml_run_lines(packages)
        for expected in (
            'package_root="${RUNNER_TEMP}/wasm-packages"',
            'tarball_root="${RUNNER_TEMP}/wasm-tarballs"',
            'npm_cache="${RUNNER_TEMP}/npm-cache"',
            "wasm-pack build --target bundler --scope tensorbee --release "
            '--out-dir "$package_root/rdocx-wasm" crates/rdocx-wasm --locked',
            "wasm-pack build --target bundler --scope tensorbee --release "
            '--out-dir "$package_root/rpptx-wasm" crates/rpptx-wasm --locked',
            'verify_package "$package_root/rdocx-wasm" "@tensorbee/rdocx-wasm" '
            '"0.5.0" "rdocx_wasm"',
            'verify_package "$package_root/rpptx-wasm" "@tensorbee/rpptx-wasm" '
            '"0.1.3" "rpptx_wasm"',
            "npm install --prefix \"$consumer_root\" --cache \"$npm_cache\" "
            "--ignore-scripts --no-audit --no-fund --package-lock=false "
            '"$tarball_root/$tarball"',
        ):
            self.assertEqual(package_lines.count(expected), 1, expected)
        self.assertIn(
            'npm pack "$package_dir" --cache "$npm_cache" --ignore-scripts '
            '--pack-destination "$tarball_root"',
            packages,
        )
        self.assertIn('import(\\"$expected_name\\")', packages)
        self.assertIn('consumer_root="$(mktemp -d ', packages)
        self.assertIn('manifest.name !== expectedName', packages)
        self.assertIn('manifest.version !== expectedVersion', packages)
        self.assertIn('${stem}_bg.wasm', packages)
        self.assertIn('${stem}.js', packages)
        self.assertIn('${stem}.d.ts', packages)
        forbidden = (
            "npm publish",
            "npm login",
            "npm adduser",
            "npm token",
            "wasm-pack publish",
            "NODE_AUTH_TOKEN",
            "NPM_TOKEN",
            "--registry",
            "id-token:",
            "git tag",
            "gh release",
        )
        operative_job = "\n".join(self.operative_lines(job))
        for command in forbidden:
            self.assertNotIn(command, operative_job)
        self.assert_no_success_short_circuit(package_lines)
        self.assertNotIn("|| true", job)
        self.assertNotIn("set +e", job)

    def assert_wasm_optimizer_metadata_contract(
        self, manifest_overrides: dict[str, str] | None = None
    ) -> None:
        manifest_overrides = manifest_overrides or {}
        expected = {
            "wasm-opt": [
                "-Oz",
                "--enable-bulk-memory",
                "--enable-nontrapping-float-to-int",
            ]
        }
        for member in ("crates/rdocx-wasm", "crates/rpptx-wasm"):
            manifest = tomllib.loads(
                manifest_overrides.get(
                    member,
                    (workflow.REPO / member / "Cargo.toml").read_text(
                        encoding="utf-8"
                    ),
                )
            )
            wasm_pack = manifest["package"].get("metadata", {}).get("wasm-pack", {})
            release = wasm_pack.get("profile", {}).get("release")
            self.assertEqual(
                release,
                expected,
                member,
            )

    def test_wasm_pr_job_checks_both_targets_and_runs_node_tests(self) -> None:
        ci = (workflow.REPO / ".github/workflows/ci.yml").read_text(
            encoding="utf-8"
        )
        self.assert_wasm_pr_job_contract(ci)

    def test_wasm_packages_use_the_reviewed_release_optimizer(self) -> None:
        self.assert_wasm_optimizer_metadata_contract()

    def test_wasm_package_contract_rejects_optimizer_mutations(self) -> None:
        for member in ("crates/rdocx-wasm", "crates/rpptx-wasm"):
            manifest = (workflow.REPO / member / "Cargo.toml").read_text(
                encoding="utf-8"
            )
            mutations = {
                "missing-bulk-memory": manifest.replace(
                    '["-Oz", "--enable-bulk-memory", '
                    '"--enable-nontrapping-float-to-int"]',
                    '["-Oz", "--enable-nontrapping-float-to-int"]',
                    1,
                ),
                "missing-nontrapping-float-to-int": manifest.replace(
                    '["-Oz", "--enable-bulk-memory", '
                    '"--enable-nontrapping-float-to-int"]',
                    '["-Oz", "--enable-bulk-memory"]',
                    1,
                ),
                "wrong-size-level": manifest.replace(
                    '["-Oz", "--enable-bulk-memory", '
                    '"--enable-nontrapping-float-to-int"]',
                    '["-Os", "--enable-bulk-memory", '
                    '"--enable-nontrapping-float-to-int"]',
                    1,
                ),
            }
            for name, mutated in mutations.items():
                self.assertNotEqual(mutated, manifest, f"{member}:{name}")
                with self.subTest(member=member, name=name), self.assertRaises(
                    AssertionError
                ):
                    self.assert_wasm_optimizer_metadata_contract({member: mutated})

    def assert_wasm_setup_node_provenance_contract(
        self, ci: str, testing_hld: str
    ) -> None:
        reviewed_sha = "249970729cb0ef3589644e2896645e5dc5ba9c38"
        reviewed_tag = "v6.5.0"
        job = self.yaml_block(ci, "  wasm:")
        provenance_line = (
            f"        uses: actions/setup-node@{reviewed_sha} # {reviewed_tag}"
        )
        self.assertEqual(job.count(provenance_line), 1)
        self.assertIn(f"setup-node {reviewed_tag}", testing_hld)
        self.assertNotIn("setup-node v6.1.0", testing_hld)

    def test_wasm_setup_node_provenance_matches_the_testing_hld(self) -> None:
        ci = (workflow.REPO / ".github/workflows/ci.yml").read_text(
            encoding="utf-8"
        )
        testing_hld = (workflow.REPO / "docs/hld/12-testing-strategy.md").read_text(
            encoding="utf-8"
        )
        self.assert_wasm_setup_node_provenance_contract(ci, testing_hld)

        mutations = {
            "stale-workflow-comment": (
                ci.replace(
                    "249970729cb0ef3589644e2896645e5dc5ba9c38 # v6.5.0",
                    "249970729cb0ef3589644e2896645e5dc5ba9c38 # v6.1.0",
                    1,
                ),
                testing_hld,
            ),
            "stale-hld-label": (
                ci,
                testing_hld.replace("setup-node v6.5.0", "setup-node v6.1.0", 1),
            ),
        }
        for name, (mutated_ci, mutated_hld) in mutations.items():
            self.assertTrue(
                mutated_ci != ci or mutated_hld != testing_hld,
                name,
            )
            with self.subTest(name=name), self.assertRaises(AssertionError):
                self.assert_wasm_setup_node_provenance_contract(
                    mutated_ci, mutated_hld
                )

    def test_wasm_pr_job_rejects_skipped_or_weakened_gates(self) -> None:
        ci = (workflow.REPO / ".github/workflows/ci.yml").read_text(
            encoding="utf-8"
        )
        self.assert_wasm_pr_job_contract(ci)
        wasm_job = self.yaml_block(ci, "  wasm:")

        def mutate_job(old: str, new: str) -> str:
            self.assertIn(old, wasm_job)
            return ci.replace(wasm_job, wasm_job.replace(old, new, 1), 1)

        mutations = {
            "missing-pull-request-trigger": ci.replace(
                "  pull_request:\n", "", 1
            ),
            "commented-pull-request-trigger": ci.replace(
                "  pull_request:\n", "  # pull_request:\n", 1
            ),
            "root-contents-write": ci.replace(
                "  contents: read\n", "  contents: write\n", 1
            ),
            "root-id-token-write": ci.replace(
                "  contents: read\n",
                "  contents: read\n  id-token: write\n",
                1,
            ),
            "job-condition": mutate_job(
                "    name: WASM\n", "    name: WASM\n    if: true\n"
            ),
            "wrong-checkout-sha": mutate_job(
                "de0fac2e4500dabe0009e67214ff5f5447ce83dd",
                "0000000000000000000000000000000000000000",
            ),
            "wrong-rust-toolchain-sha": mutate_job(
                "4360b52568e2003a75bf9bc1d59f33a8e3fc893c",
                "0000000000000000000000000000000000000000",
            ),
            "wrong-rust-cache-sha": mutate_job(
                "c19371144df3bb44fab255c43d04cbc2ab54d1c4",
                "0000000000000000000000000000000000000000",
            ),
            "wrong-setup-node-sha": mutate_job(
                "249970729cb0ef3589644e2896645e5dc5ba9c38",
                "0000000000000000000000000000000000000000",
            ),
            "wrong-node-version": mutate_job("24.11.1", "24"),
            "unlocked-wasm-pack-install": mutate_job(
                "cargo install wasm-pack --version 0.15.0 --locked",
                "cargo install wasm-pack --version 0.15.0",
            ),
            "floating-wasm-pack-version": mutate_job(
                "cargo install wasm-pack --version 0.15.0 --locked",
                "cargo install wasm-pack --locked",
            ),
            "wrong-wasm-opt-version": mutate_job(
                "binaryen-version_125-x86_64-linux.tar.gz",
                "binaryen-version_124-x86_64-linux.tar.gz",
            ),
            "wrong-wasm-opt-checksum": mutate_job(
                "7c3bc16599c8274a04d34a504fe4be2047884f900e0e2da2f6fb9cd667183be4",
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            "missing-wasm-opt-version-check": mutate_job(
                "          \"$binaryen_root/bin/wasm-opt\" --version | grep "
                "--fixed-strings --line-regexp \"wasm-opt version 125\"\n",
                "",
            ),
            "unlocked-target-check": mutate_job(
                "cargo check --locked --target wasm32-unknown-unknown -p rdocx-wasm",
                "cargo check --target wasm32-unknown-unknown -p rdocx-wasm",
            ),
            "missing-rdocx-target-check": mutate_job(
                "          cargo check --locked --target wasm32-unknown-unknown -p rdocx-wasm\n",
                "",
            ),
            "missing-rpptx-target-check": mutate_job(
                "          cargo check --locked --target wasm32-unknown-unknown -p rpptx-wasm\n",
                "",
            ),
            "missing-rdocx-node-test": mutate_job(
                "          wasm-pack test --node crates/rdocx-wasm\n", ""
            ),
            "missing-rpptx-node-test": mutate_job(
                "          wasm-pack test --node crates/rpptx-wasm\n", ""
            ),
            "missing-node-runner": mutate_job(
                "wasm-pack test --node crates/rdocx-wasm",
                "wasm-pack test crates/rdocx-wasm",
            ),
            "listing-only-node-test": mutate_job(
                "wasm-pack test --node crates/rdocx-wasm",
                "wasm-pack test --node crates/rdocx-wasm -- --list",
            ),
            "check-condition": mutate_job(
                "      - name: Check WASM targets\n",
                "      - name: Check WASM targets\n        if: true\n",
            ),
            "node-test-condition": mutate_job(
                "      - name: Run WASM Node tests\n",
                "      - name: Run WASM Node tests\n        if: true\n",
            ),
            "continue-on-error": mutate_job(
                "      - name: Run WASM Node tests\n",
                "      - name: Run WASM Node tests\n"
                "        continue-on-error: true\n",
            ),
            "successful-fallback": mutate_job(
                "wasm-pack test --node crates/rdocx-wasm",
                "wasm-pack test --node crates/rdocx-wasm || true",
            ),
            "early-success": mutate_job(
                "        run: |\n"
                "          wasm-pack test --node crates/rdocx-wasm",
                "        run: |\n"
                "          exit 0\n"
                "          wasm-pack test --node crates/rdocx-wasm",
            ),
            "missing-rdocx-package": mutate_job(
                "          wasm-pack build --target bundler --scope tensorbee "
                "--release --out-dir \"$package_root/rdocx-wasm\" "
                "crates/rdocx-wasm --locked\n",
                "",
            ),
            "wrong-package-target": mutate_job(
                "wasm-pack build --target bundler",
                "wasm-pack build --target nodejs",
            ),
            "wrong-package-scope": mutate_job(
                "--scope tensorbee --release",
                "--scope other --release",
            ),
            "unlocked-package-build": mutate_job(
                "crates/rdocx-wasm --locked",
                "crates/rdocx-wasm",
            ),
            "missing-clean-install": mutate_job(
                "          npm install --prefix \"$consumer_root\" --cache "
                "\"$npm_cache\" --ignore-scripts --no-audit --no-fund "
                "--package-lock=false \"$tarball_root/$tarball\"\n",
                "",
            ),
            "registry-authentication": mutate_job(
                "      - name: Build and install local WASM packages\n",
                "      - name: Build and install local WASM packages\n"
                "        env:\n"
                "          NPM_TOKEN: forbidden\n",
            ),
            "npm-publish-authority": mutate_job(
                '          assert_inventory "$package_dir" "$expected_name" '
                '"$expected_version" "$stem"\n',
                '          assert_inventory "$package_dir" "$expected_name" '
                '"$expected_version" "$stem"\n'
                '          npm publish "$package_dir"\n',
            ),
            "release-tag-authority": mutate_job(
                "      - name: Build and install local WASM packages\n",
                "      - name: Build and install local WASM packages\n"
                "        env:\n"
                "          RELEASE_COMMAND: git tag v0.0.0\n",
            ),
        }
        for name, mutated in mutations.items():
            self.assertNotEqual(mutated, ci, name)
            with self.subTest(name=name), self.assertRaises(AssertionError):
                self.assert_wasm_pr_job_contract(mutated)

    def assert_wheels_workflow_contract(self, workflow_bytes: bytes) -> None:
        self.assertEqual(
            hashlib.sha256(workflow_bytes).hexdigest(),
            "56491248b4ffa7ea40abe75b04a16fcfd5c24744d16ccb9a8c6f7110d39be35a",
        )
        wheels = workflow_bytes.decode("utf-8", errors="strict")
        expected_packages = (
            ("rdocx", "rdocx-py", "rdocx"),
            ("rpptx", "rpptx-py", "rpptx"),
        )
        expected_platforms = (
            (
                "manylinux_2_28-x86_64",
                "ubuntu-24.04",
                "x86_64-unknown-linux-gnu",
                "2_28",
                "native",
            ),
            (
                "manylinux_2_28-aarch64",
                "ubuntu-24.04-arm",
                "aarch64-unknown-linux-gnu",
                "2_28",
                "native",
            ),
            (
                "musllinux_1_2-x86_64",
                "ubuntu-24.04",
                "x86_64-unknown-linux-musl",
                "musllinux_1_2",
                "musl",
            ),
            (
                "macos-x86_64",
                "macos-15-intel",
                "x86_64-apple-darwin",
                "off",
                "native",
            ),
            (
                "macos-arm64",
                "macos-14",
                "aarch64-apple-darwin",
                "off",
                "native",
            ),
            (
                "windows-x86_64",
                "windows-2025",
                "x86_64-pc-windows-msvc",
                "off",
                "native",
            ),
        )
        triggers = self.yaml_block(wheels, "on:")
        trigger_keys = tuple(
            line.split(":", 1)[0]
            for line in self.yaml_direct_lines(triggers, 2)
        )
        self.assertEqual(trigger_keys, ("push", "workflow_dispatch"))
        push_trigger = self.yaml_block(triggers, "  push:")
        self.assertEqual(
            self.yaml_direct_lines(push_trigger, 4),
            ('tags: ["py-v*"]',),
        )
        root_permissions = self.yaml_block(wheels, "permissions:")
        self.assertEqual(
            self.yaml_direct_lines(root_permissions, 2),
            ("contents: read",),
        )
        self.assertNotIn("secrets.", wheels)
        self.assertNotIn("git tag", wheels)
        self.assertNotIn("git push", wheels)
        self.assertNotIn("write-all", wheels)
        self.assertNotIn("continue-on-error:", wheels)

        jobs = self.yaml_block(wheels, "jobs:")
        job_keys = tuple(
            line.split(":", 1)[0]
            for line in self.yaml_direct_lines(jobs, 2)
        )
        self.assertEqual(job_keys, ("build-wheels", "build-sdists", "publish"))
        build_wheels = self.yaml_block(wheels, "  build-wheels:")
        build_sdists = self.yaml_block(wheels, "  build-sdists:")
        publish = self.yaml_block(wheels, "  publish:")
        job_names = {
            key: tuple(
                line.removeprefix("name: ")
                for line in self.yaml_direct_lines(job, 4)
                if line.startswith("name: ")
            )
            for key, job in (
                ("build-wheels", build_wheels),
                ("build-sdists", build_sdists),
                ("publish", publish),
            )
        }
        self.assertEqual(
            job_names,
            {
                "build-wheels": (
                    "${{ matrix.package.distribution }} "
                    "${{ matrix.platform.label }}",
                ),
                "build-sdists": ("${{ matrix.package.distribution }} sdist",),
                "publish": ("Publish Python distributions",),
            },
        )
        publication_jobs = tuple(
            key
            for key, names in job_names.items()
            if "publish" in key.lower()
            or any("publish" in name.lower() for name in names)
        )
        self.assertEqual(publication_jobs, ("publish",))

        for build_job in (build_wheels, build_sdists):
            direct = self.yaml_direct_lines(build_job, 4)
            self.assertFalse(any(line.startswith("if:") for line in direct))
            self.assertFalse(any(line.startswith("permissions:") for line in direct))

        cp39_setup = self.yaml_block(build_wheels, "      - id: cp39")
        cp312_setup = self.yaml_block(build_wheels, "      - id: cp312")
        sdist_steps = self.yaml_steps(build_sdists)
        self.assertGreaterEqual(len(sdist_steps), 2)
        sdist_setup = sdist_steps[1]
        self.assertEqual(self.yaml_step_identity(sdist_setup, 2), "step:2")
        for setup, version, condition in (
            (cp39_setup, '"3.9"', ()),
            (
                cp312_setup,
                '"3.12"',
                ("if: matrix.platform.install == 'native'",),
            ),
            (sdist_setup, '"3.9"', ()),
        ):
            setup_conditions = tuple(
                line
                for line in self.yaml_direct_lines(setup, 8)
                if line.startswith("if:")
            )
            self.assertEqual(setup_conditions, condition)
            setup_inputs = self.yaml_block(setup, "        with:")
            self.assertEqual(
                self.yaml_direct_lines(setup_inputs, 10),
                (f"python-version: {version}",),
            )

        matrix = self.yaml_block(build_wheels, "      matrix:")
        matrix_axes = tuple(
            line.strip().split(":", 1)[0]
            for line in matrix.splitlines()[1:]
            if line.strip() and len(line) - len(line.lstrip()) == 8
        )
        self.assertEqual(matrix_axes, ("package", "platform"))

        package_matrix = self.yaml_block(matrix, "        package:")
        package_entries = tuple(
            line.strip()
            for line in package_matrix.splitlines()[1:]
            if len(line) - len(line.lstrip()) == 10
        )
        expected_package_entries = tuple(
            (
                f"- {{ distribution: {distribution}, crate: {crate}, "
                f"module: {module} }}"
            )
            for distribution, crate, module in expected_packages
        )
        self.assertEqual(package_entries, expected_package_entries)

        platform_matrix = self.yaml_block(matrix, "        platform:")
        platform_entries = tuple(
            line.strip()
            for line in platform_matrix.splitlines()[1:]
            if len(line) - len(line.lstrip()) == 10
        )
        expected_platform_entries = tuple(
            (
                f"- {{ label: {label}, runner: {runner}, target: {target}, "
                f"manylinux: {manylinux}, install: {install} }}"
            )
            for label, runner, target, manylinux, install in expected_platforms
        )
        self.assertEqual(platform_entries, expected_platform_entries)

        wheel_build = self.yaml_step(build_wheels, "Build cp39-abi3 wheel")
        self.assertFalse(
            any(
                line.startswith("if:")
                for line in self.yaml_direct_lines(wheel_build, 8)
            )
        )
        wheel_build_inputs = self.yaml_block(wheel_build, "        with:")
        self.assertEqual(
            self.yaml_direct_lines(wheel_build_inputs, 10),
            (
                "command: build",
                "maturin-version: v1.13.3",
                "target: ${{ matrix.platform.target }}",
                "manylinux: ${{ matrix.platform.manylinux }}",
                "working-directory: crates/${{ matrix.package.crate }}",
                "args: --release --locked --compatibility pypi --out ../../dist",
            ),
        )

        wheel_metadata = self.yaml_step(build_wheels, "Validate wheel metadata")
        native_install = self.yaml_step(build_wheels, "Install and test native wheel")
        typing = self.yaml_step(build_wheels, "Validate installed typing surface")
        for step in (native_install, typing):
            conditions = tuple(
                line.strip()
                for line in step.splitlines()
                if len(line) - len(line.lstrip()) == 8
                and line.strip().startswith("if:")
            )
            self.assertEqual(
                conditions,
                ("if: matrix.platform.install == 'native'",),
            )
        musllinux_install = self.yaml_step(
            build_wheels, "Install and test musllinux wheel"
        )
        musllinux_conditions = tuple(
            line
            for line in self.yaml_direct_lines(musllinux_install, 8)
            if line.startswith("if:")
        )
        self.assertEqual(
            musllinux_conditions,
            ("if: matrix.platform.install == 'musl'",),
        )

        metadata_run = self.yaml_run_lines(wheel_metadata)
        native_run = self.yaml_run_lines(native_install)
        typing_run = self.yaml_run_lines(typing)
        musllinux_run = self.yaml_run_lines(musllinux_install)
        for run in (metadata_run, native_run, typing_run, musllinux_run):
            self.assert_no_success_short_circuit(run)
        self.assertEqual(
            metadata_run,
            (
                "python - <<'PY'",
                "from pathlib import Path",
                "import zipfile",
                'wheel = next(Path("dist").glob('
                '"${{ matrix.package.distribution }}-*.whl"))',
                "with zipfile.ZipFile(wheel) as archive:",
                "wheel_metadata = next(",
                "name for name in archive.namelist() "
                'if name.endswith(".dist-info/WHEEL")',
                ")",
                "metadata = archive.read(wheel_metadata).decode()",
                'assert "Tag: cp39-abi3-" in metadata, metadata',
                "PY",
            ),
        )
        self.assertEqual(
            native_run,
            (
                '"${{ steps.cp39.outputs.python-path }}" -m venv .wheel-venv',
                'if [[ "${{ runner.os }}" == "Windows" ]]; then',
                "venv_python=.wheel-venv/Scripts/python",
                "else",
                "venv_python=.wheel-venv/bin/python",
                "fi",
                '"$venv_python" -m pip install --upgrade pip',
                '"$venv_python" -m pip install '
                'dist/${{ matrix.package.distribution }}-*.whl pytest '
                "python-docx==1.2.0 python-pptx==1.0.2",
                '"$venv_python" -c "import ${{ matrix.package.module }}"',
                'if [[ "${{ matrix.package.distribution }}" == "rdocx" ]]; then',
                '"$venv_python" -m pytest \\',
                "crates/rdocx-py/tests/test_core.py " + chr(92),
                "crates/rdocx-py/tests/test_formatting_tables.py " + chr(92),
                "crates/rdocx-py/tests/test_shared.py " + chr(92),
                "crates/rdocx-py/tests/test_python_docx_parity.py",
                "else",
                '"$venv_python" -m pytest '
                "crates/rpptx-py/tests/test_documented_examples.py",
                "fi",
            ),
        )
        self.assertEqual(
            typing_run,
            (
                '"${{ steps.cp312.outputs.python-path }}" -m venv .typing-venv',
                'if [[ "${{ runner.os }}" == "Windows" ]]; then',
                "typing_python=.typing-venv/Scripts/python",
                "else",
                "typing_python=.typing-venv/bin/python",
                "fi",
                '"$typing_python" -m pip install --upgrade pip',
                '"$typing_python" -m pip install '
                'dist/${{ matrix.package.distribution }}-*.whl mypy==2.3.0',
                '"$typing_python" -m mypy --strict \\',
                "crates/${{ matrix.package.crate }}/tests/typing_smoke.py "
                + chr(92),
                "crates/${{ matrix.package.crate }}/python/"
                "${{ matrix.package.module }}",
                'if [[ "${{ matrix.package.distribution }}" == "rdocx" ]]; then',
                '"$typing_python" -m mypy.stubtest rdocx',
                "else",
                '"$typing_python" -m mypy.stubtest rpptx',
                "fi",
            ),
        )
        self.assertEqual(
            musllinux_run,
            (
                "docker run --rm " + chr(92),
                '-v "$PWD:/workspace:ro" ' + chr(92),
                '-v "$PWD/dist:/dist:ro" ' + chr(92),
                "-w /workspace " + chr(92),
                '-e PACKAGE_DISTRIBUTION="${{ matrix.package.distribution }}" '
                + chr(92),
                '-e PACKAGE_MODULE="${{ matrix.package.module }}" ' + chr(92),
                "python:3.9-alpine " + chr(92),
                "sh -euxc '",
                "python -m venv /tmp/wheel-venv",
                "venv_python=/tmp/wheel-venv/bin/python",
                '"$venv_python" -m pip install --upgrade pip',
                '"$venv_python" -m pip install '
                "/dist/${PACKAGE_DISTRIBUTION}-*.whl pytest "
                "python-docx==1.2.0 python-pptx==1.0.2",
                '"$venv_python" -c "import ${PACKAGE_MODULE}"',
                'if [ "$PACKAGE_DISTRIBUTION" = rdocx ]; then',
                '"$venv_python" -m pytest ' + chr(92),
                "crates/rdocx-py/tests/test_core.py " + chr(92),
                "crates/rdocx-py/tests/test_formatting_tables.py " + chr(92),
                "crates/rdocx-py/tests/test_shared.py " + chr(92),
                "crates/rdocx-py/tests/test_python_docx_parity.py",
                "else",
                '"$venv_python" -m pytest '
                "crates/rpptx-py/tests/test_documented_examples.py",
                "fi",
                "'",
            ),
        )

        distribution_branch = (
            'if [[ "${{ matrix.package.distribution }}" == "rdocx" ]]; then'
        )
        self.assertEqual(native_install.count(distribution_branch), 1)
        self.assertIn("crates/rdocx-py/tests/test_python_docx_parity.py", native_install)
        self.assertIn(
            "crates/rpptx-py/tests/test_documented_examples.py", native_install
        )
        self.assertIn(
            'if [ "$PACKAGE_DISTRIBUTION" = rdocx ]; then', musllinux_install
        )
        self.assertIn(
            "crates/rdocx-py/tests/test_python_docx_parity.py",
            musllinux_install,
        )
        self.assertIn(
            "crates/rpptx-py/tests/test_documented_examples.py",
            musllinux_install,
        )
        self.assertEqual(typing.count(distribution_branch), 1)
        self.assertIn('"$typing_python" -m mypy.stubtest rdocx\n', typing)
        self.assertIn('"$typing_python" -m mypy.stubtest rpptx\n', typing)

        upload_wheel = self.yaml_step(build_wheels, "Upload wheel")
        self.assertFalse(
            any(
                line.startswith("if:")
                for line in self.yaml_direct_lines(upload_wheel, 8)
            )
        )
        self.assertIn(
            "uses: actions/upload-artifact@"
            "ea165f8d65b6e75b540449e92b4886f43607fa02",
            upload_wheel,
        )
        upload_wheel_inputs = self.yaml_block(upload_wheel, "        with:")
        self.assertEqual(
            self.yaml_direct_lines(upload_wheel_inputs, 10),
            (
                "name: artifact-wheel-${{ matrix.package.distribution }}-"
                "${{ matrix.platform.label }}",
                "path: dist/*.whl",
                "if-no-files-found: error",
            ),
        )

        self.assertIn("Tag: cp39-abi3-", wheels)
        self.assertIn('python-version: "3.9"', wheels)
        self.assertIn('python-version: "3.12"', wheels)
        self.assertIn('-m venv .wheel-venv', wheels)
        self.assertIn('-m venv .typing-venv', wheels)
        self.assertIn("python-docx==1.2.0", wheels)
        self.assertIn("python-pptx==1.0.2", wheels)
        self.assertIn("test_python_docx_parity.py", wheels)
        self.assertIn("test_documented_examples.py", wheels)
        self.assertIn('"$typing_python" -m mypy --strict', wheels)
        self.assertIn('"$typing_python" -m mypy.stubtest rdocx\n', wheels)
        self.assertIn('"$typing_python" -m mypy.stubtest rpptx\n', wheels)
        self.assertNotIn("mypy.stubtest rdocx rdocx.", wheels)
        self.assertNotIn("mypy.stubtest rpptx rpptx.", wheels)

        sdist_matrix = self.yaml_block(build_sdists, "      matrix:")
        sdist_axes = tuple(
            line.split(":", 1)[0]
            for line in self.yaml_direct_lines(sdist_matrix, 8)
        )
        self.assertEqual(sdist_axes, ("package",))
        sdist_packages = self.yaml_block(sdist_matrix, "        package:")
        self.assertEqual(
            self.yaml_direct_lines(sdist_packages, 10), expected_package_entries
        )
        sdist_build = self.yaml_step(build_sdists, "Build source distribution")
        self.assertFalse(
            any(
                line.startswith("if:")
                for line in self.yaml_direct_lines(sdist_build, 8)
            )
        )
        sdist_build_inputs = self.yaml_block(sdist_build, "        with:")
        self.assertEqual(
            self.yaml_direct_lines(sdist_build_inputs, 10),
            (
                "command: sdist",
                "maturin-version: v1.13.3",
                "working-directory: crates/${{ matrix.package.crate }}",
                "args: --out ../../dist",
            ),
        )
        upload_sdist = self.yaml_step(build_sdists, "Upload source distribution")
        self.assertFalse(
            any(
                line.startswith("if:")
                for line in self.yaml_direct_lines(upload_sdist, 8)
            )
        )
        self.assertIn(
            "uses: actions/upload-artifact@"
            "ea165f8d65b6e75b540449e92b4886f43607fa02",
            upload_sdist,
        )
        upload_sdist_inputs = self.yaml_block(upload_sdist, "        with:")
        self.assertEqual(
            self.yaml_direct_lines(upload_sdist_inputs, 10),
            (
                "name: artifact-sdist-${{ matrix.package.distribution }}",
                "path: dist/*.tar.gz",
                "if-no-files-found: error",
            ),
        )
        self.assertIn("artifact-sdist-${{ matrix.package.distribution }}", wheels)
        self.assertIn("artifact-wheel-${{ matrix.package.distribution }}-", wheels)
        self.assertIn("pattern: artifact-*", wheels)
        self.assertIn("merge-multiple: true", wheels)
        self.assertIn("assert len(wheels) == 12", wheels)
        self.assertIn("assert len(sdists) == 2", wheels)

        publish_conditions = tuple(
            line.strip()
            for line in publish.splitlines()[1:]
            if len(line) - len(line.lstrip()) == 4
            and line.strip().startswith("if:")
        )
        self.assertEqual(
            publish_conditions,
            (
                "if: github.event_name == 'push' && "
                "startsWith(github.ref, 'refs/tags/py-v')",
            ),
        )
        publish_header = publish[: publish.index("    steps:\n")]
        publish_needs = tuple(
            line.removeprefix("needs: ").split(" #", 1)[0].rstrip()
            for line in self.yaml_direct_lines(publish_header, 4)
            if line.startswith("needs:")
        )
        self.assertEqual(publish_needs, ("[build-wheels, build-sdists]",))
        environments = tuple(
            line.removeprefix("environment: ")
            for line in self.yaml_direct_lines(publish_header, 4)
            if line.startswith("environment:")
        )
        self.assertEqual(environments, ("pypi",))
        publish_permissions = self.yaml_block(publish_header, "    permissions:")
        self.assertEqual(
            self.yaml_direct_lines(publish_permissions, 6),
            ("contents: read", "id-token: write"),
        )
        self.assertNotIn(
            "id-token: write", wheels[: wheels.index("  publish:\n")]
        )
        publication_validation = self.yaml_step(
            publish, "Validate complete publication set"
        )
        publication_download = self.yaml_step(
            publish, "Download all distributions"
        )
        publication_action = self.yaml_step(
            publish, "Publish to PyPI with trusted publishing"
        )
        publish_steps = self.yaml_block(publish, "    steps:")
        self.assertEqual(
            tuple(
                line
                for line in self.yaml_direct_lines(publish_steps, 6)
                if line.startswith("-")
            ),
            (
                "- name: Download all distributions",
                "- name: Validate complete publication set",
                "- name: Publish to PyPI with trusted publishing",
            ),
        )
        download_uses = tuple(
            line.removeprefix("uses: ").split(" #", 1)[0]
            for line in self.yaml_direct_lines(publication_download, 8)
            if line.startswith("uses:")
        )
        self.assertEqual(
            download_uses,
            (
                "actions/download-artifact@"
                "d3f86a106a0bac45b974a628896c90dbdf5c8093",
            ),
        )
        download_inputs = self.yaml_block(publication_download, "        with:")
        self.assertEqual(
            self.yaml_direct_lines(download_inputs, 10),
            ("path: dist", "pattern: artifact-*", "merge-multiple: true"),
        )
        validation_lines = self.yaml_run_lines(publication_validation)
        self.assert_no_success_short_circuit(validation_lines)
        self.assertEqual(
            validation_lines,
            (
                "python - <<'PY'",
                "from pathlib import Path",
                'wheels = list(Path("dist").glob("*.whl"))',
                'sdists = list(Path("dist").glob("*.tar.gz"))',
                "assert len(wheels) == 12, wheels",
                "assert len(sdists) == 2, sdists",
                "PY",
            ),
        )
        publication_uses = tuple(
            line.removeprefix("uses: ").split(" #", 1)[0]
            for line in self.yaml_direct_lines(publication_action, 8)
            if line.startswith("uses:")
        )
        self.assertEqual(
            publication_uses,
            (
                "pypa/gh-action-pypi-publish@"
                "cef221092ed1bacb1cc03d23a2d87d1d172e277b",
            ),
        )
        publication_inputs = self.yaml_block(publication_action, "        with:")
        self.assertEqual(
            self.yaml_direct_lines(publication_inputs, 10),
            ("packages-dir: dist",),
        )
        for step in (publication_validation, publication_action):
            self.assertFalse(
                any(
                    line.startswith("if:")
                    for line in self.yaml_direct_lines(step, 8)
                )
            )
        self.assertNotIn("continue-on-error:", publication_validation)
        self.assertNotIn("continue-on-error:", publication_action)

        action_uses = []
        for job_name, job in (
            ("build-wheels", build_wheels),
            ("build-sdists", build_sdists),
            ("publish", publish),
        ):
            for position, step in enumerate(self.yaml_steps(job), start=1):
                identity = self.yaml_step_identity(step, position)
                for action in self.yaml_step_actions(step):
                    action_uses.append((job_name, identity, action))
        self.assertEqual(
            tuple(action_uses),
            (
                (
                    "build-wheels",
                    "step:1",
                    "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd",
                ),
                (
                    "build-wheels",
                    "id:cp39",
                    "actions/setup-python@"
                    "a309ff8b426b58ec0e2a45f0f869d46889d02405",
                ),
                (
                    "build-wheels",
                    "Build cp39-abi3 wheel",
                    "PyO3/maturin-action@"
                    "86b9d133d34bc1b40018696f782949dac11bd380",
                ),
                (
                    "build-wheels",
                    "id:cp312",
                    "actions/setup-python@"
                    "a309ff8b426b58ec0e2a45f0f869d46889d02405",
                ),
                (
                    "build-wheels",
                    "Upload wheel",
                    "actions/upload-artifact@"
                    "ea165f8d65b6e75b540449e92b4886f43607fa02",
                ),
                (
                    "build-sdists",
                    "step:1",
                    "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd",
                ),
                (
                    "build-sdists",
                    "step:2",
                    "actions/setup-python@"
                    "a309ff8b426b58ec0e2a45f0f869d46889d02405",
                ),
                (
                    "build-sdists",
                    "Build source distribution",
                    "PyO3/maturin-action@"
                    "86b9d133d34bc1b40018696f782949dac11bd380",
                ),
                (
                    "build-sdists",
                    "Upload source distribution",
                    "actions/upload-artifact@"
                    "ea165f8d65b6e75b540449e92b4886f43607fa02",
                ),
                (
                    "publish",
                    "Download all distributions",
                    "actions/download-artifact@"
                    "d3f86a106a0bac45b974a628896c90dbdf5c8093",
                ),
                (
                    "publish",
                    "Publish to PyPI with trusted publishing",
                    "pypa/gh-action-pypi-publish@"
                    "cef221092ed1bacb1cc03d23a2d87d1d172e277b",
                ),
            ),
        )

    def test_wheels_workflow_covers_every_package_target_and_clean_install(
        self,
    ) -> None:
        workflow_bytes = (
            workflow.REPO / ".github/workflows/wheels.yml"
        ).read_bytes()
        self.assert_wheels_workflow_contract(workflow_bytes)

    def test_wheels_workflow_rejects_matrix_and_security_mutations(self) -> None:
        workflow_bytes = (
            workflow.REPO / ".github/workflows/wheels.yml"
        ).read_bytes()
        self.assert_wheels_workflow_contract(workflow_bytes)
        wheels = workflow_bytes.decode("utf-8", errors="strict")
        native_condition = "if: matrix.platform.install == 'native'"
        sdist_start = wheels.index("  build-sdists:\n")
        sdist_head = wheels[:sdist_start]
        sdist_tail = wheels[sdist_start:]
        publish_start = wheels.index("  publish:\n")
        publish_head = wheels[:publish_start]
        publish_tail = wheels[publish_start:]
        publication_download = self.yaml_step(
            publish_tail, "Download all distributions"
        )
        publication_validation = self.yaml_step(
            publish_tail, "Validate complete publication set"
        )
        publication_action = self.yaml_step(
            publish_tail, "Publish to PyPI with trusted publishing"
        )
        cp39_setup = self.yaml_block(wheels, "      - id: cp39")
        cp312_setup = self.yaml_block(wheels, "      - id: cp312")
        critical_run_steps = (
            "Validate wheel metadata",
            "Install and test native wheel",
            "Validate installed typing surface",
            "Install and test musllinux wheel",
            "Validate complete publication set",
        )
        musllinux_step = self.yaml_step(
            wheels, "Install and test musllinux wheel"
        )
        musllinux_parity_start = musllinux_step.index(
            '              if [ "$PACKAGE_DISTRIBUTION" = rdocx ]; then\n'
        )
        musllinux_parity_end = musllinux_step.index(
            "            '\n", musllinux_parity_start
        )
        musllinux_import_only_step = (
            musllinux_step[:musllinux_parity_start]
            + musllinux_step[musllinux_parity_end:]
        )
        early_success_mutations = tuple(
            (
                f"{name.lower().replace(' ', '-')}-{command.replace(' ', '-')}",
                wheels.replace(
                    step,
                    step.replace(
                        "        run: |\n",
                        f"        run: |\n          {command}\n",
                        1,
                    ),
                    1,
                ),
            )
            for name in critical_run_steps
            for command in ("exit 0", "return 0", "true")
            for step in (self.yaml_step(wheels, name),)
        )

        def mutate_run(
            name: str,
            *,
            prefix: str = "",
            suffix: str = "",
            first_line_suffix: str = "",
        ) -> str:
            step = self.yaml_step(wheels, name)
            mutated_step = step
            marker = "        run: |\n"
            if first_line_suffix:
                marker_end = mutated_step.index(marker) + len(marker)
                line_end = mutated_step.index("\n", marker_end)
                first_line = mutated_step[marker_end:line_end]
                mutated_step = (
                    mutated_step[:marker_end]
                    + first_line
                    + first_line_suffix
                    + mutated_step[line_end:]
                )
            if prefix:
                mutated_step = mutated_step.replace(
                    marker, marker + f"          {prefix}\n", 1
                )
            if suffix:
                mutated_step += f"          {suffix}\n"
            return wheels.replace(step, mutated_step, 1)

        control_flow_mutations = tuple(
            (f"{name.lower().replace(' ', '-')}-{label}", mutation)
            for name in critical_run_steps
            for label, mutation in (
                (
                    "if-false-wrapper",
                    mutate_run(name, prefix="if false; then", suffix="fi"),
                ),
                (
                    "if-true-wrapper",
                    mutate_run(name, prefix="if true; then", suffix="fi"),
                ),
                (
                    "set-plus-e-trailing-noop",
                    mutate_run(name, prefix="set +e", suffix=":"),
                ),
                (
                    "or-true",
                    mutate_run(name, first_line_suffix=" || true"),
                ),
                (
                    "or-noop",
                    mutate_run(name, first_line_suffix=" || :"),
                ),
                (
                    "semicolon-noop",
                    mutate_run(name, first_line_suffix="; :"),
                ),
                (
                    "semicolon-true",
                    mutate_run(name, first_line_suffix="; true"),
                ),
                ("trailing-noop", mutate_run(name, suffix=":")),
            )
        )
        duplicate_rdocx_sdist = sdist_head + sdist_tail.replace(
            "          - { distribution: rpptx, crate: rpptx-py, "
            "module: rpptx }",
            "          - { distribution: rdocx, crate: rdocx-py, "
            "module: rdocx }",
            1,
        )
        extra_sdist_axis = sdist_head + sdist_tail.replace(
            "      matrix:\n        package:\n",
            "      matrix:\n        python: [3.9, 3.12]\n        package:\n",
            1,
        )
        mutations = (
            (
                "missing-platform",
                wheels.replace(
                    "label: windows-x86_64", "label: windows-missing", 1
                ),
            ),
            (
                "missing-package",
                wheels.replace("distribution: rpptx", "distribution: absent", 1),
            ),
            (
                "missing-install",
                wheels.replace("-m venv .wheel-venv", "-m pip --version", 1),
            ),
            (
                "native-pytest-collect-only-environment",
                wheels.replace(
                    "      - name: Install and test native wheel\n",
                    "      - name: Install and test native wheel\n"
                    "        env:\n"
                    "          PYTEST_ADDOPTS: --collect-only\n",
                    1,
                ),
            ),
            (
                "typing-mypy-config-environment",
                wheels.replace(
                    "      - name: Validate installed typing surface\n",
                    "      - name: Validate installed typing surface\n"
                    "        env:\n"
                    "          MYPY_CONFIG_FILE: /dev/null\n",
                    1,
                ),
            ),
            (
                "wheel-checkout-ref-input",
                wheels.replace(
                    "      - uses: actions/checkout@"
                    "de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2\n",
                    "      - uses: actions/checkout@"
                    "de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2\n"
                    "        with: { ref: main }\n",
                    1,
                ),
            ),
            (
                "wheel-checkout-repository-input",
                wheels.replace(
                    "      - uses: actions/checkout@"
                    "de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2\n",
                    "      - uses: actions/checkout@"
                    "de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2\n"
                    "        with:\n"
                    "          repository: example/other\n"
                    "          ref: main\n",
                    1,
                ),
            ),
            (
                "sdist-checkout-ref-input",
                sdist_head
                + sdist_tail.replace(
                    "      - uses: actions/checkout@"
                    "de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2\n",
                    "      - uses: actions/checkout@"
                    "de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2\n"
                    "        with: { ref: main }\n",
                    1,
                ),
            ),
            (
                "sdist-checkout-repository-input",
                sdist_head
                + sdist_tail.replace(
                    "      - uses: actions/checkout@"
                    "de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2\n",
                    "      - uses: actions/checkout@"
                    "de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2\n"
                    "        with:\n"
                    "          repository: example/other\n"
                    "          ref: main\n",
                    1,
                ),
            ),
            (
                "cp39-wrong-version-with-required-comment",
                wheels.replace(
                    '          python-version: "3.9"\n',
                    '          python-version: "3.12" # '
                    'python-version: "3.9"\n',
                    1,
                ),
            ),
            (
                "cp312-wrong-version-with-required-comment",
                wheels.replace(
                    '          python-version: "3.12"\n',
                    '          python-version: "3.9" # '
                    'python-version: "3.12"\n',
                    1,
                ),
            ),
            (
                "sdist-wrong-python-version",
                sdist_head
                + sdist_tail.replace(
                    '          python-version: "3.9"\n',
                    '          python-version: "3.8"\n',
                    1,
                ),
            ),
            (
                "sdist-wrong-version-with-required-comment",
                sdist_head
                + sdist_tail.replace(
                    '          python-version: "3.9"\n',
                    '          python-version: "3.12" # '
                    'python-version: "3.9"\n',
                    1,
                ),
            ),
            (
                "renamed-cp39-id",
                wheels.replace("      - id: cp39\n", "      - id: py39\n", 1),
            ),
            (
                "renamed-cp312-id",
                wheels.replace(
                    "      - id: cp312\n", "      - id: py312\n", 1
                ),
            ),
            (
                "cp39-package-restricted",
                wheels.replace(
                    cp39_setup,
                    cp39_setup.replace(
                        "      - id: cp39\n",
                        "      - id: cp39\n"
                        "        if: matrix.package.distribution == 'rdocx'\n",
                        1,
                    ),
                    1,
                ),
            ),
            (
                "cp312-unconditional",
                wheels.replace(
                    cp312_setup,
                    cp312_setup.replace(
                        "        if: matrix.platform.install == 'native'\n",
                        "",
                        1,
                    ),
                    1,
                ),
            ),
            (
                "sdist-setup-wrong-position",
                sdist_head
                + sdist_tail.replace(
                    "      - uses: actions/setup-python@",
                    "      - name: Setup prelude\n"
                    "        run: echo setup\n"
                    "      - uses: actions/setup-python@",
                    1,
                ),
            ),
            (
                "cp39-setup-wrong-position",
                wheels.replace(cp39_setup, "", 1).replace(
                    cp312_setup, cp39_setup + cp312_setup, 1
                ),
            ),
            (
                "missing-artifact-dependency",
                wheels.replace(
                    "needs: [build-wheels, build-sdists]",
                    "needs: build-wheels",
                    1,
                ),
            ),
            (
                "missing-sdist-need-preserved-in-comment",
                wheels.replace(
                    "needs: [build-wheels, build-sdists]",
                    "needs: build-wheels # "
                    "needs: [build-wheels, build-sdists]",
                    1,
                ),
            ),
            (
                "missing-wheel-need-preserved-in-comment",
                wheels.replace(
                    "needs: [build-wheels, build-sdists]",
                    "needs: build-sdists # "
                    "needs: [build-wheels, build-sdists]",
                    1,
                ),
            ),
            (
                "extra-publish-need",
                wheels.replace(
                    "needs: [build-wheels, build-sdists]",
                    "needs: [build-wheels, build-sdists, audit]",
                    1,
                ),
            ),
            (
                "reversed-publish-needs",
                wheels.replace(
                    "needs: [build-wheels, build-sdists]",
                    "needs: [build-sdists, build-wheels]",
                    1,
                ),
            ),
            (
                "wrong-tag-prefix",
                wheels.replace(
                    "startsWith(github.ref, 'refs/tags/py-v')",
                    "startsWith(github.ref, 'refs/heads/')",
                    1,
                ),
            ),
            (
                "tag-ignore-with-required-tag-comment",
                wheels.replace(
                    '    tags: ["py-v*"]',
                    '    tags-ignore: ["py-v*"] # tags: ["py-v*"]',
                    1,
                ),
            ),
            (
                "commented-push-trigger",
                wheels.replace("  push:\n", "  # push:\n", 1),
            ),
            (
                "comment-only-tag-filter",
                wheels.replace(
                    '    tags: ["py-v*"]', '    # tags: ["py-v*"]', 1
                ),
            ),
            (
                "extra-schedule-trigger",
                wheels.replace(
                    "  workflow_dispatch:\n",
                    "  workflow_dispatch:\n  schedule: []\n",
                    1,
                ),
            ),
            (
                "commented-workflow-dispatch",
                wheels.replace(
                    "  workflow_dispatch:\n", "  # workflow_dispatch:\n", 1
                ),
            ),
            (
                "extra-matrix-axis",
                wheels.replace(
                    "        platform:\n",
                    "        python: [3.9, 3.12]\n        platform:\n",
                    1,
                ),
            ),
            (
                "rdocx-only-native-gates",
                wheels.replace(
                    native_condition,
                    native_condition
                    + " && matrix.package.distribution == 'rdocx'",
                ),
            ),
            (
                "manual-dispatch-publication",
                wheels.replace(
                    "startsWith(github.ref, 'refs/tags/py-v')",
                    "startsWith(github.ref, 'refs/tags/py-v') || "
                    "github.event_name == 'workflow_dispatch'",
                    1,
                ),
            ),
            (
                "root-write-permission",
                wheels.replace("  contents: read", "  contents: write", 1),
            ),
            (
                "build-write-all",
                wheels.replace(
                    "    name: ${{ matrix.package.distribution }} "
                    "${{ matrix.platform.label }}\n",
                    "    name: ${{ matrix.package.distribution }} "
                    "${{ matrix.platform.label }}\n"
                    "    permissions: write-all\n",
                    1,
                ),
            ),
            (
                "sdist-write-all",
                wheels.replace(
                    "    name: ${{ matrix.package.distribution }} sdist\n",
                    "    name: ${{ matrix.package.distribution }} sdist\n"
                    "    permissions: write-all\n",
                    1,
                ),
            ),
            (
                "publish-contents-write",
                publish_head
                + publish_tail.replace(
                    "      contents: read", "      contents: write", 1
                ),
            ),
            (
                "publish-id-token-read",
                publish_head
                + publish_tail.replace(
                    "      id-token: write", "      id-token: read", 1
                ),
            ),
            (
                "publish-extra-permission",
                publish_head
                + publish_tail.replace(
                    "      contents: read\n",
                    "      contents: read\n      issues: write\n",
                    1,
                ),
            ),
            (
                "publish-staging-environment",
                wheels.replace("    environment: pypi", "    environment: pypi-staging", 1),
            ),
            (
                "native-continue-on-error",
                wheels.replace(
                    "      - name: Install and test native wheel\n",
                    "      - name: Install and test native wheel\n"
                    "        continue-on-error: true\n",
                    1,
                ),
            ),
            (
                "typing-continue-on-error",
                wheels.replace(
                    "      - name: Validate installed typing surface\n",
                    "      - name: Validate installed typing surface\n"
                    "        continue-on-error: true\n",
                    1,
                ),
            ),
            (
                "musllinux-if-false",
                wheels.replace(
                    "if: matrix.platform.install == 'musl'", "if: false", 1
                ),
            ),
            (
                "musllinux-import-only",
                wheels.replace(
                    musllinux_step, musllinux_import_only_step, 1
                ),
            ),
            (
                "musllinux-package-restriction",
                wheels.replace(
                    "if: matrix.platform.install == 'musl'",
                    "if: matrix.platform.install == 'musl' && "
                    "matrix.package.distribution == 'rdocx'",
                    1,
                ),
            ),
            (
                "musllinux-or-native",
                wheels.replace(
                    "if: matrix.platform.install == 'musl'",
                    "if: matrix.platform.install == 'musl' || "
                    "matrix.platform.install == 'native'",
                    1,
                ),
            ),
            (
                "wheel-upload-continue-on-error",
                wheels.replace(
                    "      - name: Upload wheel\n",
                    "      - name: Upload wheel\n"
                    "        continue-on-error: true\n",
                    1,
                ),
            ),
            (
                "sdist-upload-continue-on-error",
                wheels.replace(
                    "      - name: Upload source distribution\n",
                    "      - name: Upload source distribution\n"
                    "        continue-on-error: true\n",
                    1,
                ),
            ),
            (
                "wheel-upload-warn-with-error-comment",
                wheels.replace(
                    "          if-no-files-found: error\n",
                    "          if-no-files-found: warn # "
                    "if-no-files-found: error\n",
                    1,
                ),
            ),
            (
                "sdist-upload-warn-with-error-comment",
                sdist_head
                + sdist_tail.replace(
                    "          if-no-files-found: error\n",
                    "          if-no-files-found: warn # "
                    "if-no-files-found: error\n",
                    1,
                ),
            ),
            (
                "wheel-upload-policy-only-in-comment",
                wheels.replace(
                    "          if-no-files-found: error\n",
                    "          # if-no-files-found: error\n",
                    1,
                ),
            ),
            (
                "sdist-upload-policy-only-in-comment",
                sdist_head
                + sdist_tail.replace(
                    "          if-no-files-found: error\n",
                    "          # if-no-files-found: error\n",
                    1,
                ),
            ),
            (
                "publication-validation-continue-on-error",
                wheels.replace(
                    "      - name: Validate complete publication set\n",
                    "      - name: Validate complete publication set\n"
                    "        continue-on-error: true\n",
                    1,
                ),
            ),
            (
                "publication-action-continue-on-error",
                wheels.replace(
                    "      - name: Publish to PyPI with trusted publishing\n",
                    "      - name: Publish to PyPI with trusted publishing\n"
                    "        continue-on-error: true\n",
                    1,
                ),
            ),
            (
                "publication-validation-if-false",
                wheels.replace(
                    "      - name: Validate complete publication set\n",
                    "      - name: Validate complete publication set\n"
                    "        if: false\n",
                    1,
                ),
            ),
            (
                "publication-validation-if-always",
                wheels.replace(
                    "      - name: Validate complete publication set\n",
                    "      - name: Validate complete publication set\n"
                    "        if: always()\n",
                    1,
                ),
            ),
            (
                "publication-action-if-false",
                wheels.replace(
                    "      - name: Publish to PyPI with trusted publishing\n",
                    "      - name: Publish to PyPI with trusted publishing\n"
                    "        if: false\n",
                    1,
                ),
            ),
            (
                "publication-action-if-always",
                wheels.replace(
                    "      - name: Publish to PyPI with trusted publishing\n",
                    "      - name: Publish to PyPI with trusted publishing\n"
                    "        if: always()\n",
                    1,
                ),
            ),
            (
                "publish-before-validation",
                wheels.replace(
                    publication_validation + publication_action,
                    publication_action + publication_validation,
                    1,
                ),
            ),
            (
                "validation-before-download",
                wheels.replace(
                    publication_download + publication_validation,
                    publication_validation + publication_download,
                    1,
                ),
            ),
            (
                "download-other-path",
                wheels.replace("          path: dist\n", "          path: other\n", 1),
            ),
            (
                "download-specific-pattern",
                wheels.replace(
                    "          pattern: artifact-*\n",
                    "          pattern: artifact-wheel-*\n",
                    1,
                ),
            ),
            (
                "download-no-merge",
                wheels.replace(
                    "          merge-multiple: true\n",
                    "          merge-multiple: false\n",
                    1,
                ),
            ),
            (
                "narrow-wheel-validation-glob",
                wheels.replace(
                    'glob("*.whl")', 'glob("rdocx-*.whl")', 1
                ),
            ),
            (
                "narrow-sdist-validation-glob",
                wheels.replace(
                    'glob("*.tar.gz")', 'glob("rdocx-*.tar.gz")', 1
                ),
            ),
            (
                "wheel-count-preserved-only-in-comment",
                wheels.replace(
                    "          assert len(wheels) == 12, wheels\n",
                    "          assert len(wheels) == 1, wheels\n"
                    "          # assert len(wheels) == 12, wheels\n",
                    1,
                ),
            ),
            (
                "sdist-count-preserved-only-in-comment",
                wheels.replace(
                    "          assert len(sdists) == 2, sdists\n",
                    "          assert len(sdists) == 1, sdists\n"
                    "          # assert len(sdists) == 2, sdists\n",
                    1,
                ),
            ),
            (
                "publish-empty-input",
                wheels.replace(
                    "          packages-dir: dist\n",
                    "          packages-dir: empty\n",
                    1,
                ),
            ),
            (
                "rdocx-only-wheel-upload",
                wheels.replace(
                    "      - name: Upload wheel\n",
                    "      - name: Upload wheel\n"
                    "        if: matrix.package.distribution == 'rdocx'\n",
                    1,
                ),
            ),
            (
                "rdocx-only-sdist-upload",
                wheels.replace(
                    "      - name: Upload source distribution\n",
                    "      - name: Upload source distribution\n"
                    "        if: matrix.package.distribution == 'rdocx'\n",
                    1,
                ),
            ),
            (
                "push-only-wheel-build",
                wheels.replace(
                    "  build-wheels:\n",
                    "  build-wheels:\n    if: github.event_name == 'push'\n",
                    1,
                ),
            ),
            (
                "push-only-sdist-build",
                wheels.replace(
                    "  build-sdists:\n",
                    "  build-sdists:\n    if: github.event_name == 'push'\n",
                    1,
                ),
            ),
            (
                "second-publication-job",
                wheels
                + "\n  publish-copy:\n"
                + "    runs-on: ubuntu-24.04\n"
                + "    steps: []\n",
            ),
            (
                "publication-named-build-job",
                wheels.replace(
                    "    name: ${{ matrix.package.distribution }} "
                    "${{ matrix.platform.label }}",
                    "    name: Publish ${{ matrix.package.distribution }} "
                    "${{ matrix.platform.label }}",
                    1,
                ),
            ),
            ("missing-rpptx-sdist", duplicate_rdocx_sdist),
            ("extra-sdist-axis", extra_sdist_axis),
            (
                "wrong-wheel-artifact-path",
                wheels.replace("path: dist/*.whl", "path: dist/rdocx-*.whl", 1),
            ),
            (
                "wrong-wheel-artifact-name",
                wheels.replace(
                    "name: artifact-wheel-${{ matrix.package.distribution }}-"
                    "${{ matrix.platform.label }}",
                    "name: artifact-wheel-rdocx-${{ matrix.platform.label }}",
                    1,
                ),
            ),
            (
                "wrong-sdist-artifact-name",
                wheels.replace(
                    "name: artifact-sdist-${{ matrix.package.distribution }}",
                    "name: artifact-sdist-rdocx",
                    1,
                ),
            ),
            (
                "wrong-sdist-artifact-path",
                wheels.replace(
                    "path: dist/*.tar.gz", "path: dist/rdocx-*.tar.gz", 1
                ),
            ),
            (
                "wheel-fixed-target-with-expression-comment",
                wheels.replace(
                    "          target: ${{ matrix.platform.target }}\n",
                    "          target: x86_64-unknown-linux-gnu # "
                    "target: ${{ matrix.platform.target }}\n",
                    1,
                ),
            ),
            (
                "wheel-fixed-manylinux-with-expression-comment",
                wheels.replace(
                    "          manylinux: ${{ matrix.platform.manylinux }}\n",
                    "          manylinux: off # "
                    "manylinux: ${{ matrix.platform.manylinux }}\n",
                    1,
                ),
            ),
            (
                "wheel-fixed-package-with-expression-comment",
                wheels.replace(
                    "          working-directory: "
                    "crates/${{ matrix.package.crate }}\n",
                    "          working-directory: crates/rdocx-py # "
                    "working-directory: crates/${{ matrix.package.crate }}\n",
                    1,
                ),
            ),
            (
                "wheel-weakened-args-with-required-comment",
                wheels.replace(
                    "          args: --release --locked --compatibility pypi "
                    "--out ../../dist\n",
                    "          args: --release --out ../../dist # "
                    "--locked --compatibility pypi\n",
                    1,
                ),
            ),
            (
                "wheel-wrong-command-with-build-comment",
                wheels.replace(
                    "          command: build\n",
                    "          command: sdist # command: build\n",
                    1,
                ),
            ),
            (
                "wheel-wrong-maturin-version",
                wheels.replace(
                    "          maturin-version: v1.13.3\n",
                    "          maturin-version: v1.13.2\n",
                    1,
                ),
            ),
            (
                "wheel-package-restricted-build",
                wheels.replace(
                    "      - name: Build cp39-abi3 wheel\n",
                    "      - name: Build cp39-abi3 wheel\n"
                    "        if: matrix.package.distribution == 'rdocx'\n",
                    1,
                ),
            ),
            (
                "sdist-fixed-package",
                sdist_head
                + sdist_tail.replace(
                    "          working-directory: "
                    "crates/${{ matrix.package.crate }}\n",
                    "          working-directory: crates/rdocx-py\n",
                    1,
                ),
            ),
            (
                "sdist-fixed-package-with-expression-comment",
                sdist_head
                + sdist_tail.replace(
                    "          working-directory: "
                    "crates/${{ matrix.package.crate }}\n",
                    "          working-directory: crates/rdocx-py # "
                    "working-directory: crates/${{ matrix.package.crate }}\n",
                    1,
                ),
            ),
            (
                "sdist-wrong-command-with-sdist-comment",
                sdist_head
                + sdist_tail.replace(
                    "          command: sdist\n",
                    "          command: build # command: sdist\n",
                    1,
                ),
            ),
            (
                "sdist-wrong-maturin-version",
                sdist_head
                + sdist_tail.replace(
                    "          maturin-version: v1.13.3\n",
                    "          maturin-version: v1.13.2\n",
                    1,
                ),
            ),
            (
                "sdist-wrong-args-with-required-comment",
                sdist_head
                + sdist_tail.replace(
                    "          args: --out ../../dist\n",
                    "          args: --out dist # args: --out ../../dist\n",
                    1,
                ),
            ),
            (
                "sdist-package-restricted-build",
                wheels.replace(
                    "      - name: Build source distribution\n",
                    "      - name: Build source distribution\n"
                    "        if: matrix.package.distribution == 'rdocx'\n",
                    1,
                ),
            ),
            (
                "wheel-checkout-unreviewed-sha",
                wheels.replace(
                    "actions/checkout@"
                    "de0fac2e4500dabe0009e67214ff5f5447ce83dd",
                    "actions/checkout@1111111111111111111111111111111111111111 "
                    "# actions/checkout@"
                    "de0fac2e4500dabe0009e67214ff5f5447ce83dd",
                    1,
                ),
            ),
            (
                "wheel-setup-unreviewed-sha",
                wheels.replace(
                    "actions/setup-python@"
                    "a309ff8b426b58ec0e2a45f0f869d46889d02405",
                    "actions/setup-python@2222222222222222222222222222222222222222 "
                    "# actions/setup-python@"
                    "a309ff8b426b58ec0e2a45f0f869d46889d02405",
                    1,
                ),
            ),
            (
                "wheel-maturin-unreviewed-sha",
                wheels.replace(
                    "PyO3/maturin-action@"
                    "86b9d133d34bc1b40018696f782949dac11bd380",
                    "PyO3/maturin-action@"
                    "3333333333333333333333333333333333333333 "
                    "# PyO3/maturin-action@"
                    "86b9d133d34bc1b40018696f782949dac11bd380",
                    1,
                ),
            ),
            (
                "wheel-upload-unreviewed-sha",
                wheels.replace(
                    "actions/upload-artifact@"
                    "ea165f8d65b6e75b540449e92b4886f43607fa02",
                    "actions/upload-artifact@"
                    "4444444444444444444444444444444444444444 "
                    "# actions/upload-artifact@"
                    "ea165f8d65b6e75b540449e92b4886f43607fa02",
                    1,
                ),
            ),
            (
                "sdist-checkout-unreviewed-sha",
                sdist_head
                + sdist_tail.replace(
                    "actions/checkout@"
                    "de0fac2e4500dabe0009e67214ff5f5447ce83dd",
                    "actions/checkout@"
                    "6666666666666666666666666666666666666666 "
                    "# actions/checkout@"
                    "de0fac2e4500dabe0009e67214ff5f5447ce83dd",
                    1,
                ),
            ),
            (
                "sdist-maturin-unreviewed-sha",
                sdist_head
                + sdist_tail.replace(
                    "PyO3/maturin-action@"
                    "86b9d133d34bc1b40018696f782949dac11bd380",
                    "PyO3/maturin-action@"
                    "7777777777777777777777777777777777777777 "
                    "# PyO3/maturin-action@"
                    "86b9d133d34bc1b40018696f782949dac11bd380",
                    1,
                ),
            ),
            (
                "extra-unreviewed-action",
                wheels.replace(
                    "      - uses: actions/checkout@",
                    "      - uses: example/unknown@"
                    "5555555555555555555555555555555555555555\n"
                    "      - uses: actions/checkout@",
                    1,
                ),
            ),
        ) + early_success_mutations + control_flow_mutations + (
            (
                "crlf-workflow-bytes",
                workflow_bytes.replace(b"\n", b"\r\n"),
            ),
        )

        for name, mutated in mutations:
            mutated_bytes = (
                mutated if isinstance(mutated, bytes) else mutated.encode("utf-8")
            )
            self.assertNotEqual(mutated_bytes, workflow_bytes, name)
            with self.subTest(name=name):
                with self.assertRaises(AssertionError):
                    self.assert_wheels_workflow_contract(mutated_bytes)

    def assert_publish_preflight_contract(self, publish: str) -> None:
        publishable_crates = (
            "oxml-core",
            "oxml-drawing",
            "oxml-layout",
            "oxml-media",
            "oxml-opc",
            "oxml-pdf",
            "oxml-sml",
            "oxml-cli-support",
            "rdocx",
            "rdocx-cli",
            "rdocx-html",
            "rdocx-layout",
            "rdocx-opc",
            "rdocx-oxml",
            "rdocx-pdf",
            "rpptx",
            "rpptx-cli",
            "rpptx-chart",
            "rpptx-layout",
            "rpptx-oxml",
            "rpptx-render",
        )
        marker = "      - name: Verify publication archives\n"
        self.assertEqual(publish.count(marker), 1)
        start = publish.index(marker)
        end = publish.index("\n      - name:", start + len(marker))
        block = publish[start:end]

        self.assertEqual(block.count("cargo publish --workspace --dry-run"), 1)
        for package in publishable_crates:
            config = (
                f"--config 'patch.crates-io.{package}.path=\"crates/{package}\"'"
            )
            self.assertEqual(block.count(config), 1, package)
        self.assertEqual(block.count("--config 'patch.crates-io."), 21)
        self.assertNotIn("--no-verify", block)
        self.assertNotIn("continue-on-error", block)

    def assert_publish_workflow_contract(self, publish: str) -> None:
        stable_crates = (
            "rdocx-opc",
            "rdocx-oxml",
            "rdocx-layout",
            "rdocx-html",
            "rdocx-pdf",
            "rdocx",
            "rdocx-cli",
        )
        incubating_crates = (
            "oxml-core",
            "oxml-opc",
            "oxml-media",
            "oxml-layout",
            "oxml-drawing",
            "oxml-pdf",
            "oxml-sml",
            "oxml-cli-support",
            "rpptx-oxml",
            "rpptx-chart",
            "rpptx-layout",
            "rpptx-render",
            "rpptx",
            "rpptx-cli",
        )

        self.assertIn('tags: ["v*", "rpptx-v*"]', publish)
        for step_name, condition, packages in (
            (
                "Publish stable allowlist",
                "if: startsWith(github.ref_name, 'v')",
                stable_crates,
            ),
            (
                "Publish incubating allowlist",
                "if: startsWith(github.ref_name, 'rpptx-v')",
                incubating_crates,
            ),
        ):
            marker = f"      - name: {step_name}\n"
            self.assertEqual(publish.count(marker), 1)
            start = publish.index(marker)
            block_lines = []
            for line in publish[start:].splitlines():
                if block_lines and (
                    line.startswith("      - ")
                    or (line.strip() and len(line) - len(line.lstrip()) <= 4)
                ):
                    break
                block_lines.append(line)
            block = "\n".join(block_lines)

            conditions = [line.strip() for line in block_lines if "if:" in line]
            self.assertEqual(conditions, [condition])
            self.assertNotIn("continue-on-error", block)
            run_index = next(
                index
                for index, line in enumerate(block_lines)
                if line.strip() == "run: |"
            )
            commands = [
                line.strip()
                for line in block_lines[run_index + 1 :]
                if line.strip()
            ]
            expected_commands = []
            for index, package in enumerate(packages):
                expected_commands.append(f"cargo publish -p {package}")
                if index + 1 < len(packages):
                    expected_commands.append("sleep 60")
            self.assertEqual(commands, expected_commands)

            package_position = {name: index for index, name in enumerate(packages)}
            for name in packages:
                manifest = tomllib.loads(
                    (workflow.REPO / f"crates/{name}/Cargo.toml").read_text(
                        encoding="utf-8"
                    )
                )
                for dependency in manifest.get("dependencies", {}):
                    if dependency in package_position:
                        self.assertLess(
                            package_position[dependency],
                            package_position[name],
                            f"{dependency} must publish before {name}",
                        )

        actual_publish_commands = [
            line.strip()
            for line in publish.splitlines()
            if line.strip().startswith("cargo publish -p ")
        ]
        expected_publish_commands = [
            f"cargo publish -p {package}"
            for package in stable_crates + incubating_crates
        ]
        self.assertEqual(actual_publish_commands, expected_publish_commands)

    def test_preset_geometry_provenance_is_recorded(self) -> None:
        rendering = (workflow.REPO / "docs/hld/08-rendering-spec.md").read_text(
            encoding="utf-8"
        )
        risks = (
            workflow.REPO / "docs/hld/13-risks-and-open-questions.md"
        ).read_text(encoding="utf-8")
        decision = rendering + risks

        self.assertIn("ECMA-376-1_5th_edition_december_2016.zip", decision)
        self.assertIn(
            "OfficeOpenXML-DrawingMLGeometries.zip/presetShapeDefinitions.xml",
            decision,
        )
        self.assertIn("187 preset shape definitions", decision)
        self.assertIn(
            "2f7c868d857c1e3c4b5a6068759fe0e07d77ad58377a6618d1b02ba3507b6939",
            decision,
        )
        self.assertIn("Ecma software policy", decision)
        self.assertIn("three-clause BSD", decision)
        self.assertIn("retain the Ecma copyright notice", decision)

    def test_libreoffice_preset_table_remains_rejected(self) -> None:
        rendering = (workflow.REPO / "docs/hld/08-rendering-spec.md").read_text(
            encoding="utf-8"
        )
        risks = (
            workflow.REPO / "docs/hld/13-risks-and-open-questions.md"
        ).read_text(encoding="utf-8")
        decision = rendering + risks

        self.assertIn("LibreOffice's preset table must not be used", decision)
        self.assertIn("MPL-2.0 file-level copyleft", decision)

    def test_validation_only_sprint_initialises_without_wave_rows(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            current = root / "CURRENT_SPRINT.md"
            scratch = root / "scratch"
            current.write_text(
                "# Current Sprint, S11\n\n"
                "**Validation-only**: yes\n\n"
                "## The wave\n\n"
                "| F-ID | Title | Size | Status | Owner |\n"
                "|---|---|---|---|---|\n",
                encoding="utf-8",
            )
            args = argparse.Namespace(
                sprint="S11",
                resume=False,
                force=False,
                max_review_passes=3,
                max_workers=None,
            )

            with (
                patch.object(workflow, "CURRENT_SPRINT", current),
                patch.object(workflow, "SCRATCH", scratch),
            ):
                workflow.cmd_init(args)

            saved = json.loads((scratch / "S11-run.json").read_text(encoding="utf-8"))
            self.assertEqual(saved["features"], {})
            self.assertEqual(saved["phase"], "design")

    def test_empty_sprint_without_validation_marker_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            current = Path(directory) / "CURRENT_SPRINT.md"
            current.write_text(
                "# Current Sprint, S11\n\n"
                "| F-ID | Title | Size | Status | Owner |\n"
                "|---|---|---|---|---|\n",
                encoding="utf-8",
            )

            with patch.object(workflow, "CURRENT_SPRINT", current):
                with self.assertRaises(SystemExit):
                    workflow.parse_current_sprint()

    def test_workspace_release_versions_move_in_lockstep(self) -> None:
        root = tomllib.loads((workflow.REPO / "Cargo.toml").read_text(encoding="utf-8"))
        workspace = root["workspace"]
        version = workspace["package"]["version"]

        for name in (
            "rdocx-opc",
            "rdocx-oxml",
            "rdocx",
            "rdocx-layout",
            "rdocx-pdf",
            "rdocx-html",
        ):
            self.assertEqual(workspace["dependencies"][name]["version"], version)

        wasm = tomllib.loads(
            (workflow.REPO / "crates/rdocx-wasm/Cargo.toml").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(wasm["package"]["version"], {"workspace": True})
        self.assertFalse(wasm["package"]["publish"])

    def test_stable_release_family_is_prepared_at_0_5_0(self) -> None:
        expected_version = "0.5.0"
        stable_members = (
            "oxml-py-support",
            "rpptx-py",
            "rdocx-opc",
            "rdocx-oxml",
            "rdocx",
            "rdocx-layout",
            "rdocx-pdf",
            "rdocx-html",
            "rdocx-py",
            "rdocx-cli",
            "rdocx-wasm",
        )
        stable_pins = (
            "oxml-py-support",
            "rpptx-py",
            "rdocx-opc",
            "rdocx-oxml",
            "rdocx",
            "rdocx-layout",
            "rdocx-pdf",
            "rdocx-html",
            "rdocx-py",
        )
        stable_publishable = {
            "rdocx-opc",
            "rdocx-oxml",
            "rdocx-layout",
            "rdocx-html",
            "rdocx-pdf",
            "rdocx",
            "rdocx-cli",
        }
        incubating_members = (
            "oxml-core",
            "oxml-drawing",
            "oxml-layout",
            "oxml-media",
            "oxml-opc",
            "oxml-pdf",
            "oxml-sml",
            "oxml-cli-support",
            "rpptx",
            "rpptx-cli",
            "rpptx-chart",
            "rpptx-layout",
            "rpptx-oxml",
            "rpptx-render",
            "rpptx-wasm",
        )

        root_text = (workflow.REPO / "Cargo.toml").read_text(encoding="utf-8")
        root = tomllib.loads(root_text)
        workspace = root["workspace"]
        self.assertEqual(workspace["package"]["version"], expected_version)
        dependencies = workspace["dependencies"]
        for name in stable_pins:
            self.assertEqual(dependencies[name]["version"], expected_version, name)

        lock = tomllib.loads((workflow.REPO / "Cargo.lock").read_text(encoding="utf-8"))
        lock_versions = {
            package["name"]: package["version"]
            for package in lock["package"]
            if package["name"] in stable_members
        }
        self.assertEqual(set(lock_versions), set(stable_members))
        for name in stable_members:
            self.assertEqual(lock_versions[name], expected_version, name)

        publishable = set()
        for name in stable_members:
            manifest = tomllib.loads(
                (workflow.REPO / f"crates/{name}/Cargo.toml").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(manifest["package"]["version"], {"workspace": True})
            if manifest["package"].get("publish", True):
                publishable.add(name)
        self.assertEqual(publishable, stable_publishable)

        for name in ("rdocx-py", "rpptx-py"):
            pyproject = tomllib.loads(
                (workflow.REPO / f"crates/{name}/pyproject.toml").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(pyproject["project"]["version"], expected_version, name)

        ci = (workflow.REPO / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        self.assertEqual(
            ci.count(
                'verify_package "$package_root/rdocx-wasm" '
                '"@tensorbee/rdocx-wasm" "0.5.0" "rdocx_wasm"'
            ),
            1,
        )
        wasm_source = (workflow.REPO / "crates/rdocx-wasm/src/lib.rs").read_text(
            encoding="utf-8"
        )
        for dependency in ("rdocx", "rdocx-layout"):
            self.assertEqual(
                wasm_source.count(
                    f'{dependency} = {{ path = \\"crates/{dependency}\\", '
                    f'version = \\"{expected_version}\\", '
                    'default-features = false }'
                ),
                1,
                dependency,
            )

        readme_requirements = {
            "README.md": ('rdocx = "0.5"', 'version = "0.5"'),
            "crates/rdocx-cli/README.md": ("--version '^0.5'",),
            "crates/rdocx-html/README.md": ('rdocx-html = "0.5"',),
            "crates/rdocx-layout/README.md": ('rdocx-layout = "0.5"',),
            "crates/rdocx-opc/README.md": ('rdocx-opc = "0.5"',),
            "crates/rdocx-oxml/README.md": ('rdocx-oxml = "0.5"',),
            "crates/rdocx-pdf/README.md": ('rdocx-pdf = "0.5"',),
        }
        for path, requirements in readme_requirements.items():
            text = (workflow.REPO / path).read_text(encoding="utf-8")
            for requirement in requirements:
                self.assertIn(requirement, text, path)

        for name in incubating_members:
            manifest = tomllib.loads(
                (workflow.REPO / f"crates/{name}/Cargo.toml").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(manifest["package"]["version"], "0.1.3", name)
            self.assertIs(
                manifest["package"].get("publish", True),
                name != "rpptx-wasm",
                name,
            )

    def test_stable_release_family_has_lockstep_preparation_metadata(self) -> None:
        stable_packages = (
            "rdocx-opc",
            "rdocx-oxml",
            "rdocx-layout",
            "rdocx-html",
            "rdocx-pdf",
            "rdocx",
            "rdocx-cli",
            "rdocx-wasm",
        )

        for name in stable_packages:
            binary = os.environ.get("CARGO_RELEASE_BIN")
            command = [binary] if binary else ["cargo"]
            command.extend(
                (
                    "release",
                    "config",
                    "--manifest-path",
                    str(workflow.REPO / f"crates/{name}/Cargo.toml"),
                )
            )
            result = subprocess.run(
                command,
                check=True,
                capture_output=True,
                text=True,
            )
            release = tomllib.loads(result.stdout)
            self.assertEqual(release["shared-version"], "workspace")
            self.assertEqual(release["tag-name"], "v{{version}}")

    def test_incubating_release_family_has_lockstep_preparation_metadata(self) -> None:
        incubating_packages = (
            "oxml-core",
            "oxml-drawing",
            "oxml-layout",
            "oxml-media",
            "oxml-opc",
            "oxml-pdf",
            "oxml-sml",
            "oxml-cli-support",
            "rpptx-oxml",
            "rpptx-layout",
            "rpptx-render",
            "rpptx-chart",
            "rpptx",
            "rpptx-cli",
        )

        for name in incubating_packages:
            manifest = tomllib.loads(
                (workflow.REPO / f"crates/{name}/Cargo.toml").read_text(
                    encoding="utf-8"
                )
            )
            release = manifest["package"]["metadata"]["release"]
            self.assertEqual(release["shared-version"], "incubating")
            self.assertEqual(release["tag-name"], "rpptx-v{{version}}")

    def test_incubating_release_family_is_prepared_at_0_1_3(self) -> None:
        incubating_packages = (
            "oxml-core",
            "oxml-opc",
            "oxml-media",
            "oxml-layout",
            "oxml-drawing",
            "oxml-pdf",
            "oxml-sml",
            "oxml-cli-support",
            "rpptx-oxml",
            "rpptx-chart",
            "rpptx-layout",
            "rpptx-render",
            "rpptx",
            "rpptx-cli",
        )
        expected_version = "0.1.3"
        root = tomllib.loads((workflow.REPO / "Cargo.toml").read_text(encoding="utf-8"))
        dependencies = root["workspace"]["dependencies"]
        lock = tomllib.loads((workflow.REPO / "Cargo.lock").read_text(encoding="utf-8"))
        lock_versions = {
            package["name"]: package["version"]
            for package in lock["package"]
            if package["name"] in incubating_packages
        }

        self.assertEqual(set(lock_versions), set(incubating_packages))
        for name in incubating_packages:
            manifest = tomllib.loads(
                (workflow.REPO / f"crates/{name}/Cargo.toml").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(manifest["package"]["version"], expected_version, name)
            self.assertTrue(manifest["package"].get("description", "").strip(), name)
            self.assertEqual(dependencies[name]["version"], expected_version, name)
            self.assertEqual(lock_versions[name], expected_version, name)

    def assert_release_preparation_metadata_contract(
        self, manifest_overrides: dict[str, str] | None = None
    ) -> None:
        manifest_overrides = manifest_overrides or {}
        root = tomllib.loads((workflow.REPO / "Cargo.toml").read_text(encoding="utf-8"))
        release = root["workspace"]["metadata"]["release"]

        self.assertTrue(release["consolidate-commits"])
        self.assertEqual(release["dependent-version"], "upgrade")
        self.assertTrue(release["verify"])
        self.assertFalse(release["publish"])
        self.assertFalse(release["tag"])
        self.assertFalse(release["push"])
        self.assertNotIn("pre-release-replacements", release)

        family_members = {"workspace": [], "incubating": []}
        manifests = {}
        for member in root["workspace"]["members"]:
            manifest_text = manifest_overrides.get(
                member,
                (workflow.REPO / member / "Cargo.toml").read_text(encoding="utf-8"),
            )
            manifest = tomllib.loads(manifest_text)
            manifests[member] = manifest
            family = manifest["package"]["metadata"]["release"]["shared-version"]
            self.assertIn(family, family_members)
            family_members[family].append(member)

        self.assertEqual(
            tuple(family_members["workspace"]),
            (
                "crates/oxml-py-support",
                "crates/rpptx-py",
                "crates/rdocx-opc",
                "crates/rdocx-oxml",
                "crates/rdocx",
                "crates/rdocx-layout",
                "crates/rdocx-pdf",
                "crates/rdocx-html",
                "crates/rdocx-py",
                "crates/rdocx-cli",
                "crates/rdocx-wasm",
            ),
        )
        self.assertEqual(
            tuple(family_members["incubating"]),
            (
                "crates/oxml-core",
                "crates/oxml-drawing",
                "crates/oxml-layout",
                "crates/oxml-media",
                "crates/oxml-opc",
                "crates/oxml-pdf",
                "crates/oxml-sml",
                "crates/oxml-cli-support",
                "crates/rpptx",
                "crates/rpptx-cli",
                "crates/rpptx-chart",
                "crates/rpptx-layout",
                "crates/rpptx-oxml",
                "crates/rpptx-render",
                "crates/rpptx-wasm",
            ),
        )

        family_counts = {
            family: len(members) for family, members in family_members.items()
        }
        self.assertEqual(family_counts, {"workspace": 11, "incubating": 15})

        wasm_package = manifests["crates/rpptx-wasm"]["package"]
        self.assertEqual(wasm_package["name"], "rpptx-wasm")
        self.assertEqual(wasm_package["version"], "0.1.3")
        self.assertTrue(wasm_package.get("description", "").strip())
        self.assertFalse(wasm_package["publish"])
        self.assertEqual(
            wasm_package["metadata"]["release"],
            {
                "shared-version": "incubating",
                "tag-name": "rpptx-v{{version}}",
            },
        )

        dependencies = root["workspace"]["dependencies"]
        self.assertNotIn("rpptx-wasm", dependencies)
        lock = tomllib.loads((workflow.REPO / "Cargo.lock").read_text(encoding="utf-8"))
        wasm_lock_versions = tuple(
            package["version"]
            for package in lock["package"]
            if package["name"] == "rpptx-wasm"
        )
        self.assertEqual(wasm_lock_versions, ("0.1.3",))

    def test_release_preparation_metadata_cannot_mutate_external_state(self) -> None:
        self.assert_release_preparation_metadata_contract()

    def test_release_preparation_metadata_rejects_a_wasm_family_mutation(
        self,
    ) -> None:
        member = "crates/rpptx-wasm"
        manifest = (workflow.REPO / member / "Cargo.toml").read_text(
            encoding="utf-8"
        )
        mutated = manifest.replace(
            'shared-version = "incubating"',
            'shared-version = "workspace"',
            1,
        )
        self.assertNotEqual(mutated, manifest)
        with self.assertRaises(AssertionError):
            self.assert_release_preparation_metadata_contract({member: mutated})

    def test_release_preparation_metadata_rejects_wasm_tag_and_version_mutations(
        self,
    ) -> None:
        member = "crates/rpptx-wasm"
        manifest = (workflow.REPO / member / "Cargo.toml").read_text(
            encoding="utf-8"
        )
        mutations = {
            "stable-tag-template": manifest.replace(
                'tag-name = "rpptx-v{{version}}"',
                'tag-name = "v{{version}}"',
                1,
            ),
            "workspace-version": manifest.replace(
                'version = "0.1.3"',
                "version.workspace = true",
                1,
            ),
        }
        for name, mutated in mutations.items():
            self.assertNotEqual(mutated, manifest, name)
            with self.subTest(name=name), self.assertRaises(AssertionError):
                self.assert_release_preparation_metadata_contract({member: mutated})

    def test_release_command_is_the_only_release_tag_authority(self) -> None:
        release = (workflow.REPO / ".claude/commands/release.md").read_text(
            encoding="utf-8"
        )
        run_sprint = (workflow.REPO / ".claude/commands/run-sprint.md").read_text(
            encoding="utf-8"
        )
        close_sprint = (workflow.REPO / ".claude/commands/close-sprint.md").read_text(
            encoding="utf-8"
        )
        complete_feature = (
            workflow.REPO / ".claude/commands/complete-feature.md"
        ).read_text(encoding="utf-8")
        normalized_release = " ".join(release.split())

        self.assertIn("only command", release)
        self.assertIn("# /release {vX.Y.Z | rpptx-vX.Y.Z}", release)
        self.assertIn(
            "The exact seven-package stable set is `rdocx-opc`, `rdocx-oxml`, "
            "`rdocx-layout`, `rdocx-html`, `rdocx-pdf`, `rdocx`, and "
            "`rdocx-cli`,",
            normalized_release,
        )
        self.assertIn(
            "The exact 14-package incubating set is `oxml-core`, `oxml-opc`, "
            "`oxml-media`, `oxml-layout`, `oxml-drawing`, `oxml-pdf`, "
            "`oxml-sml`, `oxml-cli-support`, `rpptx-oxml`, `rpptx-chart`, `rpptx-layout`, "
            "`rpptx-render`, `rpptx`, and `rpptx-cli`.",
            normalized_release,
        )
        self.assertIn("go or no-go immediately", normalized_release)
        self.assertIn(
            "The `oxml-layout` archive must contain its complete bundled TTF "
            "and legal-file inventory. The `rdocx-layout` archive must not "
            "duplicate those assets.",
            normalized_release,
        )
        self.assertNotIn(
            "The `rdocx-layout` and `oxml-layout` archives must contain",
            normalized_release,
        )
        self.assertIn(
            "Create one annotated tag for the requested argument",
            normalized_release,
        )
        self.assertIn("Push only that requested tag", normalized_release)
        self.assertIn("/release", run_sprint)
        self.assertIn("Leave it\n`reviewed`", run_sprint)
        self.assertIn("/release", close_sprint)
        self.assertIn("deferred to /release", complete_feature)

    def test_completed_shared_and_powerpoint_crates_are_publication_candidates(
        self,
    ) -> None:
        incubating_packages = (
            "oxml-core",
            "oxml-opc",
            "oxml-media",
            "oxml-layout",
            "oxml-drawing",
            "oxml-pdf",
            "oxml-sml",
            "oxml-cli-support",
            "rpptx-oxml",
            "rpptx-chart",
            "rpptx-layout",
            "rpptx-render",
            "rpptx",
        )

        for name in incubating_packages:
            manifest = tomllib.loads(
                (workflow.REPO / f"crates/{name}/Cargo.toml").read_text(
                    encoding="utf-8"
                )
            )
            self.assertIs(manifest["package"].get("publish"), True, name)

    def test_publish_workflow_routes_exact_dependency_ordered_allowlists(self) -> None:
        publish = (workflow.REPO / ".github/workflows/publish.yml").read_text(
            encoding="utf-8"
        )
        self.assert_publish_workflow_contract(publish)

    def test_publish_workflow_rejects_swapped_namespace_predicates(self) -> None:
        publish = (workflow.REPO / ".github/workflows/publish.yml").read_text(
            encoding="utf-8"
        )
        mutated = publish.replace(
            "if: startsWith(github.ref_name, 'v')",
            "if: TEMPORARY_PREDICATE",
            1,
        )
        mutated = mutated.replace(
            "if: startsWith(github.ref_name, 'rpptx-v')",
            "if: startsWith(github.ref_name, 'v')",
            1,
        ).replace(
            "if: TEMPORARY_PREDICATE",
            "if: startsWith(github.ref_name, 'rpptx-v')",
            1,
        )

        with self.assertRaises(AssertionError):
            self.assert_publish_workflow_contract(mutated)

    def test_publish_workflow_rejects_an_extra_package(self) -> None:
        publish = (workflow.REPO / ".github/workflows/publish.yml").read_text(
            encoding="utf-8"
        )
        mutated = publish.replace(
            "\n  release:\n",
            "\n      - name: Publish an extra package\n"
            "        if: startsWith(github.ref_name, 'v')\n"
            "        run: |\n"
            "          cargo publish -p rdocx-wasm\n"
            "\n  release:\n",
            1,
        )

        with self.assertRaises(AssertionError):
            self.assert_publish_workflow_contract(mutated)

    def test_publish_workflow_rejects_continue_on_error(self) -> None:
        publish = (workflow.REPO / ".github/workflows/publish.yml").read_text(
            encoding="utf-8"
        )
        mutated = publish.replace(
            "      - name: Publish stable allowlist\n",
            "      - name: Publish stable allowlist\n"
            "        continue-on-error: true\n",
            1,
        )

        with self.assertRaises(AssertionError):
            self.assert_publish_workflow_contract(mutated)

    def test_publish_workflow_rejects_successful_fallback_commands(self) -> None:
        publish = (workflow.REPO / ".github/workflows/publish.yml").read_text(
            encoding="utf-8"
        )
        mutated = publish.replace(
            "          cargo publish -p rdocx-opc\n",
            "          cargo publish -p rdocx-opc || true\n",
            1,
        )

        with self.assertRaises(AssertionError):
            self.assert_publish_workflow_contract(mutated)

    def test_publish_workflow_preflights_and_propagates_failures(self) -> None:
        publish = (workflow.REPO / ".github/workflows/publish.yml").read_text(
            encoding="utf-8"
        )
        stable_check = (
            "scripts.test_sprint_workflow.SprintWorkflowTests."
            "test_stable_release_family_is_prepared_at_0_5_0"
        )
        incubating_check = (
            "scripts.test_sprint_workflow.SprintWorkflowTests."
            "test_incubating_release_family_is_prepared_at_0_1_3"
        )
        metadata_command = (
            "python3 -m unittest "
            f"{stable_check} {incubating_check}"
        )

        self.assert_publish_preflight_contract(publish)
        self.assertEqual(publish.count(metadata_command), 1)
        self.assertLess(publish.index(stable_check), publish.index(incubating_check))
        self.assertLess(
            publish.index("python3 scripts/hash_harness.py --check"),
            publish.index(metadata_command),
        )
        self.assertLess(
            publish.index(metadata_command),
            publish.index("cargo publish --workspace --dry-run"),
        )
        self.assertLess(
            publish.index("cargo publish --workspace --dry-run"),
            publish.index("cargo publish -p rdocx-opc"),
        )
        self.assertNotIn("--no-verify", publish)
        self.assertNotIn("continue-on-error", publish)

    def test_publish_workflow_rejects_a_missing_local_patch(self) -> None:
        publish = (workflow.REPO / ".github/workflows/publish.yml").read_text(
            encoding="utf-8"
        )
        mutated = publish.replace(
            "            --config 'patch.crates-io.oxml-core.path=\"crates/oxml-core\"' \\\n",
            "",
            1,
        )

        with self.assertRaises(AssertionError):
            self.assert_publish_preflight_contract(mutated)

    def test_review_and_verification_evidence_is_bound_to_head(self) -> None:
        data = {
            "reviews": [{"pass": 4, "blocking": 0, "head": "current"}],
            "verifications": [
                {"scope": "full", "passed": True, "head": "current"}
            ],
        }
        self.assertEqual(workflow.closure_evidence_problems(data, "current"), [])

        data["reviews"][-1]["head"] = "reviewed-old"
        data["verifications"][-1]["head"] = "verified-old"
        self.assertEqual(
            workflow.closure_evidence_problems(data, "current"),
            [
                "latest sprint review covered reviewed-old, current HEAD is current",
                "no passing `/verify --full` recorded for current HEAD current",
            ],
        )

    def test_recorded_evidence_captures_head(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            scratch = Path(directory)
            state = {
                "schema_version": workflow.SCHEMA_VERSION,
                "sprint": "S01",
                "phase": "review",
                "max_review_passes": 3,
                "features": {},
                "reviews": [],
                "verifications": [],
            }
            (scratch / "S01-run.json").write_text(json.dumps(state), encoding="utf-8")
            review_args = argparse.Namespace(
                sprint="S01",
                passno=4,
                blocking=0,
                should_fix=0,
                nice_to_have=0,
                extend=True,
            )
            verify_args = argparse.Namespace(
                sprint="S01",
                scope="full",
                passed=True,
                harness="unchanged",
            )

            with (
                patch.object(workflow, "SCRATCH", scratch),
                patch.object(workflow, "git_head", return_value="abc123"),
            ):
                workflow.cmd_record_review(review_args)
                workflow.cmd_record_verification(verify_args)

            saved = json.loads((scratch / "S01-run.json").read_text(encoding="utf-8"))
            self.assertEqual(saved["reviews"][-1]["head"], "abc123")
            self.assertEqual(saved["verifications"][-1]["head"], "abc123")

    def test_run_sprint_phase_sequence_is_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            scratch = Path(directory)
            state = {
                "schema_version": workflow.SCHEMA_VERSION,
                "sprint": "S01",
                "phase": "design",
                "features": {},
                "reviews": [],
                "verifications": [],
            }
            (scratch / "S01-run.json").write_text(json.dumps(state), encoding="utf-8")

            with patch.object(workflow, "SCRATCH", scratch):
                for phase in (
                    "questions",
                    "implementation",
                    "integration",
                    "verification",
                    "review",
                    "ready_to_close",
                ):
                    workflow.cmd_set_phase(argparse.Namespace(sprint="S01", phase=phase))
                    saved = json.loads((scratch / "S01-run.json").read_text(encoding="utf-8"))
                    self.assertEqual(saved["phase"], phase)

    def test_completed_feature_requires_every_delivery_record(self) -> None:
        with tempfile.TemporaryDirectory(dir=workflow.REPO) as directory:
            root = Path(directory)
            current = root / "CURRENT_SPRINT.md"
            backlog = root / "BACKLOG.md"
            tracker = root / "SPRINT_TRACKER.md"
            as_built = root / "AS_BUILT.md"
            plans = root / "plans"
            plans.mkdir()
            current.write_text(
                "# Current Sprint, S01\n\n"
                "| F-ID | Title | Size | Status | Owner |\n"
                "|---|---|---|---|---|\n"
                "| F-001 | Example | S | done | - |\n",
                encoding="utf-8",
            )
            backlog.write_text(
                "| F-ID | Title | Sprint | Size | Status |\n"
                "|---|---|---|---|---|\n"
                "| F-001 | Example | S01 | S | done |\n",
                encoding="utf-8",
            )
            tracker.write_text("| F-001 | S01 | S | 1 | 1 | date | note |\n", encoding="utf-8")
            as_built.write_text("### F-001, Example\n", encoding="utf-8")
            (plans / "F-001-design.md").write_text(
                "**Status**: completed\n", encoding="utf-8"
            )

            with patch.multiple(
                workflow,
                CURRENT_SPRINT=current,
                BACKLOG=backlog,
                SPRINT_TRACKER=tracker,
                AS_BUILT=as_built,
                PLANS=plans,
            ):
                self.assertEqual(workflow.completed_record_problems("S01", "F-001"), [])
                current.write_text(
                    "# Current Sprint, S01\n\n"
                    "| F-ID | Title | Size | Status | Owner |\n"
                    "|---|---|---|---|---|\n"
                    "| F-001 | Example | S | done | |\n",
                    encoding="utf-8",
                )
                self.assertEqual(workflow.completed_record_problems("S01", "F-001"), [])
                current.write_text(
                    "# Current Sprint, S01\n\n"
                    "| F-ID | Title | Size | Status | Owner |\n"
                    "|---|---|---|---|---|\n"
                    "| F-001 | Example | S | done | codex |\n",
                    encoding="utf-8",
                )
                self.assertEqual(
                    workflow.completed_record_problems("S01", "F-001"),
                    ["F-001 is completed but CURRENT_SPRINT.md owner is 'codex'"],
                )
                tracker.write_text("", encoding="utf-8")
                current.write_text(
                    "# Current Sprint, S01\n\n"
                    "| F-ID | Title | Size | Status | Owner |\n"
                    "|---|---|---|---|---|\n"
                    "| F-001 | Example | S | done | |\n",
                    encoding="utf-8",
                )
                self.assertEqual(
                    workflow.completed_record_problems("S01", "F-001"),
                    ["F-001 has no S01 row in SPRINT_TRACKER.md"],
                )

    def test_completed_run_state_requires_a_cleared_owner(self) -> None:
        data = {
            "features": {
                "F-001": {"state": "completed", "owner": None},
                "F-002": {"state": "completed", "owner": "codex"},
                "F-003": {"state": "carried", "owner": "claude"},
            }
        }

        self.assertEqual(
            workflow.completed_owner_problems(data),
            ["F-002 is completed but run-state owner is 'codex'"],
        )

    def test_close_preflight_rejects_a_completed_run_state_owner(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            reviews = Path(directory)
            (reviews / "S01-sprint-review-pass-1.md").write_text(
                "**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have\n",
                encoding="utf-8",
            )
            data = {
                "phase": "review",
                "features": {
                    "F-001": {"state": "completed", "owner": "codex"},
                    "F-002": {"state": "carried", "owner": "claude"},
                },
                "reviews": [{"pass": 1, "blocking": 0, "head": "current"}],
                "verifications": [
                    {
                        "scope": "full",
                        "passed": True,
                        "harness": "unchanged",
                        "head": "current",
                    }
                ],
            }

            with (
                patch.object(workflow, "load", return_value=data),
                patch.object(workflow, "git_head", return_value="current"),
                patch.object(workflow, "HANDOFFS", reviews / "handoffs"),
                patch.object(workflow, "REVIEWS", reviews),
                patch.object(workflow, "backlog_statuses", return_value={"F-001": "done"}),
                patch.object(workflow, "completed_record_problems", return_value=[]),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                self.assertEqual(
                    workflow.cmd_close_preflight(argparse.Namespace(sprint="S01")),
                    1,
                )
                data["features"]["F-001"]["owner"] = None
                self.assertEqual(
                    workflow.cmd_close_preflight(argparse.Namespace(sprint="S01")),
                    0,
                )


if __name__ == "__main__":
    unittest.main()

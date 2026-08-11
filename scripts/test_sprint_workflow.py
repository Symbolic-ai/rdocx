from __future__ import annotations

import argparse
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
    def assert_publish_preflight_contract(self, publish: str) -> None:
        publishable_crates = (
            "oxml-core",
            "oxml-drawing",
            "oxml-layout",
            "oxml-media",
            "oxml-opc",
            "oxml-pdf",
            "oxml-sml",
            "rdocx",
            "rdocx-cli",
            "rdocx-html",
            "rdocx-layout",
            "rdocx-opc",
            "rdocx-oxml",
            "rdocx-pdf",
            "rpptx",
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
        self.assertEqual(block.count("--config 'patch.crates-io."), 19)
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
            "rpptx-oxml",
            "rpptx-chart",
            "rpptx-layout",
            "rpptx-render",
            "rpptx",
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
            "rpptx-oxml",
            "rpptx-layout",
            "rpptx-render",
            "rpptx-chart",
            "rpptx",
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

    def test_release_preparation_metadata_cannot_mutate_external_state(self) -> None:
        root = tomllib.loads((workflow.REPO / "Cargo.toml").read_text(encoding="utf-8"))
        release = root["workspace"]["metadata"]["release"]

        self.assertTrue(release["consolidate-commits"])
        self.assertEqual(release["dependent-version"], "upgrade")
        self.assertTrue(release["verify"])
        self.assertFalse(release["publish"])
        self.assertFalse(release["tag"])
        self.assertFalse(release["push"])
        self.assertNotIn("pre-release-replacements", release)

        family_counts = {"workspace": 0, "incubating": 0}
        for member in root["workspace"]["members"]:
            manifest = tomllib.loads(
                (workflow.REPO / member / "Cargo.toml").read_text(encoding="utf-8")
            )
            family = manifest["package"]["metadata"]["release"]["shared-version"]
            self.assertIn(family, family_counts)
            family_counts[family] += 1

        self.assertEqual(family_counts, {"workspace": 8, "incubating": 12})

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
            "The exact 12-package incubating set is `oxml-core`, `oxml-opc`, "
            "`oxml-media`, `oxml-layout`, `oxml-drawing`, `oxml-pdf`, "
            "`oxml-sml`, `rpptx-oxml`, `rpptx-chart`, `rpptx-layout`, "
            "`rpptx-render`, and `rpptx`.",
            normalized_release,
        )
        self.assertIn("go or no-go immediately", normalized_release)
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

        self.assert_publish_preflight_contract(publish)
        self.assertLess(
            publish.index("python3 scripts/hash_harness.py --check"),
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
                tracker.write_text("", encoding="utf-8")
                self.assertEqual(
                    workflow.completed_record_problems("S01", "F-001"),
                    ["F-001 has no S01 row in SPRINT_TRACKER.md"],
                )


if __name__ == "__main__":
    unittest.main()

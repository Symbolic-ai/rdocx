from __future__ import annotations

import argparse
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import sprint_workflow as workflow


class SprintWorkflowTests(unittest.TestCase):
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

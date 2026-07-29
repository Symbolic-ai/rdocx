#!/usr/bin/env python3
"""Output-stability hash harness for the generated rdocx samples.

The harness regenerates the seven named sample documents, hashes selected OOXML
parts and a deterministic page-one PNG, then compares those values with the
checked-in baseline.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import unittest
import zipfile
from pathlib import Path
from tempfile import TemporaryDirectory


HashValue = str | None
REPO_ROOT = Path(__file__).resolve().parent.parent
SAMPLES_DIR = REPO_ROOT / "samples"
BASELINE_PATH = Path(__file__).resolve().with_name("hash_baseline.json")
SAMPLES = (
    "feature_showcase",
    "proposal",
    "quote",
    "invoice",
    "report",
    "letter",
    "contract",
)
OOXML_PARTS = (
    "word/document.xml",
    "word/styles.xml",
    "word/numbering.xml",
)
EXPECTED_ENTRY_COUNT = len(SAMPLES) * (len(OOXML_PARTS) + 1)


def display_hash(value: HashValue) -> str:
    return "absent" if value is None else value


def compare_hashes(
    expected: dict[str, HashValue], actual: dict[str, HashValue]
) -> list[str]:
    """Return precise differences between two hash manifests."""
    differences = [
        f"added: {key} = {display_hash(actual[key])}"
        for key in sorted(actual.keys() - expected.keys())
    ]
    differences.extend(
        f"removed: {key} = {display_hash(expected[key])}"
        for key in sorted(expected.keys() - actual.keys())
    )
    differences.extend(
        f"changed: {key} expected {display_hash(expected[key])}, "
        f"got {display_hash(actual[key])}"
        for key in sorted(expected.keys() & actual.keys())
        if expected[key] != actual[key]
    )
    return differences


def validate_update_reason(reason: str | None) -> str:
    if reason is None or not reason.strip():
        raise ValueError("--update requires a non-empty --reason")
    return reason.strip()


def write_baseline(
    path: Path, hashes: dict[str, HashValue], reason: str | None
) -> None:
    """Write a baseline after validating its audit reason."""
    reason = validate_update_reason(reason)
    payload = {"entries": dict(sorted(hashes.items())), "reason": reason}
    path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


def read_baseline(path: Path) -> dict[str, HashValue]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict) or not isinstance(payload.get("entries"), dict):
        raise ValueError(f"{path} does not contain an entries object")

    entries = payload["entries"]
    for key, value in entries.items():
        if not isinstance(key, str) or not (isinstance(value, str) or value is None):
            raise ValueError(f"{path} contains an invalid hash entry")
    return entries


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run_sample_generator() -> None:
    for sample in SAMPLES:
        for extension in ("docx", "png"):
            (SAMPLES_DIR / f"{sample}.{extension}").unlink(missing_ok=True)

    subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "-p",
            "rdocx",
            "--example",
            "generate_all_samples",
        ],
        cwd=REPO_ROOT,
        check=True,
    )


def collect_hashes(samples_dir: Path) -> dict[str, HashValue]:
    hashes: dict[str, HashValue] = {}
    for sample in SAMPLES:
        docx_path = samples_dir / f"{sample}.docx"
        with zipfile.ZipFile(docx_path) as package:
            package_parts = set(package.namelist())
            for part in OOXML_PARTS:
                key = f"{sample}:{part}"
                hashes[key] = sha256(package.read(part)) if part in package_parts else None

        png_path = samples_dir / f"{sample}.png"
        hashes[f"{sample}:page1.png"] = sha256(png_path.read_bytes())

    if len(hashes) != EXPECTED_ENTRY_COUNT:
        raise ValueError(
            f"expected {EXPECTED_ENTRY_COUNT} hash entries, collected {len(hashes)}"
        )
    return dict(sorted(hashes.items()))


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="compare with the baseline")
    mode.add_argument("--update", action="store_true", help="replace the baseline")
    parser.add_argument("--reason", help="required audit reason for --update")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        if args.update:
            validate_update_reason(args.reason)
        elif args.reason is not None:
            raise ValueError("--reason is only valid with --update")

        run_sample_generator()
        actual = collect_hashes(SAMPLES_DIR)

        if args.update:
            write_baseline(BASELINE_PATH, actual, args.reason)
            print(
                f"hash_harness: wrote {len(actual)} entries to "
                f"{BASELINE_PATH.relative_to(REPO_ROOT)}"
            )
            return 0

        expected = read_baseline(BASELINE_PATH)
        differences = compare_hashes(expected, actual)
        if differences:
            print("hash_harness: output delta detected", file=sys.stderr)
            for difference in differences:
                print(f"  - {difference}", file=sys.stderr)
            return 1

        print(f"hash_harness: {len(actual)} entries match")
        return 0
    except (OSError, ValueError, zipfile.BadZipFile, subprocess.CalledProcessError) as error:
        print(f"hash_harness: {error}", file=sys.stderr)
        return 2


class HashHarnessTests(unittest.TestCase):
    def test_missing_added_and_changed_digests_produce_precise_failures(self) -> None:
        expected = {"changed": "old", "removed": "gone"}
        actual = {"added": "new", "changed": "new"}

        self.assertEqual(
            compare_hashes(expected, actual),
            [
                "added: added = new",
                "removed: removed = gone",
                "changed: changed expected old, got new",
            ],
        )

    def test_update_requires_a_non_empty_reason(self) -> None:
        with TemporaryDirectory() as temp_dir:
            baseline = Path(temp_dir) / "baseline.json"

            with self.assertRaisesRegex(ValueError, "non-empty --reason"):
                write_baseline(baseline, {"sample/page1.png": "digest"}, "  ")

            self.assertFalse(baseline.exists())


if __name__ == "__main__":
    sys.exit(main())

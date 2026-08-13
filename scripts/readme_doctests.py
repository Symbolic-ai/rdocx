#!/usr/bin/env python3
"""Compile the root README's Rust examples without running them."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import subprocess
import sys


REPO_ROOT = Path(__file__).resolve().parent.parent
README = REPO_ROOT / "README.md"
EXPECTED_RUST_FENCES = 6
RUST_FENCE = re.compile(r"^```rust(?P<attributes>[^\n]*)$", re.MULTILINE)


def validate_fences(readme: Path) -> bool:
    text = readme.read_text(encoding="utf-8")
    attributes = RUST_FENCE.findall(text)
    if len(attributes) != EXPECTED_RUST_FENCES or any(
        attribute != ",no_run" for attribute in attributes
    ):
        print(
            f"README doctest error: expected {EXPECTED_RUST_FENCES} "
            f"exact rust,no_run fences, found {len(attributes)} with "
            f"attributes {attributes!r}",
            file=sys.stderr,
        )
        return False
    return True


def build_rdocx_rlib() -> Path | None:
    command = [
        "cargo",
        "build",
        "--locked",
        "-p",
        "rdocx",
        "--message-format=json-render-diagnostics",
    ]
    result = subprocess.run(
        command,
        cwd=REPO_ROOT,
        stdout=subprocess.PIPE,
        text=True,
        check=False,
    )
    artifacts: set[Path] = set()
    for line in result.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("reason") == "compiler-message":
            rendered = message.get("message", {}).get("rendered")
            if rendered:
                print(rendered, file=sys.stderr, end="")
        if message.get("reason") != "compiler-artifact":
            continue
        target = message.get("target", {})
        if target.get("name") != "rdocx" or "lib" not in target.get("crate_types", []):
            continue
        artifacts.update(
            Path(filename).resolve()
            for filename in message.get("filenames", [])
            if filename.endswith(".rlib")
        )
    if result.returncode != 0:
        return None
    if len(artifacts) != 1:
        print(
            "README doctest error: expected one rdocx rlib, found "
            f"{sorted(str(path) for path in artifacts)!r}",
            file=sys.stderr,
        )
        return None
    return artifacts.pop()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("readme", nargs="?", type=Path, default=README)
    args = parser.parse_args()
    readme = args.readme.resolve()
    if not validate_fences(readme):
        return 1

    rlib = build_rdocx_rlib()
    if rlib is None:
        return 1
    dependency_dir = rlib.parent / "deps"
    if not dependency_dir.is_dir():
        dependency_dir = rlib.parent
    result = subprocess.run(
        [
            "rustdoc",
            "--test",
            str(readme),
            "--crate-name",
            "rdocx_readme",
            "--edition=2024",
            "-Dwarnings",
            "-L",
            f"dependency={dependency_dir}",
            "--extern",
            f"rdocx={rlib}",
        ],
        cwd=REPO_ROOT,
        check=False,
    )
    if result.returncode == 0:
        print(
            f"readme_doctests: {EXPECTED_RUST_FENCES} Rust examples compiled "
            f"from {readme}"
        )
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())

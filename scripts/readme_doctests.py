#!/usr/bin/env python3
"""Compile stable crate README Rust examples without running them."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
from pathlib import Path
import re
import subprocess
import sys


REPO_ROOT = Path(__file__).resolve().parent.parent
RUST_FENCE = re.compile(r"^```rust(?P<attributes>[^\n]*)$", re.MULTILINE)


@dataclass(frozen=True)
class ReadmeCase:
    package: str
    crate_name: str
    readme: Path
    expected_rust_fences: int


README_CASES = (
    ReadmeCase("rdocx", "rdocx", REPO_ROOT / "README.md", 7),
    ReadmeCase(
        "rdocx-opc",
        "rdocx_opc",
        REPO_ROOT / "crates/rdocx-opc/README.md",
        1,
    ),
    ReadmeCase(
        "rdocx-oxml",
        "rdocx_oxml",
        REPO_ROOT / "crates/rdocx-oxml/README.md",
        1,
    ),
    ReadmeCase(
        "rdocx-layout",
        "rdocx_layout",
        REPO_ROOT / "crates/rdocx-layout/README.md",
        1,
    ),
    ReadmeCase(
        "rdocx-html",
        "rdocx_html",
        REPO_ROOT / "crates/rdocx-html/README.md",
        1,
    ),
    ReadmeCase(
        "rdocx-pdf",
        "rdocx_pdf",
        REPO_ROOT / "crates/rdocx-pdf/README.md",
        1,
    ),
)

README_INVENTORY = (
    ("rdocx", REPO_ROOT / "README.md", '../../README.md'),
    ("rdocx-opc", REPO_ROOT / "crates/rdocx-opc/README.md", "README.md"),
    ("rdocx-oxml", REPO_ROOT / "crates/rdocx-oxml/README.md", "README.md"),
    ("rdocx-layout", REPO_ROOT / "crates/rdocx-layout/README.md", "README.md"),
    ("rdocx-html", REPO_ROOT / "crates/rdocx-html/README.md", "README.md"),
    ("rdocx-pdf", REPO_ROOT / "crates/rdocx-pdf/README.md", "README.md"),
    ("rdocx-cli", REPO_ROOT / "crates/rdocx-cli/README.md", "README.md"),
)

README_REQUIRED_TEXT = {
    REPO_ROOT / "README.md": (
        'rdocx = "0.5"',
        'rdocx = { version = "0.5", default-features = false }',
        "rdocx convert report.docx --to pdf -o report.pdf",
        "rdocx convert report.docx --to html -o report.html",
        "rdocx convert report.docx --to md -o report.md",
        'rdocx replace report.docx --placeholder "Draft" --value "Final" -o final.docx',
    ),
    REPO_ROOT / "crates/rdocx-cli/README.md": (
        "cargo install rdocx-cli --version '^0.5'",
        "rdocx convert report.docx --to pdf -o report.pdf",
    ),
    REPO_ROOT / "crates/rdocx-html/README.md": ('rdocx-html = "0.5"',),
    REPO_ROOT / "crates/rdocx-layout/README.md": ('rdocx-layout = "0.5"',),
    REPO_ROOT / "crates/rdocx-opc/README.md": (
        'rdocx-opc = "0.5"',
        "use rdocx_opc::OpcPackage;",
    ),
    REPO_ROOT / "crates/rdocx-oxml/README.md": ('rdocx-oxml = "0.5"',),
    REPO_ROOT / "crates/rdocx-pdf/README.md": (
        'rdocx-pdf = "0.5"',
        "use rdocx_pdf::render_to_pdf;",
    ),
}


def validate_fences(readme: Path, expected: int) -> bool:
    text = readme.read_text(encoding="utf-8")
    attributes = RUST_FENCE.findall(text)
    if len(attributes) != expected or any(
        attribute != ",no_run" for attribute in attributes
    ):
        print(
            f"README doctest error: expected {expected} "
            f"exact rust,no_run fences, found {len(attributes)} with "
            f"attributes {attributes!r}",
            file=sys.stderr,
        )
        return False
    return True


def validate_inventory() -> bool:
    valid = True
    for package, readme, manifest_value in README_INVENTORY:
        manifest = REPO_ROOT / f"crates/{package}/Cargo.toml"
        expected = f'readme = "{manifest_value}"'
        if not readme.is_file():
            print(f"README doctest error: missing {readme}", file=sys.stderr)
            valid = False
        if expected not in manifest.read_text(encoding="utf-8"):
            print(
                f"README doctest error: {manifest} does not contain {expected!r}",
                file=sys.stderr,
            )
            valid = False
    for readme, required_items in README_REQUIRED_TEXT.items():
        text = readme.read_text(encoding="utf-8")
        for required in required_items:
            if required not in text:
                print(
                    f"README doctest error: {readme} does not contain "
                    f"{required!r}",
                    file=sys.stderr,
                )
                valid = False
    return valid


def build_rlib(package: str, crate_name: str) -> Path | None:
    command = [
        "cargo",
        "build",
        "--locked",
        "-p",
        package,
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
        if target.get("name") != crate_name or "lib" not in target.get("crate_types", []):
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
            f"README doctest error: expected one {package} rlib, found "
            f"{sorted(str(path) for path in artifacts)!r}",
            file=sys.stderr,
        )
        return None
    return artifacts.pop()


def compile_readme(case: ReadmeCase) -> bool:
    if not validate_fences(case.readme, case.expected_rust_fences):
        return False

    rlib = build_rlib(case.package, case.crate_name)
    if rlib is None:
        return False
    dependency_dir = rlib.parent / "deps"
    if not dependency_dir.is_dir():
        dependency_dir = rlib.parent
    result = subprocess.run(
        [
            "rustdoc",
            "--test",
            str(case.readme),
            "--crate-name",
            f"{case.crate_name}_readme",
            "--edition=2024",
            "-Dwarnings",
            "-L",
            f"dependency={dependency_dir}",
            "--extern",
            f"{case.crate_name}={rlib}",
        ],
        cwd=REPO_ROOT,
        check=False,
    )
    if result.returncode == 0:
        print(
            f"readme_doctests: {case.expected_rust_fences} Rust examples "
            f"compiled from {case.readme}"
        )
    return result.returncode == 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("readme", nargs="?", type=Path)
    args = parser.parse_args()
    if args.readme is not None:
        case = ReadmeCase("rdocx", "rdocx", args.readme.resolve(), 7)
        return 0 if compile_readme(case) else 1

    if not validate_inventory():
        return 1
    return 0 if all(compile_readme(case) for case in README_CASES) else 1


if __name__ == "__main__":
    raise SystemExit(main())

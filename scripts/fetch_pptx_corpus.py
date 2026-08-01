#!/usr/bin/env python3
"""Fetch and verify the pinned PresentationML corpus."""

from __future__ import annotations

import argparse
import hashlib
import os
from pathlib import Path
import sys
import urllib.request


REPO_ROOT = Path(__file__).resolve().parent.parent
MANIFEST = REPO_ROOT / "scripts" / "pptx-corpus-manifest.tsv"
OUTPUT = REPO_ROOT / "corpus" / "pptx"
EXPECTED_COUNT = 50


def load_manifest(path: Path) -> list[tuple[str, str, str, str]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0] != "path\tproducer\tsha256\turl":
        raise ValueError("manifest header must be path, producer, sha256, url")
    entries = []
    seen_paths: set[str] = set()
    for line_number, line in enumerate(lines[1:], 2):
        fields = line.split("\t")
        if len(fields) != 4:
            raise ValueError(f"manifest line {line_number} has {len(fields)} fields")
        relative_path, producer, digest, url = fields
        candidate = Path(relative_path)
        if candidate.name != relative_path or candidate.suffix.lower() != ".pptx":
            raise ValueError(f"manifest line {line_number} has unsafe path {relative_path!r}")
        if relative_path in seen_paths:
            raise ValueError(f"manifest line {line_number} duplicates {relative_path}")
        if not producer:
            raise ValueError(f"manifest line {line_number} has no producer")
        if len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest):
            raise ValueError(f"manifest line {line_number} has invalid SHA-256")
        if not url.startswith("https://"):
            raise ValueError(f"manifest line {line_number} does not use HTTPS")
        seen_paths.add(relative_path)
        entries.append((relative_path, producer, digest, url))
    if len(entries) != EXPECTED_COUNT:
        raise ValueError(f"manifest has {len(entries)} entries, expected {EXPECTED_COUNT}")
    return entries


def file_digest(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def verify_directory(output: Path, entries: list[tuple[str, str, str, str]]) -> None:
    expected = {entry[0] for entry in entries}
    actual = {path.name for path in output.iterdir()} if output.is_dir() else set()
    missing = sorted(expected - actual)
    extra = sorted(actual - expected)
    if missing:
        raise ValueError(f"corpus is missing: {', '.join(missing)}")
    if extra:
        raise ValueError(f"corpus has unmanifested files: {', '.join(extra)}")
    for relative_path, _, expected_digest, _ in entries:
        actual_digest = file_digest(output / relative_path)
        if actual_digest != expected_digest:
            raise ValueError(
                f"digest mismatch for {relative_path}: {actual_digest}, expected {expected_digest}"
            )


def fetch(output: Path, entries: list[tuple[str, str, str, str]]) -> None:
    output.mkdir(parents=True, exist_ok=True)
    expected = {entry[0] for entry in entries}
    extra = sorted(path.name for path in output.iterdir() if path.name not in expected)
    if extra:
        raise ValueError(f"corpus has unmanifested files: {', '.join(extra)}")
    for relative_path, _, expected_digest, url in entries:
        destination = output / relative_path
        if destination.is_file() and file_digest(destination) == expected_digest:
            print(f"verified {relative_path}")
            continue
        temporary = output / f".{relative_path}.download"
        request = urllib.request.Request(url, headers={"User-Agent": "rdocx-corpus-fetcher/1"})
        try:
            with urllib.request.urlopen(request, timeout=60) as response, temporary.open(
                "wb"
            ) as stream:
                while chunk := response.read(1024 * 1024):
                    stream.write(chunk)
            actual_digest = file_digest(temporary)
            if actual_digest != expected_digest:
                raise ValueError(
                    f"digest mismatch for {relative_path}: {actual_digest}, expected {expected_digest}"
                )
            os.replace(temporary, destination)
            print(f"fetched {relative_path}")
        finally:
            temporary.unlink(missing_ok=True)
    verify_directory(output, entries)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    try:
        entries = load_manifest(MANIFEST)
        if args.check:
            verify_directory(OUTPUT, entries)
        else:
            fetch(OUTPUT, entries)
        print(f"verified {len(entries)} pinned decks in {OUTPUT}")
        return 0
    except (OSError, ValueError) as error:
        print(f"pptx corpus error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

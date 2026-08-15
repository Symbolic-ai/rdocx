#!/usr/bin/env python3
"""Build the exact Poppler command-line oracle used by CI."""

from __future__ import annotations

import argparse
import hashlib
import os
from pathlib import Path
import shutil
import stat
import subprocess
import tarfile
import tempfile
import urllib.request


POPLER_VERSION = "26.01.0"
POPLER_SHA256 = "1cb944a4b88847f5fb6551683bc799db59f04990f5d8be07aba2acbf38601089"
POPLER_URL = f"https://poppler.freedesktop.org/poppler-{POPLER_VERSION}.tar.xz"
TOOLS = ("pdftoppm", "pdfinfo", "pdftotext")
MAX_DOWNLOAD_BYTES = 8 * 1024 * 1024
MAX_ARCHIVE_MEMBERS = 2_048
MAX_EXTRACTED_BYTES = 64 * 1024 * 1024


def download_archive(destination: Path) -> None:
    digest = hashlib.sha256()
    written = 0
    request = urllib.request.Request(
        POPLER_URL,
        headers={"User-Agent": "rdocx-ci-poppler-installer/1"},
    )
    with urllib.request.urlopen(request, timeout=60) as response, destination.open(
        "wb"
    ) as output:
        while chunk := response.read(1024 * 1024):
            written += len(chunk)
            if written > MAX_DOWNLOAD_BYTES:
                raise RuntimeError("Poppler archive exceeds the download bound")
            digest.update(chunk)
            output.write(chunk)
    if digest.hexdigest() != POPLER_SHA256:
        raise RuntimeError("Poppler archive SHA-256 does not match the reviewed source")


def safe_extract(archive_path: Path, destination: Path) -> Path:
    destination = destination.resolve()
    destination.mkdir(parents=True)
    member_count = 0
    extracted_bytes = 0
    with tarfile.open(archive_path, mode="r|xz") as archive:
        for member in archive:
            member_count += 1
            if member_count > MAX_ARCHIVE_MEMBERS:
                raise RuntimeError("Poppler archive exceeds the member-count bound")
            if member.size < 0:
                raise RuntimeError("Poppler archive contains a negative member size")
            extracted_bytes += member.size
            if extracted_bytes > MAX_EXTRACTED_BYTES:
                raise RuntimeError("Poppler archive exceeds the extracted-size bound")
            target = (destination / member.name).resolve()
            try:
                target.relative_to(destination)
            except ValueError as error:
                raise RuntimeError("Poppler archive contains an unsafe path") from error
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
                continue
            if not member.isfile():
                raise RuntimeError("Poppler archive contains a non-file entry")
            source = archive.extractfile(member)
            if source is None:
                raise RuntimeError("Poppler archive member could not be read")
            target.parent.mkdir(parents=True, exist_ok=True)
            with source, target.open("wb") as output:
                shutil.copyfileobj(source, output)
            target.chmod(stat.S_IMODE(member.mode))

    source_root = destination / f"poppler-{POPLER_VERSION}"
    if not (source_root / "CMakeLists.txt").is_file():
        raise RuntimeError("Poppler archive has an unexpected root layout")
    return source_root


def run(command: list[str]) -> None:
    subprocess.run(command, check=True)


def verify_tools(prefix: Path) -> None:
    for tool in TOOLS:
        executable = prefix / "bin" / tool
        if not executable.is_file():
            raise RuntimeError(f"missing Poppler tool: {executable}")
        result = subprocess.run(
            [str(executable), "-v"],
            check=True,
            capture_output=True,
            text=True,
        )
        lines = tuple(
            line.strip()
            for line in (result.stdout + result.stderr).splitlines()
            if line.strip()
        )
        expected = f"{tool} version {POPLER_VERSION}"
        if not lines or lines[0] != expected:
            raise RuntimeError(f"unexpected {tool} identity: {lines[:1]}")


def expose_tools(prefix: Path) -> None:
    github_path = os.environ.get("GITHUB_PATH")
    if github_path:
        with Path(github_path).open("a", encoding="utf-8") as path_file:
            path_file.write(f"{prefix / 'bin'}\n")


def build(prefix: Path) -> None:
    prefix = prefix.resolve()
    if prefix.exists() and (not prefix.is_dir() or any(prefix.iterdir())):
        raise RuntimeError(
            f"Poppler prefix must be empty so the reviewed source is rebuilt: {prefix}"
        )

    runner_temp = Path(os.environ.get("RUNNER_TEMP", tempfile.gettempdir()))
    with tempfile.TemporaryDirectory(prefix="rdocx-poppler-", dir=runner_temp) as work:
        work_root = Path(work)
        archive_path = work_root / f"poppler-{POPLER_VERSION}.tar.xz"
        download_archive(archive_path)
        source_root = safe_extract(archive_path, work_root / "source")
        build_root = work_root / "build"
        configure = [
            "cmake",
            "-S",
            str(source_root),
            "-B",
            str(build_root),
            "-G",
            "Ninja",
            "-DCMAKE_BUILD_TYPE=Release",
            "-DENABLE_UTILS=ON",
            "-DENABLE_CPP=OFF",
            "-DENABLE_GLIB=OFF",
            "-DENABLE_GOBJECT_INTROSPECTION=OFF",
            "-DENABLE_QT5=OFF",
            "-DENABLE_QT6=OFF",
            "-DENABLE_BOOST=OFF",
            "-DENABLE_LIBCURL=OFF",
            "-DENABLE_NSS3=OFF",
            "-DENABLE_GPGME=OFF",
            "-DBUILD_GTK_TESTS=OFF",
            "-DBUILD_QT5_TESTS=OFF",
            "-DBUILD_QT6_TESTS=OFF",
            "-DBUILD_CPP_TESTS=OFF",
            "-DBUILD_MANUAL_TESTS=OFF",
            "-DBUILD_SHARED_LIBS=OFF",
            "-DENABLE_UNSTABLE_API_ABI_HEADERS=OFF",
            "-DRUN_GPERF_IF_PRESENT=OFF",
        ]
        run(configure)
        jobs = max(1, min(os.cpu_count() or 1, 4))
        run(
            [
                "cmake",
                "--build",
                str(build_root),
                "--parallel",
                str(jobs),
                "--target",
                *TOOLS,
            ]
        )
        binary_root = prefix / "bin"
        binary_root.mkdir(parents=True, exist_ok=True)
        for tool in TOOLS:
            shutil.copy2(build_root / "utils" / tool, binary_root / tool)

    verify_tools(prefix)
    expose_tools(prefix)
    print(f"Installed Poppler {POPLER_VERSION} at {prefix}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    default_prefix = Path(
        os.environ.get("RUNNER_TEMP", tempfile.gettempdir())
    ) / f"poppler-{POPLER_VERSION}"
    parser.add_argument("--prefix", type=Path, default=default_prefix)
    arguments = parser.parse_args()
    build(arguments.prefix)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Compare decoded pixels rasterised from deterministic sample PDFs."""

from __future__ import annotations

import argparse
import binascii
import hashlib
import json
import shutil
import struct
import subprocess
import sys
import unittest
import zlib
from pathlib import Path
from tempfile import TemporaryDirectory


REPO_ROOT = Path(__file__).resolve().parent.parent
SAMPLES_DIR = REPO_ROOT / "samples"
MANIFEST_PATH = Path(__file__).resolve().with_name("golden_pixel_manifest.json")
SAMPLES = (
    "feature_showcase",
    "proposal",
    "quote",
    "invoice",
    "report",
    "letter",
    "contract",
)
RASTERIZER = "pdftoppm"
DPI = 150
PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def compare_pixels(
    sample: str,
    expected_width: int,
    expected_height: int,
    expected_digest: str,
    actual_width: int,
    actual_height: int,
    actual_rgba: bytes,
) -> list[str]:
    """Return precise dimension and decoded-pixel differences."""
    differences = []
    if (expected_width, expected_height) != (actual_width, actual_height):
        differences.append(
            f"{sample}: dimensions expected {expected_width}x{expected_height}, "
            f"got {actual_width}x{actual_height}"
        )

    actual_digest = sha256(actual_rgba)
    if expected_digest != actual_digest:
        differences.append(
            f"{sample}: decoded RGBA digest expected {expected_digest}, "
            f"got {actual_digest}"
        )
    return differences


def validate_update_reason(reason: str | None) -> str:
    if reason is None or not reason.strip():
        raise ValueError("--update requires a non-empty --reason")
    return reason.strip()


def rasterizer_version() -> str:
    completed = subprocess.run(
        [RASTERIZER, "-v"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    lines = (completed.stdout + completed.stderr).splitlines()
    if not lines:
        raise ValueError(f"{RASTERIZER} did not report a version")
    return lines[0].strip()


def run_sample_generator() -> None:
    for sample in SAMPLES:
        (SAMPLES_DIR / f"{sample}.pdf").unlink(missing_ok=True)

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


def rasterize_page_one(pdf_path: Path, output_prefix: Path) -> Path:
    subprocess.run(
        [
            RASTERIZER,
            "-f",
            "1",
            "-l",
            "1",
            "-singlefile",
            "-r",
            str(DPI),
            "-png",
            str(pdf_path),
            str(output_prefix),
        ],
        cwd=REPO_ROOT,
        check=True,
    )
    png_path = output_prefix.with_suffix(".png")
    if not png_path.is_file():
        raise ValueError(f"{RASTERIZER} did not create {png_path}")
    return png_path


def paeth(left: int, above: int, upper_left: int) -> int:
    estimate = left + above - upper_left
    left_distance = abs(estimate - left)
    above_distance = abs(estimate - above)
    upper_left_distance = abs(estimate - upper_left)
    if left_distance <= above_distance and left_distance <= upper_left_distance:
        return left
    if above_distance <= upper_left_distance:
        return above
    return upper_left


def undo_png_filter(
    filter_type: int, encoded: bytes, prior: bytes, bytes_per_pixel: int
) -> bytes:
    decoded = bytearray(len(encoded))
    for index, value in enumerate(encoded):
        left = decoded[index - bytes_per_pixel] if index >= bytes_per_pixel else 0
        above = prior[index] if prior else 0
        upper_left = (
            prior[index - bytes_per_pixel]
            if prior and index >= bytes_per_pixel
            else 0
        )
        if filter_type == 0:
            predictor = 0
        elif filter_type == 1:
            predictor = left
        elif filter_type == 2:
            predictor = above
        elif filter_type == 3:
            predictor = (left + above) // 2
        elif filter_type == 4:
            predictor = paeth(left, above, upper_left)
        else:
            raise ValueError(f"unsupported PNG filter type {filter_type}")
        decoded[index] = (value + predictor) & 0xFF
    return bytes(decoded)


def decode_png(path: Path) -> tuple[int, int, bytes]:
    """Decode a non-interlaced, 8-bit PNG into RGBA pixels."""
    data = path.read_bytes()
    if not data.startswith(PNG_SIGNATURE):
        raise ValueError(f"{path} is not a PNG")

    offset = len(PNG_SIGNATURE)
    width = height = bit_depth = color_type = interlace = None
    compressed = bytearray()
    while offset < len(data):
        if offset + 12 > len(data):
            raise ValueError(f"{path} has a truncated PNG chunk")
        length = struct.unpack(">I", data[offset : offset + 4])[0]
        chunk_type = data[offset + 4 : offset + 8]
        chunk_start = offset + 8
        chunk_end = chunk_start + length
        if chunk_end + 4 > len(data):
            raise ValueError(f"{path} has a truncated PNG chunk")
        chunk_data = data[chunk_start:chunk_end]
        expected_crc = struct.unpack(">I", data[chunk_end : chunk_end + 4])[0]
        actual_crc = binascii.crc32(chunk_type + chunk_data) & 0xFFFFFFFF
        if expected_crc != actual_crc:
            raise ValueError(f"{path} has a PNG CRC mismatch")

        if chunk_type == b"IHDR":
            if length != 13:
                raise ValueError(f"{path} has an invalid IHDR")
            (
                width,
                height,
                bit_depth,
                color_type,
                compression,
                filter_method,
                interlace,
            ) = struct.unpack(">IIBBBBB", chunk_data)
            if compression != 0 or filter_method != 0:
                raise ValueError(f"{path} uses unsupported PNG compression")
        elif chunk_type == b"IDAT":
            compressed.extend(chunk_data)
        elif chunk_type == b"IEND":
            break
        offset = chunk_end + 4

    if width is None or height is None or bit_depth is None or color_type is None:
        raise ValueError(f"{path} has no PNG header")
    if bit_depth != 8 or interlace != 0:
        raise ValueError(f"{path} must be a non-interlaced 8-bit PNG")

    channels = {0: 1, 2: 3, 4: 2, 6: 4}.get(color_type)
    if channels is None:
        raise ValueError(f"{path} uses unsupported PNG color type {color_type}")
    row_bytes = width * channels
    raw = zlib.decompress(bytes(compressed))
    if len(raw) != height * (row_bytes + 1):
        raise ValueError(f"{path} has an unexpected decoded PNG size")

    rows = []
    prior = b""
    for row_index in range(height):
        row_start = row_index * (row_bytes + 1)
        filter_type = raw[row_start]
        encoded = raw[row_start + 1 : row_start + 1 + row_bytes]
        decoded = undo_png_filter(filter_type, encoded, prior, channels)
        rows.append(decoded)
        prior = decoded

    rgba = bytearray(width * height * 4)
    output = 0
    for row in rows:
        for pixel in range(0, len(row), channels):
            if color_type == 0:
                red = green = blue = row[pixel]
                alpha = 255
            elif color_type == 2:
                red, green, blue = row[pixel : pixel + 3]
                alpha = 255
            elif color_type == 4:
                red = green = blue = row[pixel]
                alpha = row[pixel + 1]
            else:
                red, green, blue, alpha = row[pixel : pixel + 4]
            rgba[output : output + 4] = bytes((red, green, blue, alpha))
            output += 4
    return width, height, bytes(rgba)


def png_chunk(chunk_type: bytes, data: bytes) -> bytes:
    crc = binascii.crc32(chunk_type + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + chunk_type + data + struct.pack(">I", crc)


def write_rgba_png(path: Path, width: int, height: int, rgba: bytes) -> None:
    if len(rgba) != width * height * 4:
        raise ValueError("RGBA buffer length does not match its dimensions")
    rows = b"".join(
        b"\x00" + rgba[row * width * 4 : (row + 1) * width * 4]
        for row in range(height)
    )
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    path.write_bytes(
        PNG_SIGNATURE
        + png_chunk(b"IHDR", header)
        + png_chunk(b"IDAT", zlib.compress(rows))
        + png_chunk(b"IEND", b"")
    )


def inject_one_pixel(source: Path, destination: Path) -> None:
    shutil.copy2(source, destination)
    width, height, rgba = decode_png(destination)
    if not rgba:
        raise ValueError(f"{source} has no pixels to mutate")
    mutated = bytearray(rgba)
    mutated[0] ^= 1
    write_rgba_png(destination, width, height, bytes(mutated))


def collect_pixels(
    output_dir: Path, inject_sample: str | None
) -> dict[str, dict[str, object]]:
    entries = {}
    for sample in SAMPLES:
        png_path = rasterize_page_one(
            SAMPLES_DIR / f"{sample}.pdf", output_dir / sample
        )
        if inject_sample == sample:
            mutated_path = output_dir / f"{sample}-one-pixel-offset.png"
            inject_one_pixel(png_path, mutated_path)
            png_path = mutated_path
        width, height, rgba = decode_png(png_path)
        entries[sample] = {
            "width": width,
            "height": height,
            "rgba": rgba,
        }
    return entries


def manifest_samples(
    entries: dict[str, dict[str, object]],
) -> dict[str, dict[str, object]]:
    return {
        sample: {
            "width": entry["width"],
            "height": entry["height"],
            "rgba_sha256": sha256(bytes(entry["rgba"])),
        }
        for sample, entry in sorted(entries.items())
    }


def build_manifest(
    entries: dict[str, dict[str, object]], version: str, reason: str
) -> dict[str, object]:
    return {
        "dpi": DPI,
        "rasterizer": {"command": RASTERIZER, "version": version},
        "reason": reason,
        "samples": manifest_samples(entries),
    }


def write_manifest(
    path: Path,
    entries: dict[str, dict[str, object]],
    version: str,
    reason: str | None,
) -> None:
    payload = build_manifest(entries, version, validate_update_reason(reason))
    path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


def read_manifest(path: Path) -> dict[str, object]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"{path} does not contain a JSON object")
    if payload.get("dpi") != DPI:
        raise ValueError(f"{path} does not record the fixed {DPI} DPI")
    rasterizer = payload.get("rasterizer")
    if not isinstance(rasterizer, dict) or rasterizer.get("command") != RASTERIZER:
        raise ValueError(f"{path} does not record {RASTERIZER}")
    if not isinstance(rasterizer.get("version"), str):
        raise ValueError(f"{path} does not record the rasterizer version")
    samples = payload.get("samples")
    if not isinstance(samples, dict):
        raise ValueError(f"{path} does not contain a samples object")
    return payload


def compare_manifest(
    expected: dict[str, object], actual_entries: dict[str, dict[str, object]], version: str
) -> list[str]:
    differences = []
    rasterizer = expected["rasterizer"]
    if not isinstance(rasterizer, dict):
        raise ValueError("manifest rasterizer must be an object")
    if rasterizer["version"] != version:
        differences.append(
            f"rasterizer version expected {rasterizer['version']}, got {version}"
        )
    expected_samples = expected["samples"]
    if not isinstance(expected_samples, dict):
        raise ValueError("manifest samples must be an object")
    for sample in sorted(expected_samples.keys() - actual_entries.keys()):
        differences.append(f"{sample}: missing rasterized sample")
    for sample in sorted(actual_entries.keys() - expected_samples.keys()):
        differences.append(f"{sample}: unexpected rasterized sample")
    for sample in sorted(expected_samples.keys() & actual_entries.keys()):
        expected_entry = expected_samples[sample]
        actual_entry = actual_entries[sample]
        if not isinstance(expected_entry, dict):
            raise ValueError(f"manifest entry for {sample} must be an object")
        differences.extend(
            compare_pixels(
                sample,
                int(expected_entry["width"]),
                int(expected_entry["height"]),
                str(expected_entry["rgba_sha256"]),
                int(actual_entry["width"]),
                int(actual_entry["height"]),
                bytes(actual_entry["rgba"]),
            )
        )
    return differences


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="compare with the manifest")
    mode.add_argument("--update", action="store_true", help="replace the manifest")
    mode.add_argument("--self-test", action="store_true", help="run harness unit tests")
    parser.add_argument("--reason", help="required audit reason for --update")
    parser.add_argument(
        "--inject-one-pixel",
        choices=SAMPLES,
        metavar="SAMPLE",
        help="mutate one decoded pixel to prove check mode rejects it",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.self_test:
        if args.reason is not None or args.inject_one_pixel is not None:
            print("golden_png_harness: self-test accepts no other options", file=sys.stderr)
            return 2
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(GoldenPngHarnessTests)
        return 0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1

    try:
        if args.update:
            validate_update_reason(args.reason)
            if args.inject_one_pixel is not None:
                raise ValueError("--inject-one-pixel is only valid with --check")
        elif args.reason is not None:
            raise ValueError("--reason is only valid with --update")

        version = rasterizer_version()
        print(f"golden_png_harness: rasterizer {version}")
        run_sample_generator()
        with TemporaryDirectory(prefix="rdocx-golden-png-") as temp_dir:
            entries = collect_pixels(Path(temp_dir), args.inject_one_pixel)

        if args.update:
            write_manifest(MANIFEST_PATH, entries, version, args.reason)
            print(
                f"golden_png_harness: wrote {len(entries)} samples to "
                f"{MANIFEST_PATH.relative_to(REPO_ROOT)}"
            )
            return 0

        expected = read_manifest(MANIFEST_PATH)
        differences = compare_manifest(expected, entries, version)
        if differences:
            print("golden_png_harness: pixel delta detected", file=sys.stderr)
            for difference in differences:
                print(f"  - {difference}", file=sys.stderr)
            return 1

        print(
            f"golden_png_harness: {len(entries)} page-one pixel buffers match at "
            f"{DPI} DPI"
        )
        return 0
    except (OSError, ValueError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        print(f"golden_png_harness: {error}", file=sys.stderr)
        return 2


class GoldenPngHarnessTests(unittest.TestCase):
    def test_pixel_comparison_reports_dimension_and_digest_changes(self) -> None:
        differences = compare_pixels(
            "proposal", 2, 2, "expected-digest", 3, 2, bytes(3 * 2 * 4)
        )

        self.assertEqual(
            differences,
            [
                "proposal: dimensions expected 2x2, got 3x2",
                "proposal: decoded RGBA digest expected expected-digest, got "
                "9d908ecfb6b256def8b49a7c504e6c889c4b0e41fe6ce3e01863dd7b61a20aa0",
            ],
        )

    def test_update_requires_a_non_empty_reason(self) -> None:
        with TemporaryDirectory() as temp_dir:
            manifest = Path(temp_dir) / "manifest.json"
            with self.assertRaisesRegex(ValueError, "non-empty --reason"):
                write_manifest(manifest, {}, "pdftoppm version test", "  ")
            self.assertFalse(manifest.exists())

    def test_one_pixel_offset_is_rejected(self) -> None:
        with TemporaryDirectory() as temp_dir:
            source = Path(temp_dir) / "source.png"
            mutated = Path(temp_dir) / "mutated.png"
            rgba = bytes((10, 20, 30, 255, 40, 50, 60, 255))
            write_rgba_png(source, 2, 1, rgba)
            inject_one_pixel(source, mutated)

            width, height, mutated_rgba = decode_png(mutated)
            changed_pixels = sum(
                rgba[index : index + 4] != mutated_rgba[index : index + 4]
                for index in range(0, len(rgba), 4)
            )
            differences = compare_pixels(
                "proposal", 2, 1, sha256(rgba), width, height, mutated_rgba
            )

            self.assertEqual(changed_pixels, 1)
            self.assertEqual(len(differences), 1)
            self.assertIn("proposal: decoded RGBA digest", differences[0])

    def test_rasterizer_version_is_recorded(self) -> None:
        payload = build_manifest({}, "pdftoppm version 26.01.0", "reviewed")

        self.assertEqual(payload["dpi"], 150)
        self.assertEqual(
            payload["rasterizer"],
            {"command": "pdftoppm", "version": "pdftoppm version 26.01.0"},
        )


if __name__ == "__main__":
    sys.exit(main())

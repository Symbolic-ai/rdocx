#!/usr/bin/env python3
"""Compare deterministic PresentationML renders with a pinned oracle."""

from __future__ import annotations

import argparse
from concurrent.futures import ProcessPoolExecutor
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys
from tempfile import TemporaryDirectory
import unittest

from fetch_pptx_corpus import EXPECTED_COUNT, load_manifest, verify_directory
from golden_png_harness import decode_png


REPO_ROOT = Path(__file__).resolve().parent.parent
CORPUS_MANIFEST = REPO_ROOT / "scripts" / "pptx-corpus-manifest.tsv"
DEFAULT_CORPUS = REPO_ROOT / "corpus" / "pptx"
SOFFICE = "soffice"
PDFTOPPM = "pdftoppm"
SOFFICE_VERSION = "LibreOffice 26.2.5.2 cd7284b4cbbfeb507e630c1aac019f4157393acb"
PDFTOPPM_VERSION = "pdftoppm version 26.01.0"
SOFFICE_PDF_FILTER = (
    'pdf:impress_pdf_Export:{"ExportHiddenSlides":{"type":"boolean","value":"true"}}'
)
DPI = 150
SSIM_TARGET = 0.95
COVERAGE_TARGET = 0.80
RENDER_MANIFEST_HEADER = (
    "deck\tslide\tsource_shapes\tresolved_shapes\tdropped_shapes"
    "\tdiagnostics\toutput"
)
RESULT_HEADER = "deck\tslide\tssim\trust_png\toracle_png"
EVIDENCE_ENV = "RDOCX_PPTX_GATE_EVIDENCE"


def tool_version(command: str, arguments: list[str]) -> str:
    completed = subprocess.run(
        [command, *arguments],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    lines = (completed.stdout + completed.stderr).splitlines()
    if not lines:
        raise ValueError(f"{command} did not report a version")
    return lines[0].strip()


def assert_tool_versions() -> tuple[str, str]:
    soffice = tool_version(SOFFICE, ["--version"])
    pdftoppm = tool_version(PDFTOPPM, ["-v"])
    if soffice != SOFFICE_VERSION:
        raise ValueError(
            f"LibreOffice version expected {SOFFICE_VERSION!r}, got {soffice!r}"
        )
    if pdftoppm != PDFTOPPM_VERSION:
        raise ValueError(
            f"pdftoppm version expected {PDFTOPPM_VERSION!r}, got {pdftoppm!r}"
        )
    return soffice, pdftoppm


def composite_luminance(rgba: bytes) -> list[int]:
    luminance = []
    for offset in range(0, len(rgba), 4):
        red, green, blue, alpha = rgba[offset : offset + 4]
        red = (red * alpha + 255 * (255 - alpha) + 127) // 255
        green = (green * alpha + 255 * (255 - alpha) + 127) // 255
        blue = (blue * alpha + 255 * (255 - alpha) + 127) // 255
        luminance.append((2126 * red + 7152 * green + 722 * blue + 5000) // 10_000)
    return luminance


def structural_similarity(
    first: tuple[int, int, bytes], second: tuple[int, int, bytes]
) -> float:
    """Return global luminance SSIM after compositing RGBA over white.

    The calculation uses population variance and covariance with the standard
    8-bit constants K1=0.01, K2=0.03, and L=255. A global window is deliberate:
    it makes the metric dependency-free and bit-for-bit reproducible while
    retaining luminance, contrast, and structural terms.
    """
    first_width, first_height, first_rgba = first
    second_width, second_height, second_rgba = second
    if (first_width, first_height) != (second_width, second_height):
        raise ValueError(
            "image dimensions differ: "
            f"{first_width}x{first_height} != {second_width}x{second_height}"
        )
    if first_rgba == second_rgba:
        return 1.0
    first_luma = composite_luminance(first_rgba)
    second_luma = composite_luminance(second_rgba)
    if len(first_luma) != len(second_luma) or not first_luma:
        raise ValueError("decoded images have different or empty pixel buffers")
    count = len(first_luma)
    sum_first = sum(first_luma)
    sum_second = sum(second_luma)
    mean_first = sum_first / count
    mean_second = sum_second / count
    variance_first = sum(value * value for value in first_luma) / count - mean_first**2
    variance_second = (
        sum(value * value for value in second_luma) / count - mean_second**2
    )
    covariance = (
        sum(left * right for left, right in zip(first_luma, second_luma)) / count
        - mean_first * mean_second
    )
    c1 = (0.01 * 255) ** 2
    c2 = (0.03 * 255) ** 2
    numerator = (2 * mean_first * mean_second + c1) * (2 * covariance + c2)
    denominator = (
        mean_first**2 + mean_second**2 + c1
    ) * (variance_first + variance_second + c2)
    score = numerator / denominator
    return max(-1.0, min(1.0, score))


def meets_coverage(scores: list[float]) -> bool:
    if not scores:
        return False
    passing = sum(score >= SSIM_TARGET for score in scores)
    return passing * 100 >= len(scores) * 80


def run_rust_renderer(corpus: Path, output: Path, manifest: Path) -> None:
    subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "-p",
            "rpptx",
            "--example",
            "render_deck",
            "--",
            str(corpus),
            str(output),
            str(manifest),
        ],
        cwd=REPO_ROOT,
        check=True,
    )


def unescape_tsv(value: str) -> str:
    output = []
    index = 0
    replacements = {"t": "\t", "r": "\r", "n": "\n", "\\": "\\"}
    while index < len(value):
        if value[index] == "\\" and index + 1 < len(value):
            escaped = value[index + 1]
            if escaped in replacements:
                output.append(replacements[escaped])
                index += 2
                continue
        output.append(value[index])
        index += 1
    return "".join(output)


def read_render_manifest(path: Path) -> list[dict[str, object]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0] != RENDER_MANIFEST_HEADER:
        raise ValueError(f"{path} has an invalid render manifest header")
    records = []
    for line_number, line in enumerate(lines[1:], 2):
        fields = line.split("\t")
        if len(fields) != 7:
            raise ValueError(f"{path}:{line_number} has {len(fields)} fields")
        deck, slide, source, resolved, dropped, diagnostics, output = fields
        record = {
            "deck": unescape_tsv(deck),
            "slide": int(slide),
            "source_shapes": int(source),
            "resolved_shapes": int(resolved),
            "dropped_shapes": int(dropped),
            "diagnostics": unescape_tsv(diagnostics),
            "output": Path(unescape_tsv(output)),
        }
        if record["source_shapes"] != record["resolved_shapes"]:
            raise ValueError(f"{deck} slide {slide} has a source-to-resolved shape delta")
        if record["dropped_shapes"] != 0:
            raise ValueError(f"{deck} slide {slide} dropped a shape")
        output_path = record["output"]
        if not isinstance(output_path, Path) or not output_path.is_file():
            raise ValueError(f"{deck} slide {slide} has no rendered PNG")
        records.append(record)
    if not records:
        raise ValueError("the renderer emitted no slide records")
    return records


def render_oracle_deck(deck: Path, pdf_dir: Path, png_dir: Path, profile: Path) -> list[Path]:
    subprocess.run(
        [
            SOFFICE,
            "--headless",
            "--norestore",
            "--nodefault",
            "--nolockcheck",
            f"-env:UserInstallation={profile.as_uri()}",
            "--convert-to",
            SOFFICE_PDF_FILTER,
            "--outdir",
            str(pdf_dir),
            str(deck),
        ],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
        timeout=180,
    )
    pdf = pdf_dir / f"{deck.stem}.pdf"
    if not pdf.is_file():
        raise ValueError(f"LibreOffice did not create {pdf}")
    prefix = png_dir / deck.name
    subprocess.run(
        [PDFTOPPM, "-r", str(DPI), "-png", str(pdf), str(prefix)],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
        timeout=180,
    )
    pages = list(png_dir.glob(f"{deck.name}-*.png"))
    pages.sort(key=lambda path: int(path.stem.rsplit("-", 1)[1]))
    if not pages:
        raise ValueError(f"pdftoppm did not rasterize {pdf}")
    return pages


def write_results(
    path: Path, records: list[dict[str, object]], oracle_pages: dict[str, list[Path]]
) -> list[float]:
    pairs = []
    per_deck_index: dict[str, int] = {}
    for record in records:
        deck = str(record["deck"])
        index = per_deck_index.get(deck, 0)
        pages = oracle_pages[deck]
        if index >= len(pages):
            raise ValueError(f"{deck} is missing oracle slide {index + 1}")
        rust_png = record["output"]
        if not isinstance(rust_png, Path):
            raise ValueError(f"{deck} slide {index + 1} has an invalid renderer path")
        pairs.append((deck, index + 1, rust_png, pages[index]))
        per_deck_index[deck] = index + 1
    for deck, pages in oracle_pages.items():
        if per_deck_index.get(deck, 0) != len(pages):
            raise ValueError(
                f"{deck} rendered {per_deck_index.get(deck, 0)} Rust pages and "
                f"{len(pages)} oracle pages"
            )
    workers = min(8, os.cpu_count() or 1)
    with ProcessPoolExecutor(max_workers=workers) as executor:
        scores = list(executor.map(score_png_pair, pairs))
    rows = [RESULT_HEADER]
    rows.extend(
        f"{deck}\t{slide}\t{score:.9f}\t{rust_png}\t{oracle_png}"
        for (deck, slide, rust_png, oracle_png), score in zip(pairs, scores)
    )
    path.write_text("\n".join(rows) + "\n", encoding="utf-8")
    return scores


def score_png_pair(pair: tuple[str, int, Path, Path]) -> float:
    _, _, rust_png, oracle_png = pair
    return structural_similarity(decode_png(rust_png), decode_png(oracle_png))


def evidence_payload(
    records: list[dict[str, object]], scores: list[float], results: Path
) -> dict[str, object]:
    ordered = sorted(scores)
    passing = sum(score >= SSIM_TARGET for score in scores)
    decks = len({str(record["deck"]) for record in records})
    return {
        "decks": decks,
        "slides": len(records),
        "dropped_shapes": sum(int(record["dropped_shapes"]) for record in records),
        "minimum": ordered[0],
        "median": ordered[len(ordered) // 2],
        "maximum": ordered[-1],
        "passing": passing,
        "coverage": passing / len(scores),
        "target": SSIM_TARGET,
        "coverage_target": COVERAGE_TARGET,
        "trend_target_met": meets_coverage(scores),
        "results": str(results),
    }


def run_gate(corpus: Path, output: Path) -> tuple[dict[str, object], Path]:
    entries = load_manifest(CORPUS_MANIFEST)
    if len(entries) != EXPECTED_COUNT:
        raise ValueError(f"corpus has {len(entries)} decks, expected {EXPECTED_COUNT}")
    verify_directory(corpus, entries)
    soffice_version, pdftoppm_version = assert_tool_versions()
    rust_dir = output / "rust"
    oracle_pdf_dir = output / "oracle-pdf"
    oracle_png_dir = output / "oracle-png"
    profile = output / "libreoffice-profile"
    for directory in [rust_dir, oracle_pdf_dir, oracle_png_dir, profile]:
        directory.mkdir(parents=True, exist_ok=True)
    render_manifest = output / "render-manifest.tsv"
    run_rust_renderer(corpus, rust_dir, render_manifest)
    records = read_render_manifest(render_manifest)
    oracle_pages = {}
    for relative_path, _, _, _ in entries:
        deck = corpus / relative_path
        oracle_pages[relative_path] = render_oracle_deck(
            deck, oracle_pdf_dir, oracle_png_dir, profile
        )
    results = output / "ssim-results.tsv"
    scores = write_results(results, records, oracle_pages)
    payload = evidence_payload(records, scores, results)
    payload["libreoffice"] = soffice_version
    payload["pdftoppm"] = pdftoppm_version
    if payload["decks"] != EXPECTED_COUNT:
        raise ValueError(f"rendered {payload['decks']} decks, expected {EXPECTED_COUNT}")
    if payload["dropped_shapes"] != 0:
        raise ValueError(f"rendered with {payload['dropped_shapes']} dropped shapes")
    evidence = output / "gate-evidence.json"
    evidence.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return payload, evidence


def run_suite(test_names: list[str] | None = None) -> bool:
    loader = unittest.defaultTestLoader
    suite = (
        loader.loadTestsFromNames(test_names, module=sys.modules[__name__])
        if test_names
        else loader.loadTestsFromTestCase(PptxSsimHarnessTests)
    )
    return unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful()


class PptxSsimHarnessTests(unittest.TestCase):
    def test_identical_images_have_ssim_one(self) -> None:
        image = (2, 1, bytes((10, 20, 30, 255, 40, 50, 60, 128)))
        self.assertEqual(structural_similarity(image, image), 1.0)

    def test_pixel_changes_lower_ssim(self) -> None:
        first = (2, 1, bytes((0, 0, 0, 255, 255, 255, 255, 255)))
        second = (2, 1, bytes((0, 0, 0, 255, 200, 200, 200, 255)))
        score = structural_similarity(first, second)
        self.assertTrue(math.isclose(score, 0.943293398, abs_tol=0.000000001))
        self.assertLess(score, 1.0)

    def test_dimension_mismatch_is_a_hard_failure(self) -> None:
        with self.assertRaisesRegex(ValueError, "image dimensions differ"):
            structural_similarity((1, 1, bytes(4)), (2, 1, bytes(8)))

    def test_trend_target_classifies_eighty_percent(self) -> None:
        self.assertTrue(meets_coverage([0.95] * 4 + [0.94]))
        self.assertFalse(meets_coverage([0.95] * 3 + [0.94] * 2))

    def test_missed_ssim_trend_is_recorded_without_changing_completeness(self) -> None:
        payload = evidence_payload(
            [
                {
                    "deck": "representative.pptx",
                    "dropped_shapes": 0,
                }
            ],
            [0.94],
            Path("ssim-results.tsv"),
        )

        self.assertFalse(payload["trend_target_met"])
        self.assertEqual(payload["dropped_shapes"], 0)

    def test_hidden_slides_are_included_in_oracle_export(self) -> None:
        self.assertEqual(
            SOFFICE_PDF_FILTER,
            'pdf:impress_pdf_Export:{"ExportHiddenSlides":{"type":"boolean","value":"true"}}',
        )

    def test_oracle_tool_versions_are_exactly_pinned(self) -> None:
        self.assertEqual(
            SOFFICE_VERSION,
            "LibreOffice 26.2.5.2 cd7284b4cbbfeb507e630c1aac019f4157393acb",
        )
        self.assertEqual(PDFTOPPM_VERSION, "pdftoppm version 26.01.0")

    def test_all_corpus_slides_render_without_panic_or_dropped_shape(self) -> None:
        evidence = os.environ.get(EVIDENCE_ENV)
        if evidence is None:
            self.skipTest("requires corpus gate evidence")
        payload = json.loads(Path(evidence).read_text(encoding="utf-8"))
        self.assertEqual(payload["decks"], EXPECTED_COUNT)
        self.assertGreater(payload["slides"], 0)
        self.assertEqual(payload["dropped_shapes"], 0)

    def test_corpus_render_fidelity_records_ssim_trend(self) -> None:
        evidence = os.environ.get(EVIDENCE_ENV)
        if evidence is None:
            self.skipTest("requires corpus gate evidence")
        payload = json.loads(Path(evidence).read_text(encoding="utf-8"))
        self.assertEqual(payload["target"], SSIM_TARGET)
        self.assertEqual(payload["coverage_target"], COVERAGE_TARGET)
        self.assertEqual(
            payload["trend_target_met"], payload["coverage"] >= COVERAGE_TARGET
        )


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--self-test", action="store_true")
    mode.add_argument("--check", action="store_true")
    parser.add_argument(
        "--corpus-dir",
        type=Path,
        default=Path(os.environ.get("RDOCX_PPTX_CORPUS_DIR", DEFAULT_CORPUS)),
    )
    parser.add_argument("--output-dir", type=Path)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.self_test:
        return 0 if run_suite() else 1
    if not run_suite():
        return 1
    temporary = None
    try:
        if args.output_dir is None:
            temporary = TemporaryDirectory(prefix="rpptx-ssim-")
            output = Path(temporary.name)
        else:
            output = args.output_dir.resolve()
            if output.exists() and any(output.iterdir()):
                raise ValueError(f"output directory is not empty: {output}")
            output.mkdir(parents=True, exist_ok=True)
        payload, evidence = run_gate(args.corpus_dir.resolve(), output)
        os.environ[EVIDENCE_ENV] = str(evidence)
        external = [
            "PptxSsimHarnessTests.test_all_corpus_slides_render_without_panic_or_dropped_shape",
            "PptxSsimHarnessTests.test_corpus_render_fidelity_records_ssim_trend",
        ]
        if not run_suite(external):
            return 1
        print(
            "pptx_ssim_harness: SSIM trend "
            f"{payload['passing']}/{payload['slides']} slides at SSIM >= {SSIM_TARGET:.2f} "
            f"({payload['coverage']:.3%}), min {payload['minimum']:.6f}, "
            f"median {payload['median']:.6f}, max {payload['maximum']:.6f}, "
            f"target met {str(payload['trend_target_met']).lower()}"
        )
        print(f"pptx_ssim_harness: results {payload['results']}")
        return 0
    except (OSError, ValueError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
        print(f"pptx_ssim_harness: {error}", file=sys.stderr)
        return 2
    finally:
        if temporary is not None:
            temporary.cleanup()


if __name__ == "__main__":
    raise SystemExit(main())

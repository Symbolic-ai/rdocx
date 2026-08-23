#!/usr/bin/env python3
"""Output-stability hash harness for the generated rdocx samples.

The harness regenerates the seven named sample documents, hashes selected OOXML
parts, a deterministic page-one PNG and a three-part fingerprint of the
deterministic PDF, then compares those values with the checked-in baseline.

PDF is a first-class output written by a different code path from the PNG, so it
is fingerprinted directly. Two of the three entries hash inflated bytes, which
says what moved and survives a change of Deflate implementation. The third is
the file digest, which says that something moved and cannot be evaded.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import unittest
import zipfile
import zlib
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
# Three PDF entries per sample alongside the three XML parts and the PNG.
PDF_PARTS = ("pages", "resources", "bytes")
EXPECTED_ENTRY_COUNT = len(SAMPLES) * (len(OOXML_PARTS) + 1 + len(PDF_PARTS))


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


OBJECT_RE = re.compile(rb"(?m)^(\d+) 0 obj\b")
STREAM_RE = re.compile(rb"\r?\nstream\r?\n")
LENGTH_RE = re.compile(rb"/Length\s+(\d+)\b")
FILTER_RE = re.compile(rb"/Filter\s*(/\w+|\[[^\]]*\])")
REFERENCE_RE = re.compile(rb"(\d+) 0 R\b")
MEDIA_BOX_RE = re.compile(rb"/MediaBox\s*\[([^\]]*)\]")
KIDS_RE = re.compile(rb"/Kids\s*\[([^\]]*)\]")
PARENT_RE = re.compile(rb"/Parent\s+(\d+) 0 R\b")
CONTENTS_ARRAY_RE = re.compile(rb"/Contents\s*\[([^\]]*)\]")
CONTENTS_REF_RE = re.compile(rb"/Contents\s+(\d+) 0 R\b")
ROOT_RE = re.compile(rb"/Root\s+(\d+) 0 R\b")
PAGES_RE = re.compile(rb"/Pages\s+(\d+) 0 R\b")
TYPE_METADATA_RE = re.compile(rb"/Type\s*/Metadata\b")


class PdfError(ValueError):
    """The scanner met something it does not understand.

    Raised rather than skipped. A harness that quietly ignores an object it
    cannot read reports green for the wrong reason, which is the failure this
    story exists to remove.
    """


def parse_pdf_objects(data: bytes) -> dict[int, tuple[bytes, bytes | None]]:
    """Map each object number to its body text and its raw stream payload.

    A scanner over the object syntax, not a general PDF reader. It is enough
    because `to_pdf_deterministic` writes a classic cross reference table with
    no object streams, no `/CreationDate` and no `/ID`, which is also what makes
    the byte digest stable enough to record.
    """
    objects: dict[int, tuple[bytes, bytes | None]] = {}
    for match in OBJECT_RE.finditer(data):
        number = int(match.group(1))

        # A stream payload is arbitrary compressed bytes, so it can contain
        # `endobj` by chance. Take the payload by its declared `/Length` and
        # then look for `endobj` beyond it, rather than searching through it.
        stream = STREAM_RE.search(data, match.end())
        end = data.find(b"endobj", match.end())
        if end == -1:
            raise PdfError(f"object {number} has no endobj")

        payload = None
        if stream is not None and stream.start() < end:
            body = data[match.end() : stream.start()]
            declared = LENGTH_RE.search(body)
            if declared is None:
                raise PdfError(f"object {number} has a stream with no /Length")
            payload = data[stream.end() : stream.end() + int(declared.group(1))]
            after = data[stream.end() + len(payload) :].lstrip(b"\r\n")
            if not after.startswith(b"endstream"):
                raise PdfError(
                    f"object {number} does not end its stream where /Length says"
                )
            end = data.find(b"endobj", stream.end() + len(payload))
            if end == -1:
                raise PdfError(f"object {number} has no endobj")
        else:
            body = data[match.end() : end]

        if number in objects:
            raise PdfError(f"object {number} is defined twice")
        objects[number] = (body, payload)
    if not objects:
        raise PdfError("no objects found, this is not a PDF this scanner reads")
    return objects


def decoded_stream(number: int, objects: dict[int, tuple[bytes, bytes | None]]) -> bytes:
    """The stream of one object, inflated when it is Deflate compressed.

    Hashing the inflated bytes is what makes the structural entries survive a
    change of compressor or compression level, and what makes them move when
    the glyph, subset or image data actually changes.
    """
    body, payload = objects[number]
    if payload is None:
        raise PdfError(f"object {number} was expected to hold a stream")
    declared = FILTER_RE.search(body)
    if declared is None:
        return payload
    # The parsed value, not a substring test. A filter chain contains
    # `/FlateDecode` and is not `/FlateDecode`, and inflating it anyway reports
    # a corrupt stream when the scanner is what is out of its depth.
    if declared.group(1) != b"/FlateDecode":
        raise PdfError(
            f"object {number} uses {declared.group(1).decode('ascii', 'replace')}, "
            "a filter this scanner does not read"
        )
    try:
        return zlib.decompress(payload)
    except zlib.error as error:
        raise PdfError(f"object {number} did not inflate: {error}") from error


def page_numbers_in_order(
    root: int, objects: dict[int, tuple[bytes, bytes | None]]
) -> list[int]:
    """Every page object, in `/Kids` order rather than object-number order.

    Object numbering is an implementation detail of the writer. The page tree
    is the document's own statement of what order its pages are in.
    """
    pages: list[int] = []
    queue = [root]
    seen: set[int] = set()
    while queue:
        number = queue.pop(0)
        if number in seen:
            raise PdfError(f"the page tree revisits object {number}")
        seen.add(number)
        body, _ = objects[number]
        kids = KIDS_RE.search(body)
        if kids is None:
            pages.append(number)
            continue
        children = [int(ref) for ref in REFERENCE_RE.findall(kids.group(1))]
        queue = children + queue
    if not pages:
        raise PdfError("the page tree holds no pages")
    return pages


def media_box_of(number: int, objects: dict[int, tuple[bytes, bytes | None]]) -> str:
    """A page's `/MediaBox`, inherited from its parents when it has none."""
    seen = 0
    current = number
    while seen < len(objects):
        body, _ = objects[current]
        box = MEDIA_BOX_RE.search(body)
        if box is not None:
            return b" ".join(box.group(1).split()).decode("ascii")
        parent = PARENT_RE.search(body)
        if parent is None:
            raise PdfError(f"page {number} has no MediaBox and no parent holding one")
        current = int(parent.group(1))
        seen += 1
    raise PdfError(f"page {number} has a parent chain that does not terminate")


def content_numbers_of(
    number: int, objects: dict[int, tuple[bytes, bytes | None]]
) -> list[int]:
    body, _ = objects[number]
    array = CONTENTS_ARRAY_RE.search(body)
    if array is not None:
        return [int(ref) for ref in REFERENCE_RE.findall(array.group(1))]
    single = CONTENTS_REF_RE.search(body)
    if single is None:
        raise PdfError(f"page {number} has no contents")
    return [int(single.group(1))]


def pdf_fingerprint(data: bytes) -> dict[str, str]:
    """Page geometry, inflated content streams and inflated resource streams.

    Three values. `pages` and `resources` say what moved and survive a change of
    Deflate implementation, because they hash inflated bytes. `bytes` says that
    something moved and cannot be evaded, including by a change that is purely
    in compression.
    """
    objects = parse_pdf_objects(data)

    # `/Root` is a trailer key. Searching the whole file would accept it from an
    # object dictionary, or from bytes inside a compressed payload.
    trailer_at = data.rfind(b"trailer")
    if trailer_at == -1:
        raise PdfError("no trailer, this is not a PDF this scanner reads")
    root = ROOT_RE.search(data, trailer_at)
    if root is None:
        raise PdfError("no /Root in the trailer")
    catalog, _ = objects[int(root.group(1))]
    pages_ref = PAGES_RE.search(catalog)
    if pages_ref is None:
        raise PdfError("the catalog names no page tree")

    page_numbers = page_numbers_in_order(int(pages_ref.group(1)), objects)

    described = [f"pages {len(page_numbers)}"]
    content_streams: set[int] = set()
    for index, number in enumerate(page_numbers):
        contents = content_numbers_of(number, objects)
        content_streams.update(contents)
        inflated = b"".join(decoded_stream(ref, objects) for ref in contents)
        described.append(f"{index} mediabox {media_box_of(number, objects)}")
        described.append(f"{index} content {sha256(inflated)}")

    resources = sorted(
        sha256(decoded_stream(number, objects))
        for number, (body, payload) in objects.items()
        if payload is not None
        and number not in content_streams
        and TYPE_METADATA_RE.search(body) is None
    )

    return {
        "pages": sha256("\n".join(described).encode("utf-8")),
        "resources": sha256("\n".join([f"streams {len(resources)}", *resources]).encode("utf-8")),
        "bytes": sha256(data),
    }


def run_sample_generator() -> None:
    for sample in SAMPLES:
        for extension in ("docx", "png", "pdf"):
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

        # A missing PDF is an error rather than an absent entry. `None` means
        # "this optional XML part is absent by design", and a sample whose PDF
        # failed to generate is not that.
        pdf_path = samples_dir / f"{sample}.pdf"
        if not pdf_path.is_file():
            raise ValueError(f"{pdf_path.name} was not generated")
        for part, digest in pdf_fingerprint(pdf_path.read_bytes()).items():
            hashes[f"{sample}:pdf/{part}"] = digest

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


def build_pdf(content: bytes, resource: bytes, level: int = 6) -> bytes:
    """A minimal one-page PDF, constructed here rather than checked in.

    Real enough for the scanner: a catalog, a page tree, one page with a
    MediaBox and a Flate content stream, and one Flate resource stream standing
    in for an embedded font subset. The cross reference table is deliberately
    not written, because the scanner does not read one and a test that supplied
    a fake would be testing the fake.
    """
    parts = [b"%PDF-1.7\n"]
    parts.append(b"1 0 obj\n<<\n  /Type /Catalog\n  /Pages 2 0 R\n>>\nendobj\n")
    parts.append(b"2 0 obj\n<<\n  /Type /Pages\n  /Kids [3 0 R]\n  /Count 1\n>>\nendobj\n")
    parts.append(
        b"3 0 obj\n<<\n  /Type /Page\n  /Parent 2 0 R\n"
        b"  /MediaBox [0 0 612 792]\n  /Contents 4 0 R\n"
        b"  /Resources <<\n    /Font <<\n      /F0 5 0 R\n    >>\n  >>\n>>\nendobj\n"
    )
    for number, payload in ((4, content), (5, resource)):
        compressed = zlib.compress(payload, level)
        parts.append(
            f"{number} 0 obj\n<<\n  /Length {len(compressed)}\n"
            "  /Filter /FlateDecode\n>>\nstream\n".encode("ascii")
            + compressed
            + b"\nendstream\nendobj\n"
        )
    parts.append(b"trailer\n<<\n  /Size 6\n  /Root 1 0 R\n>>\n%%EOF")
    return b"".join(parts)


class PdfFingerprintTests(unittest.TestCase):
    def test_a_changed_content_stream_moves_the_pdf_entries_and_no_other(self) -> None:
        before = pdf_fingerprint(build_pdf(b"BT /F0 12 Tf 72 720 Td (One) Tj ET", b"subset"))
        after = pdf_fingerprint(build_pdf(b"BT /F0 12 Tf 72 700 Td (One) Tj ET", b"subset"))

        self.assertNotEqual(before["pages"], after["pages"])
        self.assertNotEqual(before["bytes"], after["bytes"])
        self.assertEqual(before["resources"], after["resources"])

    def test_a_changed_resource_stream_moves_only_the_resource_entries(self) -> None:
        before = pdf_fingerprint(build_pdf(b"BT ET", b"subset one"))
        after = pdf_fingerprint(build_pdf(b"BT ET", b"subset two"))

        self.assertNotEqual(before["resources"], after["resources"])
        self.assertNotEqual(before["bytes"], after["bytes"])
        self.assertEqual(before["pages"], after["pages"])

    def test_a_metadata_stream_moves_bytes_but_not_page_resources(self) -> None:
        before_bytes = build_pdf(b"BT ET", b"subset")
        metadata = b"<x:xmpmeta>PDF/UA metadata</x:xmpmeta>"
        metadata_object = (
            f"6 0 obj\n<<\n  /Type /Metadata\n  /Subtype /XML\n"
            f"  /Length {len(metadata)}\n>>\nstream\n".encode("ascii")
            + metadata
            + b"\nendstream\nendobj\n"
        )
        after_bytes = before_bytes.replace(
            b"  /Pages 2 0 R\n",
            b"  /Pages 2 0 R\n  /Metadata 6 0 R\n",
        ).replace(b"trailer\n", metadata_object + b"trailer\n")
        before = pdf_fingerprint(before_bytes)
        after = pdf_fingerprint(after_bytes)

        self.assertEqual(before["pages"], after["pages"])
        self.assertEqual(before["resources"], after["resources"])
        self.assertNotEqual(before["bytes"], after["bytes"])

    def test_refingerprinting_identical_bytes_reproduces_every_entry(self) -> None:
        data = build_pdf(b"BT /F0 12 Tf 72 720 Td (One) Tj ET", b"subset")

        self.assertEqual(pdf_fingerprint(data), pdf_fingerprint(bytes(data)))

    def test_recompressing_the_same_content_leaves_the_structural_entries_still(
        self,
    ) -> None:
        # The division of labour, stated as a test. The structural pair hashes
        # inflated bytes and survives a change of compression level. The byte
        # digest does not, which is what makes it the entry nothing can evade.
        low = pdf_fingerprint(build_pdf(b"BT ET", b"subset", level=1))
        high = pdf_fingerprint(build_pdf(b"BT ET", b"subset", level=9))

        self.assertEqual(low["pages"], high["pages"])
        self.assertEqual(low["resources"], high["resources"])
        self.assertNotEqual(low["bytes"], high["bytes"])

    def test_a_page_geometry_change_moves_the_pages_entry(self) -> None:
        letter = build_pdf(b"BT ET", b"subset")
        a4 = letter.replace(b"/MediaBox [0 0 612 792]", b"/MediaBox [0 0 595 842]")

        self.assertNotEqual(pdf_fingerprint(letter)["pages"], pdf_fingerprint(a4)["pages"])

    def test_a_payload_that_looks_like_pdf_syntax_does_not_confuse_the_scanner(
        self,
    ) -> None:
        # A compressed stream is arbitrary bytes, so it can contain `endobj` or
        # `/Root n 0 R` by chance. Taking the payload by its declared /Length
        # and reading /Root from the trailer is what makes that a non-event.
        hostile = b"endobj\ntrailer\n/Root 9 0 R\nstream\n" * 4
        fingerprint = pdf_fingerprint(build_pdf(b"BT ET", hostile))
        benign = pdf_fingerprint(build_pdf(b"BT ET", b"subset"))

        self.assertEqual(fingerprint["pages"], benign["pages"])
        self.assertNotEqual(fingerprint["resources"], benign["resources"])

    def test_a_filter_chain_is_refused_rather_than_inflated(self) -> None:
        chained = build_pdf(b"BT ET", b"subset").replace(
            b"/Filter /FlateDecode", b"/Filter [/FlateDecode /ASCIIHexDecode]", 1
        )

        with self.assertRaisesRegex(PdfError, "a filter this scanner does not read"):
            pdf_fingerprint(chained)

    def test_an_unparseable_pdf_is_an_error_rather_than_an_absent_entry(self) -> None:
        with self.assertRaisesRegex(PdfError, "not a PDF this scanner reads"):
            pdf_fingerprint(b"%PDF-1.7\nnot really\n%%EOF")

        rootless = build_pdf(b"BT ET", b"subset").replace(b"/Root 1 0 R", b"/Size 6")
        with self.assertRaisesRegex(PdfError, "no /Root"):
            pdf_fingerprint(rootless)

        unknown_filter = build_pdf(b"BT ET", b"subset").replace(
            b"/Filter /FlateDecode", b"/Filter /LZWDecode", 1
        )
        with self.assertRaisesRegex(PdfError, "filter this scanner does not read"):
            pdf_fingerprint(unknown_filter)

    def test_a_missing_pdf_is_an_error_rather_than_an_absent_entry(self) -> None:
        # `None` means "this optional XML part is absent by design". A sample
        # whose PDF failed to generate is not that, so it must not be recorded
        # the same way. The generator prints a message and carries on when PDF
        # rendering fails, so this is reachable.
        with TemporaryDirectory() as temp_dir:
            samples = Path(temp_dir)
            for sample in SAMPLES:
                with zipfile.ZipFile(samples / f"{sample}.docx", "w") as package:
                    for part in OOXML_PARTS:
                        package.writestr(part, f"<{sample}/>")
                (samples / f"{sample}.png").write_bytes(b"not really a png")

            with self.assertRaisesRegex(ValueError, "was not generated"):
                collect_hashes(samples)


if __name__ == "__main__":
    sys.exit(main())

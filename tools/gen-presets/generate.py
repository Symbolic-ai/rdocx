#!/usr/bin/env python3
"""Generate the checked-in DrawingML preset geometry lookup."""

from __future__ import annotations

import argparse
import copy
import hashlib
import sys
import xml.etree.ElementTree as ET
import zipfile
from xml.parsers import expat
from pathlib import Path


EXPECTED_SOURCE_SHA256 = (
    "2f7c868d857c1e3c4b5a6068759fe0e07d77ad58377a6618d1b02ba3507b6939"
)
EXPECTED_DEFINITION_COUNT = 187
EXPECTED_CORPUS_DECKS = 50
DRAWINGML_NS = "http://schemas.openxmlformats.org/drawingml/2006/main"
SOURCE_URL = (
    "https://ecma-international.org/wp-content/uploads/"
    "ECMA-376-1_5th_edition_december_2016.zip"
)

TOOL_DIR = Path(__file__).resolve().parent
WORKSPACE = TOOL_DIR.parents[1]
SOURCE = TOOL_DIR / "presetShapeDefinitions.xml"
LICENSE = TOOL_DIR / "LICENSE-ECMA.txt"
OUTPUT = WORKSPACE / "crates/oxml-drawing/src/preset_shape_data.rs"


def fail(message: str) -> None:
    raise SystemExit(message)


def local_name(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def remove_formatting_whitespace(element: ET.Element) -> None:
    for node in element.iter():
        if node.text is not None and not node.text.strip():
            node.text = None
        if node.tail is not None and not node.tail.strip():
            node.tail = None


def direct_definition_source_bytes(source: bytes) -> list[tuple[str, bytes]]:
    parser = expat.ParserCreate(namespace_separator="}")
    depth = 0
    start_index: int | None = None
    current_name: str | None = None
    definitions: list[tuple[str, bytes]] = []

    def start_element(name: str, _attributes: dict[str, str]) -> None:
        nonlocal current_name, depth, start_index
        if depth == 1:
            current_name = local_name(name)
            start_index = parser.CurrentByteIndex
        depth += 1

    def end_element(_name: str) -> None:
        nonlocal current_name, depth, start_index
        depth -= 1
        if depth != 1 or current_name is None or start_index is None:
            return
        close_end = source.find(b">", parser.CurrentByteIndex)
        if close_end == -1:
            fail(f"unterminated source definition: {current_name}")
        definitions.append((current_name, source[start_index : close_end + 1]))
        current_name = None
        start_index = None

    parser.StartElementHandler = start_element
    parser.EndElementHandler = end_element
    parser.Parse(source, True)
    return definitions


def load_definitions() -> dict[str, bytes]:
    source = SOURCE.read_bytes()
    digest = hashlib.sha256(source).hexdigest()
    if digest != EXPECTED_SOURCE_SHA256:
        fail(
            f"source SHA-256 mismatch: expected {EXPECTED_SOURCE_SHA256}, got {digest}"
        )

    root = ET.fromstring(source.decode("utf-8-sig"))
    direct_definitions = list(root)
    if len(direct_definitions) != EXPECTED_DEFINITION_COUNT:
        fail(
            "source definition count mismatch: "
            f"expected {EXPECTED_DEFINITION_COUNT}, got {len(direct_definitions)}"
        )

    source_slices = direct_definition_source_bytes(source)
    if len(source_slices) != len(direct_definitions):
        fail("could not map every direct definition to its exact source bytes")

    definitions: dict[str, bytes] = {}
    source_definitions: dict[str, bytes] = {}
    ET.register_namespace("a", DRAWINGML_NS)

    for definition, (source_name, source_definition) in zip(
        direct_definitions, source_slices, strict=True
    ):
        name = local_name(definition.tag)
        if source_name != name:
            fail(f"source definition order mismatch: expected {name}, got {source_name}")
        if name in source_definitions:
            if source_definitions[name] != source_definition:
                fail(f"conflicting duplicate preset shape definition: {name}")
            continue
        source_definitions[name] = source_definition
        custom_geometry = ET.Element(f"{{{DRAWINGML_NS}}}custGeom")
        for child in list(definition):
            custom_geometry.append(copy.deepcopy(child))
        remove_formatting_whitespace(custom_geometry)
        definitions[name] = ET.tostring(
            custom_geometry,
            encoding="utf-8",
            short_empty_elements=True,
        )

    expected_unique = EXPECTED_DEFINITION_COUNT - 1
    if len(definitions) != expected_unique:
        fail(
            "unique source name count mismatch: "
            f"expected {expected_unique}, got {len(definitions)}"
        )
    return definitions


def rust_byte_literal(xml: bytes) -> str:
    text = xml.decode("utf-8")
    for count in range(1, 10):
        hashes = "#" * count
        if f'"{hashes}' not in text:
            return f'br{hashes}"{text}"{hashes}'
    fail("could not choose a Rust raw byte string delimiter")


def render_table(definitions: dict[str, bytes]) -> bytes:
    licence = LICENSE.read_text(encoding="utf-8").rstrip().splitlines()
    required_notice = (
        "Copyright (c) 2016 Ecma International",
        "Redistribution and use in source and binary forms",
        "Neither the name of the authors nor Ecma International",
        'THIS SOFTWARE IS PROVIDED BY ECMA INTERNATIONAL "AS IS"',
    )
    licence_text = "\n".join(licence)
    for phrase in required_notice:
        if phrase not in licence_text:
            fail(f"Ecma licence notice is missing required text: {phrase}")

    lines = [
        "// @generated by tools/gen-presets/generate.py. Do not edit.",
        f"// Source: {SOURCE_URL}",
        "// Inner path: OfficeOpenXML-DrawingMLGeometries.zip/"
        "presetShapeDefinitions.xml",
        f"// Source SHA-256: {EXPECTED_SOURCE_SHA256}",
        "//",
        "// Third-party licence notice:",
    ]
    lines.extend(f"// {line}" if line else "//" for line in licence)
    lines.extend(
        [
            "",
            "#[allow(dead_code)]",
            "#[rustfmt::skip]",
            "pub(crate) fn preset_shape_definition(name: &str) -> Option<&'static [u8]> {",
            "    match name {",
        ]
    )
    for name in sorted(definitions):
        literal = rust_byte_literal(definitions[name])
        lines.append(f'        "{name}" => Some({literal}),')
    lines.extend(["        _ => None,", "    }", "}", ""])
    return "\n".join(lines).encode("utf-8")


def scan_corpus(corpus: Path, definitions: dict[str, bytes]) -> tuple[int, int, int]:
    decks = sorted(corpus.rglob("*.pptx"))
    if len(decks) != EXPECTED_CORPUS_DECKS:
        fail(
            f"expected {EXPECTED_CORPUS_DECKS} corpus decks under {corpus}, "
            f"found {len(decks)}"
        )

    used: set[str] = set()
    uses = 0
    for deck in decks:
        try:
            package = zipfile.ZipFile(deck)
        except zipfile.BadZipFile as error:
            fail(f"invalid pptx corpus deck {deck}: {error}")
        with package:
            for member in sorted(package.namelist()):
                if not member.lower().endswith(".xml"):
                    continue
                try:
                    root = ET.fromstring(package.read(member))
                except ET.ParseError as error:
                    fail(f"invalid XML in {deck}:{member}: {error}")
                for element in root.iter():
                    if element.tag != f"{{{DRAWINGML_NS}}}prstGeom":
                        continue
                    name = element.get("prst")
                    if name:
                        used.add(name)
                        uses += 1

    missing = sorted(used.difference(definitions))
    if missing:
        fail("corpus presets missing from generated table: " + ", ".join(missing))
    return len(decks), uses, len(used)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="compare fresh output byte for byte with the checked-in table",
    )
    parser.add_argument(
        "--corpus",
        type=Path,
        help="scan every pptx below this directory for preset names",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    definitions = load_definitions()
    generated = render_table(definitions)

    if args.check:
        if not OUTPUT.is_file():
            fail(f"generated table is missing: {OUTPUT}")
        if OUTPUT.read_bytes() != generated:
            fail(
                "generated table is stale: run "
                "python3 tools/gen-presets/generate.py"
            )
        print("generator_reproduces_checked_in_table: ok")
    else:
        OUTPUT.write_bytes(generated)
        print(f"wrote {OUTPUT} with {len(definitions)} preset definitions")

    print(
        "source_has_187_direct_definitions: ok "
        f"({EXPECTED_DEFINITION_COUNT} direct definitions, "
        f"{len(definitions)} unique names)"
    )
    if args.corpus is not None:
        decks, uses, unique = scan_corpus(args.corpus, definitions)
        print(
            "generated_table_covers_every_corpus_preset: ok "
            f"({decks} decks, {uses} uses, {unique} unique names)"
        )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, UnicodeError, zipfile.BadZipFile) as error:
        print(f"generate.py: {error}", file=sys.stderr)
        raise SystemExit(1) from error

#!/usr/bin/env python3
"""Validate public from-scratch DOCX authoring and private local evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
from tempfile import TemporaryDirectory
import tomllib
import unittest
from unittest import mock
import xml.etree.ElementTree as ET
import zipfile

from golden_png_harness import decode_png
from pptx_ssim_harness import assert_tool_versions, structural_similarity


REPO_ROOT = Path(__file__).resolve().parent.parent
PRIVATE_ROOT = REPO_ROOT / "corpus" / "private-docx"
PRIVATE_MANIFEST = PRIVATE_ROOT / "manifest.json"
PRIVATE_EVIDENCE = PRIVATE_ROOT / "evidence"
PUBLIC_DPI = 150
PRIVATE_CASES = ("P1", "P2", "P3", "P4", "P5")
MAX_PACKAGE_MEMBERS = 2048
MAX_EXPANDED_BYTES = 256 * 1024 * 1024
CONTENT_TYPES_NS = "http://schemas.openxmlformats.org/package/2006/content-types"
RELATIONSHIPS_NS = "http://schemas.openxmlformats.org/package/2006/relationships"
WORD_NS = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
WORD_MAIN_CONTENT_TYPES = {
    "docx": "application/vnd.openxmlformats-officedocument."
    "wordprocessingml.document.main+xml",
    "docm": "application/vnd.ms-word.document.macroEnabled.main+xml",
    "dotx": "application/vnd.openxmlformats-officedocument."
    "wordprocessingml.template.main+xml",
    "dotm": "application/vnd.ms-word.template.macroEnabledTemplate.main+xml",
}
PRIVATE_SKIP = "private conformance: skipped (local inputs unavailable)"

PUBLIC_CONSUMER_SOURCE = r'''use rdocx::{
    BodyItemRef, Document, WordCreationProfile, WordPackageClass,
};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args_os().skip(1);
    let output = PathBuf::from(arguments.next().ok_or("missing output directory")?);
    if arguments.next().is_some() {
        return Err("unexpected input argument".into());
    }
    fs::create_dir_all(&output)?;

    for (extension, class) in [
        ("docx", WordPackageClass::Document),
        ("docm", WordPackageClass::MacroEnabledDocument),
        ("dotx", WordPackageClass::Template),
        ("dotm", WordPackageClass::MacroEnabledTemplate),
    ] {
        let path = output.join(format!("profile.{extension}"));
        let mut profile = Document::new_with_profile(
            WordCreationProfile::WordCompatible(class),
        );
        profile.save(&path)?;
        if Document::open(&path)?.package_class()? != class {
            return Err(format!("{extension} profile changed package class").into());
        }
    }

    let authored_path = output.join("authored.docx");
    let reopened_path = output.join("reopened.docx");
    let first_png = output.join("render-first.png");
    let second_png = output.join("render-second.png");
    let report_path = output.join("report.txt");

    let mut document = Document::new();
    document.set_title("Public Authoring Conformance Fixture");
    document.set_subject("Public facade package and render validation");
    document.set_header("Public conformance header");
    document.set_footer("Public conformance footer");
    document.add_paragraph("Public authoring starts from a blank document.");
    document.add_numbered_list_item("First modeled item", 0);
    document.add_numbered_list_item("Second modeled item", 0);
    let mut table = document.add_table(2, 2);
    table.cell(0, 0).ok_or("missing table cell")?.set_text("Capability");
    table.cell(0, 1).ok_or("missing table cell")?.set_text("Status");
    table.cell(1, 0).ok_or("missing table cell")?.set_text("Public facade");
    table.cell(1, 1).ok_or("missing table cell")?.set_text("Pass");
    document.save(&authored_path)?;

    let mut reopened = Document::open(&authored_path)?;
    if reopened.title() != Some("Public Authoring Conformance Fixture") {
        return Err("title did not survive reopen".into());
    }
    if reopened.subject() != Some("Public facade package and render validation")
        || reopened.header_text().as_deref() != Some("Public conformance header")
        || reopened.footer_text().as_deref() != Some("Public conformance footer")
    {
        return Err("package metadata or stories did not survive reopen".into());
    }
    if reopened.table_count() != 1 || !reopened.text().contains("Public facade") {
        return Err("modeled state did not survive reopen".into());
    }
    let numbered = reopened
        .paragraphs()
        .iter()
        .filter(|paragraph| paragraph.numbering().is_some())
        .count();
    if numbered != 2 {
        return Err("numbering did not survive reopen".into());
    }
    let unsupported = reopened
        .body_items()
        .filter(|item| matches!(item, BodyItemRef::UnsupportedXml(_)))
        .count();
    let first = reopened
        .render_page_to_png_deterministic(0, 150.0)?
        .ok_or("fixture rendered no first page")?;
    let second = reopened
        .render_page_to_png_deterministic(0, 150.0)?
        .ok_or("fixture rendered no repeated first page")?;
    if first != second {
        return Err("deterministic render changed between calls".into());
    }
    fs::write(&first_png, &first)?;
    fs::write(&second_png, &second)?;
    reopened.save(&reopened_path)?;
    let final_document = Document::open(&reopened_path)?;
    if final_document.title() != Some("Public Authoring Conformance Fixture")
        || final_document.table_count() != 1
        || final_document.text() != reopened.text()
    {
        return Err("save and reopen changed modeled state".into());
    }
    fs::write(
        report_path,
        format!(
            "paragraphs={}\ntables={}\nnumbered={}\nheader_footer={}\nunsupported={}\ndpi=150\n",
            final_document.paragraph_count(),
            final_document.table_count(),
            numbered,
            usize::from(final_document.has_header_footer_content()),
            unsupported,
        ),
    )?;
    Ok(())
}
'''


class ConformanceError(ValueError):
    """A sanitized conformance failure safe to print."""


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def safe_member_name(name: str) -> None:
    path = PurePosixPath(name)
    if (
        not name
        or name.startswith("/")
        or "\\" in name
        or any(part in ("", ".", "..") for part in path.parts)
    ):
        raise ConformanceError("package contains an unsafe member path")


def package_members(path: Path) -> dict[str, bytes]:
    try:
        with zipfile.ZipFile(path) as archive:
            infos = archive.infolist()
            if not infos or len(infos) > MAX_PACKAGE_MEMBERS:
                raise ConformanceError("package member inventory is invalid")
            names: set[str] = set()
            expanded = 0
            members: dict[str, bytes] = {}
            for info in infos:
                safe_member_name(info.filename)
                if info.is_dir() or info.filename in names:
                    raise ConformanceError("package member inventory is invalid")
                names.add(info.filename)
                expanded += info.file_size
                if expanded > MAX_EXPANDED_BYTES:
                    raise ConformanceError("package expansion exceeds the safety ceiling")
                members[info.filename] = archive.read(info)
    except (OSError, zipfile.BadZipFile, RuntimeError) as error:
        raise ConformanceError("DOCX package cannot be read") from error
    return members


def parse_xml(data: bytes, label: str) -> ET.Element:
    try:
        return ET.fromstring(data)
    except ET.ParseError as error:
        raise ConformanceError(f"{label} is not well-formed XML") from error


def content_type_map(members: dict[str, bytes]) -> dict[str, str]:
    data = members.get("[Content_Types].xml")
    if data is None:
        raise ConformanceError("package has no content type manifest")
    root = parse_xml(data, "content type manifest")
    if root.tag != f"{{{CONTENT_TYPES_NS}}}Types":
        raise ConformanceError("content type manifest has the wrong root")
    defaults: dict[str, str] = {}
    overrides: dict[str, str] = {}
    for child in root:
        if child.tag == f"{{{CONTENT_TYPES_NS}}}Default":
            extension = child.attrib.get("Extension", "").lower()
            content_type = child.attrib.get("ContentType", "")
            if not extension or not content_type or extension in defaults:
                raise ConformanceError("content type defaults are invalid")
            defaults[extension] = content_type
        elif child.tag == f"{{{CONTENT_TYPES_NS}}}Override":
            part = child.attrib.get("PartName", "")
            content_type = child.attrib.get("ContentType", "")
            if not part.startswith("/") or not content_type or part in overrides:
                raise ConformanceError("content type overrides are invalid")
            overrides[part] = content_type
        else:
            raise ConformanceError("content type manifest contains an unknown child")
    mapped: dict[str, str] = {}
    for name in members:
        if name == "[Content_Types].xml":
            continue
        part = f"/{name}"
        if part in overrides:
            mapped[part] = overrides[part]
            continue
        extension = (
            "rels"
            if name.endswith(".rels")
            else PurePosixPath(name).suffix.removeprefix(".").lower()
        )
        if extension not in defaults:
            raise ConformanceError("package part has no declared content type")
        mapped[part] = defaults[extension]
    return mapped


def relationship_records(
    members: dict[str, bytes],
) -> tuple[tuple[str, str, str, str, str], ...]:
    records = []
    for name, data in members.items():
        if not name.endswith(".rels"):
            continue
        root = parse_xml(data, "relationship part")
        if root.tag != f"{{{RELATIONSHIPS_NS}}}Relationships":
            raise ConformanceError("relationship part has the wrong root")
        seen: set[str] = set()
        for child in root:
            if child.tag != f"{{{RELATIONSHIPS_NS}}}Relationship":
                raise ConformanceError("relationship part contains an unknown child")
            relationship_id = child.attrib.get("Id", "")
            relationship_type = child.attrib.get("Type", "")
            target = child.attrib.get("Target", "")
            target_mode = child.attrib.get("TargetMode", "")
            if (
                not relationship_id
                or relationship_id in seen
                or not relationship_type
                or not target
            ):
                raise ConformanceError("relationship inventory is invalid")
            if target_mode not in ("", "External"):
                raise ConformanceError("relationship target mode is invalid")
            if target_mode != "External" and "\\" in target:
                raise ConformanceError("relationship target is unsafe")
            seen.add(relationship_id)
            records.append(
                (name, relationship_id, relationship_type, target, target_mode)
            )
    return tuple(sorted(records))


def validate_document_order(members: dict[str, bytes]) -> None:
    data = members.get("word/document.xml")
    if data is None:
        raise ConformanceError("package has no main document part")
    root = parse_xml(data, "main document part")
    if root.tag != f"{{{WORD_NS}}}document":
        raise ConformanceError("main document part has the wrong root")
    body = root.find(f"{{{WORD_NS}}}body")
    if body is None:
        raise ConformanceError("main document part has no body")
    section_positions = [
        index
        for index, child in enumerate(body)
        if child.tag == f"{{{WORD_NS}}}sectPr"
    ]
    if len(section_positions) != 1 or section_positions[0] != len(body) - 1:
        raise ConformanceError("main document children violate schema order")


def normalized_package_graph(
    path: Path, expected_main_content_type: str = WORD_MAIN_CONTENT_TYPES["docx"],
) -> tuple[
    tuple[tuple[str, str], ...],
    tuple[tuple[str, str, str, str, str], ...],
]:
    members = package_members(path)
    types = content_type_map(members)
    relationships = relationship_records(members)
    validate_document_order(members)
    if types.get("/word/document.xml") != expected_main_content_type:
        raise ConformanceError("main document content type is invalid")
    return tuple(sorted(types.items())), relationships


def validate_word_compatible_profile(path: Path, extension: str) -> None:
    members = package_members(path)
    expected_members = {
        "[Content_Types].xml",
        "_rels/.rels",
        "docProps/app.xml",
        "docProps/core.xml",
        "word/_rels/document.xml.rels",
        "word/document.xml",
        "word/fontTable.xml",
        "word/settings.xml",
        "word/styles.xml",
        "word/theme/theme1.xml",
    }
    if set(members) != expected_members:
        raise ConformanceError("fresh profile part inventory is incomplete")
    content_types, relationships = normalized_package_graph(
        path, WORD_MAIN_CONTENT_TYPES[extension]
    )
    required_content_types = {
        "/docProps/app.xml": "application/vnd.openxmlformats-officedocument."
        "extended-properties+xml",
        "/docProps/core.xml": "application/vnd.openxmlformats-package."
        "core-properties+xml",
        "/word/fontTable.xml": "application/vnd.openxmlformats-officedocument."
        "wordprocessingml.fontTable+xml",
        "/word/settings.xml": "application/vnd.openxmlformats-officedocument."
        "wordprocessingml.settings+xml",
        "/word/styles.xml": "application/vnd.openxmlformats-officedocument."
        "wordprocessingml.styles+xml",
        "/word/theme/theme1.xml": "application/vnd.openxmlformats-officedocument."
        "theme+xml",
    }
    mapped = dict(content_types)
    if any(mapped.get(part) != kind for part, kind in required_content_types.items()):
        raise ConformanceError("fresh profile content types are incomplete")
    expected_relationships = {
        (
            "_rels/.rels",
            "rId1",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument",
            "word/document.xml",
            "",
        ),
        (
            "_rels/.rels",
            "rId2",
            "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties",
            "docProps/core.xml",
            "",
        ),
        (
            "_rels/.rels",
            "rId3",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties",
            "docProps/app.xml",
            "",
        ),
        (
            "word/_rels/document.xml.rels",
            "rId0",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
            "styles.xml",
            "",
        ),
        (
            "word/_rels/document.xml.rels",
            "rdocxSettings",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings",
            "settings.xml",
            "",
        ),
        (
            "word/_rels/document.xml.rels",
            "rdocxTheme",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme",
            "theme/theme1.xml",
            "",
        ),
        (
            "word/_rels/document.xml.rels",
            "rdocxFontTable",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable",
            "fontTable.xml",
            "",
        ),
    }
    if set(relationships) != expected_relationships:
        raise ConformanceError("fresh profile relationship graph is incomplete")
    core = members["docProps/core.xml"]
    if b"dcterms:created" in core or b"dcterms:modified" in core:
        raise ConformanceError("fresh profile invented timestamps")
    if any(b"vbaProject" in data for data in members.values()):
        raise ConformanceError("fresh profile invented a VBA project")


def assert_preserved_opaque_parts(before: Path, after: Path) -> None:
    first = package_members(before)
    second = package_members(after)
    mutable = {
        "[Content_Types].xml",
        "_rels/.rels",
        "word/document.xml",
        "word/_rels/document.xml.rels",
        "docProps/core.xml",
    }
    opaque = sorted(set(first) - mutable)
    if not opaque:
        raise ConformanceError("public package has no preservation probe")
    if any(name not in second or first[name] != second[name] for name in opaque):
        raise ConformanceError("save and reopen changed an unmodeled package part")


def public_consumer_manifest() -> str:
    crate = (REPO_ROOT / "crates" / "rdocx").as_posix()
    return (
        "[package]\n"
        'name = "rdocx-authoring-conformance"\n'
        'version = "0.0.0"\n'
        'edition = "2024"\n'
        "publish = false\n\n"
        "[dependencies]\n"
        f'rdocx = {{ path = "{crate}" }}\n'
    )


def validate_public_boundary(manifest: str, source: str) -> None:
    try:
        parsed = tomllib.loads(manifest)
    except tomllib.TOMLDecodeError as error:
        raise ConformanceError("public consumer manifest is invalid") from error
    dependencies = parsed.get("dependencies")
    if not isinstance(dependencies, dict):
        raise ConformanceError("public consumer has no dependency boundary")
    rdocx_dependency = dependencies.get("rdocx")
    if (
        set(dependencies) != {"rdocx"}
        or not isinstance(rdocx_dependency, dict)
        or set(rdocx_dependency) != {"path"}
        or rdocx_dependency["path"]
        != (REPO_ROOT / "crates" / "rdocx").as_posix()
    ):
        raise ConformanceError("public consumer depends on a private crate")
    forbidden = (
        "rdocx_oxml",
        "rdocx-oxml",
        "oxml_",
        "oxml-",
        "OpcPackage",
        "set_raw_",
        "RawXml",
        "include_bytes!",
        "Document::from_bytes",
    )
    if any(token in source for token in forbidden):
        raise ConformanceError("public fixture bypasses the rdocx authoring boundary")
    if source.count("Document::new()") != 1:
        raise ConformanceError("public fixture must start from Document::new()")
    if "Document::open(&authored_path)" not in source:
        raise ConformanceError("public fixture does not reopen its own output")


def parse_public_report(path: Path) -> dict[str, int]:
    try:
        values = {
            key: int(value)
            for key, value in (
                line.split("=", 1)
                for line in path.read_text(encoding="utf-8").splitlines()
            )
        }
    except (OSError, ValueError) as error:
        raise ConformanceError("public consumer report is invalid") from error
    if (
        values.get("paragraphs", 0) < 3
        or values.get("tables") != 1
        or values.get("numbered") != 2
        or values.get("header_footer") != 1
    ):
        raise ConformanceError("public modeled-state report is incomplete")
    if values.get("unsupported") != 0 or values.get("dpi") != PUBLIC_DPI:
        raise ConformanceError("public diagnostics or render resolution is invalid")
    return values


def run_public() -> None:
    with TemporaryDirectory(prefix="rdocx-authoring-public-") as directory:
        root = Path(directory)
        source_root = root / "src"
        output = root / "output"
        source_root.mkdir()
        manifest = public_consumer_manifest()
        validate_public_boundary(manifest, PUBLIC_CONSUMER_SOURCE)
        (root / "Cargo.toml").write_text(manifest, encoding="utf-8")
        (source_root / "main.rs").write_text(PUBLIC_CONSUMER_SOURCE, encoding="utf-8")
        environment = os.environ.copy()
        environment["CARGO_TARGET_DIR"] = str(
            REPO_ROOT / "target" / "docx-authoring-conformance"
        )
        completed = subprocess.run(
            ("cargo", "run", "--quiet", "--offline", "--", str(output)),
            cwd=root,
            env=environment,
            capture_output=True,
            text=True,
            timeout=300,
        )
        if completed.returncode != 0:
            raise ConformanceError("public rdocx-only consumer failed")

        authored = output / "authored.docx"
        reopened = output / "reopened.docx"
        first_png = output / "render-first.png"
        second_png = output / "render-second.png"
        for path in (authored, reopened, first_png, second_png, output / "report.txt"):
            if not path.is_file() or path.stat().st_size == 0:
                raise ConformanceError("public consumer evidence is incomplete")
        for extension in WORD_MAIN_CONTENT_TYPES:
            profile = output / f"profile.{extension}"
            if not profile.is_file() or profile.stat().st_size == 0:
                raise ConformanceError("public profile evidence is incomplete")
            validate_word_compatible_profile(profile, extension)
        if normalized_package_graph(authored) != normalized_package_graph(reopened):
            raise ConformanceError("save and reopen changed the normalized package graph")
        assert_preserved_opaque_parts(authored, reopened)
        parse_public_report(output / "report.txt")
        try:
            first = decode_png(first_png)
            second = decode_png(second_png)
        except ValueError as error:
            raise ConformanceError("public deterministic PNG evidence is invalid") from error
        if first != second or first[0] <= 0 or first[1] <= 0:
            raise ConformanceError("repeated deterministic public render changed")
    print("public authoring conformance: pass")


def git_paths(*arguments: str) -> tuple[str, ...]:
    completed = subprocess.run(
        ("git", *arguments),
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
        timeout=30,
    )
    return tuple(line for line in completed.stdout.splitlines() if line)


def private_artifact_path(path: str) -> bool:
    lowered = path.lower()
    return (
        lowered.startswith("corpus/private-docx/")
        or lowered.endswith(".docx")
        or lowered.endswith(".docm")
        or lowered.endswith(".dotx")
        or lowered.endswith(".dotm")
        or "private-docx" in lowered
        or "private_docx" in lowered
    )


def reject_tracked_or_staged_private_artifacts(
    tracked: tuple[str, ...] | None = None,
    staged: tuple[str, ...] | None = None,
) -> None:
    tracked = git_paths("ls-files") if tracked is None else tracked
    staged = (
        git_paths("diff", "--cached", "--name-only", "--diff-filter=ACMR")
        if staged is None
        else staged
    )
    if any(private_artifact_path(path) for path in (*tracked, *staged)):
        raise ConformanceError("tracked or staged private artifact detected")


def load_private_manifest(root: Path) -> dict[str, object]:
    path = root / "manifest.json"
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ConformanceError("private manifest is unavailable or invalid") from error
    if not isinstance(payload, dict) or set(payload) != {
        "schema",
        "dpi",
        "tools",
        "cases",
    }:
        raise ConformanceError("private manifest fields are invalid")
    if payload["schema"] != 1 or payload["dpi"] != PUBLIC_DPI:
        raise ConformanceError("private manifest schema or resolution is invalid")
    tools = payload["tools"]
    if not isinstance(tools, dict) or set(tools) != {"libreoffice", "pdftoppm"}:
        raise ConformanceError("private tool identity fields are invalid")
    cases = payload["cases"]
    if not isinstance(cases, list) or len(cases) != len(PRIVATE_CASES):
        raise ConformanceError("private case inventory is incomplete")
    expected_keys = {
        "id",
        "reference",
        "candidate",
        "reference_sha256",
        "candidate_sha256",
        "minimum_ssim",
        "reference_pages",
        "candidate_pages",
    }
    for index, case in enumerate(cases, 1):
        if not isinstance(case, dict) or set(case) != expected_keys:
            raise ConformanceError("private case fields are invalid")
        alias = PRIVATE_CASES[index - 1]
        if case["id"] != alias:
            raise ConformanceError("private aliases are missing or out of order")
        if case["reference"] != f"reference-{index:02d}.docx":
            raise ConformanceError("private reference aliases are invalid")
        if case["candidate"] != f"candidate-{alias}.docx":
            raise ConformanceError("private candidate aliases are invalid")
        for digest_key in ("reference_sha256", "candidate_sha256"):
            if not isinstance(case[digest_key], str) or not re.fullmatch(
                r"[0-9a-f]{64}", case[digest_key]
            ):
                raise ConformanceError("private digest evidence is invalid")
        threshold = case["minimum_ssim"]
        if type(threshold) not in (int, float) or not 0.0 <= threshold <= 1.0:
            raise ConformanceError("private visual threshold is invalid")
        for page_key in ("reference_pages", "candidate_pages"):
            if type(case[page_key]) is not int or case[page_key] < 1:
                raise ConformanceError("private page expectation is invalid")
    return payload


def private_inventory(root: Path) -> tuple[str, ...]:
    inventory = []
    for path in root.iterdir():
        if path.name == "evidence" and path.is_dir():
            continue
        if not path.is_file():
            raise ConformanceError("private input inventory is missing or unexpected")
        inventory.append(path.name)
    return tuple(sorted(inventory))


def validate_private_inventory(root: Path, payload: dict[str, object]) -> None:
    cases = payload["cases"]
    assert isinstance(cases, list)
    expected = {"manifest.json"}
    for case in cases:
        assert isinstance(case, dict)
        expected.add(str(case["reference"]))
        expected.add(str(case["candidate"]))
    if set(private_inventory(root)) != expected:
        raise ConformanceError("private input inventory is missing or unexpected")
    for case in cases:
        assert isinstance(case, dict)
        reference = root / str(case["reference"])
        candidate = root / str(case["candidate"])
        if sha256(reference) != case["reference_sha256"]:
            raise ConformanceError("private reference identity changed")
        if sha256(candidate) != case["candidate_sha256"]:
            raise ConformanceError("private candidate identity changed")


def render_private_alias(source: Path, alias: str, output: Path) -> list[Path]:
    with TemporaryDirectory(prefix="rdocx-private-render-") as directory:
        temporary = Path(directory)
        generic_input = temporary / f"{alias}.docx"
        render_root = temporary / "rendered"
        shutil.copyfile(source, generic_input)
        completed = subprocess.run(
            (
                "cargo",
                "run",
                "--quiet",
                "--locked",
                "--offline",
                "-p",
                "rdocx-cli",
                "--",
                "render",
                "--dpi",
                str(PUBLIC_DPI),
                "--output-dir",
                str(render_root),
                str(generic_input),
            ),
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            timeout=300,
        )
        if completed.returncode != 0:
            raise ConformanceError(f"{alias} deterministic render failed")
        pages = sorted(
            render_root.glob(f"{alias}_page*.png"),
            key=lambda path: int(path.stem.rsplit("page", 1)[1]),
        )
        if not pages:
            raise ConformanceError(f"{alias} deterministic render produced no pages")
        output.mkdir(parents=True, exist_ok=True)
        published = []
        for index, page in enumerate(pages, 1):
            target = output / f"{alias}-page-{index}.png"
            shutil.copyfile(page, target)
            published.append(target)
        return published


def modeled_projection(source: Path, alias: str) -> dict[str, object]:
    with TemporaryDirectory(prefix="rdocx-private-inspect-") as directory:
        generic_input = Path(directory) / f"{alias}.docx"
        shutil.copyfile(source, generic_input)
        completed = subprocess.run(
            (
                "cargo",
                "run",
                "--quiet",
                "--locked",
                "--offline",
                "-p",
                "rdocx-cli",
                "--",
                "inspect",
                str(generic_input),
                "--json",
            ),
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            timeout=300,
        )
        if completed.returncode != 0:
            raise ConformanceError(f"{alias} modeled inspection failed")
        try:
            projection = json.loads(completed.stdout)
        except json.JSONDecodeError as error:
            raise ConformanceError(f"{alias} modeled inspection is invalid") from error
        if not isinstance(projection, dict) or projection.pop("file", None) is None:
            raise ConformanceError(f"{alias} modeled inspection is incomplete")
        return projection


def private_case_evidence(
    root: Path, case: dict[str, object], evidence: Path
) -> dict[str, object]:
    alias = str(case["id"])
    reference = root / str(case["reference"])
    candidate = root / str(case["candidate"])
    if normalized_package_graph(reference) != normalized_package_graph(candidate):
        raise ConformanceError(f"{alias} normalized package graph differs")
    if modeled_projection(reference, f"{alias}-reference") != modeled_projection(
        candidate, f"{alias}-candidate"
    ):
        raise ConformanceError(f"{alias} modeled projection differs")
    reference_pages = render_private_alias(reference, f"{alias}-reference", evidence)
    candidate_pages = render_private_alias(candidate, f"{alias}-candidate", evidence)
    if len(reference_pages) != case["reference_pages"]:
        raise ConformanceError(f"{alias} reference page count differs")
    if len(candidate_pages) != case["candidate_pages"]:
        raise ConformanceError(f"{alias} candidate page count differs")
    if len(reference_pages) != len(candidate_pages):
        raise ConformanceError(f"{alias} page counts differ")
    scores = []
    dimensions = []
    for reference_page, candidate_page in zip(reference_pages, candidate_pages):
        try:
            first = decode_png(reference_page)
            second = decode_png(candidate_page)
        except ValueError as error:
            raise ConformanceError(f"{alias} rendered PNG evidence is invalid") from error
        if first[:2] != second[:2]:
            raise ConformanceError(f"{alias} rendered page geometry differs")
        score = structural_similarity(first, second)
        if score < float(case["minimum_ssim"]):
            raise ConformanceError(f"{alias} rendered similarity is below its threshold")
        scores.append(round(score, 9))
        dimensions.append([first[0], first[1]])
    return {
        "id": alias,
        "reference_pages": len(reference_pages),
        "candidate_pages": len(candidate_pages),
        "dimensions": dimensions,
        "scores": scores,
        "minimum_ssim": case["minimum_ssim"],
    }


def run_private(required: bool, root: Path = PRIVATE_ROOT) -> bool:
    reject_tracked_or_staged_private_artifacts()
    if not root.is_dir() or not (root / "manifest.json").is_file():
        if required:
            raise ConformanceError("private corpus is unavailable")
        print(PRIVATE_SKIP)
        return False
    payload = load_private_manifest(root)
    validate_private_inventory(root, payload)
    try:
        expected_soffice, expected_pdftoppm = assert_tool_versions()
    except (OSError, subprocess.SubprocessError, ValueError) as error:
        raise ConformanceError("required private renderer versions are unavailable") from error
    tools = payload["tools"]
    assert isinstance(tools, dict)
    if tools["libreoffice"] != expected_soffice or tools["pdftoppm"] != expected_pdftoppm:
        raise ConformanceError("private manifest tool identities changed")
    evidence = root / "evidence"
    if evidence.exists():
        shutil.rmtree(evidence)
    evidence.mkdir()
    records = []
    cases = payload["cases"]
    assert isinstance(cases, list)
    for case in cases:
        assert isinstance(case, dict)
        records.append(private_case_evidence(root, case, evidence))
    (evidence / "results.json").write_text(
        json.dumps(
            {
                "schema": 1,
                "dpi": PUBLIC_DPI,
                "tools": tools,
                "cases": records,
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    print("private authoring conformance: pass (P1 through P5)")
    return True


class HarnessSelfTests(unittest.TestCase):
    def test_public_boundary_rejects_private_dependencies_and_raw_bypasses(self) -> None:
        manifest = public_consumer_manifest()
        validate_public_boundary(manifest, PUBLIC_CONSUMER_SOURCE)
        with self.assertRaisesRegex(ConformanceError, "private crate"):
            validate_public_boundary(manifest + "oxml-opc = \"0.1\"\n", PUBLIC_CONSUMER_SOURCE)
        aliased = manifest.replace(
            "rdocx = { path = ",
            'rdocx = { package = "rdocx-oxml", path = ',
        )
        with self.assertRaisesRegex(ConformanceError, "private crate"):
            validate_public_boundary(aliased, PUBLIC_CONSUMER_SOURCE)
        with self.assertRaisesRegex(ConformanceError, "authoring boundary"):
            validate_public_boundary(manifest, PUBLIC_CONSUMER_SOURCE + "\nOpcPackage")
        with self.assertRaisesRegex(ConformanceError, "Document::new"):
            validate_public_boundary(
                manifest, PUBLIC_CONSUMER_SOURCE.replace("Document::new()", "Document::default()")
            )

    def test_normalized_relationship_records_retain_ids(self) -> None:
        first = b'''<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="urn:test" Target="item.xml"/></Relationships>'''
        second = first.replace(b'rId1', b'rId2')
        self.assertNotEqual(
            relationship_records({"_rels/.rels": first}),
            relationship_records({"_rels/.rels": second}),
        )

    def test_private_artifact_scan_rejects_without_disclosing_paths(self) -> None:
        reject_tracked_or_staged_private_artifacts(("src/lib.rs",), ("README.md",))
        secret = "corpus/private-docx/customer-name.docx"
        with self.assertRaises(ConformanceError) as context:
            reject_tracked_or_staged_private_artifacts((secret,), ())
        self.assertNotIn(secret, str(context.exception))

    def test_optional_and_required_private_modes_fail_closed(self) -> None:
        with TemporaryDirectory(prefix="rdocx-private-mode-test-") as directory:
            root = Path(directory) / "missing"
            with mock.patch(
                f"{__name__}.reject_tracked_or_staged_private_artifacts"
            ):
                self.assertFalse(run_private(False, root))
                with self.assertRaisesRegex(ConformanceError, "unavailable"):
                    run_private(True, root)

    def test_private_manifest_rejects_missing_extra_changed_and_incomplete_evidence(self) -> None:
        with TemporaryDirectory(prefix="rdocx-private-manifest-test-") as directory:
            root = Path(directory)
            (root / "manifest.json").write_text("[]", encoding="utf-8")
            with self.assertRaisesRegex(ConformanceError, "fields are invalid"):
                load_private_manifest(root)
            payload = {
                "schema": 1,
                "dpi": PUBLIC_DPI,
                "tools": {"libreoffice": "pinned", "pdftoppm": "pinned"},
                "cases": [],
            }
            for index, alias in enumerate(PRIVATE_CASES, 1):
                reference = root / f"reference-{index:02d}.docx"
                candidate = root / f"candidate-{alias}.docx"
                reference.write_bytes(f"reference-{alias}".encode())
                candidate.write_bytes(f"candidate-{alias}".encode())
                payload["cases"].append(
                    {
                        "id": alias,
                        "reference": reference.name,
                        "candidate": candidate.name,
                        "reference_sha256": sha256(reference),
                        "candidate_sha256": sha256(candidate),
                        "minimum_ssim": 0.5 + index / 100,
                        "reference_pages": 1,
                        "candidate_pages": 1,
                    }
                )
            (root / "manifest.json").write_text(json.dumps(payload), encoding="utf-8")
            loaded = load_private_manifest(root)
            validate_private_inventory(root, loaded)

            missing = root / "candidate-P2.docx"
            missing_bytes = missing.read_bytes()
            missing.unlink()
            with self.assertRaisesRegex(ConformanceError, "missing or unexpected"):
                validate_private_inventory(root, loaded)
            missing.write_bytes(missing_bytes)

            (root / "unexpected.bin").write_bytes(b"extra")
            with self.assertRaisesRegex(ConformanceError, "unexpected"):
                validate_private_inventory(root, loaded)
            (root / "unexpected.bin").unlink()

            (root / "candidate-P3.docx").write_bytes(b"changed")
            with self.assertRaisesRegex(ConformanceError, "identity changed"):
                validate_private_inventory(root, loaded)

            incomplete = dict(payload)
            incomplete["cases"] = payload["cases"][:-1]
            (root / "manifest.json").write_text(
                json.dumps(incomplete), encoding="utf-8"
            )
            with self.assertRaisesRegex(ConformanceError, "inventory is incomplete"):
                load_private_manifest(root)


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--self-test", action="store_true")
    mode.add_argument("--public", action="store_true")
    mode.add_argument("--private-required", action="store_true")
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        if arguments.self_test:
            suite = unittest.defaultTestLoader.loadTestsFromTestCase(HarnessSelfTests)
            result = unittest.TextTestRunner(verbosity=2).run(suite)
            return 0 if result.wasSuccessful() else 1
        if arguments.public:
            reject_tracked_or_staged_private_artifacts()
            run_public()
            return 0
        if arguments.private_required:
            run_private(True)
            return 0
        reject_tracked_or_staged_private_artifacts()
        run_public()
        run_private(False)
        return 0
    except subprocess.TimeoutExpired:
        print("docx_authoring_conformance: external command timed out", file=sys.stderr)
        return 1
    except (ConformanceError, subprocess.CalledProcessError, OSError) as error:
        print(f"docx_authoring_conformance: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

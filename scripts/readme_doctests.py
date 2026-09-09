#!/usr/bin/env python3
"""Validate every workspace README and compile its Rust examples."""

from __future__ import annotations

import argparse
from collections import Counter
from dataclasses import dataclass
import json
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tomllib
from urllib.parse import unquote, urlsplit
from urllib.request import Request, urlopen


REPO_ROOT = Path(__file__).resolve().parent.parent
RUST_FENCE = re.compile(r"^```rust(?P<attributes>[^\n]*)$", re.MULTILINE)
EXAMPLE_FENCE = re.compile(
    r"^```(?:rust[^\n]*|toml|python|javascript|sh|text)$",
    re.MULTILINE,
)
WORKSPACE_PACKAGE_COUNT = 27
PUBLISHABLE_PACKAGE_COUNT = 22
LOCAL_PATCHES = (
    ("oxml-core", "crates/oxml-core"),
    ("oxml-drawing", "crates/oxml-drawing"),
    ("oxml-layout", "crates/oxml-layout"),
    ("oxml-media", "crates/oxml-media"),
    ("oxml-opc", "crates/oxml-opc"),
    ("oxml-pdf", "crates/oxml-pdf"),
    ("oxml-sml", "crates/oxml-sml"),
    ("oxml-cli-support", "crates/oxml-cli-support"),
    ("oxml-chart", "crates/oxml-chart"),
    ("rdocx", "crates/rdocx"),
    ("rdocx-cli", "crates/rdocx-cli"),
    ("rdocx-html", "crates/rdocx-html"),
    ("rdocx-layout", "crates/rdocx-layout"),
    ("rdocx-opc", "crates/rdocx-opc"),
    ("rdocx-oxml", "crates/rdocx-oxml"),
    ("rdocx-pdf", "crates/rdocx-pdf"),
    ("rpptx", "crates/rpptx"),
    ("rpptx-cli", "crates/rpptx-cli"),
    ("rpptx-chart", "crates/rpptx-chart"),
    ("rpptx-layout", "crates/rpptx-layout"),
    ("rpptx-oxml", "crates/rpptx-oxml"),
    ("rpptx-render", "crates/rpptx-render"),
)


@dataclass(frozen=True)
class ReadmeCase:
    package: str
    crate_name: str
    readme: Path
    expected_rust_fences: int
    companion_crates: tuple[tuple[str, str], ...] = ()


README_CASES = (
    ReadmeCase("rdocx", "rdocx", REPO_ROOT / "README.md", 3),
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
    ReadmeCase(
        "oxml-cli-support",
        "oxml_cli_support",
        REPO_ROOT / "crates/oxml-cli-support/README.md",
        1,
    ),
    ReadmeCase(
        "oxml-core", "oxml_core", REPO_ROOT / "crates/oxml-core/README.md", 1
    ),
    ReadmeCase(
        "oxml-drawing",
        "oxml_drawing",
        REPO_ROOT / "crates/oxml-drawing/README.md",
        1,
    ),
    ReadmeCase(
        "oxml-layout",
        "oxml_layout",
        REPO_ROOT / "crates/oxml-layout/README.md",
        1,
    ),
    ReadmeCase(
        "oxml-media", "oxml_media", REPO_ROOT / "crates/oxml-media/README.md", 1
    ),
    ReadmeCase(
        "oxml-opc", "oxml_opc", REPO_ROOT / "crates/oxml-opc/README.md", 1
    ),
    ReadmeCase(
        "oxml-pdf",
        "oxml_pdf",
        REPO_ROOT / "crates/oxml-pdf/README.md",
        1,
        (("oxml-layout", "oxml_layout"),),
    ),
    ReadmeCase(
        "oxml-py-support",
        "oxml_py_support",
        REPO_ROOT / "crates/oxml-py-support/README.md",
        1,
    ),
    ReadmeCase(
        "oxml-sml",
        "oxml_sml",
        REPO_ROOT / "crates/oxml-sml/README.md",
        1,
    ),
    ReadmeCase(
        "oxml-chart",
        "oxml_chart",
        REPO_ROOT / "crates/oxml-chart/README.md",
        1,
    ),
    ReadmeCase(
        "rpptx",
        "rpptx",
        REPO_ROOT / "crates/rpptx/README.md",
        1,
    ),
    ReadmeCase(
        "rpptx-chart",
        "rpptx_chart",
        REPO_ROOT / "crates/rpptx-chart/README.md",
        1,
    ),
    ReadmeCase(
        "rpptx-layout",
        "rpptx_layout",
        REPO_ROOT / "crates/rpptx-layout/README.md",
        1,
    ),
    ReadmeCase(
        "rpptx-oxml",
        "rpptx_oxml",
        REPO_ROOT / "crates/rpptx-oxml/README.md",
        1,
    ),
    ReadmeCase(
        "rpptx-render",
        "rpptx_render",
        REPO_ROOT / "crates/rpptx-render/README.md",
        1,
    ),
)

README_REQUIRED_TEXT = {
    REPO_ROOT / "README.md": (
        'rdocx = "0.13.1"',
        'rdocx = { version = "0.13.1", default-features = false }',
        "rdocx convert report.docx --to pdf -o report.pdf",
        "rdocx convert report.docx --to html -o report.html",
        "rdocx convert report.docx --to md -o report.md",
        'rdocx replace report.docx --placeholder "Draft" --value "Final" -o final.docx',
    ),
    REPO_ROOT / "crates/rdocx-cli/README.md": (
        "cargo install rdocx-cli --version '^0.13.1'",
        "rdocx convert report.docx --to pdf -o report.pdf",
    ),
    REPO_ROOT / "crates/rdocx-html/README.md": ('rdocx-html = "0.13.1"',),
    REPO_ROOT / "crates/rdocx-layout/README.md": ('rdocx-layout = "0.13.1"',),
    REPO_ROOT / "crates/rdocx-opc/README.md": (
        'rdocx-opc = "0.13.1"',
        "use rdocx_opc::OpcPackage;",
    ),
    REPO_ROOT / "crates/rdocx-oxml/README.md": ('rdocx-oxml = "0.13.1"',),
    REPO_ROOT / "crates/rdocx-pdf/README.md": (
        'rdocx-pdf = "0.13.1"',
        "use rdocx_pdf::render_to_pdf;",
    ),
    REPO_ROOT / "crates/oxml-cli-support/README.md": (
        'oxml_cli_support::parse_range("2,4-6")?',
    ),
    REPO_ROOT / "crates/oxml-core/README.md": ("Length::inches(8.5)",),
    REPO_ROOT / "crates/oxml-drawing/README.md": ("Fill::from_xml",),
    REPO_ROOT / "crates/oxml-layout/README.md": ('Color::from_hex("3366CC")',),
    REPO_ROOT / "crates/oxml-media/README.md": ("resolve(b\"\\x89PNG",),
    REPO_ROOT / "crates/oxml-opc/README.md": ("ContentTypes::from_xml",),
    REPO_ROOT / "crates/oxml-pdf/README.md": ("render_to_pdf(&layout)",),
    REPO_ROOT / "crates/oxml-py-support/README.md": (
        "emu_from_inches(8.5)",
    ),
    REPO_ROOT / "crates/rdocx-py/README.md": (
        "doc.add_paragraph(\"Hello from Python\")",
        'doc.save("hello.docx")',
    ),
    REPO_ROOT / "crates/rdocx-wasm/README.md": (
        'from "@tensorbee/rdocx-wasm"',
        "doc.toDocxBytes()",
        "wasm-pack build --target bundler --scope tensorbee crates/rdocx-wasm --out-name rdocx_wasm",
    ),
    REPO_ROOT / "crates/rpptx/README.md": (
        "use rpptx::Presentation;",
        "Presentation::new()?",
    ),
    REPO_ROOT / "crates/oxml-chart/README.md": ("AxisId::new(10_000_001)?",),
    REPO_ROOT / "crates/rpptx-chart/README.md": ("AxisId::new(10_000_001)?",),
    REPO_ROOT / "crates/rpptx-cli/README.md": (
        "cargo install rpptx-cli --version '^0.11.0'",
        "rpptx convert deck.pptx --to pdf -o deck.pdf",
    ),
    REPO_ROOT / "crates/rpptx-py/README.md": (
        'Presentation("deck.pptx")',
        "len(presentation.slides)",
        "The binding does not expose PDF or raster rendering.",
    ),
    REPO_ROOT / "crates/rpptx-layout/README.md": ("ScopedMediaIds::default()",),
    REPO_ROOT / "crates/rpptx-oxml/README.md": (
        "CT_Presentation::from_xml",
        "part-level parsing and",
    ),
    REPO_ROOT / "crates/rpptx-render/README.md": (
        "RelScopes::default()",
        "does not write PDF or raster files",
    ),
    REPO_ROOT / "crates/rpptx-wasm/README.md": (
        'from "@tensorbee/rpptx-wasm"',
        "deck.toBytes()",
        "wasm-pack build --target bundler --scope tensorbee crates/rpptx-wasm --out-name rpptx_wasm",
    ),
}

ROOT_WORKFLOW_CLAIMS = {
    "DOCX": "Create, open, edit, validate, and save complete packages, with encryption and signing through opt-in features",
    "Rich authoring": "Paragraphs, runs, tables, styles, numbering, fields, forms, equations, drawings, comments, and metadata",
    "Preservation": "Retain unknown safe producer XML byte for byte when it is not modelled",
    "Native layout": "Resolve Word flow content into positioned pages with bundled, system, embedded, or caller-provided fonts",
    "Fixed output": "PDF, PDF/A, PNG, JPEG, TIFF, and SVG",
    "Flow output": "HTML, HTML fragments, Markdown, MHTML, RTF, ODT, and EPUB",
    "Automation": "Rust facade, CLI, Python binding, and locally built browser binding",
}
CAPABILITY_CLASSIFICATIONS = {
    "complete",
    "partial",
    "unsupported",
    "preserve-only",
    "permanent-non-goal",
}
COMPARISON_ROWS = {
    "rdocx": (
        "Open, create, edit, and save",
        "Unknown safe producer XML is retained byte for byte when it is not modelled",
        "Yes, Word flow layout with deterministic bundled fonts",
        "PDF and page images",
        "HTML and Markdown export",
        "`rdocx-cli`",
        "`rdocx-py`, with a narrower facade",
        "Workspace `rdocx-wasm` facade, deliberately unpublished",
    ),
    "python-docx": (
        "Create, open, change, and save",
        "Existing content that its API cannot manipulate is left alone on load and save. No byte-exact guarantee is stated",
        "ND",
        "ND",
        "ND",
        "ND",
        "Primary API",
        "ND",
    ),
    "docx-rs": (
        "Create and parse into the model used by its writer. Editing a parsed document is not separately documented",
        "ND. The project states that its OOXML support is not exhaustive",
        "ND",
        "ND",
        "ND",
        "ND",
        "ND",
        "Document generation and DOCX-to-JSON parsing through WebAssembly",
    ),
    "docx4j": (
        "Open, create, edit, and save",
        "ND for unknown XML or byte-exact round trips",
        "No project-owned page engine is documented. PDF paths use XSL-FO with Apache FOP, Microsoft Word through documents4j, or Microsoft Graph",
        "PDF through the documented conversion paths. Raster output is ND",
        "HTML export and first-party Markdown import and export",
        "ND",
        "ND",
        "ND",
    ),
    "Aspose.Words": (
        "Create, load, modify, and save DOCX",
        "Feature-level preservation is documented during conversion. No byte-exact unknown-XML contract is stated",
        "Yes, its own page-layout engine",
        "PDF plus PNG, JPEG, BMP, and TIFF",
        "HTML and Markdown",
        "ND",
        "Official Python via .NET API",
        "ND",
    ),
}
COMPARISON_EVIDENCE = (
    "https://github.com/tensorbee/rdocx",
    "https://python-docx.readthedocs.io/en/stable/user/documents.html",
    "https://github.com/bokuweb/docx-rs",
    "https://github.com/plutext/docx4j",
    "https://www.docx4java.org/blog/2020/09/office-pptxxlsxdocx-to-pdf-to-in-docx4j-8-2-3/",
    "https://github.com/plutext/docx4j/tree/VERSION_17_1_1/docx4j-markdown",
    "https://docs.aspose.com/words/python-net/product-overview/",
    "https://docs.aspose.com/words/python-net/converting-to-fixed-page-format/",
    "https://docs.aspose.com/words/python-net/supported-document-formats/",
    "https://docs.aspose.com/words/python-net/supported-features-on-document-load/",
)
COMPARISON_EVIDENCE_USAGE = COMPARISON_EVIDENCE + (
    "https://python-docx.readthedocs.io/en/stable/user/documents.html",
    "https://github.com/bokuweb/docx-rs",
)
ROOT_UNIQUENESS_CLAIM = (
    "Among these reviewed projects, rdocx alone documents the complete combination\n"
    "of a native Rust API, project-owned Word layout, fixed and flow outputs, a CLI,\n"
    "Python, and a browser surface. That statement is bounded to the official\n"
    "evidence set and review date."
)
MARKDOWN_LINK = re.compile(r"(?<!!)\[[^]]+\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")
MARKDOWN_IMAGE = re.compile(r"!\[[^]]*\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")


def markdown_destinations(text: str) -> tuple[str, ...]:
    images = tuple(MARKDOWN_IMAGE.findall(text))
    text_without_images = MARKDOWN_IMAGE.sub("image", text)
    return images + tuple(MARKDOWN_LINK.findall(text_without_images))


def markdown_section(text: str, heading: str) -> str | None:
    marker = f"## {heading}\n"
    if text.count(marker) != 1:
        return None
    section = text.split(marker, 1)[1]
    return section.split("\n## ", 1)[0]


def capability_matrix(text: str) -> dict[str, str]:
    matrix: dict[str, str] = {}
    for line in text.splitlines():
        if not line.startswith("| DOCX-"):
            continue
        cells = tuple(cell.strip() for cell in line.strip("|").split("|"))
        if len(cells) != 19 or cells[16] not in CAPABILITY_CLASSIFICATIONS:
            continue
        matrix[cells[0]] = cells[16]
    return matrix


def validate_root_capability_claims(readme: str, matrix_text: str) -> bool:
    section = markdown_section(readme, "Built for complete document workflows")
    matrix = capability_matrix(matrix_text)
    if section is None or not matrix:
        print(
            "README doctest error: missing capability section or approved matrix",
            file=sys.stderr,
        )
        return False
    observed: dict[str, str] = {}
    for line in section.splitlines():
        if not line.startswith("|"):
            continue
        cells = tuple(cell.strip() for cell in line.strip("|").split("|"))
        if len(cells) != 2 or cells[0] in {"Workflow", "---"}:
            continue
        if cells[0] in observed:
            print(f"README doctest error: duplicate workflow {cells[0]}", file=sys.stderr)
            return False
        observed[cells[0]] = cells[1]
    if observed != ROOT_WORKFLOW_CLAIMS:
        print(
            "README doctest error: root workflow claims differ from the approved "
            f"summary, observed={observed!r}",
            file=sys.stderr,
        )
        return False
    required_matrix_state = {
        "DOCX-001": "complete",
        "DOCX-002": "complete",
        "DOCX-080": "preserve-only",
        "DOCX-082": "permanent-non-goal",
        "DOCX-083": "permanent-non-goal",
    }
    if any(matrix.get(key) != value for key, value in required_matrix_state.items()):
        print(
            "README doctest error: approved matrix no longer supports the root boundary",
            file=sys.stderr,
        )
        return False
    return True


def validate_opt_in_security_claims(
    root_readme: str, rpptx_readme: str, metadata: dict[str, object]
) -> bool:
    packages = metadata.get("packages")
    if not isinstance(packages, list):
        print("README doctest error: invalid metadata for opt-in features", file=sys.stderr)
        return False
    package_metadata = {
        package.get("name"): package
        for package in packages
        if isinstance(package, dict)
    }
    expected = {"rdocx": root_readme, "rpptx": rpptx_readme}
    valid = True
    for package_name, readme in expected.items():
        package = package_metadata.get(package_name)
        features = package.get("features") if isinstance(package, dict) else None
        version = package.get("version") if isinstance(package, dict) else None
        defaults = features.get("default") if isinstance(features, dict) else None
        dependency = (
            f'{package_name} = {{ version = "{version}", features = '
            '["agile-encryption", "digital-signatures"] }'
        )
        if (
            not isinstance(version, str)
            or not isinstance(features, dict)
            or "agile-encryption" not in features
            or "digital-signatures" not in features
            or not isinstance(defaults, list)
            or "agile-encryption" in defaults
            or "digital-signatures" in defaults
            or readme.count(dependency) != 1
        ):
            print(
                f"README doctest error: {package_name} security features are not "
                "documented as exact opt-ins",
                file=sys.stderr,
            )
            valid = False
    return valid


def validate_root_narrative(readme: str) -> bool:
    headings = (
        "## Built for complete document workflows",
        "## Examples",
        "## Installation",
        "## Evidence-based alternatives",
        "## Honest boundaries",
    )
    positions = tuple(readme.find(heading) for heading in headings)
    required_intro = (
        "one `Document` for the complete Word workflow",
        "integrated native document stack",
        "without an Office installation or conversion service",
    )
    valid = all(position >= 0 for position in positions)
    valid = valid and positions == tuple(sorted(positions))
    valid = valid and all(item in readme[: positions[0]] for item in required_intro)
    valid = valid and "## Capability status" not in readme
    valid = valid and "## Project status" not in readme
    if not valid:
        print(
            "README doctest error: root README does not lead with the approved product narrative",
            file=sys.stderr,
        )
    return valid


def validate_crate_narratives(overrides: dict[Path, str] | None = None) -> bool:
    overrides = {} if overrides is None else overrides
    metadata = cargo_metadata()
    if metadata is None or not isinstance(metadata.get("packages"), list):
        print("README doctest error: invalid metadata for crate narratives", file=sys.stderr)
        return False
    valid = True
    for package in metadata["packages"]:
        if not isinstance(package, dict) or package.get("name") == "rdocx":
            continue
        readme = package_readme(package)
        if readme is None or not readme.is_file():
            valid = False
            continue
        text = overrides.get(readme, readme.read_text(encoding="utf-8"))
        headings = ("## Capabilities", "## Use it when", "## Relationship", "## Example")
        positions = tuple(text.find(heading) for heading in headings)
        section = markdown_section(text, "Capabilities")
        bullet_count = 0 if section is None else sum(
            line.startswith("- ") for line in section.splitlines()
        )
        if any(position < 0 for position in positions) or positions != tuple(
            sorted(positions)
        ) or bullet_count < 3:
            print(
                f"README doctest error: {readme} lacks the capability-led crate narrative",
                file=sys.stderr,
            )
            valid = False
    forbidden = {
        REPO_ROOT / "crates/rpptx-py/README.md": ("reading, editing, and rendering",),
        REPO_ROOT / "crates/rpptx-render/README.md": ("layout, raster, and PDF output",),
        REPO_ROOT / "crates/rpptx-oxml/README.md": ("package-preserving parse",),
        REPO_ROOT / "crates/rdocx-oxml/README.md": ("## Migrating from 0.4",),
    }
    for readme, claims in forbidden.items():
        text = overrides.get(readme, readme.read_text(encoding="utf-8"))
        if any(claim in text for claim in claims):
            print(
                f"README doctest error: {readme} retains a rejected capability claim",
                file=sys.stderr,
            )
            valid = False
    return valid


def validate_root_versions(readme: str, metadata: dict[str, object]) -> bool:
    packages = metadata.get("packages")
    if not isinstance(packages, list):
        print("README doctest error: invalid Cargo metadata", file=sys.stderr)
        return False
    versions = {
        package.get("name"): package.get("version")
        for package in packages
        if isinstance(package, dict)
    }
    rdocx_version = versions.get("rdocx")
    cli_version = versions.get("rdocx-cli")
    if not isinstance(rdocx_version, str) or not isinstance(cli_version, str):
        print(
            "README doctest error: Cargo metadata lacks the rdocx package family",
            file=sys.stderr,
        )
        return False
    requirements = (
        f'rdocx = "{rdocx_version}"',
        f'rdocx = {{ version = "{rdocx_version}", default-features = false }}',
        f"cargo install rdocx-cli --version '^{cli_version}'",
    )
    valid = True
    for requirement in requirements:
        if readme.count(requirement) != 1:
            print(
                "README doctest error: expected one metadata-derived root "
                f"requirement {requirement!r}",
                file=sys.stderr,
            )
            valid = False
    return valid


def markdown_anchors(text: str) -> set[str]:
    anchors: set[str] = set()
    for line in text.splitlines():
        match = re.match(r"^#{1,6}\s+(.+?)\s*#*$", line)
        if match is None:
            continue
        heading = re.sub(r"[`*_~]", "", match.group(1)).strip().lower()
        heading = re.sub(r"[^\w\- ]", "", heading)
        anchors.add(re.sub(r"[ ]+", "-", heading))
    return anchors


def validate_local_links(readme: str, source: Path | None = None) -> bool:
    source = REPO_ROOT / "README.md" if source is None else source
    valid = True
    for destination in markdown_destinations(readme):
        parsed = urlsplit(destination)
        if parsed.scheme or parsed.netloc:
            continue
        relative_path = unquote(parsed.path)
        target = (
            (source.parent / relative_path).resolve()
            if relative_path
            else source
        )
        try:
            target.relative_to(REPO_ROOT)
        except ValueError:
            print(
                f"README doctest error: local link escapes repository {destination!r}",
                file=sys.stderr,
            )
            valid = False
            continue
        if not target.is_file():
            print(
                f"README doctest error: local link target is missing {destination!r}",
                file=sys.stderr,
            )
            valid = False
            continue
        if parsed.fragment and target.suffix.lower() == ".md":
            anchors = markdown_anchors(target.read_text(encoding="utf-8"))
            if unquote(parsed.fragment).lower() not in anchors:
                print(
                    f"README doctest error: local anchor is missing {destination!r}",
                    file=sys.stderr,
                )
                valid = False
    return valid


def validate_all_local_links(overrides: dict[Path, str] | None = None) -> bool:
    overrides = {} if overrides is None else overrides
    metadata = cargo_metadata()
    if metadata is None or not isinstance(metadata.get("packages"), list):
        print("README doctest error: invalid metadata for README links", file=sys.stderr)
        return False
    readmes = {
        readme
        for package in metadata["packages"]
        if isinstance(package, dict)
        for readme in (package_readme(package),)
        if readme is not None
    }
    return all(
        validate_local_links(
            overrides.get(readme, readme.read_text(encoding="utf-8")), readme
        )
        for readme in sorted(readmes)
    )


def validate_comparison_evidence(readme: str) -> bool:
    section = markdown_section(readme, "Evidence-based alternatives")
    if section is None:
        print("README doctest error: missing alternatives section", file=sys.stderr)
        return False
    volatile = re.compile(
        r"\b(downloads?|stars?|pricing?|prices?|costs?|memory|cold start|"
        r"binary size|install size|faster|fastest|most popular)\b",
        re.IGNORECASE,
    )
    if volatile.search(section):
        print(
            "README doctest error: alternatives contain a volatile claim",
            file=sys.stderr,
        )
        return False
    urls = tuple(
        destination
        for destination in markdown_destinations(section)
        if urlsplit(destination).scheme
    )
    if Counter(urls) != Counter(COMPARISON_EVIDENCE_USAGE):
        print(
            "README doctest error: comparison links differ from approved "
            f"official evidence, observed={urls!r}",
            file=sys.stderr,
        )
        return False
    rows: list[tuple[str, tuple[str, ...]]] = []
    for line in section.splitlines():
        if not line.startswith("|"):
            continue
        cells = tuple(cell.strip() for cell in line.strip("|").split("|"))
        if len(cells) != 9:
            print(
                "README doctest error: comparison table has a malformed row",
                file=sys.stderr,
            )
            return False
        if cells[0] in {"Project", "---"}:
            continue
        product = re.sub(r"^\[([^]]+)\]\([^)]+\)$", r"\1", cells[0])
        rows.append((product, cells[1:]))
    if tuple(product for product, _ in rows) != tuple(COMPARISON_ROWS):
        print(
            "README doctest error: comparison projects differ from the approved set",
            file=sys.stderr,
        )
        return False
    for product, cells in rows:
        if cells != COMPARISON_ROWS[product]:
            print(
                f"README doctest error: unsupported comparison claim for {product}",
                file=sys.stderr,
            )
            return False
    if section.count(ROOT_UNIQUENESS_CLAIM) != 1:
        print(
            "README doctest error: scoped comparison conclusion changed",
            file=sys.stderr,
        )
        return False
    return True


def check_official_links() -> bool:
    valid = True
    for url in sorted(COMPARISON_EVIDENCE):
        request = Request(url, headers={"User-Agent": "rdocx-readme-check/1"})
        try:
            with urlopen(request, timeout=20) as response:
                status = response.status
        except OSError as error:
            print(
                "README doctest error: official evidence did not resolve: "
                f"{url}: {error}",
                file=sys.stderr,
            )
            valid = False
            continue
        if not 200 <= status < 400:
            print(
                f"README doctest error: official evidence returned {status}: {url}",
                file=sys.stderr,
            )
            valid = False
    return valid


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


def cargo_metadata() -> dict[str, object] | None:
    result = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=REPO_ROOT,
        stdout=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return None
    return json.loads(result.stdout)


def package_readme(package: dict[str, object]) -> Path | None:
    value = package.get("readme")
    manifest = package.get("manifest_path")
    if not isinstance(value, str) or not isinstance(manifest, str):
        return None
    return (Path(manifest).parent / value).resolve()


def validate_local_patches(packages: list[object]) -> bool:
    expected: set[tuple[str, str]] = set()
    for package in packages:
        if not isinstance(package, dict) or package.get("publish") == []:
            continue
        name = package.get("name")
        manifest = package.get("manifest_path")
        if not isinstance(name, str) or not isinstance(manifest, str):
            print(
                "README doctest error: invalid publishable package metadata",
                file=sys.stderr,
            )
            return False
        try:
            package_path = Path(manifest).parent.resolve().relative_to(REPO_ROOT)
        except ValueError:
            print(
                f"README doctest error: {name} is outside the repository",
                file=sys.stderr,
            )
            return False
        expected.add((name, package_path.as_posix()))

    actual = set(LOCAL_PATCHES)
    if len(actual) != len(LOCAL_PATCHES) or actual != expected:
        missing = sorted(expected - actual)
        unexpected = sorted(actual - expected)
        print(
            "README doctest error: local patches differ from publishable "
            f"metadata, missing={missing!r}, unexpected={unexpected!r}",
            file=sys.stderr,
        )
        return False
    return True


def validate_package_archive(package: dict[str, object], readme: Path) -> bool:
    name = package["name"]
    version = package.get("version")
    if not isinstance(name, str) or not isinstance(version, str):
        print("README doctest error: invalid package identity", file=sys.stderr)
        return False
    command = [
        "cargo",
        "package",
        "--locked",
        "--allow-dirty",
        "--no-verify",
        "-p",
        name,
    ]
    for patch_name, patch_path in LOCAL_PATCHES:
        command.extend(
            ["--config", f'patch.crates-io.{patch_name}.path="{patch_path}"']
        )
    result = subprocess.run(
        command,
        cwd=REPO_ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        print(result.stderr, file=sys.stderr, end="")
        return False
    archive = REPO_ROOT / f"target/package/{name}-{version}.crate"
    if not archive.is_file():
        print(
            f"README doctest error: missing generated archive {archive}",
            file=sys.stderr,
        )
        return False
    with tarfile.open(archive, "r:gz") as package_archive:
        readmes = [
            member
            for member in package_archive.getmembers()
            if Path(member.name).name == "README.md"
        ]
        if len(readmes) != 1:
            print(
                f"README doctest error: {name} archive has {len(readmes)} "
                "README files",
                file=sys.stderr,
            )
            return False
        packaged = package_archive.extractfile(readmes[0])
        if packaged is None or packaged.read() != readme.read_bytes():
            print(
                f"README doctest error: {name} archive README differs from "
                f"{readme}",
                file=sys.stderr,
            )
            return False
    return True


def validate_inventory() -> bool:
    metadata = cargo_metadata()
    if metadata is None:
        return False
    packages = metadata.get("packages")
    if not isinstance(packages, list) or len(packages) != WORKSPACE_PACKAGE_COUNT:
        observed = len(packages) if isinstance(packages, list) else "invalid"
        print(
            f"README doctest error: expected {WORKSPACE_PACKAGE_COUNT} workspace "
            f"packages, found {observed}",
            file=sys.stderr,
        )
        return False

    valid = True
    readme_paths: set[Path] = set()
    publishable: list[tuple[dict[str, object], Path]] = []
    for package in packages:
        if not isinstance(package, dict) or not isinstance(package.get("name"), str):
            print("README doctest error: invalid Cargo package metadata", file=sys.stderr)
            valid = False
            continue
        name = package["name"]
        readme = package_readme(package)
        manifest_value = package.get("manifest_path")
        if not isinstance(manifest_value, str):
            print(
                f"README doctest error: {name} lacks a manifest path",
                file=sys.stderr,
            )
            valid = False
            continue
        manifest = Path(manifest_value)
        manifest_data = tomllib.loads(manifest.read_text(encoding="utf-8"))
        declared_readme = manifest_data.get("package", {}).get("readme")
        if not isinstance(declared_readme, str):
            print(
                f"README doctest error: {manifest} lacks an explicit package.readme",
                file=sys.stderr,
            )
            valid = False
        if readme is None or not readme.is_file():
            print(
                f"README doctest error: {name} does not declare an existing README",
                file=sys.stderr,
            )
            valid = False
            continue
        readme_paths.add(readme)
        text = readme.read_text(encoding="utf-8")
        if not text.startswith(f"# {name}\n"):
            print(
                f"README doctest error: {readme} must start with '# {name}'",
                file=sys.stderr,
            )
            valid = False
        if name != "rdocx":
            for heading in ("## Use it when", "## Relationship", "## Example"):
                if heading not in text:
                    print(
                        f"README doctest error: {readme} lacks {heading!r}",
                        file=sys.stderr,
                    )
                    valid = False
        if not EXAMPLE_FENCE.search(text):
            print(
                f"README doctest error: {readme} lacks a supported example fence",
                file=sys.stderr,
            )
            valid = False
        if package.get("publish") != []:
            publishable.append((package, readme))

    if len(readme_paths) != WORKSPACE_PACKAGE_COUNT:
        print(
            f"README doctest error: expected {WORKSPACE_PACKAGE_COUNT} distinct "
            f"README sources, found {len(readme_paths)}",
            file=sys.stderr,
        )
        valid = False
    if len(publishable) != PUBLISHABLE_PACKAGE_COUNT:
        print(
            f"README doctest error: expected {PUBLISHABLE_PACKAGE_COUNT} "
            f"publishable packages, found {len(publishable)}",
            file=sys.stderr,
        )
        valid = False
    if not validate_local_patches(packages):
        valid = False

    root_readme = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
    capability_source = (
        REPO_ROOT / "docs/hld/02-scope-and-non-goals.md"
    ).read_text(encoding="utf-8")
    if not validate_root_capability_claims(root_readme, capability_source):
        valid = False
    if not validate_root_narrative(root_readme):
        valid = False
    if not validate_crate_narratives():
        valid = False
    if not validate_root_versions(root_readme, metadata):
        valid = False
    rpptx_readme = (REPO_ROOT / "crates/rpptx/README.md").read_text(encoding="utf-8")
    if not validate_opt_in_security_claims(root_readme, rpptx_readme, metadata):
        valid = False
    if not validate_all_local_links():
        valid = False
    if not validate_comparison_evidence(root_readme):
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
    if valid:
        for package, readme in sorted(
            publishable, key=lambda item: str(item[0]["name"])
        ):
            if not validate_package_archive(package, readme):
                valid = False
    if valid:
        print(
            f"readme_doctests: {WORKSPACE_PACKAGE_COUNT} distinct workspace "
            f"READMEs and {PUBLISHABLE_PACKAGE_COUNT} publishable package "
            "inventories validated"
        )
    return valid


def build_rlibs(package: str, crate_names: tuple[str, ...]) -> dict[str, Path] | None:
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
    artifacts: dict[str, set[Path]] = {name: set() for name in crate_names}
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
        crate_name = target.get("name")
        if crate_name not in artifacts or "lib" not in target.get("crate_types", []):
            continue
        artifacts[crate_name].update(
            Path(filename).resolve()
            for filename in message.get("filenames", [])
            if filename.endswith(".rlib")
        )
    if result.returncode != 0:
        return None
    for crate_name, crate_artifacts in artifacts.items():
        if len(crate_artifacts) != 1:
            print(
                f"README doctest error: expected one {crate_name} rlib from "
                f"the {package} build, found "
                f"{sorted(str(path) for path in crate_artifacts)!r}",
                file=sys.stderr,
            )
            return None
    return {name: paths.pop() for name, paths in artifacts.items()}


def compile_readme(case: ReadmeCase) -> bool:
    if not validate_fences(case.readme, case.expected_rust_fences):
        return False

    crate_names = (case.crate_name,) + tuple(
        crate_name for _, crate_name in case.companion_crates
    )
    rlibs = build_rlibs(case.package, crate_names)
    if rlibs is None:
        return False
    resolved_rlibs = list(rlibs.items())
    rlib = rlibs[case.crate_name]
    dependency_dir = rlib.parent / "deps"
    if not dependency_dir.is_dir():
        dependency_dir = rlib.parent
    command = [
        "rustdoc",
        "--test",
        str(case.readme),
        "--crate-name",
        f"{case.crate_name}_readme",
        "--edition=2024",
        "-Dwarnings",
        "-L",
        f"dependency={dependency_dir}",
    ]
    for crate_name, crate_rlib in resolved_rlibs:
        command.extend(("--extern", f"{crate_name}={crate_rlib}"))
    result = subprocess.run(
        command,
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
    parser.add_argument("--check-official-links", action="store_true")
    args = parser.parse_args()
    if args.check_official_links:
        return 0 if check_official_links() else 1
    if args.readme is not None:
        case = ReadmeCase("rdocx", "rdocx", args.readme.resolve(), 3)
        return 0 if compile_readme(case) else 1

    if not validate_inventory():
        return 1
    return 0 if all(compile_readme(case) for case in README_CASES) else 1


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Sprint state machine.

Holds the resumable state of a sprint in `.claude/scratch/SNN-run.json`, and
enforces the transitions that `.claude/WORKFLOW.md` describes as law. The point
is that "an agent may not merge" is checkable rather than aspirational.

Subcommands:
    init SNN                     create the run state from CURRENT_SPRINT.md
    status [SNN]                 print the current state
    set-phase SNN PHASE          advance the sprint phase
    mark-feature SNN F-ID STATE  advance one feature's state and its worker facts
    record-review SNN N ...      record a sprint-review pass result
    record-verification SNN ...  record a /verify result
    validate-handoff PATH ...    check a worker handoff before integration
    close-preflight SNN          the checks /close-sprint requires
    release-notes TAG            validate or render reviewed release notes

Exit codes: 0 ok, 1 refused, 2 usage.
"""

from __future__ import annotations

import argparse
import json
import re
import string
import subprocess
import sys
import unicodedata
from html.parser import HTMLParser
from pathlib import Path

SCHEMA_VERSION = 2

REPO = Path(__file__).resolve().parent.parent
SCRATCH = REPO / ".claude" / "scratch"
HANDOFFS = REPO / ".claude" / "handoffs"
CURRENT_SPRINT = REPO / "docs" / "sprints" / "CURRENT_SPRINT.md"
BACKLOG = REPO / "docs" / "sprints" / "BACKLOG.md"
SPRINT_TRACKER = REPO / "docs" / "sprints" / "SPRINT_TRACKER.md"
AS_BUILT = REPO / "docs" / "sprints" / "AS_BUILT.md"
PLANS = REPO / ".claude" / "plans"
REVIEWS = REPO / ".claude" / "reviews"
CHANGELOG = REPO / "CHANGELOG.md"

SPRINT_RE = re.compile(r"^# Current Sprint, (S\d+(?:\.\d+)?)$", re.MULTILINE)
VALIDATION_ONLY_RE = re.compile(
    r"^\*\*Validation-only\*\*:\s*yes\s*$", re.MULTILINE | re.IGNORECASE
)
WAVE_ROW_RE = re.compile(
    r"^\|\s*(F-[A-Za-z0-9]+)\s*\|\s*([^|]+?)\s*\|\s*([SMLX]+)\s*\|\s*"
    r"(pending|in-progress|done|archived)\s*\|\s*([^|]*?)\s*\|$",
    re.MULTILINE,
)
BACKLOG_ROW_RE = re.compile(
    r"^\|\s*(F-[A-Za-z0-9]+)\s*\|[^|]*\|[^|]*\|\s*([SMLX]+)\s*\|\s*"
    r"(pending|in-progress|done|archived)\s*\|$",
    re.MULTILINE,
)
FID_RE = re.compile(r"^F-[A-Za-z0-9]+$")
SPRINT_ID_RE = re.compile(r"^S\d+(?:\.\d+)?$")
HANDOFF_FIELD_RE = re.compile(r"^\*\*([A-Za-z][A-Za-z -]*)\*\*:\s*(.+?)\s*$", re.MULTILINE)
SEMVER_COMPONENT_RE = r"(?:0|[1-9][0-9]*)"
RELEASE_TAG_RE = re.compile(
    rf"^(?:rpptx-)?v{SEMVER_COMPONENT_RE}\."
    rf"{SEMVER_COMPONENT_RE}\.{SEMVER_COMPONENT_RE}$"
)
RELEASE_NOTE_PLACEHOLDER_RE = re.compile(
    r"\b(?:TBD|TODO|FIXME|CHANGEME|PLACEHOLDER)\b|\?\?\?|\[insert\b",
    re.IGNORECASE,
)
RELEASE_NOTE_HEADINGS = (
    "Highlights",
    "Added",
    "Fixed",
    "Compatibility",
    "Contributors",
)
COMMONMARK_BLOCK_TAGS = (
    "address|article|aside|base|basefont|blockquote|body|caption|center|col|"
    "colgroup|dd|details|dialog|dir|div|dl|dt|fieldset|figcaption|figure|footer|"
    "form|frame|frameset|h1|h2|h3|h4|h5|h6|head|header|hr|html|iframe|legend|"
    "li|link|main|menu|menuitem|nav|noframes|ol|optgroup|option|p|param|search|"
    "section|summary|table|tbody|td|tfoot|th|thead|title|tr|track|ul"
)
COMMONMARK_BLOCK_TAG_RE = re.compile(
    rf"^ {{0,3}}</?(?:{COMMONMARK_BLOCK_TAGS})(?:[ \t/>]|$)",
    re.IGNORECASE,
)
COMMONMARK_COMPLETE_TAG_RE = re.compile(
    r"^ {0,3}(?:"
    r"</[A-Za-z][A-Za-z0-9-]*[ \t]*>|"
    r"<[A-Za-z][A-Za-z0-9-]*(?:[ \t]+[^<>]*)?[ \t]*/?>"
    r")[ \t]*$"
)
COMMONMARK_BOUNDED_RAW_HTML = (
    (re.compile(r"^ {0,3}<\?"), re.compile(r"\?>")),
    (re.compile(r"^ {0,3}<![A-Z]"), re.compile(r">")),
    (re.compile(r"^ {0,3}<!\[CDATA\["), re.compile(r"\]\]>")),
)
HTML_VOID_ELEMENTS = frozenset(
    (
        "area",
        "base",
        "br",
        "col",
        "embed",
        "hr",
        "img",
        "input",
        "link",
        "meta",
        "param",
        "source",
        "track",
        "wbr",
    )
)
COMMONMARK_REFERENCE_DEFINITION_RE = re.compile(
    r"^ {0,3}\[(?:\\.|[^\[\]\r\n]){1,999}\]:"
)
COMMONMARK_REFERENCE_TITLE_RE = re.compile(
    r"^ {0,3}(?:"
    r'"(?:\\.|[^"\r\n])*"|'
    r"'(?:\\.|[^'\r\n])*'|"
    r"\((?:\\.|[^)\r\n])*\)"
    r")[ \t]*$"
)
COMMONMARK_AUTOLINK_RE = re.compile(
    r"<(?P<label>"
    r"(?:[A-Za-z][A-Za-z0-9+.-]{1,31}:[^<>\s]*|"
    r"[A-Za-z0-9.!#$%&'*+/=?^_`{|}~-]+@[^<>\s@]+)"
    r")>"
)
COMMONMARK_ESCAPABLE_PUNCTUATION = frozenset(string.punctuation)
MARKDOWN_ESCAPE_GUARD = "\N{WORD JOINER}"

# Every field a worker must have filled in before /integrate-feature will look
# at its branch. A missing one means the worker stopped early, and integrating
# on the strength of a half-written handoff is how unreviewed code lands.
HANDOFF_REQUIRED = (
    "F-ID",
    "Owner",
    "Branch",
    "Worktree",
    "Base",
    "Head",
    "Design plan",
    "Microscope",
    "Verify",
    "Hash harness",
    "Test gate",
)

PHASES = (
    "design",
    "questions",
    "implementation",
    "integration",
    "verification",
    "review",
    "ready_to_close",
    "blocked",
    "closed",
)

FEATURE_STATES = (
    "pending",
    "draft",
    "approved",
    "running",
    "reviewed",
    "completed",
    "blocked",
    "carried",
)

FEATURE_TRANSITIONS: dict[str, set[str]] = {
    "pending": {"draft", "approved", "blocked", "carried"},
    "draft": {"approved", "blocked", "carried"},
    "approved": {"running", "blocked", "carried"},
    "running": {"reviewed", "blocked", "carried"},
    "reviewed": {"running", "completed", "blocked"},
    "completed": set(),
    "blocked": {"draft", "approved", "running", "reviewed", "completed", "carried"},
    "carried": set(),
}

MAX_REVIEW_PASSES = 3

# Worker facts `mark-feature` records alongside the state. The CLI flag is the
# lowercase name with hyphens, so `--integration-commit` writes
# `integration_commit`.
WORKER_FIELDS = (
    "wave",
    "branch",
    "worktree",
    "base",
    "head",
    "handoff",
    "integration_commit",
)


def die(msg: str) -> None:
    print(f"sprint_workflow: {msg}", file=sys.stderr)
    sys.exit(1)


def protect_markdown_escapes(source: str) -> str:
    """Protect visible CommonMark escapes before classifying raw HTML."""
    protected: list[str] = []
    index = 0
    while index < len(source):
        if (
            source[index] == "\\"
            and index + 1 < len(source)
            and source[index + 1] in COMMONMARK_ESCAPABLE_PUNCTUATION
        ):
            # The format-only guard cannot collide with Markdown syntax and
            # does not make an otherwise empty section meaningful.
            protected.append(MARKDOWN_ESCAPE_GUARD)
            index += 2
            continue
        protected.append(source[index])
        index += 1
    return "".join(protected)


def markdown_heading_lines(lines: list[str], prefix: str) -> list[tuple[int, str]]:
    """Return exact Markdown headings outside non-rendered Markdown contexts."""
    headings: list[tuple[int, str]] = []
    fence: tuple[str, int] | None = None
    in_comment = False
    raw_html_end: re.Pattern[str] | None = None
    raw_html_until_blank = False
    for index, line in enumerate(lines):
        without_newline = protect_markdown_escapes(line.rstrip("\r\n"))
        if fence is not None:
            marker, minimum_length = fence
            close = re.fullmatch(rf" {{0,3}}({re.escape(marker)}+)[ \t]*", without_newline)
            if close is not None and len(close.group(1)) >= minimum_length:
                fence = None
            continue
        if raw_html_end is not None:
            if raw_html_end.search(without_newline):
                raw_html_end = None
            continue
        if raw_html_until_blank:
            if not without_newline.strip():
                raw_html_until_blank = False
            continue

        visible: list[str] = []
        cursor = 0
        while cursor < len(without_newline):
            if in_comment:
                end = without_newline.find("-->", cursor)
                if end < 0:
                    visible.append(" " * (len(without_newline) - cursor))
                    cursor = len(without_newline)
                    continue
                visible.append(" " * (end + 3 - cursor))
                cursor = end + 3
                in_comment = False
                continue
            start = without_newline.find("<!--", cursor)
            if start < 0:
                visible.append(without_newline[cursor:])
                break
            visible.append(without_newline[cursor:start])
            visible.append(" " * 4)
            cursor = start + 4
            in_comment = True

        visible_line = "".join(visible)
        opening = re.match(r"^ {0,3}(`{3,}|~{3,})", visible_line)
        if opening is not None:
            marker = opening.group(1)
            fence = (marker[0], len(marker))
            continue
        raw_html = re.match(
            r"^ {0,3}<(?P<tag>script|pre|style|textarea)(?:[ \t>]|$)",
            visible_line,
            re.IGNORECASE,
        )
        if raw_html is not None:
            tag = raw_html.group("tag").lower()
            close = re.compile(rf"</{re.escape(tag)}\s*>", re.IGNORECASE)
            if close.search(visible_line[raw_html.end() :]) is None:
                raw_html_end = close
            continue
        matched_bounded_raw_html = False
        for start_pattern, close in COMMONMARK_BOUNDED_RAW_HTML:
            start = start_pattern.match(visible_line)
            if start is None:
                continue
            if close.search(visible_line[start.end() :]) is None:
                raw_html_end = close
            matched_bounded_raw_html = True
            break
        if matched_bounded_raw_html:
            continue
        if COMMONMARK_BLOCK_TAG_RE.match(visible_line) is not None:
            raw_html_until_blank = True
            continue
        if COMMONMARK_COMPLETE_TAG_RE.fullmatch(visible_line) is not None:
            raw_html_until_blank = True
            continue
        if visible_line.startswith(prefix):
            headings.append((index, visible_line.rstrip()[len(prefix) :]))
    return headings


class MarkdownOutsideRawHtml(HTMLParser):
    """Collect Markdown source that is not contained by a raw HTML element."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.hidden_tags: list[str] = []
        self.text: list[str] = []

    def handle_starttag(
        self, tag: str, attrs: list[tuple[str, str | None]]
    ) -> None:
        del attrs
        tag = tag.lower()
        if tag not in HTML_VOID_ELEMENTS:
            self.hidden_tags.append(tag)

    def handle_endtag(self, tag: str) -> None:
        tag = tag.lower()
        for index in range(len(self.hidden_tags) - 1, -1, -1):
            if self.hidden_tags[index] == tag:
                del self.hidden_tags[index:]
                break

    def handle_data(self, data: str) -> None:
        if not self.hidden_tags:
            self.text.append(data)


def closing_markdown_delimiter(
    source: str, opening_index: int, opening: str, closing: str
) -> int | None:
    """Find one escaped, balanced Markdown delimiter within bounded source."""
    depth = 1
    index = opening_index + 1
    while index < len(source):
        character = source[index]
        if character == "\\":
            index += 2
            continue
        if character == opening:
            depth += 1
        elif character == closing:
            depth -= 1
            if depth == 0:
                return index
        index += 1
    return None


def markdown_without_link_destinations(source: str) -> str:
    """Keep rendered labels while removing inline and reference destinations."""
    visible: list[str] = []
    index = 0
    while index < len(source):
        if source[index] == "\\" and index + 1 < len(source):
            visible.append(source[index : index + 2])
            index += 2
            continue
        image = source[index] == "!" and source[index + 1 : index + 2] == "["
        label_start = index + 1 if image else index
        if source[label_start : label_start + 1] != "[":
            visible.append(source[index])
            index += 1
            continue
        label_end = closing_markdown_delimiter(source, label_start, "[", "]")
        if label_end is None:
            visible.append(source[index])
            index += 1
            continue
        suffix_start = label_end + 1
        suffix_end: int | None = None
        if source[suffix_start : suffix_start + 1] == "(":
            suffix_end = closing_markdown_delimiter(source, suffix_start, "(", ")")
        elif source[suffix_start : suffix_start + 1] == "[":
            suffix_end = closing_markdown_delimiter(source, suffix_start, "[", "]")
        if suffix_end is None:
            visible.append(source[index : label_end + 1])
            index = label_end + 1
            continue
        visible.append(source[label_start + 1 : label_end])
        index = suffix_end + 1
    return "".join(visible)


def markdown_code_span(source: str, index: int) -> tuple[int, int] | None:
    """Return the content bounds for a code span beginning at index."""
    run_end = index
    while run_end < len(source) and source[run_end] == "`":
        run_end += 1
    marker = source[index:run_end]
    close = source.find(marker, run_end)
    while close >= 0 and (
        source[close - 1 : close] == "`"
        or source[close + len(marker) : close + len(marker) + 1] == "`"
    ):
        close = source.find(marker, close + len(marker))
    if close < 0:
        return None
    return run_end, close


def markdown_without_inline_code(source: str) -> tuple[str, bool]:
    """Remove code-span syntax and report whether a span renders code content."""
    visible: list[str] = []
    index = 0
    while index < len(source):
        if source[index] != "`":
            visible.append(source[index])
            index += 1
            continue
        bounds = markdown_code_span(source, index)
        if bounds is None:
            run_end = index
            while run_end < len(source) and source[run_end] == "`":
                run_end += 1
            marker = source[index:run_end]
            visible.append(marker)
            index = run_end
            continue
        run_end, close = bounds
        marker = source[index:run_end]
        code = source[run_end:close].replace("\n", " ")
        if code.startswith(" ") and code.endswith(" ") and code.strip():
            code = code[1:-1]
        if contains_meaningful_rendered_character(code):
            return "", True
        index = close + len(marker)
    return "".join(visible), False


def escape_html_syntax(source: str) -> str:
    """Escape code text only for the internal HTML visibility pass."""
    return source.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def protect_inline_code_from_html(source: str) -> str:
    """Keep HTML-like code-span content visible to the Markdown pass."""
    protected: list[str] = []
    index = 0
    while index < len(source):
        if source[index] != "`":
            protected.append(source[index])
            index += 1
            continue
        bounds = markdown_code_span(source, index)
        if bounds is None:
            run_end = index
            while run_end < len(source) and source[run_end] == "`":
                run_end += 1
            marker = source[index:run_end]
            protected.append(marker)
            index = run_end
            continue
        run_end, close = bounds
        marker = source[index:run_end]
        protected.append(marker)
        protected.append(escape_html_syntax(source[run_end:close]))
        protected.append(marker)
        index = close + len(marker)
    return "".join(protected)


def protect_markdown_code_from_html(section: str) -> str:
    """Protect fenced and inline code before parsing surrounding raw HTML."""
    protected: list[str] = []
    prose: list[str] = []
    fence: tuple[str, int] | None = None

    def flush_prose() -> None:
        if prose:
            protected.append(protect_inline_code_from_html("".join(prose)))
            prose.clear()

    for line in section.splitlines(keepends=True):
        without_newline = line.rstrip("\r\n")
        if fence is not None:
            marker, minimum_length = fence
            close = re.fullmatch(
                rf" {{0,3}}({re.escape(marker)}+)[ \t]*", without_newline
            )
            if close is not None and len(close.group(1)) >= minimum_length:
                protected.append(line)
                fence = None
            else:
                protected.append(escape_html_syntax(line))
            continue
        opening = re.match(r"^ {0,3}(`{3,}|~{3,})", without_newline)
        if opening is not None:
            marker = opening.group(1)
            info = without_newline[opening.end() :]
            if marker[0] == "~" or "`" not in info:
                flush_prose()
                protected.append(line)
                fence = (marker[0], len(marker))
                continue
        prose.append(line)
    flush_prose()
    return "".join(protected)


def markdown_prose_and_code(section: str) -> tuple[str, bool]:
    """Remove non-rendered block syntax and report visible fenced code."""
    prose: list[str] = []
    fence: tuple[str, int] | None = None
    reference_continuation: str | None = None
    for line in section.splitlines(keepends=True):
        without_newline = line.rstrip("\r\n")
        if fence is not None:
            marker, minimum_length = fence
            close = re.fullmatch(
                rf" {{0,3}}({re.escape(marker)}+)[ \t]*", without_newline
            )
            if close is not None and len(close.group(1)) >= minimum_length:
                fence = None
            elif contains_meaningful_rendered_character(without_newline):
                return "", True
            continue
        if reference_continuation is not None:
            if not without_newline.strip():
                reference_continuation = None
                continue
            if reference_continuation == "destination":
                reference_continuation = "title"
                continue
            if COMMONMARK_REFERENCE_TITLE_RE.fullmatch(without_newline) is not None:
                reference_continuation = None
                continue
            reference_continuation = None
        opening = re.match(r"^ {0,3}(`{3,}|~{3,})", without_newline)
        if opening is not None:
            marker = opening.group(1)
            info = without_newline[opening.end() :]
            if marker[0] == "~" or "`" not in info:
                fence = (marker[0], len(marker))
                continue
        reference = COMMONMARK_REFERENCE_DEFINITION_RE.match(without_newline)
        if reference is not None:
            reference_continuation = (
                "destination"
                if not without_newline[reference.end() :].strip()
                else "title"
            )
            continue
        prose.append(line)
    return "".join(prose), False


def markdown_without_block_markers(source: str) -> str:
    """Remove list and quote markers that do not represent section text."""
    visible: list[str] = []
    for line in source.splitlines(keepends=True):
        while True:
            quote = re.match(r"^ {0,3}>[ \t]?", line)
            if quote is None:
                break
            line = line[quote.end() :]
        while True:
            marker = re.match(r"^ {0,3}(?:[-+*]|[0-9]{1,9}[.)])(?:[ \t]+|$)", line)
            if marker is None:
                break
            line = line[marker.end() :]
        task = re.match(r"^\[[ xX]\](?:[ \t]+|$)", line)
        if task is not None:
            line = line[task.end() :]
        visible.append(line)
    return "".join(visible)


def contains_meaningful_rendered_character(source: str) -> bool:
    """Accept alphanumeric text and pictographs, but not Markdown syntax alone."""
    return any(
        character.isalnum() or unicodedata.category(character) == "So"
        for character in source
    )


def has_meaningful_visible_text(section: str) -> bool:
    """Return whether one bounded section renders meaningful HTML or Markdown."""
    section = protect_markdown_escapes(section)
    section = protect_markdown_code_from_html(section)
    section = COMMONMARK_AUTOLINK_RE.sub(lambda match: match.group("label"), section)
    parser = MarkdownOutsideRawHtml()
    parser.feed(section)
    parser.close()
    prose, fenced_code = markdown_prose_and_code("".join(parser.text))
    if fenced_code:
        return True
    prose, inline_code = markdown_without_inline_code(prose)
    if inline_code:
        return True
    prose = markdown_without_link_destinations(prose)
    prose = markdown_without_block_markers(prose)
    return contains_meaningful_rendered_character(prose)


def render_release_notes(changelog: str, tag: str) -> str:
    """Validate and return one reviewed changelog section body."""
    if not RELEASE_TAG_RE.fullmatch(tag):
        raise ValueError(
            f"{tag!r} is not a release tag, expected vX.Y.Z or rpptx-vX.Y.Z"
        )

    lines = changelog.splitlines(keepends=True)
    release_matches = [
        index
        for index, heading in markdown_heading_lines(lines, "## ")
        if heading == tag
    ]
    if len(release_matches) != 1:
        raise ValueError(
            f"CHANGELOG.md must contain exactly one `## {tag}` heading, "
            f"found {len(release_matches)}"
        )

    start = release_matches[0] + 1
    later_sections = [
        index
        for index, _ in markdown_heading_lines(lines[start:], "## ")
    ]
    end = start + later_sections[0] if later_sections else len(lines)
    body_lines = lines[start:end]
    while body_lines and not body_lines[0].strip():
        body_lines.pop(0)
    while body_lines and not body_lines[-1].strip():
        body_lines.pop()
    body = "".join(body_lines)
    if not body.strip():
        raise ValueError(f"`## {tag}` has no release-note body")
    if RELEASE_NOTE_PLACEHOLDER_RE.search(body):
        raise ValueError(f"`## {tag}` contains a placeholder token")

    headings = markdown_heading_lines(body_lines, "### ")
    found = [heading for _, heading in headings]
    for required in RELEASE_NOTE_HEADINGS:
        count = found.count(required)
        if count != 1:
            raise ValueError(
                f"`## {tag}` must contain exactly one `### {required}` heading, "
                f"found {count}"
            )
    required_positions = [found.index(required) for required in RELEASE_NOTE_HEADINGS]
    if required_positions != sorted(required_positions):
        raise ValueError(f"`## {tag}` required headings are out of order")

    for heading_index, (line_index, heading) in enumerate(headings):
        if heading not in RELEASE_NOTE_HEADINGS:
            continue
        next_line = (
            headings[heading_index + 1][0]
            if heading_index + 1 < len(headings)
            else len(body_lines)
        )
        section_text = "".join(body_lines[line_index + 1 : next_line])
        if not has_meaningful_visible_text(section_text):
            raise ValueError(f"`## {tag}` section `### {heading}` is empty")

    return body.rstrip() + "\n"


def cmd_release_notes(args: argparse.Namespace) -> int:
    try:
        notes = render_release_notes(CHANGELOG.read_text(encoding="utf-8"), args.tag)
    except (OSError, ValueError) as error:
        die(f"release notes: {error}")
    if args.render:
        sys.stdout.write(notes)
    else:
        print(f"release-notes {args.tag}: ok")
    return 0


def state_path(sprint: str) -> Path:
    return SCRATCH / f"{sprint}-run.json"


def load(sprint: str) -> dict:
    p = state_path(sprint)
    if not p.exists():
        die(f"no run state for {sprint}. Run `init {sprint}` first.")
    data = json.loads(p.read_text(encoding="utf-8"))
    if data.get("schema_version") != SCHEMA_VERSION:
        die(
            f"run state schema {data.get('schema_version')} != {SCHEMA_VERSION}. "
            "Delete the file and re-init."
        )
    return data


def save(sprint: str, data: dict) -> None:
    SCRATCH.mkdir(parents=True, exist_ok=True)
    state_path(sprint).write_text(
        json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


def git_head() -> str:
    """Return the commit whose evidence is being recorded or checked."""
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=REPO,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def parse_current_sprint() -> tuple[str, list[dict]]:
    if not CURRENT_SPRINT.exists():
        die(f"{CURRENT_SPRINT} not found")
    text = CURRENT_SPRINT.read_text(encoding="utf-8")
    m = SPRINT_RE.search(text)
    if not m:
        die("CURRENT_SPRINT.md has no parseable `# Current Sprint, SNN` heading")
    features = [
        {"fid": r[0], "title": r[1], "size": r[2], "tracker_status": r[3], "owner": r[4]}
        for r in WAVE_ROW_RE.findall(text)
    ]
    if not features and not VALIDATION_ONLY_RE.search(text):
        die(
            "CURRENT_SPRINT.md has no parseable wave rows. An intentionally "
            "empty sprint must declare `**Validation-only**: yes`."
        )
    return m.group(1), features


def backlog_statuses() -> dict[str, str]:
    if not BACKLOG.exists():
        die(f"{BACKLOG} not found")
    return {r[0]: r[2] for r in BACKLOG_ROW_RE.findall(BACKLOG.read_text(encoding="utf-8"))}


def completed_record_problems(sprint: str, fid: str) -> list[str]:
    """Return missing or stale durable records for one completed feature."""
    problems: list[str] = []
    current_id, current_features = parse_current_sprint()
    current = {feature["fid"]: feature for feature in current_features}
    feature = current.get(fid)
    if current_id != sprint:
        problems.append(f"CURRENT_SPRINT.md says {current_id}, run state says {sprint}")
    elif feature is None:
        problems.append(f"{fid} has no CURRENT_SPRINT.md row")
    else:
        if feature["tracker_status"] != "done":
            problems.append(
                f"{fid} is completed in run state but "
                f"'{feature['tracker_status']}' in CURRENT_SPRINT.md"
            )
        if feature["owner"] not in ("", "-"):
            problems.append(
                f"{fid} is completed but CURRENT_SPRINT.md owner is "
                f"'{feature['owner']}'"
            )

    plan = PLANS / f"{fid}-design.md"
    if not plan.exists():
        problems.append(f"{fid} has no design plan at {plan.relative_to(REPO)}")
    elif "**Status**: completed" not in plan.read_text(encoding="utf-8"):
        problems.append(f"{fid} design plan is not completed")

    if not AS_BUILT.exists() or not re.search(
        rf"^### {re.escape(fid)},", AS_BUILT.read_text(encoding="utf-8"), re.MULTILINE
    ):
        problems.append(f"{fid} has no AS_BUILT.md entry")

    tracker_text = (
        SPRINT_TRACKER.read_text(encoding="utf-8") if SPRINT_TRACKER.exists() else ""
    )
    if not re.search(
        rf"^\|\s*{re.escape(fid)}\s*\|\s*{re.escape(sprint)}\s*\|",
        tracker_text,
        re.MULTILINE,
    ):
        problems.append(f"{fid} has no {sprint} row in SPRINT_TRACKER.md")
    return problems


def completed_owner_problems(data: dict) -> list[str]:
    """Return completed features whose run-state owner was not cleared."""
    return [
        f"{fid} is completed but run-state owner is '{feature['owner']}'"
        for fid, feature in data["features"].items()
        if feature["state"] == "completed" and feature.get("owner") is not None
    ]


def cmd_init(args: argparse.Namespace) -> int:
    sprint, features = parse_current_sprint()
    if args.sprint != sprint:
        die(f"CURRENT_SPRINT.md says {sprint}, you asked for {args.sprint}")

    exists = state_path(sprint).exists()
    if exists and args.resume:
        # The point of --resume is that an interrupted /run-sprint picks up
        # where it stopped rather than redoing landed work. Adopt the limits
        # from this invocation, keep every feature and review record.
        data = load(sprint)
        data["max_review_passes"] = args.max_review_passes
        data["max_workers"] = args.max_workers
        for f in features:
            data["features"].setdefault(
                f["fid"],
                {
                    "state": "pending",
                    "size": f["size"],
                    "title": f["title"],
                    "owner": f["owner"] if f["owner"] != "-" else None,
                },
            )
        save(sprint, data)
        print(
            f"resumed {sprint}, phase={data['phase']}, "
            f"{len(data['features'])} features, {len(data['reviews'])} review pass(es)"
        )
        return 0
    if exists and not args.force:
        die(
            f"run state for {sprint} already exists. Pass --resume to continue "
            "it, or --force to throw it away and recreate."
        )

    data = {
        "schema_version": SCHEMA_VERSION,
        "sprint": sprint,
        "phase": "design",
        "max_review_passes": args.max_review_passes,
        "max_workers": args.max_workers,
        "features": {
            f["fid"]: {
                "state": "pending",
                "size": f["size"],
                "title": f["title"],
                "owner": f["owner"] if f["owner"] != "-" else None,
            }
            for f in features
        },
        "reviews": [],
        "verifications": [],
    }
    save(sprint, data)
    print(f"initialised {sprint} with {len(features)} features, phase=design")
    return 0


def cmd_status(args: argparse.Namespace) -> int:
    sprint = args.sprint or parse_current_sprint()[0]
    data = load(sprint)
    print(
        f"{sprint}  phase={data['phase']}  "
        f"max-review-passes={data.get('max_review_passes', MAX_REVIEW_PASSES)}  "
        f"max-workers={data.get('max_workers') or 'unbounded'}"
    )
    counts: dict[str, int] = {}
    for fid, f in sorted(data["features"].items()):
        counts[f["state"]] = counts.get(f["state"], 0) + 1
        owner = f" ({f['owner']})" if f.get("owner") else ""
        wave = f" w{f['wave']}" if f.get("wave") is not None else ""
        print(f"  {fid:<8} {f['size']:<2}{wave} {f['state']}{owner}  {f['title']}")
        if args.workers:
            for field in WORKER_FIELDS:
                if field != "wave" and f.get(field):
                    print(f"      {field:<19} {f[field]}")
    summary = ", ".join(f"{k}={v}" for k, v in sorted(counts.items()))
    print("  " + (summary or "features=0"))
    if data["reviews"]:
        last = data["reviews"][-1]
        print(f"  reviews: {len(data['reviews'])} passes, last blocking={last['blocking']}")
    return 0


def cmd_set_phase(args: argparse.Namespace) -> int:
    if args.phase not in PHASES:
        die(f"unknown phase {args.phase}. One of: {', '.join(PHASES)}")
    data = load(args.sprint)
    if data["phase"] == "closed":
        die(f"{args.sprint} is closed. Phases do not move after that.")
    data["phase"] = args.phase
    save(args.sprint, data)
    print(f"{args.sprint} phase={args.phase}")
    return 0


def cmd_mark_feature(args: argparse.Namespace) -> int:
    if not FID_RE.match(args.fid):
        die(f"{args.fid} is not a valid F-ID")
    if args.state not in FEATURE_STATES:
        die(f"unknown state {args.state}. One of: {', '.join(FEATURE_STATES)}")
    data = load(args.sprint)
    f = data["features"].get(args.fid)
    if f is None:
        die(f"{args.fid} is not in {args.sprint}")
    old = f["state"]
    allowed = FEATURE_TRANSITIONS[old]
    if args.state != old and args.state not in allowed:
        die(
            f"{args.fid}: {old} -> {args.state} is not a legal transition. "
            f"From {old} you may go to: {', '.join(sorted(allowed)) or '(nothing)'}"
        )
    f["state"] = args.state
    if args.owner:
        f["owner"] = args.owner
    if args.clear_owner:
        f["owner"] = None
    for field in WORKER_FIELDS:
        value = getattr(args, field, None)
        if value is not None:
            f[field] = value
    save(args.sprint, data)
    print(f"{args.fid}: {old} -> {args.state}")
    return 0


def cmd_record_review(args: argparse.Namespace) -> int:
    data = load(args.sprint)
    head = git_head()
    bound = data.get("max_review_passes", MAX_REVIEW_PASSES)
    if args.passno > bound and not args.extend:
        die(
            f"pass {args.passno} exceeds the bound of {bound}. "
            "Another pass means the sprint is not ready. Carry work, or pass "
            "--extend and record why in the review file."
        )
    data["reviews"].append(
        {
            "pass": args.passno,
            "blocking": args.blocking,
            "should_fix": args.should_fix,
            "nice_to_have": args.nice_to_have,
            "head": head,
        }
    )
    save(args.sprint, data)
    verdict = "clean" if args.blocking == 0 else f"{args.blocking} blocking"
    print(
        f"{args.sprint} review pass {args.passno} recorded at {head[:12]}: {verdict}"
    )
    return 0


def cmd_record_verification(args: argparse.Namespace) -> int:
    data = load(args.sprint)
    head = git_head()
    data["verifications"].append(
        {
            "scope": args.scope,
            "passed": args.passed,
            "harness": args.harness,
            "head": head,
        }
    )
    save(args.sprint, data)
    print(
        f"{args.sprint} verification recorded at {head[:12]}: "
        f"{args.scope}, passed={args.passed}"
    )
    return 0


def closure_evidence_problems(data: dict, head: str) -> list[str]:
    """Return stale or missing review and verification evidence for HEAD."""
    problems: list[str] = []
    if not data["reviews"]:
        problems.append("no sprint-review pass recorded")
    else:
        review = data["reviews"][-1]
        if review["blocking"] != 0:
            problems.append(
                f"last review pass has {review['blocking']} blocking finding(s)"
            )
        if review.get("head") != head:
            problems.append(
                "latest sprint review covered "
                f"{review.get('head') or 'an unrecorded HEAD'}, current HEAD is {head}"
            )

    verified = [
        verification
        for verification in data["verifications"]
        if verification["scope"] == "full"
        and verification["passed"]
        and verification.get("head") == head
    ]
    if not verified:
        problems.append(f"no passing `/verify --full` recorded for current HEAD {head}")
    return problems


def cmd_validate_handoff(args: argparse.Namespace) -> int:
    """Check a worker's `.claude/handoffs/F-XXX-ready.md` before integration.

    This is a shape check, not a trust check. It catches the half-written
    handoff. Whether the claims inside are true is what /integrate-feature
    verifies against the branch.
    """
    path = Path(args.path)
    if not path.exists():
        die(f"{path} not found. The worker never ran `/complete-feature --prepare`.")

    fields = dict(HANDOFF_FIELD_RE.findall(path.read_text(encoding="utf-8")))
    problems: list[str] = []

    for key in HANDOFF_REQUIRED:
        if not fields.get(key):
            problems.append(f"missing or empty field: **{key}**")

    if fields.get("F-ID") and fields["F-ID"] != args.fid:
        problems.append(f"handoff is for {fields['F-ID']}, you asked for {args.fid}")

    microscope = fields.get("Microscope", "")
    if microscope and not ("0 defects" in microscope and "0 smells" in microscope):
        problems.append(
            "Microscope must cite a pass reporting '0 defects, 0 smells'. "
            f"It says: {microscope}"
        )

    verify = fields.get("Verify", "")
    if verify and "fail" in verify.lower():
        problems.append(f"Verify records a failure: {verify}")
    if verify and "--fast" in verify:
        problems.append(
            "Verify records a --fast run. --fast is the inner loop, not the gate."
        )

    if problems:
        print(f"validate-handoff {path}: REFUSED", file=sys.stderr)
        for p in problems:
            print(f"  - {p}", file=sys.stderr)
        return 1

    print(
        f"validate-handoff {args.fid}: ok, "
        f"{fields['Base'][:12]}..{fields['Head'][:12]} on {fields['Branch']}"
    )
    return 0


def cmd_close_preflight(args: argparse.Namespace) -> int:
    data = load(args.sprint)
    problems: list[str] = []
    head = git_head()

    if data["phase"] == "blocked":
        problems.append("sprint is blocked. Resolve what blocked it and re-run.")

    # /integrate-feature deletes each handoff it consumed. One still sitting
    # here means a worker was prepared and never integrated.
    stale = sorted(p.name for p in HANDOFFS.glob("F-*-ready.md"))
    if stale:
        problems.append("unconsumed worker handoffs: " + ", ".join(stale))

    incomplete = [
        fid
        for fid, f in data["features"].items()
        if f["state"] not in ("completed", "carried")
    ]
    if incomplete:
        problems.append(
            "features neither completed nor carried: " + ", ".join(sorted(incomplete))
        )
    problems.extend(completed_owner_problems(data))

    problems.extend(closure_evidence_problems(data, head))

    # The durable delivery record must agree with the run state. This catches a
    # completion that updated the JSON but missed one of the canonical ledgers.
    statuses = backlog_statuses()
    for fid, f in data["features"].items():
        want = "done" if f["state"] == "completed" else None
        if want and statuses.get(fid) != want:
            problems.append(
                f"{fid} is completed in run state but '{statuses.get(fid)}' in BACKLOG.md"
            )
        if want:
            problems.extend(completed_record_problems(args.sprint, fid))

    if data["reviews"]:
        last_pass = data["reviews"][-1]["pass"]
        review = REVIEWS / f"{args.sprint}-sprint-review-pass-{last_pass}.md"
        if not review.exists():
            problems.append(
                f"latest sprint review file is missing: {review.relative_to(REPO)}"
            )
        elif not re.search(
            r"^\*\*Verdict\*\*:\s*0 blocking,",
            review.read_text(encoding="utf-8"),
            re.MULTILINE,
        ):
            problems.append(
                "latest sprint review file does not report zero blocking: "
                f"{review.relative_to(REPO)}"
            )

    if problems:
        print(f"close-preflight {args.sprint}: REFUSED", file=sys.stderr)
        for p in problems:
            print(f"  - {p}", file=sys.stderr)
        return 1

    print(f"close-preflight {args.sprint}: ok, ready to close")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("init"); p.add_argument("sprint"); p.add_argument("--force", action="store_true"); p.add_argument("--resume", action="store_true"); p.add_argument("--max-review-passes", type=int, default=MAX_REVIEW_PASSES); p.add_argument("--max-workers", type=int, default=None); p.set_defaults(fn=cmd_init)
    p = sub.add_parser("status"); p.add_argument("sprint", nargs="?"); p.add_argument("--workers", action="store_true", help="show branch, worktree and handoff per feature"); p.set_defaults(fn=cmd_status)
    p = sub.add_parser("set-phase"); p.add_argument("sprint"); p.add_argument("phase"); p.set_defaults(fn=cmd_set_phase)
    p = sub.add_parser("mark-feature"); p.add_argument("sprint"); p.add_argument("fid"); p.add_argument("state"); p.add_argument("--owner"); p.add_argument("--clear-owner", action="store_true"); p.add_argument("--wave", type=int); p.add_argument("--branch"); p.add_argument("--worktree"); p.add_argument("--base"); p.add_argument("--head"); p.add_argument("--handoff"); p.add_argument("--integration-commit"); p.set_defaults(fn=cmd_mark_feature)
    p = sub.add_parser("record-review"); p.add_argument("sprint"); p.add_argument("passno", type=int); p.add_argument("--blocking", type=int, required=True); p.add_argument("--should-fix", type=int, default=0); p.add_argument("--nice-to-have", type=int, default=0); p.add_argument("--extend", action="store_true"); p.set_defaults(fn=cmd_record_review)
    p = sub.add_parser("record-verification"); p.add_argument("sprint"); p.add_argument("--scope", choices=["fast", "feature", "full"], required=True); p.add_argument("--passed", action="store_true"); p.add_argument("--harness", default="unchecked"); p.set_defaults(fn=cmd_record_verification)
    p = sub.add_parser("validate-handoff"); p.add_argument("path"); p.add_argument("--fid", required=True); p.set_defaults(fn=cmd_validate_handoff)
    p = sub.add_parser("close-preflight"); p.add_argument("sprint"); p.set_defaults(fn=cmd_close_preflight)
    p = sub.add_parser("release-notes"); p.add_argument("tag"); mode = p.add_mutually_exclusive_group(required=True); mode.add_argument("--check", action="store_true"); mode.add_argument("--render", action="store_true"); p.set_defaults(fn=cmd_release_notes)

    args = ap.parse_args()
    if getattr(args, "fid", None) and not FID_RE.match(args.fid):
        die(f"{args.fid} is not a valid F-ID")
    if hasattr(args, "sprint") and args.sprint and not SPRINT_ID_RE.match(args.sprint):
        die(f"{args.sprint} is not a valid sprint id, expected SNN")
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())

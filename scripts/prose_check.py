#!/usr/bin/env python3
"""Voice rules gate.

Enforces two rules over tracked prose, per `.claude/WORKFLOW.md`:

  - No em-dash. Use a comma, a hyphen, or rewrite the sentence.
  - No semicolon in prose. Use a full stop or a comma.

Prose only. Code is exempt, so fenced code blocks, indented code blocks and
inline spans are skipped. Rust and Python semicolons are not a style question.

Usage:
    python3 scripts/prose_check.py                 # all tracked markdown
    python3 scripts/prose_check.py --staged        # staged files only
    python3 scripts/prose_check.py --commit-msg F  # a commit message file
    python3 scripts/prose_check.py path/to/file.md
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

# Directories whose markdown is gated. Everything else is advisory.
# `.agents/` is deliberately absent. It is generated from these sources by
# scripts/sync_agent_skills.py, so gating it would report the same violation
# twice and invite someone to fix the copy.
GATED_PREFIXES = (
    "docs/",
    ".claude/plans/",
    ".claude/reviews/",
    ".claude/commands/",
    ".claude/skills/",
)
GATED_FILES = ("CLAUDE.md", "AGENTS.md", ".claude/WORKFLOW.md")

EM_DASH = "—"
EN_DASH_RANGE = "–"

FENCE_RE = re.compile(r"^\s*(```|~~~)")
INLINE_CODE_RE = re.compile(r"`[^`]*`")
# A markdown table row is prose, but its pipes are not sentence punctuation.
# Semicolons inside a URL or an HTML entity are not prose either.
ENTITY_RE = re.compile(r"&[A-Za-z0-9#]+;")
URL_RE = re.compile(r"<?https?://\S+>?")


def is_gated(path: str) -> bool:
    return path.startswith(GATED_PREFIXES) or path in GATED_FILES


def strip_code(text: str) -> str:
    """Blank out anything that is code, preserving line numbers."""
    out: list[str] = []
    in_fence = False
    for line in text.splitlines():
        if FENCE_RE.match(line):
            in_fence = not in_fence
            out.append("")
            continue
        if in_fence:
            out.append("")
            continue
        # An indented code block: four spaces or a tab, outside a list context
        # is ambiguous in markdown, so only treat a tab or 4+ spaces followed by
        # a non-list character as code.
        if re.match(r"^(\t| {4,})(?![-*+]\s|\d+\.\s)", line):
            out.append("")
            continue
        line = INLINE_CODE_RE.sub(lambda m: " " * len(m.group(0)), line)
        line = ENTITY_RE.sub(lambda m: " " * len(m.group(0)), line)
        line = URL_RE.sub(lambda m: " " * len(m.group(0)), line)
        out.append(line)
    return "\n".join(out)


def check_text(text: str, label: str) -> list[str]:
    findings: list[str] = []
    for n, line in enumerate(strip_code(text).splitlines(), start=1):
        col = line.find(EM_DASH)
        if col >= 0:
            findings.append(
                f"{label}:{n}:{col + 1}: em-dash. Use a comma, a hyphen, or rewrite."
            )
        col = line.find(EN_DASH_RANGE)
        if col >= 0:
            findings.append(
                f"{label}:{n}:{col + 1}: en-dash. Use 'to' for ranges, or a hyphen."
            )
        col = line.find(";")
        if col >= 0:
            findings.append(
                f"{label}:{n}:{col + 1}: semicolon in prose. Use a full stop or a comma."
            )
    return findings


def tracked_markdown() -> list[str]:
    out = subprocess.run(
        ["git", "ls-files", "*.md"], capture_output=True, text=True, check=True
    ).stdout
    return [p for p in out.splitlines() if is_gated(p)]


def staged_markdown() -> list[str]:
    out = subprocess.run(
        ["git", "diff", "--cached", "--name-only", "--diff-filter=ACM"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    return [p for p in out.splitlines() if p.endswith(".md") and is_gated(p)]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("paths", nargs="*", help="explicit files to check")
    ap.add_argument("--staged", action="store_true", help="staged files only")
    ap.add_argument("--commit-msg", metavar="FILE", help="check a commit message")
    args = ap.parse_args()

    findings: list[str] = []

    if args.commit_msg:
        p = Path(args.commit_msg)
        # Ignore the comment block git appends.
        body = "\n".join(
            l for l in p.read_text(encoding="utf-8").splitlines() if not l.startswith("#")
        )
        findings += check_text(body, "commit message")
    elif args.paths:
        targets = args.paths
        for t in targets:
            findings += check_text(Path(t).read_text(encoding="utf-8"), t)
    else:
        targets = staged_markdown() if args.staged else tracked_markdown()
        for t in targets:
            path = Path(t)
            if not path.exists():
                continue
            findings += check_text(path.read_text(encoding="utf-8"), t)

    if findings:
        for f in findings:
            print(f, file=sys.stderr)
        print(
            f"\nprose_check: {len(findings)} violation(s). "
            "See the voice rules in .claude/WORKFLOW.md.",
            file=sys.stderr,
        )
        return 1

    print("prose_check: 0 violations.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

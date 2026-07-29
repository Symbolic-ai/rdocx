---
description: The prose conventions /verify enforces. No em-dash, no en-dash, no prose semicolon, and exactly where those rules apply.
---

# Skill: voice-rules

Two punctuation rules govern tracked Markdown and commit messages. They are
arbitrary. The value is that a document set several sessions write into over
many months reads as one voice.

**`python3 scripts/prose_check.py` is the single source of truth.** This file
explains the rule. The script decides whether you broke it, and `/verify` step 6
runs it.

## The rules

1. **No em-dash (`U+2014`).** Replace it with one of:
   - A hyphen, for a compound modifier such as `pre-commit hook`.
   - A comma, for a parenthetical clause.
   - A full stop and a fresh sentence.
   - A rewrite that never needed the dash.

2. **No en-dash (`U+2013`).** Write `M1 to M6`, not a dashed range.

3. **No semicolon in prose.** Replace it with a full stop for two related
   sentences, or a comma for short coordinated clauses.

Rust and Python semicolons are not a style question. The scanner blanks code
before it looks.

## Where the rules apply

The gated set is `GATED_PREFIXES` and `GATED_FILES` in `scripts/prose_check.py`.
Everything else is advisory.

| Path | Gated |
|---|---|
| `docs/**/*.md`, so the whole HLD and sprint record | yes |
| `.claude/plans/**/*.md` | yes |
| `.claude/reviews/**/*.md` | yes |
| `.claude/commands/**/*.md` | yes |
| `.claude/skills/**/*.md`, including this file | yes |
| `CLAUDE.md`, `AGENTS.md`, `.claude/WORKFLOW.md` | yes |
| Commit messages, via `--commit-msg` | yes |
| `.agents/skills/**`, generated from the gated sources | no |
| Rust doc comments and code comments | no |
| Identifiers in code | no |
| Fenced blocks, indented blocks and inline spans inside gated Markdown | no |

## What the scanner already exempts

Do not work around these by hand. `strip_code` blanks them while preserving
line numbers, so a finding's line number is the real one:

- Fenced blocks opened by ``` or `~~~`.
- Indented blocks of a tab or four or more spaces, unless the indent is a list
  marker.
- Inline code spans.
- HTML entities such as `&amp;`, whose trailing semicolon is not punctuation.
- URLs, bare or angle-bracketed, whose semicolons are not punctuation.

## Fixing a violation

The report is `path:line:col` with the rule named. Open the line, replace the
character, and re-run:

```bash
python3 scripts/prose_check.py <path>
python3 scripts/prose_check.py --staged
```

An em-dash usually wants a comma or a full stop. A semicolon usually wants a
full stop. If neither reads well, the sentence was doing too much and wants to
be two sentences.

## Quoting an external source that uses the punctuation

Do not silently alter a quotation. Put the quoted text in a fenced block, which
is exempt, and keep the surrounding prose compliant.

## Related

- `.claude/WORKFLOW.md`, "Voice rules", which is where the rule is law.
- `.claude/commands/verify.md`, step 6, which runs the gate.

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

Exit codes: 0 ok, 1 refused, 2 usage.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
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

SPRINT_RE = re.compile(r"^# Current Sprint, (S\d+(?:\.\d+)?)$", re.MULTILINE)
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
    if not features:
        die("CURRENT_SPRINT.md has no parseable wave rows")
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
        if feature["owner"] != "-":
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
    print("  " + ", ".join(f"{k}={v}" for k, v in sorted(counts.items())))
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

    args = ap.parse_args()
    if getattr(args, "fid", None) and not FID_RE.match(args.fid):
        die(f"{args.fid} is not a valid F-ID")
    if hasattr(args, "sprint") and args.sprint and not SPRINT_ID_RE.match(args.sprint):
        die(f"{args.sprint} is not a valid sprint id, expected SNN")
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())

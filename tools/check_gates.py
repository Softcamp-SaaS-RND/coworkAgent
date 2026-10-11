#!/usr/bin/env python
"""평가 항목 gate 실행기 — "완료 주장" 과 "완료" 사이를 **명령으로** 잇는다.

`plan.md` 의 ``## 평가 항목`` 은 사용자 승인을 받은 **계약**이다. 이 도구는 그 계약을
읽어 각 항목의 ``CHECK:`` 를 실제로 실행하고, 결과를 `eval.md` 의 ``## 평가 원장`` 에
기록한다. **박스는 명령이 뒤집는다** — 자기 채점 경로를 없애는 것이 목적이다.

- ``plan.md`` 는 **읽기 전용**이다. 승인된 계약이므로 도구가 고치지 않는다.
- ``eval.md`` 의 원장은 **기계 산출**이다. 실행할 때마다 다시 쓰인다.
  단 CHECK 가 없는 **수동 gate 의 EVIDENCE** 와 ``ABANDON:`` 줄은 사람이 적고,
  재실행해도 보존된다.
- 원장에는 **측정 커밋 sha** 를 함께 적는다 (docs/etc/eval-protocol.md "순환 방지 규칙" 4 —
  코드를 바꾼 뒤 남은 옛 수치는 증거가 아니다). 재실행하면 HEAD 로 갱신된다.

**미충족(UNMET)** 판정 — 셋 중 하나면 미충족이다:

1. 박스가 비었고 ``ABANDON:`` 도 없다. **사유 없는** ``ABANDON:`` 은 포기가 아니라 미충족이다.
2. 박스가 체크됐는데 ``EVIDENCE:`` 가 여전히 ``pending`` 이다.
   체크박스는 주장이고 EVIDENCE 가 증거다 — 증거 없는 체크는 미체크보다 나쁘다.
3. CHECK 를 실행했는데 출력이 EXPECT 와 맞지 않는다 (박스를 다시 비운다).

🔴 **WEAK 표시** — 미충족은 아니지만 기계가 **내용을 단정하지 않는** gate 는 원장에
``WEAK: <id> <사유>`` 로 표시된다 (``EXPECT`` 없음 = 종료 코드만 봄 · ``CHECK`` 없음 =
수동 gate). 판정에는 영향이 없다 — **셀 수 있게** 하는 것이 목적이다. 약한 gate 는
"대상이 없어 아무 일도 안 해도 통과" 하는 헛통과의 주경로이며, 그것이 원장에서
안 보이는 것이 문제였다. 약한 gate 가 늘면 그 Task 가 기계 판정 가능한 형태로
명세되지 않은 것이다.

gate 형식(정본은 docs/etc/eval-protocol.md)::

    - [ ] E1: 8개 경로 전부 판정된다
      CHECK: python -m pytest tests/test_route.py -q
      EXPECT: /8 passed/
      EVIDENCE: pending

성공은 **종료 코드 0 이면서** EXPECT 가 출력에 있는 것이다 — 실패 출력이 성공 문자열을
함께 갖는 일이 흔하다(``1 failed, 23 passed``). CHECK 안의 ``{TASK}`` 는 그 Task 폴더의
리포 상대경로로 바뀐다 — 종결(archive 이동) 뒤에도 같은 CHECK 가 그대로 돈다.

``EXPECT`` 는 stdout+stderr 결합 출력에 대한 **부분 문자열** 매치이며, ``/.../`` 로 감싸면
정규식이다. ``EXPECT`` 를 생략하면 **종료 코드 0** 을 성공으로 본다.
``CHECK`` 가 없는 항목은 수동 gate 이며, EVIDENCE 를 사람이 적는다.

🔴 **신뢰 경계**: ``CHECK`` 는 ``shell=True`` 로 실행된다 — ``plan.md`` 는 **실행되는
파일**이고, 이 도구의 노출은 그 트리에서 ``pytest``(conftest) 나 ``make`` 를 돌리는 것과
같은 등급이다. **신뢰하지 않는 브랜치·PR 의 작업 트리에서는 돌리지 않는다.**
남의 diff 를 평가할 때는 ``--list`` 로 명령을 먼저 읽는다.

사용::

    python tools/check_gates.py                     # docs/tasks/ 전 Task
    python tools/check_gates.py docs/tasks/01.foo   # Task 하나
    python tools/check_gates.py --list              # 실행 없이 gate 만 출력
    python tools/check_gates.py --selftest          # 파서·판정 자체 점검

종료 코드: 미충족 0건이면 0, 아니면 1.

gate 형식과 "증거 없는 체크 = 미충족" 규칙은 unlazy(Leonxlnx, MIT)에서 가져와
DDD 문서 체인(plan=계약 / eval=원장)에 맞게 옮겼다.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
DOCS = REPO / "docs"

PLAN_SECTION = "## 평가 항목"
LEDGER_SECTION = "## 평가 원장"
TIMEOUT = 600  # 초 — 매달리지 않는다. 초과는 실패로 기록한다.
EVIDENCE_CAP = 200  # 자 — 근거는 결정적인 줄만. 로그를 붙이지 않는다.

GATE_RE = re.compile(r"^- \[( |x|X)\] (?P<id>[^:\s]+):\s*(?P<title>.*)$")
FIELD_RE = re.compile(r"^\s+(CHECK|EXPECT|EVIDENCE):\s?(.*)$")
#: 🔴 사유는 **비어도 파싱한다** — 형식이 틀린 줄을 버리면 사람이 적은 `ABANDON: E4` 가 다음 재렌더에서
#:    원장에서 조용히 사라진다. 빈 사유는 포기로 인정하지 않고 :func:`unmet` 이 미충족으로 드러낸다.
ABANDON_RE = re.compile(r"^ABANDON:\s*([^:\s]+):?\s*(.*)$")
HEADING_RE = re.compile(r"^## ")

LEDGER_NOTE = (
    "<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의\n"
    "     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.\n"
    "     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며\n"
    "     판정에는 영향이 없다. -->"
)


class Gate:
    """평가 항목 하나. ``ok`` 는 판정 결과(수동 gate 는 EVIDENCE 로 판정)."""

    __slots__ = ("id", "title", "checked", "check", "expect", "evidence")

    def __init__(self, gid: str, title: str, checked: bool) -> None:
        self.id = gid
        self.title = title
        self.checked = checked
        self.check: str | None = None
        self.expect: str | None = None
        self.evidence: str | None = None

    @property
    def pending(self) -> bool:
        return self.evidence is None or self.evidence.strip().lower() in ("", "pending")

    def render(self) -> str:
        box = "x" if self.checked else " "
        out = [f"- [{box}] {self.id}: {self.title}"]
        if self.check is not None:
            out.append(f"  CHECK: {self.check}")
        if self.expect is not None:
            out.append(f"  EXPECT: {self.expect}")
        out.append(f"  EVIDENCE: {self.evidence or 'pending'}")
        return "\n".join(out)


def visible_text(raw: str) -> str:
    """fenced code block · HTML comment 를 걷어낸 **본문**만 돌려준다.

    예시 코드블록이나 주석 안의 heading·gate·표는 문서의 계약이 아니다 — 계약을 지우고
    예시로 위장하는 우회를 막는다. 🔴 파서가 둘이면 한쪽만 막힌다 — ``check_ddd_docs`` 도
    이 함수와 :func:`section_body` · :func:`parse_gates` 를 **그대로** 가져다 쓴다.
    fence 는 delimiter 종류·길이별로 추적하고(CommonMark: 닫는 fence 는 같은 종류 +
    여는 길이 이상, 뒤에 공백만), 닫히지 않은 ``<!--`` 는 EOF 까지 주석으로 본다.
    """
    fence_re = re.compile(r"^\s*(`{3,}|~{3,})(.*)$")
    lines: list[str] = []
    open_fence: tuple[str, int] | None = None
    for line in raw.splitlines():
        m = fence_re.match(line)
        if m:
            marker, rest = m.group(1), m.group(2)
            if open_fence is None:
                open_fence = (marker[0], len(marker))  # 여는 fence 는 info string 허용
            elif marker[0] == open_fence[0] and len(marker) >= open_fence[1] and not rest.strip():
                open_fence = None
            continue  # fence 줄 자체(여닫음 · 내부의 fence 유사 행)는 본문이 아니다
        if open_fence is None:
            lines.append(line)

    text = re.sub(r"<!--.*?-->", "", "\n".join(lines), flags=re.DOTALL)
    unclosed = text.find("<!--")
    return text if unclosed == -1 else text[:unclosed]


def section_body(text: str, heading: str) -> str | None:
    """``## heading`` 본문(다음 ``## `` 전까지). 없으면 None.

    heading 뒤 괄호 부연을 허용한다 — `## 평가 항목 (개정 2026-01-01)`.
    fenced 예시·HTML 주석 안의 heading 은 **계약이 아니다** — 걷어내고 본다.
    """
    lines = visible_text(text).splitlines()
    start = None
    for i, line in enumerate(lines):
        if line.rstrip() == heading or line.startswith(heading + " ("):
            start = i + 1
            break
    if start is None:
        return None
    end = len(lines)
    for i in range(start, len(lines)):
        if HEADING_RE.match(lines[i]):
            end = i
            break
    return "\n".join(lines[start:end])


def parse_gates(body: str) -> tuple[list[Gate], dict[str, str]]:
    """gate 목록과 ``{gate id: ABANDON 사유}`` — 사유가 빈 줄도 담는다(판정은 :func:`unmet`)."""
    gates: list[Gate] = []
    abandoned: dict[str, str] = {}
    cur: Gate | None = None
    for line in body.splitlines():
        m = GATE_RE.match(line.rstrip())
        if m:
            cur = Gate(m.group("id"), m.group("title").strip(), m.group(1).lower() == "x")
            gates.append(cur)
            continue
        a = ABANDON_RE.match(line.strip())
        if a:
            abandoned[a.group(1)] = a.group(2).strip()
            cur = None
            continue
        f = FIELD_RE.match(line.rstrip())
        if f and cur is not None:
            setattr(cur, f.group(1).lower(), f.group(2).strip())
    return gates, abandoned


def matches(expect: str | None, output: str, returncode: int) -> bool:
    """성공 = **종료 코드 0** 이고 EXPECT 가 출력에 있다. EXPECT 생략 시 종료 코드만 본다.

    🔴 EXPECT 가 있다고 종료 코드를 무시하면 안 된다 — 실패 출력이 성공 문자열을 **함께**
    갖는 일이 흔하다. `1 failed, 23 passed` (rc=1) 가 `/passed/` gate 를 통과한다.
    깨진 정규식은 **불일치**로 본다(예외로 터뜨리면 gate 하나가 도구를 멈추고,
    참으로 보면 오타가 통과한다).
    """
    if returncode != 0:
        return False
    if expect is None or expect == "":
        return True
    if len(expect) > 2 and expect.startswith("/") and expect.endswith("/"):
        try:
            return re.search(expect[1:-1], output) is not None
        except re.error:
            return False
    return expect in output


def deciding_line(expect: str | None, output: str) -> str:
    """근거 = 판정을 가른 줄. 로그 전문을 붙이지 않는다."""
    lines = [ln.strip() for ln in output.splitlines() if ln.strip()]
    if expect and len(expect) > 2 and expect.startswith("/") and expect.endswith("/"):
        try:
            hit = [ln for ln in lines if re.search(expect[1:-1], ln)]
        except re.error:
            hit = []
    elif expect:
        hit = [ln for ln in lines if expect in ln]
    else:
        hit = []
    picked = hit[-1:] if hit else lines[-3:]
    return (" / ".join(picked) or "(출력 없음)")[:EVIDENCE_CAP]


def run_gate(gate: Gate, task_rel: str = "") -> bool:
    """CHECK 를 실행해 박스·EVIDENCE 를 갱신하고 충족 여부를 반환한다.

    🔴 CHECK 안의 ``{TASK}`` 는 **그 Task 폴더의 리포 상대경로**로 바뀐다. 종결하면
    폴더가 ``docs/tasks/`` → ``docs/archive/`` 로 옮겨지므로, 경로를 글자로 박은 CHECK 는
    **종결하는 순간 깨진다**. 치환은 **실행 직전에만** 하고 ``gate.check`` 원문은 그대로
    둔다 — 원장은 plan 과 글자 그대로 대조되기 때문이다.

    🔴 자식 프로세스에 ``CHECK_GATES_TASK`` 를 준다. 문서 규약 체커를 CHECK 로 부르면
    그 Task 의 원장은 아직 **이전 실행**의 결과다 — 그것을 근거로 실패시키면 원장이
    자기 자신 때문에 영원히 빨간 **고정점**에 빠진다.
    """
    if gate.check is None:  # 수동 gate — 사람이 적은 EVIDENCE 로 판정
        return gate.checked and not gate.pending
    command = gate.check.replace("{TASK}", task_rel) if task_rel else gate.check
    env = {**os.environ, "CHECK_GATES_TASK": task_rel} if task_rel else None
    try:
        proc = subprocess.run(
            command, shell=True, cwd=REPO, capture_output=True, env=env,
            text=True, encoding="utf-8", errors="replace", timeout=TIMEOUT,
        )
    except subprocess.TimeoutExpired:
        gate.checked = False
        gate.evidence = f"FAIL: CHECK 가 {TIMEOUT}s 를 넘겼다"
        return False
    output = (proc.stdout or "") + (proc.stderr or "")
    ok = matches(gate.expect, output, proc.returncode)
    gate.checked = ok
    prefix = "" if ok else f"FAIL(rc={proc.returncode}) "
    gate.evidence = prefix + deciding_line(gate.expect, output)
    return ok


def weak_reason(gate: Gate) -> str | None:
    """기계가 **내용을 단정하지 않는** gate 의 사유. 아니면 ``None``.

    🔴 약한 gate 는 FAIL 이 아니다 — 정당한 쓰임이 있다(관측이 정말 수동인 항목).
    문제는 약하다는 사실이 **원장에서 안 보인다**는 것이었다: ``EXPECT`` 가 없으면
    :func:`matches` 가 종료 코드만 보고 통과시키고, 원장에는 EXPECT 줄이 빠져 있을 뿐
    *약한 gate 라는 표시*가 없다. 대상이 사라져 아무 일도 안 하고 rc 0 으로 끝나는
    명령이 정확히 이 자리로 들어와 **몇 달간 초록**일 수 있다(실측 사례).
    표시만 해도 리뷰어가 **셀 수 있게** 된다.

    ABANDON 된 gate 는 제외한다 — 이미 불가능으로 판정돼 사유가 따로 적혀 있다.
    """
    if gate.check is None:
        return "CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)"
    if gate.expect is None or gate.expect.strip() == "":
        return "EXPECT 없음 — 종료 코드만 본다(대상이 없어 아무 일도 안 해도 통과)"
    return None


def render_ledger(gates: list[Gate], abandoned: dict[str, str], sha: str) -> str:
    """원장 절 전문.

    🔴 ``ABANDON``·``WEAK`` 는 gate 항목이 **아니라 독립 줄**로 나간다. 원장은
    ``check_ddd_docs._ledger_vs_plan`` 이 plan 과 **id·제목·CHECK·EXPECT·순서까지 글자
    그대로** 대조하므로, 표시를 gate 제목에 붙이면 그 즉시 FAIL 한다.
    """
    parts = [LEDGER_SECTION, "", LEDGER_NOTE, "", f"측정 커밋: {sha}", ""]
    parts += [g.render() + "\n" for g in gates]
    parts += [f"ABANDON: {gid} {reason}".rstrip() for gid, reason in sorted(abandoned.items())]
    weak = [f"WEAK: {g.id} {reason}" for g in gates
            if not abandoned.get(g.id) and (reason := weak_reason(g)) is not None]
    if weak:
        parts += [""] + weak
    return "\n".join(parts).rstrip() + "\n"


def replace_section(text: str, heading: str, block: str) -> str:
    """``## heading`` 절을 통째로 치환한다. 없으면 문서 끝에 덧붙인다."""
    lines = text.splitlines()
    start = None
    for i, line in enumerate(lines):
        if line.rstrip() == heading or line.startswith(heading + " ("):
            start = i
            break
    if start is None:
        sep = "" if text.endswith("\n\n") else ("\n" if text.endswith("\n") else "\n\n")
        return text + sep + block
    end = len(lines)
    for i in range(start + 1, len(lines)):
        if HEADING_RE.match(lines[i]):
            end = i
            break
    return "\n".join(lines[:start] + block.rstrip().splitlines() + [""] + lines[end:]) + "\n"


def head_sha() -> str:
    try:
        proc = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=REPO,
            capture_output=True, text=True, timeout=30,
        )
        # 🔴 종료 코드를 함께 본다 — 커밋이 하나도 없는 리포에서 `git rev-parse HEAD` 는
        #    rc 128 로 실패하면서 stdout 에 `HEAD` 를 그대로 뱉는다. 그것을 sha 로 적으면
        #    원장이 **측정 커밋을 가진 것처럼** 보인다.
        if proc.returncode != 0:
            return "(커밋 없음 — 첫 커밋 뒤 gate 를 다시 돌린다)"
        return (proc.stdout or "").strip() or "(git repo 아님)"
    except (OSError, subprocess.SubprocessError):
        return "(git 실행 불가)"


def unmet(gates: list[Gate], abandoned: dict[str, str]) -> list[str]:
    """미충족 gate 사유 목록 — 판정 정본(check_ddd_docs 는 :func:`ledger_unmet` 으로 쓴다)."""
    found: list[str] = []
    for g in gates:
        if g.id in abandoned:
            if not abandoned[g.id]:
                found.append(f"{g.id}: ABANDON 에 사유가 없다 — 'ABANDON: {g.id} <사유>' 로 적을 것"
                             "(사유 없는 포기는 조용한 범위 축소다)")
            continue
        if not g.checked:
            found.append(f"{g.id}: 미충족 — {g.evidence or g.title}")
        elif g.pending:
            found.append(f"{g.id}: 체크됐는데 EVIDENCE 가 pending — 증거 없는 체크는 미충족이다")
    return found


def ledger_unmet(eval_text: str) -> list[str]:
    """eval.md 원장의 미충족 목록. 원장 절이 없으면 그 사실을 반환한다."""
    body = section_body(eval_text, LEDGER_SECTION)
    if body is None:
        return [f"'{LEDGER_SECTION}' 절 없음 — tools/check_gates.py 를 실행할 것"]
    gates, abandoned = parse_gates(body)
    if not gates:
        return [f"'{LEDGER_SECTION}' 에 gate 가 0건 — plan 의 평가 항목이 비었는지 확인할 것"]
    return unmet(gates, abandoned)


def process(task_dir: Path, list_only: bool) -> tuple[int, list[str]]:
    """Task 하나 처리. (미충족 건수, 보고 줄)."""
    report: list[str] = []
    rel = task_dir.relative_to(REPO).as_posix()
    plan = task_dir / "plan.md"
    if not plan.is_file():
        return 1, [f"{rel}/: plan.md 없음 — 평가 항목이 계약으로 고정되지 않았다"]
    body = section_body(plan.read_text(encoding="utf-8"), PLAN_SECTION)
    if body is None:
        return 1, [f"{rel}/plan.md: '{PLAN_SECTION}' 절 없음"]
    gates, _ = parse_gates(body)
    if not gates:
        return 1, [f"{rel}/plan.md: '{PLAN_SECTION}' 에 gate 형식 항목이 0건 "
                   f"— '- [ ] E1: ...' 형식으로 적을 것"]

    evaluation = task_dir / "eval.md"
    prior_gates: dict[str, Gate] = {}
    abandoned: dict[str, str] = {}
    if evaluation.is_file():
        prior_body = section_body(evaluation.read_text(encoding="utf-8"), LEDGER_SECTION)
        if prior_body is not None:
            prior_list, abandoned = parse_gates(prior_body)
            prior_gates = {g.id: g for g in prior_list}

    if list_only:
        report.append(f"{rel}/plan.md — gate {len(gates)}건")
        report += [f"  {g.id}: {'CHECK ' + g.check if g.check else '수동 gate'}" for g in gates]
        return 0, report

    for gate in gates:
        if gate.check is None:  # 수동 gate — 사람이 적은 판정을 원장에서 이어받는다
            prior = prior_gates.get(gate.id)
            if prior is not None:
                gate.checked, gate.evidence = prior.checked, prior.evidence
        ok = run_gate(gate, rel)
        mark = "✅" if ok else ("(ABANDON)" if abandoned.get(gate.id) else "❌")
        report.append(f"  {gate.id:<6} {mark} {gate.evidence}")

    # 🔴 **실행 중에 사람이 적은 수동 필드를 잃지 않는다**.
    #    CHECK 는 건당 최대 600초다. 그 사이 사람이 수동 gate 의 EVIDENCE 나 ABANDON 을
    #    적을 수 있는데, 실행 **전**에 파싱한 값으로 원장을 통째로 덮으면 그 손이 지워진다
    #    — 파일 상단이 약속한 "수동 필드는 재실행해도 보존된다" 를 어긴다.
    #    그래서 **쓰기 직전에 다시 읽어** 수동 gate 와 ABANDON 을 최신본에서 이어받는다.
    text = evaluation.read_text(encoding="utf-8") if evaluation.is_file() else \
        f"# eval.md — {task_dir.name} 적대적 평가\n"
    latest = section_body(text, LEDGER_SECTION)
    if latest is not None:
        latest_gates, latest_abandoned = parse_gates(latest)
        abandoned = latest_abandoned
        latest_map = {g.id: g for g in latest_gates}
        for gate in gates:
            if gate.check is None and (latest_gate := latest_map.get(gate.id)) is not None:
                gate.checked, gate.evidence = latest_gate.checked, latest_gate.evidence
    evaluation.write_text(
        replace_section(text, LEDGER_SECTION, render_ledger(gates, abandoned, head_sha())),
        encoding="utf-8", newline="\n",
    )
    problems = unmet(gates, abandoned)
    report.insert(0, f"{rel} — gate {len(gates)}건, 미충족 {len(problems)}건 "
                     f"→ {evaluation.relative_to(REPO).as_posix()}")
    return len(problems), report


def task_dirs(argv: list[str]) -> list[Path]:
    targets = [a for a in argv if not a.startswith("--")]
    if targets:
        return [Path(t).resolve() for t in targets]
    tasks = DOCS / "tasks"
    return sorted(d for d in tasks.iterdir() if d.is_dir()) if tasks.is_dir() else []


def selftest() -> int:
    body = (
        "- [ ] E1: 경로 8개 판정\n"
        "  CHECK: echo 8 passed\n"
        "  EXPECT: /8 passed/\n"
        "  EVIDENCE: pending\n"
        "\n- [x] E2: 사람이 확인한다\n"
        "  EVIDENCE: docs/tech/note.md:12\n"
        "\n- [x] E3: 증거 없는 체크\n"
        "  EVIDENCE: pending\n"
        "\nABANDON: E4 외부 API 미개통\n"
        "- [ ] E4: 외부 연동\n"
        "  EVIDENCE: pending\n"
    )
    gates, abandoned = parse_gates(body)
    assert [g.id for g in gates] == ["E1", "E2", "E3", "E4"], [g.id for g in gates]
    assert gates[0].expect == "/8 passed/" and gates[0].check == "echo 8 passed"
    assert abandoned == {"E4": "외부 API 미개통"}, abandoned
    assert matches("/8 passed/", "... 8 passed in 0.1s", 0) is True
    assert matches("/8 passed/", "... 1 failed, 8 passed", 1) is False  # rc 를 함께 본다
    assert matches("8 passed", "7 passed", 0) is False
    assert matches(None, "", 0) is True and matches(None, "", 1) is False
    sample = ("## 평가 항목" + chr(10) + chr(10) + "```text" + chr(10)
            + "- [ ] E9: 예시일 뿐이다" + chr(10) + "```" + chr(10) + chr(10)
            + "- [ ] E1: 진짜 항목" + chr(10) + "  EVIDENCE: pending" + chr(10))
    visible = parse_gates(section_body(sample, PLAN_SECTION) or "")[0]
    assert [g.id for g in visible] == ["E1"], [g.id for g in visible]  # fence 안의 E9 는 계약이 아니다
    assert deciding_line("/8 passed/", "junk\n8 passed in 0.1s\ntail") == "8 passed in 0.1s"
    problems = unmet(gates, abandoned)
    # E1 미체크 / E3 증거 없는 체크 → 2건. E2 는 충족, E4 는 ABANDON 으로 해소.
    assert len(problems) == 2 and problems[0].startswith("E1") and problems[1].startswith("E3"), problems
    assert run_gate(gates[0]) is True and gates[0].checked and not gates[0].pending
    assert section_body("## 평가 항목 (2026-01-01)\nx\n## 다음\ny", PLAN_SECTION) == "x"
    replaced = replace_section("# t\n\n## 평가 원장\nold\n\n## 뒤\nz\n", LEDGER_SECTION, "## 평가 원장\nnew\n")
    assert "old" not in replaced and "new" in replaced and "## 뒤" in replaced, replaced
    # WEAK — CHECK+EXPECT 를 가진 E1 은 강하고, CHECK 가 없는 E2·E3 은 약하다.
    # ABANDON 된 E4 는 이미 불가능으로 판정돼 표시하지 않는다.
    assert weak_reason(gates[0]) is None, weak_reason(gates[0])
    ledger = render_ledger(gates, abandoned, "0" * 40)
    assert "WEAK: E2 CHECK 없음" in ledger and "WEAK: E3 CHECK 없음" in ledger, ledger
    assert "WEAK: E1" not in ledger and "WEAK: E4" not in ledger, ledger
    # 🔴 WEAK 줄이 gate·ABANDON 파서에 닿으면 안 된다 — 원장은 plan 과 글자 그대로
    #    대조되므로 새 줄이 항목 대조에 섞이는 순간 문서 규약 체커가 FAIL 한다.
    reparsed, reparsed_abandoned = parse_gates(ledger)
    assert [g.id for g in reparsed] == ["E1", "E2", "E3", "E4"], [g.id for g in reparsed]
    assert reparsed_abandoned == {"E4": "외부 API 미개통"}, reparsed_abandoned
    # 🔴 사유 없는 ABANDON 은 포기가 아니다 — 미충족으로 드러내고, 재렌더에서 그 줄을 잃지도 않는다
    #    (조용히 버리면 사람이 적은 줄이 다음 실행에서 원장에서 사라진다).
    bare, bare_abandoned = parse_gates("- [ ] E6: 사유 없는 포기\n  EVIDENCE: pending\nABANDON: E6\n")
    assert bare_abandoned == {"E6": ""}, bare_abandoned
    bare_problems = unmet(bare, bare_abandoned)
    assert len(bare_problems) == 1 and bare_problems[0].startswith("E6") and "사유" in bare_problems[0], bare_problems
    bare_ledger = render_ledger(bare, bare_abandoned, "0" * 40)
    assert "\nABANDON: E6\n" in bare_ledger and "WEAK: E6" in bare_ledger, bare_ledger
    weak_gate = Gate("E5", "EXPECT 없는 gate", False)
    weak_gate.check = "true"
    assert weak_reason(weak_gate) is not None and "EXPECT 없음" in weak_reason(weak_gate)
    print("PASS — check_gates 자체 점검 통과")
    return 0


def main(argv: list[str]) -> int:
    if "--selftest" in argv:
        return selftest()
    list_only = "--list" in argv
    dirs = task_dirs(argv)
    if not dirs:
        print("대상 Task 없음 — docs/tasks/ 가 비었다")
        return 0
    total = 0
    for task_dir in dirs:
        count, report = process(task_dir, list_only)
        total += count
        for line in report:
            print(line)
    if list_only:
        return 0
    if total:
        print(f"\nFAIL — 미충족 gate {total}건. 남은 항목을 처리하거나, 정말 불가능하면 "
              f"eval.md 원장에 'ABANDON: <id> <사유>' 를 적고 보고할 것")
        return 1
    print("\nPASS — 평가 항목 전건 충족(근거 기록됨)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

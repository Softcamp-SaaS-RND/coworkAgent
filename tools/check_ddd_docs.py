"""DDD 문서 규약 conformance 체커 (docs/etc/ddd-workflow.md "문서 체계" 절).

빌드·테스트 green 과 **별개로** 문서 규약을 기계적으로 검증한다. 매 커밋 전과 문서
이동·이름 변경 후에 돌린다. 실행형은 루트 PROJECT.md "검증 명령" 표.

    python tools/check_ddd_docs.py

검사 항목:

1. 루트 ``CLAUDE.md`` 에 TODO 주석이 없다(프로젝트 무관 뼈대). PROJECT.md·PRODUCT.md·STATE.md·
   masterPlan·manual/README 의 TODO 는 **위반이 아니라 INFO 로 집계**한다(부트스트랩 미해소 항목).
2. 템플릿이 요구하는 ``docs/`` 하위 디렉토리 8개가 존재한다.
3. ``docs/`` **와 루트 5문서**(``CLAUDE.md``·``PROJECT.md``·``PRODUCT.md``·``STATE.md``·
   ``README.md``)의
   모든 **상대 마크다운 링크**가 실제 파일로 해소된다. 🔴 ``[[wikilink]]`` 는 백틱·울타리
   안까지 위반으로 잡는다. 같은 Task 폴더 형제는 폴더 밖으로 나가지 않는다(``git mv`` 로
   archive 에 옮길 때 끊긴다). ``docs/`` 안에서 **현존 문서**를 백틱 경로로 쓰지 않는다.
4. **intent.md 계약** — Task 폴더(활성·archive, :data:`GATE_LEGACY_KEYS` 밖)에 ``intent.md`` 가
   있고, 필수 6 섹션(:data:`INTENT_MD_SECTIONS`) heading + **본문**, ``승인:`` 줄이 **정확히 1건**,
   ``plan.md`` 가 있으면 ``승인: 대기`` 가 아니다(의도가 확정되기 전에 방법을 쓴 순서 역전을 막는다).
   ``plan.md`` 가 있으면 ``## 성공 기준`` 의 ``- A{n}:`` 을 **plan gate 제목 첫머리가 전부 덮는다**
   (intent-protocol 규약 4) — 덮이지 않은 기준은 *조용한 축소*이고, 구현이 끝난 뒤
   적대적 평가자가 물어야 드러난다.
   종결(``report.md``)한 Task 는 ``docs/masterPlan.md`` ``## 개정 기록`` 에 **자기 번호로 한 줄**을
   남긴다(intent-protocol 규약 5) — 환류하지 않으면 프로젝트 의도가 Task 폴더에 갇힌다.
   ``docs/masterPlan.md`` 도 ``승인:`` 줄이 **정확히 1건**이고, Task 폴더가 있으면 ``승인: 대기`` 가
   아니다(프로젝트 수준 의도가 확정되기 전에 Task 를 열지 않는다).
5. ``report.md`` 가 있는 Task 는 ``docs/archive/`` 에 있고, ``docs/archive/`` 의 모든 Task 에
   ``report.md`` 가 있다(종결 = archive 이동).
6. **PROJECT.md 계약** — 필수 섹션(:data:`PROJECT_MD_SECTIONS`) heading + 본문,
   ``## 치명 가드레일`` 이 첫 heading, 가드레일 **줄 수·자리별 키워드**(빈 키워드 금지).
   **PRODUCT.md 계약** — 필수 섹션(:data:`PRODUCT_MD_SECTIONS`) heading + 본문. 일하는 방식
   (PROJECT) · 만든 것(PRODUCT) · 지금 어디인가(STATE)를 한 문서에 섞지 않는다.
7. **Task 목록은 폴더가 SSOT** — 목록 표를 문서에 두지 않는다. 상태는 위치(``docs/tasks/`` =
   진행, ``docs/archive/`` = 종결)와 ``report.md`` 의 ``⛔ 폐기`` 마커로 **이미 결정**되므로,
   표는 정보를 더하지 않고 동기화 부담만 만든다. 목록이 필요하면 ``--tasks`` 로 파생한다.
8. **STATE.md 계약** — 필수 섹션 2개(:data:`STATE_MD_SECTIONS`) heading + **본문**. STATE.md 는
   **전부 덮어쓰는** 재개 좌표다 — 누적 기록(목록·원장)을 여기 두지 않는다.
   heading 만 남고 비면 "상태를 적었다" 는 침묵이 된다 — 없으면 ``없음`` 이라고 적는다.
9. **적대적 평가자 규약** — 구현이 시작된 Task 에 ``plan.md`` 존재 + ``## 평가 항목`` 절,
   ``report.md`` 가 있으면 ``eval.md`` 도 있고 ``plan.md`` 에 ``사용자 미승인`` 개정이 남아
   있지 않다. :data:`GATE_LEGACY_KEYS` 의 Task 는 면제(규약 도입 전에 종결된 Task 에 평가
   기록을 사후 작성하게 만들지 않는다).
10. **평가 항목 gate 규약** — :data:`GATE_LEGACY_KEYS` 에 없는 Task 는
    ① ``plan.md`` 의 ``## 평가 항목`` 이 **gate 형식**(``- [ ] E1:`` + CHECK/EXPECT/EVIDENCE),
    ② ``eval.md`` 가 있으면 ``## 평가 원장`` 절이 있고, **진행 중이거나 종결하는 중이면
    측정이 신선**하다 (그 sha 이후 ``docs/`` 밖 변경 0 — 커밋·미커밋·untracked 전부),
    ③ 종결(``report.md``)이면 **미충족 0건** + 원장이 plan 과 **id·제목·CHECK·EXPECT·순서**
    까지 같다(원장만 줄이거나 다른 계약으로 갈아끼우는 축소를 막는다).
    🔴 **종결 Task 의 신선도는 다시 재지 않는다** — 종결 원장은 *그 시점의 측정 기록*이고,
    뒤에 온 커밋이 그것을 거짓으로 만들지 않는다(그러지 않으면 archive 전체가 영원히 빨갛다).
    단 **종결하는 그 커밋에서는 잰다**(``report.md`` 미커밋 = 지금 종결 중) — 그것이
    stale 측정을 실어 나르는 유일한 경로였다.
    🔴 **판정 중인 Task 는 자기 원장을 읽지 않는다**(``CHECK_GATES_TASK``) — 이 검사를 gate
    CHECK 로 부르면 원장은 아직 *이전 실행*의 결과라, 자기 실패를 읽고 다시 실패하는
    **고정점**이 된다.
11. **Task 번호는 주소다** — ``{NN}.{기능명}`` 이 아닌 폴더는 위반이고(이름으로 규약을
    빠져나가는 길을 막는다), 같은 번호가 둘이면 위반이다(``24``·``24-2``·``0a`` 는 서로 다른
    주소 — 숫자 뒤 소문자 한 글자는 버퍼 오버플로 확장). ``#00``·``#0``·``00.`` 은 같은 번호다.
12. **미조치 원장 계약** — :data:`LEDGER_DOC` ``## 미조치 원장`` 의 5열(출처·요지·판단·상태·
    마지막 확인) 빈 칸 없음 · 상태 어휘 5값(:data:`LEDGER_STATES`) · archive Task 의 ``eval.md`` 에
    ``## 미조치 지적`` 절이 **있고**, ``없음`` 또는 표(지적·판단·사유)만 담고, 표에 행이 있으면 그 Task 가
    원장 출처(``#NN``)에 **있다**. 미조치가 각 ``eval.md`` 에 흩어지면 *다음 Task 가 조용히
    갚은 것*이 생긴다.
13. **fence·HTML 주석 안의 heading 은 계약이 아니다** — 실제 절을 지우고 예시 블록으로
    위장하는 우회를 모든 절 검사에서 함께 막는다. 절·gate 파서는 ``check_gates`` 의 것을
    **그대로** 쓴다(파서가 둘이면 한쪽만 막힌다).

종료 코드: 위반 0건이면 0, 아니면 1.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path
from urllib.parse import unquote

sys.path.insert(0, str(Path(__file__).resolve().parent))   # 형제 check_gates — `-m`·importlib 로 불러도 찾는다
from check_gates import LEDGER_SECTION, PLAN_SECTION, ledger_unmet, parse_gates, section_body, visible_text  # noqa: E402

REPO = Path(__file__).resolve().parent.parent
DOCS = REPO / "docs"

TODO_RE = re.compile(r"<!--\s*TODO")
#: 원장이 적는 측정 커밋 — 40자 full sha 하나.
MEASURED_SHA_RE = re.compile(r"^측정 커밋: ([0-9a-f]{40})$", re.MULTILINE)
#: 🔴 gate 규약에서 **면제되는 legacy Task key**. 새 프로젝트는 비어 있다.
#: 규약을 **도중에** 도입하는 리포만, 그 시점에 이미 있던 key 의 **정확한 집합**을
#: 여기 적고 동결한다 — 번호 컷오프(`>= 90`)로 자르면 새 Task 를 `89.newTask` 로
#: 만들어 영구 우회할 수 있다(실측). 여기 없는 key 는 **전부 대상**이다.
#: 🔴 **활성 Task 는 넣지 않는다** — 면제하면 gate·eval·신선도 검사가 전부 꺼진다(실측).
#:    종결·폐기가 끝난 key 만.
GATE_LEGACY_KEYS: frozenset[str] = frozenset()
#: **미착수 폐기** 선언 마커 — `report.md` 에 이 문자열이 있고 `work/stage*.md` 가
#: 0건이면 평가할 구현이 없으므로 plan·eval·gate 를 요구하지 않는다. 선언만으로
#: 열리면 '폐기라고 써 두고 구현' 이 규약을 통째로 우회한다 — **구현 흔적 0** 을 함께 본다.
DISCARD_MARKER = "⛔ 폐기"
# 미조치 원장 — **상태 어휘 고정**. 어휘가 열리면 `보류`·`검토중` 이 늘어 상태를 셀 수
# 없게 되고, 셀 수 없는 원장은 아무도 안 본다.
# 🔴 `닫힘`(해결됨)과 `수용`(위험을 알고 고치지 않기로 결정)을 섞지 않는다 —
#    섞으면 원장이 거짓말을 하고 후속 추적 대상이 사라진다.
#: 미조치 원장의 집 — **STATE.md 가 아니다.** 원장은 Task 를 가로질러 누적되는 유일한 기록이라
#: 덮어쓰는 재개 좌표와 같은 문서에 살 수 없다(상태 전이·마지막 확인은 폴더에서 파생되지 않는다).
LEDGER_DOC = "docs/overview/ledger.md"
LEDGER_HEADING = "## 미조치 원장"
LEDGER_STATES = ("열림", "닫힘", "반박", "조건 대기", "수용")
LEDGER_COLUMNS = 5          # 출처 · 요지 · 판단 · 상태 · 마지막 확인
#: eval.md 의 남은 지적 절 — `없음` 또는 표(지적 · 판단 · 사유)만 허용한다. 산문·bullet 은
#: "남은 지적 없음" 과 "남은 지적: …" 을 기계가 가르지 못한다(키워드 휴리스틱의 실측 오탐).
LEFTOVER_HEADING = "## 미조치 지적"
LEFTOVER_HEAD = "지적"
#: Task 번호 — 숫자 + 소문자 한 글자(`0a`, 버퍼 오버플로) + `-n`(하위 트랙). 형식 정본은
#: docs/etc/doc-conventions.md 이며 폴더명·원장 출처·`#NN` 인용이 **이 한 조각**을 함께 쓴다.
_NUM = r"\d+[a-z]?(?:-\d+)?"
_FOLDER_RE = re.compile(rf"^({_NUM})\.")
_TASK_REF_RE = re.compile(rf"#{_NUM}")
#: Task 상태는 **폴더 위치 + 폐기 마커**에서 파생한다 — 목록 표를 어디에도 두지 않는다(`--tasks`).
TASK_STATES = {"tasks": "⏳", "archive": "✅", "discard": "⛔"}
#: plan 개정 줄의 미승인 표기 — 종결 Task 에 남아 있으면 FAIL (eval-protocol 규약 3).
UNAPPROVED_MARK = "사용자 미승인"

# docs/etc/ddd-workflow.md "루트 3문서 계약" 의 PROJECT.md 표와 **같은 값**(바꾸려면 표와 동시 개정).
# 🔴 **일하는 방식**만 여기 — 제품 구조는 PRODUCT.md, 진행 현황·미조치는 STATE.md 가 갖는다.
PROJECT_MD_SECTIONS = (
    "치명 가드레일",
    "프로젝트 개요",
    "개발 환경",
    "검증 명령",
    "참조 문서",
)
# docs/etc/ddd-workflow.md "루트 3문서 계약" 의 PRODUCT.md 표와 **같은 값**.
# 제품 문서에 프로세스(도구 전제·브랜치·Task 현황)를 섞지 않는다.
PRODUCT_MD_SECTIONS = (
    "제품 개요",
    "기술 스택",
    "모듈 Layer 구조",
    "도메인 규칙",
    "개발된 모듈 / 매뉴얼",
    "설계 문서",
)
# docs/etc/intent-protocol.md 골격의 필수 섹션 — 골격·규약 3 의 표와 **같은 값**이어야 한다.
# 목록을 바꾸려면 intent-protocol.md 골격·표와 본 상수를 동시 개정한다.
INTENT_MD_SECTIONS = (
    "왜",
    "범위",
    "제외",
    "성공 기준",
    "결정",
    "미해소",
)
#: intent.md 의 승인 줄(열 0) — `승인: 대기` → 사용자 확인 후 `승인: {YYYY-MM-DD}`.
INTENT_APPROVAL_RE = re.compile(r"^승인:[ \t]*(.*)$", re.MULTILINE)
INTENT_PENDING = "대기"
#: intent 의 성공 기준 절과 그 번호 — `- A1: …` 줄. plan 의 gate 는 제목 **첫머리**에 같은 번호를
#: 달아 대응을 드러낸다(`E3: A2 회귀 없음` · `E4: A1·A3 …` — intent-protocol 규약 4).
#: 🔴 번호는 **자리**로 읽는다 — 아무 데서나 `A\d+` 를 찾으면 산문의 "A4 용지" 가 기준이 되고,
#:    제목 속 "(A1 과 무관)" 이 대응 표시가 된다. 형식을 바꾸면 규약 4·골격을 함께 고친다.
INTENT_SUCCESS_SECTION = "## 성공 기준"
CRITERION_RE = re.compile(r"^\s*[-*]\s+A(\d+)\s*:", re.MULTILINE)
GATE_CRITERIA_RE = re.compile(r"^A\d+(?:[\s·,]+A\d+)*(?![0-9A-Za-z])")
#: 기준이 없는 Task — 본문이 이 낱말로 시작하면 번호를 요구하지 않는다(규약 3 의 `없음`).
NO_CRITERIA = ("없음", "미확정")
#: 종결 환류처 — masterPlan 의 개정 기록 절(intent-protocol 규약 5). 한 줄에 Task 번호(`#N`)를 단다.
MASTERPLAN = "docs/masterPlan.md"
REVISION_SECTION = "## 개정 기록"
# docs/etc/ddd-workflow.md "STATE.md — 필수 2절" bullet 목록과 **같은 값**이어야 한다.
# 목록을 바꾸려면 그 절과 본 상수를 동시 개정한다. 두 절 모두 **덮어쓴다** — 누적은 여기 없다.
STATE_MD_SECTIONS = (
    "현재 작업",
    "다음 행동",
)

# 치명 가드레일 계약 — **정확값**. ddd-workflow.md 계약 표 1행("3~7줄")을 프로젝트 확정 개수로 못박는다.
# 개수만 세면 5줄을 아무 내용으로나 바꿔치기할 수 있으므로 **각 줄의 키워드**까지 동결한다.
# **부트스트랩 시 이 프로젝트의 가드레일로 작성한다** — n행 = GUARDRAIL_KEYWORDS[n-1].
# 비어 있으면 FAIL 한다(자리별 검사가 조용히 사라지는 것을 막는다).
GUARDRAIL_BULLETS = 6
GUARDRAIL_KEYWORDS: tuple[str, ...] = ("단일 실행 파일", "macOS", "서버·클라이언트", "웹UI", "SQLite", "신뢰하지 않는 입력")


REQUIRED_DIRS = (
    "tasks",
    "archive",
    "overview",
    "manual",
    "tech",
    "troubleshootings",
    "feedback",
    "etc",
)

MDLINK_RE = re.compile(r"\[[^\]\n]*\]\(([^)\s]+)\)")
"""상대 마크다운 링크 ``[텍스트](경로)`` — 문서 간 참조의 **유일한** 형식이다."""

WIKILINK_RE = re.compile(r"\[\[[^\]\n]+\]\]")
"""폐지된 ``[[wikilink]]``. 🔴 **백틱·울타리 안까지** 잡는다.

백틱 안을 먼저 지우면 리치 마크다운 에디터가 링크를 내부 토큰으로 치환했을 때
(``[[ORCA_RICH_MD:...]]``) 죽은 좌표가 검사를 통과한다 — 실측 2건.
파이썬 타입 표기 ``Callable[[str], str]`` 는 ``[[`` 와 ``]]`` 사이에 ``]`` 가 있어
이 패턴에 걸리지 않는다.
"""

LINE_CODE_SPAN_RE = re.compile(r"`[^`\n]*`")
"""한 줄 안에서 닫히는 백틱만 지운다 — 줄 번호를 어긋나게 하지 않는다."""

FENCE_RE = re.compile(r"^```.*?^```", re.M | re.S)
"""울타리 친 코드 블록. **인용한 원문은 규약 대상이 아니다** — 리뷰 원문·명령·나무 그림."""

BACKTICK_DOC_RE = re.compile(r"`(docs/[^`]*\.md)`")
_EXTERNAL_RE = re.compile(r"^(?:https?:|mailto:|#)")


def _task_folder(md: Path) -> Path | None:
    """이 문서가 속한 Task 폴더(``docs/{tasks,archive}/{key}/``). 아니면 ``None``."""
    try:
        parts = md.relative_to(DOCS).parts
    except ValueError:
        return None
    if len(parts) >= 3 and parts[0] in ("tasks", "archive"):
        return DOCS / parts[0] / parts[1]
    return None


def _link_problems(md: Path, body: str) -> list[str]:
    """상대 마크다운 링크 규약 — 해소 · 트리 이탈 · 형제 참조 형태.

    형제 참조가 Task 폴더 밖으로 나가면 안 되는 이유: Task 폴더는 종결 때 ``git mv`` 로
    ``tasks/`` → ``archive/`` 로 통째로 옮겨진다. 폴더 안에서 닫힌 링크는 그대로
    살아 있고, 밖으로 나갔다 돌아오는 링크는 그때 끊긴다.
    """
    rel = md.relative_to(REPO).as_posix()
    home = _task_folder(md)
    depth = len(md.parent.relative_to(home).parts) if home is not None else 0
    problems: list[str] = []
    # 🔴 링크 해소는 울타리·백틱 **밖만** 본다 — 인용 원문과 정규식 조각은 링크가 아니다.
    #    반면 폐지된 표기는 **안까지** 잡는다(에디터의 토큰 치환을 놓치지 않으려고).
    unfenced = FENCE_RE.sub(lambda m: "\n" * m.group(0).count("\n"), body)
    plain_lines = LINE_CODE_SPAN_RE.sub("", unfenced).split("\n")
    for line_no, line in enumerate(body.split("\n"), 1):
        for dead in WIKILINK_RE.findall(line):
            problems.append(
                f"{rel}:{line_no} 폐지된 wikilink {dead} — [텍스트](상대경로.md) 로 쓸 것"
            )
        plain = plain_lines[line_no - 1] if line_no <= len(plain_lines) else ""
        for href in MDLINK_RE.findall(plain):
            if _EXTERNAL_RE.match(href):
                continue
            target = unquote(href.split("#", 1)[0])
            if not target:
                continue
            dest = (md.parent / target).resolve()
            if not dest.exists():
                problems.append(f"{rel}:{line_no} dead link -> {href}")
                continue
            try:
                dest.relative_to(REPO)
            except ValueError:
                problems.append(f"{rel}:{line_no} 리포 밖으로 나가는 링크 -> {href}")
                continue
            if md.is_relative_to(DOCS) and not dest.is_relative_to(DOCS):
                problems.append(
                    f"{rel}:{line_no} docs/ 밖을 링크로 가리켰다 -> {href} "
                    "(코드 파일·루트 문서는 백틱 경로로 쓴다)"
                )
                continue
            if home is not None and dest.is_relative_to(home) and target.count("../") > depth:
                problems.append(
                    f"{rel}:{line_no} 같은 Task 형제는 폴더 안에서 닫는다 -> {href} "
                    "(archive 이동 때 끊긴다)"
                )
    return problems


_KEY_RE = re.compile(r"^#?0*(\d+)([a-z]?)(?:-0*(\d+))?")


def _task_dirs(folder: str) -> list[Path]:
    """``docs/{tasks|archive}/`` 의 Task 폴더 — 없으면 빈 목록."""
    d = DOCS / folder
    return sorted(t for t in d.iterdir() if t.is_dir()) if d.is_dir() else []


def _norm_key(token: str) -> str | None:
    """``00``·``#00``·``0a``·``24-2`` → 정규 번호 ``0``·``0a``·``24-2``. 번호가 아니면 ``None``.

    원장 출처·폴더명이 같은 번호를 ``#00``·``#0``·``00.`` 로 달리 적어도 한 주소로 읽는다
    (앞자리 0 을 지우고 ``#0(?![0-9])`` 로 찾던 옛 방식은 ``#00`` 을 놓쳤다 — 실측).
    """
    m = _KEY_RE.match(token.strip())
    if not m:
        return None
    return f"{int(m.group(1))}{m.group(2)}" + (f"-{int(m.group(3))}" if m.group(3) else "")


def _task_number(key: str) -> str | None:
    """폴더명 ``12.someTask`` → ``12`` · ``24-2.x`` → ``24-2`` · ``0a.x`` → ``0a`` · ``007.x`` → ``7``.

    번호로 시작하지 않으면 ``None`` — 그 자체가 **명명 규약 위반**이다.
    면제로 두면 새 Task 를 ``experiment`` 같은 이름으로 만들어 gate 규약 전체를 우회한다.
    """
    m = _FOLDER_RE.match(key)
    return _norm_key(m.group(1)) if m else None


def _started(task_dir: Path) -> bool:
    """구현이 시작됐는가 — ``work/stage*.md`` 가 1건 이상."""
    work = task_dir / "work"
    return work.is_dir() and any(work.glob("stage*.md"))


def _is_discarded(task_dir: Path) -> bool:
    """**미착수 폐기** Task 인가 (plan·eval·gate·환류 규약 면제 조건).

    종결 기준은 `report.md` 를 요구하는데 평가자 규약은 `report.md` 존재를 **구현 시작**으로
    읽으므로, 착수조차 하지 않은 폐기 Task 에 plan·eval 을 요구하는 모순이 생긴다.

    🔴 면제는 **두 조건을 함께** 요구한다 — 이것이 :data:`GATE_LEGACY_KEYS`(동결된 역사적
    집합)와 다른 점이다. 여기는 상시 경로이므로 선언만으로 열리면 안 된다:

    1. `report.md` 가 :data:`DISCARD_MARKER` 로 폐기를 **선언**하고,
    2. 구현 흔적이 **없다**(`work/stage*.md` 0건).

    2가 방어선이다. 선언만으로 면제하면 "폐기라고 써 두고 구현" 이 규약을 통째로 우회한다.
    """
    report = task_dir / "report.md"
    if _started(task_dir) or not report.is_file():
        return False
    return DISCARD_MARKER in report.read_text(encoding="utf-8")


def _ledger_vs_plan(plan: Path, eval_text: str) -> list[str]:
    """원장이 plan 의 계약을 **그대로** 담는지.

    `ledger_unmet` 은 원장 안의 항목만 본다 — E1~E9 짜리 plan 을 `[x] E1` 한 건짜리
    원장으로 줄여도 "미충족 0건" 이 된다. **조용한 범위 축소**가 정확히 이 모양이다.
    id 만 대조하면 제목·CHECK·EXPECT 를 갈아 끼운 원장이 통과하므로 넷을 함께 본다
    (EVIDENCE·박스는 **판정 결과**라 뺀다 — 원장은 plan 에서 다시 렌더된다).
    """
    if not plan.is_file():
        return []
    contract, _ = parse_gates(section_body(plan.read_text(encoding="utf-8"), PLAN_SECTION) or "")
    ledger, abandoned = parse_gates(section_body(eval_text, LEDGER_SECTION) or "")
    if not contract:
        return []
    problems: list[str] = []
    for name, items in (("plan", contract), ("원장", ledger)):
        ids = [g.id for g in items]
        dupes = sorted({i for i in ids if ids.count(i) > 1})
        if dupes:
            problems.append(f"{name} 에 중복된 gate {dupes} — 판정이 둘이면 어느 쪽이 계약인지 흐려진다")
    contract_map = {g.id: g for g in contract}
    ledger_map = {g.id: g for g in ledger}
    missing = [i for i in contract_map if i not in ledger_map]
    added = [i for i in ledger_map if i not in contract_map]
    if missing:
        problems.append(f"원장에 빠진 gate {sorted(missing)} — plan 의 평가 항목이 조용히 줄었다")
    if added:
        problems.append(f"원장에만 있는 gate {sorted(added)} — plan(계약)에 없는 항목이다")
    for i in sorted(set(contract_map) & set(ledger_map)):
        for attr, name in (("title", "제목"), ("check", "CHECK"), ("expect", "EXPECT")):
            if getattr(contract_map[i], attr) != getattr(ledger_map[i], attr):
                problems.append(f"{i} 의 {name} 이 plan 과 다르다 — 원장은 계약을 다시 렌더한 것이어야 한다")
    # 🔴 빠짐·더함이 있으면 순서 비교는 **같은 사실을 두 번 말한다** — 그때는 침묵한다.
    if not missing and not added and [g.id for g in contract] != [g.id for g in ledger]:
        problems.append("원장의 gate 순서가 plan 과 다르다")
    # ABANDON 은 사람이 적는다 — **계약에 있는 id 만** 인정한다.
    unknown = sorted(set(abandoned) - set(contract_map))
    if unknown:
        problems.append(f"plan 에 없는 gate 를 ABANDON 했다 {unknown} — 계약 밖의 포기는 인정하지 않는다")
    return problems


#: 측정과 무관한 **생성물** — 인터프리터·빌드가 만든다. 이것을 코드 변경으로 세면
#: `.gitignore` 가 아직 없는 갓 부트스트랩한 리포에서 신선도가 곧바로 FAIL 한다(실측).
BUILD_ARTIFACTS = ("__pycache__/", ".pytest_cache/", ".mypy_cache/", ".ruff_cache/",
                   "node_modules/", ".venv/", "dist/", "build/")


def _is_code_path(path: str) -> bool:
    """`docs/` 밖 · 루트 마크다운 밖 · **생성물 밖** = 측정을 무효화하는 변경."""
    path = path.strip()
    if not path or path.startswith("docs/"):
        return False
    if path.endswith((".pyc", ".pyo")) or any(a in path for a in BUILD_ARTIFACTS):
        return False
    return not (path.count("/") == 0 and path.endswith(".md"))


def _closing_now(report: Path) -> bool:
    """지금 **종결하는 중**인가 — ``report.md`` 가 아직 **HEAD 에 없다**.

    🔴 종결 Task 의 신선도를 통째로 면제하면 *측정 → 코드 수정
    → report 작성 → archive 이동* 을 **한 커밋**에 담아 stale 측정으로 종결할 수 있다.
    종결 직전이야말로 신선도가 가장 중요한 시점인데 거기서만 검사가 꺼져 있었다.

    🔴 판정은 **HEAD tree** 로 한다 — ``git ls-files`` 는 **index** 를 보므로
    ``git add report.md`` 한 것만으로 tracked 가 되어 검사가 꺼졌다.
    체커를 pre-commit 훅으로 걸면 index 는 **항상** staged 상태라 우회가 상시화된다.

    반대로 **이미 커밋된**(HEAD 에 있는) 종결 Task 는 다시 재지 않는다 — 그 원장은 그 시점의
    측정 기록이고, 뒤에 온 커밋이 그것을 거짓으로 만들지 않는다(그러지 않으면 archive 가
    영원히 빨갛다). 종결 커밋 **이후**의 코드 변경은 종결 후 작업이며 이 예외의 범위다.

    HEAD 를 못 읽으면(커밋 0개 리포·git 오류) **종결 직전으로 본다** — fail-closed 다.
    """
    proc = subprocess.run(
        ["git", "cat-file", "-e", f"HEAD:{report.relative_to(REPO).as_posix()}"],
        cwd=REPO, capture_output=True, text=True,
    )
    return proc.returncode != 0



def _measurement_freshness(eval_text: str) -> list[str]:
    """원장의 **측정 커밋** 이후 코드가 바뀌지 않았는지.

    docs/etc/eval-protocol.md "순환 방지 규칙" 4 의 기계화 — 코드를 바꾼 뒤 남은 옛 수치는 증거가
    아니다. 🔴 HEAD 와의 **완전 일치**를 요구하지는 않는다: 원장을 커밋하는 순간 HEAD 가
    움직여 영원히 FAIL 이 된다. 대신 그 sha 이후 **`docs/` 밖 변경 0** 을 요구한다 —
    문서만 바뀌면 측정은 유효하다(같은 규칙 3).
    """
    body = section_body(eval_text, LEDGER_SECTION) or ""
    sha = MEASURED_SHA_RE.findall(body)
    if len(sha) != 1:
        return [f"원장의 '측정 커밋:' 이 {len(sha)}건 — 정확히 1건이어야 한다 "
                "(tools/check_gates.py 로 갱신할 것)"]
    proc = subprocess.run(["git", "diff", "--name-only", f"{sha[0]}..HEAD"],
                          cwd=REPO, capture_output=True, text=True)
    if proc.returncode != 0:
        return [f"측정 커밋 {sha[0][:8]} 을 git 이 모른다 — 원장을 다시 뜰 것"]
    # 🔴 커밋된 것만 보면 **작업 트리**가 빈다 — 측정 뒤 고친 코드를
    #    커밋하지 않은 채 종결 검사를 지나면 옛 측정이 유효하다고 판정된다.
    status = subprocess.run(["git", "status", "--porcelain"], cwd=REPO,
                         capture_output=True, text=True)
    # 🔴 rc 를 안 보면 status 실패(권한·safe.directory)가
    #    "미커밋 변경 0건" 으로 읽힌다. 검사가 조용히 꺼지는 자리다.
    if status.returncode != 0:
        tail = (status.stderr or "").strip().split(chr(10))[-1][:120]
        return [f"git status 가 실패했다 (rc={status.returncode}) — 미커밋 변경을 확인할 수 없다: {tail}"]
    uncommitted = [line[3:].strip().strip('"') for line in (status.stdout or "").split(chr(10)) if line.strip()]
    changed = [f for f in list(proc.stdout.split(chr(10))) + uncommitted
            if f.strip() and _is_code_path(f)]
    if changed:
        preview = ", ".join(sorted(changed)[:3])
        return [f"측정 커밋 {sha[0][:8]} 이후 코드가 {len(changed)}건 바뀌었다 ({preview}) — "
                "옛 측정은 증거가 아니다. gate 를 다시 돌릴 것"]
    return []


def _duplicate_numbers() -> list[str]:
    """canonical 번호가 둘 이상 쓰이는지.

    같은 번호로 새 Task 를 만들면 legacy key 옆에 붙어 사람 눈에 안 띈다 — 번호는
    Task 의 **주소**이므로 중복은 그 자체로 규약 위반이다.
    """
    number_map: dict[str, list[str]] = {}
    for folder in ("tasks", "archive"):
        for t in _task_dirs(folder):
            # 🔴 주소는 `24` 와 `24-2` 를 **다른 것**으로 본다 — `-n` 은 같은 번호의
            #    하위 트랙이다(`24-2.someTask`). `0a` 도 `0` 과 다른 주소다. `07` 과 `7` 은 같다.
            n = _task_number(t.name)
            if n is not None:
                number_map.setdefault(n, []).append(f"{folder}/{t.name}")
    return [f"Task 번호 {n} 이 {sorted(keys)} 로 중복됐다 — 번호는 Task 의 주소다"
            for n, keys in sorted(number_map.items()) if len(keys) > 1]


def _eval_problems(task_dir: Path) -> list[str]:
    """적대적 평가자 · 평가 항목 **gate 규약** 검사 — ddd-workflow.md "적대적 평가자" 가 계약이다.

    평가 항목을 **구현 전에** 고정하지 않으면 "만든 것에 맞춰 채점" 이 되고, 서술형 항목은
    **실행되지 않아서** 구현한 세션이 손으로 PASS 를 적게 된다. 판정을 명령에 맡기는 것이
    이 규약이다. 면제는 :data:`GATE_LEGACY_KEYS`(규약 도입 전 Task 에 평가 기록을 사후에
    지어내게 하지 않는다)와 미착수 폐기(평가할 구현이 없다)뿐.

    보는 것 — 시점이 다르다:

    1. 구현이 시작됐으면 `plan.md` 가 있다 (없으면 "plan 이 없으니 평가 항목 검사도 없다" 로
       규약이 통째로 우회된다). `plan.md` 가 있으면 평가 항목이 **gate 형식**이다 (계약 고정 시점).
    2. `eval.md` 가 있으면 `## 평가 원장` 절이 있다 (기계가 한 번은 돌았다).
    3. `report.md` 가 있으면 `eval.md` 가 있고 **미충족 0건** · 원장 = plan · 미승인 개정 0건 (종결 시점).

    🔴 2와 3을 가른 이유: 평가 라운드 중에는 gate 가 빨간 것이 정상이다. 그 순간까지
    FAIL 로 막으면 *고치는 중* 인 상태에서 커밋을 못 한다. 종결에서 0건을 요구하면
    자기 채점 경로는 그대로 닫힌다.
    """
    key = task_dir.name
    where = task_dir.relative_to(REPO).as_posix()
    if _task_number(key) is None:
        return [f"{where}/: Task 폴더 이름이 '{{NN}}.{{기능명}}' 이 아니다 — "
                "번호가 없으면 gate 규약을 통째로 우회한다"]
    if key in GATE_LEGACY_KEYS or _is_discarded(task_dir):
        return []
    found: list[str] = []
    plan, evaluation = task_dir / "plan.md", task_dir / "eval.md"
    closed = (task_dir / "report.md").is_file()
    if (_started(task_dir) or closed) and not plan.is_file():
        found.append(f"{where}/: 구현이 시작됐는데 plan.md 가 없다 — "
                     "평가 항목을 구현 전에 고정하는 규약이 우회된다")
    if plan.is_file():
        plan_text = plan.read_text(encoding="utf-8")
        # 🔴 문자열 포함이 아니라 **heading 파싱**이다 — 본문·코드 예시에 같은 문자열만 있어도
        #    통과하면 규약이 뚫린다.
        body = section_body(plan_text, PLAN_SECTION)
        if body is None:
            found.append(f"{where}/plan.md: '{PLAN_SECTION}' 절 없음 — "
                         "평가 항목은 구현 전(plan)에 정확값·판정 방법과 함께 고정한다")
        elif not parse_gates(body)[0]:
            found.append(f"{where}/plan.md: '{PLAN_SECTION}' 에 gate 가 0건이다 "
                         "— '- [ ] E1: ...' + CHECK/EXPECT/EVIDENCE 로 적을 것")
        # 승인 전 개정은 `사용자 미승인` 으로 표기한다(eval-protocol 규약 3) — 종결에 남으면 미승인 계약이다.
        if closed and UNAPPROVED_MARK in visible_text(plan_text):
            found.append(f"{where}/plan.md: '{UNAPPROVED_MARK}' 개정이 남아 있다 — "
                         "종결 전 사용자 승인을 받거나 되돌린다 (eval-protocol 규약 3)")
    if not evaluation.is_file():
        if closed:
            found.append(f"{where}/: report.md 는 있는데 eval.md 가 없다 — "
                         "종결 전에 적대적 평가를 수행하고 결과를 남길 것")
        return found
    eval_text = evaluation.read_text(encoding="utf-8")
    if section_body(eval_text, LEDGER_SECTION) is None:
        return found + [f"{where}/eval.md: '{LEDGER_SECTION}' 절 없음 — tools/check_gates.py {where} 를 실행할 것"]
    # 🔴 신선도는 *진행 중* Task 에서 종결을 기다리지 않고 본다. 라운드 중 gate 가 빨간 것은
    #    정상이지만, *옛 측정을 초록인 채로 두는 것*은 아니다. 코드를 고쳤으면 원장을 다시 뜬다.
    # 🔴 **단 종결한 Task 는 제외한다** — 종결 원장은 *그 시점의 측정 기록*이고 뒤에 온 커밋이
    #    그것을 거짓으로 만들지 않는다. 이 예외가 없으면 **archive 의 모든 Task 가 영원히
    #    빨갛다** — 그러면 아무도 체커를 안 본다.
    # 🔴 종결 **직전**(report.md 미커밋)에는 본다 — 그 커밋이 stale 측정을 실어 나르는 유일한 경로다.
    reasons: list[str] = []
    if not closed or _closing_now(task_dir / "report.md"):
        reasons += _measurement_freshness(eval_text)
    # 🔴 **판정 중인 Task 는 자기 원장을 읽지 않는다.** `check_gates` 가 이 검사를 CHECK 로
    #    부르면 원장은 아직 **이전 실행**의 결과다 — 그것을 근거로 실패시키면 원장이 자기
    #    자신 때문에 영원히 빨갛다.
    if closed and os.environ.get("CHECK_GATES_TASK", "") != where:
        # 종결 Task 는 원장의 **기록된 판정**을 읽는다 (CHECK 재실행은 check_gates 의 몫).
        reasons += ledger_unmet(eval_text) + _ledger_vs_plan(plan, eval_text)
    return found + [f"{where}/eval.md: {reason}" for reason in reasons]


def _sections(text: str) -> list[tuple[str, list[str]]]:
    """``## `` heading → 그 본문(다음 ``## `` 전까지). 순서 보존.

    열 0 고정: 들여쓴 heading 은 heading 이 아니다.
    """
    sections: list[tuple[str, list[str]]] = []
    # 🔴 예시 코드블록·주석 안의 heading 은 계약이 아니다 — 계약을 지우고 예시로
    #    위장하는 우회를 막는다.
    for line in visible_text(text).splitlines():
        if line.startswith("## "):
            sections.append((line.strip(), []))
        elif sections:
            sections[-1][1].append(line)
    return sections


def _heading_matches(heading: str, name: str) -> bool:
    """정확 일치, 또는 괄호 부연만 허용: ``## 도메인 규칙 (2026-01-01 기준…)``.

    임의 접미사("## 도메인 규칙 강령" 류)로 계약 heading 을 위장하는 것을 막는다.
    """
    return heading == f"## {name}" or heading.startswith(f"## {name} (")


def _required_sections(label: str, text: str, names: tuple[str, ...], source: str) -> list[str]:
    """필수 섹션 heading + **본문** — heading 만 남기고 비우면 "적었다" 는 침묵이 된다."""
    sections = _sections(text)
    found: list[str] = []
    for name in names:
        bodies = [body for h, body in sections if _heading_matches(h, name)]
        if not bodies:
            found.append(f"{label}: 필수 섹션 '## {name}' 없음 ({source})")
        elif not any(ln.strip() for body in bodies for ln in body):
            found.append(f"{label}: '## {name}' 본문이 비어 있다 — 없으면 '없음' 이라고 적는다")
    return found


def _table_rows(body: str, head: str) -> list[list[str]]:
    """절 본문의 표 행(머리 ``head`` 행·구분선 제외) — 셀 목록."""
    rows: list[list[str]] = []
    for line in body.splitlines():
        line = line.strip()
        if not line.startswith("|") or set(line) <= set("|- :"):
            continue
        cells = [c.strip() for c in line.strip("|").split("|")]
        if cells and cells[0] == head:
            continue
        rows.append(cells)
    return rows


def _ledger_rows(state_text: str) -> list[list[str]]:
    """`## 미조치 원장` 절의 표 본문 행 — 머리·구분선을 뺀 것."""
    return _table_rows(section_body(state_text, LEDGER_HEADING) or "", head="출처")


def _ledger_sources(state_text: str) -> set[str]:
    """원장 출처 칸의 Task 번호(정규형) 집합 — ``#00 R3-F2`` → ``{"0"}``."""
    found: set[str] = set()
    for cells in _ledger_rows(state_text):
        for token in _TASK_REF_RE.findall(cells[0] if cells else ""):
            if (n := _norm_key(token)) is not None:
                found.add(n)
    return found


def _leftover_problems(task_dir: Path, sources: set[str]) -> list[str]:
    """archive Task 의 ``eval.md`` ``## 미조치 지적`` ↔ 원장.

    🔴 **이 검사가 원장을 살아 있게 한다.** 없으면 원장은 한 번 쓰고 낡는다. 형식을 ``없음``
    또는 표로 **고정**해 셀 수 있게 한다 — 산문·bullet 은 "남은 지적 없음" 과 "남은 지적: …"
    을 기계가 가르지 못한다(키워드 휴리스틱은 정확히 그 문장에서 오탐했다). 절 자체가 없는 것도
    위반이다 — 침묵은 "없음" 이 아니다.
    """
    f = task_dir / "eval.md"
    if not f.is_file():
        return []
    body = section_body(f.read_text(encoding="utf-8"), LEFTOVER_HEADING)
    rel = f"docs/{task_dir.parent.name}/{task_dir.name}"
    if body is None:
        # 🔴 절이 없으면 "남은 지적이 없다" 와 "적지 않았다" 를 가를 수 없다 — 없으면 `없음` 이라고 적는다.
        return [f"{rel}/eval.md: '{LEFTOVER_HEADING}' 절 없음 — 남은 지적이 없으면 `없음` 한 줄, "
                "있으면 표(지적 · 판단 · 사유)로 적는다 (eval-protocol 평가 실행 절차)"]
    for ln in (s.strip() for s in body.splitlines() if s.strip()):
        if ln != "없음" and not ln.startswith("|"):
            return [f"{rel}/eval.md: '{LEFTOVER_HEADING}' 은 `없음` 또는 표(지적 · 판단 · 사유)만 "
                    f"담는다 — 산문·bullet 은 셀 수 없다: {ln[:40]!r}"]
    rows = _table_rows(body, head=LEFTOVER_HEAD)
    if rows and _task_number(task_dir.name) not in sources:
        return [f"{LEDGER_DOC} 미조치 원장에 {rel} 이 없다 — 그 eval.md '{LEFTOVER_HEADING}' 표에 "
                f"{len(rows)}행이 있다 (출처 `#{task_dir.name.split('.', 1)[0]} …` 로 등재 — 종결 의무)"]
    return []


def _task_sort_key(key: str) -> tuple[int, str, int, str]:
    """Task 폴더명 → 정렬 키. 문자열 정렬은 ``#100`` 을 ``#11`` 앞에 둔다(실측) — 숫자로 읽는다.

    오버플로 글자(``0a``)와 하위 트랙(``24-2``)은 번호 뒤 순서로 붙인다.
    """
    m = re.match(r"(\d+)([a-z]?)(?:-(\d+))?$", _task_number(key) or "")
    if not m:
        return (10 ** 6, "", 0, key)
    return (int(m.group(1)), m.group(2), int(m.group(3) or 0), key)


def derive_tasks() -> list[tuple[str, str, str, str]]:
    """Task 목록을 **폴더에서 파생**한다 — (상태, key, 문서 경로, 범위 한 줄).

    상태는 어디에도 적지 않는다: ``docs/tasks/`` = ``⏳``, ``docs/archive/`` = ``✅``,
    그중 ``report.md`` 에 :data:`DISCARD_MARKER` 가 있으면 ``⛔``. 표로 적어 두면 폴더와
    어긋날 수 있지만, 파생은 어긋날 자리가 없다.
    """
    rows: list[tuple[str, str, str, str]] = []
    for folder in ("tasks", "archive"):
        for task_dir in _task_dirs(folder):
            report = task_dir / "report.md"
            state = TASK_STATES[folder]
            if folder == "archive" and report.is_file() and DISCARD_MARKER in report.read_text(encoding="utf-8"):
                state = TASK_STATES["discard"]
            doc = report if report.is_file() else task_dir / "intent.md"
            scope = ""
            intent = task_dir / "intent.md"
            if intent.is_file():
                body = section_body(intent.read_text(encoding="utf-8"), "## 범위") or ""
                scope = next((ln.strip(" -*\t") for ln in body.splitlines() if ln.strip()), "")
            rows.append((state, task_dir.name, doc.relative_to(REPO).as_posix(), scope[:60]))
    return sorted(rows, key=lambda r: _task_sort_key(r[1]))


def _ledger_row_problems(state_text: str) -> list[str]:
    """원장 행의 **빈 칸**과 **상태 어휘**를 본다.

    🔴 *마지막 확인* 이 비면 그 행이 **언제** 사실이었는지 알 수 없다 — 원장의 값어치가
    바로 그 시점이다 — 뒤 Task 가 조용히 갚아도 시점 기록이 없으면 아무도 모른다.
    """
    found: list[str] = []
    rows = _ledger_rows(state_text)
    for cells in rows:
        summary = cells[1][:28] if len(cells) > 1 else "?"
        if len(cells) != LEDGER_COLUMNS:
            found.append(f"{LEDGER_DOC} 미조치 원장: 열이 {len(cells)}개다 "
                         f"(정확히 {LEDGER_COLUMNS}: 출처·요지·판단·상태·마지막 확인) — {summary}")
            continue
        if any(not c for c in cells[:LEDGER_COLUMNS]):
            found.append(f"{LEDGER_DOC} 미조치 원장: **빈 칸**이 있다 — {cells[0]} {summary}")
        if cells[3] not in LEDGER_STATES:
            found.append(f"{LEDGER_DOC} 미조치 원장: 상태 어휘가 아니다 {cells[3]!r} "
                         f"(허용 {list(LEDGER_STATES)}) — {cells[0]}")
    return found


def _intent_problems(task_dir: Path) -> list[str]:
    """intent.md 계약 — 존재 · 필수 6 섹션 heading + 본문 · ``승인:`` 줄 · plan 전 승인.

    intent 는 그 Task 의 **무엇·왜** 의 단일 기록이다(docs/etc/intent-protocol.md). 없으면 plan 이
    의도 없이 방법을 적은 것이고, ``승인: 대기`` 인데 plan 이 있으면 인터뷰(사용자 확인)를 건너뛴
    것이다. archive 도 본다 — 전 Task 의 의도가 남아야 "전체 의도" 가 모인다. 면제는
    :data:`GATE_LEGACY_KEYS` 뿐(규약 도입 전 Task 에 의도를 사후 작성하게 만들지 않는다).
    ``승인:`` 줄은 **존재**도 요구한다 — 줄을 지우는 것이 우회가 되면 안 된다.
    """
    key = task_dir.name
    if key in GATE_LEGACY_KEYS:
        return []
    rel = f"docs/{task_dir.parent.name}/{key}"
    intent = task_dir / "intent.md"
    if not intent.is_file():
        return [f"{rel}/: intent.md 없음 — Task 는 인터뷰로 확정한 의도에서 시작한다 "
                "(골격은 docs/etc/intent-protocol.md)"]
    text = intent.read_text(encoding="utf-8")
    found = _required_sections(f"{rel}/intent.md", text, INTENT_MD_SECTIONS, "intent-protocol 규약 3")
    approvals = INTENT_APPROVAL_RE.findall(visible_text(text))
    if len(approvals) != 1:
        found.append(f"{rel}/intent.md: '승인:' 줄이 {len(approvals)}건 — 정확히 1건"
                     "('승인: 대기' 또는 '승인: {YYYY-MM-DD}', 열 0)")
    elif approvals[0].strip() in ("", INTENT_PENDING) and (task_dir / "plan.md").is_file():
        found.append(f"{rel}/intent.md: '승인: 대기' 인데 plan.md 가 있다 — 인터뷰로 의도를 확정"
                     "(사용자 확인 → 승인: {날짜})한 뒤 방법을 쓴다")
    return found


def _coverage_problems(task_dir: Path) -> list[str]:
    """intent 의 ``## 성공 기준`` A* 를 plan 의 gate 가 **전부** 덮는가 (intent-protocol 규약 4).

    덮이지 않은 기준은 조용한 축소다 — 구현이 끝난 뒤 적대적 평가자가 물어야 드러나고,
    그때는 이미 "범위 밖" 으로 밀 유인이 생긴다. 대응 표시는 **gate 제목 첫머리의** ``A{n}`` 이며
    (기계 판정이 어려워 수동 gate 로 두더라도 같다), 그 표시가 계약이다. 번호가 아예 없으면
    대조가 성립하지 않으므로 그것도 위반이다 — 번호를 지우는 것이 우회가 되면 안 된다.
    미착수 폐기 Task 는 면제 — 평가할 구현이 없다(:func:`_is_discarded`).
    """
    if task_dir.name in GATE_LEGACY_KEYS or _is_discarded(task_dir):
        return []
    intent, plan = task_dir / "intent.md", task_dir / "plan.md"
    if not (intent.is_file() and plan.is_file()):
        return []
    rel = f"docs/{task_dir.parent.name}/{task_dir.name}"
    body = section_body(intent.read_text(encoding="utf-8"), INTENT_SUCCESS_SECTION)
    if body is None:
        return []                       # 섹션 부재는 _intent_problems 가 잡는다
    items = [ln.strip().lstrip("-*").strip() for ln in body.splitlines() if ln.strip()]
    if not items or all(it.startswith(NO_CRITERIA) for it in items):
        return []
    criteria = {int(n) for n in CRITERION_RE.findall(body)}
    if not criteria:
        return [f"{rel}/intent.md: '## 성공 기준' 에 번호(A1·A2…)가 없다 — '- A1: …' 줄로 적는다. plan gate 와의 "
                "대응을 기계가 대조할 수 없다 (intent-protocol 규약 4)"]
    gates, _ = parse_gates(section_body(plan.read_text(encoding="utf-8"), PLAN_SECTION) or "")
    covered = {int(n) for gate in gates if (lead := GATE_CRITERIA_RE.match(gate.title))
               for n in re.findall(r"\d+", lead.group(0))}
    missing = sorted(criteria - covered)
    if missing:
        names = ", ".join(f"A{n}" for n in missing)
        return [f"{rel}/plan.md: 성공 기준 {names} 을 덮는 gate 가 없다 — gate 제목 첫머리에 'A{{n}}' 을 "
                "적어 대응을 드러낸다(수동 gate 라도). 기준을 범위 밖으로 미는 것은 intent 개정"
                "(사용자 승인)으로만 (intent-protocol 규약 4)"]
    return []


def _feedback_problems(task_dir: Path) -> list[str]:
    """종결 Task 가 masterPlan ``## 개정 기록`` 에 환류 한 줄을 남겼는가 (intent-protocol 규약 5).

    intent 의 결정·명세 변경분은 종결 시 masterPlan 으로 **내용을 옮긴다**. 옮기지 않으면
    프로젝트 수준 의도가 Task 폴더에 갇히고, 다음 Task 는 이미 뒤집힌 D 를 인용한다.
    체커가 보는 것은 **그 Task 번호를 단 줄의 존재**다 — 내용까지는 못 본다(환류할 것이
    없었다면 그 사실을 한 줄로 적는다: 그것이 "봤고 없었다" 와 "안 봤다" 를 가르는 유일한 기록).
    미착수 폐기 Task 는 면제 — 옮길 결정이 없다.
    """
    key = task_dir.name
    if key in GATE_LEGACY_KEYS or not (task_dir / "report.md").is_file() or _is_discarded(task_dir):
        return []
    number = _task_number(key)
    if number is None:
        return []                       # 명명 규약 위반은 별도 검사가 잡는다
    master = REPO / MASTERPLAN
    body = section_body(master.read_text(encoding="utf-8"), REVISION_SECTION) if master.is_file() else None
    if body is None:
        return [f"{MASTERPLAN}: '{REVISION_SECTION}' 절 없음 — 종결 Task 의 환류를 적을 자리다 "
                "(intent-protocol 규약 5)"]
    if number not in {_norm_key(t) for t in _TASK_REF_RE.findall(body)}:
        return [f"docs/{task_dir.parent.name}/{key}/: 종결했는데 {MASTERPLAN} '{REVISION_SECTION}' 에 "
                f"#{number} 줄이 없다 — intent 의 결정·명세 변경분을 masterPlan 으로 옮기고 "
                "한 줄(날짜 · Task #N · D/S 번호 · 무엇을)을 남긴다 (intent-protocol 규약 5)"]
    return []


def _masterplan_problems() -> list[str]:
    """masterPlan 의 ``승인:`` 줄 — **프로젝트 수준 의도**의 사용자 확인 (Task intent 의 승인 줄과 같은 표지).

    Task intent 는 ``승인: 대기`` 인 채 plan 을 쓰면 FAIL 하는데, 그 Task 들이 인용하는 masterPlan
    (의도 · D · S)에는 같은 표지가 없었다 — 부트스트랩 인터뷰의 "공유 이해" 확인이 기록 없이 지나간다.
    ``승인: 대기`` 인 채 Task 폴더를 열면 확정되지 않은 의도를 인용한 것이다.
    문서의 부재는 여기서 보지 않는다 — 루트 문서의 dead link 가 잡는다.
    """
    master = REPO / MASTERPLAN
    if not master.is_file():
        return []
    approvals = INTENT_APPROVAL_RE.findall(visible_text(master.read_text(encoding="utf-8")))
    if len(approvals) != 1:
        return [f"{MASTERPLAN}: '승인:' 줄이 {len(approvals)}건 — 정확히 1건"
                "('승인: 대기' 또는 '승인: {YYYY-MM-DD}', 열 0)"]
    opened = any(t.name not in GATE_LEGACY_KEYS for folder in ("tasks", "archive") for t in _task_dirs(folder))
    if approvals[0].strip() in ("", INTENT_PENDING) and opened:
        return [f"{MASTERPLAN}: '승인: 대기' 인데 Task 폴더가 있다 — 프로젝트 수준 의도를 사용자와 확정"
                "(승인: {날짜})한 뒤 Task 를 연다"]
    return []


def _state_md_problems() -> list[str]:
    """STATE.md 계약 — 필수 2 섹션 heading + 본문. **누적 기록은 여기 없다.**

    STATE.md 는 매 단계 덮어쓰는 재개 좌표다. Task 목록은 폴더에서 파생하고(:func:`derive_tasks`),
    미조치 원장은 :data:`LEDGER_DOC` 가 갖는다 — 덮어쓰는 문서와 누적 문서를 섞지 않는다.
    """
    state = REPO / "STATE.md"
    if not state.is_file():
        return ["루트 STATE.md 가 없다 — 재개 좌표(현재 작업 · 다음 행동)의 단일 위치가 필수다 "
                "(ddd-workflow.md 'STATE.md — 필수 2절')"]
    return _required_sections("STATE.md", state.read_text(encoding="utf-8"), STATE_MD_SECTIONS,
                              "ddd-workflow.md 'STATE.md — 필수 2절'")


def _ledger_problems() -> list[str]:
    """미조치 원장 — 전용 문서(:data:`LEDGER_DOC`)의 5열 · 상태 어휘 · archive Task 등재."""
    ledger = REPO / LEDGER_DOC
    if not ledger.is_file():
        return [f"{LEDGER_DOC} 가 없다 — 적대적 평가 미조치·반박·조건부의 단일 추적처가 필수다 "
                "(ddd-workflow.md '미조치 원장')"]
    text = ledger.read_text(encoding="utf-8")
    if section_body(text, LEDGER_HEADING) is None:
        return [f"{LEDGER_DOC}: '{LEDGER_HEADING}' 절 없음 — 표의 자리가 고정이어야 기계가 센다"]
    found = _ledger_row_problems(text)
    sources = _ledger_sources(text)
    for task_dir in _task_dirs("archive"):
        if task_dir.name not in GATE_LEGACY_KEYS:
            found.extend(_leftover_problems(task_dir, sources))
    return found


def _product_md_problems() -> list[str]:
    """PRODUCT.md 계약 — 존재 · 필수 6 섹션 heading + 본문.

    제품(무엇을 만들었는가)과 프로젝트(어떻게 일하는가)가 한 문서에 있으면 구조 변경과 절차
    변경이 같은 diff 에 섞여 무엇이 계약인지 흐려진다.
    """
    product = REPO / "PRODUCT.md"
    if not product.is_file():
        return ["루트 PRODUCT.md 가 없다 — 제품 구조·규칙·설계의 단일 위치가 필수다 "
                "(ddd-workflow.md 'PRODUCT.md — 필수 6절')"]
    return _required_sections("PRODUCT.md", product.read_text(encoding="utf-8"),
                              PRODUCT_MD_SECTIONS, "ddd-workflow.md 'PRODUCT.md — 필수 6절'")


def _project_md_problems() -> list[str]:
    """PROJECT.md 계약 검사 — 존재 · 필수 5 섹션 · 치명 가드레일(줄 수 · 자리별 키워드)."""
    project = REPO / "PROJECT.md"
    if not project.is_file():
        return ["루트 PROJECT.md 가 없다 — 가드레일 · 개발 환경 · 검증의 단일 위치가 필수다 "
                "(ddd-workflow.md 'PROJECT.md — 필수 5절')"]

    text = project.read_text(encoding="utf-8")
    found = _required_sections("PROJECT.md", text, PROJECT_MD_SECTIONS,
                               "ddd-workflow.md 'PROJECT.md — 필수 5절'")
    sections = _sections(text)
    if sections and not _heading_matches(sections[0][0], "치명 가드레일"):
        found.append(
            f"PROJECT.md: 첫 heading 이 '## 치명 가드레일' 이 아니라 '{sections[0][0]}' — "
            "가드레일은 문서 최상단이어야 읽는 순간 최우선 계약이 먼저 들어온다"
        )
    guard = next((body for h, body in sections if _heading_matches(h, "치명 가드레일")), None)
    if guard is not None:
        bullets = [ln.strip() for ln in guard if ln.lstrip().startswith("- ")]
        if len(bullets) != GUARDRAIL_BULLETS:
            found.append(
                f"PROJECT.md: 치명 가드레일 bullet {len(bullets)}개 — 계약은 "
                f"**정확히 {GUARDRAIL_BULLETS}줄**이다. 많으면 가드레일이 아니라 목록이고, "
                "적으면 계약이 비었다"
            )
        if len(GUARDRAIL_KEYWORDS) != GUARDRAIL_BULLETS:
            found.append(
                f"check_ddd_docs.GUARDRAIL_KEYWORDS 가 {len(GUARDRAIL_KEYWORDS)}개다 — "
                f"GUARDRAIL_BULLETS({GUARDRAIL_BULLETS})와 같아야 자리별 검사가 전 줄을 덮는다. "
                "부트스트랩 시 각 줄의 키워드로 작성할 것(zip 이 짧은 쪽으로 잘려 뒷줄 검사가 사라진다)"
            )
        elif any(not k.strip() for k in GUARDRAIL_KEYWORDS):
            found.append(
                "check_ddd_docs.GUARDRAIL_KEYWORDS 에 빈 키워드가 있다 — 빈 문자열은 어느 줄에나 "
                "들어 있어 그 자리의 검사가 꺼진다"
            )
        # 개수만 세면 5줄을 다른 내용으로 바꿔치기해도 통과한다 — 각 줄의 키워드까지 본다.
        # **자리별 1:1** 로 본다. "어딘가에 다 있다" 로 보면 한 줄에 5개를 몰아넣고
        # 나머지 4줄을 무관한 내용으로 바꿔도 통과한다.
        mismatched = [
            f"{i + 1}행={keyword!r}"
            for i, (keyword, bullet) in enumerate(zip(GUARDRAIL_KEYWORDS, bullets))
            if keyword not in bullet
        ]
        if mismatched:
            found.append(
                f"PROJECT.md: 치명 가드레일 자리별 계약 키워드 불일치 {mismatched} — "
                "n행은 GUARDRAIL_KEYWORDS[n-1] 을 담아야 한다. 항목·순서를 바꾸려면 "
                "check_ddd_docs.GUARDRAIL_KEYWORDS 와 함께 개정한다"
            )
    return found


def check() -> list[str]:
    problems = _duplicate_numbers()

    claude = REPO / "CLAUDE.md"
    if not claude.is_file():
        problems.append("루트 CLAUDE.md 가 없다 — DDD 컨텍스트 진입점이 필수다")
    elif hits := TODO_RE.findall(claude.read_text(encoding="utf-8")):
        problems.append(f"CLAUDE.md: 미채움 템플릿 흔적 {TODO_RE.pattern} {len(hits)}건")

    problems.extend(_project_md_problems())
    problems.extend(_product_md_problems())
    problems.extend(_state_md_problems())
    problems.extend(_ledger_problems())
    problems.extend(_masterplan_problems())

    for name in REQUIRED_DIRS:
        if not (DOCS / name).is_dir():
            problems.append(f"docs/{name}/ 없음")

    # 루트 CLAUDE.md·PROJECT.md·PRODUCT.md·STATE.md·README.md 도 docs/ 문서를 가리킨다 —
    # git mv 로 Task 를 archive 로 옮기면 그 링크가 끊긴다.
    roots = [
        p
        for p in (claude, REPO / "PROJECT.md", REPO / "PRODUCT.md", REPO / "STATE.md",
                  REPO / "README.md")
        if p.is_file()
    ]
    for md in [*DOCS.rglob("*.md"), *roots]:
        rel = md.relative_to(REPO).as_posix()
        body = md.read_text(encoding="utf-8")
        problems.extend(_link_problems(md, body))
        # 🔴 울타리 안(인용한 리뷰 원문·명령·나무 그림)은 보지 않는다.
        for path in BACKTICK_DOC_RE.findall(FENCE_RE.sub("", body)):
            # 현존 문서는 링크로 참조한다. 삭제된 역사 경로는 허용.
            if (REPO / path).exists():
                problems.append(f"{rel}: 백틱 문서경로 -> {path} (마크다운 링크로 바꿀 것)")

    # 종결 = archive 이동 — report.md 는 archive/ 에만, archive/ 에는 반드시 있다.
    for folder in ("tasks", "archive"):
        for task_dir in _task_dirs(folder):
            problems.extend(_intent_problems(task_dir))
            problems.extend(_coverage_problems(task_dir))
            problems.extend(_feedback_problems(task_dir))
            problems.extend(_eval_problems(task_dir))
            if (folder == "archive") != (task_dir / "report.md").is_file():
                problems.append(
                    f"docs/tasks/{task_dir.name}/: report.md 가 있으면 종결이다 — "
                    "git mv docs/tasks/... docs/archive/... 로 이동할 것"
                    if folder == "tasks" else
                    f"docs/archive/{task_dir.name}/: report.md 없음 — 종결 기준 미충족이면 "
                    "docs/tasks/ 로 되돌릴 것"
                )
    return problems


def todo_notes() -> list[str]:
    """PROJECT.md·PRODUCT.md·masterPlan 의 **미해소 TODO 집계** — 위반이 아니라 보고 항목이다.

    부트스트랩 정책: 인터뷰로 해소되지 않은 항목은 TODO 로 남기되 최종 보고에
    목록화한다. CLAUDE.md 의 TODO 만 위반으로 검사한다(위 1번).
    """
    notes: list[str] = []
    for rel in ("PROJECT.md", "PRODUCT.md", "STATE.md", "docs/masterPlan.md",
                "docs/manual/README.md"):
        path = REPO / rel
        if path.is_file():
            count = len(TODO_RE.findall(path.read_text(encoding="utf-8")))
            if count:
                notes.append(f"{rel} {count}건")
    return notes


def print_tasks() -> int:
    """``--tasks`` — 폴더에서 파생한 Task 목록. 문서에 적어 두는 표를 **대체**한다."""
    rows = derive_tasks()
    if not rows:
        print("Task 폴더 없음 — 첫 Task 는 docs/tasks/{NN.기능명}/ 개설로 시작한다")
        return 0
    active = [r for r in rows if r[0] == TASK_STATES["tasks"]]
    closed = [r for r in rows if r[0] != TASK_STATES["tasks"]]
    for group, label in ((active, "진행"), (closed, "종결·폐기")):
        if not group:
            continue
        print(f"— {label} {len(group)}건 —")
        for state, key, doc, scope in group:
            print(f"{state} #{key}  {doc}" + (f"  — {scope}" if scope else ""))
    return 0


def main() -> int:
    if "--tasks" in sys.argv[1:]:
        return print_tasks()
    problems = check()
    notes = todo_notes()
    if notes:
        print(f"INFO — 미해소 TODO: {' · '.join(notes)} (허용 — 최종 보고에 목록화할 것)")
    if not problems:
        print("PASS — DDD 문서 규약 위반 없음")
        return 0
    print(f"FAIL — {len(problems)}건")
    for problem in problems:
        print(f"  - {problem}")
    return 1


if __name__ == "__main__":
    sys.exit(main())

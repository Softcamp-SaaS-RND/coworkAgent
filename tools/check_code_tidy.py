#!/usr/bin/env python
"""제품 원본 정리 검사 — 루트 PROJECT.md 검증 명령 "코드 정리".

① 파일 크기: `crates/*/src/**/*.rs` · `crates/app/web/js/**/*.js` 파일마다 1,000줄 이하(`app.css` · `index.html` 은 한 파일로 둔다).
② 주석의 결정 이력 표식 0: 위 파일 + `crates/app/web/app.css` 의 주석에서 Task 번호 · 리뷰 라운드 · 사용자 지시 · 날짜 · 시안 이름 ·
   plan · intent 항목 번호를 센다(이력은 masterPlan · archive 에 있다 — 주석은 왜 · 보안 · 함정만). D · S 번호는 세지 않는다.
   주석은 문자열 · 문자 · 정규식 · 템플릿 리터럴을 건너뛰는 렉서로 가른다(문자열 안의 `//` · 정규식 안의 따옴표에 속지 않게).
마지막 줄 `PASS — 제품 원본 N개 · 최대 M줄 · 이력 표식 K` — 대상 수를 함께 단정한다(대상이 사라지면 N 이 드러난다).

    python tools/check_code_tidy.py

종료 코드: 위반 0 이면 0.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MAX_LINES = 1000
MARKER = re.compile(
    r"(?<![0-9A-Za-z])#(?:\d{1,2}(?:-\d+)?|1[a-z](?:-\d+)?)(?![0-9A-Za-z])"  # Task 번호 #13 · #05-2 · #1g · #1z-8(`PKCS#8` 같은 이름 안의 `#` 은 뺌)
    r"|(?<![\w#.-])1[abcefgijklnopqrtuvwxyz](?:-\d+)?(?!\w)"  # '#' 없이 쓴 Task 번호(1a · 1u — 1s · 1m · 1h · 1d 는 시간 단위와 겹쳐 뺌)
    r"|\b[ATUQE]\d{1,2}\b"  # intent · plan · 인터뷰 항목(A3 · T2 · Q3)
    r"|\bR[1-9]\b|라운드 ?[1-9]|적대적 (?:평가|리뷰)"  # 리뷰 라운드
    r"|사용자 (?:지시|원문|결정|요청|답)|20\d\d-\d\d-\d\d"  # 사용자 지시 · 결정 날짜
    r"|시안|plan 해석|구현자 해석|intent|결정 \d|Task"  # 시안 이름 · plan · intent 항목
)
IDC = re.compile(r"[\w$]")
REGEX_PREV = set("(,=:[!&|?{};+-*%<>~^")
KW_BEFORE_REGEX = {"return", "typeof", "case", "do", "else", "in", "of", "new", "delete", "void", "throw", "yield", "await"}


def skip_quoted(t, i, q):
    """`t[i]` 가 여는 따옴표 — 닫는 따옴표 다음 위치."""
    j = i + 1
    while j < len(t) and t[j] != q:
        j += 2 if t[j] == "\\" else 1
    return j + 1


def rs_comments(t):
    out, i, n = [], 0, len(t)
    while i < n:
        if t.startswith("//", i):
            j = t.find("\n", i)
            j = n if j < 0 else j
            out.append(t[i:j])
            i = j
        elif t.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if t.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif t.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            out.append(t[i:j])
            i = j
        else:
            ident_before = i > 0 and IDC.match(t[i - 1])
            raw = None if ident_before else re.match(r'(?:br|cr|r)(#*)"', t[i:i + 300])
            if raw:
                end = t.find('"' + raw.group(1), i + raw.end())
                i = n if end < 0 else end + 1 + len(raw.group(1))
            elif t[i] == '"' or (not ident_before and t[i] in "bc" and t[i + 1:i + 2] == '"'):
                i = skip_quoted(t, i + (t[i] != '"'), '"')
            elif t[i] == "'" and t[i + 1:i + 2] == "\\":
                i = skip_quoted(t, i, "'")
            elif t[i] == "'" and t[i + 2:i + 3] == "'":
                i += 3
            else:
                i += 1
    return out


def js_comments(t, i=0, inner=False, out=None):
    """`inner` = 템플릿의 `${` 안 — 짝 `}` 위치를 돌려준다."""
    out = [] if out is None else out
    n, depth, prev, word = len(t), 0, "", ""
    while i < n:
        c = t[i]
        if t.startswith("//", i):
            j = t.find("\n", i)
            j = n if j < 0 else j
            out.append(t[i:j])
            i = j
            continue
        if t.startswith("/*", i):
            j = t.find("*/", i + 2)
            j = n if j < 0 else j + 2
            out.append(t[i:j])
            i = j
            continue
        if c in "'\"":
            i, prev = skip_quoted(t, i, c), "s"
            continue
        if c == "`":
            j = i + 1
            while j < n and t[j] != "`":
                if t[j] == "\\":
                    j += 2
                elif t.startswith("${", j):
                    j = js_comments(t, j + 2, True, out) + 1
                else:
                    j += 1
            i, prev = j + 1, "`"
            continue
        if c == "/" and (prev == "" or prev in REGEX_PREV or (prev == "w" and word in KW_BEFORE_REGEX)):
            j, cls = i + 1, False
            while j < n and t[j] != "\n" and (t[j] != "/" or cls):
                if t[j] == "\\":
                    j += 1
                elif t[j] == "[":
                    cls = True
                elif t[j] == "]":
                    cls = False
                j += 1
            j += 1
            while j < n and t[j].isalpha():
                j += 1
            i, prev = j, "r"
            continue
        if IDC.match(c):
            j = i + 1
            while j < n and IDC.match(t[j]):
                j += 1
            prev, word, i = "w", t[i:j], j
            continue
        if c == "{":
            depth += 1
        elif c == "}":
            if inner and depth == 0:
                return i
            depth -= 1
        if not c.isspace():
            prev = c
        i += 1
    return i if inner else out


def css_comments(t):
    out, i, n = [], 0, len(t)
    while i < n:
        if t.startswith("/*", i):
            j = t.find("*/", i + 2)
            j = n if j < 0 else j + 2
            out.append(t[i:j])
            i = j
        elif t[i] in "'\"":
            i = skip_quoted(t, i, t[i])
        else:
            i += 1
    return out


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    capped = sorted(ROOT.glob("crates/*/src/**/*.rs")) + sorted(ROOT.glob("crates/app/web/js/**/*.js"))
    files = capped + [ROOT / "crates/app/web/app.css"]
    problems, marks, longest = [], 0, 0
    for f in files:
        text = f.read_text(encoding="utf-8").replace("\r\n", "\n")
        rel = f.relative_to(ROOT).as_posix()
        if f in capped:
            lines = text.count("\n") + (not text.endswith("\n"))
            longest = max(longest, lines)
            if lines > MAX_LINES:
                problems.append(f"{rel}: {lines}줄 — 상한 {MAX_LINES}")
        comments = rs_comments(text) if f.suffix == ".rs" else js_comments(text) if f.suffix == ".js" else css_comments(text)
        found = [m.group(0) for c in comments for m in MARKER.finditer(c)]
        if found:
            marks += len(found)
            problems.append(f"{rel}: 이력 표식 {len(found)} — {' '.join(found[:6])}{' …' if len(found) > 6 else ''}")
    for p in problems:
        print("  " + p)
    verdict = "PASS" if not problems else "FAIL"
    print(f"{verdict} — 제품 원본 {len(files)}개 · 최대 {longest}줄 · 이력 표식 {marks}")
    return 0 if not problems else 1


if __name__ == "__main__":
    sys.exit(main())

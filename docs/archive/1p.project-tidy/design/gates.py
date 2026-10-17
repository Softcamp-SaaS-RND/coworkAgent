"""1p 평가 항목 측정 — plan.md `## 평가 항목` 의 CHECK 가 부른다.

출력은 ASCII 한 줄(EXPECT 가 단정한다 — 콘솔 코드페이지에서 비-ASCII 는 깨진다). 부정형 단정은 대상 수를 함께 찍는다.

    python {TASK}/design/gates.py size            # A1 · A2 필독 4문서 크기 · CLAUDE.md 필독 목록 · PRODUCT.md 크기
    python {TASK}/design/gates.py moved <sha>     # A1·A2 지운 문장의 사실(코드 이름 · 단위 붙은 수)이 살아 있는 문서에 남았나
    python {TASK}/design/gates.py code            # A3 죽은 코드 · 중복 도우미 · 미사용 의존 · ux_literals 표
    python {TASK}/design/gates.py ux <sha>        # A3 ux_literals 표 전후 단정 짝(있음 · 없음 · 개수 × 파일 × 문구)이 같은가
    python {TASK}/design/gates.py mono            # A5 var(--mono) 를 쓰는 선택자 집합
    python {TASK}/design/gates.py copy [before]   # A6 plan-copy.md 승인 행이 화면 원본에 반영됐나(before = 현재 문구 실재)
    python {TASK}/design/gates.py terms           # A7 용어 표의 쓰지 않는 말이 화면 문구에 없나
    python {TASK}/design/gates.py design          # A11 고른 시안 토큰 = app.css 첫 :root · AI 티 패턴 0
    python {TASK}/design/gates.py contrast        # A11 첫 :root 토큰의 글자/바탕 · 타일 · 말풍선 대비 4.5:1 이상
    python {TASK}/design/gates.py extra <sha>     # A9 로컬 캐시 · STATE 낡은 항목 · 캡처
"""

import hashlib
import html
import json
import re
import subprocess
import sys
from pathlib import Path

TASK = Path(__file__).resolve().parents[1]
ROOT = TASK.parents[2]
WEB = ROOT / "crates" / "app" / "web"
HANGUL = re.compile(r"[가-힣]")

MUST_READ = ["CLAUDE.md", "STATE.md", "PROJECT.md", "docs/etc/ddd-workflow.md"]
MUST_HEADING = "## 매 세션 필독"
SLIMMED = ["CLAUDE.md", "PROJECT.md", "PRODUCT.md", "docs/etc/ddd-workflow.md"]
STATE_STALE = ["agentNetwork-t19", "시안 차이 8건", "미구현 코드 제안 7건", "#1g R2-F1", "1c 원장 미조치"]
SHOT_DOCS = ["README.md", "docs/manual/user/user-guide.md"]


# 승인된 모집단(#1p 적대적 리뷰 라운드 1 · 2) — 개수와 내용(정렬한 항목의 sha256 앞 12자리)을 gate 안에서 단정한다. 대상이 줄거나 · 다른
# 것으로 바뀌거나 · 같은 열쇠가 두 번이면 "위반 0" 이어도 실패한다. 바꿀 때는 plan 개정과 함께.
POP = {"moved": (167, "6b3ce4a92bb6"), "copy": (193, "8fece94740b2"), "terms": (7, "400042cc772a"), "shots": (5, "c7b564ebd03b"), "design": (99, "0fa2acbaa441"),
       "contrast": (122, "6f4b8da5c5f5")}


def digest(items):
    return hashlib.sha256("\n".join(sorted(items)).encode("utf-8")).hexdigest()[:12]


def pin_ok(items, want, keys=None):
    """모집단이 승인값(개수 · digest)과 같고 열쇠(없으면 항목 자체)가 겹치지 않나."""
    keys = list(items if keys is None else keys)
    return len(set(keys)) == len(keys) and (len(items), digest(items)) == want


def pin(name, items, keys=None):
    """모집단이 승인값과 다르면 한 줄 찍고 종료 코드 1 — EXPECT 와 별개로 gate 를 떨어뜨린다."""
    items = list(items)
    if not pin_ok(items, POP[name], keys):
        k = list(items if keys is None else keys)
        print(f"population {name}: want={POP[name]} got={(len(items), digest(items))} dup_keys={len(k) - len(set(k))}")
        sys.exit(1)


def pin_selftest():
    """pin 회귀 — 같은 개수로 한 항목을 다른 항목의 복제로 바꾸거나 · 내용을 바꾸거나 · 열쇠가 겹치면 실패해야 한다."""
    rows = ["A1|x", "A2|y", "A3|z"]
    want = (3, digest(rows))
    assert pin_ok(rows, want, ["A1", "A2", "A3"])
    assert not pin_ok(["A1|x", "A1|x", "A3|z"], want)
    assert not pin_ok(["A1|x", "A2|q", "A3|z"], want)
    assert not pin_ok(rows, want, ["A1", "A1", "A3"])


def text(rel):
    return (ROOT / rel).read_bytes().replace(b"\r", b"").decode("utf-8")


def git_show(sha, rel):
    out = subprocess.run(["git", "show", f"{sha}:{rel}"], cwd=ROOT, capture_output=True)
    return out.stdout.replace(b"\r", b"").decode("utf-8") if out.returncode == 0 else ""


def section(body, heading):
    m = re.search(rf"(?m)^{re.escape(heading)}[^\n]*\n(.*?)(?=^## |\Z)", body, re.S)
    return m.group(1) if m else ""


def live_docs():
    """살아 있는 문서 — archive · 진행 중 Task 폴더는 빼고 찾는다(옮긴 사실이 정본 자리에 있어야 한다)."""
    paths = [ROOT / p for p in ("CLAUDE.md", "PROJECT.md", "PRODUCT.md", "STATE.md", "README.md")]
    for p in (ROOT / "docs").rglob("*.md"):
        rel = p.relative_to(ROOT).as_posix()
        if not rel.startswith(("docs/archive/", "docs/tasks/")):
            paths.append(p)
    return paths


REGEX_BEFORE = re.compile(r"(?:^|[(,=:\[!&|?{};+\-*%<>~^]|\b(?:return|typeof|case|in|of|void|delete|throw|new))\s*$")


def regex_start(s, i):
    """`s[i]` 의 `/` 가 정규식을 여나 — 앞이 연산자 · 여는 괄호 · 키워드이거나, `if (…)` · `while (…)` · `for (…)` · `with (…)` 의 닫는 괄호 뒤
    (문장 자리 — 그 밖의 `)` 뒤는 나눗셈). 괄호 안 문자열 속 괄호는 세지 않는다(휴리스틱)."""
    if REGEX_BEFORE.search(s[max(0, i - 16):i]):
        return True
    k = i - 1
    while k >= 0 and s[k] in " \t\r\n":
        k -= 1
    if k < 0 or s[k] != ")":
        return False
    depth = 0
    while k >= 0:
        depth += {")": 1, "(": -1}.get(s[k], 0)
        if depth == 0:
            break
        k -= 1
    return bool(re.search(r"\b(?:if|while|for|with)\s*$", s[max(0, k - 8):max(k, 0)]))


def regex_end(s, i):
    """`s[i]` 의 `/` 가 정규식 literal 이면 끝(플래그 뒤) 위치, 아니면 None — 줄을 넘으면 나눗셈으로 본다."""
    j, cls = i + 1, False
    while j < len(s) and s[j] != "\n":
        c = s[j]
        if c == "\\":
            j += 2
            continue
        if c == "/" and not cls:
            j += 1
            while j < len(s) and s[j].isalpha():
                j += 1
            return j
        cls = (cls or c == "[") and c != "]"
        j += 1
    return None


def js_literals(src):
    """JS 문자열 · 템플릿 literal — 템플릿은 통째(`${…}` 원문 포함)와 그 `${…}` 안의 literal 을 모두 낸다. 주석 · 정규식은 건너뛴다
    (정규식 속 백틱 `/^\\s*```/` · 중첩 템플릿이 문자열 경계를 어긋내지 않게 — #1p 적대적 리뷰 라운드 1)."""
    out, n = [], len(src)

    def code(i, nested):
        """`i` 부터 코드를 읽는다 — `nested` 면 짝 없는 `}`(템플릿 `${…}` 끝) 다음 자리를 돌려준다."""
        depth = 0
        while i < n:
            c = src[i]
            if src.startswith("//", i):
                j = src.find("\n", i)
                i = n if j < 0 else j
            elif src.startswith("/*", i):
                j = src.find("*/", i + 2)
                i = n if j < 0 else j + 2
            elif c == "/" and regex_start(src, i) and (j := regex_end(src, i)):
                i = j
            elif c in "\"'":
                j = i + 1
                while j < n and src[j] != c:
                    j += 2 if src[j] == "\\" else 1
                out.append(src[i + 1:j])
                i = j + 1
            elif c == "`":
                i = template(i)
            else:
                if c == "{":
                    depth += 1
                elif c == "}":
                    if nested and depth == 0:
                        return i + 1
                    depth -= 1
                i += 1
        return i

    def template(i):
        j = i + 1
        while j < n and src[j] != "`":
            if src[j] == "\\":
                j += 2
            elif src.startswith("${", j):
                j = code(j + 2, True)
            else:
                j += 1
        out.append(src[i + 1:j])
        return j + 1

    code(0, False)
    return [" ".join(s.split()) for s in out]


def tokenizer_selftest():
    """문구 추출 회귀 — 중첩 템플릿 안 문구 · 정규식 속 백틱 · 주석 속 따옴표(어긋나면 한글 literal 집합이 달라진다)."""
    src = ('const a = `${x ? ` · 실행 상세 ${n}` : "없음"} · 끝`; // it\'s\n'
           "const re = /`([^`]+)`/g; const b = '둘째';\n"
           "if (ok) /'/.test(v); const c = '셋째'; const d = (a) / 2 / 4; const e = '넷째';")
    got = {s for s in js_literals(src) if HANGUL.search(s)}
    want = {"· 실행 상세 ${n}", "없음", '${x ? ` · 실행 상세 ${n}` : "없음"} · 끝', "둘째", "셋째", "넷째"}
    assert got == want, got


def html_literals(h):
    h = re.sub(r"<script.*?</script>|<style.*?</style>", "", h, flags=re.S)
    for m in re.finditer(r">([^<>]+)<", h):
        yield " ".join(html.unescape(m.group(1)).split())
    for m in re.finditer(r'(?:title|placeholder|aria-label|data-tip|alt)="([^"]+)"', h):
        yield " ".join(html.unescape(m.group(1)).split())


def literals(name):
    src = (WEB / name).read_text(encoding="utf-8")
    found = html_literals(src) if name.endswith(".html") else js_literals(src)
    return {t for t in found if HANGUL.search(t)}


def size():
    """A1 · A2 — 필독 4문서 합 ≤ 20KB · CLAUDE.md 필독 절이 정확히 STATE · PROJECT · ddd-workflow · PRODUCT ≤ 15KB."""
    n = sum(len(text(p).encode("utf-8")) for p in MUST_READ)
    sec = section(text("CLAUDE.md"), MUST_HEADING)
    named = set(re.findall(r"\]\(([^)\s]+\.md)\)", sec)) | set(re.findall(r"`([^`\s]+\.md)`", sec))
    ok = named == set(MUST_READ[1:])
    p = len(text("PRODUCT.md").encode("utf-8"))
    print(f"files={len(MUST_READ)} bytes={n} under20k={n <= 20480} must_read_ok={ok} product={p} under15k={p <= 15360}")


FACT_RE = [
    re.compile(r"`([\x20-\x5f\x61-\x7e]{2,})`"),                          # 코드 이름 · 경로 · 플래그(ASCII 백틱 한 쌍)
    re.compile(r"(?<![A-Za-z#\d.])(\d+(?:\.\d+)?\s?(?:초|분|시간|일|자|KiB|MB|KB|%|명|건|개|회|번|px))"),  # 단위 붙은 수(D26 · #1k 같은 번호 제외)
]


def moved(sha):
    removed = []
    for rel in SLIMMED:
        now = set(text(rel).splitlines())
        removed += [ln for ln in git_show(sha, rel).splitlines() if ln not in now]
    facts = sorted({m.group(1) for ln in removed for r in FACT_RE for m in r.finditer(ln)})
    live = "\n".join(p.read_bytes().replace(b"\r", b"").decode("utf-8") for p in live_docs())
    missing = [f for f in facts if f not in live]
    print(f"targets={len(facts)} missing={len(missing)}")
    pin("moved", facts)
    if missing:
        print(ascii(missing[:30]))


def code():
    rs = [p for p in (ROOT / "crates").rglob("*.rs")]
    notice = sum(p.read_text(encoding="utf-8").count("notice_for") for p in rs)
    defs = sum(len(re.findall(r"\bfn db_err\(", p.read_text(encoding="utf-8"))) for p in rs if "src" in p.parts)
    serde = re.search(r"(?m)^serde\s*=", text("crates/agent/Cargo.toml")) is not None
    ux = text("crates/app/tests/ux_literals.rs").count("#[test]")
    print(f"notice_for={notice} db_err_defs={defs} agent_serde={serde} ux_tests={ux} end")


RS_TOK = re.compile(r'//[^\n]*|r(?P<h>#*)"(?P<raw>.*?)"(?P=h)|"(?P<s>(?:[^"\\]|\\.)*)"|\'(?:[^\'\\]|\\.)\''
                    r'|(?P<id>[A-Za-z_]\w*)|(?P<num>\d+)|(?P<p>\S)', re.S)
UX_FILES = {"INDEX": "index.html", "APP": "app.js", "VIEW": "view.js", "CSS": "app.css", "TRAY": "tray.rs", "USAGE": "usage"}


def rs_tokens(src):
    """Rust 원본 → (종류, 값) — 주석 · 문자 literal 은 버리고 문자열은 이스케이프를 푼다."""
    esc = {"n": "\n", "t": "\t"}
    unesc = lambda m: "" if m.group(1)[0] == "\n" else chr(int(m.group(1)[2:-1], 16)) if m.group(1)[0] == "u" \
        else esc.get(m.group(1), m.group(1))
    out = []
    for m in RS_TOK.finditer(src):
        if m.group("raw") is not None:
            out.append(("str", m.group("raw")))
        elif m.group("s") is not None:
            out.append(("str", re.sub(r"\\(\n\s*|u\{[0-9a-fA-F]+\}|.)", unesc, m.group("s"), flags=re.S)))
        elif m.lastgroup in ("id", "num", "p"):
            out.append((m.lastgroup, m.group(0)))
    return out


def rs_close(toks, i):
    depth = 0
    for j in range(i, len(toks)):
        depth += (toks[j][0] == "p" and toks[j][1] in "([{") - (toks[j][0] == "p" and toks[j][1] in ")]}")
        if depth == 0:
            return j


def rs_split(toks):
    """최상위 쉼표로 나눈다."""
    items, cur, depth = [], [], 0
    for t in toks:
        depth += (t[0] == "p" and t[1] in "([{") - (t[0] == "p" and t[1] in ")]}")
        if depth == 0 and t == ("p", ","):
            items, cur = items + [cur], []
        else:
            cur.append(t)
    return [c for c in items + [cur] if c]


def ux_resolve(recv, arg, loops, lets):
    """`recv.contains(arg)` → (파일 | 이어 붙인 파일 목록, 문구) 짝 — 반복문 변수는 둘러싼 `for` 배열에서 푼다."""
    if recv in UX_FILES:
        files = [UX_FILES[recv]]
    elif recv in lets:
        files = [lets[recv]]
    elif recv == "text":
        pat, items = next(lp for lp in reversed(loops) if "text" in lp[0])
        parts = [rs_split(it[1:-1]) for it in items]
        if len(pat) == 3:  # (file, text, label | lits)
            return [(UX_FILES[p[1][0][1]], x[1]) for p in parts for x in p[2] if x[0] == "str"]
        files = [UX_FILES[p[1][0][1]] for p in parts]
    else:
        return []  # invite 등 계산 시험
    if arg[0] == "str":
        lits = [arg[1]]
    else:
        lits = next(([x[0][1] for x in items] for pat, items in reversed(loops) if pat == [arg[1]]), [])
    return [(f, lit) for f in files for lit in lits]


def ux_old_pairs(src):
    """옛 시험(직접 단정 · `for` 배열)의 (P 있음 | A 없음 | N 개수, 파일, 문구) 짝."""
    toks, pairs = rs_tokens(src), set()

    def walk(lo, hi, loops, lets):
        i = lo
        while i < hi:
            t = toks[i]
            if t == ("id", "let") and toks[i + 2] == ("p", "=") and toks[i + 3] == ("p", "["):
                close = rs_close(toks, i + 3)
                if toks[close + 2] == ("id", "concat"):
                    lets[toks[i + 1][1]] = [UX_FILES[x[1]] for x in toks[i + 4:close] if x[0] == "id"]
            if t == ("id", "for"):
                k = toks.index(("id", "in"), i)
                if toks[k + 1] == ("p", "["):
                    close = rs_close(toks, k + 1)
                    end = rs_close(toks, close + 1)
                    pat = [x[1] for x in toks[i + 1:k] if x[0] == "id"]
                    walk(close + 2, end, loops + [(pat, rs_split(toks[k + 2:close]))], lets)
                    i = end + 1
                    continue
            if t[0] == "id" and toks[i + 1:i + 4] == [("p", "."), ("id", "contains"), ("p", "(")]:
                kind = "A" if toks[i - 1] == ("p", "!") else "P"
                for f, lit in ux_resolve(t[1], toks[i + 4], loops, lets):
                    if isinstance(f, list):  # [INDEX, APP, VIEW].concat()
                        pairs.update([("P", "web", lit)] if kind == "P" else [("A", x, lit) for x in f])
                    else:
                        pairs.add((kind, f, lit))
            if t[0] == "id" and t[1] in UX_FILES and toks[i + 1:i + 4] == [("p", "."), ("id", "matches"), ("p", "(")] \
                    and toks[i + 4][0] == "str":
                pairs.add(("N", UX_FILES[t[1]], f"{toks[i + 4][1]} x{toks[rs_close(toks, i + 3) + 6][1]}"))
            i += 1

    walk(0, len(toks), [], {})
    return pairs


def ux_new_pairs(src):
    pairs = set()
    for name in ("PRESENT", "ABSENT", "COUNTS"):
        body = re.search(rf"const {name}\b[^=]*= &\[(.*?)\n\];", src, re.S).group(1)
        for row in rs_split(rs_tokens(body)):
            key, lit, *rest = [x[1] for x in row if x[0] in ("str", "num")]
            if name == "PRESENT":
                pairs.add(("P", key, lit))
            elif name == "ABSENT":
                pairs.update(("A", f, lit) for f in key.split("+"))
            else:
                pairs.add(("N", key, f"{lit} x{rest[0]}"))
    return pairs


def ux(sha):
    old = ux_old_pairs(git_show(sha, "crates/app/tests/ux_literals.rs"))
    new = ux_new_pairs(text("crates/app/tests/ux_literals.rs"))
    n = lambda s, k: sum(p[0] == k for p in s)
    print(f"present={n(old, 'P')}/{n(new, 'P')} absent={n(old, 'A')}/{n(new, 'A')} counts={n(old, 'N')}/{n(new, 'N')} "
          f"only_old={len(old - new)} only_new={len(new - old)}")
    if old != new:
        print(ascii(sorted(old ^ new)[:20]))


CONTRAST_PAIRS = [
    ("accent", "accent-soft"), ("agent", "surface"), ("faint", "subtle"), ("muted", "sidebar"), ("muted", "subtle"),
    ("muted", "surface"), ("muted", "surface-2"), ("on-accent", "accent"), ("recv", "recv-bg"), ("run", "subtle"),
    ("sent", "sent-bg"), ("text", "bg"), ("text", "surface"), ("text", "surface-2"), ("text", "sidebar"), ("text", "topbar"),
    ("text", "subtle"), ("muted", "bg"), ("muted", "topbar"), ("faint", "surface"), ("faint", "sidebar"),
    ("faint", "surface-2"), ("accent", "surface"), ("accent", "sidebar"), ("accent", "surface-2"), ("agent", "sidebar"),
    ("run", "surface"), ("run", "sidebar"), ("ok", "surface"), ("ok", "sidebar"), ("danger", "surface"), ("wait", "surface"),
    ("rail-text", "rail"), ("on-dark", "rail"), ("on-dark", "rail-2"), ("on-danger", "danger"), ("on-accent", "accent-hover"),
]
TILES = ["peer-1", "peer-2", "peer-3", "peer-4", "peer-5", "agent-tile"]


def contrast():
    """A11 · E14 — 첫 `:root` 토큰으로 글자/바탕 짝(app.css 규칙 · 본문 · 상태 · 레일) · 타일 흰 글자 · 말풍선 대비가 4.5:1 이상.

    다크 = `--dark-{이름}`(없으면 같은 이름). 말풍선 바탕 = 층 색 × `--bubble-*-alpha` + surface, 글자 = 본문 · muted · 링크(내 글 sent · 상대 글 accent).
    """
    css = re.sub(r"/\*.*?\*/", "", (WEB / "app.css").read_text(encoding="utf-8"), flags=re.S)
    root = {k: v.strip() for k, v in re.findall(r"(--[\w-]+)\s*:\s*([^;]+);", re.search(r":root\s*\{([^{}]*)\}", css).group(1))}

    def rgb(theme, name):
        v = root.get(f"--dark-{name}", root[f"--{name}"]) if theme == "dark" else root[f"--{name}"]
        return [int(v[i:i + 2], 16) for i in (1, 3, 5)]

    def lum(c):
        f = lambda x: x / 255 / 12.92 if x / 255 <= 0.03928 else ((x / 255 + 0.055) / 1.055) ** 2.4  # noqa: E731
        return 0.2126 * f(c[0]) + 0.7152 * f(c[1]) + 0.0722 * f(c[2])

    def ratio(a, b):
        x, y = lum(a), lum(b)
        return (max(x, y) + 0.05) / (min(x, y) + 0.05)

    checks = []
    for th in ("light", "dark"):
        checks += [(f"{th} {fg}/{bg}", rgb(th, fg), rgb(th, bg)) for fg, bg in CONTRAST_PAIRS]
        checks += [(f"{th} on-dark/{t}", rgb(th, "on-dark"), rgb(th, t)) for t in TILES]
        surf = rgb(th, "surface")
        layers = [("mine", rgb(th, "accent"), float(root["--bubble-mine-alpha"]), rgb(th, "sent"))]
        layers += [(p, rgb(th, p), float(root["--bubble-theirs-alpha"]), rgb(th, "accent")) for p in TILES[:5]]
        for who, tone, a, link in layers:
            bg = [t * a + s * (1 - a) for t, s in zip(tone, surf)]
            checks += [(f"{th} bubble {who} {what}", fg, bg) for what, fg in (("text", rgb(th, "text")), ("muted", rgb(th, "muted")), ("link", link))]
    fails = [f"{name} {ratio(fg, bg):.2f}" for name, fg, bg in checks if ratio(fg, bg) < 4.5]
    print(f"pairs={len(checks)} fail={len(fails)}")
    pin("contrast", [name for name, _, _ in checks])
    if fails:
        print(ascii(fails))


def mono():
    css = re.sub(r"/\*.*?\*/", "", (WEB / "app.css").read_text(encoding="utf-8"), flags=re.S)
    css = re.sub(r":root\s*\{[^{}]*\}", "", css, count=1)
    sels = set()
    for m in re.finditer(r"([^{}]+)\{([^{}]*)\}", css):
        if "var(--mono)" in m.group(2):
            sels |= {" ".join(s.split()) for s in m.group(1).split(",")}
    print("mono=" + "|".join(sorted(sels)) + " end")


def cells_of(line):
    """마크다운 표 한 줄의 칸 — `\\|` 는 칸 구분이 아니라 글자 `|`."""
    if not line.startswith("|"):
        return []
    inner = line.strip()[1:-1]
    return [c.strip().replace("\\|", "|") for c in re.split(r"(?<!\\)\|", inner)]


def table_rows(body, heading_re):
    return [c for c in map(cells_of, body.splitlines()) if len(c) >= 6 and re.match(heading_re, c[0])]


def unquote(cell):
    cell = cell.strip()
    return cell[1:-1] if cell.startswith("`") and cell.endswith("`") and len(cell) >= 2 else cell


def copy_rows():
    return table_rows((TASK / "plan-copy.md").read_text(encoding="utf-8"), r"[A-Z]\d+[a-z]?$")


def has(file, cell, cache):
    """문구 하나가 그 파일 화면 문구에 있나 — `raw:` 는 원본 그대로의 부분 문자열(같은 낱말이 다른 자리에도 있을 때)."""
    if cell.startswith("raw:"):
        return cell[4:] in cache.setdefault(("raw", file), (WEB / file).read_text(encoding="utf-8"))
    return cell in cache.setdefault(file, literals(file))


def copy(mode="applied"):
    """plan-copy.md 표 행: | id | 파일 | 현재 | 제안 | 이유 | 승인 | — 승인이 ✅ 인 행만 본다.

    현재는 그 파일 화면 문구에서 사라지고, 제안은 있어야 한다. `(없음)` 은 새로 넣는 문구, `(지움)` 은 빼기만,
    한글이 없는 제안(`${name}` 같은 이름만)은 사라짐만 본다. `before` 모드는 표를 쓸 때 현재 문구가 실재하는지 본다.
    """
    tokenizer_selftest()
    pin_selftest()
    cache, stale, doc_stale = {}, [], []
    rows = copy_rows() if mode == "before" else [r for r in copy_rows() if r[5].startswith("✅")]
    guides = "\n".join(text(p) for p in SHOT_DOCS)   # 사용자에게 보이는 문서도 옛 문구를 쓰지 않는다
    for r in rows:
        before, after = unquote(r[2]), unquote(r[3])
        plain = before not in ("(없음)",) and not before.startswith("raw:") and "${" not in before
        if plain and before in guides:
            doc_stale.append(r[0])
        if mode == "before":
            if before != "(없음)" and not has(r[1], before, cache):
                stale.append(r[0])
            continue
        gone = before == "(없음)" or not has(r[1], before, cache)
        there = after == "(지움)" or not HANGUL.search(after) or has(r[1], after, cache)
        if not (gone and there):
            stale.append(r[0])
    print(f"rows={len(rows)} applied={len(rows) - len(stale)} stale={len(stale)} doc_stale={len(doc_stale)}")
    if mode != "before":
        pin("copy", [f"{r[0]}|{r[1]}|{r[2]}|{r[3]}" for r in rows], [r[0] for r in rows])
    if stale or doc_stale:
        print(ascii(stale[:40]), ascii(doc_stale[:40]))


def terms():
    """docs/manual/dev/web-ui.md `## 용어 표` — | 개념 | 쓰는 말 | 쓰지 않는 말 | ."""
    body = section(text("docs/manual/dev/web-ui.md"), "## 용어 표") if (ROOT / "docs/manual/dev/web-ui.md").exists() else ""
    tokenizer_selftest()
    banned, n, table = [], 0, []
    for ln in body.splitlines():
        cells = cells_of(ln)
        if len(cells) >= 3 and cells[0] not in ("개념", "") and not set(cells[0]) <= set("-: "):
            n += 1
            table.append(cells[:3])
            banned += [unquote(b) for b in re.split(r"\s*·\s*", cells[2]) if unquote(b) not in ("", "없음")]
    lits = literals("index.html") | literals("app.js") | literals("view.js")
    hits = sorted({b for b in banned for t in lits if b in t})
    print(f"terms={n} banned={len(banned)} banned_hits={len(hits)}")
    pin("terms", ["|".join(c) for c in table], [c[0] for c in table])
    if hits:
        print(ascii(hits))


def design():
    """A11 — 고른 시안 `design/tokens.json`: {"tokens": {"--이름": "값"}, "slop": [{"where": "css|html|js|ui", "pattern": 정규식}]}.

    토큰은 `app.css` 첫 `:root` 값과 같아야 하고(공백 · 대소문자 무시), slop 패턴은 css = 첫 `:root` 밖 CSS · html = index.html 원본 ·
    js = app.js + view.js 원본 · ui = 화면 한글 문구에서 0 이어야 한다.
    """
    path = TASK / "design" / "tokens.json"
    spec = json.loads(path.read_text(encoding="utf-8")) if path.exists() else {"tokens": {}, "slop": []}
    css = re.sub(r"/\*.*?\*/", "", (WEB / "app.css").read_text(encoding="utf-8"), flags=re.S)
    m = re.search(r":root\s*\{([^{}]*)\}", css)
    root = dict(re.findall(r"(--[\w-]+)\s*:\s*([^;]+);", m.group(1))) if m else {}
    norm = lambda v: " ".join(v.split()).lower()  # noqa: E731
    mismatch = sorted(k for k, v in spec["tokens"].items() if norm(root.get(k, "")) != norm(v))
    where = {
        "css": css[m.end():] if m else css,
        "html": (WEB / "index.html").read_text(encoding="utf-8"),
        "js": (WEB / "app.js").read_text(encoding="utf-8") + (WEB / "view.js").read_text(encoding="utf-8"),
        "ui": "\n".join(literals("index.html") | literals("app.js") | literals("view.js")),
    }
    hits = [r["pattern"] for r in spec["slop"] if re.search(r["pattern"], where[r["where"]])]
    print(f"tokens={len(spec['tokens'])} mismatch={len(mismatch)} slop_rules={len(spec['slop'])} slop={len(hits)}")
    pin("design", [f"{k}={v}" for k, v in spec["tokens"].items()] + [f"slop:{r['where']}:{r['pattern']}" for r in spec["slop"]])
    if mismatch or hits:
        print(ascii(mismatch[:30]), ascii(hits))


def extra(sha):
    t12 = (ROOT / "target-t12").exists()
    wt = (ROOT / ".claude" / "worktrees" / "agent-abd1086a96c70114e").exists()
    state = text("STATE.md")
    stale_state = sum(m in state for m in STATE_STALE)
    refs = set()
    for rel in SHOT_DOCS:
        refs |= set(re.findall(r"images/([\w.-]+\.png)", text(rel)))
    changed = subprocess.run(["git", "diff", "--name-only", sha, "--", "docs/images"], cwd=ROOT,
                             capture_output=True, text=True).stdout.split()
    changed = {Path(c).name for c in changed}
    stale_shots = sorted(refs - changed)
    every = "\n".join(p.read_text(encoding="utf-8") for p in [ROOT / "README.md", *(ROOT / "docs").rglob("*.md")])
    orphans = sorted(p.name for p in (ROOT / "docs" / "images").glob("*.png") if p.name not in every)
    print(f"t12={t12} wt={wt} state_stale={stale_state} shots={len(refs)} stale_shots={len(stale_shots)} orphans={len(orphans)}")
    pin("shots", sorted(refs))
    if stale_shots or orphans:
        print(ascii(stale_shots + orphans))


if __name__ == "__main__":
    cmd, *args = sys.argv[1:] or ["size"]
    {"size": size, "moved": moved, "code": code, "ux": ux, "mono": mono, "copy": copy, "terms": terms, "design": design,
     "contrast": contrast, "extra": extra}[cmd](*args)

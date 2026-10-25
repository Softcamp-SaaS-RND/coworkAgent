"""1z-13 AI 티 검사 — 웹UI 원본(app.css · index.html · js)에서 taste 목록과 결정 5 · 12 를 센다. 위반이 있으면 stderr 에 이름 · 개수."""
import colorsys
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[4]  # docs/{tasks|archive}/{key}/design/taste.py
WEB = ROOT / "crates/app/web"
FILES = [WEB / "app.css", WEB / "index.html", *sorted((WEB / "js").rglob("*.js"))]
TEXT = {f: f.read_text(encoding="utf-8") for f in FILES}
ALL = "\n".join(TEXT.values())
CSS = TEXT[WEB / "app.css"]
ROOT_BLOCK = CSS[CSS.index(":root {") : CSS.index("\n}", CSS.index(":root {"))]


def sat(name):
    m = re.search(rf"--{name}: #([0-9A-Fa-f]{{6}});", ROOT_BLOCK)
    if not m:
        return 1.0
    r, g, b = (int(m.group(1)[i : i + 2], 16) / 255 for i in (0, 2, 4))
    return colorsys.rgb_to_hls(r, g, b)[2]


SLOP = (
    r"border-(?:left|right|top)(?:-color)?:\s*(?:\d+px solid\s+)?var\(--(?:tone|accent|run|ok|danger|wait|muted)",  # 옆 · 위 색 띠(상태 색 포함 — 개정 ⑨)
    r"inset \d+px 0 0",  # 안쪽 띠
    r"letter-spacing:\s*\.\d+em",  # 절 제목 자간
    r">[✎⋯]</button>",  # 글자 기호 단추
    r'text: "[✎⋯]"',
)
CHECKS = {
    "gradient": ALL.count("gradient"),
    "black": len(re.findall(r"#000(?:000)?\b", ALL)),
    "emoji": len(re.findall("[\U0001F000-\U0001FAFF]", ALL)),
    # 사람 · 그룹 모양 그림(대화 종류 칩의 사람 아이콘은 아바타가 아니라 문의 = 사람 표시 — 결정 16, 개정 ⑦ → ⑨)
    "person-icon-avatar": len(re.findall(r'PERSON_SVG|GROUP_SVG|M4 21c0-4\.4 3\.6-7 8-7s8 2\.6 8 7z', ALL)),
    "old-blue": len(re.findall(r"#(?:245BAB|104A97|E8F1FE|7FACF0|9AC0F8|1A273A|AACDFF)\b", ALL, re.I)),
    "accent-saturation": sum(sat(n) >= 0.45 for n in ("accent", "dark-accent")),
    "primary-not-neutral": 0 if re.search(r"button\.primary \{[^}]*background: var\(--primary\)", CSS) else 1,
    "slop": sum(len(re.findall(p, ALL)) for p in SLOP),
    "selected-row-stripe": len(re.findall(r"\.current::before", CSS)),
}
for k, v in CHECKS.items():
    if v:
        print(f"violation {k} {v}", file=sys.stderr)
print(f"taste files={len(FILES)} checks={len(CHECKS)} violations={sum(CHECKS.values())}")

"""README 검사(#1c E10) — 사용자 관점 문서인가.

- 상대 링크(`](경로)` · `src="경로"`)가 모두 저장소에 있다.
- 기술 절(통신 메커니즘 · 피어 프로토콜 · 빌드)이 제목으로 없다 — 개발자 가이드로 옮겼다.
- 목적 4가지 문구가 있다.

통과하면 `readme ok`, 아니면 문제 목록과 종료 코드 1.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FORBIDDEN = ["통신 메커니즘", "피어 프로토콜", "빌드"]
PURPOSES = ["웹 서버 없음", "AI API 비용 없음", "페어링으로 코워크", "대화가 서버에 쌓이지 않음"]


def problems(text: str) -> list[str]:
    out = []
    links = re.findall(r"\]\(([^)\s]+)\)", text) + re.findall(r'src="([^"]+)"', text)
    for link in links:
        if re.match(r"[a-z]+:", link) or link.startswith("#"):
            continue
        if not (ROOT / link.split("#")[0]).exists():
            out.append(f"없는 링크: {link}")
    for line in text.splitlines():
        if line.startswith("#") and any(w in line for w in FORBIDDEN):
            out.append(f"기술 절: {line}")
    out += [f"목적 문구 없음: {p}" for p in PURPOSES if p not in text]
    return out


def main() -> int:
    found = problems((ROOT / "README.md").read_text(encoding="utf-8"))
    for p in found:
        print(p)
    if found:
        return 1
    print("readme ok")
    return 0


if __name__ == "__main__":
    assert problems("## 빌드 · 실행\n[x](nope.md)") == ["없는 링크: nope.md", "기술 절: ## 빌드 · 실행"] + [
        f"목적 문구 없음: {p}" for p in PURPOSES
    ]
    sys.exit(main())

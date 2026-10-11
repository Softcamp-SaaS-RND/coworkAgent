#!/usr/bin/env python
"""워크스페이스 크레이트 의존 경계 검사 — 루트 PROJECT.md 검증 명령 "의존 경계" (check_layers.py 의 Rust 대체).

`cargo metadata --no-deps` 로 워크스페이스 크레이트와 내부(path) 간선을 실측해
① Layer 역방향(하위가 상위를 의존) ② LAYER 에 없는 미분류 크레이트를 위반으로 센다.
마지막 줄 `crates=N edges=M violations=V` — 대상 수와 위반 수를 함께 단정한다(대상이 사라지면 N 이 드러난다).

    python tools/check_crates.py

새 크레이트를 만들면 LAYER 에 등록하고 루트 PRODUCT.md 모듈 Layer 표와 맞춘다. 종료 코드: 위반 0 이면 0.
"""

import json
import subprocess
import sys

ORDER = ("base", "comm", "domain", "app")  # 의존은 app → domain → comm → base 단방향
LAYER = {
    "an-core": "base",
    "an-net": "comm",
    "an-agent": "domain",
    "coworkagent-app": "app",
}


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    meta = json.loads(subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        capture_output=True, text=True, encoding="utf-8", check=True,
    ).stdout)
    members = set(meta["workspace_members"])
    pkgs = [p for p in meta["packages"] if p["id"] in members]
    names = {p["name"] for p in pkgs}
    edges = sorted({(p["name"], d["name"]) for p in pkgs for d in p["dependencies"]
                    if d.get("path") and d["name"] in names})
    problems = [f"미분류 크레이트: {p['name']}" for p in pkgs if p["name"] not in LAYER]
    problems += [f"역방향 의존: {a} ({LAYER[a]}) -> {b} ({LAYER[b]})" for a, b in edges
                 if a in LAYER and b in LAYER and ORDER.index(LAYER[a]) < ORDER.index(LAYER[b])]
    for a, b in edges:
        print(f"edge {a} -> {b}")
    for p in problems:
        print(f"VIOLATION {p}")
    print(f"crates={len(pkgs)} edges={len(edges)} violations={len(problems)}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())

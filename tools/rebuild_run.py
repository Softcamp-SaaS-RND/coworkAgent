#!/usr/bin/env python
"""Windows 개발 PC 의 앱을 새 빌드로 바꿔 띄운다 — 떠 있는 coworkAgent 종료 → 릴리스 빌드 → 실행.

    python tools/rebuild_run.py

떠 있는 coworkAgent(트레이 · CLI)는 모두 강제 종료한다 — 그때 실행 중이던 요청은 트레이 "종료" 때처럼 다음 시작 때
실패 답("다시 보내 주세요")이 된다. 빌드가 실패하면 앱을 띄우지 않는다(꺼진 채로 둔다).
새 앱은 탐색기로 띄운다 — 더블클릭과 같은 환경이라 이 스크립트를 부른 셸(예: Claude Code 세션)의 환경 변수가 앱과 그
아래 `claude -p` 로 새지 않는다. 새 앱이 `instance.json` 을 쓰면(시작 끝) 성공이다.

종료 코드: 0 성공 · 1 기존 앱 종료 실패 또는 시작 확인 실패 · 그 밖은 빌드 실패(cargo 의 종료 코드).
"""

import os
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EXE = ROOT / "target" / "release" / "coworkAgent.exe"
DATA = Path(os.environ.get("LOCALAPPDATA", "")) / "coworkAgent"  # 데이터 루트(D15)


def running() -> bool:
    # 바이트로 본다 — 일치 없음 안내문은 OS 언어 코드페이지(예: cp949)라 UTF-8 모드 파이썬에서 디코드가 깨진다.
    out = subprocess.run(["tasklist", "/FI", "IMAGENAME eq coworkAgent.exe", "/NH"], capture_output=True).stdout
    return b"coworkAgent.exe" in out


def stop() -> bool:
    """떠 있는 coworkAgent 를 모두 끝낸다. 다 끝났으면 참."""
    subprocess.run(["taskkill", "/F", "/IM", "coworkAgent.exe"], capture_output=True)
    for _ in range(50):  # 실행 파일 잠금이 풀릴 때까지 최대 5초
        if not running():
            return True
        time.sleep(0.1)
    return False


def start() -> int:
    began = time.time()
    subprocess.run(["explorer.exe", str(EXE)])  # 성공해도 종료 코드 1 — 보지 않는다
    info = DATA / "instance.json"
    for _ in range(100):  # 시작 끝(새 instance.json)을 최대 10초 — 강제 종료한 앱이 남긴 옛 파일은 시각으로 거른다
        if info.exists() and info.stat().st_mtime >= began - 1:
            print('실행 — 트레이 아이콘 메뉴 "열기" 로 웹UI')
            return 0
        time.sleep(0.1)
    log = DATA / "logs" / "coworkAgent.log"
    print(f'시작 확인 실패 — 붉은 트레이 아이콘이면 메뉴 "오류 보기", 아니면 {log} 확인', file=sys.stderr)
    return 1


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
    if sys.platform != "win32":
        print("Windows 전용", file=sys.stderr)
        return 1
    was = running()
    if not stop():
        print("coworkAgent 를 끝내지 못했다 — 작업 관리자에서 확인", file=sys.stderr)
        return 1
    print("기존 coworkAgent 종료" if was else "떠 있는 coworkAgent 없음", flush=True)  # cargo 출력보다 먼저
    build = subprocess.run(["cargo", "build", "--release", "-p", "coworkagent-app"], cwd=ROOT)
    if build.returncode != 0:
        print("빌드 실패 — 앱은 꺼진 채로 둔다", file=sys.stderr)
        return build.returncode
    return start()


if __name__ == "__main__":
    sys.exit(main())

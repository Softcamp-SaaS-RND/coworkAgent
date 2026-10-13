#!/usr/bin/env python
"""포트 풀림 실측(1b A6 · A8) — Windows 개발 PC 의 릴리스 앱으로 잰다.

    python tools/port_check.py            # 강제 종료 → 5초 안에 웹UI · 피어 포트 풀림 → 남은 instance.json 인 채로 다시 띄워 47100
    python tools/port_check.py --occupy   # 47100 을 이 스크립트가 먼저 잡고 띄워 /api/me 의 잡은 프로세스 · 임의 포트를 읽는다

떠 있는 coworkAgent 는 먼저 강제 종료한다(`rebuild_run.py` 와 같음 — 실행 중이던 요청은 다음 시작 때 실패 답). 끝나면 앱을 정상으로
다시 띄워 둔다. 앱은 탐색기로 띄운다 — 이 셸의 환경 변수가 앱 · `claude -p` 로 새지 않게.

출력 한 줄: 기본 `freed=0|1 relisten=0|1 instance_left=0|1` · `--occupy` `fixed=0|1 holder=<이름>`. 종료 코드 0 = 기대값 · 1 = 어긋남.
"""

import json
import os
import socket
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EXE = ROOT / "target" / "release" / "coworkAgent.exe"
DATA = Path(os.environ.get("LOCALAPPDATA", "")) / "coworkAgent"  # 데이터 루트(D15)
INFO = DATA / "instance.json"
PEER_PORT = 47100


def pids() -> list[int]:
    out = subprocess.run(
        ["tasklist", "/FI", "IMAGENAME eq coworkAgent.exe", "/FO", "CSV", "/NH"], capture_output=True
    ).stdout.decode("utf-8", "replace")
    return [int(l.split('","')[1]) for l in out.splitlines() if l.startswith('"coworkAgent.exe"')]


def listening(port: int) -> int | None:
    """TCP `port` 를 듣는 PID — 없으면 None. 상태 글자는 언어마다 달라 원격 주소 :0 으로 듣기를 가린다."""
    out = subprocess.run(["netstat", "-ano", "-p", "TCP"], capture_output=True).stdout.decode("utf-8", "replace")
    if " TCP " not in out:  # 표를 못 읽었으면 "풀림" 으로 보지 않는다(거짓 통과 방지)
        raise SystemExit("netstat 출력을 읽지 못했다")
    for line in out.splitlines():
        f = line.split()
        if len(f) >= 5 and f[0] == "TCP" and f[1].rsplit(":", 1)[-1] == str(port) and f[2].endswith(":0"):
            return int(f[-1])
    return None


def stop_all() -> bool:
    subprocess.run(["taskkill", "/F", "/IM", "coworkAgent.exe"], capture_output=True)
    return wait(lambda: not pids(), 5)


def wait(cond, secs: float) -> bool:
    until = time.time() + secs
    while time.time() < until:
        if cond():
            return True
        time.sleep(0.1)
    return cond()


def token() -> str | None:
    try:
        return json.loads(INFO.read_text(encoding="utf-8"))["token"]
    except (OSError, ValueError, KeyError):
        return None


def start() -> tuple[int, dict]:
    """앱을 띄우고 (PID, instance.json) — 토큰이 바뀐 새 instance.json 을 최대 15초(남은 옛 파일은 그대로 둔 채 띄운다)."""
    old = token()
    subprocess.run(["explorer.exe", str(EXE)])  # 성공해도 종료 코드 1 — 보지 않는다
    if not wait(lambda: token() not in (None, old) and pids(), 15):
        raise SystemExit("시작 확인 실패 — instance.json 이 새로 쓰이지 않았다")
    return pids()[0], json.loads(INFO.read_text(encoding="utf-8"))


def me(info: dict) -> dict:
    req = urllib.request.Request(
        f"http://127.0.0.1:{info['port']}/api/me", headers={"Authorization": f"Bearer {info['token']}"}
    )
    with urllib.request.urlopen(req, timeout=5) as r:
        return json.loads(r.read())


def release() -> int:
    pid, info = start()
    ui = info["port"]
    if listening(PEER_PORT) != pid or listening(ui) != pid:
        raise SystemExit(f"시작한 앱(PID {pid})이 웹UI {ui} · 피어 {PEER_PORT} 를 듣지 않는다")
    subprocess.run(["taskkill", "/F", "/PID", str(pid)], capture_output=True)
    freed = wait(lambda: listening(ui) is None and listening(PEER_PORT) is None, 5)
    left = INFO.exists()  # 강제 종료는 instance.json 을 지우지 못한다
    pid2, _ = start()
    relisten = wait(lambda: listening(PEER_PORT) == pid2, 5)
    print(f"freed={int(freed)} relisten={int(relisten)} instance_left={int(left)}")
    return 0 if (freed, relisten, left) == (True, True, True) else 1


def occupy() -> int:
    s = socket.socket()
    s.bind(("0.0.0.0", PEER_PORT))
    s.listen()
    try:
        _, info = start()
        m = me(info)
    finally:
        s.close()
    holder = (m.get("peer_port_holder") or {}).get("name", "")
    name = holder.lower().removesuffix(".exe") or "없음"
    print(f"fixed={int(bool(m.get('peer_port_fixed')))} holder={name}")
    # 정상으로 다시 띄운다.
    stop_all()
    _, info = start()
    back = me(info).get("peer_port_fixed") is True
    if not back:
        print("다시 띄운 앱이 47100 을 열지 못했다", file=sys.stderr)
    return 0 if (not m.get("peer_port_fixed") and name == "python" and back) else 1


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
    if sys.platform != "win32":
        print("Windows 전용", file=sys.stderr)
        return 1
    if not EXE.exists():
        print(f"릴리스 빌드가 없다 — cargo build --release -p coworkagent-app ({EXE})", file=sys.stderr)
        return 1
    if not stop_all():
        print("떠 있는 coworkAgent 를 끝내지 못했다", file=sys.stderr)
        return 1
    try:
        return occupy() if "--occupy" in sys.argv[1:] else release()
    finally:
        # 중간에 실패해도 앱을 정상으로 띄워 둔다(47100 을 연 앱 하나).
        if listening(PEER_PORT) not in pids() or len(pids()) != 1:
            stop_all()
            start()


if __name__ == "__main__":
    sys.exit(main())

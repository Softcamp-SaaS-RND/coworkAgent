#!/usr/bin/env python
"""개발 PC(Windows · macOS)의 앱을 새 빌드로 바꿔 띄운다 — 떠 있는 coworkAgent 종료 → 릴리스 빌드 → 실행.

    python tools/rebuild_run.py          # 빠진 빌드 도구가 있으면 설치할지 묻는다
    python tools/rebuild_run.py --yes    # 묻지 않고 설치(터미널이 아닌 곳 — 예: Claude Code 세션 — 에서는 이것만 설치한다)

빌드 도구(`ensure_toolchain`)를 먼저 맞춘다 — Rust stable(rustup · `MIN_RUST` 이상) · C 컴파일러 · 링커(`ring` · rusqlite `bundled` 가
쓴다 — Windows MSVC Build Tools(winget), macOS Xcode Command Line Tools). crate 는 cargo 가 받는다. 도구가 없으면 떠 있는 앱을 끄지 않는다.
떠 있는 coworkAgent(트레이 · CLI)는 모두 강제 종료한다 — 그때 실행 중이던 요청은 트레이 "종료" 때처럼 다음 시작 때
실패 답("다시 보내 주세요")이 된다. 빌드가 실패하면 앱을 띄우지 않는다(꺼진 채로 둔다).
새 앱은 이 스크립트를 부른 셸(예: Claude Code 세션)의 환경 변수가 앱과 그 아래 `claude -p` 로 새지 않게 띄운다 — Windows 는
탐색기(더블클릭과 같은 환경), macOS 는 로그인 기본 변수만 남긴 환경(`KEEP_ENV`)의 새 세션. 새 앱이 `instance.json` 을 쓰면
(시작 끝) 성공이다. macOS 는 이 PC 의 아키텍처 하나만 빌드한다(universal `lipo` 는 배포 절차 — Phase 4).

종료 코드: 0 성공 · 1 빌드 도구 없음 · 기존 앱 종료 실패 또는 시작 확인 실패 · 그 밖은 빌드 실패(cargo 의 종료 코드).
"""

from __future__ import annotations

import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WINDOWS = sys.platform == "win32"
NAME = "coworkAgent.exe" if WINDOWS else "coworkAgent"
EXE = ROOT / "target" / "release" / NAME
# 데이터 루트(D15) — an_core `Paths::default_root` 와 같은 자리
if WINDOWS:
    DATA = Path(os.environ.get("LOCALAPPDATA", "")) / "coworkAgent"
else:
    DATA = Path.home() / "Library" / "Application Support" / "coworkAgent"
# macOS 에서 새 앱에 물려줄 변수 — 로그인 때 launchd 가 주는 것과 같은 종류만. 나머지(CLAUDE_* · COWORKAGENT_* 등)는 버린다.
KEEP_ENV = ("HOME", "USER", "LOGNAME", "SHELL", "TMPDIR", "PATH", "LANG", "LC_ALL", "LC_CTYPE", "__CF_USER_TEXT_ENCODING")
MIN_RUST = (1, 85, 0)  # edition 2024
CARGO_BIN = Path.home() / ".cargo" / "bin"  # rustup 기본 설치 자리
RUSTUP_INIT = "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
RUSTUP_SH = "https://sh.rustup.rs"
VC_COMPONENT = "Microsoft.VisualStudio.Component.VC.Tools.x86.x64"


def ask(what: str, yes: bool) -> bool:
    """설치해도 되는가 — `--yes` 면 참, 터미널이면 묻고, 아니면 거짓."""
    if yes:
        return True
    if not sys.stdin.isatty():
        print(f"{what} 없음 — 설치하려면 --yes 로 다시", file=sys.stderr)
        return False
    return input(f"{what} 없음 — 설치할까? [y/N] ").strip().lower() in ("y", "yes")


def download(url: str, name: str) -> Path:
    dest = Path(tempfile.mkdtemp()) / name
    print(f"내려받기 {url}", flush=True)
    if shutil.which("curl"):  # macOS python.org 빌드의 urllib 는 인증서 묶음이 없을 수 있다 — OS 의 curl 이 먼저(Windows 10+ 도 기본 포함)
        subprocess.run(["curl", "--proto", "=https", "--tlsv1.2", "-sSfL", "-o", str(dest), url], check=True)
    else:
        urllib.request.urlretrieve(url, dest)
    return dest


def has_c_tools() -> bool:
    if WINDOWS:
        base = Path(os.environ.get("ProgramFiles(x86)", r"C:\Program Files (x86)"))
        vswhere = base / "Microsoft Visual Studio" / "Installer" / "vswhere.exe"
        if not vswhere.is_file():
            return False
        out = subprocess.run(
            [str(vswhere), "-products", "*", "-requires", VC_COMPONENT, "-property", "installationPath"],
            capture_output=True,
            text=True,
        ).stdout
        return bool(out.strip())
    return subprocess.run(["xcrun", "--find", "clang"], capture_output=True).returncode == 0


def install_c_tools() -> bool:
    if WINDOWS:
        if shutil.which("winget") is None:
            print("winget 없음 — Visual Studio Build Tools(C++ 데스크톱 개발)를 직접 설치", file=sys.stderr)
            return False
        # 관리자 승인(UAC) 창이 뜬다. 끝난 결과는 종료 코드 대신 다시 확인한다(이미 있음도 0 이 아니다).
        subprocess.run([
            "winget", "install", "--id", "Microsoft.VisualStudio.2022.BuildTools", "-e",
            "--accept-package-agreements", "--accept-source-agreements",
            "--override", f"--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --add {VC_COMPONENT} --includeRecommended",
        ])
        return has_c_tools()
    # 설치 창은 따로 돈다 — 끝날 때까지 기다린다(최대 1시간).
    subprocess.run(["xcode-select", "--install"], capture_output=True)
    print("Xcode Command Line Tools 설치 창에서 \"설치\" — 끝날 때까지 기다린다", flush=True)
    for _ in range(3600):
        if has_c_tools():
            return True
        time.sleep(1)
    return False


def rust_version() -> tuple[int, ...] | None:
    if shutil.which("rustc") is None:
        return None
    out = subprocess.run(["rustc", "--version"], capture_output=True, text=True).stdout  # rustc 1.95.0 (… )
    try:
        return tuple(int(x) for x in out.split()[1].split("-")[0].split("."))
    except (IndexError, ValueError):
        return None


def install_rust() -> bool:
    args = ["-y", "--default-toolchain", "stable", "--profile", "default"]  # default = clippy · rustfmt 포함(검증 명령)
    if WINDOWS:
        init = download(RUSTUP_INIT, "rustup-init.exe")
        cmd = [str(init), *args, "--default-host", "x86_64-pc-windows-msvc"]
    else:
        init = download(RUSTUP_SH, "rustup-init.sh")
        cmd = ["sh", str(init), *args]
    return subprocess.run(cmd).returncode == 0


def ensure_toolchain(yes: bool) -> bool:
    """빌드 도구가 다 있으면 참 — 없는 것은 허락받고 설치한다."""
    if CARGO_BIN.is_dir() and str(CARGO_BIN) not in os.environ.get("PATH", "").split(os.pathsep):
        os.environ["PATH"] = str(CARGO_BIN) + os.pathsep + os.environ.get("PATH", "")  # 설치 직후 · 새 셸 전에도 찾게
    c_tools = "MSVC Build Tools" if WINDOWS else "Xcode Command Line Tools"
    if not has_c_tools() and not (ask(c_tools, yes) and install_c_tools()):
        print(f"{c_tools} 없음 — 빌드할 수 없다", file=sys.stderr)
        return False
    version = rust_version()
    if version is None or shutil.which("cargo") is None:
        if not (ask("Rust(rustup)", yes) and install_rust()):
            print("Rust 없음 — 빌드할 수 없다", file=sys.stderr)
            return False
        os.environ["PATH"] = str(CARGO_BIN) + os.pathsep + os.environ.get("PATH", "")
        version = rust_version()
    elif version < MIN_RUST:
        need = ".".join(map(str, MIN_RUST))
        if shutil.which("rustup") is None or not ask(f"Rust {need} 이상", yes):
            print(f"Rust {need} 이상 필요 — 지금 {'.'.join(map(str, version))}", file=sys.stderr)
            return False
        subprocess.run(["rustup", "update", "stable"])
        version = rust_version()
    if version is None or version < MIN_RUST or shutil.which("cargo") is None:
        print("Rust 설치 확인 실패 — 새 터미널에서 rustc --version 확인", file=sys.stderr)
        return False
    return True


def pids() -> list[int]:
    """떠 있는 coworkAgent 의 PID(macOS)."""
    out = subprocess.run(["pgrep", "-x", NAME], capture_output=True, text=True).stdout
    return [int(p) for p in out.split()]


def running() -> bool:
    if WINDOWS:
        # 바이트로 본다 — 일치 없음 안내문은 OS 언어 코드페이지(예: cp949)라 UTF-8 모드 파이썬에서 디코드가 깨진다.
        out = subprocess.run(["tasklist", "/FI", f"IMAGENAME eq {NAME}", "/NH"], capture_output=True).stdout
        return NAME.encode() in out
    return bool(pids())


def kill_mac() -> None:
    # 앱이 띄운 `claude -p` 는 자기 프로세스 그룹(process_group(0))이라 앱과 함께 죽지 않는다 — Windows 의 kill-on-close Job 처럼
    # 앱의 자식 그룹을 먼저 묶음째 끝낸다.
    apps = pids()
    for pid in apps:
        out = subprocess.run(["pgrep", "-P", str(pid)], capture_output=True, text=True).stdout
        for child in (int(c) for c in out.split()):
            if child in apps:  # 감시 프로세스의 자식 = 주 인스턴스 — 아래에서 끝낸다
                continue
            try:
                os.killpg(child, signal.SIGKILL)
            except (ProcessLookupError, PermissionError):
                pass
    for pid in apps:
        try:
            os.kill(pid, signal.SIGKILL)
        except ProcessLookupError:
            pass


def stop() -> bool:
    """떠 있는 coworkAgent 를 모두 끝낸다. 다 끝났으면 참."""
    if WINDOWS:
        subprocess.run(["taskkill", "/F", "/IM", NAME], capture_output=True)
    else:
        kill_mac()
    for _ in range(50):  # 실행 파일 잠금이 풀릴 때까지 최대 5초
        if not running():
            return True
        time.sleep(0.1)
    return False


def launch() -> None:
    if WINDOWS:
        subprocess.run(["explorer.exe", str(EXE)])  # 성공해도 종료 코드 1 — 보지 않는다
        return
    env = {k: os.environ[k] for k in KEEP_ENV if k in os.environ}
    subprocess.Popen(
        [str(EXE)],
        env=env,
        cwd=Path.home(),
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,  # 부른 셸의 Ctrl+C · 종료가 닿지 않게
    )


def start() -> int:
    began = time.time()
    launch()
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
    if not WINDOWS and sys.platform != "darwin":
        print("Windows · macOS 전용", file=sys.stderr)
        return 1
    if not ensure_toolchain("--yes" in sys.argv[1:]):
        return 1
    was = running()
    if not stop():
        where = "작업 관리자" if WINDOWS else "활성 상태 보기"
        print(f"coworkAgent 를 끝내지 못했다 — {where}에서 확인", file=sys.stderr)
        return 1
    print("기존 coworkAgent 종료" if was else "떠 있는 coworkAgent 없음", flush=True)  # cargo 출력보다 먼저
    build = subprocess.run(["cargo", "build", "--release", "-p", "coworkagent-app"], cwd=ROOT)
    if build.returncode != 0:
        print("빌드 실패 — 앱은 꺼진 채로 둔다", file=sys.stderr)
        return build.returncode
    return start()


if __name__ == "__main__":
    sys.exit(main())

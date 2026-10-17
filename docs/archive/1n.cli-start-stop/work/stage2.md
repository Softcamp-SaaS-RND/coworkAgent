# stage2 — 문서 · 이 PC 실제 확인

> 근거: [plan](../plan.md) Stage 2. plan 개정 4건 승인 2026-10-17(사용자 "승인. 개발 진행.").

## 목표

`start` · `open` · `stop` 을 쓰는 사람 · 개발자 문서에 싣고, 이 PC 에서 디버그 실행 파일 + 임시 데이터 경로로 실제 동작을 본다. 대상 gate: E4 ~ E6 · E8.

## 산출 파일

- [user-guide](../../../manual/user/user-guide.md) — 3장(트레이 없는 모드 한 줄 · 두 번째 실행 실패 = 사유 파일 열기), **14.1** 다른 도구의 메뉴에서 켜고 끄기(예시 ·
  트레이 없는 모드 · 부모 · 출력 · 종료 코드 0 · 2 · 4 · 6 · 8), 16장 두 줄(트레이 아이콘 없음 · 종료 코드 6).
- [developer-guide](../../../manual/dev/developer-guide.md) — 3.1 감시 · 주 인스턴스 · CLI 줄, 4장 표(`/api/quit` · `supervisor.lock` · `instance.pid`), 4.1 시작 · 종료 순서,
  7.1 데이터 폴더(`instance.pid` · `supervisor.pid` · `supervisor.lock` · 디버그 `COWORKAGENT_DATA_DIR`), **8.10** 부모 도구 메뉴(순서 그림 · 떼어 띄우기 ·
  `start` · `open` · `stop` · `/api/quit` · 시험), 9장 CLI 경계에 `/api/quit` · 강제 종료.
- [an_app](../../../manual/dev/an_app.md) — `router_control` · `control` · `proc` · `tray`(`start_error_file` · `read_start_error` · `quitter` · `Attention::none`) ·
  `watchdog`(`io::Result<i32>` · `StartFailed` · `PARENT_ENV` · `parent` · `acquire`) · `EXIT_START_FAILED`.
- [an_core](../../../manual/dev/an_core.md) — 경로 3개 · `DATA_DIR_ENV`("환경 변수 스위치 없음" 문구 고침) · `is_held` · `write_pid` · `read_pid` · `acquire` 다시 잡기.
- `PRODUCT.md` — 실행 구조(감시 잠금 · 코드 3) · 시작 실패 · **부모 도구 메뉴** 줄. `README.md` — 기능 표 CLI 줄.

## 이 PC 실제 확인(E8)

디버그 실행 파일(`target\debug\coworkAgent.exe`) + 임시 `COWORKAGENT_DATA_DIR`(`%TEMP%\cowork-e8-*`), 스크립트로 차례대로. 프로세스 수 = 그 실행 파일 경로의
`Win32_Process`(감시 + 앱).

```text
real instance procs (not debug): 0
1) 인자 없는 실행 = 트레이 모드
  tray mode: True  procs=2
2) start -> 트레이 없는 모드 (부모 = 이 셸)
  > coworkAgent start --parent 18988 -> rc=0 out=[시작함]
  mode=headless procs=2
3) open -> 브라우저로 화면
  > coworkAgent open --parent 18988 -> rc=0 out=[열었음]
4) 인자 없는 두 번째 실행 = 화면 열기(핸드오프)
  second run exit=0 mode=headless procs=2
5) stop
  > coworkAgent stop -> rc=0 out=[종료함]
  procs after stop=0  instance.json exists=False
6) 부모(셸) 닫으면 끝 — 별도 powershell 이 start 후 6초 뒤 종료
  headless up: True  procs=2  shell alive=True
  shell exited=True  procs after parent exit=0
7) 꺼진 상태 stop
  > coworkAgent stop -> rc=0 out=[실행 중이 아님]
real data dir unchanged: True  (files=16)
real instance.json exists: False
```

- **A1 · A2** — 트레이 모드에서 `start` → `headless` 로 바뀜, 프로세스 2개 그대로(감시 + 앱). **A4** — `open` 204("열었음"). **A8** — 인자 없는 두 번째 실행
  0 · 모드 · 프로세스 그대로. **A3** — 부모 셸이 끝나자 감시 · 앱 모두 끝(0개). **A5** — `stop` 뒤 0개 · `instance.json` 지워짐.
- **실제 인스턴스** — 이번 확인 동안 실제(릴리스) 인스턴스는 떠 있지 않았다. 실제 데이터 폴더 파일 16개의 이름 · 크기 · 수정 시각이 전후 같다.
- **트레이 아이콘** — 작업 표시줄 캡처는 두 장(트레이 · 트레이 없음)이 같았다(Windows 11 은 새 아이콘을 숨은 아이콘(^) 안에 둔다). 그래서 UI Automation 으로
  숨은 아이콘을 펼쳐 이름을 찾았다: 실행 전 없음 → 인자 없는 실행 뒤 `SystemTray.NormalButton:coworkAgent` → `start` 뒤 없음.
- **브라우저** — `POST /api/open` 204(`ShellExecuteW` 성공) · Edge 에 `coworkAgent` 탭이 있다(이전 탭과 섞여 이번 열기만 가려내지는 못함).

## 테스트 결과

```text
python tools/check_readme.py   → readme ok
python tools/check_ddd_docs.py → PASS (INFO — 미해소 TODO: docs/masterPlan.md 1건 — D14, 종결 때 보강)
```

코드는 stage 1 리뷰 커밋(`861e8b2`) 뒤 바뀌지 않았다 — Rust 시험 · clippy · `check_crates` 는 stage 1 결과 그대로(eval 에서 `check_gates` 로 다시 잰다).

## 다음 단계

- eval — `check_gates`(E1 ~ E7) · E8 근거 = 이 문서 · 적대적 리뷰(라운드 ≤ 2).

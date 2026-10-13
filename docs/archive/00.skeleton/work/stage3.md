# stage3 — 트레이 · 단일 인스턴스 배선 · Windows 릴리스

## 목표

[plan](../plan.md) Stage 3 — 실행 파일 `agentNetwork`(Cargo bin), tao 이벤트 루프 + tray-icon 메뉴 "열기" · "종료",
단일 인스턴스 배선(잠금 → 서버 → `instance.json` / 두 번째 실행은 핸드오프), 파일 로거, Windows 릴리스 빌드.
대상 gate: E1 · E2 · E3 · E10(수동).

## 산출 파일

- `crates/app/Cargo.toml` — 실행 파일 타깃 `agentNetwork` + tao 0.37.1 · tray-icon 0.26.1
- `crates/app/src/main.rs` — 시작 순서(경로 → 로거 → 잠금 → DB → 서버 → `instance.json` → 트레이), 핸드오프 재시도
- `crates/app/src/tray.rs` — 이벤트 루프 · 메뉴 · 코드로 그린 32×32 아이콘 · macOS Dock 숨김
- `crates/app/src/browser.rs` — Windows `rundll32 url.dll,FileProtocolHandler` · macOS `open`

## 설계 결정

- 시작해도 브라우저를 자동으로 열지 않는다 — 자동 실행(D13)으로 로그인할 때마다 창이 뜨지 않게. 여는 경로는 트레이
  "열기" 와 두 번째 실행(핸드오프) 둘.
- 두 번째 실행은 `instance.json` 읽기 · 요청을 300ms 간격 최대 10회 다시 시도한다 — 주 인스턴스가 잠금을 잡은 뒤
  `instance.json` 을 쓰기 전 구간(stage 2 리뷰 지적 3 의 경쟁 구간).
- "종료" 는 `instance.json` 삭제 → 잠금 해제 → 런타임 1초 정지 후 프로세스 종료. 강제 종료 뒤 남은 `instance.json` 은
  다음 주 인스턴스가 덮어쓴다.
- 로거는 주 · 두 번째 실행 모두 같은 파일에 덧붙인다(두 번째 실행의 실패도 남기려고).
- 트레이 이벤트는 tray-icon 문서 권장대로 `EventLoopProxy` 로 넘긴다. macOS 는 루프 시작(`StartCause::Init`) 뒤에
  트레이를 만든다.

## 테스트 결과

```text
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 23.73s
$ cargo build --release --workspace && python -c "…exes=…"
exes=1 names=agentNetwork.exe
$ cargo test --workspace      → test result: ok 13줄, FAILED 0
$ python tools/check_crates.py
crates=2 edges=1 violations=0
```

실제 실행(Windows 11, 2026-10-11, 릴리스 exe 를 빈 임시 폴더에 복사해 실행 — 브라우저를 여는 동작은 하지 않음):

```text
프로세스 agentNetwork.exe 1개
%LOCALAPPDATA%\agentNetwork: agentNetwork.db · instance.json · instance.lock · logs\agentNetwork.log
LISTEN 127.0.0.1:62905 (0.0.0.0 아님)
GET /api/status (쿠키) → {"version":"0.1.0","data_dir":"%LOCALAPPDATA%\agentNetwork","db":"ok (스키마 1)"}
GET /api/status (토큰 없음) → 401
exe 폴더: agentNetwork.exe 하나
로그: "INFO agentNetwork: 시작 — 127.0.0.1:62905"
```

트레이 아이콘 표시 · "열기" 로 브라우저 · 두 번째 실행 핸드오프 · "종료" 는 화면 확인이 필요해 **E10(수동)** 으로 남긴다.

## 다음 단계

- E10 Windows 실동작(사용자) · [plan](../plan.md) Stage 4 — 보유한 Mac 에서 universal 빌드 · E9 · E11(사용자).

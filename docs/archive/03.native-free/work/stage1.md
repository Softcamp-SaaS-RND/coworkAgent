# stage1 — 브라우저 알림 · 트레이 표시 · 트레이 오류 모드 · 네이티브 제거

- **적대적 평가 기준선(BASE)**: `48ec624` — D30 · plan 커밋(사용자 지시 · 확인 답, 2026-10-11).

## 목표

[plan](../plan.md) Stage 1 — Notifier 재작성(broadcast · 트레이 건수) · SSE `/api/events` · 트레이 "새 요청" · 트레이 오류 모드 ·
WinRT 토스트 · 레지스트리 · AppleScript · 메시지 상자 제거. 대상 gate: E1~E7(E8 수동은 D28).

## 산출 파일

- `crates/app/src/{notify,tray,main}.rs` · `crates/app/Cargo.toml`(`windows` 제거 · `windows-sys` 는 Shell · WindowsAndMessaging 만 ·
  `futures-util` · tokio `sync` · `time`) · `crates/app/web/{index.html,app.js}`
- `crates/app/tests/{notifier,events_api,start_error}.rs`(최종 6 · 4 · 1 — 개정 1 · 2 로 늘었다) · `settings_api.rs`(새 `Notifier::new` 시그니처)

## 설계 결정

- 트레이 이벤트 루프를 `main` 첫 줄에서 만든다(`Tray::new`) — 시작이 실패해도 같은 루프로 오류 모드를 띄운다. 핸드오프로 끝나는
  보조 인스턴스는 트레이를 띄우지 않는다.
- `Notifier::on_event` 는 원자 연산 · 프록시 전송 · broadcast 전송뿐 — 02 라운드 1·2 의 "막힘" 지적 경로(OS 호출 · 파일 로그)가 사라졌다.
- 웹UI 가 SSE 에 붙은 뒤 `POST /api/seen` 으로 "봤다" 를 알린다(트레이 표시 지움 — GET 은 상태를 바꾸지 않는다, 개정 1). 트레이
  "열기" · "내 정보 보기" 도 브라우저 열기에 성공하면 같다.
- 시작 실패 파일은 임시 폴더 — 데이터 경로가 원인이어도 쓸 수 있게. "오류 보기" 는 `browser::open(파일 경로)`(두 OS 기본 앱).

## 테스트 결과

- 워크스페이스 71 통과 · clippy(dev · release) 경고 0 · E6 `native=none`.
- mutation: 켜짐 무시 → notifier 2건 FAIL · SSE 붙어도 안 지움 → events_stream_delivers_and_clears FAIL · 꺼도 안 지움 →
  clear_and_off_reset_tray FAIL.
- 브라우저 알림 실표시 · 트레이 표시 · 오류 모드는 수동(E8, D28).

## 다음 단계

gate 원장 → 적대적 평가(2 라운드) → 종결.

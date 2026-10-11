# stage1 — 인바운드 이벤트 · 알림 설정 · 토스트

- **적대적 평가 기준선(BASE)**: `7528ce8` — plan 커밋(자동 진행 위임 + 사용자 확인 답, 2026-10-11). 평가 범위에 01 개정 3
  (`48c1264`)을 함께 싣는다 — 요청 base 는 그 앞 `dfab977`.

## 목표

[plan](../plan.md) Stage 1 — `Node` 인바운드 이벤트 · `settings` 스키마 3 · `Notifier` · 토스트 XML · 설정 API · 웹UI 토글.
대상 gate: E1~E6(E7 수동은 D28).

## 산출 파일

- `crates/net/src/node.rs`(`PeerEvent` · `on_event` · `answer` 가 이벤트를 함께 돌려줌) · `crates/net/tests/events.rs`(2)
- `crates/core/src/{db,settings,lib}.rs`(스키마 3) · `crates/core/tests/paths.rs`(스키마 3 단정)
- `crates/app/src/{notify,main,lib}.rs` · `crates/app/Cargo.toml`(`windows` 0.62.2 · `Win32_System_Registry`) ·
  `crates/app/web/{index.html,app.js}` · `crates/app/tests/{notifier,settings_api}.rs`(3 · 2)

## 설계 결정

- 이벤트 콜백은 DB 잠금을 놓은 뒤, 응답을 쓰기 전에 부른다 — 발신 쪽이 응답을 받았을 때 이벤트는 이미 났다(테스트가 기다리지 않음).
- `Notifier` 는 Node 와 다른 DB 연결(설정 표만)을 쓴다 — Node 의 연결 잠금과 엮이지 않는다.
- AUMID 레지스트리 등록은 첫 표시 때 한 번(`Once`) — 알림이 꺼진 채로는 레지스트리에 쓰지 않는다(N5).
- macOS 는 `osascript display notification`, 원격 문자열은 AppleScript 문자열 이스케이프(`\` · `"`). 확인은 Phase 4(D25).

## 테스트 결과

- 워크스페이스 67 통과 · clippy(dev · release) 경고 0.
- mutation: 켜짐 무시 → notifier 2건 FAIL · `<` 미이스케이프 → 이스케이프 FAIL · 저장 안 함 → toggle_persists FAIL ·
  stale 승인에도 이벤트 → applied_approve_emits_approved_only FAIL.
- 실제 토스트 표시 · 집중 지원 억제는 수동(E7, D28 이관) — 테스트가 사용자 데스크톱에 알림을 띄우지 않게 OS 표시는 대역.

## 다음 단계

gate 원장 → 적대적 평가(2 라운드) → 종결.

# stage1 — 이벤트 종류 · 거르기 · 설정 API · 상한 기본값

## 목표

[plan](../plan.md) Stage 1. 대상 gate: E1 ~ E6.

## 산출 파일

- `crates/net/src/node.rs` — `PeerEvent::Message.kind` · `MessageKind{Request, Reply, RelayStart, RelayStep, RelayEnd}`
- `crates/agent/src/chat.rs` — 종류를 붙여 냄(`notify`), 자동 멈춤 `relay_end`(상한 · 상대/내 실행 실패 · 거절 · 전달 확인 안 됨 · 상태
  저장 실패 · 상대가 멈춤), `RELAY_CAP_KEY` · `default_relay_cap`
- `crates/app/src/notify.rs` — `Prefs`(종류별 · 자동 왕복 모드 · 미리보기) 거르기, `SettingsPatch` · `Notifier::update`(검증 후 한 트랜잭션),
  `GET/PUT /api/settings` 확장(틀리면 400, 모르는 항목 거부)
- `crates/app/src/chat_api.rs` — 상한 없는 자동 왕복 = 설정 기본값
- 테스트: notifier 3 · settings_api 1 · relay 1 · chat_api 1 신규

## 설계 결정

- 받은 쪽 시작 = 대화의 첫 에이전트발 요청 · 재개 알림. 같은 uuid 를 실행 요청으로 올리는 재개 넘김은 중간(`RelayStep`) — 재개 알림이 이미
  시작을 알렸다.
- 끝 알림 본문은 이 PC 가 만든 사유라 미리보기 끄기와 무관하게 보인다.
- 사람이 멈춘 것(멈춤 버튼 · 글쓰기)은 끝 알림을 내지 않는다 — 본인이 한 일.

## 테스트 결과

- 워크스페이스 · clippy 통과. mutation 5종 모두 FAIL(첫 요청 판정 · 상한 끝 알림 · 중간 글 모드 · 미리보기 · 거르기).

## 다음 단계

Stage 2 — 웹UI 설정 화면.

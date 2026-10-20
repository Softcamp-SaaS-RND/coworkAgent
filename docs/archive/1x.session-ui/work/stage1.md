# stage 1 — 서버(세션 생성 · 첫 글 자동 왕복 · 기록 삭제 · `Clear` · 설정 · 받은 요청 알림 글)

## 목표

plan stage 1 — an-core `clear_conversation` · `busy` · `empty` · `waiting_requests` · 테마 기본 라이트 · `ui_tile_size`, an-net `Request::Clear` · `Pong.clear` · `MessageKind::Clear`,
an-agent `open_session` · 첫 글 자동 왕복 · `clear_conversation` · `inbound_clear` · `deliver` 의 `clear`, an-app 빈 본문 생성 · `POST .../clear` · 알림 설정 + Rust 시험(E1 · E2 · E4 ~ E6).

## 산출 파일

- `crates/core/src/chat.rs` — `clear_conversation`(글 · cli_tx · relay_events · 그 글의 outbox · agent_sessions · ctx_seeds · 세션 id · 자동 왕복 켜짐/횟수 → 0, 대화 행 · 제목 · 그룹 · 즐겨찾기 · 공유 ·
  자동 왕복 설정은 남김, `lower_marks`), `busy`(running · queued), `empty`, `waiting_requests`(받은 요청 대기 최신 n 건 + 대기 수).
- `crates/core/src/run_opts.rs` — `theme` 기본 `light`(`THEMES[0]`), `TILE_SIZE_KEY` · `TILE_SIZES` · `tile_size`. `crates/core/src/outbox.rs` — `CLEAR`.
- `crates/net/src/proto.rs` — `Request::Clear{conv}`(`conv()` · `ids_ok`), `Pong.clear`. `crates/net/src/node.rs` — 처리기 넘김 목록 · `Pong{clear: true}` · `knows_clear` · `MessageKind::Clear`.
- `crates/agent/src/chat.rs` — `open_session`, `send_with(Some)` 의 "빈 자동 왕복 대화 첫 글 = 자동 왕복 시작"(요청이면 `REQUEST_NO_RELAY`), `clear_conversation(id, all)` · `reshare` · `share_head`(share_add 와 공유) ·
  `inbound_clear`, `deliver` 의 `clear`(모르는 상대 = 버림 + 로그), 포기 제외, `CLEAR_BUSY` · `CLEAR_UNKNOWN`.
- `crates/app/src/chat_api.rs` — `NewConversation.text` 기본값 · 빈 본문 = `open_session`(201, `via` · `request` 와 함께 400), `POST /api/conversations/{id}/clear`.
- `crates/app/src/notify.rs` — `Prefs.notify_sound`(기본 참) · `SettingsPatch.notify_sound` · `ui_tile_size`(검증 · 저장 · `settings_json`), `request_notice` · `REQUEST_LINES` · `Notifier::waiting`,
  `on_event` 의 받은 요청 = 묶음 알림(상태 잠금을 놓고 DB 를 읽는다), `MessageKind::Clear` 알림 글.
- 시험: `crates/agent/tests/session_open.rs`(새 4) · `conv_clear.rs`(새 7), `crates/app/tests/ux_api.rs` 새 6 + `theme_setting_persists` 기본 단정 삭제(→ `theme_default_light`),
  `Pong` 모양 2(`context.rs` · `remote_input.rs`).

## 설계 결정

- **관람자 사본 비우기 = 머리(`Share`) 다시 보내기** — 사본은 같은 uuid 머리를 받으면 글을 비운다(#1o). 새 프레임 없음. 그 대화의 기다리는 `SharePost` · `ShareDelete` 행을 먼저 버린다
  (줄이 순서대로 보내므로 지운 글이 머리 뒤에 다시 오지 않게).
- **받는 쪽 실행 중 = `retry`** — 보낸 쪽 `send_to` 가 `retry` 를 전송 오류로 바꿔 `backoff` 로 다시 보낸다(plan 의 `busy` 대신 기존 재시도 코드). 포기 대상에서 뺐다(나가기와 같게).
- **기록 삭제는 대기 글(`queued`)도 막는다**(`busy`) — 대기열이 지운 글을 찾지 않게. plan 은 "실행 중" 만 적었다 — 같은 축(D40 막음)으로 넓힘.
- **비우면 자동 왕복을 끈다**(설정은 남김) — 비운 뒤 첫 글은 빈 자동 왕복 대화 규칙으로 다시 연다.
- `clear_conversation(all)` 의 상대 확인 — `knows_clear` 가 `PermissionDenied`(페어링 아님)면 그 오류, 그 밖의 전송 오류(꺼짐)면 이 PC 를 비우고 줄에 맡긴다(plan 개정 기록의 해석).
- 닫힌 대화로 온 `Clear` 는 `handle` 앞 `chat::left` 가 `left` 로 거절한다(따로 막지 않음). 다른 상대의 같은 uuid 는 `find_own` 이 못 찾아 `unknown`.

### 구현자 해석(사용자 승인 2026-10-20 — 종결 승인)

1. **모르는 상대에 늦게 닿은 `Clear`** — 누를 때는 꺼져 있다가 나중에 이전 버전으로 확인되면 행을 버리고 로그만 남긴다(plan 은 "내 활동 기록" — 내 활동은 대화 글 · 자동 왕복 기록 표라 대화 없는
   메모 자리가 없다). 이 경우 상대 쪽 기록은 그대로다.

## 테스트 결과

- 물기 확인 — `inbound_clear` 의 실행 중 막음을 끈 빌드에서 `clear_busy_running` FAILED, `clear_conversation` 의 모르는 상대 거절을 끈 빌드에서 `clear_all_old_peer` FAILED → 되돌림.
  처음 `clear_busy_running` 은 막음을 꺼도 통과했다(프레임이 닿기 전에 단정) → 실행 중인 동안 0.9초 기다린 뒤 단정하도록 고침.
- gate(이 stage):

```text
E4  cargo test -p an-agent --test session_open session_  → test result: ok. 4 passed; 0 failed
E5  cargo test -p an-agent --test conv_clear clear_       → test result: ok. 7 passed; 0 failed
E6  cargo test -p coworkagent-app --test ux_api -- theme_default_light tile_size_setting_persists notify_sound_setting request_notice_text create_session_without_text clear_route
                                                           → test result: ok. 6 passed; 0 failed
E1  cargo test --workspace --no-fail-fast                  → 78 binaries, 461 passed, 0 failed
E2  cargo clippy --workspace --all-targets -- -D warnings  → Finished(경고 0)
```

- `cargo fmt --check` 는 기준 커밋에서도 실패한다(검증 명령 표에 없음) — 손대지 않았다.

## 다음 단계

stage 2 — 웹UI(세션 생성 · ⋯ 하위 메뉴 · 나가기 확인 · 사이드바 손잡이 · 머리 부제 · 타일 크기 · 받은 요청 알림 · 효과음 · 이름 바꾸기) · 문서(E3 · E7 ~ E12).

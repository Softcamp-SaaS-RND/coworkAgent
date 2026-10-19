# stage 1 — 변경 신호 · 부분 다시 읽기

## 목표

plan stage 1 — 서버 변경 신호(net · agent · app) + 웹UI 소비자 표 · 합치기 · 부분 다시 읽기 · 시험 · 문서(E1 ~ E10).

## 산출 파일

- `crates/net/src/node.rs` — `PeerEvent::Message.conv`(이 PC 대화 id) · `PeerEvent::Changed { conv }`.
- `crates/net/src/proto.rs` — `Request::conv()`(대화 uuid, `ids_ok` 와 같은 모양).
- `crates/agent/src/chat.rs` — `notify` 가 `conv` 를 싣고, `handle` 이 대화 있는 요청을 처리한 뒤 `chat::find`(공유 사본 포함)로 id 를 찾아 `Changed` 를 낸다.
- `crates/app/src/notify.rs` — `Change` · `Res` · `change_of` · `next_change`, `Notifier.changes` · `subscribe_changes`, `on_event` 맨 앞에서 변경 신호,
  SSE `events` = 알림 스트림 + `event: change` 스트림(`stream::select`).
- `crates/app/web/view.js` — `CONV_LOADERS` · `loadersFor` · `mergeChanges`. `app.js` — `refresh(only)` · `onChange`(50ms 합치기) · `change` 구독 ·
  알림 처리기에서 `refresh()` 제거 · `setInterval(() => refresh(), 5000)`.
- 시험: `crates/app/tests/notifier.rs`(change_ 4) · `events_api.rs`(1) · `crates/agent/tests/message_tidy.rs`(1 + 기존 1 보정) · `view.test.mjs`(6).
- 문서: [web-ui](../../../manual/dev/web-ui.md) `## 알림` · [an_app](../../../manual/dev/an_app.md) `notify::routes`.

## 설계 결정

- 변경 신호는 `on_event` 의 켜짐 · `allows` **앞** — 알림을 꺼도 · 집중이어도 간다(결정 1). `Changed` 는 거기서 끝(건수 · 알림 없음).
- `handle` 은 결과(오류 포함)와 무관하게 신호를 보낸다 — 다시 읽기는 해가 없다.
- `chat::find`(공유 사본 포함) — 관람자 사본(D38 · #1o)도 id 로 찾아 열린 대화만 거른다.
- `notice_with` 의 `Changed` 갈래는 쓰이지 않는 빈 `Notice`(공개 함수 시그니처 유지, `on_event` 가 먼저 돌려보낸다).
- 기존 시험 보정: `message_tidy.rs` `tidy_runs_and_notifies_nothing` 의 이벤트 수는 `Changed` 를 빼고 센다("알림 없음" 뜻 그대로 — plan 명세 편차 4).
- `cargo fmt --all` 이 이 Task 밖 파일 수십 개를 바꿔(저장소가 fmt 기준과 다름) 그 파일들은 되돌리고 손댄 파일만 남겼다.

## 테스트 결과

- RED 먼저: node `loadersFor|mergeChanges` → `does not provide an export named 'CONV_LOADERS'`(fail 1), plan E5 · E8 실행 → `0 passed` · `notice_refresh=1`.
- GREEN(2026-10-19):

```text
cargo test --workspace                         → TEST_RC=0 (test result: ok 67 묶음, 실패 0)
cargo clippy --workspace --all-targets -D warnings → Finished · CLIPPY_RC=0
node --test "crates/app/web/*.test.mjs"        → # pass 130 · # fail 0
cargo test -p coworkagent-app --test notifier change_   → ok. 4 passed; 0 failed
cargo test -p coworkagent-app --test events_api change  → ok. 1 passed; 0 failed
cargo test -p an-agent --test message_tidy     → ok. 12 passed; 0 failed
python tools/check_crates.py                   → crates=4 edges=6 violations=0
python tools/check_readme.py                   → readme ok
```

- E10(실화면 두 PC)은 19 인수로 이월(D28).

## 다음 단계

`check_gates` → 적대적 평가 라운드 1/2.

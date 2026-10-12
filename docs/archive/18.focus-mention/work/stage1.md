# stage1 — 집중 · 접속 상태

> 근거: [plan](../plan.md) Stage 1. 사용자 검증은 진행 목표대로 18 종결 뒤 일괄(D28 — 19.prototype-acceptance).

## 목표

사용자가 켜는 집중(이 PC)으로 상대 PC 의 새 실행 요청을 즉시 거부하고(새 오류 코드 `focus`), 보낸 쪽은 "상대가 집중 중" 으로 보이며, 진행 중
자동 왕복은 다음 차례에서 멈춘다. 상대의 집중 · 온라인을 `Ping` · `Pong` 으로 재서 레일 · 사이드바 머리 · 대화 머리글에 보인다. 스키마 12 는 이
Task 의 열을 한 번에 올린다. 대상 gate: E3 · E4 · E5 · E6 · E7 · E8 · E9 · E10 · E11(이 단계 몫).

## 산출 파일

- `crates/core/src/db.rs` — `SCHEMA_VERSION` 12: `messages.request` · `done_at` · `agent` · `refused`, `peers.seen_at` · `focus`, `agent_sessions` 표
  (기존 규칙 — 한 트랜잭션 · 있는 열 건너뜀 · `{db}.v11.bak`).
- `crates/core/src/settings.rs` — `FOCUS_KEY`(`focus`) · `focus(conn)`(값 `1` 만 켜짐, 읽기 오류는 그대로).
- `crates/core/src/peers.rs` — `Peer.seen_at` · `Peer.focus` · `set_presence(id, focus)`(받은 시각 = `unixepoch()`).
- `crates/core/src/chat.rs` — `Message.request` · `done_at` · `agent` · `refused`(2 · 3 단계가 쓴다), `set_refused` · `agent_session` ·
  `set_agent_session`. `crates/core/src/agents.rs` — 폴더가 바뀌면 `agent_sessions` 의 그 에이전트 행도 지운다(표를 만드는 이 단계에서).
- `crates/net/src/proto.rs` — `Response::Pong { focus }` · 오류 코드 목록에 `focus`. `crates/net/src/node.rs` — `Ping` 의 답에 이 PC 집중 ·
  `ping(id) -> bool`(받은 `Pong` 을 접속 상태로) · `watch_presence()`(시작 직후와 `PRESENCE_EVERY` 30초마다 페어링된 상대마다 Ping).
- `crates/agent/src/chat.rs` — `inbound_message` 형식 검사 뒤 집중이면 `focus` 거부(기록 · 실행 · 알림 없음) · `inbound_reply` 자동 왕복 —
  집중이면 넘기지 않고 끔(끝 사유 "집중 중") · 보낸 쪽 거절(`send_human` · `relay_forward`) → `refused_by_peer`(실패 + 거절 코드, `focus` 면 그
  상대 접속 · 집중) · 오류 문구 "상대가 집중 중 — 요청을 받지 않습니다. 나중에 다시 보내세요" · 자동 왕복 끝 사유 "상대가 집중 중".
- `crates/app/src/chat_api.rs` — 메시지 JSON 에 `request` · `done` · `agent` · `refused`(보강 `dbd5230` — 거절 사유가 화면에 닿지 않던 것).
- `crates/app/src/notify.rs` — `Prefs.focus` · `SettingsPatch.focus` · `allows()` 는 `notify_paused || focus` 면 거짓. `crates/app/src/peers_api.rs` —
  `PeerView.online` · `focus`(`ONLINE_WITHIN` 75초, 시각은 내지 않음). `crates/app/src/main.rs` — `watch_presence` 를 띄운다.
- 웹UI — `view.js` `profileMenu(paused, focus)` · `presence(p)` · `refusedText(code)`, `index.html` 상태 칩 `#focus_chip` · 머리글 `#conv_presence`,
  `app.js` 칩 · 메뉴 · 레일 점(`.rail-dot`) · 레일 이름 끝 상태 · 사이드바 머리 상태 · 머리글 상태 · 실패 사유 · 새로 고침에 피어 목록,
  `app.css` 칩 · 점 · 상태 글자(토큰만).
- 테스트 — `core/tests/paths.rs` `schema12_focus_mention`(+ 지금 버전 11 → 12 여덟 곳 · "더 새 스키마" 12 → 13 · `schema11_workspace` 는
  `db::SCHEMA_VERSION`) · `net/tests/pairing.rs` `pong_carries_focus_and_presence` · `agent/tests/roundtrip.rs` `focus_refuses_new_request_unrecorded` ·
  `agent/tests/relay.rs` `focus_stops_relay_both_sides` · `app/tests/notifier.rs` `focus_pauses_notices` · `app/tests/ux_api.rs`
  `focus_setting_and_peer_presence` · `app/tests/ux_literals.rs` `focus_mention_labels`(이 단계 문구) · `web/view.test.mjs` `presence`(+ `profile_menu` 보강).

## 설계 결정

- **집중 판정은 받는 순간 설정을 읽는다**(재시작 불필요). 판정 값을 먼저 받고 `if`(조건식의 DB 잠금이 블록 끝까지 사는 edition 2024 함정 —
  17 Stage 1 리뷰 H1). **설정을 못 읽으면 받는다** — 실행이 같은 설정 오류로 실패 답을 낸다(15 R2 결정 `settings_read_error_fails_run` 그대로,
  기본값으로 감추지 않는다). 거부로 바꾸면 그 회귀가 깨진다(실측).
- **`POST /api/peers/{id}/ping` 은 204 그대로** — plan 은 `{focus}` 를 돌려준다고 했으나 15 결정 `direct_add_with_code_pairs_both_sides` 가 204
  를 단정한다(실측 RED). 받은 `Pong` 은 접속 상태로 남고 화면은 `GET /api/peers` 의 `online` · `focus` 로 본다 — **plan 편차 1**(API 문구, gate 무관 —
  plan 본문도 고쳤다).
- **`peers::set_focus` 는 두지 않았다** — 거절도 접속 증거라 `set_presence(id, true)`(시각 = 지금) 하나로 Pong · 거절을 같이 남긴다(plan 문구의
  `set_focus` 삭제 — 편차 2, gate 무관).
- **자동 왕복 끝 사유 우선** — 상대 실행 실패 → 상한 → 집중(같은 답이 둘 이상이면 앞 것).
- **화면 갱신** — 접속 상태는 30초 주기 Ping 이 바꾸므로 5초 새로 고침이 `GET /api/peers` 도 읽는다(레일 · 사이드바 머리 · 머리글 · 연결 목록은
  서명이 같으면 다시 그리지 않는다 — 17 의 `unchanged` · `needsRender`).
- **집중과 알림 일시 중지** — 집중은 알림도 막지만 `notify_paused` 값은 바꾸지 않는다. 종 아이콘의 흐림 표시는 둘 중 하나라도 켜지면.

## 테스트 결과

```text
$ cargo test --workspace            (전부 ok — paths 14 · pairing 6 · roundtrip 14 · relay 11 · notifier 12 · ux_api 25 · ux_literals 9 …)
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.12s
$ node --test --test-reporter=tap "crates/app/web/*.test.mjs"
# pass 32
# fail 0
```

RED 확인: 테스트를 먼저 넣고 돌렸을 때 view(`presence` 없음 — SyntaxError) · Rust(새 API 없음 — 컴파일 오류)로 실패. 구현 중 RED 2건 —
`settings_read_error_fails_run`(집중 판정이 설정 오류로 거부 → 받기로 고침) · `direct_add_with_code_pairs_both_sides`(ping 204 → 유지).

눈 확인(가짜 API `mock18.py`, 엣지 headless 1440×900 · 다크): 상단 바 칩 "받기 가능" → 누르면 "집중"(상태 줄 안내) · 프로필 메뉴 "집중 — 요청 받지
않음"(켜짐 표시) · 레일 점(집중 = 주황 · 오프라인 = 회색, 이름 끝에 "— 집중 중" · "— 오프라인") · 머리글 "집중 중" · 거절된 글 "오류 · 상대가 집중
중" · 사이드바 머리 "… · 페어링됨 · 집중 중". 페이지 오류 0.

## 리뷰

자체 발견 1건(`dbd5230`): 메시지 JSON 에 `refused` 가 없어 거절 사유가 화면에 닿지 않았다 — ux_api 단정을 먼저 넣어 RED 확인 뒤 고침.

읽기 전용 리뷰 에이전트(범위 `7d7766b..dbd5230`, 임시 worktree 에서 재실행 — H 0 · M 2 · L 7 · N 7). 판단 · 조치:

| # | 지적 | 판단 | 조치 |
|---|---|---|---|
| M1 | 집중으로 거부된 자동 왕복 차례를 재개로 다시 보내 성공해도 `refused = focus` 가 남아 완료된 글 옆에 "상대가 집중 중" | 사실(재현) | 다시 보낼 때 상태와 한 잠금에서 `set_refused(None)`(`Option` 으로) · 화면은 실패 상태일 때만 사유(`refusedText(m)`) · 대화 서명에 사유. relay `focus_stops_relay_both_sides` · view `presence` 보강 |
| M2 | 새로 고침의 `loadPeers` 가 `renderSideAgents` · `fillPeerSelects` 를 5초마다 다시 만들어 초점 · 고르는 중인 칸을 빼앗는다(17 `unchanged` 규약 회귀) | 사실(재현) | 두 곳에 서명 가드. 눈 확인 — 링크 초점 6.5초 유지 · option 그대로 |
| L1 | 연결 목록 서명에 `online` · `focus` — 접속이 바뀌면 저장 전 입력이 지워진다 | 사실 | 서명에서 뺀다 |
| L2 | 자동 왕복 시작이 거절돼도 켜짐이 남아 재개가 상대에 없는 대화로 간다 | 사실 | `send_human` 거절 분기에서 여는 글이면 끈다. relay 보강 |
| L3 | 이미 받은 글의 재전송도 집중이면 `focus` — 연 쪽이 실패로 확정해 늦게 온 답을 버린다 | 사실(프레임) | 재전송(같은 uuid · 승격 아님)은 거부하지 않는다 — `Chat::resent`. plan 개정 2(해석 7). roundtrip `focus_refuses_new_request_unrecorded` 보강 |
| L4 | 주기 Ping 이 놓친 tick 을 몰아 보낸다(절전 복귀) | 사실(tokio 기본값) | `MissedTickBehavior::Skip` |
| L5 | 상대가 준 거절 코드를 제한 없이 저장 · 표시 | 사실 | `an_net::proto::clean_code`(`[a-z_]` 1 ~ 32, 그 밖 `unknown`) — 저장 · 오류 문구 둘 다. 단위 테스트 |
| L6 | 온라인 판정이 `From` 안의 지금 시각 — 경계 시험 불가 · 시계를 되돌리면 계속 온라인 | 사실 | `Peer::online(now)` 메서드(`0..=75`) + 경계 74 · 75 · 76 · 음수 단위 테스트 |
| L7 | `watch_presence` 테스트 없음 · relay 가 재개 뒤를 안 봄 · view 가 상태와 함께 보지 않음 | 사실 | 시작 직후 Ping 단정(pairing 보강) · M1 · view 보강. `.filter(Peer::paired)` 제거는 단정으로 가르지 못한다(상대가 페어링 아닌 Ping 을 거절해 기록이 남지 않음) · `main.rs` 의 띄우기는 E14 수동 — 미조치 |
| N1 | `Ping` 답의 설정 읽기 오류가 연결을 끊는다 | 사실 | 꺼짐으로 답하고 기록 |
| N2 | 연 쪽 집중은 `inbound_reply` 에서만 — 상대가 한 차례 더 돈다 | 설계대로(해석 8) | 미조치 |
| N3 | 상태 칩이 `aria-pressed` 와 라벨 전환을 함께 | plan 수준(해석 15) | 미조치 — 19 인수 때 확인 |
| N4 | 알림마다 `GET /api/peers` 두 번 | 사실 | `listen()` 의 `loadPeers()` 제거(`refresh()` 가 읽는다) |
| N5 | 레일 aria-label 의 상태가 끝이 아니다 | 사실 | 끝으로 |
| N6 | plan 본문 수정에 개정 기록 없음 · 산출 파일에 `chat_api.rs` 빠짐 | 사실 | plan 개정 1(Stage 2 커밋) · 위 산출 파일 |
| N7 | `message_json` 의 `refused` | 이미 고침 | — |

고친 뒤: `cargo test --workspace` 전부 ok(새 단위 테스트 `peers::tests::online_window_edges` · `proto::tests::clean_code_keeps_known_shape_only`) ·
clippy `-D warnings` 통과 · node `# pass 33`. RED 확인 — relay(M1 재개 뒤 사유) · roundtrip(L3 재전송) · view(실패 상태만)가 고치기 전 실패.

## 다음 단계

Stage 2 — 질의 · 요청 · 완료(프레임 `request` · `Done` · API 거르기 · 입력창 종류 · 태그 · 완료 작업 · 실행 상세 '요청' · 내 활동 · 보낸 메시지 거르기).

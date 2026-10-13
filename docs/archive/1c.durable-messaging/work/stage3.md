# stage3 — 접속 상태는 통신할 때만

> 근거: [plan](../plan.md) Stage 3. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

30초 주기 접속 확인을 없애고 통신할 때만 접속 시각을 갱신한다 — 화면 "마지막 통신 n분 전"(A8). 바뀐 포트 · IP 따라가기는 시작 알림으로(A9).
대상 gate: E1 ~ E3 · E7.

## 산출 파일

- `crates/net/src/node.rs` — (Stage 1 커밋에 먼저) `watch_presence` · `PRESENCE_EVERY` 삭제 · `announce`(시작 때 페어링된 상대마다 `Ping` 한 번) · 보내기
  성공 · 페어링된 상대 요청 · `Ping` 받기마다 `touch` + 이번: `refresh(id, within)`(마지막 통신이 `within` 초 안이면 보내지 않음).
- `crates/core/src/peers.rs` — `touch`(10초에 한 번 — Stage 1). `crates/app/src/main.rs` — `announce`(Stage 1).
- `crates/app/src/peers_api.rs` — 피어 JSON `seen_at` · `POST /api/peers/{id}/ping` 은 `refresh(id, 60)`.
- 웹UI — `view.js` `presence(p, now)`("방금 통신" · "n분 · n시간 · n일 전 통신" · "통신 기록 없음", 방금일 때만 점 · 집중), `app.js` 웹UI 를 열 때 페어링된
  상대 모두 · 레일 타일을 고를 때 그 상대 ping(`pingPeers`), 레일 서명에 표시 문구.
- 테스트 — `net/tests/presence.rs`(4) · `view.test.mjs` presence 1 + 기존 presence 단정 갱신.

## 설계 결정

- **피어 JSON 에 시각(`seen_at`)을 싣는다** — 1b 까지는 "레일이 30초마다 다시 그려지지 않게" 시각을 내지 않았다. 이제 시각은 통신할 때만 바뀌고, 화면이
  "n분 전" 을 직접 센다(레일 서명 = 표시 문구 — 분이 바뀔 때만 다시 그림).
- **"온라인" 은 "방금 통신"(75초 안)** — 주기 확인이 없으니 오래 통신이 없던 상대는 실제로 켜져 있어도 "n분 전 통신" 이다(사용자 수용 — 결정 3). 보내기는
  상대가 꺼져 있어도 보관되므로(Stage 1) 표시가 보내기를 막지 않는다.
- **화면 확인은 서버가 60초로 거른다** — 웹UI 를 여러 탭 · 타일을 빠르게 바꿔도 상대에 연결이 몰리지 않는다.
  대가: 상대가 집중을 켜고 끈 것은 60초 안 다시 고르면 바로 보이지 않는다(다음 통신 · 기한 뒤 확인에서 반영 — `ux_api` 집중 테스트는 기한을 돌려 확인).

## 테스트 결과

```text
cargo test -p an-net --test presence                         → test result: ok. 4 passed; 0 failed
node --test --test-reporter=tap "crates/app/web/*.test.mjs"  → # pass 42 / # fail 0
cargo test --workspace                                       → 실패 0 (ux_api 28 · ux_literals 9 — 피어 JSON seen_at · 새 문구로 갱신)
cargo clippy --workspace --all-targets -- -D warnings        → 경고 0
```

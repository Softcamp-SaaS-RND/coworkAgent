# stage2 — 질의 · 요청 · 완료

> 근거: [plan](../plan.md) Stage 2. 사용자 검증은 진행 목표대로 18 종결 뒤 일괄(D28 — 19.prototype-acceptance).

## 목표

사람 글마다 종류(질의 — 기본 · 요청)를 골라 보내고 양쪽 이력에 남긴다. 받은 요청은 받은 쪽만 완료 표시 · 취소하고 쓴 쪽에 상태로 보낸다(새
프레임 `Done`). 요청은 대화 안(실행 상세 '요청' 절) · 내 활동 종류 "요청"('내게 요청된 목록') · 보낸 메시지 탭 종류 칸으로 모아 본다. 대상 gate:
E3 · E4 · E6 · E8(이 단계 몫).

## 산출 파일

- `crates/net/src/proto.rs` — `Request::Message` · `Request::Post` 에 `#[serde(default)] request` · 새 `Request::Done { conv, msg, done }`.
  `crates/net/src/node.rs` — `Done` 은 페어링된 상대만 처리기로(미페어링은 `not_paired`).
- `crates/core/src/chat.rs` — `set_request` · `set_done(id, bool)`(이미 완료면 처음 시각을 둔다) · `message_by_id`, `ActivityFilter::Query` ·
  `Request`(상대 쪽 사람 글), `messages_across(…, request: Option<bool>, …)`(사람 글만 — 건수 제한 전).
- `crates/agent/src/chat.rs` — `Compose { via, title, request }`(Default · Copy) · `send_with` · `start_relay_with` · `post_with`(기존 `send_via` ·
  `start_relay` · `post_via` 는 기본 `Compose` 로 위임 — 호출부 그대로), 보낼 때 `request` 를 싣고 받은 쪽(`inbound_message` · `inbound_post` 사람
  글)이 남긴다. `mark_done(id, done, saved) -> (Message, 경고)`(받은 요청만 — 그 밖 `InvalidInput`, 저장 표시는 같은 트랜잭션 · 대화 차례 안 — 리뷰 L2 · L7) · `inbound_done`(그 상대 대화 · 내 요청 사람 글에만,
  그 밖 `unknown`). 실행 답 미러 · 자동 왕복 프레임은 `request: false`.
- `crates/app/src/chat_api.rs` — `POST /api/conversations` · `POST …/messages` 에 `request`, `PATCH /api/messages/{id}` 는 `saved` · `done` 중
  하나 이상(둘 다 없으면 400, `done` 은 받은 요청만 — 그 밖 400) → `{id, done?, warning?, saved?}`, `GET /api/messages` 의 `ask=query|request`(그 밖
  400), `GET /api/activity` 종류 `query` · `request`.
- 웹UI — `view.js` `askTag(m)`("요청" · "요청 · 완료") · `messageActions` 받은 요청 "완료 표시" | "완료 취소" · `inboxRows` · `activityRows` 라벨에
  요청 태그. `index.html` 입력창 · 새 대화 라디오(`name="ask"`, 묶음 "글 종류", "질의" 기본 · "요청") · 내 활동 종류 "질의" · "요청" · 보낸 메시지
  탭 `#sent_ask`("전체" · "질의" · "요청") · 실행 상세 절 "요청"(`#detail_req`). `app.js` 보내기에 `request`(보낸 뒤 "질의" 로) · 다시 보내기는
  원래 종류 · 글 머리 요청 태그 · 완료 작업(PATCH 뒤 새로 고침) · 대화 서명에 완료 · 실행 상세 '요청' 절(최근 글부터, 누르면 그 글로 — 저장된 항목과
  같은 줄 `detailItem`) · 보낸 메시지 종류 거르기. `app.css` `.tag.ask` · `.ask-kind`(토큰만).
- 테스트 — `agent/tests/roundtrip.rs` `ask_kind_and_done_sync_both_sides`(양쪽 종류 · 완료 · 취소 · 미러 요청 · 쓴 쪽 · 질의 · 답 거절 · 알림 실패
  경고) · `app/tests/ux_api.rs` `ask_kind_recorded_and_filtered` · `done_only_by_receiver` · `app/tests/ux_literals.rs` `focus_mention_labels` 보강
  ("글 종류" · "질의" · "요청" · "요청 · 완료" · "완료 표시" · "완료 취소") · `web/view.test.mjs` `ask_marks`. 프레임 리터럴에 `request: false`
  (cli_origin · relay · roundtrip · two_sided).

## 설계 결정

- **완료는 받은 쪽 PC 에 먼저 쓰고 보낸다** — 알리지 못하면 경고(이 PC 표시는 남는다). 누를 때마다 그때 값을 보내므로 상대 쪽은 다음에 바꿀 때
  맞춰진다. 경고 문구를 "다시 누르면 다시 보낸다" 에서 바꿨다 — 단추가 표시 ↔ 취소로 바뀌어 다시 누르면 취소가 간다(plan 개정 1).
- **`inbound_done` 은 조용히 거른다** — 그 상대 소유 대화가 아니거나, 그 대화의 내 요청 사람 글이 아니면 `unknown`(쓴 쪽 기록을 바꾸지 않는다).
  uuid 는 `messages.uuid UNIQUE` 라 대화 일치까지 보면 다른 대화 글을 건드리지 못한다.
- **종류 거르기는 사람 글만** — `ask` 가 있으면 에이전트 답은 빠진다(답에는 종류가 없다). 제한 전에 거른다(15 R2 결정 그대로).
- **다시 보내기는 원래 글의 종류** — 요청을 다시 보내면 요청(입력창 라디오와 별개). plan 이 정하지 않은 항목 — 위임 권장안.
- **완료 표시 뒤 화면** — 대화 서명(`messagesSig` — `id:state:길이:완료:거절 사유`)에 완료가 들어 상대가 바꾼 완료도 5초 새로 고침에 보인다. 내가
  누르면 그 글의 태그 · 단추 · 실행 상세만 바꾸고 서명을 맞춘다(리뷰 L3 — 대화를 다시 그리면 누른 단추의 초점이 사라지고 `aria-live` 대화를 다시
  읽어 준다).
- **새 대화 칸에도 글 종류** — plan 웹UI 는 입력창(서식 버튼 줄)만 적었으나 첫 글도 요청일 수 있어 새 대화 칸에 같은 라디오를 넣었다(plan 개정 3).
- **plan 편차(개정 1 — 설계 문구, gate 무관)**: `set_done(id, Option<i64>)` → `set_done(id, bool)` · `messages_across` 인자 `ask` → `request` ·
  Stage 1 이 남긴 `peers::set_focus` 문구 → `set_presence`.

## 테스트 결과

```text
$ cargo test --workspace            (전부 ok — roundtrip 15 · ux_api 27 · ux_literals 9 …)
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.65s
$ node --test --test-reporter=tap "crates/app/web/*.test.mjs"
# pass 33
# fail 0
```

RED 확인: 테스트를 먼저 넣고 돌렸을 때 Rust(`Compose` · `mark_done` · `request` 없음 — 컴파일 오류) · view(`askTag` 없음 — SyntaxError) ·
ux_literals(문구 없음)로 실패.

눈 확인(가짜 API `mock18s2.py`, 엣지 headless 1440×900 · 다크): 입력창 라디오 "질의"(기본) · "요청" → 요청으로 보내면 몸에 `request: true`, 보낸 뒤
"질의" 로 · 내 요청 글 머리 "요청" · "요청 · 완료" · 받은 요청 작업 "완료 표시" → PATCH `{done: true}` → "요청 · 완료" · "완료 취소" · 실행 상세
"요청 2"(최근 글부터) · 보낸 메시지 종류 "요청" 이면 요청 글만 · 내 활동 종류에 "질의" · "요청" · 새 대화 라디오. 페이지 오류 0.

## 리뷰

읽기 전용 리뷰 에이전트(범위 `dbd5230..a781c3f` — H 0 · M 0 · L 7 · 정보 1). 신뢰 경계(`inbound_done` 세 조건 · 페어링 전용 분기) · 잠금(보내기 전
해제) · `request` 기록 경로 · `done_at` · 서명 · `textContent` 는 문제 없음으로 봤다. 판단 · 조치:

| # | 지적 | 판단 | 조치 |
|---|---|---|---|
| L1 | PATCH 응답 모양이 plan(`done` 항상)과 다름 | 사실(문서) | 준 항목만이 맞다 — plan 개정 3(`done?`) |
| L2 | `{saved, done}` 이 원자적이지 않음(완료 · 알림 뒤 저장 실패면 502 인데 완료는 적용) | 사실 | `mark_done(id, done, saved)` — 한 트랜잭션, 자격이 없으면 아무것도 쓰지 않음. ux_api `done_only_by_receiver` 보강(둘 다 준 400 · 부분 적용 없음) |
| L3 | 완료 토글 뒤 대화 전체 다시 그리기 — 초점 소실 · `aria-live` 다시 읽기 | 사실 | 제자리 갱신 + 서명 맞춤(`messagesSig`). 눈 확인 — 초점 · 글 요소 6초 새로 고침 뒤에도 그대로 |
| L4 | 취소 알림 실패 경고가 "표시했다" | 사실 | 취소면 "이 PC 에서는 취소했다" |
| L5 | `Done` 신뢰 경계 테스트 공백 | 사실 | 미페어링 목록에 `Done` · 페어링된 상대의 조작 `Done`(내 질의 · 답 · 다른 대화의 요청) → `unknown` · 기록 그대로(roundtrip). 다른 상대 소유 대화는 세 번째 PC 가 필요해 미조치 — 같은 `chat::find(conv, peer)` 를 모든 들어온 처리기가 쓴다 |
| L6 | 문서 — 새 대화 라디오가 plan 에 없음 · E3 문구 · "(그 밖 `bad_request`)" | 사실 | plan 개정 3 · 위 정정 |
| L7 | 두 창의 반대 값 PATCH 경합(추정) | 추정 | 완료 쓰기 · 알림을 대화 차례(`Chat::turn`, 사람 글과 같은 잠금) 안에서 |
| 정보 | `kind=agent&ask=…` 는 빈 목록 | 명세대로 | 미조치 |

고친 뒤: `cargo test --workspace` 전부 ok(roundtrip 15 · ux_api 27) · clippy 통과 · node `# pass 33`.

## 다음 단계

Stage 3 — @멘션(글 `agent` · `agent_sessions` · 받은 쪽 수락 · 실행 세션 · 답 이름 · CLI · 입력창 후보).

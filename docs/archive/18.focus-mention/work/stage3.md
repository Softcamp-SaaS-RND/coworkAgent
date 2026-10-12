# stage3 — @멘션

> 근거: [plan](../plan.md) Stage 3. 사용자 검증은 진행 목표대로 18 종결 뒤 일괄(D28 — 19.prototype-acceptance).

## 목표

내가 연 대화(자동 왕복 아님)에서 글 안 `@이름` 으로 그 상대 PC 의 다른 에이전트를 부른다. 받은 쪽은 부른 에이전트를 그 에이전트의 세션
(대화 × 에이전트)으로 실행하고 대화 에이전트 세션은 그대로 둔다. 양쪽 요청 · 답에 부른 이름이 남고 답 발화자도 그 이름이다. CLI 는
`ask --conv N --agent X`. 대상 gate: E3 · E6 · E8 · E9 · E12(이 단계 몫).

## 산출 파일

- `crates/core/src/chat.rs` — `set_agent(id, Option<&str>)`. `crates/core/src/agents.rs` — `forget_sessions(name)` · `remove` 도 세션을 비운다(리뷰 4).
- `crates/net/src/proto.rs` — `Request::Message.mention`(serde default — 리뷰 1 뒤, `agent` 는 늘 대화 에이전트).
- `crates/agent/src/chat.rs` — `Compose.mention` · 오류 문구 `MENTION_ONLY`. `send_with` — 새 대화 · 자동 왕복 대화 · 빈 이름이면 `InvalidInput`(새 대화는
  대화를 만들기 전에), 대화 에이전트 이름이면 멘션 아님. `post_with` — 받은 대화 · 실행 아닌 글이면 `InvalidInput`. `send_human` — 글 `agent` ·
  프레임 `agent` = 부른 이름. `inbound_message` — 있는 대화에 다른 내 에이전트 이름이면 멘션(자동 왕복 대화 · `relay` · `by_agent` 와 함께면
  `bad_request`, 없는 에이전트는 지금처럼 `unknown_agent`), 글 `agent` 를 남긴다. `process` — 실행 대상 `asked.agent ?? 대화 에이전트` ·
  `execute` — 세션 `agent_session(대화, 이름)` 아니면 `conversations.session` · `record` — 같은 자리에 세션, 답 `agent` = 요청의 값 ·
  `inbound_reply` — 답 `agent` = 요청의 값(알림의 에이전트 이름도).
- `crates/app/src/chat_api.rs` — `POST /api/conversations/{id}/messages` 의 `agent`(멘션).
- `crates/app/src/cli.rs` — `ask --conv N --agent X` 가 대화 에이전트가 아니면 멘션(자동 왕복 대화는 지금 오류 문구 그대로, 상대 PC 목록에 없는
  이름은 사용법 오류 — 기록을 남기지 않게) · `history` 발화자 = `m.agent ?? 대화 에이전트` · 사용법 한 줄.
- 웹UI — `view.js` `mentionQuery(text, caret)` · `mentionTarget(text, names)`(해석 2 — 첫 `@이름`, 긴 이름 먼저, 앞 = 글 머리 · 공백, 뒤 = 글 끝 ·
  공백 · 문장 부호) · `authorLabel` 이 `m.agent`. `index.html` 후보 목록 `#mention_box`(listbox "에이전트 멘션") · 힌트 `#mention_hint`
  "@ 로 상대 에이전트 부르기" · 입력창 `aria-autocomplete` · `aria-controls`. `app.js` — 내가 연 대화(자동 왕복 아님)만 힌트 · 후보(상대 PC
  에이전트 — 타일과 같은 `state.peerAgents`, 없으면 대화를 열 때 묻는다), `@…` 입력 중 후보(앞부분 일치 · 대소문자 무시), ↑ ↓ 고르기 ·
  Enter · Tab 넣기(`@이름 `) · Esc 닫기 · 누르면 넣기(초점 그대로) · 그동안 Enter 는 보내지 않음 · 고른 항목 `aria-activedescendant`, 보낼 때
  `mentionTarget` 을 `agent` 로 · 다시 보내기는 원래 글의 멘션. `app.css` `.mention-box`(토큰만).
- 테스트 — `agent/tests/roundtrip.rs` `mention_runs_named_agent_own_session` · `agent/tests/relay.rs` `mention_rejected_in_relay` ·
  `app/tests/ux_api.rs` `mention_api_rules` · `app/tests/cli.rs` `ask_mention_in_conversation` · `web/view.test.mjs` `mention` ·
  `app/tests/ux_literals.rs` `focus_mention_labels` 보강("에이전트 멘션" · "@ 로 상대 에이전트 부르기").

## 설계 결정

- **멘션 판정은 보내는 쪽 · 받는 쪽 둘 다** — 보내는 쪽이 새 대화 · 받은 대화 · 자동 왕복 대화를 막고(`InvalidInput` → 400), 받는 쪽도 다른 이름이
  온 자동 왕복 대화 글을 `bad_request` 로 거절한다(상대가 옛 · 다른 구현이어도). 새 대화는 대화를 만들기 전에 거른다 — 빈 대화가 남지 않게.
- **멘션은 프레임의 새 필드 `Message.mention`**(리뷰 1 뒤 — plan 개정 4) — 처음에는 `Message.agent` 에 부른 이름을 실었으나, 첫 글이 받은 쪽에
  닿지 않은 대화(집중 · 없는 에이전트 거절 · 전송 오류)에 멘션을 쓰면 받은 쪽이 부른 에이전트로 대화를 열어 양쪽이 어긋났다. 지금은 `agent` 가 늘
  대화 에이전트 — 받은 쪽은 대화를 그 에이전트로 열고 멘션은 필드로 안다. 답(`Reply`)에는 싣지 않는다 — 보낸 쪽이 요청의 값을 답에 그대로 둔다
  (해석 4).
- **CLI 는 상대 PC 목록으로 먼저 거른다** — 없는 이름을 보내면 상대가 `unknown_agent` 로 거절해 실패 글이 남는다. 사용법 오류(종료 코드 2)로 —
  기존 `ask_conv_continues_same_conversation` 의 `--agent dev`(없는 이름) 단정이 그대로 맞는다.
- **후보에 대화 에이전트도 있다** — plan 해석 1 "후보 = 그 상대 PC 의 에이전트 목록" 그대로. 고르면 멘션이 아니다(서버가 거른다).
- **후보 목록을 못 받으면(상대 오프라인 등) 멘션 없음** — 손으로 친 `@이름` 도 목록에 있어야 멘션이다(해석 2). 글은 대화 에이전트에게 간다.

## 테스트 결과

```text
$ cargo test --workspace            (전부 ok — roundtrip 16 · relay 12 · ux_api 28 · cli 16 · ux_literals 9 …)
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.95s
$ node --test --test-reporter=tap "crates/app/web/*.test.mjs"
# pass 34
# fail 0
```

RED 확인: 테스트를 먼저 넣고 돌렸을 때 Rust(`Compose.mention` 없음 — 컴파일 오류) · view(`mentionQuery` 없음 — SyntaxError)로 실패.

눈 확인(가짜 API `mock18s3.py`, 엣지 headless 1440×900 · 다크): 보낸 대화에 힌트 "@ 로 상대 에이전트 부르기" · `@` 입력 → 후보 "guide" ·
"qa-check"(첫 항목 고름) · ↓ → `aria-activedescendant` = 둘째 · Esc 닫힘 · `@q` → "qa-check" 하나 · Enter → "배포 전 @qa-check " (보내지 않음) ·
이어 쓰고 Enter → 몸 `{text, request: false, agent: "qa-check"}` · 멘션 답 발화자 "이도윤/qa-check" · 자동 왕복 대화 · 받은 대화는 힌트 없음 ·
받은 대화에서 `@` 는 후보 없음. 페이지 오류 0.

## 리뷰

읽기 전용 리뷰 에이전트(범위 `863c8b9..8a77161`, 임시 worktree 에서 재실행 — H 0 · M 1 · L 4). 신뢰 경계(멘션으로 부를 수 있는 범위 = 새 대화로
부를 수 있는 범위) · 세션 읽기 · 쓰기 자리 · `recover()` · 자동 왕복 경로 · 잠금 · 접근성 역할은 문제 없음으로 봤다. 판단 · 조치:

| # | 지적 | 판단 | 조치 |
|---|---|---|---|
| 1 [M] | 첫 글이 받은 쪽에 닿지 않은 대화(집중 거절 등)에 멘션 → 받은 쪽이 부른 에이전트를 대화 에이전트로 삼아 대화를 연다 — 양쪽 `agent` 어긋남 | 사실(재현) | 프레임에 `mention` 필드(`agent` = 대화 에이전트 늘) — plan 개정 4. 받은 쪽은 `relay` · `by_agent` 와 함께인 멘션을 대화를 만들기 전에 거르고, 부른 이름도 등록 확인. 알림의 에이전트 = 부른 이름. roundtrip `mention_runs_named_agent_own_session` 보강(집중으로 첫 글이 막힌 대화의 멘션 → 받은 쪽 대화 에이전트 = review · 글 `agent` = guide) · relay `mention_rejected_in_relay` 프레임 갱신 |
| 2 [L–M] | `pickMention` 이 옛 `@` 자리부터 지금 커서까지만 바꿔 본문이 망가진다(커서를 이미 쓴 멘션 안으로 옮기고 Enter · 열린 채 전체 선택 뒤 Enter) | 사실(시뮬레이션) | 넣을 때 `mentionQuery` 를 다시 구해 같은 `@` 가 아니거나 선택이 있으면 닫기만, 바꾸는 끝 = 그 낱말 끝. 커서 이동(클릭 · ← → Home End)은 열린 후보만 다시 거른다(닫힌 창을 열지 않음). 눈 확인 — 다섯 경우 |
| 3 [L] | 후보 목록을 못 받으면(`null`) 다시 묻지 않고, 힌트는 보인 채 손으로 친 `@이름` 이 대화 에이전트에게 간다 | 사실(코드) | 대화를 열 때 `null` 이면 다시 묻고, 힌트는 목록을 받은 대화에서만(받으면 그때 보임) |
| 4 [L] | `agents::remove` 가 `agent_sessions` 를 지우지 않는다(`conversations.session` 도 — 이전부터) | 사실(코드) | `agents::forget_sessions` — `update`(폴더 변경) · `remove` 가 같이 쓴다. paths `schema12_focus_mention` 보강 — plan 개정 4 |
| 5 [L] | Shift+Tab 도 넣는다 · IME `keyCode 229` · 목록 스크롤바를 누르면 닫힘 · `\p{P}` 로 `@qa-check` 가 `qa` 로 잡힘(이름이 `qa` 만일 때) | 앞 셋 사실 · 넷째 명세대로(해석 2) | Shift 가 있으면 넣지 않음 · 229 도 조합 중 · 목록(ul)의 mousedown 을 막고 항목은 click. 넷째 미조치 |

고친 뒤: `cargo test --workspace` 전부 ok(roundtrip 16 · relay 12 · paths 14) · clippy 통과(받은 `message` 나머지 필드는 `MessageExtra` 별칭 — 6개
튜플 경고) · node `# pass 34`.

## 다음 단계

공통 — 검증 명령 · `check_gates` · 적대적 평가(codex 2 라운드) · 종결.

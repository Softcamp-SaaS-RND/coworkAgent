# plan — 1m.context-compact

> 근거: [intent](intent.md)(승인 2026-10-16 — 라운드 1 · 2 · 3 + 1M 지시). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `eab15be`(task/1m-context-compact — main `6e4d4a2` 위 intent). plan 승인: 2026-10-16 — 사용자 원문 "승인. 개발진행"(해석 1 ~ 13 그대로).

## 목표

대화마다 답하는 PC 의 Claude 세션 크기를 대화 머리에 "컨텍스트 42k / 1M" 으로 보이고, 그 표시를 누르면 압축(요약 후 새 세션) · 비우기(세션만)를 고른다.
80% 를 넘으면 답하는 PC 가 실행 직후 자동 압축한다. 묻는 쪽이 누르면 상대 PC 에 요청하고(꺼져 있으면 보관했다 보냄), 자동 왕복 대화는 두 PC 세션을 함께 처리한다.

### 성공 기준(intent 사본)

- A1: 답을 받으면 대화 머리에 그 대화의 컨텍스트 크기(토큰 / 1M)가 보이고, 크기를 모르면 "알 수 없음" 이다.
- A2: 머리 표시를 누르면 압축 · 비우기 메뉴가 열리고, 확인 뒤 실행된다. 실행 중에는 꺼져 있다.
- A3: 압축하면 답하는 PC 에서 요약이 만들어지고 다음 글은 그 요약으로 시작한 새 세션이 이어 받는다. 요약이 양쪽 대화에 보인다.
- A4: 비우면 다음 글은 새 세션(앞 대화 기억 없음)으로 실행되고, 대화 이력은 양쪽 그대로이며 구분선이 보인다.
- A5: 0.4.0 상대와 대화 · 주고받기가 그대로 되고, 그 상대에는 크기 "알 수 없음" · 압축 · 비우기는 안내와 함께 거절된다.
- A6: 80% 를 넘으면 자동 압축되고(설정으로 끔), 상한 60% · 80% 를 넘으면 표시 색이 바뀐다.
- A7: 상대가 꺼져 있을 때 누른 압축 · 비우기는 상대가 켜진 뒤 실행된다.
- A8: 자동 왕복 대화에서 한쪽 세션이 (자동 · 수동) 압축되거나 비워지면 다른 쪽 PC 세션도 같이 된다.
- A9: 검증 명령(`cargo test --workspace` · clippy · node 시험 · `check_gates` · `check_ddd_docs`) 통과.

## 코드 사실(실측, 기준 `eab15be`)

- 실행 — `agent/src/chat.rs` `execute`(2969): 세션 = 멘션이면 `chat::agent_session`, 아니면 `conv.session` → `Resume` · 없으면 `New`. 프롬프트 = `with_notes`(3075 —
  `<대화 메모>` + `<원격 본문>`). `record`(3138) 한 트랜잭션에서 세션 저장(`set_agent_session` · `set_session`) · 답 행(`add_reply` — turns · duration_ms · model ·
  effort) · 보낼 줄(outbox) 기록. 상대에게 `Reply`(상대가 물음) 또는 `Post{author:"agent"}`(이 PC 사람이 물음).
- 실행기 — `runner.rs` `parse_output`(180)은 `result` · `session_id` · `is_error` · `num_turns` · `duration_ms` 만 읽는다. `RunOutput` 은 시험 7곳에서 구조체 글자로 만든다.
- 실측(2026-10-16, Claude Code 2.1.299, 빈 폴더 `-p` stream-json): 결과 `usage.iterations[]` = 실행 안 API 호출마다 `input_tokens` · `cache_creation_input_tokens` ·
  `cache_read_input_tokens` · `output_tokens`(결과 `usage` 자체는 실행 합계), `modelUsage.{모델}.contextWindow` = 1000000(`sonnet` · `haiku` · `opus` 모두). 빈
  실행도 약 30k(시스템 프롬프트 · 도구 정의).
- 대화마다 작업자 하나(`wake` 2574 → `process` 2609)가 `chat::pending`(exec=1 · queued/running, id 순)을 하나씩 — 한 대화의 실행은 겹치지 않는다.
- 자동 왕복 — 연 쪽(Dir::Out)도 자기 `conv.session` 으로 자기 에이전트를 돌린다(`relay_forward` 2801). 두 PC 세션은 `conv.uuid` 로만 이어진다.
- 통신 — `Request` 는 `tag = "type"`. **0.4.0 은 모르는 `type` 을 읽지 못해 응답 없이 연결을 닫는다**(`read_frame` 오류 → `Node::handle` Err) → 보내는 쪽은 전송
  오류로 보고 24시간 다시 보낸다. 모르는 필드는 무시(`#[serde(default)]`). 새 종류는 `Node::answer`(node.rs 492 목록 · 627 갈래)와 `Chat::handle`(1559)에 둘 다 넣는다.
- 보낼 줄 — `outbox` 종류는 글자(`chat.rs` 62–75), `deliver`(833) → `send_to` → `acked` → `settle`(895). `Ping` → `Pong{focus}`(node.rs 598 · 847).
- 화면 — 대화 머리 `index.html` 192–221(`#conv_more_btn` · `#saved_btn` · `#detail_btn`), `app.renderConvHead`(1714), `messageEl`(1748), 구분선 선례
  `.day-break`(`messageEls` 1917). 글 목록 API `GET /api/conversations/{id}/messages` → `message_json`(chat_api.rs 349).
- 설정 — 불리언 선례 `core/settings.rs` `AUTOSTART_KEY`(기본 켜짐), `notify.rs` `SettingsPatch`(498) · `settings_json`(539), 화면 `#run` 카드 · `PREFS`.
- 스키마 18(`db.rs` 5).

## intent 대비 해석 (plan 승인 대상)

1. **크기 = 마지막 호출의 창 사용량** — `usage.iterations` 마지막 항목의 input + cache_creation + cache_read + output(없으면 결과 `usage` 같은 네 값). 상한은
   1M 고정(intent 결정 — `contextWindow` 는 읽지 않는다). 압축 · 비우기 직후는 0 으로 본다(다음 답부터 새 값).
2. **표시 위치 · 모양** — 대화 머리 "북마크" 단추 왼쪽에 단추 `#ctx_btn`: 아이콘 + "42k / 1M"(1000 미만은 그대로, 1M 이상은 "1.0M"). 모르면 "컨텍스트 알 수 없음".
   60% 이상 주황 · 80% 이상 붉은색(글자 · 테두리). 대화 크기 = **그 대화의 세션 전부 중 가장 큰 값**(대화 에이전트 · @멘션 에이전트 · 자동 왕복 양쪽 — 마지막
   압축 · 비우기 구분선 뒤 답들 중 에이전트별 마지막 값의 최대).
3. **메뉴** — 단추를 누르면 메뉴 "압축 — 요약 후 새 세션" · "비우기 — 세션만, 이력은 그대로". 고르면 확인 한 번(`#float` 확인 꼴 — 그룹 삭제와 같음): 압축은
   "답하는 PC 의 Claude 사용량을 씁니다" 안내. 그 대화에 실행 중 · 대기 글(또는 진행 중 압축)이 있으면 두 항목 꺼짐. 세션이 없는 대화(답이 아직 없음)도 꺼짐.
4. **압축 = 요약 1회 + 다음 실행에 주입** — 세션마다 그 세션을 `Resume` 해 요약 프롬프트(아래)를 실행 → 요약 글을 저장 → 세션을 비움. 다음 실행은 새 세션이고,
   첫 프롬프트 앞에 `[이전 대화 요약]\n<대화 요약>\n…\n</대화 요약>` 을 붙인다(요약은 한 번 쓰면 지움). intent "요약을 첫 입력으로 새 세션" 을 실행 1회로 —
   새 세션을 빈 실행으로 미리 열지 않는다(사용량 절약). 요약 프롬프트: "지금까지 이 대화를 다음 세션이 이어 받을 수 있게 요약하라 — 오간 질문 · 답의 요점, 확인한
   파일 · 사실, 남은 질문. 한국어 2000자 이내. 비밀은 싣지 않는다." 요약 실행도 가드 · 읽기 전용 도구 · 같은 model · effort. 요약은 8KB 에서 자른다(통신 본문 한도
   16KB 안).
5. **비우기** — 그 대화 세션 전부를 지운다(`conversations.session = NULL` · `agent_sessions` 행 삭제 · 남은 요약 지움). 실행 없음.
6. **구분선 글** — 압축 · 비우기는 대화에 글 한 줄로 남는다(새 author `context`, 열 `ctx_op` = `compact` · `auto_compact` · `clear`). 화면은 말풍선이 아니라
   구분선: "컨텍스트 압축 — 요약 ▸"(누르면 요약 펼침) · "자동 압축(80%) — 요약 ▸" · "컨텍스트 비움". 진행 중이면 "압축 중…", 실패면 사유. 양쪽 PC 에 같은
   uuid 로 남는다. 메모(`notes_between`) · 안 읽음 · 대화방 미리보기에는 넣지 않는다.
7. **순서** — 압축 · 비우기도 그 대화 작업자 줄(`pending`)에 들어가 앞의 실행이 끝난 뒤, 뒤의 글보다 먼저 처리된다(id 순). 실행 중 단추를 끄는 것은 화면 쪽 보호.
8. **누가 어디서** — 이 PC 에 그 대화 세션이 있으면(Dir::In, 또는 자동 왕복 연 쪽) 이 PC 에서 처리한다. 상대 PC 에 세션이 있을 수 있으면(Dir::Out, 또는 자동
   왕복) 상대에 요청을 보낸다. 자동 왕복은 둘 다 — 한쪽에서 누르거나 자동 압축이 걸리면 자기 세션을 처리하고 상대에도 보낸다. **받은 요청은 다시 넘기지 않는다**(되풀이 없음).
9. **새 통신 종류 하나** — `Request::Context { conv, msg, op, text, ctx }`: `op` = `compact` · `auto_compact` · `clear`(해 달라 — `text` 빈 값) · `done`(처리
   결과 — `text` = 요약 · `ctx` 쓰지 않음) · `failed`(`text` = 사유 코드). `msg` = 구분선 글 uuid(양쪽 같음 — 멱등). 답 · 글 프레임 `Reply` · `Post` 에
   `#[serde(default)] ctx: Option<i64>`(그 답의 창 사용량)를 더한다 — 0.4.0 은 무시하고, 0.4.0 이 보낸 답은 크기 없음("알 수 없음").
10. **0.4.0 상대 보호** — `Pong` 에 `#[serde(default)] ctx: bool`(이 기능을 아는가)을 더한다. `Context` 줄을 보내기 전에 그 상대에 `Ping` 을 먼저 해, `ctx`
    가 거짓이면 보내지 않고 그 구분선 글을 "상대가 0.5.0 이상이어야 합니다"(코드 `unsupported`)로 실패 처리한다. Ping 이 닿지 않으면 보통 줄처럼 다시 보낸다
    (꺼짐 — A7). 이렇게 0.4.0 이 연결을 닫는 일 · 24시간 재전송을 피한다.
11. **자동 압축** — 답하는 PC 가 `record` 직후 그 실행의 크기가 800k(80%) 이상이고 설정 `auto_compact`(기본 켜짐)면 그 대화에 `auto_compact` 구분선 글을
    줄에 넣는다(다음 글보다 먼저). 자동 왕복이면 상대에도 `Context{op:auto_compact}`. 설정 → 실행 카드 체크 "자동 압축 — 컨텍스트 80%(800k)를 넘으면 요약 후
    새 세션" — 그 PC 의 설정(세션 · 사용량 주인).
12. **꺼진 상대 · 묻는 쪽 표시** — 묻는 쪽은 구분선 글을 먼저 "보내는 중"(outbox 종류 `context`)으로 남기고, 상대의 `done` 이 오면 요약 · 완료로 바꾼다.
13. **스키마 19** — `messages.ctx_tokens INTEGER` · `messages.ctx_op TEXT`, 새 표 `ctx_seeds(conversation_id, agent, text, PRIMARY KEY(conversation_id, agent))`
    (`agent` 빈 글자 = 대화 에이전트). 올리기 전 사본(기존 `backup_before_migration`). 0.4.0 은 19 를 열지 않는다(기존 규칙).

## 설계

- `an_net::proto` — `Request::Context` · `Reply.ctx` · `Post.ctx` · `Pong.ctx`. `node.rs` — 답 목록 · 갈래에 `Context`, Pong 에 `ctx: true`, `ping` 이 `ctx` 를
  돌려준다.
- `an_core` — `db.rs` 스키마 19 · `chat.rs` `add_context`(구분선 글) · `set_context_done` · `ctx_seed` / `set_ctx_seed` / `take_ctx_seed` · `clear_sessions` ·
  `add_reply` 에 `ctx_tokens` · `message` 행에 `ctx_tokens` · `ctx_op` · `notes_between` · 미리보기 · 안 읽음에서 `context` 빼기. `settings.rs` `AUTO_COMPACT_KEY` ·
  `auto_compact()`(기본 켜짐).
- `an_agent` — `runner.rs` `RunOutput.ctx_tokens`(`Default` 파생 — 시험 글자 `..Default::default()`) · `parse_output` 이 `usage` 를 읽음 · `SUMMARY_PROMPT`.
  `chat.rs` — `Chat::context(conv, op)`(화면 요청 → 이 PC 처리 · 상대 요청), `inbound_context`, `process` 가 `context` 글 갈래(압축 = 세션마다 요약 실행 → 요약 저장
  · 세션 비움 / 비우기), `execute` 가 새 세션이면 요약 주입, `record` 뒤 자동 압축 판정, `deliver` 의 `context` 줄 Ping 먼저, `settle` 거절 → 구분선 실패.
- `app` — `chat_api.rs` `POST /api/conversations/{id}/context`(`{op: "compact"|"clear"}`, 틀린 값 400) · `message_json` 에 `ctx_tokens` · `ctx_op`. `notify.rs`
  `SettingsPatch.auto_compact` · `settings_json`.
- 웹 — `view.js` `contextSize(messages)`(구분선 뒤 에이전트별 마지막 값의 최대) · `contextLabel(n)`("42k / 1M" · 단계 ok/warn/high) · `ctxMenu(busy, has)`.
  `app.js` 머리 `#ctx_btn` · 메뉴 · 확인 · 구분선 글 그리기(요약 펼침). `index.html` · `app.css`(아이콘 `gauge` 하나 더 · 단계 색). 설정 `#run` 체크 `auto_compact`.
- 시험 — `runner.rs` usage 읽기(iterations · 없음), `roundtrip`/새 `context.rs`(가짜 실행기 — 압축 → 다음 실행 새 세션 + 요약 주입 · 비우기 → 새 세션 · 묻는 쪽
  요청 → 상대 처리 → 양쪽 구분선 · 꺼진 상대 보관 뒤 전달 · Pong ctx 거짓 → unsupported · 자동 압축 800k · 설정 끔 · 자동 왕복 연동 · 받은 요청 되넘기지 않음),
  `ux_api` 설정 · API 400, node `contextSize` · `contextLabel` · `ctxMenu`, `ux_literals` 문구.
- 문서 — 사용자 가이드(대화 — 컨텍스트) · an_app · 개발자 가이드(프로토콜 `context` · 스키마 19) · PRODUCT · README · 캡처(대화 머리).

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 통신 · 스키마 19 · 실행기 크기 · 압축 · 비우기 · 자동 압축 · 자동 왕복 연동 · 0.4.0 보호 | Rust 시험 | E1 · E2 · E7 |
| 2 | API · 설정 · 화면(머리 단추 · 메뉴 · 구분선) · 문서 · mock 화면 · 실제 claude 요약 1회 | node · 문구 시험 · mock 캡처 | E3 ~ E9 |

단계 끝 mock(가상 데이터) 화면을 사용자에게 보인다.

## 평가 항목

- [ ] E1: A1·A3·A4·A5·A6·A7·A8·A9 — 워크스페이스 테스트 전부 통과(usage 읽기 · 압축 · 비우기 · 요약 주입 · 꺼진 상대 보관 · unsupported · 자동 압축 · 설정 · 연동 · API 400)
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A9 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2·A6·A9 — 화면 로직 node 테스트 103건 이상 · 실패 0(`contextSize` · `contextLabel` 단계 · `ctxMenu` 꺼짐)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (10[3-9]|1[1-9][0-9])\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A4 — 웹UI 문구 · 자리(`#ctx_btn` · 압축 · 비우기 메뉴 문구 · 구분선 문구 · 설정 `auto_compact`)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A9 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: pending
- [ ] E6: A9 — README 사용자 관점
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: pending
- [ ] E7: A9 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: pending
- [ ] E8: A1·A2·A4·A6 — mock 화면(가상 데이터, 다크 · 라이트): 머리 "컨텍스트 n / 1M" · 60 · 80% 색 · 알 수 없음 · 메뉴 · 확인 · 실행 중 꺼짐 · 압축 · 비움 구분선 · 요약
  펼침 · 설정 자동 압축 체크 · JS 오류 없음.
  EVIDENCE: pending
- [ ] E9: A3 — 실제 claude 1회(이 PC · 임시 폴더 에이전트 · 비파괴 — 실제 상대 · 이력은 건드리지 않음): 실행 → 크기 읽힘 → 요약 실행 → 새 세션 첫 실행에 요약이 들어가
  앞 대화 내용을 답한다.
  EVIDENCE: pending

## 종결 시 할 일

- masterPlan: D19 개정(세션 압축 · 비우기 · 자동 압축 80% · 상한 1M) · 프로토콜 `context` · 스키마 19 기록 · 개정 기록. 원장 없음(예정).
- 실제 두 PC 확인(상대 압축 · 꺼진 상대 · 자동 왕복 연동 · 0.4.0 상대)은 19 인수 TC 로.
- 이 PC 반영(릴리스 빌드 — 스키마 19 로 올림, 사본 남음). GitHub 반영 · 릴리스(0.5.0)는 사용자 확인 뒤.

## 개정 기록

- 없음

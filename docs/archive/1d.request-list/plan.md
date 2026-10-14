# plan — 1d.request-list

> 근거: [intent](intent.md)(승인 2026-10-13 — 라운드 2). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `31ee236`(main — Merge chore/virtual-paths, `task/1d-request-list` 분기점). plan 승인: 2026-10-13 — 사용자 원문 "승인. 개발 진행".

## 목표

에이전트 설정을 단순하게 하고, 실행하지 않는 **요청**을 목록으로 다룬다. 모델은 `opus` · `sonnet` · `haiku` 드롭다운, 탐색 도구(읽기 전용 5개)는
모든 실행에 고정하고 도구 체크 칸을 없앤다. 입력창에서 고른 **요청** 글은 받는 PC 에이전트를 실행하지 않고 받는 PC 의 요청 목록에 '대기' 로 남는다.
받은 사람이 수용 · 반려 · 완료(한 줄 메모)로 답하면 그 상태가 outbox 로 요청자에게 간다. 왼쪽 메뉴 '요청' 화면이 받은 · 보낸 요청을 상태로 거른다.

### 성공 기준(intent 사본)

- A1: 모델은 드롭다운에서만 고르고(`opus` · `sonnet` · `haiku`, 상대별은 "기본값" 포함), 목록 밖 값은 저장 · 실행 거부, 이미 저장된 목록 밖
  값은 마이그레이션으로 기본값이 된다.
- A2: 프로필 화면 · API 에 도구 칸이 없고, 어떤 프로필이든 실행 인자 `--tools` · `--allowedTools` 가 읽기 전용 5개다.
- A3: 요청 글은 받는 PC 에이전트를 실행하지 않고(대기열 · 실행 기록 없음) 받는 PC 요청 목록에 '대기' 로 남는다. 집중 중에도 받는다.
- A4: 받은 사람만 상태를 대기 → 수용 · 반려, 수용 → 완료로 바꾸고(메모 한 줄), 바뀐 상태 · 메모가 요청자 쪽 대화 · 요청 목록에 보인다 —
  요청자가 꺼져 있어도 outbox 로 나중에 간다.
- A5: 왼쪽 메뉴 '요청' 화면이 받은 요청 · 보낸 요청을 상태로 걸러 보여 주고, 항목에서 그 대화로 간다. '내 활동' 의 요청 거르기는 없다.
- A6: 이전 이력의 완료 표시된 요청은 '완료', 나머지 요청은 '대기' 로 보인다.
- A7: 루트 `PROJECT.md` 검증 명령이 통과한다.

## intent 대비 해석 (plan 승인 대상)

1. **모델 목록 = `an_core::run_opts::MODELS = ["opus", "sonnet", "haiku"]`** — `valid_model` 은 이 목록 포함 여부로 바꾼다(지금은 글자 규칙). 전역
   설정 `claude_model` · 상대별 `peers.model` 저장 API 가 목록 밖 값을 400, 실행 직전 `resolve` 는 목록 밖 값을 쓰지 않고 기본값(지금 규칙 그대로 — D22 의 "실행 실패" 는 설정을 못 읽는 DB 오류). 상대별 빈 값 =
   기본값. 웹UI 는 `<input list>` · `<datalist id="model_names">` 를 `<select>` 로(상대별은 첫 항목 "기본값(…)" = 빈 값).
2. **스키마 14 마이그레이션** — ① `settings.claude_model` 이 목록 밖이면 지운다(없으면 기본 `sonnet`) · `peers.model` 이 목록 밖이면 `NULL`.
   ② `messages` 에 `req_state TEXT`(`wait` · `accepted` · `rejected` · `done`, 요청 글만) · `req_note TEXT` · `req_at INTEGER` 를 더하고, `request = 1` 인
   글은 `done_at` 이 있으면 `done`(`req_at = done_at`), 없으면 `wait`. ③ `outbox` 의 `kind = 'done'` 행은 같은 자리에서 `ReqState` 프레임(완료 → `done` ·
   취소 → `wait`)으로 바꾸고 그 요청 글에 잇는다(개정 3 — 처음엔 "지운다"). `done_at` 열은 남기되 읽지 않는다(사용자 데이터 보존).
3. **도구 고정** — `runner::claude_args` 의 `--tools` · `--allowedTools` = `agents::READ_ONLY_TOOLS.join(",")`(5개, 프로필 값 무시). `Agent.tools` 필드 ·
   `agents::add` · `update` 의 도구 인자 · `DEFAULT_TOOLS` · `read_only` · `unknown_tools` · API `checked_tools` 를 지운다(DB `agents.tools` 열은 남김 —
   새 행은 5개를 적어 둔다). 에이전트 JSON 에 `tools` 없음, 등록 · 수정 요청의 `tools` 는 422(모르는 필드 — axum 의 본문 거절, 수정은 이미 `deny_unknown_fields`, 등록도 맞춘다). 웹UI 의 `toolChecks` ·
   `chosenTools` · 도구 체크를 지우고(`READ_ONLY_TOOLS`(view.js)는 안내 문구에 쓴다) 등록 · 편집 · 연결 목록 자리에 안내 한 줄 "탐색 도구(읽기 전용) 항상 사용 —
   Read · Grep · Glob · LS · NotebookRead".
4. **요청 보내기 — 실행 없는 사람 글** — `Compose.request = true` 면:
   - 보낸 대화(`out`, 새 대화 · 이어 쓰기 모두)는 지금처럼 `Request::Message{request: true}`(받는 쪽에 대화가 없을 수 있어 `Post` 가 아니다). 내 글은
     `exec = false`, outbox 종류는 `note`(받으면 `done`, 거절이면 `error` — 기존 `NOTE` 처리 그대로). 멘션과 함께면 400(`MENTION_ONLY` 와 같은 꼴 —
     "요청은 멘션 없이"). `post_with` 에서 `out` 대화의 요청은 `to_agent` 와 무관하게 이 길로.
   - 받은 대화(`in`)는 `post_with` 의 미러 길에서 `to_agent = false` 로 강제(`Post{exec: false, request: true}` — 내 에이전트 실행 없음).
   - CLI(`via = cli`) 글은 요청이 될 수 없다 — 400 "CLI 는 질의만"(CLI 는 답을 기다린다 — 요청은 답이 없다, intent 제외).
5. **요청 받기** — `inbound_message` 에서 `request = true` 면: 멘션 · `relay` · `by_agent` 가 있으면 `bad_request`. **집중 검사 앞에서** 갈라 집중 중에도
   받는다. 글은 사람 글 `mine = false` · `exec = false` · `state = done` · `req_state = wait` 로 저장(대기열 등록 · `wake` · `tell_waiting` 없음 — 실행 기록
   없음), 같은 uuid 재전송은 `Ok`(한 번만). `inbound_post` 의 사람 글은 `request && exec` 면 `bad_request`, 요청이면 `req_state = wait`. 알림 =
   `MessageKind::Ask`(새 종류 — 알림 설정 `notify_request` 를 따른다).
6. **상태 바꾸기 = `Chat::set_req_state(id, state, note)`**(`mark_done` 대체) — 받은 요청(`author = human` · `mine = false` · `request`)만, 옮김은
   `wait → accepted | rejected` · `accepted → done` 만(그 밖 400 "대기 → 수용 · 반려, 수용 → 완료만"). 메모는 앞뒤 공백을 지우고 0 ~ 200자 · 줄바꿈 ·
   제어 문자 없음(`run_opts::clean_label` 꼴). 같은 트랜잭션에 `req_state` · `req_note` · `req_at = now` 와 outbox 행(종류 `req_state` — 개정 2:
   바뀔 때마다 한 행, 합치지 않음) — 1c `stage` · `dispatch`. 대화 차례(`turn`) 안에서. 응답 = 바뀐 글 + 경고(상대가 거절했을 때만).
7. **프레임 `Request::ReqState{conv, msg, state, note}`**(`Request::Done` 삭제) — 요청자 쪽 `inbound_req_state`: 그 상대 소유 대화의 내 요청
   사람 글(`mine` · `request`)에만, `state` 는 넷 중 하나 · 메모 200자 이하가 아니면 `bad_request`, 없는 글이면 `unknown`. 옮김 검사는 하지 않는다(받은
   쪽이 정하고 outbox 가 순서 · 마지막 값을 지킨다). 알림 = `MessageKind::AskState`(`notify_request` 따름, 미리보기 "요청 수용 — {메모}" 꼴).
   `ids_ok` · node 의 처리기 목록에서 `Done` 자리를 `ReqState` 로.
8. **API** — `PATCH /api/messages/{id}` 의 `done` → `req_state`(+ `note`), `saved` 와 함께 와도 한 트랜잭션. 메시지 JSON 의 `done` →
   `req_state` · `req_note` · `req_at`(요청 아니면 `null`). `GET /api/messages` 에 `req`(`wait` | `accepted` | `rejected` | `done`, `ask=request` 와
   함께 — 건수 제한 전에 거름). `GET /api/activity` 의 `kind=query` · `request` 는 지운다(400 목록에서도 뺀다).
9. **웹UI** — `#requests` 화면(사이드바 '요청', 받은 요청 중 대기 수를 배지로): 탭 받은 요청 · 보낸 요청(`side=theirs` · `mine`), 상태 거르기(전체 ·
   대기 · 수용 · 반려 · 완료), 행 = 상대 · 대화 제목 · 본문 앞부분 · 상태 · 메모 · 시각, 누르면 `#c/{대화}/m/{글}`. 받은 요청 행 · 대화의 메시지 작업
   막대: 대기 → "수용" · "반려", 수용 → "완료"(누르면 메모 한 줄 칸 + 확인). 대화의 요청 태그 = "요청 · 대기|수용|반려|완료" + 메모 줄. 입력창에서
   요청을 고르면 안내 "실행 없이 상대 요청 목록에 등록 — 상대가 수용 · 반려합니다" · 멘션을 보내지 않는다. '내 활동' 종류에서 질의 · 요청 항목을 뺀다.
   실행 상세의 '요청' 절은 상태 문구로.

## 설계

### `an-core`
- `run_opts.rs` — `MODELS` · `valid_model`(목록).
- `db.rs` 스키마 14(해석 2).
- `agents.rs` — 도구 필드 · 인자 · 상수 정리(해석 3).
- `chat.rs` — `Message.req_state` · `req_note` · `req_at`(MSG_COLS), `set_request` 가 `req_state = wait` 도, `set_req_state` · `req_next(from, to)`,
  `messages_across` 에 상태 거르기, `ActivityFilter::Query` · `Request` 삭제, `set_done` · `done_at` 필드 삭제.

### `an-net`
- `proto.rs` — `Request::ReqState` · `Done` 삭제. `node.rs` — `MessageKind::Ask` · `AskState`, 처리기 목록.

### `an-agent`
- `runner.rs` — 도구 고정. `chat.rs` — 해석 4 ~ 7(`REQ_STATE` outbox 종류, `DONE` 삭제).

### `an-app`
- `chat_api.rs` · `peers_api.rs` · 설정 저장 — 해석 1 · 3 · 4 · 8. `notify.rs` — `Ask` · `AskState` → `notify_request`.
- 웹UI — `index.html`(모델 `<select>` · 도구 칸 삭제 · `#requests` 화면 · 활동 종류 정리), `view.js`(`MODELS` · `reqStateText` · `messageActions` ·
  `askTag` · `sideNav` 배지 · `route('requests')` · `reqTransitions`), `app.js`(요청 화면 · 상태 바꾸기 · 입력창 안내).

### 문서
- 매뉴얼: user-guide(요청 · 모델 · 도구 절) · an_core · an_net · an_agent · an_app.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 모델 드롭다운 · 탐색 도구 고정 · 스키마 14 | `models_tools` 시험 · runner 시험 갱신 | E1 · E2 · E3 · E6 · E7 |
| 2 | 요청 = 실행 없음 · 상태 바꾸기 · `ReqState` 동기화 | `requests` 시험(core · agent) | E1 · E2 · E4 · E5 |
| 3 | 요청 화면 · 대화 태그 · 입력창 · 내 활동 정리 · 매뉴얼 · 실화면 | node 시험 | E1 ~ E3 · E8 |

## 평가 항목

- [ ] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A7 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A4·A5·A7 — 화면 로직 node 테스트 47건 이상 · 실패 0(기존 42 + 모델 목록 · 요청 상태 문구 · 작업 막대 옮김 · 사이드바 '요청' 배지 · `#requests` 경로)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (4[7-9]|[5-9][0-9])\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A4·A6 — 요청 저장 단위 4건(스키마 14 가 완료 표시 → `done` · 나머지 → `wait` · 목록 밖 모델 지움 / 옮김 규칙 넷(대기→수용·반려, 수용→완료, 그 밖 거부) / 메모 규칙(200자 · 줄바꿈 거부) / 목록 상태 거르기)
  CHECK: cargo test -p an-core --test requests
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A3·A4 — 두 PC 통합 6건(새 대화 요청은 받는 쪽 실행 · 대기열 없음 · '대기' / 이어 쓰기 · 받은 대화(`in`)의 요청도 실행 없음 / 집중 중에도 요청은 받고 질의는 거부 / 수용 · 메모가 요청자 글에 도착 / 요청자가 꺼져 있을 때 바꾼 상태가 켜지면 도착(outbox) / 쓴 쪽 · 다른 상대 · 위조 `ReqState` 는 거부 · 멘션 · CLI 요청은 400 / 요청자에 오래 닿지 않아 포기하면 그 글에 표시 · 같은 상태로 다시 알리기 — 개정 3)
  CHECK: cargo test -p an-agent --test requests
  EXPECT: test result: ok. 7 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A1·A2 — 앱 API 4건(전역 · 상대별 모델 목록 밖 400 · 목록 안 · 빈 값(상대별) 저장 / 에이전트 JSON 에 `tools` 없음 · 등록 · 수정에 `tools` 를 보내면 422 / `GET /api/messages?ask=request&req=wait` 거르기 / `GET /api/activity?kind=request` 400)
  CHECK: cargo test -p coworkagent-app --test models_tools
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A2 — 실행 인자: 어떤 프로필 값이든 `--tools` · `--allowedTools` = `Read,Grep,Glob,LS,NotebookRead`(runner 시험 8건)
  CHECK: cargo test -p an-agent --test runner
  EXPECT: test result: ok. 8 passed; 0 failed
  EVIDENCE: pending
- [ ] E8: A1·A2·A3·A4·A5 — 실화면(헤드리스 Edge · 실행 중 앱 2개): 설정 · 연결 목록 모델 드롭다운 · 에이전트 화면 도구 칸 없음 · 요청 보내기 → 받는 쪽 '요청' 배지 · 목록 '대기' → 수용(메모) → 보낸 쪽 '수용' · 메모 · 콘솔 오류 0(사람 확인은 19 TC)
  EVIDENCE: pending

## 종결 시 할 일

- masterPlan: S1 #18 보강 개정(요청 = 실행 없음 · 목록 · 상태 4단계 · 받은 쪽만), D21 보강(도구 칸 없음), D22 보강(모델 = 목록 셋 · 도구 고정),
  D31 보강(읽기 전용 5개를 모든 실행에), D17 개정 보강(집중 중에도 요청은 받음), D32 적용(`ReqState`), Phase 1 오버플로 줄에 `1d.request-list`.
- PRODUCT 도메인 규칙(요청 · 모델 · 도구), 매뉴얼.
- 19 acceptance TC — 요청 보내기 · 수용 · 반려 · 완료 · 꺼진 요청자 · 모델 드롭다운 · 도구 칸 없음.
- GitHub 반영(push)은 사용자 확인 뒤.

## 개정 기록

- 개정 1(2026-10-13, Stage 1 착수 — 코드 대조): 해석 1 의 "실행 실패" → 기존 `resolve` 규칙(목록 밖 값은 기본값, DB 오류만 실패). 해석 3 의
  `tools` 입력 "무시" → 422(수정 API 가 이미 모르는 필드를 거절 — 등록도 맞춤). E6 문구 같이.
- 개정 2(2026-10-13, Stage 2 구현): 해석 6 의 보내기 행 "키 = 그 글 — 마지막 값만" → 바뀔 때마다 한 행(`stage` · `dispatch` 그대로, 키 없음).
  이유: 키로 합치면 요청자가 꺼져 있는 동안 '수용' 을 건너뛰고 '완료' 만 받는다 — 상태 단계마다 메모가 있어 중간 값도 요청자에게 의미가 있다.
  outbox 는 대화 순서를 지키므로 마지막 값은 그대로 마지막에 닿는다.
- 개정 3(2026-10-13, 적대적 평가 라운드 1/2 — high 1 · medium 1 사실): ① 스키마 14 는 못 보낸 옛 완료 알림(`kind = 'done'`)을 지우지 않고 같은
  자리 · 순서로 `ReqState` 프레임(완료 → `done` · 취소 → `wait`)으로 바꾸고 그 요청 글(`msg_id`)에 잇는다. ② 상태 보내기 행을 그 요청 글에 잇고
  (`msg_id`), 포기하면 그 글 `refused = unreachable` — 화면 "요청자에게 상태를 알리지 못함" · 작업 "다시 알리기"(같은 상태 · 메모를 다시 보냄, 이때만
  같은 상태로 옮김 허용 · 표시를 지움). E5 6 → 7건.
- 개정 4(2026-10-13, 적대적 평가 라운드 2/2 — high 1 · medium 1 사실): ① 요청자가 상태 알림을 거절하면(`unknown` 등 — 바로 보낼 때 · 줄이 보낼 때
  모두) 그 받은 요청 글에 거절 코드(`refused`) — 포기와 같이 "다시 알리기" 로 같은 상태를 다시 보낸다(같은 상태 옮김은 `refused` 가 있을 때만).
  ② 요청자는 같은 상태 · 메모를 다시 받으면(응답 유실 · 다시 알리기) 시각 · 알림을 바꾸지 않고 `Ok`. E5 건수 그대로(7 — 기존 시험에 시나리오 추가).

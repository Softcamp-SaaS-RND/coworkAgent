# plan — 10.slack-ui

> 근거: [intent](intent.md)(승인 2026-10-11 — 자동 진행 위임 + 확인 답 4건). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).

## 목표

웹UI 를 시안 A 3단 + C 타임라인으로, 화면 로직은 `node --test` 로 시험하는 순수 함수 모듈로, 답에 실행 정보 · 원 요청 대응을 싣는다.

### 성공 기준(intent 사본)

- A1: 웹UI 첫 화면이 시안 A 3단(PC 레일 · 사이드바 · 대화 영역)이고, 지금의 관리 기능(내 정보 · 피어 · 에이전트 등록 · 알림 설정)은
  모두 레일 하단 "설정" 화면에서 쓸 수 있다. 트레이 "내 정보 보기" 는 그 화면의 내 정보를 연다.
- A2: 레일에서 PC 를 고르면 사이드바 대화가 그 상대의 것만 남고, 새 대화 · 이어 쓰기 · 대화 전환이 된다. 늦게 온 다른 대화의 응답은
  화면에 그려지지 않는다.
- A3: 대화 머리글에서 C 타임라인을 열면 요청마다 요청 → 실행(턴 · 시간) → 응답이 시각 · 상태와 함께 보인다(양쪽 PC 모두).
- A4: 답에 실행 정보(`num_turns` · `duration_ms`)와 원 요청 대응이 실리고 양쪽 DB 에 남는다(스키마 5). 실행 정보가 없는 출력 · 이전
  버전 상대도 받는다.
- A5: 화면 로직의 자동 테스트가 `node --test` 로 돌고 루트 `PROJECT.md` 검증 명령에 들어간다.
- A6: 원격에서 온 이름 · 본문은 여전히 텍스트로만 그린다(`innerHTML` 없음).
- A7: 루트 `PROJECT.md` 검증 명령(테스트 · 경고 0 · 의존 경계 · JS 테스트)이 통과한다.

### intent 결정 · 제외(사본)

- 분할 10 → 11 → 12 → 19 · 관리 화면 = 레일 하단 설정 → 메인 영역 · JS 테스트 = `node --test` 순수 함수 · 타임라인 = 있는 데이터 +
  턴 · 시간 · 레일 = 상대별 거르기 · 레일 `+` = 초대 붙여넣기 · 대화 제목 = 첫 사람 메시지 앞부분 · 받은 대화 입력창은 막음(11).
- 제외: 사람 발화(11) · 자동 왕복(12) · 도구별 횟수 · 이력 검색 · 터미널 열기 · 실제 브라우저 확인(D28) · 자동 발견(Phase 2).
- 가정(미해소): `claude` JSON 의 `num_turns` · `duration_ms` 이름은 공식 문서 기준 — 없으면 `None`, 화면은 표시하지 않는다.

## 설계

### 저장(스키마 5, `an-core`)

```sql
ALTER TABLE messages ADD COLUMN reply_to INTEGER;     -- 답(agent)이 대응하는 사람 메시지의 로컬 id
ALTER TABLE messages ADD COLUMN turns INTEGER;        -- 실행 턴 수(num_turns)
ALTER TABLE messages ADD COLUMN duration_ms INTEGER;  -- 실행 시간(duration_ms)
```

- `Message` 에 `created_at`(유닉스 초) · `reply_to: Option<i64>` · `turns: Option<i64>` · `duration_ms: Option<i64>`.
- `chat::add_reply(conn, conv_id, uuid, text, state, reply_to, turns, duration_ms)` — 에이전트 답 저장. 기존 `add_message` 는 그대로.
- `chat::summaries(conn)` — 대화마다 `{conversation, title(첫 사람 메시지 앞 40자), last_state, last_at}`, 마지막 시각 내림차순.

### 프로토콜 · 실행기 · 조율

- `Request::Reply` 에 `#[serde(default)] turns: Option<i64>` · `duration_ms: Option<i64>` — 이전 버전 상대의 답(필드 없음)도 받는다.
- `RunOutput` 에 `turns` · `duration_ms`(`parse_output` 이 `num_turns` · `duration_ms` 를 읽음, 없으면 `None`).
- `Chat::record` 는 `add_reply(…, reply_to = asked.id, …)`, `Reply` 에 실행 정보를 싣는다. `inbound_reply` 도 `add_reply(…, asked.id, …)`.

### API(`chat_api`)

- `GET /api/conversations` → `[{id, uuid, peer_id, agent, dir, title, last_state, last_at}]`.
- `GET /api/conversations/{id}/messages` → `[{id, author, text, state, created_at, reply_to, turns, duration_ms}]`.
- 정적 경로 `/view.js`(`server`) — 화면 로직 모듈.

### 웹UI(`crates/app/web`)

- `view.js`(DOM 없음, `export`): `stateText(state)` · `peerState(peer)` · `convLabel(conv, peerNames)` · `filterByPeer(convs, peerId)` ·
  `timeline(messages)`(사람 메시지마다 `{request, reply|null, steps}` — 단계: 요청 · 실행(턴 · 시간) · 응답/대기/미전달) ·
  `keepsConversation(conv, railPeer)` · `formatDuration(ms)` · `latest()`(요청 세대 — 늦게 온 응답 판별: `const t = guard.next(); …; if (!guard.is(t)) return;`).
- `app.js`: 화면 조립만(`textContent` · `replaceChildren`). 해시 경로: `#c/{id}`(대화) · `#t/{id}`(타임라인) · `#settings` · `#me`(설정의 내 정보) ·
  `#new`(새 대화). 대화 목록 · 열린 대화는 5초마다 다시 읽고 알림 이벤트에도 다시 읽는다.
- `index.html` · `app.css`: 레일(72px) · 사이드바(260~300px) · 메인. 좁은 화면에서 감싼다(flex-wrap).
- 테스트: `crates/app/web/view.test.mjs` — `node --test --test-reporter=tap "crates/app/web/*.test.mjs"`(Node 22+ 글롭).

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 스키마 5 · 실행 정보 · 원 요청 대응 | 테스트(store · runner · roundtrip) | E3 · E4 · E5 |
| 2 | API 확장 · `/view.js` · 화면 로직 모듈 | 테스트(chat_api · view.test) | E6 · E7 |
| 3 | 화면 조립(A · 설정 · C) · 검증 명령 표 | 정적 검사 | E1 · E2 · E8 · E9 · E10 |

## 평가 항목

- [ ] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A7 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A4 — 저장 4건(agent_defaults_read_only_tools · conversation_by_uuid_per_peer · messages_in_order_with_state · reply_meta_and_summaries)
  CHECK: cargo test -p an-core --test store
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: pending
- [ ] E4: A4 — 실행기 출력 실행 정보(parse_claude_json 이 num_turns · duration_ms 를 읽고 없으면 None)
  CHECK: cargo test -p an-agent --test runner parse_claude_json
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A4 — 왕복 실행 정보 · 원 요청 대응 · 이전 버전 답(reply_carries_run_meta · reply_without_meta_accepted 포함 11건)
  CHECK: cargo test -p an-agent --test roundtrip
  EXPECT: test result: ok. 11 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A2·A3 — 웹UI API 3건(agents_crud_api · conversation_api_roundtrip · conversation_summaries_and_message_meta)
  CHECK: cargo test -p agentnetwork-app --test chat_api
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A2·A3·A5 — 화면 로직 node 테스트(상태 문구 · 대화 제목 · 상대별 거르기 · 타임라인 · 시간 표기 · 늦은 응답 폐기)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E8: A6 — 웹UI 에 innerHTML · outerHTML · insertAdjacentHTML · document.write 없음
  CHECK: python -c "import pathlib;src=''.join(p.read_text(encoding='utf-8') for p in pathlib.Path('crates/app/web').glob('*.js'));bad=[w for w in ['innerHTML','outerHTML','insertAdjacentHTML','document.write'] if w in src];print('html_sinks=' + (','.join(bad) or 'none'))"
  EXPECT: html_sinks=none
  EVIDENCE: pending
- [ ] E9: A1·A5 — 정적 구조: 레일 · 사이드바 · 대화 · 설정 · 타임라인 영역 id 와 검증 명령 표의 node 행
  CHECK: python -c "import pathlib;h=pathlib.Path('crates/app/web/index.html').read_text(encoding='utf-8');p=pathlib.Path('PROJECT.md').read_text(encoding='utf-8');need=['id=\"rail\"','id=\"sidebar\"','id=\"conversation\"','id=\"settings\"','id=\"timeline\"','id=\"me\"'];miss=[n for n in need if n not in h]+([] if 'node --test --test-reporter=tap' in p else ['PROJECT.md node row']);print('missing=' + (','.join(miss) or 'none'))"
  EXPECT: missing=none
  EVIDENCE: pending
- [ ] E10: A1·A2·A3 — 실화면(수동): 3단 · 설정 · 트레이 내 정보 · 레일 거르기 · 새 대화 · 이어 쓰기 · 타임라인 · 좁은 화면
  EVIDENCE: pending

## 종결 시 할 일

- 매뉴얼 `an_core`(스키마 5 · `add_reply` · `summaries`) · `an_net`(`Reply` 실행 정보) · `an_agent`(`RunOutput`) · `an_app`(API · `/view.js` ·
  화면 구조) · PRODUCT(웹UI 구조 · JS 테스트) · 원장 4건(JS 테스트 기반) 갱신 · masterPlan `#10` · Phase 1 분할 반영.

## 개정 기록

- 개정 1(2026-10-11, 적대적 평가 라운드 1 반영): 스키마 5 이전은 한 트랜잭션 + 있는 열 건너뛰기(`paths`
  `schema5_migration_survives_partial_state`, E1 안). 레일에서 다른 PC 를 고르면 열린 대화를 닫고, 보내기는 해시의 대화 · 레일 상대가
  맞을 때만(`view.js` `keepsConversation`, E7 안).
- 개정 2(2026-10-11, 적대적 평가 라운드 2 반영): 스키마를 올리기 전 `{db}.v{버전}.bak` 사본(`VACUUM INTO`, 이전 실행 파일로 롤백할 때
  바꿔 쓴다 — `paths` `migration_keeps_backup_for_rollback`, E1 안). 이전 이력(`reply_to` 없음) 답은 기다리는 요청이 하나일 때만 연결,
  여럿이면 "어느 요청의 답인지 모르는 답" 으로 따로(`view.js` `linkReplies` · `unlinkedReplies`, E7 안).

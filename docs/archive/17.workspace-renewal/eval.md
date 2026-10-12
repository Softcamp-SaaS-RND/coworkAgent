# eval.md — 17.workspace-renewal 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: f003ab3072eea78ea9d5c2523572c4ea9dd273e8

- [x] E1: 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E2: 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

- [x] E3: A1·A2·A3·A4·A6·A7·A8·A9·A10 — 화면 로직 node 테스트 31건(기존 21 + side_nav · new_menu · profile_menu · insert_markdown · enter_sends · message_actions · run_steps · search_groups · activity_rows · conv_sections, 기존 "화면 위치" 는 새 경로로 고침)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass 31\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 81.2381

- [x] E4: A1·A3·A8·A11 — 화면 문구 literal(ux_literals 8건: 기존 5 + workspace_labels(설계 "문구 literal" 목록 전부 · `id="raw_btn"` 1회) · no_received_label('주고받은 글' 밖의 '받은 글' 0) · no_timeline_tab(`tab_timeline` · `panel_timeline` · `#t/` · "타임라인" 0))
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: test result: ok. 8 passed; 0 failed
  EVIDENCE: test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E5: A1 — 색 · 글꼴 크기 리터럴은 `app.css` 첫 `:root` 안에만이고, 다크 토큰이 Nocturne 값 · 글꼴 첫 이름이 Noto Sans KR
  CHECK: python -c "import re;c=open('crates/app/web/app.css',encoding='utf-8').read();j=open('crates/app/web/app.js',encoding='utf-8').read();m=re.search(r':root\s*\{[^}]*\}',c);r=m.group(0) if m else '';rest=c.replace(r,'');v=len(re.findall(r'#[0-9a-fA-F]{3,8}\b|rgba?\(',rest))+len(re.findall(r'font-size:\s*[0-9.]+(?:px|rem|em)',rest))+len(re.findall(r'#[0-9a-fA-F]{6}\b',j));need=['--dark-bg: #0E1117','--dark-surface: #171C28','--dark-border: #262D3D','--dark-text: #E8EBF2','--dark-accent: #7C83FF','--dark-agent: #3DD6B5','--dark-run: #F5B83D','--dark-ok: #3DD68C','--dark-danger: #FF6B6B','--font: '+chr(39)+'Noto Sans KR'+chr(39)];n=sum(x in r for x in need);print(f'root={int(bool(m))} violations={v} nocturne={n}')"
  EXPECT: root=1 violations=0 nocturne=10
  EVIDENCE: root=1 violations=0 nocturne=10

- [x] E6: A2·A3·A4·A5·A10 — 로컬 API(ux_api 24건: 기존 16 + search_groups_messages_titles_peers_agents · search_treats_like_wildcards_literally · activity_merges_theirs_failures_relay_events · activity_unseen_by_peer_and_mark_seen · star_toggles_local_only · saved_toggle_and_list_scoped · conversation_read_clears_unread · settings_notify_paused_roundtrip)
  CHECK: cargo test -p coworkagent-app --test ux_api
  EXPECT: test result: ok. 24 passed; 0 failed
  EVIDENCE: test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.67s

- [x] E7: A3·A4·A5·A8 — 스키마 11 이전(paths 13건: 기존 12 + schema11_workspace — 새 열 · `relay_events` · read_id · 내 활동 본 표시 id 채움 · 대화별 색인 · `v10.bak`)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 13 passed; 0 failed
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s

- [x] E8: A8 — 실행 model · effort 가 양쪽 답에 남는다(roundtrip 13건: 기존 12 + reply_records_model_effort_both_sides)
  CHECK: cargo test -p an-agent --test roundtrip
  EXPECT: test result: ok. 13 passed; 0 failed
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.37s

- [x] E9: A3 — 자동 왕복 시작 · 끝(상한 도달)이 알림 설정과 무관하게 기록된다(relay 10건: 기존 9 + relay_start_end_events_recorded)
  CHECK: cargo test -p an-agent --test relay
  EXPECT: test result: ok. 10 passed; 0 failed
  EVIDENCE: test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.70s

- [x] E10: A10 — 알림 일시 중지(notifier 11건: 기존 9 + paused_drops_notices_and_tray · resume_after_pause_notifies_again)
  CHECK: cargo test -p coworkagent-app --test notifier
  EXPECT: test result: ok. 11 passed; 0 failed
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.25s

- [x] E11: A11 — 15 결정을 지키는 테스트 29건이 이름 그대로 남아 있다(ux_api 16 · ux_literals 5 · view 8 — 개정 1 에서 15 적대적 평가 회귀 5건 추가)
  CHECK: python -c "fs={'crates/app/tests/ux_api.rs':['favicon_served','peer_patch_group_model_effort','agent_patch_edits_but_not_name','settings_claude_defaults','inbox_lists_their_messages_with_search','outbox_lists_my_messages_to_peer','theme_setting_persists','to_human_rejected','conversation_title_set_and_edit','peer_tile_alias_icon','settings_me_tile_side_width','direct_add_with_code_pairs_both_sides','direct_readd_known_key_waits_for_approval','new_conversation_to_human_rejected','inbox_kind_filter_applies_before_limit','settings_read_error_is_500'],'crates/app/tests/ux_literals.rs':['turn_label_literal','invite_labels_without_messenger','favicon_link_present','title_search_label','tray_menu_open_quit_only'],'crates/app/web/view.test.mjs':['settings_tree','markdown_blocks_safe','html_doc_detect','group_filter_search','inbox_rows','peer_messages','direction_tag','tile_text']};n=sum(len(v) for v in fs.values());m=sum(((chr(34)+t+' ') if f.endswith('.mjs') else ('fn '+t+'(')) not in open(f,encoding='utf-8').read() for f,v in fs.items() for t in v);print(f'targets={n} missing={m}')"
  EXPECT: targets=29 missing=0
  EVIDENCE: targets=29 missing=0

- [ ] E12: A1·A2·A3·A4·A5·A6·A7·A8·A9·A10 — 실화면(수동): 5영역 · 다크/라이트 · 통합 검색 · 내 활동 · ★ · 저장 · 메시지 작업 · 입력창 · 실행 상세 · + 메뉴 · 프로필 · 알림 일시 중지 · 두 PC 왕복의 model · effort
  EVIDENCE: pending

ABANDON: E12 실화면 수동 확인은 원격 세션이라 불가 — 프로토타입(Phase 1 끝) 인수 Task 19.prototype-acceptance 에 TC-L 을 더해 Task 18 종결 뒤 수동 테스트 일괄(masterPlan D28 · 진행 목표 "사용자 테스트는 task 18 완료 후 일괄 검증.")

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base 8db4db4 --scope branch` · 실행 HEAD `3774ede`(요청과 같음, 코드 측정 `6484365`) · 2026-10-12
- Verdict: needs-attention(high 2 · medium 1)

### 원문

```text
출시 보류. 읽음·활동 ACK 경쟁 조건이 새 항목을 보지 않았는데도 읽은 것으로 숨길 수 있으며, 주기적 활동 조회는 이력 증가에 따라 DB를 지속적으로 전수 스캔한다.

Findings:
- [high] 읽음 처리가 화면에 표시되지 않은 동시 도착 메시지까지 삼킨다 (crates/core/src/chat.rs:302-305)
  UI는 메시지 GET 후 별도 POST로 읽음을 기록하지만, `mark_read`는 화면이 받은 마지막 ID가 아니라 실행 시점의 `max(id)`를 저장한다. 두 요청 사이에 상대 메시지가 도착하면 해당 메시지는 렌더되지 않았어도 읽음 처리되어 사이드바·레일 배지가 사라진다. 알림이 꺼진 상태에서 사용자가 즉시 이동하면 새 답을 놓칠 수 있다.
  Recommendation: 읽음 POST에 GET 결과의 `last_seen_message_id`를 보내고, 해당 대화 소속인지 검증한 뒤 `read_id = max(current, acknowledged_id)`만 적용한다. GET과 POST 사이 메시지 도착 회귀 테스트를 추가한다.
- [high] 초 단위 활동 워터마크가 새 활동을 영구히 seen으로 분류한다 (crates/core/src/chat.rs:428-510)
  활동은 `created_at > activity_seen_at`으로 판정하지만 열람 시각과 이벤트 시각 모두 `unixepoch()` 초 단위다. 열람 POST 직후 같은 초에 저장된 메시지나 relay event는 시각이 같아 unseen이 되지 않는다. GET과 열람 POST 사이에 도착한 항목도 사용자가 보지 않았는데 ACK된다. 결과적으로 실패·자동 왕복 종료 같은 중요한 활동의 배지와 강조가 조용히 누락된다.
  Recommendation: 벽시계 대신 ACK 가능한 단조 커서를 사용한다. 예: GET이 peer별 message ID와 relay_event ID 워터마크를 반환하고 POST가 실제 렌더된 워터마크만 전진시키도록 한다. 같은 초 및 GET/POST 사이 도착 테스트를 추가한다.
- [medium] 5초 폴링이 무인덱스 활동 이력을 반복 전수 스캔한다 (crates/app/web/app.js:1715-1723)
  `refresh()`는 항상 `loadCounts()`를 호출하고 5초마다 실행된다. 이 요청은 `activity(limit=1)` 외에 `activity_unseen`을 두 번 호출하며, peer가 없으면 동일한 전체 count를 중복 계산한다. schema 11은 `messages(conversation_id, created_at)`·`relay_events(conversation_id, created_at)` 인덱스를 만들지 않았다. 이력이 누적될수록 매 5초 여러 전체 스캔과 정렬이 발생해 공유 DB mutex를 점유하고 UI 갱신·메시지 기록을 지연시킬 수 있다는 추론이 코드 구조상 직접 성립한다.
  Recommendation: peer가 없으면 unseen 쿼리를 한 번만 실행하고, 필요한 복합 인덱스를 schema 11에 추가한다. 가능하면 활동 워터마크 변경을 SSE로 전달해 고정 5초 count 폴링을 제거하고 대규모 이력 성능 테스트를 추가한다.

Next steps:
- Q1: 읽음 API를 `last_seen_message_id` ACK 계약으로 바꾼다.
- Q2: 활동 seen 상태를 초 단위 시각 대신 단조 ID 워터마크로 바꾼다.
- Q3: schema 11 인덱스와 대규모 이력 회귀 테스트 후 재리뷰한다.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [high] 읽음 처리가 화면에 표시되지 않은 동시 도착 메시지까지 삼킨다(A5 · E6) | 사실 — `openConversation` 은 GET 뒤 따로 `POST …/read`, `mark_read` 는 그때의 `max(id)`. 테스트로 재현: 앞서 받은 글까지만 읽음을 보내도 새 답까지 읽음(`unread` 0, 기대 1 — RED) | `POST …/read {upto}`(화면이 받은 글 최대 id) — `read_id = max(read_id, upto)`, 그 대화 글이 아니면 400 · 없는 대화 404 · 몸 없음 400. `app.js` 가 받은 글의 최대 id 를 보낸다. `conversation_read_clears_unread` 보강(받은 뒤 온 답 = 안 읽음 · 뒤로 가지 않음 · 다른 대화 글 · 없는 글 · 몸 없음 400). plan 개정 2 · `3a23fa0` |
| [high] 초 단위 활동 워터마크가 새 활동을 영구히 seen 으로 분류한다(A4 · E6) | 사실 — `created_at > activity_seen_at`, 둘 다 `unixepoch()` 초. 재현: 기존 테스트의 `sleep(1100)`(초 단위 회피)을 빼면 본 직후 같은 초의 글이 `(0, 0)`(기대 `(1, 1)` — RED). GET 과 본 표시 사이 도착분도 본 것이 된다(본 표시 = 그때 시각) | 본 시점을 id 로 — `peers.activity_seen_msg` · `activity_seen_event`(올릴 때 max id 로 채움). `GET /api/activity` 가 목록과 같은 잠금에서 `upto{message, event}` 를 주고 `POST /api/activity/seen {peer?, message, event}` 는 그 범위까지만(뒤로 가지 않음 · 지금 마지막 항목을 넘지 않음 · 범위 없는 몸은 415 · 422 — 빈 몸 = 전부 폐지). 글 · 기록을 지우는 곳이 없어(`DELETE` 없음) id 가 단조. `activity_unseen_by_peer_and_mark_seen` 보강(같은 초 · 받은 뒤 본 표시 전 도착 · 범위 넘는 값 · 범위 없는 몸) · `schema11_workspace` 보강(채움 = max id). 스키마 11 은 배포 전이라 같은 단계에서 고쳤다(D32) — 이 커밋 전 코드로 이미 11 이 된 개발 DB 는 지우고 다시 만든다. plan 개정 2 · `3a23fa0` |
| [medium] 5초 폴링이 무인덱스 활동 이력을 반복 전수 스캔한다(A4 · E6 · E7) | 일부 사실 — `peer` 없을 때 같은 수를 두 번 세고, 수만 필요한 `loadCounts` 가 목록 · 대화 요약까지 만든다. 색인이 없다 — `EXPLAIN QUERY PLAN`(python sqlite3 3.49.1, 글 2만 · 대화 200): 안 본 수 `SCAN m`, 대화 글(`chat::messages`) `SCAN messages`. "공유 DB 뮤텍스 점유로 기록 지연" 은 측정하지 않았다 | `limit=0` = 수만(목록 · 요약 생략 — `loadCounts` 가 쓴다) · `peer` 없으면 한 번 · `messages(conversation_id)` · `relay_events(conversation_id)` 색인 · 안 본 수는 가장 앞선 본 표시 뒤 id 만. 고친 뒤 계획: 안 본 수 `SEARCH m USING INDEX messages_by_conv (conversation_id=? AND rowid>?)` · 대화 글 `SEARCH messages USING INDEX messages_by_conv (conversation_id=?)` · 대화 목록 안 읽음도 같은 색인. 색인 존재는 `schema11_workspace` 가 단정. 지적의 복합 `(…, created_at)` 은 id 기준으로 바꿔 필요 없다. SSE 로 수 전달 · 대규모 이력 성능 테스트는 미조치(`## 미조치 지적`) · `3a23fa0` |

## 적대적 평가 — 라운드 2/2

- 수단: codex-review 세션 `/codex:adversarial-review --base 8db4db4 --scope branch` · 실행 HEAD `30683ec`(요청과 같음, 코드 측정 `3a23fa0`) · 2026-10-12
- Verdict: needs-attention(high 1 · medium 1 — 라운드 1 처리 3건에는 새 지적 없음)
- 라운드 상한(2/2) — 반영 뒤 재리뷰하지 않는다. 남은 것은 `## 미조치 지적`.

### 원문

```text
라운드 2/2: 출시 보류. 후발 실패가 읽은 활동으로 숨고, 자동 왕복 lifecycle 기록도 불완전하다.

Findings:
- [high] 가변 메시지 상태를 삽입 ID로 ACK해 후발 실패를 숨긴다 (crates/core/src/chat.rs:489-533)
  `activity_upto`는 활동 대상이 아닌 `sent`/`running` 메시지까지 전체 `max(id)`로 ACK하고, `activity_unseen`은 이후 `error`가 된 행도 `id > activity_seen_msg`일 때만 센다. 사용자가 요청 N 실행 중 활동을 열고 나서 같은 행이 `error`로 바뀌면 새 실패인데도 `unseen=false`, 배지 0이 된다. 별도 실패 행이 생기지 않는 명시적 거절·기록 실패 경로에서는 작업 실패를 조용히 놓친다.
  Recommendation: 실패 상태 전환마다 불변 `activity_event`를 트랜잭션으로 추가하고 그 이벤트 ID를 ACK한다. `sent` 상태에서 ACK한 뒤 같은 메시지를 `error`로 바꿨을 때 unseen이 1이 되는 회귀 테스트를 추가한다.
- [medium] 자동 왕복 기록이 알림 호출에 종속돼 수동 stop/resume을 누락한다 (crates/agent/src/chat.rs:731-768)
  `relay_events`는 `notify()`가 `RelayStart`/`RelayEnd`를 받을 때만 추가된다. 그러나 `relay_end()`는 사람의 중지를 명시적으로 제외하고, opener의 `relay(false)`는 상태만 끈다. 최초 start도 실제 시작 시점이 아니라 첫 reply 도착 때 기록된다. 따라서 응답이 지연·실패한 왕복에는 start가 없고, 사용자가 중지한 쪽에는 end가 없으며, 재개도 opener 기록에서 빠져 A3 활동 이력이 실제 lifecycle과 어긋난다.
  Recommendation: 알림이 아니라 relay 상태 전환 지점에서 start/end를 같은 DB 트랜잭션으로 기록한다. 무응답 시작, 양쪽 수동 중지, 재개, 중복 요청을 검증한다.

Next steps:
- Q1: 후발 `error`를 불변 activity event로 전환한다.
- Q2: relay start/stop/resume 기록을 상태 전환과 원자화한다.
- Q3: 관련 API·relay 회귀 테스트 후 라운드 상한에 따라 재리뷰 없이 종료한다.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [high] 가변 메시지 상태를 삽입 ID 로 ACK 해 후발 실패를 숨긴다(A3 · E6) | 사실 — 안 본 판정이 글 id 하나라, 본 뒤 `error` 로 바뀐 내 글(이미 본 id)은 다시 세지 않는다. 테스트로 재현: 다 본 뒤 내 글 하나를 `error` 로 → `(0, 0)`(기대 `(1, 1)` — RED), `schema11_workspace` 도 0(기대 1). 흔한 실패(상대의 실패 답 · 내 에이전트의 실패 답)는 새 글이 따로 생겨 세지만, 상대 거절(보내는 중 열람) · 실행 기록 실패 경로는 새 글이 없다 — Stage 1 리뷰 L2 와 같은 뿌리 | 실패 전환을 불변 기록으로 — 모든 실패 전환이 지나는 `chat::set_state` · `set_flags` 가 내 글이 `error` 로 바뀌는 순간 `relay_events` 에 `failed` 행(`message_id`)을 남긴다(이미 실패 · 남의 글이면 남기지 않음). 안 본 판정 = 글 id > 본 글 id **또는** 그 글의 실패 기록 id > 본 기록 id(ACK 는 같은 `upto.event`). 안 본 수는 본 id 안쪽 글의 늦은 실패를 따로 센다(겹침 없음). 자동 왕복 목록 · 수는 `start` · `end` 만. 지적이 권한 새 이벤트 표 대신 기존 기록 표에 열 하나 · 색인 하나(스키마 11 배포 전 — D32). `activity_unseen_by_peer_and_mark_seen` 보강(늦은 실패 → `(1, 1)` · 그 항목 `unseen` · 자동 왕복 목록 1건 그대로) · `schema11_workspace` 보강(두 번 바꿔도 1건 · 색인 3). plan 개정 3 · `f003ab3` |
| [medium] 자동 왕복 기록이 알림 호출에 종속돼 수동 stop/resume 을 누락한다(A3 · E9) | 서술은 사실 · 설계대로 — plan 해석 4 가 "시작 · 끝 알림을 내는 자리에서 … 남긴다" 로 정했다. 알림과 같은 기준(D29 보강 — 자동 왕복은 시작 1 + 저절로 · 상대가 멈춘 끝 1)이라 내가 한 멈춤 · 재개는 내 활동에 없고, 연 쪽 시작은 첫 답이 온 때다(첫 실행이 실패해도 시작 · 끝 둘 다 — `relay_events_first_run_fails`). 전달이 안 된 시작은 보낼 때 오류로 바로 보인다 | 고치지 않음 — 상태 전환마다 남기면 내가 한 일까지 안 본 수 · 배지에 든다(알림 기준과 어긋남). 전체 수명 기록이 필요하면 사용자 결정(`## 미조치 지적`) |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| E12 수동 gate(실화면 · 두 PC) 미충족 | 사실 | 프로토타입(Phase 1 끝) 인수 Task 로 이관(D28). ABANDON 기록 · 19 TC-L |
| 라운드 2 [medium] 자동 왕복 기록이 알림 기준 — 내가 한 멈춤 · 재개 · 응답 없는 시작이 내 활동에 없다 | 서술은 사실 · 설계대로 | plan 해석 4(알림 기준). 상태 전환 기록으로 바꾸면 내가 한 일이 안 본 수에 든다 — 전체 수명 기록은 사용자 결정 대상 |
| 라운드 1 [medium] 남은 부분 — 활동 수를 SSE 로 전달(5초 새로 고침 제거) · 대규모 이력 성능 테스트 | 일부 사실(측정 안 함) | 수만 조회 · 색인으로 전수 스캔은 없앴다(`EXPLAIN QUERY PLAN`). 5초 새로 고침은 15 설계 — SSE 전환은 화면 갱신 구조 변경이라 이 Task 밖. 벤치 도구 없음 |
| Stage 1 리뷰 파생 — `peers.id` 재사용(AUTOINCREMENT 없음): 가장 큰 id 의 상대를 지우고 새 상대를 이으면 남은 대화 · 내 활동이 새 상대에 붙는다 | 사실(측정) · 17 이전부터 | 상대 삭제 정책(이력 보존 · id 재사용) 결정이 먼저 — 원장 |
| Stage 3 리뷰 "기존" — 공유 `latest` 번호표와 5초 새로 고침이 겹치면 느린 조회(4초)는 다음 새로 고침이 계속 앞질러 그리기가 늦는다 | 사실(재현 12초) · Stage 3 이전부터 | 로컬 API 는 보통 수 ms. 화면별 번호표로 나누는 구조 변경은 이 Task 밖 |
| Stage 1 리뷰 L1 — 받은 쪽은 `Relay{on:true}` 마다 시작을 기록(자동 왕복이 아닌 대화 · 겹친 프레임도) | 사실 · 영향 작음 | 정상 상대는 자동 왕복 대화의 재개 때만 보낸다(D32 — 같은 빌드) |
| Stage 1 리뷰 N — 검색이 `summaries` 를 두 번 · LIKE(ASCII) 와 `to_lowercase`(유니코드)의 대소문자 차이 · `execute` 실패 답에 model · effort 없음 | 사실 · 영향 작음 | 성능 · 표시 차이만 |
| Stage 2 리뷰 T — `workspace_labels` 는 화면에 안 그려지는 코드로도 통과(라벨 일부가 다른 글자의 일부로도 맞는다) | 사실 · plan 정의 | 문구 literal 목록은 plan 이 정했다 — 실화면은 19 TC-L |

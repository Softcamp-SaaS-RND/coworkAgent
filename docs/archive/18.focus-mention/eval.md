# eval.md — 18.focus-mention 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 70b2c494e7d2b0ca137ddae3fb01943535941e91

- [x] E1: 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E2: 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.15s

- [x] E3: A1·A3·A5·A6·A9·A10 — 화면 로직 node 테스트 34건(기존 31 + presence · mention · ask_marks — 받은 요청 작업 · 내 활동 · 보낸 메시지 라벨은 ask_marks 에, 기존 profile_menu 보강)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass 34\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 86.8819

- [x] E4: A1·A3·A5·A6·A9·A10 — 화면 문구 literal(ux_literals 9건: 기존 8 + focus_mention_labels — 설계 "문구 literal" 목록 전부)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: test result: ok. 9 passed; 0 failed
  EVIDENCE: test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E5: A1·A5 — 색 · 글꼴 크기 리터럴은 `app.css` 첫 `:root` 안에만이고, 다크 토큰이 Nocturne 값 · 글꼴 첫 이름이 Noto Sans KR
  CHECK: python -c "import re;c=open('crates/app/web/app.css',encoding='utf-8').read();j=open('crates/app/web/app.js',encoding='utf-8').read();m=re.search(r':root\s*\{[^}]*\}',c);r=m.group(0) if m else '';rest=c.replace(r,'');v=len(re.findall(r'#[0-9a-fA-F]{3,8}\b|rgba?\(',rest))+len(re.findall(r'font-size:\s*[0-9.]+(?:px|rem|em)',rest))+len(re.findall(r'#[0-9a-fA-F]{6}\b',j));need=['--dark-bg: #0E1117','--dark-surface: #171C28','--dark-border: #262D3D','--dark-text: #E8EBF2','--dark-accent: #7C83FF','--dark-agent: #3DD6B5','--dark-run: #F5B83D','--dark-ok: #3DD68C','--dark-danger: #FF6B6B','--font: '+chr(39)+'Noto Sans KR'+chr(39)];n=sum(x in r for x in need);print(f'root={int(bool(m))} violations={v} nocturne={n}')"
  EXPECT: root=1 violations=0 nocturne=10
  EVIDENCE: root=1 violations=0 nocturne=10

- [x] E6: A1·A3·A5·A6·A9·A10 — 로컬 API(ux_api 28건: 기존 24 + focus_setting_and_peer_presence · ask_kind_recorded_and_filtered · done_only_by_receiver · mention_api_rules)
  CHECK: cargo test -p coworkagent-app --test ux_api
  EXPECT: test result: ok. 28 passed; 0 failed
  EVIDENCE: test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.91s

- [x] E7: A1·A5·A6·A7·A9·A10 — 스키마 12 이전(paths 14건: 기존 13 + schema12_focus_mention — 새 열 · `agent_sessions` · `v11.bak`)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 14 passed; 0 failed
  EVIDENCE: test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s

- [x] E8: A2·A3·A6·A7·A9·A10 — 두 PC 실행(roundtrip 16건: 기존 13 + focus_refuses_new_request_unrecorded · ask_kind_and_done_sync_both_sides · mention_runs_named_agent_own_session)
  CHECK: cargo test -p an-agent --test roundtrip
  EXPECT: test result: ok. 16 passed; 0 failed
  EVIDENCE: test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.69s

- [x] E9: A4·A6 — 자동 왕복(relay 12건: 기존 10 + focus_stops_relay_both_sides · mention_rejected_in_relay)
  CHECK: cargo test -p an-agent --test relay
  EXPECT: test result: ok. 12 passed; 0 failed
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.82s

- [x] E10: A1 — 집중 동안 알림이 나가지 않는다(notifier 12건: 기존 11 + focus_pauses_notices)
  CHECK: cargo test -p coworkagent-app --test notifier
  EXPECT: test result: ok. 12 passed; 0 failed
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.39s

- [x] E11: A5 — `Pong` 의 집중 · 접속 기록(pairing 6건: 기존 5 + pong_carries_focus_and_presence)
  CHECK: cargo test -p an-net --test pairing
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s

- [x] E12: A6·A7 — CLI `ask --conv --agent` 멘션(cli 16건: 기존 15 + ask_mention_in_conversation)
  CHECK: cargo test -p coworkagent-app --test cli
  EXPECT: test result: ok. 16 passed; 0 failed
  EVIDENCE: test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.36s

- [x] E13: A8 — 15 · 17 결정을 지키는 테스트 55건이 이름 그대로 남아 있다(15 의 29 + 17 의 26)
  CHECK: python -c "fs={'crates/app/tests/ux_api.rs': ['favicon_served', 'peer_patch_group_model_effort', 'agent_patch_edits_but_not_name', 'settings_claude_defaults', 'inbox_lists_their_messages_with_search', 'outbox_lists_my_messages_to_peer', 'theme_setting_persists', 'to_human_rejected', 'conversation_title_set_and_edit', 'peer_tile_alias_icon', 'settings_me_tile_side_width', 'direct_add_with_code_pairs_both_sides', 'direct_readd_known_key_waits_for_approval', 'new_conversation_to_human_rejected', 'inbox_kind_filter_applies_before_limit', 'settings_read_error_is_500', 'search_groups_messages_titles_peers_agents', 'search_treats_like_wildcards_literally', 'activity_merges_theirs_failures_relay_events', 'activity_unseen_by_peer_and_mark_seen', 'star_toggles_local_only', 'saved_toggle_and_list_scoped', 'conversation_read_clears_unread', 'settings_notify_paused_roundtrip'], 'crates/app/tests/ux_literals.rs': ['turn_label_literal', 'invite_labels_without_messenger', 'favicon_link_present', 'title_search_label', 'tray_menu_open_quit_only', 'workspace_labels', 'no_received_label', 'no_timeline_tab'], 'crates/app/web/view.test.mjs': ['settings_tree', 'markdown_blocks_safe', 'html_doc_detect', 'group_filter_search', 'inbox_rows', 'peer_messages', 'direction_tag', 'tile_text', 'side_nav', 'new_menu', 'profile_menu', 'insert_markdown', 'enter_sends', 'message_actions', 'run_steps', 'search_groups', 'activity_rows', 'conv_sections'], 'crates/core/tests/paths.rs': ['schema11_workspace'], 'crates/agent/tests/roundtrip.rs': ['reply_records_model_effort_both_sides'], 'crates/agent/tests/relay.rs': ['relay_start_end_events_recorded'], 'crates/app/tests/notifier.rs': ['paused_drops_notices_and_tray', 'resume_after_pause_notifies_again']};n=sum(len(v) for v in fs.values());m=sum(((chr(34)+t+' ') if f.endswith('.mjs') else ('fn '+t+'(')) not in open(f,encoding='utf-8').read() for f,v in fs.items() for t in v);print(f'targets={n} missing={m}')"
  EXPECT: targets=55 missing=0
  EVIDENCE: targets=55 missing=0

- [ ] E14: A1·A2·A3·A4·A5·A6·A7·A9·A10 — 실화면 · 두 PC(수동): 집중 칩 · 메뉴 · 거부 · "상대가 집중 중" · 다시 보내기 · 자동 왕복 멈춤 · 타일 점 · 머리글 · @ 후보 · 멘션 답 이름 · 같은 에이전트 맥락 · 질의 · 요청 · 거르기 · 완료 표시 양쪽
  EVIDENCE: pending

ABANDON: E14 실화면 · 두 PC 수동 확인은 원격 세션이라 불가 — 프로토타입(Phase 1 끝) 인수 Task 19.prototype-acceptance 에 TC-M 을 더해 수동 테스트 일괄(masterPlan D28 · 진행 목표 "사용자 테스트는 task 18 완료 후 일괄 검증.")

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base 0fba45b --scope branch` · 2026-10-12. 첫 실행은 codex 모델 용량 오류("Selected model is at
  capacity")로 실패 — 그 출력의 "approve" 는 중간 메시지라 무효(라운드로 세지 않음), 같은 명령 재실행이 정본. 실행 HEAD `056515f`(요청 `95caf13` 와
  문서 커밋 2개 차이 — 코드 측정 `6c1455f` 그대로)
- Verdict: needs-attention(high 2)

### 원문

```text
출하 불가: @멘션이 조용히 잘못된 에이전트로 라우팅될 수 있고, 삭제된 에이전트의 Claude 세션이 경합 중 부활할 수 있다.

Findings:
- [high] 에이전트 목록 장애 시 @멘션이 기본 에이전트에서 실행됨 (crates/app/web/app.js:1367-1373)
  `mentionTarget(text, mentionNames() ?? [])` 때문에 상대 에이전트 목록이 아직 로드되지 않았거나 `/agents` 조회가 실패하면, 사용자가 입력한 `@이름`을 멘션이 아닌 일반 본문으로 전송한다. 결과적으로 요청이 대화 기본 에이전트의 다른 폴더·세션에서 실행되며 UI는 라우팅 실패를 알리지 않는다.
  Recommendation: 본문에 멘션 형태가 있는데 후보 목록이 준비되지 않았으면 전송을 막고 목록을 재조회하라. 더 안전하게는 선택한 대상 ID를 입력 상태에 보존하거나 서버에서 대상 이름을 검증·해석해 기본 에이전트로의 silent fallback을 금지하라.
- [high] 삭제·재등록 경합이 제거된 Claude 세션을 되살림 (crates/agent/src/chat.rs:1685-1693)
  실행 완료 시 현재 에이전트의 이름과 `folder`만 비교해 세션을 다시 저장한다. 실행 중 에이전트를 삭제한 뒤 같은 이름·폴더로 재등록하면 `agents::remove`가 지운 `agent_sessions`가 이 코드에서 다시 생성된다. 새 프로필의 다음 멘션이 삭제 전 세션과 대화 문맥을 재개해 세션 초기화 계약과 격리를 깨뜨린다.
  Recommendation: 실행 시작 시 에이전트의 비재사용 generation/UUID를 캡처하고 기록 시 동일 generation인지 확인하라. 삭제·폴더 변경 시 generation을 증가시키며, 불일치한 실행 결과는 세션을 저장하지 않는 경합 테스트를 추가하라.

Next steps:
- 에이전트 목록 미로딩·조회 실패 상태에서 수동 @멘션 전송 회귀 테스트 추가
- 실행 중 에이전트 삭제→동일 이름·폴더 재등록 후 세션 미복원 동시성 테스트 추가
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [high] 에이전트 목록 장애 시 @멘션이 기본 에이전트에서 실행됨(A6) | 사실 — 목록이 없으면(받는 중 · 실패) `mentionTarget` 이 빈 목록으로 판정해 손으로 친 `@이름` 이 대화 에이전트에게 간다. Stage 3 리뷰 3 으로 힌트는 숨기고 다시 묻게 했으나 보내기는 막지 않았다. 다만 실행은 읽기 전용 문의(D31)라 잘못 간 글의 피해는 엉뚱한 답이다 | 멘션을 쓰는 대화에서 목록이 없고 글에 멘션처럼 보이는 `@이름`(글 머리 · 공백 뒤, `view.js` `hasMention`)이 있으면 보내지 않고 "보내지 않음 — 상대 에이전트 목록을 받지 못해 @멘션을 가릴 수 없습니다. 잠시 뒤 다시 보내세요" + 목록 다시 묻기(입력은 그대로). 낱말 안의 `@`(메일 주소)는 그대로 보낸다. CLI 는 이미 목록으로 거른다(없는 이름 · 목록 실패 = 오류). view `mention` 보강 · 눈 확인(목록 두 번 실패 → 두 번 막힘 · 글 남음 → 받은 뒤 `agent` 실어 보냄 · 메일 주소 글은 바로 보냄). plan 개정 5 · `4b90f8f` |
| [high] 삭제 · 재등록 경합이 제거된 Claude 세션을 되살림(A7) | 사실 — 끝난 실행이 이름 · 폴더만 비교해, 실행 중 지우고 같은 이름 · 같은 폴더로 다시 등록하면 비운 세션을 다시 쓴다. 테스트로 재현(RED — 지운 뒤 끝난 실행이 대화 세션을 되살림). `conversations.session` 도 같은 길(지울 때 비우기는 Stage 3 리뷰 4 에서 넣었다) | 에이전트 세대 `agents.uid`(등록 · 폴더 변경 때 새 값 — 행 id 는 SQLite 가 가장 큰 id 를 다시 쓸 수 있어 쓰지 않음) — `record` 는 지금 세대가 실행 때와 같을 때만 세션(대화 · 멘션)을 남긴다(폴더 비교를 대신). 스키마 12 안(D32 — 이전 에이전트 행은 채움). roundtrip `folder_change_during_run_drops_stale_session` 보강(실행 중 지우고 다시 등록 → 세션 없음 · 다음 실행 새 세션) · paths `schema12_focus_mention` 보강(채움 · 폴더 변경 새 세대 · 그대로면 그대로). plan 개정 5 · `4b90f8f` |

## 적대적 평가 — 라운드 2/2

- 수단: codex-review 세션(라운드 1 과 같은 세션) `/codex:adversarial-review --base 0fba45b --scope branch` · 실행 HEAD `4803eae`(요청과 같음, 코드 측정
  `2e70907`) · 2026-10-12
- Verdict: needs-attention(high 1 — 라운드 1 처리 2건에는 새 지적 없음)
- 라운드 상한(2/2) — 반영 뒤 재리뷰하지 않는다. 남은 것은 `## 미조치 지적`.

### 원문

```text
출하 불가: 잘못되거나 stale한 @멘션이 기본 에이전트로 조용히 실행된다.

Findings:
- [high] 인식되지 않은 @멘션이 기본 에이전트로 잘못 라우팅됨 (crates/app/web/app.js:1374-1382)
  후보 목록이 로드된 상태에서는 `hasMention(text)`가 참이어도 `mentionTarget()`이 일치 대상을 못 찾으면 `agent: null`로 전송한다. 오타(`@guidee`) 또는 캐시 후 새로 등록된 에이전트 이름은 이 경로를 타며, 서버는 이를 일반 메시지로 받아 대화 기본 에이전트의 폴더·세션에서 실행한다. 사용자는 멘션 실패를 보지 못하고 다른 컨텍스트의 답을 받는다.
  Recommendation: 멘션 문법이 감지됐지만 정확한 후보가 없으면 전송을 차단하고 목록을 재조회하라. 선택한 대상은 본문 재해석 대신 별도 입력 상태로 보존하고, 오타·stale 목록 회귀 테스트를 추가하라.

Next steps:
- 알 수 없는 멘션과 stale 후보 목록에서 전송이 차단되는 브라우저 테스트를 추가한다.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [high] 인식되지 않은 @멘션이 기본 에이전트로 잘못 라우팅됨(A6) | 사실 — 목록을 받은 상태에서 멘션 꼴 `@이름` 이 후보와 맞지 않으면(오타 · 그 뒤 새로 등록된 이름) 해석 2 대로 멘션이 아닌 글이 되어 대화 에이전트에게 알림 없이 간다 | 한 번 막고 알린다 — "보내지 않음 — {@이름}: 상대 에이전트 목록에 없는 이름입니다(목록을 다시 받습니다). 멘션이 아니면 한 번 더 보내세요"(목록 다시 받기 · 글 그대로), 같은 글을 다시 보내면 멘션 없이 보낸다 — 늘 막으면 코드의 `@Override` · `@app.route` 같은 글을 보낼 수 없다(권고 "차단" 과 다른 점 — 미조치에 기록). 맞는 이름이 있으면 바로 멘션. `view.js` `mentionToken` · view `mention` 보강 · 눈 확인(오타 → 막힘 → 다시 보내면 `agent` 없이, 낡은 목록 → 막힘 · 목록 다시 받음 → 다시 보내면 `agent: newbie`, 맞는 멘션 → 바로). plan 개정 6 · `70b2c49` |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| E14 수동 gate(실화면 · 두 PC) 미충족 | 사실 | 프로토타입(Phase 1 끝) 인수 Task 로 이관(D28). ABANDON 기록 · 19 TC-M |
| Stage 1 리뷰 L7 남은 부분 — `watch_presence` 의 페어링 필터 · `main.rs` 의 띄우기는 테스트로 가르지 못한다 | 사실 | 페어링 아닌 Ping 은 상대가 거절해 기록이 남지 않아 단정으로 구분할 수 없다 · `main` 은 테스트 밖 — 19 TC-M 1(접속 상태)에서 확인 |
| Stage 1 리뷰 N2 — 연 쪽 집중은 `inbound_reply` 에서만 걸려 상대가 한 차례 더 실행한다 | 사실 · 설계대로 | plan 해석 8(다음 차례에서 멈춤 — 이미 대기열에 든 차례는 끝까지) |
| Stage 1 리뷰 N3 — 상태 칩이 `aria-pressed` 와 라벨 전환("받기 가능" ↔ "집중")을 함께 쓴다(APG 는 토글 라벨 고정 권장) | 사실 · plan 수준 | plan 해석 15 가 정한 문구 — 19 인수 때 스크린리더로 확인 |
| Stage 2 리뷰 L5 남은 부분 — 다른 상대 소유 대화를 가리킨 `Done` 조작 테스트 없음 | 사실 | 세 번째 PC 가 필요하다 · 같은 `chat::find(conv, peer)` 를 모든 들어온 처리기가 쓰고 나머지 조작 셋은 테스트(roundtrip) |
| Stage 3 리뷰 5 — `\p{P}` 로 이름이 `qa` 만일 때 `@qa-check` 가 `qa` 로 잡힌다 | 사실 · 명세대로 | plan 해석 2(뒤가 문장 부호면 이름 끝) |
| R1 · R2 권고 남은 부분 — 고른 대상을 본문 재해석 대신 입력 상태로 보존 · 서버가 대상 이름을 검증 · 해석 | 사실(권고) · 설계 선택 | plan 해석 2(손으로 친 `@이름` 도 멘션 — 목록과 글자 그대로). 대신 화면이 목록 없음 · 맞지 않는 이름을 막고 알리며(R1 · R2 조치) CLI 는 상대 목록으로 거른다 |
| R2 남은 부분 — 맞지 않는 멘션 꼴을 늘 막지 않고 한 번 알린 뒤 다시 보내면 멘션 없이 보낸다 | 사실 · 위험 수용(위임) | 늘 막으면 `@` 로 시작하는 코드 · 낱말을 보낼 수 없다. 남는 위험 = 알림을 보고도 다시 보내면 대화 에이전트가 답한다(읽기 전용 문의 — D31) |

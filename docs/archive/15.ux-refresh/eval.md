# eval.md — 15.ux-refresh 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 2a3bfbd467014c333ed312f64dbf2a896fd73a61

- [x] E1: 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E2: 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

- [x] E3: A3·A4·A8·A10·A12·A15·A17 — 화면 로직 node 테스트 21건(기존 13 + settings_tree · markdown_blocks_safe · html_doc_detect · group_filter_search · inbox_rows · peer_messages · direction_tag · tile_text — 개정 4 에서 edit_preview_toggle 대체, 개정 5 에서 tile_text 추가)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass 21\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 78.4113

- [x] E4: A1 — 시안 5개 아티팩트 게시 · 사용자 선택이 intent `## 결정` 에 기록(수동: 아티팩트 링크 · 고른 안 · 날짜)
  EVIDENCE: 2026-10-12 · 사용자(선생님) 선택 · 시안 5개 아티팩트 https://claude.ai/artifact/MFCsMv39fSYUALTsS6oMPe 게시 뒤 사용자 지시 원문 "A안 기본. …" 으로 시안 A(Slate Pro) — intent `## 결정` 라운드 3 · 토큰 표 work/stage1.md(관측: intent.md 라운드 3 첫 줄)

- [x] E5: A2 — 색 · 글꼴 크기 리터럴은 `app.css` `:root` 안에만(`app.js` 색 리터럴 0)
  CHECK: python -c "import re;c=open('crates/app/web/app.css',encoding='utf-8').read();j=open('crates/app/web/app.js',encoding='utf-8').read();m=re.search(r':root\s*\{[^}]*\}',c);rest=c.replace(m.group(0),'') if m else c;v=len(re.findall(r'#[0-9a-fA-F]{3,8}\b|rgba?\(',rest))+len(re.findall(r'font-size:\s*[0-9.]+(?:px|rem|em)',rest))+len(re.findall(r'#[0-9a-fA-F]{6}\b',j));print(f'root={int(bool(m))} violations={v}')"
  EXPECT: root=1 violations=0
  EVIDENCE: root=1 violations=0

- [x] E6: A6 — 초대 자동 등록(auto_pair 2건: invite_hello_auto_pairs_both_sides · rehello_without_code_needs_approval)
  CHECK: cargo test -p an-net --test auto_pair
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

- [x] E7: A5·A7·A11·A16·A18 — 화면 문구 · favicon 링크 · 트레이 메뉴 literal(ux_literals 5건: turn_label_literal · invite_labels_without_messenger · favicon_link_present · title_search_label · tray_menu_open_quit_only — 개정 5 에서 2건 추가)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E8: A5·A6·A8·A9·A10·A12·A13·A14·A16·A17 — 로컬 API(ux_api 16건: favicon_served · peer_patch_group_model_effort · agent_patch_edits_but_not_name · settings_claude_defaults · inbox_lists_their_messages_with_search · outbox_lists_my_messages_to_peer · theme_setting_persists · to_human_rejected · conversation_title_set_and_edit · peer_tile_alias_icon · settings_me_tile_side_width · direct_add_with_code_pairs_both_sides · direct_readd_known_key_waits_for_approval · new_conversation_to_human_rejected · inbox_kind_filter_applies_before_limit · settings_read_error_is_500 — 개정 5 에서 3건, 개정 6 에서 2건, 개정 7 에서 3건 추가)
  CHECK: cargo test -p coworkagent-app --test ux_api
  EXPECT: test result: ok. 16 passed; 0 failed
  EVIDENCE: test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.47s

- [x] E9: A9 — 실행 인자 model · effort(run_options 3건: claude_args_carry_model_effort · peer_override_then_default · settings_read_error_fails_run — 개정 7 에서 1건 추가)
  CHECK: cargo test -p an-agent --test run_options
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

- [x] E10: A8·A9·A10·A16·A17 — 스키마 9 · 10 이전(paths 10 → 12건 — 개정 5 에서 schema10_title_alias_icon 추가)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 12 passed; 0 failed
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

- [ ] E11: A2·A3·A4·A6·A8·A10·A11·A12·A13·A15·A16·A17·A18 — 실화면(수동): 고른 시안대로 전 화면 · 트리 · 아코디언 · 편집 · 뷰어 · 나/보낸 목록 · 알림 크기 · 두 PC 초대 자동 등록 · 타일 클릭 · 제목 · 타일 표시 이름 · 아이콘 · 트레이 메뉴(개정 5)
  EVIDENCE: pending

- [x] E12: A14 — CLI `say` 삭제(cli 15건: Task #16 의 16건 − `say_*` 2건 + say_is_unknown_command)
  CHECK: cargo test -p coworkagent-app --test cli
  EXPECT: test result: ok. 15 passed; 0 failed
  EVIDENCE: test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.24s

ABANDON: E11 실화면 수동 확인은 원격 세션이라 불가 — 프로토타입(Phase 1 끝) 인수 Task 19.prototype-acceptance 에 TC-K 를 더해 수동 테스트 일괄(masterPlan D28)

WEAK: E4 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base 5bb31b5 --scope branch` · 실행 HEAD `a81bfcd`(요청과 같음) · 2026-10-12
- Verdict: needs-attention

### 원문

```text
출하 보류. A6 직접 추가 경로가 미완성이며, A9는 지원 여부가 확인되지 않은 Claude Code 버전을 실행 가능으로 허용한다.

Findings:
- [high] 주소+코드 직접 추가가 201을 반환해도 페어링되지 않는다 (crates/app/src/peers_api.rs:161-164)
  plan의 자동 등록 범위는 초대 텍스트와 주소+코드 직접 추가를 모두 포함한다. 그러나 초대 텍스트 분기만 `node.approve()`를 호출하고, 직접 추가 분기는 `node.add()` 결과를 그대로 201로 반환한다. `auto_pair` 테스트도 `node.add()` 직후 `their_ok && !my_ok`를 확인한 다음 수동으로 `approve()`를 호출한다. 따라서 직접 추가 UI는 성공으로 보이지만 추가 승인이 필요하고, 그전에는 메시지·ping이 거절된다.
  Recommendation: 코드가 일치해 상대 승인을 받은 직접 추가 결과도 initiator 승인을 완료하도록 흐름을 통합하라. `/api/peers` 주소+코드 호출 한 번으로 양쪽 `paired()`와 ping 성공을 검증하고, 코드 없는 재등록은 승인 대기로 남는 회귀 테스트를 추가하라.
- [high] 새 `--effort` 인자가 최소 버전 gate와 불일치한다 (crates/agent/src/runner.rs:83-89)
  이 변경은 모든 실행에 `--effort`를 무조건 추가하지만 런타임 gate는 여전히 Claude Code 2.1.267 이상을 허용한다. 저장소 증거는 2.1.293~2.1.296에서만 `--effort`를 확인했고, `work/stage2.md`도 2.1.267~2.1.292를 `확인 필요`로 남겼다. 해당 구간이 옵션을 지원하지 않으면 버전 검사는 통과한 뒤 CLI 인자 파싱에서 종료되어 모든 원격 에이전트 실행이 실패한다. 실제 지원 여부는 미확인이지만 호환성 계약이 입증되지 않은 상태다.
  Recommendation: 실행 전 `claude --help` capability 검사를 추가하거나 `--effort`를 지원하는 최초 확인 버전으로 `MIN_CLAUDE`를 올려라. 최소 버전 직전·정확 경계·지원 버전에서 실행 인자 수용 여부를 검증하는 테스트도 추가하라.

Next steps:
- 직접 추가 API의 무승인 양방향 페어링 회귀 테스트를 추가하고 E6를 재실행한다.
- Claude Code `--effort` 지원 경계를 확정한 뒤 E9와 전체 workspace gate를 다시 실행한다.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [high] 주소+코드 직접 추가가 201 을 반환해도 페어링되지 않는다(A6 · E8) | 사실 — `peers_api.rs` `add` 의 직접 추가 분기는 `node.add()` 결과만 돌려 붙인 쪽 승인(`my_ok`)이 빠진다. 새 테스트로 재현: `my_ok=false · their_ok=true · paired=false`(RED) | 코드를 담아 보냈고 상대가 승인했으면(`their_ok`) 붙인 쪽 승인까지 한 번에. `direct_add_with_code_pairs_both_sides`(양쪽 paired · ping 204) · `direct_readd_known_key_waits_for_approval`(아는 키 다시 추가 — 코드 없음 · 있음 둘 다 승인 대기). mutation: `their_ok` 조건 제거 → 후자 FAIL, 승인 호출 제거 → 전자 FAIL. 기존 `peers_api.rs` `add_then_list_peer` 의 `paired == false` 단정은 붙인 쪽 수동 승인 시절(#01) 흐름이라 `true` 로 고쳤다(D8 추가 수용 — 직접 추가도 자동 등록). plan 개정 6(E8 11 → 13, 사용자 미승인) |
| [high] `--effort` 가 최소 버전 gate(2.1.267)와 맞는지 입증되지 않았다(A9 · E9) | 오탐 — 공식 CHANGELOG(https://github.com/anthropics/claude-code/blob/main/CHANGELOG.md, 2026-10-12 받음): 2.1.72 "Fixed `--effort` CLI flag being reset by unrelated settings writes on startup"(이미 있음) · 2.1.111 "Added `xhigh` effort level … Available via `/effort`, `--effort`" · 2.1.221 "… at effort `xhigh`/`max` …" · 2.1.267 "Added `--system-prompt-snapshot off`"(`MIN_CLAUDE` 근거). 2.1.267 이상은 `--effort` 와 다섯 값을 모두 받는다 | 동작 변경 없음 — `runner.rs` 주석에 CHANGELOG 근거를 적고 `MIN_CLAUDE` 의 "확인 필요" 를 지웠다 |

## 적대적 평가 — 라운드 2/2

- 수단: codex-review 세션 `/codex:adversarial-review --base 5bb31b5 --scope branch` · 실행 HEAD `fb50ac7`(요청과 같음) · 2026-10-12
- Verdict: needs-attention(high 2 · medium 2 — 라운드 1 처리 2건은 타당 판정)
- 라운드 상한(2/2) — 반영 뒤 재리뷰하지 않는다. 남은 것은 `## 미조치 지적`.

### 원문

```text
출하 보류. 라운드 1의 직접 추가 수정·회귀 테스트는 타당하고, `--effort` 지적도 [공식 CHANGELOG](https://raw.githubusercontent.com/anthropics/claude-code/refs/heads/main/CHANGELOG.md)상 오탐이다. 그러나 A14 우회, 폴더 변경 race, 불완전 필터, 설정 DB 오류 은폐가 남았다.

Findings:
- [high] 새 대화에서는 `to: "human"`이 거부되지 않고 실행 요청으로 바뀐다 (crates/app/src/chat_api.rs:31-48)
  `NewConversation`에 `to`가 없고 `deny_unknown_fields`도 없어 Serde가 해당 필드를 조용히 버린다. 따라서 `POST /api/conversations`에 `to:"human"`을 보내면 400이 아니라 `send_via()`로 진행되어 원격 에이전트를 실행한다. A14의 모든 새 글 정책을 `/api/conversations/{id}/messages`에만 적용한 범위 축소이며, 호출자 의도와 반대로 사용량·원격 실행이 발생한다.
  Recommendation: `NewConversation`에도 `to: Option<String>`을 추가해 생략 또는 `agent`만 허용하라. 두 POST 경로 모두 `human`·알 수 없는 값이 400이고 대화·메시지·원격 실행이 남지 않는 회귀 테스트를 추가하라.
- [high] 실행 중 폴더 변경이 지운 세션을 옛 실행 결과가 되살린다 (crates/core/src/agents.rs:110-115)
  폴더 변경은 대화의 `session`을 NULL로 만들지만 실행 작업과 동기화되지 않는다. 이미 실행 중인 작업은 옛 폴더·세션을 보유하고 있으며, 변경 후 완료되면 `Chat::record`가 `crates/agent/src/chat.rs:1289`에서 옛 session을 다시 저장한다. 다음 요청은 새 폴더에서 옛 session을 `--resume`하여 실패하며, 실패 시 session도 지워지지 않아 대화가 계속 실패할 수 있다.
  Recommendation: 폴더 변경을 해당 에이전트 대화 실행 잠금과 직렬화하거나 agent/folder revision을 실행 입력에 담고 완료 시 revision이 같을 때만 session을 기록하라. 실행을 막아 둔 상태에서 폴더 변경→실행 완료→다음 실행이 `Session::New`인지 검증하라.
- [medium] 받은 글 종류 필터가 200건 제한 뒤 적용돼 일치 결과를 숨긴다 (crates/app/web/app.js:938-952)
  서버는 종류와 무관하게 최신 200건을 먼저 반환하고, UI가 그 뒤 `inboxRows(..., {kind})`로 사람/에이전트 종류를 거른다. 최신 200건이 반대 종류라면 더 오래된 일치 메시지가 있어도 화면은 `받은 글 없음`을 표시한다. A12의 필터가 전체 목록이 아니라 잘린 표본에만 적용되는 조용한 누락이다.
  Recommendation: `kind`/`author`를 `/api/messages` 쿼리와 `messages_across` SQL에 추가해 `LIMIT` 전에 적용하거나 서버 페이지네이션을 구현하라. 201건 이상에서 오래된 일치 항목이 반환되는 경계 테스트를 추가하라.
- [medium] 설정 DB 읽기 오류가 기본값 성공으로 위장된다 (crates/core/src/run_opts.rs:81-89)
  `settings::get(...).ok().flatten()`이 없는 설정과 SQLite 읽기 실패를 동일하게 처리한다. DB 손상·I/O 오류가 발생해도 설정 API는 200과 `sonnet/high`를 내보내고, 실행 경로도 그 기본값으로 계속 동작한다. 사용자가 지정한 model/effort가 적용되지 않았는데 성공처럼 보이는 조용한 실패다.
  Recommendation: `defaults`와 `resolve`가 `rusqlite::Result`를 반환하게 하고 `Ok(None)` 또는 저장값 불일치에만 기본값을 사용하라. 실제 DB 오류는 설정 API 5xx 및 실행 실패로 노출하는 테스트를 추가하라.

Next steps:
- A14의 두 POST 경로를 같은 검증 함수와 회귀 테스트로 묶는다.
- 폴더 변경·실행 완료 순서를 제어하는 race 테스트를 추가한다.
- 필터-before-limit 및 DB 오류 전파를 보강한 뒤 E8·E9·workspace gate를 재측정한다.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [high] 새 대화에서는 `to: "human"` 이 거부되지 않고 실행 요청으로 바뀐다(A14 · E8) | 사실 — `NewConversation` 에 `to` 가 없어 serde 가 버리고 `send_via()` 로 간다. 새 테스트로 재현: `to:"human"` → 201 · 상대 실행(RED) | 이어 쓰기와 같은 검증 `to_of` 를 두 POST(새 대화 · 이어 쓰기)에 — 없거나 `agent` 만, 그 밖 400. 회귀 `new_conversation_to_human_rejected`(`human` · 모르는 값 400 뒤 양쪽 대화 0 · `agent` 201). plan 개정 7 |
| [high] 실행 중 폴더 변경이 지운 세션을 옛 실행 결과가 되살린다(A10 · E1) | 사실 — `Chat::record` 가 실행 결과의 세션을 무조건 `set_session`. 새 테스트로 재현: 실행 도중 `agents::update`(폴더 변경) 뒤 세션 `Some`(RED) | `record` 가 같은 트랜잭션에서 실행한 에이전트의 지금 폴더를 대조 — 실행 때 폴더와 다르면 세션을 남기지 않는다(세션은 그 폴더에서만 이어진다). 자동 왕복은 실행한 내 에이전트로 대조(`agents::update` 가 지우는 `relay_agent` 대화와 같은 대상). 회귀 `folder_change_during_run_drops_stale_session`(roundtrip — 세션 None · 다음 실행 `Session::New`). plan 개정 7 |
| [medium] 받은 글 종류 필터가 200건 제한 뒤 적용돼 일치 결과를 숨긴다(A12 · E8) | 사실 — 서버는 종류와 무관하게 최신 `limit` 건, `inboxRows` 가 그 뒤 거른다. 새 테스트로 재현: 최신이 에이전트 답이면 `limit=1&kind=human` 이 에이전트 답(RED) | `GET /api/messages` 에 `kind`(`human` · `agent`, 그 밖 400) — `messages_across` 가 `LIMIT` 전에 거르고 `app.js` 가 보낸다(화면 쪽 거르기는 그대로 — 결과 같음). 회귀 `inbox_kind_filter_applies_before_limit`(제한 경계를 201건 대신 `limit=1` 로 — 같은 `LIMIT` 경로). 서버 페이지네이션 대안은 쓰지 않았다(상한 500 은 A12 설계). plan 개정 7 |
| [medium] 설정 DB 읽기 오류가 기본값 성공으로 위장된다(A9 · E8 · E9) | 사실 — `settings::get(...).ok().flatten()`. 새 테스트로 재현: `settings` 표를 지우면 `GET /api/settings` 200 + `sonnet` · `high`, 실행도 기본값으로 성공(RED) | `run_opts` 설정 읽기 5개(`defaults` · `resolve` · `theme` · `side_width` · `me_tile`)와 같은 꼴의 `default_relay_cap` 이 `Result` — 없거나 틀린 값만 기본값(`peer_override_then_default` 의 "틀린 값은 건너뛴다" 유지). 설정 API(GET · PUT) 500 · 실행은 실패 답 · 자동 왕복 시작 500. 회귀 `settings_read_error_is_500`(ux_api) · `settings_read_error_fails_run`(run_options — 실행 0 · 오류 답). plan 개정 7 |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| E11 수동 gate(실화면) 미충족 | 사실 | 프로토타입(Phase 1 끝) 인수 Task 로 이관(D28). ABANDON 기록 · 19 TC-K |
| 라운드 2 [high] 부수 서술 — 재개(`--resume`) 실패 시 세션을 지우지 않아 그 대화가 계속 실패한다 | 사실(측정) — 지적 원인(폴더 변경 뒤 옛 세션 기록)은 고침 | 측정(2026-10-12, claude 2.1.296): 없는 세션 `--resume` → 종료 1 · stdout 비어 있음 · stderr "No conversation found with session ID: …" → `parse_output` 오류 → 실행 실패 답, 세션은 그대로라 다음 요청도 같은 실패. 남는 원인은 폴더 변경 밖(세션 파일이 사라진 경우). 라운드 상한 — 재개 실패 시 세션 정리는 원장(#15 R2-F2r) |

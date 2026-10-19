# eval.md — 1w.conv-leave 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: f26d8138d26208657ad87e061def47073be428de

- [x] E1: A6 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A6 — 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s

- [x] E3: A6 — 화면 로직 node 테스트 · 실패 0 · 141 이상
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (14[1-9]|1[5-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 137.9769

- [x] E4: A2·A3·A9 — 나가기 · 거절 · 이전 버전 · 실행 중 막음(두 PC)
  CHECK: cargo test -p an-agent --test conv_leave leave
  EXPECT: /test result: ok\. [3-9] passed; 0 failed/
  EVIDENCE: test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 2.45s

- [x] E5: A4·A5 — 삭제 정리 · 되살아나지 않음 · 번호 재사용 없음 · 공유 사본
  CHECK: cargo test -p an-agent --test conv_leave delete
  EXPECT: /test result: ok\. [3-9] passed; 0 failed/
  EVIDENCE: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 2.31s

- [x] E6: A4·A7 — 스키마 21 이전(상대 소속 → 대화 · AUTOINCREMENT)
  CHECK: cargo test -p an-core schema21
  EXPECT: /test result: ok\. [2-9] passed; 0 failed/
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.12s

- [x] E7: A7 — 상대 소속 API 없음 · 그룹 JSON 에 peers 없음 · 나가기 · 삭제 경로
  CHECK: cargo test -p coworkagent-app -- peer_routes_gone group_json_has_no_peers leave_and_delete_routes
  EXPECT: /test result: ok\. [1-9] passed; 0 failed/
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.36s

- [x] E8: A1 — 대화 메뉴 · 닫힌 대화 문구 · 거절 문구 node 시험
  CHECK: node --test --test-reporter=tap --test-name-pattern="conv_menu_items|conv_closed|refused_left" crates/app/web/view.test.mjs
  EXPECT: /# pass [3-9]\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 63.1854

- [x] E9: A8 — 설정 그룹 절 · 사용자 목록 · 복사 node 시험
  CHECK: node --test --test-reporter=tap --test-name-pattern="settings_group_sections|conv_copy" crates/app/web/view.test.mjs
  EXPECT: /# pass [2-9]\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 64.2598

- [x] E10: A7 — 상대 소속 흔적 없음(서버 · 웹UI 원문)
  CHECK: python -c "import pathlib;fs=['crates/core/src/groups.rs','crates/core/src/db.rs','crates/agent/src/chat.rs','crates/core/src/chat.rs','crates/app/src/groups_api.rs','crates/app/web/app.js','crates/app/web/view.js'];t={f:pathlib.Path(f).read_text(encoding='utf-8') for f in fs};print('group_peers_live=%d peer_api=%d web=%d'%(sum(v.count('group_peers') for k,v in t.items() if k!='crates/core/src/db.rs'),t['crates/app/src/groups_api.rs'].count('/peers'),t['crates/app/web/app.js'].count('peerGroupMenu')+t['crates/app/web/view.js'].count('NO_PEER_GROUP')))"
  EXPECT: group_peers_live=0 peer_api=0 web=0
  EVIDENCE: group_peers_live=0 peer_api=0 web=0

- [x] E11: A6 — 문서 규약 · 의존 경계
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py
  EXPECT: crates=4 edges=6 violations=0
  EVIDENCE: crates=4 edges=6 violations=0

- [ ] E12: A1·A2·A3·A8 — 실화면(두 PC): ⋯ 나가기 → 양쪽 읽기 전용("나감" · "상대가 나감"), 삭제 → 목록에서 사라짐 · 상대 그대로, 설정 사용자 목록 대화 트리를 그룹 절로 끌어 복사. 프로토타입 인수 Task(D28)로 이월.
  EVIDENCE: pending

ABANDON: E12 실화면 두 PC 확인(⋯ 나가기 → 양쪽 읽기 전용 "나감" · "상대가 나감", 삭제 → 목록에서 사라짐 · 상대 그대로, 사용자 목록 대화 트리 → 그룹 절 복사) — 수동 테스트는 프로토타입 인수 Task 에서 일괄(masterPlan D28). 19 TC 로 넘김 · 자동 근거는 E4 · E5(두 PC Rust) · E8 · E9(화면 로직)

## 적대적 평가 — 라운드 1/2

평가자: `codex-review` 세션 · `/codex:adversarial-review --base b21b325 --scope branch "1w.conv-leave: …"`(job `review-mveut2lh-qx6cfa`, 실행 HEAD `03eb318` = 요청 일치 —
종료 때 `a0b7626`(평가 원장만, 코드 같음), 첫 시도는 Codex capacity 오류로 실패 · 재실행분) · Verdict needs-attention(high 3 · medium 1). 원문:

```text
출하 보류. 통과한 gate가 다루지 못한 종료 경계 경합 2건, `peer_left_at` 신뢰 경계 누락, 읽기 전용 UI 누락이 있습니다.

Findings:
- [high] 삭제와 수신 사이 TOCTOU로 묘비를 우회해 대화가 부활할 수 있음 (crates/agent/src/chat.rs:1759-1765)
  `handle`은 `chat::left`를 한 번 조회한 뒤 DB 잠금을 놓고 실제 수신 처리로 이동합니다. 그 사이 사용자가 삭제를 완료하면 묘비가 생기지만, 이미 검사를 통과한 `inbound_message`는 이후 `open_conversation`을 호출하며 묘비를 다시 검사하지 않습니다. 따라서 동시 요청이 삭제된 UUID의 대화를 재생성하고 원격 실행까지 예약할 수 있습니다. 이는 코드 순서에서 도출되는 경합이며 A4의 삭제·비부활 보장을 깨뜨립니다.
  Recommendation: 묘비/닫힘 확인과 대화 생성·메시지 기록을 하나의 immediate transaction에서 수행하고, `open_conversation` 자체도 해당 `(uuid, peer_id)` 묘비를 거부하게 하십시오. 삭제 commit 사이에 barrier를 둔 동시성 회귀 테스트를 추가하십시오.
- [high] 상대가 Leave를 보낸 뒤에도 원격 요청을 실행할 수 있음 (crates/core/src/chat.rs:499-503)
  수신 거절 판정인 `left()`는 로컬 `left_at`과 묘비만 확인하고 `peer_left_at`은 무시합니다. `Leave`를 보낸 상대가 버그나 악의로 같은 대화의 `Message`를 직접 보내면 상단 거절문을 통과하며, `inbound_message`도 `writable()`을 호출하지 않아 메시지를 기록하고 에이전트를 실행할 수 있습니다. 원격 입력을 신뢰하지 않는다는 프로젝트 가드레일과 양쪽 읽기 전용 계약을 위반합니다.
  Recommendation: 수신 거절용 판정을 `left_at OR peer_left_at OR tombstone`으로 만들거나 별도 `reject_inbound` 함수를 두십시오. `Leave` 수신 후 raw `Message`를 보내 `left`, 기록 0건, 실행 0회를 검증하십시오.
- [high] 대기 실행 취소 검사 뒤 경합으로 나간 대화의 실행이 시작됨 (crates/agent/src/chat.rs:2924-2929)
  gate 통과 후 메시지 상태를 한 번 읽어 취소 여부를 판단하지만, 실제 실행 직전에는 다시 검증하지 않습니다. 검사 직후 나가기 transaction이 `queued`를 `error/left`로 바꿔도 `execute`의 무조건 `set_state(..., Running)`이 이를 다시 `running`으로 덮고 runner를 시작할 수 있습니다. 삭제가 끼면 UPDATE가 0행이어도 성공으로 처리되어 삭제된 대화의 작업이 실행됩니다. 사용자 요청 뒤 실행을 막는 A9가 경합에서 깨지고 외부 도구 부작용·사용량이 발생할 수 있습니다.
  Recommendation: ticket 획득 직후 immediate transaction에서 `state='queued'` 및 conversation 미종료를 함께 검사하며 `queued → running`을 조건부 UPDATE하고, 변경 행이 정확히 1개일 때만 runner를 호출하십시오. 그 전후에 leave/delete를 끼우는 barrier 기반 테스트를 추가하십시오.
- [medium] 닫힌 자동 왕복 대화의 상세 패널에 재개 단추가 남음 (crates/app/web/app.js:2410-2413)
  머리글 자동 왕복 단추는 `convClosed(conv)`일 때 숨기지만 상세 패널 단추는 `relay.action`만 봅니다. 나가면서 `relay_on=0`이 된 `out` 대화는 `relayText`가 `resume`을 반환하므로 읽기 전용 화면의 실행 상세에 `재개`가 노출됩니다. 누르면 서버가 400을 반환해 읽기 전용 UI 계약과 사용자 경험이 어긋납니다.
  Recommendation: 상세 단추에도 `btn.hidden = !relay.action || !!convClosed(conv)`을 적용하고, 닫힌 자동 왕복 대화의 모든 mutation control이 숨겨지는 DOM/화면 로직 테스트를 추가하십시오.
```

| 지적(요지) | 판단 | 조치 |
|---|---|---|
| [high] 받기 앞 확인 ~ 대화 생성 사이 삭제 → 묘비 우회 부활 | 사실(check-then-act) | `inbound_message` · `inbound_ask` 가 나가기 · 삭제와 같은 잠금(`Chat::db`) 안에서 `chat::left` 를 다시 보고, `chat::open_conversation` 자체가 묘비 `(uuid, peer)` 를 거부(공통 진입점). 시험 `delete_race_guards`(묘비 uuid `open_conversation` 거부). barrier 시험은 미조치(아래) |
| [high] `peer_left_at` 을 거절 판정이 보지 않음 → Leave 보낸 상대 글 실행 | 사실 — 원격 입력 가드레일 위반 | `chat::left` = `left_at OR peer_left_at OR 묘비`(받기 앞 · 잠금 안 재확인 모두 이 함수). 시험 `leave_peer_left_refuses_raw`(Leave 받은 뒤 raw Message → `left` · 기록 0 · 실행 0) |
| [high] 취소 확인 뒤 `execute` 의 무조건 `running` 이 취소를 덮음 | 사실 | `chat::start_run` — `queued` · 열린 대화일 때만 `running`(UPDATE 한 문장, 1행일 때만 실행). `process` 가 줄 통과 뒤 부르고 `execute` 의 무조건 표시는 삭제, 구분선(`run_context`)도 같은 함수. 시험 `delete_race_guards`(열림 → 시작 · 이미 시작 · 나간 뒤 · 취소 되돌려도 나간 대화 · 지운 뒤 모두 거짓) |
| [medium] 실행 상세 패널 자동 왕복 '재개' 단추가 닫힌 대화에 남음 | 사실 — stage 2 에서 머리 단추만 막았다 | `detail_relay_btn` 도 `convClosed` 면 숨김. DOM 시험은 미조치(아래) |

반영 커밋 `e9ba06f`. 검증: `cargo test --workspace`(결과 줄 76 · 0 failed) · `cargo clippy --workspace --all-targets -- -D warnings`(경고 0) · node `# pass 142 # fail 0` · conv_leave 12 passed.

## 적대적 평가 — 라운드 2/2

평가자: `codex-review` 세션(라운드 1 과 같은 세션) · `/codex:adversarial-review --base b21b325 --scope branch "1w 라운드 2: …"`(job `review-mvevqkb6-m3o3y3`, 실행 HEAD `794e076` = 요청
일치 · 시작 = 끝, worktree 의 PRODUCT.md · docs/tech/security-posture.md 는 다른 세션 작업 — 커밋 diff 만 대상) · Verdict needs-attention(high 1 · medium 1).
**라운드 상한 도달 — 반영 뒤 재리뷰 없음.** 원문:

```text
선생님, 라운드 2/2 출하 보류 의견입니다. R1 수정 뒤에도 종료 경계의 형제 경로 2곳이 남습니다. Node 시험 142개 PASS; 아래 경합은 코드 추론입니다.

Findings:
- [high] 나가기 뒤에도 이미 검사한 원격 Delete가 보존 이력을 삭제함 (crates/agent/src/chat.rs:1759-1764)
  handle의 chat::left 검사와 실제 처리 사이에 DB 잠금이 풀립니다. 그 틈에 leave_conversation이 완료되어도 inbound_delete는 잠금 안에서 닫힘을 재검사하지 않고 chat::delete_message를 실행합니다. 따라서 나가기 성공 뒤 보존되어야 할 상대 사람 글이 영구 삭제됩니다. inbound_edit도 같은 방식으로 본문을 변경합니다. R1의 잠금 안 재검사가 Message·Ask에만 적용되어 A2 읽기 전용·이력 보존 계약에 경합이 남았습니다.
  Recommendation: 모든 수신 mutation 처리기의 실제 변경과 같은 DB 잠금 안에서 chat::left를 재검사하십시오. 상단 검사 후 나가기를 완료시키는 barrier 시험으로 Delete·Edit가 left를 반환하고 이력이 유지되는지 검증하십시오.
- [medium] 취소된 실행도 start_run 검사 전에 진행 프레임을 다시 생성함 (crates/agent/src/chat.rs:2932-2937)
  process는 admit를 마친 뒤에야 start_run을 호출합니다. admit는 대화를 미리 보관하고 ticket.ready().await 뒤 무조건 running Progress를 enqueue합니다. 다른 실행 때문에 기다리던 대화를 삭제한 후 자리가 나면, 삭제된 conv_id를 참조하는 outbox 행이 다시 생성된 다음 start_run이 false로 종료됩니다. enqueue에는 존재·닫힘 검사가 없고 외래 키도 없어 삽입이 가능합니다. 실행 자체는 막지만 A4의 딸린 항목 삭제와 나가기 이후 전송 중단은 지켜지지 않습니다.
  Recommendation: gate 획득과 진행 알림을 분리하고, start_run 성공 뒤에만 running 알림을 생성하십시오. Progress 저장도 같은 잠금 안에서 대화 존재·열림을 확인하고, 대기 중 삭제 후 gate를 해제하는 시험으로 outbox 잔존 0건을 검증하십시오.
```

| 지적(요지) | 판단 | 조치 |
|---|---|---|
| [high] 받기 앞 확인 뒤 나가기가 끼면 `inbound_delete` · `inbound_edit` 가 보존 이력을 바꿈 | 사실 — R1 은 대화를 여는 두 처리기만 막았다 | `chat::find_open`(= `find_own` 중 닫히지 않은 것) — 이력을 바꾸는 받기 처리기 전부(글 미러 `inbound_post` · 답 `inbound_reply` · 편집 · 삭제(`theirs`) · 요청 상태 · 진행 · 자동 왕복 · 컨텍스트)가 바꾸기와 같은 잠금 안에서 이것으로 찾는다(닫혔으면 `unknown` — 바꾸지 않음). 시험 `delete_race_guards` 에 `find_open` 거름 단정 |
| [medium] 실행 문 대기 중 지운 대화에 `running` 진행 행이 다시 생김 | 사실 | `admit` 의 무조건 `running` 알림 삭제 → `process` 가 `start_run` 성공 · 대화를 읽은 뒤 보낸다. 시험 `delete_while_waiting_leaves_no_rows`(동시 실행 1 · 앞 실행 중 대기 대화 삭제 · 보낼 줄을 막아 행이 남게 → 자리가 나도 실행 0 · 그 대화 번호 보낼 글 0). 물기 확인 — 옛 `admit` 알림을 되살린 빌드에서 `left: 1 right: 0` FAILED |

반영 커밋 `f26d813`. 검증: `cargo test --workspace`(결과 줄 76 · 0 failed) · `cargo clippy --workspace --all-targets -- -D warnings`(경고 0) · node `# pass 142 # fail 0` · conv_leave 13 passed.

## 미조치 지적

| 지적 원문 | 판단 | 미조치 사유 |
|---|---|---|
| R1 "삭제 commit 사이에 barrier를 둔 동시성 회귀 테스트를 추가하십시오" · "그 전후에 leave/delete를 끼우는 barrier 기반 테스트" · R2 "상단 검사 후 나가기를 완료시키는 barrier 시험" | 사실(지금 시험은 경합 순서를 강제하지 못한다) — 대신 막는 지점(`open_conversation` 묘비 거부 · `start_run` · `find_open`)을 직접 단정하고, 재현 가능한 한 경우(실행 문 대기 중 삭제)는 결정적 시험으로 | 받기 처리기 안에 시험용 멈춤 지점(barrier 훅)을 넣어야 한다 — 운영 코드에 시험 전용 갈래를 두지 않는 규약. 필요하면 별도 Task |
| R1 "닫힌 자동 왕복 대화의 모든 mutation control이 숨겨지는 DOM/화면 로직 테스트" | 사실(정적 · 순수 함수 시험은 DOM 대입을 못 본다) | DOM 시험은 jsdom 등 새 의존이 필요(저장소 화면 시험 규약 = DOM 없는 순수 함수 · 정적 원문, 1v 와 같은 판단). 실화면은 19 인수(E12) |
| R2 "Progress 저장도 같은 잠금 안에서 대화 존재·열림을 확인" | 부분 사실 — 실행 중(`running`)에는 나가기 · 삭제가 막혀 지운 대화에 행이 생기지 않는다. 상대가 나간 대화의 단계 알림은 상대가 `left` 로 거절해 행이 지워진다 | 남는 영향 없음(보내기 1회 · 거절) — `enqueue` 공통 경로에 대화 확인을 넣는 것은 모든 프레임 종류에 닿아 범위 밖 |

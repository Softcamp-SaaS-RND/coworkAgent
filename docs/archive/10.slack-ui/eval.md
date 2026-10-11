# eval.md — 10.slack-ui 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 6346c74eab985e9392d230d5e94ec127f6d55f74

- [x] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A7 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.24s

- [x] E3: A4 — 저장 4건(agent_defaults_read_only_tools · conversation_by_uuid_per_peer · messages_in_order_with_state · reply_meta_and_summaries)
  CHECK: cargo test -p an-core --test store
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

- [x] E4: A4 — 실행기 출력 실행 정보(parse_claude_json 이 num_turns · duration_ms 를 읽고 없으면 None)
  CHECK: cargo test -p an-agent --test runner parse_claude_json
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

- [x] E5: A4 — 왕복 실행 정보 · 원 요청 대응 · 이전 버전 답(reply_carries_run_meta · reply_without_meta_accepted 포함 11건)
  CHECK: cargo test -p an-agent --test roundtrip
  EXPECT: test result: ok. 11 passed; 0 failed
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.28s

- [x] E6: A2·A3 — 웹UI API 3건(agents_crud_api · conversation_api_roundtrip · conversation_summaries_and_message_meta)
  CHECK: cargo test -p agentnetwork-app --test chat_api
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s

- [x] E7: A2·A3·A5 — 화면 로직 node 테스트(상태 문구 · 대화 제목 · 상대별 거르기 · 타임라인 · 시간 표기 · 늦은 응답 폐기)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E8: A6 — 웹UI 에 innerHTML · outerHTML · insertAdjacentHTML · document.write 없음
  CHECK: python -c "import pathlib;src=''.join(p.read_text(encoding='utf-8') for p in pathlib.Path('crates/app/web').glob('*.js'));bad=[w for w in ['innerHTML','outerHTML','insertAdjacentHTML','document.write'] if w in src];print('html_sinks=' + (','.join(bad) or 'none'))"
  EXPECT: html_sinks=none
  EVIDENCE: html_sinks=none

- [x] E9: A1·A5 — 정적 구조: 레일 · 사이드바 · 대화 · 설정 · 타임라인 영역 id 와 검증 명령 표의 node 행
  CHECK: python -c "import pathlib;h=pathlib.Path('crates/app/web/index.html').read_text(encoding='utf-8');p=pathlib.Path('PROJECT.md').read_text(encoding='utf-8');need=['id=\"rail\"','id=\"sidebar\"','id=\"conversation\"','id=\"settings\"','id=\"timeline\"','id=\"me\"'];miss=[n for n in need if n not in h]+([] if 'node --test --test-reporter=tap' in p else ['PROJECT.md node row']);print('missing=' + (','.join(miss) or 'none'))"
  EXPECT: missing=none
  EVIDENCE: missing=none

- [ ] E10: A1·A2·A3 — 실화면(수동): 3단 · 설정 · 트레이 내 정보 · 레일 거르기 · 새 대화 · 이어 쓰기 · 타임라인 · 좁은 화면
  EVIDENCE: pending

ABANDON: E10 실화면 확인은 원격 세션이라 불가 — 프로토타입(Phase 1 끝) 인수 Task 에서 수동 테스트 일괄(masterPlan D28, 사용자 지시 2026-10-11 "사용자 테스트는 완료 후 진행")

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base dc6063e --scope branch` · 대상 HEAD `c835132` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] PC 레일 전환 후 이전 상대에게 메시지를 보낼 수 있음 (crates/app/web/app.js:115-119)
  대화 A를 연 상태에서 PC B를 선택해도 클릭 handler는 `state.railPeer`와 목록만 갱신한다. `state.current`와 `location.hash`는 계속 A를 가리키며, 전송 handler는 그 `state.current.id`를 사용한다. 따라서 화면은 B가 선택된 것처럼 보이지만 composer는 A에게 전송한다. 민감한 요청이 잘못된 PC로 유출될 수 있다.
  Recommendation: 선택한 PC와 `state.current.peer_id`가 다르면 활성 대화를 해제하고 home으로 이동하거나 해당 PC의 대화를 명시적으로 연다. 전환 완료 전 composer도 비활성화하고, PC 전환 후 POST 대상 conversation ID를 검증하는 DOM 통합 테스트를 추가한다.
- [high] 스키마 5 migration이 부분 적용되면 DB를 다시 열 수 없음 (crates/core/src/db.rs:70-76)
  세 개의 `ALTER TABLE`과 `PRAGMA user_version = 5`가 명시적 transaction 없이 `execute_batch`로 순차 실행된다. 디스크 부족·프로세스 종료가 중간에 발생하면 앞선 column은 커밋되지만 version은 4로 남을 수 있다. 다음 시작은 첫 `ALTER TABLE`을 재실행해 `duplicate column name`으로 실패하므로, 유일한 대화 이력 DB가 수동 복구 전까지 열리지 않는다.
  Recommendation: 모든 ALTER와 `user_version` 갱신을 하나의 SQLite transaction으로 묶는다. 부분 적용된 version 4 schema에서 재시작하는 회복 테스트도 추가한다.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 레일 전환 후 이전 상대로 전송 | 사실 | 다른 PC 를 고르면 열린 대화를 닫고 홈으로(`keepsConversation`). 보내기는 해시의 대화 = 열린 대화 · 레일 상대 일치일 때만, 아니면 보내지 않고 안내. 회귀: node "레일 전환" — "항상 유지" mutation FAIL. DOM 통합 테스트는 없음(미조치 후보) |
| 스키마 5 이전 비원자 | 사실 | 열 추가 · 버전 갱신을 한 트랜잭션, 이미 있는 열은 건너뜀. 회귀 `schema5_migration_survives_partial_state`(열 하나만 붙고 버전 4 인 DB) — "있는 열 확인 제거" mutation FAIL |

## 적대적 평가 — 라운드 2/2 (상한 — 재평가 없음)

- 수단: codex-review 세션 `/codex:adversarial-review --base dc6063e --scope branch` · 대상 HEAD `68a32f5` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 스키마 5 적용 즉시 이전 실행 파일로 롤백 불가 (crates/core/src/db.rs:70-79)
  `dc6063e`의 `SCHEMA_VERSION=4` 실행 파일은 더 높은 `user_version`을 거부합니다. 이 migration은 DB 백업·복원 경로 없이 `user_version=5`를 커밋하므로, 새 버전에 문제가 생겨 이전 바이너리로 롤백하면 `DB 스키마 5 가 이 실행 파일(4)보다 새롭다` 오류로 시작조차 못 합니다. 추가 열은 모두 nullable이라 구버전 SQL과 호환 가능한데도 version bump가 운영 복구를 차단합니다.
  Recommendation: SQLite backup API로 migration 전 v4 스냅샷과 복원 절차를 제공하거나, 이번 additive migration은 `user_version=4`를 유지하고 열 존재 여부로 기능을 감지해 구버전 롤백 호환성을 보장하세요.
- [medium] 스키마 4의 겹친 요청 이력이 잘못된 답과 연결됨 (crates/app/web/view.js:67-78)
  이전 이력은 `reply_to=null`인데 fallback은 다음 사람 메시지 전까지만 답을 찾습니다. 실제 가능한 순서 `H1,H2,A1,A2`를 넣으면 현재 함수는 `H1→null`, `H2→A1`을 반환하고 `A2`를 버립니다. 직접 Node 실행으로 재현했습니다. 송신 ACK 후 다음 요청을 보낼 수 있으므로 스키마 4에서도 이 순서가 가능하며, 타임라인이 답을 다른 요청의 결과로 표시합니다.
  Recommendation: 명시적 `reply_to`가 없는 legacy 이력에서 여러 요청이 겹치면 답을 임의 연결하지 말고 chronological/unlinked 상태로 표시하세요. 최소한 `H1,H2,A1,A2` 회귀 테스트를 추가하세요.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 스키마 5 후 롤백 불가 | 사실 | 스키마를 올리기 전 `{db}.v{버전}.bak` 사본(`VACUUM INTO`, 같은 버전 사본이 있으면 그대로). 롤백 = 이전 실행 파일 + 이 사본으로 교체(매뉴얼 an_core). 회귀 `migration_keeps_backup_for_rollback`(사본이 스키마 4 · 원래 행) — "사본 생략" mutation FAIL. 버전을 4 로 두는 안은 기각 — "새 스키마는 열지 않는다" 규칙을 무력화한다 |
| 겹친 이전 이력 답 오연결 | 사실 | `reply_to` 없는 답은 기다리는 요청이 하나일 때만 연결, 여럿이면 "어느 요청의 답인지 모르는 이전 이력 답" 으로 따로 표시(`linkReplies` · `unlinkedReplies`). 회귀: node `H1,H2,A1,A2` · `reply_to` 겹침 — "하나 이상이면 연결" mutation FAIL |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| E10 수동 gate(실화면) 미충족 | 사실 | 프로토타입(Phase 1 끝) 인수 Task 로 이관(D28). ABANDON 기록 |
| 레일 전환 · 보내기 대상 DOM 통합 테스트(라운드 1) | 사실 — 코드는 고침 | 판단 로직(`keepsConversation`)은 node 테스트, 조립(DOM)은 자동 테스트 대상 밖(intent 결정) — 19 수동 확인 |
| 사본 자동 복원(롤백 도구)(라운드 2) | 사실 — 사본까지 | 복원은 사본 파일 교체(수동, 매뉴얼). 자동 롤백은 배포 · 업데이트(Phase 3 S6)와 함께 |

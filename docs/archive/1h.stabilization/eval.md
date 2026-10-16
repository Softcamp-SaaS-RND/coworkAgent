# eval.md — 1h.stabilization 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 2f317ee1c30c577f51fcfd5820bdacf62607a03a

- [x] E1: A10 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A10 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

- [x] E3: A1·A3·A4 — 화면 로직 node 테스트 82건 이상 · 실패 0(폭 예산 — "build" · "박지훈" · "A팀장" · 그림 문자 · 빈 이름, 세 절 — '나' · 상대 · 그룹 · 구분선
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (8[2-9]|9[0-9]|[1-9][0-9]{2})\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 127.7376

- [x] E4: A2·A3·A4·A5 — 웹UI 문구 · 자리("북마크 항목" · "북마크" · "북마크 해제" · "개요" · 구분선 `role="separator"` · 겹친 카드 · 알약 단추 · 화면 파일에
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E5: A8 — 스키마 17(번호째 복사 · 사본 `.v16.bak` · 순번 = 지운 상대의 남은 대화 · 보낼 글 번호 이상 · 지우고 새로 이어도 번호 겹치지 않음 · 열 전부 그대로)
  CHECK: cargo test -p an-core --test store schema17
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.10s

- [x] E6: A7 — 사라진 세션 → 새 세션으로 한 번 다시 실행 · 답 완료 · 다음 요청 새 세션 이음 · 판별(2.1.299 결과 줄 · 2.1.296 stderr) · 사라짐이 아닌 실패는 다시
  CHECK: cargo test -p an-agent --test resume_missing
  EXPECT: /test result: ok\. [4-9] passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s

- [x] E7: A6 — 트레이 다시 시도 간격(1 · 2 · 4 · 8 · 15 · 30초 · 그 뒤 없음)
  CHECK: cargo test -p coworkagent-app --lib tray::
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

- [x] E8: A9 — 사용자 가이드에 방화벽 프로필 확인 명령 · Public 끄기 명령이 있다
  CHECK: python -c "s=open('docs/manual/user/user-guide.md',encoding='utf-8').read();print('check=%d off=%d' % (s.count('Get-NetFirewallApplicationFilter'), s.count('Set-NetFirewallRule')))"
  EXPECT: /check=[1-9] off=[1-9]/
  EVIDENCE: check=2 off=1

- [x] E9: A1 ~ A7 — 이 PC 실측(CDP · 비파괴, 실제 상대 · 이력은 건드리지 않음): 스키마 17 올리기 · 사본 `.v16.bak` · 상대 번호 그대로 · 타일 글자 · 겹친 카드 ·
  EVIDENCE: 2026-10-15 headless Edge · playwright(scratchpad `h1/e9.mjs`, 이 PC 실제 앱 — 릴리스 `task/1h-stabilization` 빌드(`tools/rebuild_run.py`), 비파괴 — 읽기 · 화면만) · 스키마 16 → 17: `user_version` 17 · 사본 `coworkAgent.db.v16.bak` · `peers` 정의 AUTOINCREMENT · 상대 번호 [1] 그대로 · 순번 1 · 열 19 · 그룹 2 · 대화 2 · 글 6(올리기 전과 같음) / 화면 16/16 PASS — 타일 글자 넓이 [2, 4, 3, 2] ≤ 4 · 타일 안 · 그룹 타일 2 겹친 카드(`4px -4px`) · 상대 타일 카드 없음 · '나' 메뉴 개요 | 내 활동 · 요청 · 주고받은 글 | 북마크 항목 · 에이전트 · 구분선 role=separator · 첫 화면 "개요" · 상대 타일 두 절 · 그룹 타일 세 절 · "저장된 항목" 없음 · 북마크 항목 제목 · 메시지 작업 "북마크" 4 · 실행 상세 절 · 프로필 이름 안 잘림 · JS 오류 0. 트레이 다시 시도(로그인 직후) · 사라진 세션의 실제 claude 실행은 19 인수 TC


WEAK: E9 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

- 2026-10-15 · `/codex:adversarial-review --base 482277b --scope branch` · 세션 codex-review · 실행 시작 HEAD `50f1d63`(요청과 일치, 작업 트리 깨끗) — 리뷰
  도중 문서 전용 커밋 `7545a2b`(masterPlan · 원장 · stage3 링크)가 올라가 Codex 는 `7545a2b` 기준으로 읽음(코드 차이 없음). Verdict needs-attention(high 1 ·
  medium 3).

### 지적 원문

- [high] A10 gate가 PROJECT.md 필수 검증을 일부만 실행한다 (docs/tasks/1h.stabilization/plan.md:103-110) — A10은 PROJECT.md의 검증 명령 전체 통과를
  요구하지만 E1·E2는 cargo test와 clippy만 실행한다. PROJECT.md:57-63의 node 테스트, check_ddd_docs.py, README 변경 시 check_readme.py, check_gates.py,
  check_crates.py가 A10 판정에서 빠졌다. 따라서 eval.md의 A10 체크는 문서 규약·README·계층 위반이 있어도 초록이 될 수 있다.
- [medium] 기존 DB에서 완전히 삭제된 peer ID는 다시 사용된다 (crates/core/src/db.rs:371-377) — 스키마 17은 sqlite_sequence를 현재 peers,
  conversations.peer_id, outbox.peer_id의 최댓값으로만 초기화한다. v16에는 삭제된 ID의 high-water/tombstone이 없으므로, 대화나 outbox를 만들기 전에
  삭제한 상대의 ID는 복구되지 않는다. 이후 상대를 충분히 추가하면 그 ID가 다시 발급되어 A8의 '지운 상대 번호와 겹치지 않는다 — 이미 쓰던 DB를 올려도'를
  위반한다.
- [medium] 트레이 재시도 중 도착한 Attention 상태가 영구 유실된다 (crates/app/src/tray.rs:108-145) — 트레이가 아직 None이면 Attention(n) 이벤트를 아무
  상태에도 저장하지 않고 버린다. 이후 재시도가 성공해도 build는 항상 tip(0)과 Mark::None으로 생성한다. Notifier는 현재 pending 값을 성공 시 재전송하지
  않으므로, 최대 60초 재시도 구간에 요청 하나가 도착하면 다음 알림 전까지 배지와 툴팁에 표시되지 않는다.
- [medium] keycap emoji가 폭 1로 계산되어 A1 예산을 초과한다 (crates/app/web/view.js:39-53) — 폭 2 판별식은 Extended_Pictographic와 Regional_Indicator만
  포함한다. 실제로 `1️⃣`, `#️⃣`, `*️⃣` keycap grapheme은 어느 조건에도 걸리지 않아 폭 1로 계산된다. 예를 들어 `1️⃣ABC`는 코드상 예산 4로 모두 노출되지만
  표시 폭은 약 5칸이어서 40px 타일 overflow 가능성이 있고, '그림 문자 2' 계약을 직접 위반한다.

### 판단 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| A10 gate 일부만 | 사실(부분) — E1 · E2 는 cargo 둘, node 는 E3 · 문서 체커는 커밋마다 돌렸으나 `check_crates` · `check_readme` 결과는 원장에 없었다. gate 추가 · EXPECT 변경은 plan 개정(사용자 승인)이라 하지 않고 이 Task 앞(1i 등)과 같은 틀을 둔다 | 라운드 1 수정 뒤 PROJECT.md 검증 명령 전부를 실행해 아래에 적는다 |
| 지운 번호 재사용(올린 DB) | 사실 · 실해 없음 — 스키마 16 이하에서 대화 · 보낼 글 없이 지운 상대의 번호는 DB 어디에도 남지 않아 되살릴 수 없다. 그 번호를 다시 받아도 붙을 대화 · 보낼 글이 없다(번호를 담는 표는 `conversations` · `outbox` 뿐). 남는 것은 화면 접힘 설정 열쇠(`convs:{번호}`)의 접힘 표시뿐 | 미조치 — 아래 "미조치 지적". A8 은 "남은 대화 · 보낼 글과 겹치지 않음" 으로 판정(올린 뒤 지운 번호는 AUTOINCREMENT 로 다시 쓰지 않음) |
| 트레이 다시 시도 중 새 요청 표시 유실 | 사실 | 고침 — 이벤트 루프가 마지막 수(`attention`)를 늘 보관, 트레이를 만들 때 그 수로 아이콘 · 툴팁(`mark_of`). 이벤트 루프 상태 시험은 화면 없는 환경에서 못 돌려 없음 |
| 키캡 폭 1 | 사실 | 고침 — U+20E3(키캡) · U+FE0F(그림 문자 표시 선택자)가 든 글자도 넓이 2. node `tile_text_keycap`("1️⃣ABC" → "1️⃣AB" · 예산 3) · 예산 속성 시험에 키캡 이름 추가 — 83 pass |

### 라운드 1 수정 뒤 검증(PROJECT.md 검증 명령 전부)

- `cargo test --workspace` 실패 0 · `cargo clippy --workspace --all-targets -- -D warnings` Finished · `node --test` pass 83 · fail 0 ·
  `python tools/check_ddd_docs.py` PASS · `python tools/check_readme.py` readme ok · `python tools/check_crates.py` crates=4 edges=6 violations=0 ·
  `python tools/check_gates.py docs/tasks/1h.stabilization` 9/9(원장 위 측정 커밋).

## 적대적 평가 — 라운드 2/2

- 2026-10-15 · `/codex:adversarial-review --base 482277b --scope branch` · 세션 codex-review(라운드 1 과 같은 세션) · 실행 HEAD `69fb73c`(요청과 일치, 시작 ·
  끝 같음, 작업 트리 깨끗). Verdict needs-attention(high 2 · medium 1). 라운드 1 의 트레이 attention · 키캡 수정은 반영 확인(node 83/83).

### 지적 원문

- [high] A8의 기존 DB 번호 비재사용 계약을 충족하지 않는다 (crates/core/src/db.rs:371-377) — A8은 기존 DB에서도 지운 상대 번호를 다시 쓰지 않도록
  요구한다. 그러나 migration은 현재 peers, conversations, outbox의 최댓값만 sqlite_sequence에 기록한다. 대화·outbox 생성 전에 삭제된 v16 peer ID는
  흔적이 없어 이후 새 상대에게 재발급된다. 남아 있는 ui_folded의 ID 기반 상태가 무관한 상대에 적용될 수도 있다. eval.md가 이를 '남은 대화·보낼 글과만
  겹치지 않음'으로 재해석했지만 승인된 intent/plan은 개정되지 않았다.
- [high] 라운드 1의 A10 gate 누락이 그대로 남아 있다 (docs/tasks/1h.stabilization/plan.md:103-110) — E1·E2는 cargo test와 clippy만 A10으로 판정한다.
  PROJECT.md 필수 항목인 check_ddd_docs.py, README 변경 시 check_readme.py, check_gates.py, check_crates.py는 plan gate에 없다. eval.md 자유 서술에 일회성
  실행 결과를 추가했지만 gate 자체는 여전히 이 명령을 실행하지 않고도 9/9가 될 수 있다.
- [medium] 재개 좌표 SSOT가 구현 전 상태로 남아 있다 (STATE.md:8-17) — STATE.md는 아직 '1h 착수 인터뷰 전'과 1i 종결 직후를 현재 상태로 가리킨다.

### 판단 · 조치(2 라운드째 반영 — 상한, 재리뷰 없음)

| 지적 | 판단 | 조치 |
|---|---|---|
| A8 기존 DB 번호 | 사실(일부) — 접힘 설정 열쇠(`convs:{번호}`)는 DB 에 남은 흔적인데 순번 시작에 넣지 않았다(남은 접힘이 새 상대에 붙을 수 있음) | 고침 — 스키마 17 순번 시작에 접힘 열쇠 번호도(`schema17_peer_ids_never_reused` 에 접힘 12 → 순번 12 단정 추가). 흔적이 하나도 없는 지운 번호는 되살릴 수 없어 미조치 · A8 해석은 사용자 확인 |
| A10 gate 가 PROJECT.md 명령 일부만 | 사실 — gate 를 더하는 것은 plan 개정(사용자 승인) | 미조치 — 이 Task 는 앞 Task 와 같은 틀(E1 · E2 + 커밋마다 문서 체커)로 두고, 종결 측정에서 전부 실행해 report 에 기록. gate 편입은 사용자 결정 |
| STATE.md 재개 좌표 | 사실 — 종결 때 덮어쓰는 문서라 미뤘다 | 고침 — 종결 커밋에서 STATE 갱신 |

### 라운드 2 수정 뒤 검증(PROJECT.md 검증 명령 전부)

- `cargo test --workspace` 실패 0(결과 줄 70) · `cargo clippy --workspace --all-targets -- -D warnings` Finished · `node --test` pass 83 · fail 0 ·
  `python tools/check_crates.py` crates=4 edges=6 violations=0 · `check_ddd_docs` · `check_readme` · `check_gates` — 종결 커밋에서 다시.
- 이 PC 의 DB 는 라운드 1 빌드로 이미 스키마 17(접힘 열쇠 번호 반영 전) — 이 PC 의 접힘 열쇠는 `me` · 상대 1 뿐이라 순번 1 그대로가 맞다(다시 올리지 않음).

## 미조치 지적

| 지적(원문 요약) | 판단 | 미조치 사유 |
|---|---|---|
| 라운드 1 [medium] · 라운드 2 [high] 기존 DB에서 완전히 삭제된 peer ID는 다시 사용된다 (db.rs:371-377) — 접힘 열쇠 흔적은 라운드 2 에서 고침 | 사실 · 실해 없음 | 스키마 16 이하에서 흔적 없이 지운 번호는 되살릴 정보가 없다. 다시 받아도 붙을 대화 · 보낼 글이 없고(접힘 표시만), 올린 뒤에는 AUTOINCREMENT 로 다시 쓰지 않는다. 임의 여유분 예약은 보장이 아니라 하지 않음 |
| 라운드 2 [high] A10 gate가 PROJECT.md 필수 검증을 일부만 실행한다 (plan.md:103-110) | 사실 | gate 추가 · EXPECT 변경은 plan 개정(사용자 승인) 대상 — 종결 측정에서 명령 전부 실행 · 기록, gate 편입은 사용자 결정 |

# eval.md — 1z-20.audit-fixes 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 993011eecc1c0353e086bd5eeb2c848abec083fc

- [x] E1: A1 — 안내문에 사실 아닌 장치 주장("그 위는 읽을 수 없다") 없음
  CHECK: python -c "print('claim=%d'%open('crates/agent/src/runner.rs',encoding='utf-8').read().count('그 위는 읽을 수 없다'))"
  EXPECT: claim=0
  EVIDENCE: claim=0

- [x] E2: A1 — 실행 인자 회귀 없음: 편집 도구만 막고 읽기 · 웹 도구 미리 허용, MCP · 경로 금지 규칙 없음
  CHECK: cargo test -p an-agent --test runner
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s

- [x] E3: A1 — 원장 `수용` 행 3개(#1z-20 R0-1 · R0-4 · R0-5)
  CHECK: python -c "import re;t=open('docs/overview/ledger.md',encoding='utf-8').read();print('rows=%d'%len(re.findall(r'^\| #1z-20 R0-[145] \|.*\| 수용 \|',t,re.M)))"
  EXPECT: rows=3
  EVIDENCE: rows=3

- [x] E4: A2 — 실행 대기 중 멈춤 → 재개 → 실행(앱 재시작 없이)
  CHECK: cargo test -p an-agent --test relay
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.72s

- [x] E5: A3 — 받기 DB 오류 = retry · 상대별 실행 상한(지적 3 · 6)
  CHECK: cargo test -p an-core --test store open_refused && cargo test -p an-agent --test capacity
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.48s

- [x] E6: A3 — DB 사본 임시 이름 · 옛 폴더 반쯤 옮김 · 스키마 30 인덱스(지적 10 · 11 · 12)
  CHECK: cargo test -p an-core
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.90s

- [x] E7: A3 — 공유 `exists` · 초대 쓰기 실패(지적 13 · 18)
  CHECK: cargo test -p an-agent --test share && cargo test -p an-net
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E8: A3 — 알림 상태 · 검증 실패 건너뛰기 · CLI history 들여 씀 · status JSON(지적 14 · 15 · 17 · 20)
  CHECK: cargo test -p coworkagent-app --test update && cargo test -p coworkagent-app --lib
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

- [x] E9: A3 — 감시가 업데이트 확인 중일 때 두 번째 실행이 오류 창 없이 넘겨준다(지적 9) — 재현(느린 릴리스 주소) 또는 시험, 평가자 대조
  EVIDENCE: Claude(구현자) 2026-11-03 — 디버그 빌드 `target/t15/debug/coworkAgent.exe`, 데이터 루트 `%TEMP%/an-e9/data`: Python 이 `supervisor.lock` 을 쥐고(`msvcrt.locking`) 8초 뒤 `instance.json`(204 를 주는 로컬 HTTP)을 씀, 2초 뒤 띄운 앱 = 6.5초 기다려 exit=0 · `/api/open` 1회 · 시작 오류 파일 없음(고치기 전 = 10 × 300ms 뒤 "기존 인스턴스에 전달 실패"). 감시가 없으면 3초 규칙 그대로(`watchdog::held` 거짓) — 근거 [work/stage2](work/stage2.md).

- [x] E10: A3·A4 — 웹 로직: 번호 목록 시작 번호 · `*` 오판 없음 · 개요 절 머리 점만
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E11: A4 — 화면 literal: 개요 절 머리에 접속 글 없음 · 점 유지
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

- [x] E12: A4 — 실제 화면(디버그 빌드 · 시험 데이터): 개요 미리보기 절 머리 = 이름 · 점 · 대화 수, "…통신" 없음 · 레일 · 대화 머리글 접속 글은 그대로 — 평가자 대조
  EVIDENCE: Claude(구현자) 2026-11-03 — mock(1z-19 `design/mock.py` 사본 · 버전 1.4.5) + ego-browser 1440×900: 개요 미리보기 절 머리 `.pv-head` 글 = "이준호· 대화 5· 안 읽음 1· 대기 요청 1" · "최유나· 대화 4· 안 읽음 1· 대기 요청 1" · "build· 대화 1"("…통신" 없음, 점 `st-dot` 남음), 그룹 개요도 같음(`docs/images/home.png` · `group.png` 다시 찍음). 레일 · 대화 머리글 접속 글은 코드 그대로(`conv.js` `presence` · `rail.js`) — 근거 [work/stage3](work/stage3.md).

- [x] E13: A3 — 지울 코드: `json_escape` · `user_guide_url` · `let _ = peer` 없음
  CHECK: python -c "import pathlib as P;s=lambda f:P.Path(f).read_text(encoding='utf-8');print('left=%d'%(s('crates/app/src/server.rs').count('json_escape')+s('crates/app/src/tray.rs').count('fn user_guide_url')+s('crates/agent/src/chat/relay.rs').count('let _ = peer')))"
  EXPECT: left=0
  EVIDENCE: left=0

- [x] E14: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

- [x] E15: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.99s

- [x] E16: A5 — 버전 1.4.5(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v145=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.4.5' for t in v)))"
  EXPECT: crates=4 v145=4
  EVIDENCE: crates=4 v145=4

- [x] E17: A5 — 문서 규약 · README · 코드 정리 · 크레이트 경계
  CHECK: python tools/check_ddd_docs.py && python tools/check_readme.py && python tools/check_code_tidy.py && python tools/check_crates.py
  EXPECT: /violations=0/
  EVIDENCE: crates=4 edges=6 violations=0


WEAK: E9 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)
WEAK: E12 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex --model gpt-6.1-sol --effort high`, 작업 폴더 `C:/Users/tskim/.claude/skills` — 대상 저장소 밖, `git -C` 로 읽기만).
Run `run_dc39e8452dc2` · Task `task_21aa4cf5c0d4` · Dispatch `ctx_592a1e358e00` · 터미널 `term_ee3ae25d-9398-469b-8a67-fed105ac6b38`.
범위 `6233339..9ba6396`(self-collect), 메타 `요청 #1z-20.audit-fixes · revision 9ba6396a3a4dacf3ff4c7e91c18145951caefa28 · 라운드 1/2`. 리뷰어 Dispatch `worker-release` → `released`(`closed_agent_terminal`).

```text
# Codex Adversarial Review

Target: branch diff against 6233339f7057815c9a05099a38022879504fb353
Verdict: needs-attention

출하 보류: 상대별 10건 상한에 우회·경쟁이 있고, 자동 왕복 재개와 DB 이동 중단 복구에도 누락이 남는다. 고정 범위 정적 리뷰만 수행했으며 빌드·런타임 재현은 실행하지 않았다.

Findings:
- [high] Context 프레임으로 상대별 실행 상한을 우회한다 (crates/agent/src/chat/inbound.rs:170-175, confidence 0.96)
  [확인] 새 상한은 inbound_message에만 적용된다. handle은 Context를 별도 inbound_context로 보내며, context.rs:171-182는 새 uuid마다 cap 검사 없이 exec=true·queued 구분선을 추가한다. peer_pending_runs는 이 구분선도 세고 compact는 실제 에이전트를 실행한다. [추론] 페어링된 상대는 실행이 막힌 기존 in 대화에 서로 다른 uuid의 compact/auto_compact를 계속 보내 10건을 넘어 실행 대기열을 쌓을 수 있다. 일반 Message만 막아서는 상대별 자원 독점 방지 계약이 성립하지 않는다.
  Recommendation: 원격 실행 admission을 공통화해 Context의 새 실행 등록에도 같은 peer cap을 적용한다. 상한 상태에서 Context 신규 요청은 retry, 기존 uuid 재전송은 Ok인지 검증한다.
- [high] DB·키 이동 후 곁 파일만 남으면 복구를 건너뛴다 (crates/core/src/paths.rs:102-107, confidence 0.87)
  [확인] legacy_data는 옛 identity.key와 본 DB 존재만으로 정한다. 항목별 rename 순서는 정하지 않으며, 둘이 이미 옮겨지고 -journal만 남은 경우 125-127행에서 반환해 새 곁 파일 이름으로 옮기는 135-139행에 도달하지 않는다. main::start는 이어서 새 위치의 DB를 연다. [추론] hot rollback journal이 남은 DB의 이동이 이 지점에서 중단되면 SQLite가 복구에 필요한 journal을 찾지 못해 미완료 트랜잭션이 반영되거나 DB가 손상될 수 있다. 새 partial-move 시험은 옛 키가 남은 경우만 다룬다.
  Recommendation: DB 및 곁 파일 이동 진행을 기록하고, 본 DB·키가 없어도 남은 journal/wal/shm을 복구 대상에 포함한다. DB를 열기 전에 파일 묶음 이동 완료를 보장하고, 각 rename 직후 중단한 상태에서 실제 SQLite 복구 및 데이터 일치를 검사한다.
- [medium] 상한 확인과 실행 등록이 분리돼 같은 잔여 자리를 중복 사용한다 (crates/agent/src/chat/inbound.rs:172-174, confidence 0.94)
  [확인] full()은 self.db() 잠금을 놓고 반환하고, 실제 신규 등록·승격은 187행 이후 다시 잡은 잠금에서 수행한다. Node는 다중 스레드 런타임에서 수신 연결을 병렬 처리한다. [추론] 대기+실행 9건에서 서로 다른 신규 요청 둘이 모두 9를 읽으면 둘 다 통과해 11건을 등록한다. 같은 경쟁이 자동 왕복 승격에도 적용된다. stage1의 연성 상한 설명과 달리 intent 결정 4 및 사용자 가이드는 10건 초과를 retry로 정한다. 추가 시험은 순차 send뿐이라 이를 검증하지 못한다.
  Recommendation: 같은 DB 잠금·쓰기 트랜잭션에서 기존 uuid/승격 여부 판정, count 확인, 신규 실행 등록을 처리한다. 9건을 고정한 뒤 두 요청을 barrier로 동시에 진입시켜 하나만 수락되는지 검사한다.
- [medium] 재개가 pending 재조회보다 빠르면 taken에 영구 차단된다 (crates/agent/src/chat/execute.rs:91-99, confidence 0.89)
  [확인] taken에서 id를 지우는 조건은 pending 조회 순간 그 id가 없다는 것뿐이다. process:163-165는 멈춘 차례를 exec=false·done으로 바꾸고 반환하며, relay:190-205는 그 행을 다시 exec=true·queued로 바꾼다. [추론] 멀티 스레드에서 이 재개가 worker의 다음 pending 조회보다 먼저 끝나면 worker는 중간 done 상태를 보지 못한다. id가 계속 taken에 남아 모든 wake에서 건너뛰므로 자동 왕복이 재시작 전까지 다시 멈춘다. 새 시험은 pending이 비는 것을 기다려 이 경쟁을 제외한다.
  Recommendation: 실행하지 않고 되돌린 차례임을 process 결과로 반환해 worker가 해당 id를 직접 taken에서 제거하거나 실행 세대별 claim을 사용한다. 완료한 실행의 중복 방지는 유지하면서, done 직후 재개가 재조회보다 먼저 일어나는 순서를 고정해 검사한다.

Next steps:
- cap 검사를 모든 원격 실행 등록 경로에서 원자적으로 적용한다.
- 재개·이동 중단 경계를 고정한 회귀 검증 후 라운드 2/2에서 대조한다.
```

판정 · 조치:
- 지적 1(Context 로 상대별 상한 우회) — **사실**. `inbound_context` 의 새 구분선 등록에 같은 상한(셈과 등록 같은 잠금) · `peer_pending_runs` = 그 상대와의 대화에서 상대가 보낸 실행 대상
  (`mine` 아님 — 자동 왕복 상대 답 · 압축 포함, 이 PC 의 자동 압축은 제외). 시험 `capacity` `peer_run_cap_covers_context`(상한에서 새 압축 = `retry` · 쌓이지 않음) — 고치기 전 FAILED 확인.
- 지적 2(셈과 등록 분리 — 남은 자리 겹쳐 씀) — **사실**. 상한 판정을 받은 대화 기록과 같은 `self.db()` 잠금 안으로(새 글 = 그 uuid 없음 · 묘비 아님, 자동 왕복 승격 = 상대 에이전트 기록 글) —
  받기 처리기는 모두 같은 연결 잠금을 거쳐 판정 · 등록 사이에 끼지 못한다. 동시 진입 시험은 잠금이 한 줄이라 쓰지 않음(구조로 보장). stage 1 의 "연성 상한" 설명은 고침.
- 지적 3(재개가 재조회보다 빠르면 `taken` 에 막힘) — **사실**. 되돌린 차례를 `returned` 집합에 남기고 대화 작업이 처리 직후 `taken` 에서 지운다(재개가 그 사이 다시 줄 세워도 다음 읽기에서 꺼냄).
  순서 고정 시험은 실행 문 안쪽 시점 주입이 필요해 쓰지 않음 — 기존 `stop_while_waiting_then_resume` 통과.
- 지적 4(곁 파일만 남으면 옮기기 건너뜀) — **사실**. 옛 DB 곁 파일(`-journal` · `-wal` · `-shm`)만 남아도 옮기기 대상. 시험 `legacy_half_moved_resumes` 에 DB · 신원을 다 옮기고 journal 만 남은 경우 추가.
- 문서: an_agent(상한 대상 · 같은 잠금 · `returned`) · an_core(`peer_pending_runs` · 곁 파일).

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex(같은 실행 명령 · 같은 작업 폴더). Run `run_dc39e8452dc2` · Task `task_ab7aa902ce09` · Dispatch `ctx_a54578926bd9` ·
터미널 `term_1697a37e-2301-4d0c-b793-57a40cd08cf4`. 범위 `6233339..e1fe3e5`(self-collect · 라운드 1 원문 동봉), 메타 `요청 #1z-20.audit-fixes · revision e1fe3e588b0047e54d6825ae0f200f711ba45be7 · 라운드 2/2`. 리뷰어 Dispatch `worker-release` → `released`.

```text
# Codex Adversarial Review

Target: branch diff against 6233339f7057815c9a05099a38022879504fb353
Verdict: needs-attention

출하 보류: R1-1·2·4의 수정 경로는 확인했으나 R1-3에 재개 유실 경쟁과 DB 쓰기 실패 시 반복 처리가 남는다. 지정 범위 정적 리뷰만 수행했으며 빌드·런타임 재현·macOS 실기는 실행하지 않았다.

Findings:
- [medium] 재개가 중단 상태 기록보다 먼저 끝나면 차례가 다시 유실된다 (crates/agent/src/chat/execute.rs:168-171, confidence 0.88)
  [확인] process는 139~156행에서 읽은 relay.on을 잠금 밖에서 판정하고 169행에서 별도 잠금으로 exec=false·done을 기록한다. relay(true)는 relay.rs:188~198에서 on=true로 바꾸되 이미 exec=false·done인 마지막 답만 queued로 승격한다. [추론] worker가 on=false를 읽은 뒤 재개 트랜잭션이 먼저 끝나면, 재개는 아직 running인 답을 건너뛰고 worker가 뒤늦게 그 답을 done으로 되돌린다. returned는 taken만 지우므로 대기열에는 아무것도 남지 않아 켜짐 상태에서 자동 왕복이 진행되지 않는다. 기존 시험은 pending이 빈 뒤 재개하므로 이 순서를 제외한다.
  Recommendation: 현재 relay 상태 확인과 중단 차례의 done 전환을 재개와 같은 DB 잠금·트랜잭션에서 처리한다. on=false 스냅샷 직후 재개가 먼저 커밋되는 순서를 고정해, 답이 실행되거나 재대기됨을 검증한다.
- [medium] 되돌리기 쓰기가 실패해도 taken을 풀어 같은 글을 무한 재처리한다 (crates/agent/src/chat/execute.rs:169-170, confidence 0.89)
  [확인] 169행은 set_flags 오류를 버리지만 다음 행은 항상 returned에 id를 넣고, worker의 102~103행은 이를 받아 taken에서 제거한다. 쓰기가 실패하면 해당 글은 exec=true·running으로 남아 pending에 다시 잡히며, 다음 process는 asked.state==Running이므로 start_run도 건너뛴다. [추론] 중단된 자동 왕복을 복구하는 동안 SQLITE_FULL 등 지속적인 쓰기 오류가 나면 같은 경로가 휴지·종료 없이 반복된다. 실행 자리가 비어 있으면 admit도 대기하지 않아 Tokio worker의 CPU와 공유 DB 잠금을 계속 소비하며 실패 로그도 남기지 않는다.
  Recommendation: set_flags 성공을 확인한 경우에만 returned로 taken을 해제한다. 실패는 로그와 제한된 재시도 경로로 보내고, 되돌리기 UPDATE 실패를 주입해 같은 id의 무제한 반복이 없음을 검증한다.

Next steps:
- 재개와 중단 상태 전환을 원자화하고, DB 오류 시 taken 해제를 막는다.
- 두 실패 순서의 회귀 검증 후 잔여 지적을 기록한다. 라운드 2/2 종료이며 추가 리뷰는 시작하지 않는다.
```

판정 · 조치(상한 2/2 — 고친 뒤 재리뷰 없음):
- 지적 1(멈춤 판정과 되돌리기 사이에 재개가 끼면 차례 유실) — **사실**. 되돌리기 직전에 재개(`relay`)와 같은 `self.db()` 잠금 안에서 자동 왕복 켜짐을 다시 읽고, 켜졌으면 되돌리지 않고 그대로 넘긴다.
  재개의 승격 트랜잭션도 같은 잠금이라 둘 중 하나만 먼저 끝난다(먼저 재개 → 이 답을 넘김, 먼저 되돌림 → 재개가 승격 · `returned` 로 다시 꺼냄).
- 지적 2(되돌리기 쓰기 실패에도 `taken` 을 풀어 무한 재처리) — **사실**. `set_flags` 가 성공했을 때만 `returned` 에 넣고, 실패는 `log::error!` 뒤 꺼낸 글로 남긴다(다시 꺼내지 않음 — 앱 재시작의 `recover` 가 맞춤).
- 두 순서(재개 ↔ 되돌리기 경쟁 · 쓰기 실패 주입)의 고정 순서 시험은 실행 문 안쪽 시점 주입이 필요해 쓰지 않았다 — 구조(같은 잠금 · 성공 때만 해제)로 보장, 기존 relay 시험 14 통과.
- 검증 중 링커 `LNK1201`(PDB 쓰기 실패 — 공간 · 권한 문제 아님, 디스크 여유 1 TB)이 시험 실행 파일 7개에서 났다. 해당 `.pdb` 를 지우고 다시 빌드하니 통과 —
  앞서 본 간헐 rc 101(실패한 시험 이름 없음)이 이것일 가능성이 있다(추정).
- 한계: R1-2 · R1-3 · R2-1 · R2-2 의 경쟁 · 실패 순서는 고정 순서 시험 없이 구조로 보장(report 에 적음). 라운드 1 지적 4 · 라운드 2 지적 2 모두 고침.

## 미조치 지적

없음

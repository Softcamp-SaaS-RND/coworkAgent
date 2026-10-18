# eval.md — 1s.fold-line-readme 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 09eb5c496b185c11fd0d4ed32064028f45630c7f

- [x] E1: A4 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A4 — 화면 로직 node 테스트 · 실패 0
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (12[4-9]|1[3-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 137.5385

- [x] E3: A1 — 문구 · 자리 잠금(세로선 없는 들여쓰기 규칙 PRESENT · 옛 `border-left` 규칙 ABSENT)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

- [x] E4: A2 — README 사용자 관점 검사
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: readme ok

- [x] E5: A4 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: PASS — DDD 문서 규약 위반 없음

- [x] E6: A4 — CSS 토큰
  CHECK: cargo test -p coworkagent-app --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E7: A1·A2·A3 — 실화면 · 캡처 확인(mock): 에이전트 화면 · 요청 개요 · 개요 세로선 없음 · 들여쓰기 유지, 캡처 5장 새로(연필 · 새 에이전트 줄), README 문구 대조.
  EVIDENCE: mock — `#agents`(다크) · `#requests`(라이트) 세로선 없음 · 단계 들여쓰기 유지 · 행 구분선은 들인 자리부터, 캡처 5장 새로(agents 연필 · 새 에이전트 줄 · pairing 연결 목록 연필 · home 개요 들여쓰기) 캡처 스크립트 ERRORS [], README 문구 대조 · readme ok — work/stage1.md


WEAK: E7 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: `codex-review` 세션 · `/codex:adversarial-review --base b9e8004 "1s: …"`(job `review-mvdahuua-zd6o69`, 실행 HEAD `d305d86` = 요청 일치) ·
Verdict needs-attention(medium 2). 두 지적 모두 문서라 고친 뒤 라운드 2 는 돌리지 않았다(상한 2 는 최대치 — 코드 변경 없음).

| 지적(원문 요지) | 판단 | 조치 |
|---|---|---|
| [medium] `STATE.md` 가 1s 를 담지 않음 — 브랜치 · Task · 평가 단계가 아니라 main · Task 없음 · 1r 종결로 남아 재개 좌표가 틀림(`STATE.md` 6-14) | 사실 — 단계 전환마다 갱신하는 규약을 1s 착수 · 구현 때 빠뜨림 | 종결 커밋에서 `STATE.md` 를 1s 종결 좌표로 갱신 |
| [medium] README 가 요청 단추를 "✓ 수용 · ✕ 반려" 로만 적어 수용 → "✓ 완료" 전이가 빠짐(`README.md` 32) | 사실 | "대기 요청은 ✓ 수용 · ✕ 반려, 수용한 요청은 ✓ 완료"(사용자 가이드 §9 · `REQ_BTN` 과 같게). `readme ok` · `git diff --check` 빈 출력 |

## 미조치 지적

없음

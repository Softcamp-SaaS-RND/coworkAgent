# plan — 1s.fold-line-readme

> 근거: [intent](intent.md)(승인 2026-10-18 — 결정 1 ~ 3). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `b9e8004`(main — 1r 종결). plan 승인: 2026-10-18(사용자 지시 "위 작업까지 진행하고 readme 문서 업데이트까지 완료." — 작은 범위라 intent · plan 을 함께).

## 목표

접이식 하위 줄의 세로 안내선을 지워 들여쓰기만 남기고, README 를 1q · 1r 화면에 맞춘다(문구 · 캡처).

### 성공 기준(intent 사본)

- A1: 접이식 절 몸에 세로선(`border-left`)이 없고 단계 들여쓰기는 그대로다(개요 · 내 활동 · 요청 · 주고받은 메시지 · 에이전트 화면).
- A2: README 기능 표 · 시작하기가 요청 개요 · 내 활동 범위 · 내 응답 에이전트 · 최대 응답 횟수 · 메뉴 초대 생성 · 새 에이전트 줄 · 연필 편집을 설명하고, `check_readme` 가 통과한다.
- A3: README · 사용자 가이드 캡처 5장이 지금 화면(연필 · 새 에이전트 줄 · 들여쓰기)이다.
- A4: 기존 검증 명령이 green 이다.

## 설계

- `app.css` — `.peer-fold > :not(summary) { margin-left: var(--sp-5); }`(세로선 지움, 좁은 화면 `--sp-3` 그대로). `ux_literals` 1r 잠금 행을 새 규칙으로, ABSENT 에 옛 규칙.
- README — 기능 표 5행 · 시작하기 2 ~ 4.
- 캡처 — `python docs/archive/1q.request-overview/design/mock.py crates/app/web {port} dark` + `node docs/archive/1p.project-tidy/design/shots.mjs {port} docs/images dark doc`.
- 문서 — web-ui "접이식 들여쓰기" · masterPlan D35 (#1r) 줄의 "안내선".

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 세로선 제거 · README · 캡처 · 문서 | css · 잠금 · README · 이미지 5 · 문서 | E1 ~ E7 |

## 평가 항목

- [ ] E1: A4 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A4 — 화면 로직 node 테스트 · 실패 0
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (12[4-9]|1[3-9][0-9])\s+# fail 0/
  EVIDENCE: pending
- [ ] E3: A1 — 문구 · 자리 잠금(세로선 없는 들여쓰기 규칙 PRESENT · 옛 `border-left` 규칙 ABSENT)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E4: A2 — README 사용자 관점 검사
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: pending
- [ ] E5: A4 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: pending
- [ ] E6: A4 — CSS 토큰
  CHECK: cargo test -p coworkagent-app --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E7: A1·A2·A3 — 실화면 · 캡처 확인(mock): 에이전트 화면 · 요청 개요 · 개요 세로선 없음 · 들여쓰기 유지, 캡처 5장 새로(연필 · 새 에이전트 줄), README 문구 대조.
  EVIDENCE: pending

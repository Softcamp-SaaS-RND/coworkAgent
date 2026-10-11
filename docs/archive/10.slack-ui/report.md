# report — 10.slack-ui

종결: 2026-10-11 · 브랜치 `task/10-slack-ui` → main

## 결과

웹UI 를 시안 A 3단(PC 레일 · 사이드바 · 대화) + C 요청 타임라인 + 레일 하단 설정으로 다시 짰고(D26), 화면 판단 로직을 `view.js` 로 빼서
`node --test` 기반을 들였다. 답에 실행 정보(턴 · 시간)와 원 요청 대응을 실어 양쪽에 남긴다(스키마 5).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 3단 첫 화면 · 설정에 관리 기능 · 트레이 내 정보 | 정적 충족 · 실화면 ABANDON | E9 · E10(D28) |
| A2 레일 거르기 · 새 대화 · 이어 쓰기 · 전환 · 늦은 응답 폐기 | 로직 충족 · 실화면 ABANDON | E6 · E7(`filterByPeer` · `keepsConversation` · `Latest`) · E10 |
| A3 C 타임라인(요청 → 실행 · 턴 · 시간 → 응답) | 로직 충족 · 실화면 ABANDON | E6 · E7(`timeline` · `linkReplies`) · E10 |
| A4 답 실행 정보 · 원 요청 대응 · 이전 버전 호환 | 충족 | E3 · E4 · E5(`reply_carries_run_meta` · `reply_without_meta_accepted`) |
| A5 node 테스트 · 검증 명령 표 | 충족 | E7 · E9 |
| A6 원격 값 텍스트로만 | 충족 | E8 `html_sinks=none` |
| A7 검증 명령 | 충족 | E1 · E2 · E7 |

- 적대적 평가 2 라운드(상한) — high 3 · medium 1 전부 코드 반영: 레일 전환 시 열린 대화 닫기 · 보내기 대상 검증, 스키마 5 이전 원자화 ·
  있는 열 건너뛰기(라운드 1), 스키마 올리기 전 사본(롤백), 이전 이력 답 모호하면 연결 안 함(라운드 2). 미조치 3건은 [eval.md](eval.md) →
  원장.
- 원장 #04 R2-F3(늦은 응답 폐기 JS 테스트) 닫힘. 다른 JS 테스트 대기 3건은 DOM 밖이라 19 수동으로 조건을 바꿨다.

## 산출

- `an-core`: 스키마 5 · `add_reply` · `summaries` · 이전 전 사본(`backup_path`). `an-net`: `Reply` 실행 정보. `an-agent`: `RunOutput` 실행 정보.
- `agentnetwork-app`: 대화 요약 · 메시지 실행 정보 API · `/view.js` · 웹UI 다시 씀(`index.html` · `app.js` · `app.css` · `view.js` ·
  `view.test.mjs`).
- 루트 `PROJECT.md` 검증 명령에 웹UI 화면 로직(node) 행.

## 남은 것

- 실화면 확인 — `19.prototype-acceptance`(D28). 사람 발화 — `11.two-sided-chat`. 자동 왕복 — `12.auto-relay`.

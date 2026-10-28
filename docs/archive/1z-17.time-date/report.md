# report — 1z-17.time-date

종결: 2026-10-28 · 브랜치 `task/1z-17-time-date`(원 작업 폴더 — worktree 없음) → main squash `94bd4b2`(v1.4.2 릴리스) · 리뷰 수정 `a2bbe55` · `650e8b7`(1.4.3 에 실림 — #1z-18 과 함께).

## 결과

화면 시각에 날짜를 붙이고(설정 → 화면 '날짜 표시방식' 넷, 기본 `M.D HH:MM`), 설정 실행 · 화면 · 시작 · 알림 · 디버깅을 1열 · 최대 900px · 드롭다운 너비 = 가장 긴 옵션으로,
레일 상태 점을 14 · 12px 로, 에이전트 상세 저장 · 삭제에 창을, 에이전트 시스템 프롬프트에서 추가 지시를 맨 앞에 뒀다. 버전 1.4.1 → 1.4.2(릴리스 — 사용자 서명), DB 스키마 그대로(29).

- stage 1 — 설정 `ui_date_format`(서버 검증 · 저장) · `formatTime(unix, { seconds, fmt, now })` · `dateText` · 실행 상세 + 초 · 대화 정보 날짜 중복 없음 · 설정 1열 900px ·
  `.set-tile select` 최소 220px · 상태 점 · `agentSavedText` · `agentSaveFailText` · `agentDeleteAsk` · `extra_prompt`.
- stage 2 — 디버그 빌드 · 시험 데이터 사본 실제 화면(두 테마 · 1000 ~ 1920px) · 매뉴얼 · PRODUCT · README.
- 평가 — check_gates 9/9(측정 `650e8b7`), codex 라운드 1/2 needs-attention 지적 2 → 고침(`a2bbe55`), 라운드 2/2 needs-attention 지적 2 → 고침(`650e8b7`, 상한 — 재리뷰 없음). 미조치 없음.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 화면 시각 = 날짜 표시방식(기본 M.D HH:MM) | 충족 | E3 format_time, E7 대화 · 사이드바 · 요청 의견 · 대화 정보 md · ymd · time |
| A2 설정 '날짜 표시방식' 넷 · 저장 · 즉시 · 모르는 값 400 | 충족 | E4 `date_format_setting_persists`, E5 PRESENT, E7 전환 · 다시 열기 유지, R1 수정 뒤 화면 재확인 |
| A3 실행 상세 + 초 · 디버깅 로그 그대로 | 충족 | E3 run_steps(화면은 시험 데이터에 실행 기록 없음) |
| A4 대화 정보 날짜 중복 없음 | 충족 | E3 conv_info, E7 |
| A5 설정 1열 · 900px · 로그 상자 그대로 | 충족 | E5, E7(1440 · 1920 · 로그 989) |
| A6 레일 상태 점 14 · 12px | 충족 | E5, E7 |
| A7 동작 · 문구 · 검증 · 문서 · 1.4.2 | 충족 | E1 · E2 · E6 · E8 |
| A8 에이전트 저장 · 실패 창 · 삭제 확인 | 충족 | E3 agent_dialogs, E5, E7 |
| A9 추가 지시 맨 앞 · 가드 뒤 | 충족 | E9 runner |

- 편차 · 한계:
  - 리뷰 R1 지적 1 — `relative` · 연도 생략은 자정 · 해 넘김에 그린 시각이 남았다(stage 1 에 '한계' 로 적었던 것). 서명에 `timeKey`(방식 · 지역 날짜)를 붙여 고침 — 5초 새로 고침 안에 다시 그린다.
  - 리뷰 R1 지적 2 · R2 지적 1 — 다른 설정 PUT 의 늦은 응답이 저장한 날짜 방식을 되돌릴 수 있었다. 설정 저장 = `saveSettings`(보낸 키만 반영 — 설정 화면 · 집중 · 알림 일시 중지 · '나' 타일 · 실패 복구).
  - 리뷰 R2 지적 2 — 다시 그리기가 대화의 쓰던 요청 의견을 지울 수 있었다(자정 · 새 글). 의견 칸이 열려 있으면 대화 다시 그리기를 미룸.
  - 수정은 v1.4.2 릴리스 뒤라 1.4.3 에 실린다.
  - 사이드바 대화 제목 칸이 날짜만큼 줄어든다(ymd 62px — stage2-1) — 권장안 그대로.
  - 1z-17 gate 재측정은 리뷰 수정 커밋(`650e8b7`)을 따로 꺼낸 임시 작업 트리에서 했다 — 원 작업 폴더에 #1z-18 미커밋 변경(버전 1.4.3)이 있어서.

## 사용자 결정 필요

> **2026-10-28 사용자 결정 — 종결**: 사용자 "U1. 완결처리." — 남은 결정은 권장안으로 종결한다.

- intent-12 — 추가 지시 = 우리가 덧붙이는 글의 맨 앞(Claude Code 기본 시스템 프롬프트는 그대로): 권장안 그대로 종결.
- stage2-1 — 사이드바 제목 칸 폭: 권장안 그대로(다음 Task 후보 아님).
- 구현자 해석: plan 승인 질문 없이 구현(사용자 "개발 진행해"), 드롭다운 너비 = 가장 긴 옵션(최소 220px).

## 산출

- 코드: `crates/core/src/run_opts.rs` · `crates/app/src/notify.rs` · `crates/agent/src/runner.rs` · `crates/app/web/js/logic/{format,conv,message,agent}.js` ·
  `js/settings/{prefs,agents}.js` · `js/core/dom.js`(R1) · `js/{conv,shell,tiles}.js`(R2) · `index.html` · `app.css` · `crates/*/Cargo.toml` · `Cargo.lock`.
- 시험: `view.test.mjs`(format_time · time_key · agent_dialogs · run_steps · conv_info) · `ux_api.rs`(`date_format_setting_persists`) · `ux_literals.rs` · `crates/agent/tests/runner.rs`.
- 문서: [web-ui](../../manual/dev/web-ui.md) · [an_agent](../../manual/dev/an_agent.md) · [개발자 가이드](../../manual/dev/developer-guide.md) · [사용자 가이드](../../manual/user/user-guide.md) ·
  PRODUCT · README · masterPlan(S4 · D31 · 개정 기록).
- 문서 체인: [intent](intent.md) · [plan](plan.md) · [stage1](work/stage1.md) · [stage2](work/stage2.md) · [eval](eval.md).

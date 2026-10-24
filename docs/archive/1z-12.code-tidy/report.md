# report — 1z-12.code-tidy

종결: 2026-10-24 · 브랜치 `task/1z-12-code-tidy`(원 작업 폴더 — worktree 없음) → main(squash — 사용자 승인 뒤).
한 번 종결한 뒤 사용자 "권장 방안으로 진행." 으로 다시 열어 intent 결정 8 · 9 · stage 6 을 더하고 다시 평가 · 종결했다.

## 결과

코드 정리 — 동작 불변만(사용자 "동작 불변만"). 크레이트 경계 · 공개 API · HTTP API(정적 경로 빼고) · 통신 형식 · DB 스키마 · 버전 1.3.6 그대로.

- Rust: `an_core::chat` 8 · `an_agent::chat` 13(모듈마다 `impl Chat`) · `an_app::chat_api` 7 하위 모듈. 본문 바이트 그대로 옮김, 공개 경로 `an_core::chat::X` 그대로.
- 웹: `app.js`(5,221줄) · `view.js`(1,687줄) → `crates/app/web/js/` 브라우저 ES 모듈 48개(진입 `app.js` · `core/` 6 · UI 도메인 17 · `settings/` 6 · `logic/` 18).
  서버 내장 표 `SCRIPTS` · `GET /js/{*file}` — 표의 이름과 정확히 같을 때만 토큰 없이, 그 밖은 401 · 토큰이 있어도 404. 진입이 맨 먼저 `core/token.js`(토큰이 없으면 거기서 멈춤).
- 죽은 코드: JS 3(`TRASH_SVG` · `redrawMessages` · `svgButton`) · CSS 규칙 12 + `.fp` · 쓰지 않는 import 23, 보완 넷(import `lead` · 변수 `name` · 같은 요소 · CSS 토큰 둘).
- 중복 · 늘 같은 쪽 분기(stage 6 — 결정 9): `delivery.rs` `kick` 의 늘 참인 `if let` 지움 · `chat_api/conversation.rs` 자동 왕복 상한 기본값 식을 지연 클로저 하나로(세션 경로 특성 시험 먼저).
  동작이 같아 보인 다섯 중 셋은 사유와 함께 그대로 — `tray.rs` 종료 두 갈래(합치면 겹친 조건식 — 읽기 나빠짐) · `set_blocked` 두 번(동시 실행에서 끝 상태가 다름) ·
  `convListParts` 다시 거르기(시험 계약) — [work/stage6](work/stage6.md).
- 주석: 결정 이력 표식 1,186 → 0, 주석과 코드가 다른 곳 바로잡음([work/stage4](work/stage4.md) 표), 주석 밖 코드 토큰 그대로(116 파일 differ 0).
  새 검증 명령 `python tools/check_code_tidy.py`(이력 표식 0 · rs · js 1,000줄 상한).
- 문서: 매뉴얼(web-ui 모듈 표 48 · an_app · an_core · an_agent · developer-guide) · PRODUCT · PROJECT · masterPlan D10 보강.
  R1 뒤 찾은 낡은 문서도 고침(R2 가 봤다 — 지적 없음): [security-review](../../overview/security-review.md) 의 토큰 없이 주는 화면 파일(5개 → `/` · `app.css` · `favicon.ico` + `/js/` 정확 이름) ·
  `innerHTML` 문장(한 곳 → 다섯 곳, 모두 고정 SVG 상수), 크레이트 요약 "`claude -p` 실행기" → 에이전트 실행기(`claude -p` · `codex exec`) 4곳.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 동작 불변 | 충족 | E1 · E3 · E4 · E8(stage 6 뒤 새 빌드), 옮기기만 대조(stage1 `rsverify` · stage2 `jsverify`) · 주석 전용 대조(stage4 `cmtonly`) · stage 6 특성 시험 · R1 독립 대조 · R2 동작 차이 없음 |
| A2a 도메인 구조 — 표대로 나뉨 · 층 · 크기 | 충족 | E5(파일 = 표 · import 정합 · 층) · E6(1,000줄) |
| A2b 한 파일 = 한 도메인의 상태 변경 | 기준 개정(결정 8 — 사용자 승인) 뒤 부분 충족 — report-6 | 결정 8 의 흐름(설정값 적용 · 변경 신호 · 순서 저장 · `current` · `focusAfter`) 안의 쓰기는 예외로 충족. 밖 일곱(`shell.js` `previewAll` · `focusMsg`, `rail.js` `unseen`, `sidebar.js` `waitingBy` · `unseenAll`, `conv.js` · `composer.js` `pickMsg`)은 미충족 — R2-1 · [work/stage6](work/stage6.md) 대조표 |
| A3 죽은 코드 0 | 충족 | E2 · [work/stage3](work/stage3.md) 지운 것 · 후보 표 · stage 6 |
| A4 주석 | 충족 | E6 |
| A5 회귀 없음 · 매뉴얼 | 충족 | E1 · E2 · E7 |

- 적대적 평가 2 라운드(Orca 가 라운드마다 새로 띄운 Codex) — [eval](eval.md):
  - R1 approve(지적 0, 범위 `5dc5b54..8c7a390`).
  - R2 needs-attention 1(범위 `8c7a390..4cff605` — 첫 종결 문서 고침 · 재개 · stage 6): R2-1 결정 8 밖 상태 쓰기 → 대조표 · A2b 부분 미충족으로 기록(미조치 지적 · 원장 `#1z-12 R2-1` 열림).
    stage 6 의 지운 둘은 리뷰어도 동작 차이를 찾지 못했고, 남긴 셋 · 문서 고침 · 재개 절차에는 지적 없음.
  - 두 라운드 모두 시작 지시가 Codex 입력창에 붙여 넣기로 남아(`outcome_unknown`) Enter 한 번으로 제출.
- 원장: 측정 커밋 `4cff605` — gate 8/8(첫 실행은 E7 이 되살린 옛 원장을 읽어 FAIL, 원장이 바뀐 뒤 다시 돌려 PASS), E8 = 새 빌드 57 단계 수집 오류 0.
- 미실행: macOS · 실제 두 PC 연동 · 화면 낭독기 · 릴리스(서명) 빌드.
- 편차:
  - `STATE.md` 를 stage 마다 갱신하지 않았다(intent ~ stage 5 커밋에 없음 — 첫 종결 커밋에서 한 번에). 재개 뒤 커밋은 매번 갱신.
  - plan 개정 기록 ① ~ ⑦(⑦ = 재개: 결정 8 · 9 · stage 6 — 평가 항목 EXPECT 그대로).
  - 종결 뒤 재개: 첫 종결의 report · eval 을 뺐다가(코드가 바뀌어 옛 측정이 무효) 평가 단계에서 R1 절을 그대로 되살리고 다시 썼다.
  - intent 범위 "같은 일을 하는 중복 · 늘 같은 쪽으로 가는 분기": 동작이 같아 보인 다섯 중 둘만 지웠다(셋은 사유). 서명 · 문구가 바뀌는 나머지(stage4-7)는 범위 밖 —
    사용자 목적 "불필요한 로직 제거" 는 그만큼 덜 됐다.
- 구현자 해석(사용자 미확인):
  - plan 승인 = 사용자 "권장 방안 승인." 으로 갈음.
  - `tools/check_code_tidy.py` 를 PROJECT.md 검증 명령에 등록(이후 Task 도 지키게), 표식 목록을 plan "뺌" 목록대로 넓힘(`#` 없는 Task 번호 · 항목 번호 · 날짜 · 시안 이름).
  - 남긴 D · S 번호: 보안 모델 · 프로토콜 근거 D8 · D9 · D12 · D21 · D22 · D24 · D30 · D31 · D39 ②([work/stage4](work/stage4.md)).
  - stage 6 판정 규칙 "합치려고 새 도우미나 더 어려운 조건식이 생기면 지우지 않는다"(plan 개정 ⑦) — `tray.rs` 를 남긴 근거.
  - A2 속성 소유 = `core/state.js` 의 속성 설명이 가리키는 화면(대조표 — "그 상태를 채우는 모듈" 기준이면 밖은 넷).

## 사용자 결정 필요

첫 종결의 report-1(A2 예외) · report-5(중복을 이 Task 에서)는 사용자 "권장 방안으로 진행." 으로 결정됐다(intent 결정 8 · 9). 남은 것:

| 인덱스 | 내용 | 곳 |
|---|---|---|
| report-6 | A2b 결정 8 밖 다른 도메인 상태 쓰기 일곱(`shell.js` `previewAll` · `focusMsg`, `rail.js` `unseen`, `sidebar.js` `waitingBy` · `unseenAll`, `conv.js` · `composer.js` `pickMsg`) — 같은 종류의 화면 흐름으로 예외에 넣을지, 도메인별로 쪼갤지(동작 불변 리팩터 — 다음 Task) | [work/stage6](work/stage6.md) "A2 예외 범위 대조" · [eval](eval.md) R2-1 |
| stage3-1 | `#p` 주고받은 메시지 화면(메뉴 없음 · 주소로만 열림) — 지울지 | [work/stage3](work/stage3.md) |
| stage3-2 | 시험만 쓰는 logic 넷(`roomPreview` · `unlinkedReplies` · `searchItems` · `ENGINES`) — 지울지 | [work/stage3](work/stage3.md) |
| stage4-1 ~ stage4-9 | 읽다가 찾은 이상한 로직(숨은 `peer_msg` 로 가는 문구 · 끄는 동안 다시 그리기 · 미리보기 글 / 상태가 다른 글 · `forget_peer` 남는 행 · 늦은 응답 · 문구 · 중복 · 덮인 CSS · 시험만 쓰는 것) — 다음 Task 후보. stage4-7 중 다섯은 stage 6 이 판정(둘 지움 · 셋 사유) | [work/stage4](work/stage4.md) · [work/stage6](work/stage6.md) |
| report-2 | an_app 매뉴얼의 없어진 함수 이름 5개(`keepPicked` · `groupSection` · `settingsGroupSections` · `USER_LIST` · `convCopy`) — 그 절 기능 설명부터 다시 | [work/stage5](work/stage5.md) |
| report-3 | an_agent 매뉴얼 본문에 Codex 실행(`run_agent` · `run_codex` · `codex exec --json` 단계 요약 · 최저 버전)이 없다(#1z-9 뒤 공백 — 이번엔 요약 줄만 고침) | [an_agent](../../manual/dev/an_agent.md) |
| report-4 | 다른 세션이 쓰는 미추적 문서(tech 폴더 `security-posture.md`)가 이 Task 전 이름을 근거로 든다(`app.js` `el` · 지운 `svgButton`) — 그 세션에 전달(이 Task 는 손대지 않음) | tech 폴더 `security-posture.md` 79줄 |

## 산출

- Rust: `crates/core/src/chat/` · `crates/agent/src/chat/`(stage 6 `delivery.rs` `kick`) · `crates/app/src/chat_api/`(stage 6 `conversation.rs`) · `server.rs`(`SCRIPTS` · `/js/{*file}`) · 주석.
- web: `crates/app/web/js/`(48) · `index.html` 진입 태그 · `app.css`(지운 규칙 · 토큰 · 주석).
- 시험: 새 `web_assets.rs`, `web_auth.rs`(정확 이름 · 404) · `chat_api.rs`(+ stage 6 특성 시험 `session_relay_cap_default_from_settings`) · `ux_literals.rs`(원본 = `SCRIPTS` · 7 행 코드 대조) · `view.test.mjs`(import · 원문 읽기).
- 도구: 새 `tools/check_code_tidy.py`(PROJECT.md 검증 명령 "코드 정리").
- 문서: [web-ui](../../manual/dev/web-ui.md) · [an_app](../../manual/dev/an_app.md) · [an_core](../../manual/dev/an_core.md) · [an_agent](../../manual/dev/an_agent.md) ·
  [developer-guide](../../manual/dev/developer-guide.md) · [manual 목차](../../manual/README.md) · [security-review](../../overview/security-review.md) · PRODUCT · PROJECT · masterPlan D10 보강 · 원장 `#1z-12 R2-1`.

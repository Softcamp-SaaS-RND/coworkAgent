# intent — 1z-12.code-tidy

승인: 2026-10-24(사용자 "권장 방안 승인." — 결정 5 ~ 7, "권장 방안으로 진행." — 결정 8 · 9)

## 왜

사용자 요청(대화, 2026-10-24) 원문:

> 다음 테스크 준비.
> 목적 1. stitch-design-taste 스킬 활용해서 ai designe 느낌 제거.
> 목적 2. 코드 정리. domain driven development 구조 적용. 코드에서 불필요한 로직이나 죽은 코드 제거. 장황한 주석보다는 필요한 부분에만 한정적인 주석 적용.

이 Task = **목적 2**(결정 1 — 목적 1 은 다음 Task #1z-13).
출처: 대화, PROJECT.md(크레이트 경계 — `tools/check_crates.py`), PRODUCT.md 모듈 표.

### 코드 사실 (2026-10-24 확인 — main `5dc5b54`)

| 사실 | 위치 |
| --- | --- |
| 큰 파일: 웹 `app.js` 5221줄 · `view.js` 1687, Rust `an-agent` `chat.rs` 4697 · `an-core` `chat.rs` 1928 · `app` `chat_api.rs` 1509 · `an-net` `node.rs` 960 · `runner.rs` 867 · `cli.rs` 867 · `notify.rs` 829 | `crates/*/src` · `crates/app/web` |
| `an-agent::chat` 한 파일에 실행 큐 · CLI 트랜잭션 · 질의 / 요청 보내기 · 받기 · 자동 왕복 · 컨텍스트(압축 · 비우기) · 공유 · 나가기 / 지우기 · 재시도 · 글 고치기 / 지우기가 함께 있다(`impl Chat` 하나) | `crates/agent/src/chat.rs` |
| 웹 원본은 `include_str!` 로 실행 파일에 들어가고 경로 4개만 준다(`/`, `/app.js`, `/view.js`, `/app.css` — `PUBLIC`) | `crates/app/src/server.rs` |
| 코드 주석 · 문서 주석에 Task 번호 · 리뷰 라운드 · "사용자 지시" 같은 결정 이력이 약 1,670곳 — 같은 이력은 masterPlan · `docs/archive/*` 에 이미 있다 | `crates/**` |
| `ux_literals.rs` 는 화면 원본 문자열(일부는 주석 문장)을 그대로 대조한다 | `crates/app/tests/ux_literals.rs` |

## 범위

- **도메인별 구조**(결정 2) — Rust 와 웹 둘 다. 크레이트 경계(an-core · an-net · an-agent · app)와 공개 API · HTTP API · 통신 형식 · DB 스키마는 그대로, 크레이트 **안** 모듈을 도메인으로 나눈다.
  - Rust 후보(plan 에서 확정): `an-agent::chat` → 실행 큐 · CLI 트랜잭션 · 보내기 · 받기 · 자동 왕복 · 컨텍스트 · 공유 · 대화 수명(나가기 · 지우기) 하위 모듈, `an-core::chat` → 대화 · 글 · 요약 질의 하위 모듈,
    `app::chat_api` → 대화 · 글 · 요청 · 공유 라우트 묶음.
  - 웹 후보: `app.js` 를 도메인 파일(개요 · 대화방 · 사이드바 / 레일 · 설정 · 요청 · 알림 · 공통 DOM)로, `view.js`(DOM 없는 계산)는 도메인별로 나눔 — 제공 방식은 U1.
- **죽은 코드 · 불필요 로직 제거**(결정 4 — 동작 불변): 호출되지 않는 함수 · 상수 · CSS 규칙, 같은 일을 하는 중복, 늘 같은 쪽으로 가는 분기. 기능 자체를 뺄 후보는 지우지 않고 목록으로 보고.
- **주석 정리**(결정 3): 주석 = 코드만 봐서는 모를 "왜" · 보안 · 함정만 짧게. Task 번호 · 리뷰 라운드 · 사용자 지시 인용 · 바뀐 이력은 주석에서 뺀다(masterPlan · archive 에 있음).
  주석 문장을 대조하던 `ux_literals` 행은 코드 자체를 대조하도록 바꾼다(약화 아님 — 같은 동작을 잰다).
- 매뉴얼(an_core · an_agent · an_app · an_net · web-ui · developer-guide)의 파일 · 모듈 경로를 새 구조로.

## 제외

- 목적 1(디자인 — "AI 디자인 느낌" 제거) — 다음 Task #1z-13. `stitch-design-taste` 스킬은 사용자만 부를 수 있다(`/stitch-design-taste`).
- 동작 · 화면 · HTTP API · 통신 형식 · DB 스키마 · CLI 출력 바꾸기(결정 4).
- 크레이트 추가 · 합치기 · 크레이트 경계 바꾸기.
- 기능 빼기(후보는 목록 보고 — 별도 결정).

## 성공 기준

- A1: 동작 불변 — 기존 시험 전부 그대로 통과(바뀌는 시험은 셋만 — 주석 문장을 대조하던 `ux_literals` 행 → 코드 대조, 정적 경로 `/app.js` · `/view.js` → `/js/…`(같은 성질), `view.test.mjs` import 경로 — 행마다 근거), 실제 화면 주요 흐름(개요 · 대화 · 설정 · 요청) 그대로.
- A2: 도메인 구조 — plan 의 도메인 표대로 모듈 · 파일이 나뉘고, 한 파일이 둘 이상 도메인의 상태 변경을 갖지 않는다(공통 · 저장 계층 제외, 도메인을 가로지르는 화면 흐름은 결정 8 의 예외). 파일 크기 상한은 plan 에서.
- A3: 죽은 코드 0 — `cargo clippy -D warnings`(dead_code 포함) 깨끗, 웹은 정의되고 안 불리는 함수 · CSS 규칙 0(도구 또는 grep 근거), 지운 것 목록.
- A4: 주석 — 결정 이력 표식(Task 번호 · `R1`/`R2` · "사용자 지시") 0, 남은 주석은 왜 · 보안 · 함정.
- A5: 회귀 없음 — PROJECT.md 검증 명령 PASS, 매뉴얼 경로 반영.

## 결정

1. 두 Task 로 나누고 코드 정리 먼저 — 사용자 질문 답 "둘로 나눔 · 코드 정리 먼저".
2. 범위 = Rust + 웹 — 사용자 질문 답 "Rust + 웹 둘 다".
3. 결정 이력은 문서로, 주석은 왜만 — 사용자 질문 답 "이력은 문서로, 주석은 왜만".
4. 동작 불변만 — 사용자 질문 답 "동작 불변만"(기능 제거 후보는 목록 보고).

5. 웹 파일 = 브라우저 ES 모듈 여러 파일, 서버가 내장 표로 준다(빌드 도구 없음) — 사용자 "권장 방안 승인."(U1 (a)).
6. 버전 1.3.6 그대로 — 같은 답(U2).
7. 리뷰 = 이동 감지 + 동작 불변 대조 초점 · 2 라운드 상한 그대로 — 같은 답(U3).

8. A2 예외 — 도메인을 가로지르는 화면 흐름(설정값 적용 `settings/prefs.js` · 변경 신호 `events.js` · 순서 저장 `drag.js` · 타일 · 대화 고르기 `current` · 다음 화면 초점 `focusAfter`)이
   다른 도메인 상태를 쓰는 것은 받는다 — 함수를 도메인별로 쪼개지 않는다. 사용자 "권장 방안으로 진행."(종결 report 의 report-1 — 2026-10-24).
9. intent 범위 "같은 일을 하는 중복 · 늘 같은 쪽으로 가는 분기" 를 이 Task 에서 — stage4-7 중 동작이 같아 보인 다섯(`tray.rs` 종료 두 갈래 · `chat_api/conversation.rs` 상한 기본값 계산 두 번 ·
   `delivery.rs` `kick` 의 늘 참인 `if let` · `set_blocked` 두 번 · `logic/conv.js` `convListParts` 다시 거르기)을 하나씩 확인해 동작 불변인 것만 지운다(코드가 바뀌므로 디버그 빌드 ·
   실제 화면 · gate 다시 · 리뷰 라운드 2/2) — 같은 답(report-5 (a)).

## 미해소

- 없음 — 결정 1 ~ 9.

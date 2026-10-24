# plan — 1z-12.code-tidy

> 근거: [intent](intent.md)(승인 2026-10-24 — 결정 1 ~ 7). **구현자는 이 문서만 보고 일한다.**
> 브랜치 `task/1z-12-code-tidy`(원 작업 폴더 — worktree 없음). 기준 커밋: `5dc5b54`(main). plan 승인: 2026-10-24(사용자 "권장 방안 승인." 으로 갈음, 구현자 해석). stage 6 · A2 예외 = 사용자 "권장 방안으로 진행."(개정 ⑦).

## 목표

동작 · 화면 · HTTP API · 통신 형식 · DB 스키마 · CLI 출력을 그대로 두고, 크레이트 안 모듈과 웹 원본을 도메인별로 나누고, 죽은 코드를 지우고, 주석을 "왜 · 보안 · 함정" 만 남긴다. 버전 1.3.6 그대로.

### 성공 기준(intent 사본)

- A1: 동작 불변 — 기존 시험 전부 그대로 통과(바뀌는 시험은 셋만 — 주석 문장을 대조하던 `ux_literals` 행 → 코드 대조, 정적 경로 `/app.js` · `/view.js` → `/js/…`(같은 성질), `view.test.mjs` import 경로 — 행마다 근거), 실제 화면 주요 흐름(개요 · 대화 · 설정 · 요청) 그대로.
- A2: 도메인 구조 — plan 의 도메인 표대로 모듈 · 파일이 나뉘고, 한 파일이 둘 이상 도메인의 상태 변경을 갖지 않는다(공통 · 저장 계층 제외, 도메인을 가로지르는 화면 흐름은 intent 결정 8 의 예외 — 개정 ⑦). 파일 크기 상한은 plan 에서.
- A3: 죽은 코드 0 — `cargo clippy -D warnings`(dead_code 포함) 깨끗, 웹은 정의되고 안 불리는 함수 · CSS 규칙 0(도구 또는 grep 근거), 지운 것 목록.
- A4: 주석 — 결정 이력 표식(Task 번호 · `R1`/`R2` · "사용자 지시") 0, 남은 주석은 왜 · 보안 · 함정.
- A5: 회귀 없음 — PROJECT.md 검증 명령 PASS, 매뉴얼 경로 반영.

## 설계

### 공통 규칙

- **옮기기만**: 함수 · 타입 본문은 글자 그대로 옮긴다(주석 정리는 stage 4 에서 따로). 가시성은 컴파일러가 요구하는 만큼만 넓힌다(`pub(super)` · `export`).
- **공개 경로 유지**: 다른 크레이트 · 시험이 쓰는 경로(`an_core::chat::X`, `an_agent::chat::Chat` …)는 `mod.rs` 의 `pub use` 로 그대로.
- **파일 크기 상한**: 제품 원본(`crates/*/src/**/*.rs`, `crates/app/web/js/**/*.js`) 파일마다 **1,000줄 이하**. `app.css` · `index.html` 은 한 파일로 둔다(제공 경로 하나 — 절 순서만 정리).
- **동작이 바뀌는 지움 금지**(결정 4): 기능을 뺄 후보는 지우지 않고 work 문서 표로 보고.

### stage 1 — Rust 도메인 모듈

| 지금 | 나눈 뒤(모듈 = 도메인) |
| --- | --- |
| `an-core/src/chat.rs`(1,928) | `chat/mod.rs`(대화 · 글 타입 · 행 읽기 · 공통 SQL) · `conversation.rs`(대화 열기 · 찾기 · 목록 · 제목 · 별 · 읽음 · 나가기 · 지우기 · 비우기 · 요약 · 정보) · `message.rs`(글 쓰기 · 읽기 · 상태 · 고치기 · 북마크 · 페이지) · `request.rs`(요청 상태 · 메모 · 보관 · 개수) · `activity.rs`(내 활동) · `search.rs`(검색) · `relay.rs`(자동 왕복 표시) · `session.rs`(에이전트 세션 · 컨텍스트 · 실행 명령 · 실행 표시) |
| `an-agent/src/chat.rs`(4,697) | `chat/mod.rs`(`Chat` · 생성 · 공통 상수 · 도우미) · `queue.rs`(실행 큐 · 동시 실행 상한 · 진행) · `tx.rs`(CLI 트랜잭션 · 감시) · `outbox.rs`(보낼 함 · 전달 · 재시도 · 상태 묻기) · `send.rs`(질의 · 요청 보내기 · 세션 열기) · `inbound.rs`(받은 프레임 분기 · 받은 글 · 알림) · `request.rs`(요청 상태) · `tidy.rs`(글 고치기 · 지우기 동기화) · `relay.rs`(자동 왕복) · `execute.rs`(에이전트 실행 · 실패 기록) · `context.rs`(압축 · 비우기 요청 · 요약) · `share.rs`(공유 사본 · 관람자) · `lifecycle.rs`(나가기 · 지우기 · 비우기) |
| `app/src/chat_api.rs`(1,509) | `chat_api/mod.rs`(`routes` · 응답 도우미 · JSON 변환) · `agents.rs`(내 에이전트 API) · `conversations.rs`(대화 API · 정보 · 관람자 · 나가기 · 지우기 · 비우기) · `messages.rs`(글 · 목록 · 페이지 · 북마크 · 고치기 · 지우기) · `runs.rs`(실행 큐 API) · `activity.rs`(내 활동 · 검색 · 북마크 목록 · 요청 개수) · `tx.rs`(CLI 이어 묻기 · 트랜잭션) |

- 나누는 방법: 항목(함수 · 메서드 · 타입 · 상수 + 바로 위 문서 주석 · 속성)을 이름 → 모듈 표로 잘라 붙이는 스크립트(scratchpad) — 본문은 바이트 그대로. `impl Chat` 메서드는 모듈마다 `impl Chat { … }`.
  여러 모듈이 쓰는 비공개 상수 · 도우미는 `mod.rs` 에 두고 하위 모듈은 `use super::*;`. 한 모듈만 쓰는 비공개 항목은 그 모듈로.
- 확인: `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings`.

### stage 2 — 웹 ES 모듈(결정 5)

- 배치: `crates/app/web/js/app.js`(진입 — 첫 화면 · 이벤트 연결 순서) · `js/core/`(`dom.js` 요소 · 아이콘 · 단추, `api.js` 토큰 · fetch · 연결 끊김, `state.js` 공유 상태) ·
  UI 도메인 `js/<도메인>.js`(셸 · 레일 · 사이드바 · 개요 · 대화 · 입력창 · 세부정보 · 요청 · 내 활동 · 북마크 · 에이전트 · 새 세션 · 검색 · 끌기 · 알림 스트림, 설정은 `js/settings/` 아래 연결 · 그룹 · 에이전트 · 설정값 · 디버깅) ·
  DOM 없는 계산 `js/logic/<도메인>.js`(형식 · 대화 · 마크다운 · 상대 · 레일 · 개요 · 요청 · 내 활동 · 설정 · 에이전트 · 셸 · 동기화 · 검색 · 컨텍스트).
- 모듈 사이는 상대 경로 `import`(브라우저 · node 둘 다). 공유 상태는 `core/state.js` 하나(다른 모듈은 그 객체의 속성만 바꾼다 — `let` 을 다른 모듈에서 대입하지 않는다).
  진입 `app.js` 의 **첫 import = `core/token.js`**(토큰 읽기 · 없으면 안내 후 멈춤 — `core/dom.js` · logic 만 import) — 토큰이 없으면 다른 모듈이 평가되기 전에 멈춘다(지금과 같음).
  `document` · `window` 이벤트 등록은 진입 모듈에 원래 순서대로, 같은 요소 · 같은 이벤트를 두 모듈이 등록하지 않는다. 진입이 모든 도메인 모듈을 import 한 뒤 첫 그리기(시작 순서 그대로).
- 층: `core/dom.js` · `core/state.js` · `core/token.js` 는 도메인 모듈을 import 하지 않는다, `logic/*` 는 UI 모듈을 import 하지 않고 서로 순환하지 않는다(스크립트 확인).
- 나누는 방법: 최상위 항목(함수 · `const` · `let` · `class` · 문장)을 이름 → 모듈 표로 잘라 붙이는 스크립트(scratchpad, 표와 결과는 work 문서). 최상위 선언은 전부 `export`, 다른 모듈이
  내보낸 이름이 원문에 낱말로 나오면 import(주석 · 문자열 안 포함 — 빠진 import 가 없게 넉넉히, logic 끼리 순환이 생기면 코드 낱말만으로 줄임). 다른 모듈의 `let` 에 대입하는 곳 · 두 모듈에 나뉜 같은 이벤트 등록은 스크립트가 멈추고 보고.
- 옮기기만 확인: 나누기 전 · 뒤 항목 본문 집합이 바이트로 같다(다른 곳은 `export` · `import` 줄 · Rust 가시성 낱말뿐) — 스크립트 결과를 work 문서에.
- 서버(`server.rs`): `pub const SCRIPTS: [(&str, &str)]`(경로 → 내장 원본) · `GET /js/{*file}` · 토큰 없이 주는 경로 = `/` · `/app.css` · `/favicon.ico` · `/fonts/…` · `/js/<SCRIPTS 이름>`.
  `/app.js` · `/view.js` 경로는 없앤다(`index.html` = `<script src="/js/app.js" type="module">`). 보안 헤더 · CSP(`script-src 'self'`) 그대로.
  토큰 없이 주는 판정은 `SCRIPTS` 이름과 **정확히 같을 때만**(접두 일치 없음), 표에 없는 `/js/…` = 404. `web_auth.rs` 에 `/js/<없는 이름>.js` = 404 · `/app.js` · `/view.js` = 404 를 더함.
- 시험: `view.test.mjs` 는 자리 그대로(`crates/app/web/` — PROJECT.md 명령 그대로) · import 만 `./js/logic/…` 로. 새 `crates/app/tests/web_assets.rs` =
  ① `web/js` 아래 파일 = `SCRIPTS` 이름(빠짐 · 남음 없음) ② 모듈마다 상대 `import` 대상이 `SCRIPTS` 에 있음 ③ import 한 이름을 대상 모듈이 `export` 함.
  `ux_literals.rs` 의 원본 키 `app.js` · `view.js` → `app`(UI 모듈 전부) · `view`(logic 모듈 전부), 원본은 `an_app::server::SCRIPTS` 에서.
- 확인: `node --test` · `cargo test` · 실제 화면(모든 화면 열기 · 콘솔 오류 0).

### stage 3 — 죽은 코드 · 불필요 로직

- Rust: `pub` 항목 중 워크스페이스(원본 + 시험) 어디서도 안 쓰는 것(스크립트 — 이름 출현 수), 비공개는 컴파일러(`dead_code`).
- 웹: 어느 모듈 · `index.html` 에서도 안 쓰는 최상위 함수 · 상수, 어느 JS · HTML 에도 안 나오는 CSS 클래스 · id 규칙(동적으로 만드는 이름 접두 `st-` · `k-` 등은 남김).
  시험만 쓰는 logic 함수는 지우지 않고 후보 표로 보고(시험 약화로 읽히지 않게).
- 지우지 않는 것: `ux_literals` PRESENT 행이 대조하는 것(검사가 죽었다고 해도 보고만), Rust 시험 이음새(`set_watch` · `set_retry` · `set_step_every` …).
- 불필요 로직(늘 같은 쪽 분기 · 중복)은 읽다가 찾은 것만 — 동작이 같음을 시험으로 보이고 고친다. 기능 제거 후보는 표로 보고만.

### stage 4 — 주석(결정 3)

- 남김: 코드만 봐서는 모를 이유 · 보안 · 함정 · 프로토콜 제약, 이름만으로 뜻이 안 보이는 공개 항목의 한 줄 설명. D · S 번호는 그 제약을 찾아갈 곳이 꼭 필요할 때만.
- 뺌: Task 번호(`#NN` · `#1x`) · 리뷰 라운드(`R1` · `R2`) · "사용자 지시 / 결정 / 원문" · 시안 이름 · plan · intent 항목 번호 · 개정 이력 · 코드를 그대로 다시 말하는 문장.
- 방법: 겹치지 않는 파일 묶음마다 병렬 작업(서브 에이전트) — 주석만 고치고(한국어 그대로), 주석을 뺀 코드 토큰이 전후 같음을 스크립트로 확인(주석 전용 변경 보증 — 손대지 않은 파일은 통과,
  문자열 · 정규식 하나를 바꾸면 실패함을 먼저 확인). 서브 에이전트는 코드 · 시험 · `ux_literals` · 다른 세션 파일을 고치지 않고 cargo · node · git 을 돌리지 않으며, 이상한 로직은 목록으로만 돌려준다.
- 새 `tools/check_code_tidy.py` — 제품 원본의 주석에서 결정 이력 표식 수 · 파일 줄 수 상한(1,000)을 잰다. PROJECT.md 검증 명령에 등록(이후 Task 도 지키게 — 구현자 해석).
- 주석 문장을 대조하던 `ux_literals` 행 → 같은 동작을 드러내는 코드 문자열로(행마다 work 문서에 전후).

### stage 5 — 문서

- 매뉴얼 경로 · 모듈 이름: [web-ui](../../manual/dev/web-ui.md) · [an_core](../../manual/dev/an_core.md) · [an_agent](../../manual/dev/an_agent.md) · [an_app](../../manual/dev/an_app.md) · [developer-guide](../../manual/dev/developer-guide.md) · PRODUCT 모듈 표 · PROJECT.md(검증 명령).

### stage 6 — 중복 · 늘 같은 쪽 분기(intent 결정 9)

- 대상: [work/stage4](work/stage4.md) stage4-7 중 동작이 같아 보인 다섯 — `tray.rs` 종료 두 갈래 같은 몸 · `chat_api/conversation.rs` 자동 왕복 상한 기본값 계산 두 번 ·
  `delivery.rs` `kick` 의 늘 참인 `if let` · `set_blocked` 두 번 · `logic/conv.js` `convListParts` 다시 거르기.
- 하나씩 판정: 순차 실행 · 동시 실행(다른 작업이 끼는 자리) · 오류 응답 순서(검증 400 · DB 오류) · 시험 계약(A1 — 바뀌는 시험 셋 밖은 고치지 않는다)이 모두 같으면 지운다.
  하나라도 다르면 그대로 두고 사유를 work 문서에. 합치려고 새 도우미나 더 어려운 조건식이 생기면 지우지 않는다(읽기가 나빠지는 정리는 하지 않는다).
- 서명 · 문구가 바뀌는 것(안 쓰는 `peer` · `acked` 반환값 · `sharing.rs` 같은 검사 두 번 · 종류 목록 네 곳)은 범위 밖 — 그대로.
- 고칠 갈래에 시험이 없으면 먼저 특성 시험(지금 코드에서 green) → 고친 뒤 green.
- 확인: `cargo test --workspace` · clippy · `node --test` · `check_code_tidy` → 리뷰 라운드 2/2 → 디버그 빌드 · E8 다시 · gate 다시.

### 바꾸는 시험(의도된 변경 — 약화 아님)

- `web_auth.rs` · `chat_api.rs` 의 정적 경로 `/app.js` · `/view.js` → `/js/app.js` · `/js/logic/…`(토큰 없이 받음 · 그 밖은 401 — 같은 성질을 새 경로로).
- `view.test.mjs` import 경로 · 원문 읽기 경로(`./app.js` · `./view.js` → `js/` 모듈을 이어 붙인 글). `ux_literals.rs` 원본 키 · 주석 문장 행. `web_auth.rs` 에 새 경로 성질(없는 이름 404 · 옛 경로 404) 더함.

### 리뷰 초점(결정 7)

- 이동 감지(`git diff --color-moved=dimmed-zebra`) + work 문서의 "옮기기만" 대조 결과로 동작 불변 확인, 한 파일이 둘 이상 도메인의 상태 변경을 갖는지(A2 — 기계 gate 없음), 새 `/js/` 경로의 토큰 판정.
- 라운드 2/2(stage 6 뒤): stage 6 의 지운 것 · 남긴 것 판정(순차 · 동시 실행 · 오류 순서 · 시험 계약), R1 뒤 문서 고침(security-review · 실행기 요약 · 재개 문서).

## 평가 항목

- [ ] E1: A1 · A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A3 — clippy 경고 0(dead_code 포함)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1 — 웹 logic 시험
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A1 · A4 — 화면 원본 literal · CSS 토큰
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A2 — 웹 모듈 표 · import 정합(파일 = SCRIPTS · 대상 있음 · 이름 export)
  CHECK: cargo test -p coworkagent-app --test web_assets --test web_auth
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E6: A2 · A4 — 파일 크기 상한 · 주석 결정 이력 표식 0
  CHECK: python tools/check_code_tidy.py
  EXPECT: /PASS — 제품 원본 [0-9]+개 · 최대 [0-9]+줄 · 이력 표식 0/
  EVIDENCE: pending
- [ ] E7: A5 — 문서 규약 · 크레이트 경계 · README · 매뉴얼 경로
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_readme.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('web=%d agent=%d core=%d app=%d'%(r('docs/manual/dev/web-ui.md').count('js/core/dom.js'),r('docs/manual/dev/an_agent.md').count('chat/execute.rs'),r('docs/manual/dev/an_core.md').count('chat/conversation.rs'),r('docs/manual/dev/an_app.md').count('chat_api/message.rs')))"
  EXPECT: /web=[1-9][0-9]* agent=[1-9][0-9]* core=[1-9][0-9]* app=[1-9][0-9]*/
  EVIDENCE: pending
- [ ] E8: A1 — 실제 화면: 디버그 빌드 + 시험 데이터 · ego-browser 1440 × 900 — 개요(목록 · 미리보기) · 대화 · 세부정보 · 내 활동 · 요청(목록 · 보드) · 북마크 · 에이전트 응답 · 등록된 에이전트 ·
  설정 메뉴 8 · 검색 · 새로 만들기 · 프로필 메뉴 · 레일 타일 메뉴가 열리고 콘솔 오류 0(라이트 · 다크), 토큰 없이 `/` = 다시 열기 안내 · `/api/*` 요청 없음
  EVIDENCE: pending

## 개정 기록

- 2026-10-24(구현 전 — advisor 검토): A1 의 바뀌는 시험을 셋으로 명시(intent 같이), 시험만 쓰는 logic 함수는 지우지 않고 후보 보고, `core/token.js` 첫 평가 · 층 규칙 · 넉넉한 import ·
  옮기기만 대조, `/js/` 정확 일치 · 404 시험, E7 을 새 경로 이름으로 좁힘(지금 문서에서 0), E8 에 토큰 없는 경로.
- 2026-10-24(stage 2 구현 중 — [work/stage2](work/stage2.md)): ① window 오디오 깨우기 등록(`pointerdown` · `keydown` → `audio ??=`)은 진입이 아니라 `events.js`(`let audio` 옆) —
  진입에서 대입하면 import 바인딩 대입(TypeError). window · document 최상위 등록 차례는 그대로(원본에서 맨 앞, 진입 본문 등록은 모든 import 평가 뒤).
  ② `core/*` 도 코드 낱말로 import — 주석 낱말(`/api/…`)이 `core/state.js` ↔ `core/api.js` 순환을 만들어 `state.js` 가 늦게 평가됐다(logic 과 같은 이유).
  ③ `view.test.mjs` 의 원문 읽기 3곳도 새 경로로(같은 성질 — 바꾸는 시험 절에 더함). ④ E7 의 `chat_api/messages.rs` → `chat_api/message.rs`(stage 1 이름은 단수형 — [work/stage1](work/stage1.md)).
- 2026-10-24(stage 3 · 4 구현 중): ⑤ stage 2 의 넉넉한 import(주석 · 문자열 낱말) 중 코드에서 안 쓰는 23개를 stage 3 에서 지웠다 — 나누기 안전장치였고, 옮기기만 확인 · 실제 화면 확인 뒤라
  남길 이유가 없다([work/stage3](work/stage3.md)). ⑥ 이력 표식은 `app.css` 주석도 잰다(결정 2 — 웹 원본, 표식 100곳), 줄 수 상한은 그대로 rs · js 만. 표식 목록은 plan "뺌" 목록대로
  `#` 없는 Task 번호(`1a`) · 항목 번호(`A3` · `T2` · `Q3`) · 날짜 · 시안 이름까지(구현자 해석 — `tools/check_code_tidy.py` 머리 주석).
- 2026-10-24(종결 뒤 재개 — 사용자 "권장 방안으로 진행."): ⑦ 종결 report 의 report-1 → A2 예외(intent 결정 8 — 도메인을 가로지르는 화면 흐름: 설정값 적용 `settings/prefs.js` ·
  변경 신호 `events.js` · 순서 저장 `drag.js` · 타일 · 대화 고르기 `current` · 다음 화면 초점 `focusAfter` 가 다른 도메인 상태를 쓰는 것은 받는다), report-5 (a) → stage 6(intent 결정 9).
  평가 항목 E1 ~ E8 그대로(EXPECT 그대로) — 코드가 바뀌므로 gate 를 다시 재고 E8 을 새 빌드로 다시, 리뷰 라운드 2/2. stage 6 의 "새 도우미나 더 어려운 조건식이면 지우지 않는다" 는 구현자 해석.

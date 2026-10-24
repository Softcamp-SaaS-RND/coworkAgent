# eval.md — 1z-12.code-tidy 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 4cff60597ecbfe09683571a3a111bf4c4eb66dd3

- [x] E1: A1 · A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

- [x] E2: A3 — clippy 경고 0(dead_code 포함)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.34s

- [x] E3: A1 — 웹 logic 시험
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A1 · A4 — 화면 원본 literal · CSS 토큰
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

- [x] E5: A2 — 웹 모듈 표 · import 정합(파일 = SCRIPTS · 대상 있음 · 이름 export)
  CHECK: cargo test -p coworkagent-app --test web_assets --test web_auth
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

- [x] E6: A2 · A4 — 파일 크기 상한 · 주석 결정 이력 표식 0
  CHECK: python tools/check_code_tidy.py
  EXPECT: /PASS — 제품 원본 [0-9]+개 · 최대 [0-9]+줄 · 이력 표식 0/
  EVIDENCE: PASS — 제품 원본 116개 · 최대 960줄 · 이력 표식 0

- [x] E7: A5 — 문서 규약 · 크레이트 경계 · README · 매뉴얼 경로
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_readme.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('web=%d agent=%d core=%d app=%d'%(r('docs/manual/dev/web-ui.md').count('js/core/dom.js'),r('docs/manual/dev/an_agent.md').count('chat/execute.rs'),r('docs/manual/dev/an_core.md').count('chat/conversation.rs'),r('docs/manual/dev/an_app.md').count('chat_api/message.rs')))"
  EXPECT: /web=[1-9][0-9]* agent=[1-9][0-9]* core=[1-9][0-9]* app=[1-9][0-9]*/
  EVIDENCE: web=2 agent=1 core=1 app=1

- [x] E8: A1 — 실제 화면: 디버그 빌드 + 시험 데이터 · ego-browser 1440 × 900 — 개요(목록 · 미리보기) · 대화 · 세부정보 · 내 활동 · 요청(목록 · 보드) · 북마크 · 에이전트 응답 · 등록된 에이전트 ·
  EVIDENCE: Claude(구현자) 2026-10-24 18:24 — 디버그 빌드 `target/debug/coworkAgent.exe`(17:56:06 — 제품 원본 마지막 수정 17:55:19 `chat_api/conversation.rs` 뒤, 그 뒤 코드 변경 없음 = 측정 커밋 `4cff605` 코드 · `cargo build -p coworkagent-app` 다시 불러 새로 지을 것 없음) + scratchpad `ui1z12\data`, ego-browser 1440 × 900, CDP 로 문서마다 오류 수집(error · unhandledrejection · console.error · console.warn). 앞 측정과 같은 57 단계 스크립트(포트 · 토큰만 바꿈): 첫 화면 · 라이트 26 · 다크 전환 2 · 다크 26 · 사진 · 되돌림 — 개요 미리보기 카드 6 · 목록 행 16 · 대화 `#c/2` 메시지 4 · 세부정보 열림(행 4) · 내 활동 · 요청 목록 · 보드 · 북마크 · 즐겨찾기 · 에이전트 응답 · 등록된 에이전트 · 세션 생성 · 설정 8 메뉴(누를 때마다 그 쪽 하나만 보임) · 검색 열림(항목 4) · 새로 만들기(4) · 프로필 메뉴(6) · 레일 타일 메뉴(타일 편집 · 위로 · 아래로 옮기기), 라이트 · 다크 수집 오류 0, `/js/` 48 모듈 모두 200(자원 항목 49 = 48 + 스크립트가 따로 받은 `/js/tiles.js` fetch 하나). 실패 응답은 시험 데이터의 원격 없는 상대 `/api/peers/{1,2,3}/ping` · `/agents` 502 뿐. 토큰 없이 새 탭 `/`: 안내 "연결이 끊겼습니다. 트레이 아이콘 메뉴 '열기' 로 다시 여세요." · `/api/*` 요청 0 · `/js/` 48 · 수집 1건 = `core/token.js:19` 의 `throw new Error("토큰 없음")`(의도된 멈춤). 사진 scratchpad `ego/1z12-smoke-r2/`(light_home · dark_conv · notoken — 앞 측정 사진과 같은 화면)


WEAK: E8 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills` — 대상 저장소 밖, `git -C` 로 읽기만), handle `term_6bb202ef-24af-48d6-a221-01790e7be97d`.
Run `run_416c86c48a61` · Task `task_f20cae93f054` · Dispatch `ctx_2e28d65c4e62`. 시작 receipt 는 `outcome_unknown`(`turn_start_unobserved`) — 지시가 Codex 입력창에 붙여 넣기 상태로 남아
있었다(화면 `[Pasted Content 24876 chars]` · Ready). 다시 띄우지 않고 `terminal send --enter` 한 번으로 제출, 작업 시작을 화면으로 확인. worker_done(`msg_884809a8cef8`) 발신 handle 일치 · outcome succeeded.
범위 `5dc5b54..8c7a390`(self-collect), 메타 `요청 1z-12-code-tidy-r1 · revision 8c7a390 · 라운드 1/2`. 리뷰 전후 대상 작업 폴더 같음 — HEAD `8c7a390` · `git status` · 다른 세션 파일 해시
(`PRODUCT.md` 미커밋 2줄 · 미추적 tech 폴더 `security-posture.md`). 리뷰어 Dispatch `worker-release` → `released`(`closed_agent_terminal`, 원문 transcript `captured`). 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against main
Verdict: approve

고정 범위 5dc5b54..8c7a390의 이동·삭제·주석·인증 경로와 명세를 읽기 전용으로 대조했다. Rust 원본 줄 누락 0, JS 568항목 보존, 주석 밖 토큰 차이 0, 모의 DOM 리스너 104개 일치와 logic 시험 172개 통과를 확인했고 차단할 실질 결함은 찾지 못했다. Cargo·실제 브라우저는 재실행하지 않았고 macOS·두 PC 연동·화면 낭독기·서명 빌드는 기존 미검증 범위로 남는다.

No material findings.
```

raw JSON:

```json
{"verdict":"approve","summary":"고정 범위 5dc5b54..8c7a390의 이동·삭제·주석·인증 경로와 명세를 읽기 전용으로 대조했다. Rust 원본 줄 누락 0, JS 568항목 보존, 주석 밖 토큰 차이 0, 모의 DOM 리스너 104개 일치와 logic 시험 172개 통과를 확인했고 차단할 실질 결함은 찾지 못했다. Cargo·실제 브라우저는 재실행하지 않았고 macOS·두 PC 연동·화면 낭독기·서명 빌드는 기존 미검증 범위로 남는다.","findings":[],"next_steps":[]}
```

판정: 지적 0 — 고칠 것 없음. 코드가 바뀌지 않아 라운드 2/2 는 돌리지 않았다(같은 revision 재리뷰는 새 정보가 없음). 리뷰어가 다시 돌리지 않은 cargo · 실제 브라우저는 E1 ~ E8 원장이 갈음,
macOS · 두 PC 연동 · 화면 낭독기 · 서명 빌드는 미실행(report). 리뷰 뒤 구현자가 찾은 문서 낡음만 고쳤다(코드 변경 없음 — 측정 유효): `overview/security-review.md` 의 토큰 없이 주는 화면 파일 ·
`innerHTML` 문장, 크레이트 요약의 "`claude -p` 실행기" 4곳(PRODUCT · 매뉴얼 목차 · developer-guide · an_agent).

## 적대적 평가 — 라운드 2/2

라운드 1/2 뒤 사용자 "권장 방안으로 진행."(2026-10-24)으로 Task 를 다시 열어 stage 6(코드 변경)을 더했다 — 그래서 라운드 2/2 를 돌렸다(위 라운드 1 의 "라운드 2/2 는 돌리지 않았다" 는 그때 판정 그대로 둔다).

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills` — 대상 저장소 밖, `git -C` 로 읽기만), handle `term_3ba2294f-b122-4123-8632-eae0004a58ed`.
Run `run_416c86c48a61`(라운드 1 과 같은 Run) · Task `task_868fbdccd408` · Dispatch `ctx_3a4c8bdbe28f`. 시작 receipt 는 라운드 1 처럼 `outcome_unknown`(`turn_start_unobserved`) — 지시가 Codex 입력창에
붙여 넣기 상태로 남아 있었다(draft `Y[Pasted Content 1174 chars]# [Pasted Content 13805 chars]` · Ready). 다시 띄우지 않고 `terminal send --enter` 한 번으로 제출, 작업 시작(Working)을 화면으로 확인 —
Codex 쪽에 "Hook failed — hook exited with code 1" 한 줄이 떴으나 리뷰는 이어졌다. worker_done(`msg_b105f66ddb41`) 발신 handle 일치 · payload Task · Dispatch 일치 · outcome succeeded.
범위 `8c7a390..4cff605`(self-collect — R1 뒤 문서 고침 · 재개 · stage 6), 메타 `요청 1z-12-code-tidy-r2 · revision 4cff605 · 라운드 2/2`. 리뷰 전후 대상 작업 폴더 같음 — HEAD `4cff605` ·
`git status` · 다른 세션 파일 해시 · `git diff HEAD` 해시. 리뷰어 Dispatch `worker-release` → `released`(`closed_agent_terminal`, 원문 transcript `captured`), Delivery `delivery_c8c108d6ed72` 확인. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against main
Verdict: needs-attention

고정 범위 8c7a390..4cff605를 읽기 전용으로 대조했다. Stage 6 두 변경의 동작 차이는 찾지 못했지만 A2 예외 밖 상태 쓰기가 남아 명세 충족 판정은 보류다. 시험은 재실행하지 않았으며 R1 원문 복원·최종 gate 재측정·새 빌드 E8은 평가 단계에 남는다.

Findings:
- [medium] 결정 8의 A2 예외가 입력창의 실행 상세 상태 변경을 빠뜨린다 (docs/tasks/1z-12.code-tidy/intent.md:63-64)
  결정 8은 설정 적용·변경 신호·순서 저장·current·focusAfter를 예외로 열거하지만 composer.js:29-35는 sending과 함께 pickMsg를 변경한다. core/state.js:36은 pickMsg를 실행 상세 선택값으로 정의하고 detail.js:81-85가 이를 소비하며, work/stage2.md:61에도 이 쓰기가 기록돼 있다. 입력창과 세부정보를 별도 도메인으로 보는 것은 plan의 도메인 분리에 따른 추론이지만, 이 경우 메시지 전송 뒤 실행 상세 선택을 초기화하는 흐름은 열거한 예외에 없어 A2 미충족이 남는다. 이를 모든 화면 흐름에 대한 포괄 승인으로 간주하면 사용자가 승인한 예외 범위가 조용히 넓어진다.
  Recommendation: 상태 변경 표를 속성별 도메인 소유권과 대조하고 pickMsg 초기화 등 누락 흐름을 명시하라. 해당 흐름까지 예외로 승인받아 intent·plan을 맞추거나, 기존 예외 밖 쓰기는 A2 미충족으로 남겨 종결 보고에 기록하라.

Next steps:
- A2 예외 누락을 처리하거나 미조치 명세 편차로 기록한다. 라운드 2/2 이후 재리뷰는 시작하지 않는다.
- E1~E8 블록은 기준 revision과 정확히 같다. 옛 측정 6a2769d 이후 제품 코드 세 파일이 바뀌었으므로 R1 원문 복원 뒤 최종 gate와 새 빌드 E8을 다시 측정한다.
```

raw JSON:

```json
{"verdict":"needs-attention","summary":"고정 범위 8c7a390..4cff605를 읽기 전용으로 대조했다. Stage 6 두 변경의 동작 차이는 찾지 못했지만 A2 예외 밖 상태 쓰기가 남아 명세 충족 판정은 보류다. 시험은 재실행하지 않았으며 R1 원문 복원·최종 gate 재측정·새 빌드 E8은 평가 단계에 남는다.","findings":[{"severity":"medium","title":"결정 8의 A2 예외가 입력창의 실행 상세 상태 변경을 빠뜨린다","body":"결정 8은 설정 적용·변경 신호·순서 저장·current·focusAfter를 예외로 열거하지만 composer.js:29-35는 sending과 함께 pickMsg를 변경한다. core/state.js:36은 pickMsg를 실행 상세 선택값으로 정의하고 detail.js:81-85가 이를 소비하며, work/stage2.md:61에도 이 쓰기가 기록돼 있다. 입력창과 세부정보를 별도 도메인으로 보는 것은 plan의 도메인 분리에 따른 추론이지만, 이 경우 메시지 전송 뒤 실행 상세 선택을 초기화하는 흐름은 열거한 예외에 없어 A2 미충족이 남는다. 이를 모든 화면 흐름에 대한 포괄 승인으로 간주하면 사용자가 승인한 예외 범위가 조용히 넓어진다.","file":"docs/tasks/1z-12.code-tidy/intent.md","line_start":63,"line_end":64,"confidence":0.86,"recommendation":"상태 변경 표를 속성별 도메인 소유권과 대조하고 pickMsg 초기화 등 누락 흐름을 명시하라. 해당 흐름까지 예외로 승인받아 intent·plan을 맞추거나, 기존 예외 밖 쓰기는 A2 미충족으로 남겨 종결 보고에 기록하라."}],"next_steps":["A2 예외 누락을 처리하거나 미조치 명세 편차로 기록한다. 라운드 2/2 이후 재리뷰는 시작하지 않는다.","E1~E8 블록은 기준 revision과 정확히 같다. 옛 측정 6a2769d 이후 제품 코드 세 파일이 바뀌었으므로 R1 원문 복원 뒤 최종 gate와 새 빌드 E8을 다시 측정한다."]}
```

판정:

- R2-1(medium) "결정 8의 A2 예외가 입력창의 실행 상세 상태 변경을 빠뜨린다" — **사실**. 상태 변경 표를 속성별 소유와 대조하니([work/stage6](work/stage6.md) "A2 예외 범위 대조") 결정 8 밖 쓰기가
  일곱이다 — `composer.js` · `conv.js` 의 `pickMsg`(세부정보), `shell.js` 의 `previewAll`(개요) · `focusMsg`(대화), `rail.js` 의 `unseen`(사이드바), `sidebar.js` 의 `waitingBy`(개요) · `unseenAll`(종 점).
  고침 = 지적의 둘째 권고대로 기록: 예외를 구현자가 넓히지 않고, report 에 A2b "결정 8 안 충족 · 밖 일곱 미충족" 으로 남겨 사용자 결정으로 올린다. 코드 변경 없음(라운드 상한 2/2 — 재리뷰 없음). 아래 미조치 지적 · 원장.
- stage 6 의 지운 둘(`kick` · 자동 왕복 상한 기본값)은 리뷰어도 동작 차이를 찾지 못했다. 남긴 셋 · R1 뒤 문서 고침 · 재개 절차에는 지적 없음.
- next_steps 둘째(옛 측정 무효 — R1 원문 되살림 · gate 다시 · 새 빌드 E8)는 이 평가 단계에서 했다 — 위 원장(측정 커밋 `4cff605`) · E8. `check_gates` 첫 실행은 E7 안의
  `check_ddd_docs` 가 되살린 옛 원장(`6a2769d`)을 읽어 FAIL(신선도) — 원장이 `4cff605` 로 바뀐 뒤 다시 돌려 8/8 PASS.

## 미조치 지적

| 지적 원문 | 판단 | 미조치 사유 |
|---|---|---|
| R2-1 "결정 8의 A2 예외가 입력창의 실행 상세 상태 변경을 빠뜨린다" — Recommendation "…해당 흐름까지 예외로 승인받아 intent·plan을 맞추거나, 기존 예외 밖 쓰기는 A2 미충족으로 남겨 종결 보고에 기록하라." | 사실 — 결정 8 밖 다른 도메인 상태 쓰기 일곱(`composer.js` · `conv.js` `pickMsg`, `shell.js` `previewAll` · `focusMsg`, `rail.js` `unseen`, `sidebar.js` `waitingBy` · `unseenAll`) | 예외 범위는 사용자 결정 — 구현자가 예외를 넓히지 않는다. 도메인별로 쪼개는 동작 불변 리팩터는 라운드 상한 2/2 뒤라 재리뷰 없이 할 범위가 아님. report 에 A2b 부분 미충족으로 기록 · 사용자 결정 대기 |

# eval.md — 1z-18.ui-fixes 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: b8ff6a1a576acfee57d52471eaec1784984aa7f9

- [x] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

- [x] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.37s

- [x] E3: A3·A4 — 웹UI 로직 시험(`starDetail` · `agentTag`)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A1·A2·A3·A4·A6·A8 — 화면 원본 literal(의견 칸 · archive 스위치 · 즐겨찾기 ★ · 세부정보 · 에이전트 줄 · 북마크 목록 해제 · 브라우저 알림 문구) · `:root` 밖 리터럴 0
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s

- [x] E5: A5 — 버전 1.4.3(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v143=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.4.3' for t in v)))"
  EXPECT: crates=4 v143=4
  EVIDENCE: crates=4 v143=4

- [x] E6: A1·A2·A3·A4·A6 — 실제 화면(디버그 빌드 · ego-browser) 두 테마 — 의견 칸 폭 · 단추 오른쪽 32px(목록 · 보드 · 대화), archive 토글 동작, 즐겨찾기 ★ 해제 · 둘째 줄, 에이전트 응답 항목 1100px 한 줄 · 좁은 창 두 줄, 북마크 목록 해제, 콘솔 오류 0
  EVIDENCE: Claude(구현자) 2026-10-28 — 디버그 빌드 1.4.3(`target\t17`) + scratchpad `ui1z18`(`ui1z17` 사본), ego-browser 라이트 · 다크(r01 ~ r07). 의견 칸: 요청 목록 폼 981 = 단추 줄 안쪽 981 · 보드 198 = 안쪽 198 · 대화 `.msg-body` 아래 238, 취소 · 확인 32px · 오른쪽 끝 = 줄 끝. archive `role="switch"` 36 × 20 켜면 목록 보임 · 끄면 숨김. 즐겨찾기 ★ 링크 앞 · 줄 가운데, 둘째 줄 "김도윤 · planner · 완료" 등, ★ 누름 → 그 줄 빠짐. 에이전트 응답 줄 = 타일 26px · 제목 · "10.22 22:22" · "[coder]", 항목 1100(창 1600 · 1700 · 1920) 한 줄 높이 22, 1088 · 1028 · 688 두 줄 높이 46. 북마크 목록 6 줄 머리 해제 단추(`aria-pressed=true`) 누름 → 5 줄 · 서버 `/api/saved` 에서 빠짐. 콘솔 · 페이지 오류 0. 근거 [work/stage2](work/stage2.md).

- [x] E7: A5 — 문서 규약 · 크레이트 경계 · 코드 정리 · README · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_code_tidy.py && python tools/check_readme.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('web=%d guide=%d product=%d readme=%d'%(r('docs/manual/dev/web-ui.md').count('starDetail'),r('docs/manual/user/user-guide.md').count('archive 항목 보기'),r('PRODUCT.md').count('#1z-18'),r('README.md').count('1.4.3')))"
  EXPECT: /web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]* readme=[1-9][0-9]*/
  EVIDENCE: web=1 guide=1 product=1 readme=1

- [x] E8: A7 — 트레이 확인할 일 건수 = 받은 요청 · 연결 요청 · 연결 승인 · 내 요청 상태 변경만 · 알림 설정(꺼짐 · 일시 중지 · 집중)과 무관 · 끄기로 안 지워짐, 브라우저 알림은 그대로
  CHECK: cargo test -p coworkagent-app --test notifier --test events_api
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.20s


WEAK: E6 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 측정 기록

- 첫 측정 `0e5623b` 7/7 PASS(E6 수동 근거 = work/stage2).
- 리뷰 라운드 1 · 2 수정(`104b75d` · `5f2f816`) 뒤 다시: 첫 실행 E7 만 미충족(원장의 옛 측정 커밋 — 문서 규약 체커), 둘째 실행 7/7 PASS.
- 리뷰 상한(2/2) 뒤 추가 요청 2 건 — 결정 10(트레이 새 요청 = 받은 요청 · 연결 요청, 알림 설정과 무관 — `29ab7e9`) · 결정 11(내 요청 상태 변경도 — `e14b366`), E8 추가.
  `e14b366` 에서 첫 실행 E7 만 미충족(옛 측정 커밋), 둘째 실행 8/8 PASS. 두 변경은 코드리뷰 상한이라 **재리뷰 없음**(사용자가 라운드 연장을 지시하면 그때).
- 확인 답(결정 12 · 13 · 14 — 연결 승인 · 툴팁 "확인할 일 N건" · 설정 "브라우저 알림" · 권한 없음 안내) `b8ff6a1` — 첫 실행 E7 만 미충족(옛 측정 커밋), 둘째 실행 8/8 PASS.
  재리뷰 없음(상한). RED 먼저: tray `tip_lines_and_windows_limit` · notifier 2 · ux_literals PRESENT 6 행 FAILED → 모두 ok.
  시험: `notifier` 18 · `events_api` 4 ok — 트레이 기대를 바꾼 시험 9 개(알림 기대는 그대로) · 새 시험 `tray_marks_only_requests` · `tray_ignores_settings`(RED 먼저 —
  `tray_marks` 없음 컴파일 오류, `AskState` 기대 1 FAILED). 실제 트레이 화면(Windows · macOS)은 확인하지 않음 — `tray.rs` 는 그대로, 건수 함수만 바뀜.

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex --model gpt-6.1-sol --effort high` — receipt `launch.effective` 같음, 작업 폴더 `C:/Users/tskim/.claude/skills` — 대상 저장소 밖, `git -C` 로 읽기만).
Run `run_4776cd7af05c` · Task `task_8ee96cfd2496` · Dispatch `ctx_47c7305a7ea5` · 터미널 `term_c2a65b22-c955-46c4-b2db-f6f5413caac2`(시작 receipt `turn_start_unobserved` — 결과 회신으로 시작 확인).
범위 `6086677..0e5623b`(self-collect), 메타 `요청 1z-18 · revision 0e5623b277999d9c2ddd5b1d2a8c96f21dc9b031 · 라운드 1/2`, worker_done `msg_db0ef6930e10`. 리뷰어 Dispatch `worker-release` → `released`(`closed_agent_terminal`).

```text
# Codex Adversarial Review

Target: branch diff against 60866771598d48c9bc8419db5abfd344e28a01f4
Verdict: needs-attention

라운드 1/2 고정 revision의 A1~A6와 상태·렌더 경로를 읽기 전용으로 검토했다. 북마크 해제의 기존 대화 아이콘 불일치와 화면 이동 중 조회 취소를 발견했다. 두 경로 수정과 실제 브라우저 회귀검증 전 출하 보류가 타당하다.

Findings:
- [medium] 북마크 해제 후 기존 대화의 켜진 아이콘이 남음 (crates/app/web/js/activity.js:55-58, confidence 0.93)
  [확인] unsaveRow는 state.msgs의 saved만 바꾸고 기존 말풍선 단추를 교체하지 않는다. messagesSigOf(logic/message.js:118-126)는 saved를 제외하고 openConversation(conv.js:489-518)은 같은 대화·서명이면 DOM을 유지한다. 고정 revision 함수 메모리 하네스에서 서버·두 사본 saved=false, 단추 on=true, 재렌더 1회 유지 확인. [추론] 대화→북마크 목록에서 해제→같은 대화 복귀 시 aria-pressed=true가 남고, 해제하려고 다시 누르면 오히려 재등록된다(A6 위반; 실제 브라우저 미실행).
  Recommendation: 해제 성공 시 기존 말풍선 북마크 단추도 현재 saved 값으로 교체하거나, 대화 진입 시 saved 상태만 별도 동기화한다. 같은 대화 왕복 후 모양·aria-pressed·다음 클릭 동작을 검증한다.
- [medium] 해제 완료의 백그라운드 재조회가 새 화면 조회를 무효화함 (crates/app/web/js/activity.js:58-58, confidence 0.9)
  [확인] PATCH 뒤 무조건 loadSaved()를 호출한다. 이 함수(activity.js:151)는 모든 화면이 공유하는 latest.next()를 발급하므로, PATCH 대기 중 시작한 openConversation의 ticket을 폐기한다(conv.js:495-504). 지연 Promise 하네스에서 대화 3을 열었지만 후속 loadSaved로 ticket 4→5, 대화 응답 폐기·msgsOf=2 유지 확인. [추론] 빠르게 다른 대화로 이동하면 비어 있는 대화가 다음 5초 새로 고침까지 남고 요청 실패 표시도 없다(실제 브라우저 미실행).
  Recommendation: PATCH 완료 시 현재 화면이 북마크 목록일 때만 loadSaved를 실행하고, 다른 화면에서는 해당 화면의 조회 ticket을 건드리지 않는다. PATCH 지연 중 대화·요청 화면으로 이동하는 순서 테스트를 추가한다.

Next steps:
- 두 북마크 경로 수정 후 실제 브라우저에서 대화 왕복·PATCH 지연 중 화면 이동을 검증한다.
- 기존 목록·archive switch에서 추가 차단 결함은 확인하지 못했다; 전체 빌드·테스트와 Safari 15는 이번 리뷰에서 실행하지 않았다.
```

판정 · 조치:
- 지적 1(해제 뒤 열린 대화 아이콘 켜진 채) — **사실**. 대화 서명 `messagesSigOf` 는 저장 여부를 안 봐 같은 대화로 돌아와도 말풍선 단추를 다시 그리지 않는다. `conv.js` `toggleBookmark` 의 뒤처리(값 · 아이콘 · 머리 수 · 실행 상세)를
  `markSaved(cur, on)` 으로 빼 `activity.js` `unsaveRow` 도 씀.
- 지적 2(해제 뒤 다시 읽기가 새 화면 조회를 버림) — **사실**(`loadSaved` 가 공유 `latest.next()` 를 땀). 목록 다시 읽기는 `route(location.hash).view === "saved"` 일 때만.
- `ux_literals` PRESENT 2 행(RED 먼저 — 2 행 "app 에 없음" → ok 5). node 181/0 · cargo test 84 ok · clippy · tidy PASS.
  디버그 빌드 화면(r08): 대화 2 글 5 말풍선 북마크(`aria-pressed=true`) → 북마크 목록에서 해제 → 대화 2 로 돌아옴 = `aria-pressed=false` · `on` 없음 · "북마크";
  PATCH 1.5 초 늦춘 채 해제 누르고 곧바로 대화 11 → 대화 11 글 6 개 모두 그려짐, 콘솔 오류 0.
- 수정 커밋 `104b75d`.

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex(같은 실행 명령 · `launch.effective` 같음 · 같은 작업 폴더). Run `run_4776cd7af05c` · Task `task_47fbac200ecc` · Dispatch `ctx_97e1f6beeac6` ·
터미널 `term_1017f0fd-d67e-402a-a730-050030045078`(시작 receipt `input_accepted`). 범위 `6086677..104b75d`(self-collect · 라운드 1 원문 동봉), 메타 `요청 1z-18 · revision 104b75d5ad1ea06b0f87d496d708b97e58437c55 · 라운드 2/2`,
worker_done `msg_a4047517f54d`. 리뷰어 Dispatch `worker-release` → `released`.

```text
# Codex Adversarial Review

Target: branch diff against 60866771598d48c9bc8419db5abfd344e28a01f4
Verdict: needs-attention

라운드 2/2 고정 범위의 A1~A6와 R1 수정 두 경로를 읽기 전용으로 검토했다. 화면 이동 후 공유 조회 순번 취소는 닫혔지만, 지연된 대화 응답이 북마크 아이콘 불일치를 재발시키는 경로를 함수 하네스에서 확인했다. 해당 경쟁 조건 수정과 브라우저 검증이 남아 출하 보류가 타당하다.

Findings:
- [medium] 해제 중 새 대화를 열면 늦은 조회가 켜진 북마크 아이콘을 남김 (crates/app/web/js/activity.js:58-60, confidence 0.89)
  [확인] PATCH 완료 시 state.msgs에 있는 글만 markSaved한다(58~59행). 북마크 목록에서 이전에 열지 않은 대상 대화로 이동해 GET이 해제 전 saved=true를 읽고, PATCH 완료 뒤 도착하면 대상 글은 이 검사 때 없어 동기화를 건너뛴다. 고정 revision의 unsaveRow·openConversation·messagesSigOf를 실행한 지연 Promise 하네스에서 다음 GET으로 state.saved=false를 받아도 renderedButtonOn=true, 메시지 렌더 1회 유지 확인: saved가 서명에서 빠져 있기 때문이다(conv.js:509~523, logic/message.js:118~126). [추론] 실제 브라우저에서도 켜진 아이콘·aria-pressed=true가 남고 다음 클릭이 재등록으로 작동할 수 있다(A6 미충족; 실제 HTTP·브라우저 지연 재현 미실행).
  Recommendation: 해제 완료를 메시지 ID별로 기록해 먼저 시작한 GET 결과에 적용하고, openConversation에서 본문 서명과 별도로 북마크 단추를 최신 메시지 값과 동기화한다. 다른 대화→북마크 목록→해제 대상 대화 진입에서 GET 읽기→PATCH 완료→GET 도착→다음 조회 순서를 검증한다.

Next steps:
- 잔여 경쟁 조건을 수정하고 아이콘·aria-pressed·머리 수·상세 목록·다음 클릭의 일치를 검증한다. 라운드 2/2 상한으로 추가 재리뷰는 시작하지 않는다.
- A1~A4와 기존 convItem 호출자에서 추가 차단 결함은 확인하지 못했다. 제공된 gate·화면 기록을 검토했으며 이번 리뷰에서 빌드·전체 테스트·실제 브라우저·macOS Safari 15는 실행하지 않았다.
```

판정 · 조치(라운드 2 수정 — 상한이라 재리뷰 없음):
- 지적 1(해제 중 연 대화의 늦은 응답이 켜진 아이콘을 남김) — **사실**. 실제 화면(r09 — PATCH 1.5 초 늦춤 · 대화 11 첫 GET 응답 2.5 초 붙잡음)으로 재현:
  `104b75d` 빌드 = 늦은 응답 뒤 켜진 아이콘이 다음 새로 고침(서버 `saved=false`)에도 그대로(6.5 초 뒤 `aria-pressed=true`).
  고침 — `conv.js` `openConversation` 이 다시 그리기 여부와 별개로 북마크 단추만 받은 글 값에 맞춤(서명은 그대로). `messagesSigOf` 주석 함께.
  `ux_literals` PRESENT 1 행(RED 먼저 — "app 에 없음" → ok 5). 수정 빌드에서 r09 3 회: 늦은 응답 뒤 켜짐(서버 해제 전 값) → 다음 새로 고침에 꺼짐, 콘솔 오류 0.
- 수정 커밋 `5f2f816`.

## 미조치 지적

없음

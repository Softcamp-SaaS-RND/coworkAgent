# stage 4 — 실제 화면 · 문서 · 버전

## 목표

plan "설계 · stage 4" — 실제 화면 사진(E7, 결정 13)을 사용자에게 보이고 고칠 곳을 받아 반영, stage2-2(타일 편집 이름 칸 넘침) 고침, 문서(web-ui · 사용자 가이드 ·
PRODUCT · README), 버전 1.3.7.

## 산출 파일

- 1차(커밋 `50bd3de`) — `app.css` `.tile-form` `flex-wrap: nowrap`(stage2-2 — 전역 `form` 의 wrap 이 줄 폭을 입력 기본 폭 266px 로 잡아 사이드바 밖으로 넘침 → 239px),
  [web-ui](../../../manual/dev/web-ui.md) · [사용자 가이드](../../../manual/user/user-guide.md) · PRODUCT(#1z-13 줄만 스테이징) · README 1.3.7 · `crates/{core,net,agent,app}/Cargo.toml` · `Cargo.lock` 1.3.7.
- 2차(E7 답) —
  - `crates/app/web/js/logic/conv.js` `convKindIcon` 문의(`out`) = `user`(결정 16), 주석 맞춤 `core/widgets.js` · `sidebar.js` · `app.css`.
  - `crates/app/web/app.css` — 요청 보드 칸 위 3px · 카드 왼쪽 4px · 의견 상자 왼쪽 3px 상태 색 띠와 상태별 색 규칙 12줄 지움(stage4-1), 메뉴 안 켜기 항목
    (`.menu button[role="menuitemcheckbox"]`) = 설정 화면 스위치 모양 — `::after` 막대 36 × 20 · `::before` 손잡이(stage4-2), 고르기 항목(`menuitemradio`)은 자리 맞춤용 `content: ""` 를 제 규칙에.
  - `crates/app/web/js/requests.js` — 띠에만 쓰이던 상태 class(`board-col` · `board-card` · `req-feedback` 의 `req-{상태}`) 지움, 의견 상자 주석 맞춤.
  - `crates/app/tests/ux_literals.rs` — 바뀜 1(`convKindIcon`) · 새 PRESENT 1(메뉴 스위치 손잡이) · 새 ABSENT 4(띠 셋 · 손잡이 없는 메뉴 알약).
  - `crates/app/web/view.test.mjs` — `conv_kind_icon` 문의 = `user`.
  - `design/taste.py` — '옆 띠' 를 위 띠 · 상태 색(`--run` · `--ok` · `--danger` · `--wait` · `--muted`)까지 넓힘, `person-icon-avatar` 에서 종류 칩 사람 아이콘 뺌(개정 ⑦ 되돌림).
  - `crates/agent/tests/conv_leave.rs` — stage2-3 경합 고침(아래).
  - [intent](../intent.md) 결정 16 · [plan](../plan.md) 개정 ⑨ · ⑩, web-ui · 사용자 가이드 · PRODUCT(#1z-13 줄만 스테이징) · README.

## 실제 화면(E7)

- 디버그 빌드 1.3.7 + scratchpad `ui1z12\data` 사본 + 시드(scratchpad `seed1z13.py` — 즐겨찾기 2 · 북마크 3 · 요청 5(보낸 대기 · 수용 · 완료 / 받은 대기 · 반려) · 공유 사본 1,
  이미 있던 상대 3 · 그룹 2 · 실행 중 1 · 오류 1 · 안 읽음), ego-browser 1440 × 900, 25곳 × 라이트 · 다크 = 50장(scratchpad `ego/1z13/e7.mjs`).
- 25곳 = plan 목록대로(앞 줄 "화면 23" 과 다름 — 개정 ⑨). '상대 대화 목록' = 박서연 타일 + 공유 대화를 연 화면.
- 판독(반영 뒤 50장 합계): 콘솔 · 페이지 오류 0(열기 전에 심은 error · unhandledrejection · console.error 감시 — 브라우저 자원 실패 404 · CSP 는 세지 않음),
  사람 SVG 0, 아바타 자리 사람 아이콘 0, 주 단추 14 / 14 = `--primary`(라이트 rgb(39, 39, 42) · 다크 rgb(228, 228, 231)), 빈 머리글자 0,
  글자 대비 미달 0 / 4,278(그 점에 실제로 그려진 바탕 — `elementsFromPoint`), 채도 있는 색 = 토큰만(`--accent` · `--accent-soft` · `--agent` · `--agent-soft` · `--agent-tile` ·
  `--ok` · `--run` · `--danger` · `--danger-soft`), 그라데이션 0, 그림자 = 떠 있는 창(새로 만들기 · 프로필 메뉴 · 검색) + 입력창 질의 · 요청 고른 칸 1px 고리, 상단 판 표시 v1.3.7.
  종류 아이콘 = `bot` · `user` · `eye`(검색 메시지 줄 = `chat`).
- 찍는 중 고친 것(스크립트): 원문 보기가 다음 대화 화면에 남음 → 대화 보기로 되돌린 뒤 찍음, 접힘 저장 실패 알림(stage2-1)이 뒤 화면에 남음 → 주고받은 메시지를 끝으로 · 테마마다 새로 열기,
  대비가 조상 바탕으로 잘못 잡힘(레일 타일 이름은 단추 상자 밖) → 그 점의 바탕.
- 아티팩트 https://claude.ai/artifact/Ndz7qpArMaEjgDwf845CfT — 버전 1(첫 사진 · 제안 셋) → 버전 2(반영 뒤 25곳 · 반영 전후).

## 사용자 답

- 2026-10-25 버전 1 —
  - E7: "대화목록 섹션에서 내가 보낸 메세지는 화살표위 아이콘이 아닌 사람 아이콘으로 변경" → intent 결정 16.
  - stage4-1 "지움 (Recommended)" · stage4-2 "앱 스위치와 같게 (Recommended)".
  - stage4-3 "사람은 사람 아이콘으로 사용. 단순히 에이전트,사람 구분하기 위함" → 내 활동 발화자 표시 그대로.
- 2026-10-26 버전 2 — E7 "확인 · 평가 진행", stage2-3 "이 Task 에서 시험만 고침 (Recommended)"(앞 답 "다음 Task" 개정 — 개정 ⑩).

## 설계 결정

- 결정 16 이 닿는 곳 = 유형 아이콘 전부(대화 줄 · 개요 목록 · 미리보기 · 검색 · 대화 머리) — 한 함수 `convKindIcon` 이고 답이 "사람 = 사람 아이콘" 원칙(구현자 해석, report 에 적음).
- stage4-2 질문에 '대화 메뉴' 도 적었으나, `.menu` 안 켜기 항목은 프로필 '알림 일시 중지' 하나다. 대화 ⋯ 메뉴의 그룹 항목은 `#float` 의 여럿 고르는 체크 상자
  (`.float button[aria-checked]::before`)라 그대로 — 꺼짐 1px `--border` 는 대비가 낮다(본 것, 미조치).
- 상태 class 를 지운 것은 띠 규칙이 유일한 쓰임이라서다. 보드 머리 칩의 `tag ask req-{상태}` · 요청 상자 머리 `stat req-{상태}` 는 이 Task 전부터 있던 것이라 두었다.

## stage2-3 — 시험 경합

- 증상: `conv_leave::delete_while_waiting_leaves_no_rows` 가 전체 실행 4번 중 3번 실패(left 1 · right 0), 혼자는 통과.
- 원인: B 는 글 상태를 '실행 중' 으로 바꾼 뒤 실행기를 부른다. 시험은 상태가 '실행 중' 이 된 것만 기다리고 실행기 기록 수(`runs`)를 쟀다 — 실행기에 닿기 전이면 0,
  뒤에 앞 대화(c1) 실행이 기록돼 1. 지운 대화(c2)가 실행된 것이 아니다.
- 고침: 실행기 기록 1건(`runs.len() == 1`)을 기다린 뒤 세고, 실행기 입력에 지운 대화 글("기다림")이 없음을 따로 확인. 제품 코드 그대로.

## 테스트 결과

- RED(E7 답) — `cargo test -p coworkagent-app --test ux_literals --test css_tokens` → `literals_present` · `literals_absent` FAILED(띠 셋 · 알약 · `user` · 손잡이),
  node `conv_kind_icon` not ok(# fail 1), `taste.py` → `violation slop 15`.
- GREEN — 같은 명령 `ok. 3 passed` · `ok. 5 passed`, node `# pass 172` · `# fail 0`, `taste files=50 checks=9 violations=0`, `labels n=280 missing=0 added=0`,
  `check_code_tidy.py` PASS(116 · 960 · 0), `cargo clippy --workspace --all-targets -- -D warnings` → `Finished`.
- 워크스페이스(stage2-3 고치기 전) — 84 묶음 · 521 passed · 1 failed(stage2-3). 고친 뒤 결과는 아래 줄.
- 고친 뒤 — `cargo test -p an-agent --test conv_leave` 10번 10 통과, `cargo test --workspace --no-fail-fast` 3번 모두 exit 0 · 84 묶음 · FAILED 0.

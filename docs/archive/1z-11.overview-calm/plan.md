# plan — 1z-11.overview-calm

> 근거: [intent](intent.md)(승인 2026-10-24 — 결정 1 ~ 7). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md)(D35 · S4 #1z-7).
> 브랜치 `task/1z-11-overview-calm`(작업 폴더 `D:\dev\git\coworkAgent-1z-11`). 기준 커밋: `e65f617`(main). plan 승인: 2026-10-24(사용자 "C 시안 채택." 과 질문 답 넷으로 갈음, 구현자 해석).
> 시안: https://claude.ai/artifact/94xSakSabDBSZheTcs2qUx 의 C(`CalmPaper.dc.html`).

## 목표

개요 미리보기(`#home_preview` 안)를 시안 C "편안함 · 종이 톤" 으로 — 상대 이름 14px · 500, 절 확대 없음, 카드 한 색 테두리 · 유형 아이콘 색, 말풍선 C 색 · 12px. 웹UI 만(Rust 코드 없음). 버전 1.3.5 그대로.

### 성공 기준(intent 사본)

- A1: 미리보기 절 · 카드가 시안 C 모양 — 종이 바탕 · 절 사이 가는 선 · 머리(이름 14px 500 · 접속 · 대화 수 · 안 읽음 · 대기 요청, 상대 타일 없음) · 카드(한 색 테두리 · 유형 아이콘 색 · 제목 · "새 글 n" · 말풍선 · 바닥 "에이전트 · 시각" + 상태 점 · 글자) · "+n개 더 보기"(시험 · 실제 화면).
- A2: 말풍선 = 12px · 줄간격 1.5 · 2줄 · 내 글 / 상대 글 / 에이전트 · 실행 중 / 오류 색이 C 와 같다(시험 · 실제 화면).
- A3: 절 확대 없음, "미리보기 크기" 100 ~ 200% 는 그대로 카드 폭 · 본문 높이를 바꾸고 열 수를 다시 잰다(시험 · 실제 화면 100 · 200%).
- A4: 다크 테마에서도 같은 구조 · 어두운 토큰(종이색이 밝게 남지 않음), 글자 대비 4.5:1 이상(실제 화면).
- A5: 회귀 없음 — PROJECT.md 검증 명령 PASS(토큰 시험 포함), 매뉴얼 · PRODUCT · masterPlan 반영.

## 설계

### stage 1 — 토큰 · CSS · 마크업 · 문서

- `app.css` 첫 `:root`(`--pv-scale` 옆) — 미리보기 전용 토큰(값 = 시안 C 그대로):
  - 색: `--pv-bg #F5F3EE` · `--pv-card #FBFAF7` · `--pv-line #E3DED5` · `--pv-line-soft #EEEAE2` · `--pv-line-hover #CFC8BB` · `--pv-more-line #D3CCBF` · `--pv-text #2D2B28` · `--pv-muted #5F5A52` ·
    `--pv-faint #6A655D` · `--pv-link #355E7C` · `--pv-unseen #9DB3C4` · `--pv-new-dot #4F7FA3` · `--pv-mine #ECE8E0` · `--pv-theirs #FFFFFF` · `--pv-theirs-line #E6E1D8` · `--pv-agent-bg #E9EFE9` ·
    `--pv-agent #2E4A3B` · `--pv-agent-who #3F5F4E` · `--pv-err-bg #F6E6E2` · `--pv-err #8A3A2E` · `--pv-run #7D5520` · `--pv-run-dot #C08A3E` · `--pv-ok #466B52` · `--pv-ok-dot #5E8C6A` · `--pv-wait-dot #A7A196` ·
    `--pv-err-dot #B5544C` · `--pv-kind-in #4E7A66` · `--pv-kind-out #4A6688`.
  - 크기: `--pv-fs-head 12.5px` · `--pv-fs-title 13.5px` · `--pv-fs-who 10.5px` · `--pv-fs-new 11.5px` · `--pv-lh 1.65` · `--pv-lh-msg 1.5` · `--pv-radius 10px` ·
    간격 `--pv-list-pad 0 40px 48px` · `--pv-sec-pad 22px 0 26px` · `--pv-head-gap 4px 10px` · `--pv-gap-cards 18px` · `--pv-gap-xs 6px` · `--pv-gap-who 5px` · `--pv-card-head-pad 10px 14px 4px` ·
    `--pv-body-pad 8px 14px 12px` · `--pv-msg-pad 6px 11px` · `--pv-foot-pad 9px 14px`. 이름 14px = `--fs-md`, 말풍선 · 바닥 12px = `--fs-xs`, 500 = `--fw-medium`, 말풍선 둥글기 8px = `--radius-lg`.
  - 다크: 색 토큰마다 `--dark-pv-*`(어두운 종이 톤) + `:root[data-theme="dark"]` 대입 줄.
  - `--pv-zoom` 지움(결정 5).
- `app.css` 미리보기 규칙(`.pv-*` 블록 교체):
  - `.pv-list` = 종이 바탕 · `--pv-list-pad` · 줄간격 `--pv-lh`. `.pv-sec` = `--pv-sec-pad`, 절 사이 `.pv-sec + .pv-sec` 위 1px 선(첫 절 위는 개요 머리 아래 선이 이미 있음). `zoom` 없음.
  - `.pv-head` = 기준선 맞춤 · `--pv-fs-head` · `--pv-muted`, 이름 `strong` = `--fs-md` · `--fw-medium` · 최대 240px 말줄임. 접속 `.pv-pres`(점 6px — 받기 가능 `--pv-ok-dot` · 집중 `--pv-run-dot` · 끊김 `--pv-err-dot` + 글).
    `.stat` = `--pv-link` 500, 대기 요청 `.stat.pv-wait` = `--pv-run`. 대화 없는 절 `.pv-none` = 점선 상자 없음(머리 + 오른쪽 "세션 생성").
  - `.pv-cards` = `repeat(auto-fill, minmax(min(100%, calc(256px * var(--pv-scale))), 1fr))` · 간격 `--pv-gap-cards`(`fitPreviewCols` 가 px 트랙을 그대로 잰다).
  - `.pv-card` = `--pv-card` · 1px `--pv-line` · `--pv-radius` · 그림자 없음, 마우스 = `--pv-line-hover`, 안 읽음 `.unseen` = 테두리 `--pv-unseen`(윤곽선 없음). 공유 사본 점선(기존 공통 규칙) 그대로.
  - 머리 `.pv-card-head` = 유형 아이콘(`.pv-kind` — 바탕 없음 · 18px 상자 · 아이콘 15px · `in` = `--pv-kind-in` · `out` = `--pv-kind-out`) · 제목 `--pv-fs-title` 500 · `.pv-new`(점 `--pv-new-dot` + "새 글 n", `--pv-link`).
  - 본문 `.pv-body` = 높이 `calc(132px * var(--pv-scale))` · `--pv-body-pad` · 간격 `--sp-3`(#1z-8 사용자 "말풍선 사이 여백" 행 그대로 — 시안 8px 대신, 개정 기록 ①) · 바탕 없음. 말풍선 `.pv-msg` = 최대 88% · `--pv-msg-pad` · `--radius-lg` · `--fs-xs` · `--pv-lh-msg` · 2줄(`.pv-text` 그대로).
    내 글 `--pv-mine`, 상대 글 `--pv-theirs` + 1px `--pv-theirs-line`, 에이전트 · 실행 중 `--pv-agent-bg` · 글 `--pv-agent` · 이름 `--pv-agent-who`, 오류 `--pv-err-bg` · `--pv-err`. 이름 `b` = `--pv-fs-who` 500 `--pv-muted`.
  - 바닥 `.pv-card-foot` = 위 1px `--pv-line-soft` · `--fs-xs` · `--pv-faint`, 왼쪽 `.pv-who` "{에이전트} · {시각}" 말줄임, 오른쪽 `.pv-state`(점 + `stateText` — 실행 중 `--pv-run` · 완료 `--pv-ok` · 오류 · 답 미전달 `--pv-err` · 보냄 · 실행 대기 회색).
  - "+n개" `.pv-more` = 1px 점선 `--pv-more-line` · `--pv-radius` · `--pv-link` 500 · 최소 높이 `calc(132px * var(--pv-scale))`.
  - 공유 규칙 `.home-kind, .pv-kind` → `.home-kind` 만(목록 보기 그대로).
- `app.js`:
  - `previewSectionEl` 머리 = 이름 · `.pv-pres`(점 + 접속 글) · "· 대화 n" · `stat` "· 안 읽음 n" · `stat pv-wait` "· 대기 요청 n"(`rowTile` 뺌).
  - `previewCardEl` 머리 = `kindChip(c, "pv-kind")` · 제목 · 안 읽음이면 `pv-new` "새 글 n", 바닥 = `pv-who`("{에이전트} · {시각}" — 시각 없으면 에이전트만) · 상태 있으면 `pv-state st-{상태}`(에이전트 아이콘 · `pv-unread` 뺌). "+n개" → "+n개 더 보기".
- 문서: [web-ui](../../manual/dev/web-ui.md)(미리보기 머리 · 카드 · 토큰) · [사용자 가이드](../../manual/user/user-guide.md)(개요 미리보기 줄 — 점선 줄 · "+n개" · 안 읽음 글자) · PRODUCT(#1z-11 줄) ·
  masterPlan D35 · S4 #1z-7(미리보기 카드 = 아이콘 색만) · 개정 기록(종결).

### 바꾸는 시험(의도된 변경 — 약화 아님)

- `ux_literals.rs`: 카드 폭 240 → 256(행 283), 말풍선 · 이름 규칙(287 · 288 — 11 → 12px · 10 → 10.5px, 시안 C · 사용자 채택), 본문 120 → 132(296), 안 읽음 윤곽선 → 테두리 색(436), `--pv-zoom: 1.15;` 있음 → 없음(294 → ABSENT).
  새 PRESENT: 이름 `--fs-md` · `--fw-medium`, "새 글 ", "개 더 보기", `.pv-sec + .pv-sec`, `--pv-bg` 다크 대입. 새 ABSENT: 미리보기 머리 `rowTile`, `pv-unread`. 2줄(297) · 글 읽기(302 · 305) 행 그대로.

## 평가 항목

- [ ] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A3 — 웹UI 로직 시험(미리보기 크기 · 말풍선)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A3·A4 — 화면 원본 literal(머리 · 카드 · 말풍선 · 확대 없음 · 다크 대입) · CSS 토큰
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A5 — 문서 규약 · 크레이트 경계 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('web=%d guide=%d product=%d'%(r('docs/manual/dev/web-ui.md').count('--pv-bg'),r('docs/manual/user/user-guide.md').count('새 글'),r('PRODUCT.md').count('#1z-11')))"
  EXPECT: /web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]*/
  EVIDENCE: pending
- [x] E6: A1 · A2 · A3 · A4 — 실제 화면: 디버그 빌드 + 시험 데이터 · ego-browser 1440 × 900 — 미리보기 절 · 카드 · 말풍선이 시안 C(이름 14px 500 · 말풍선 12px · 확대 없음), 미리보기 크기 100 · 200%
  열 수 다시 잼, 다크 테마 어두운 토큰 · 대비
  EVIDENCE: Claude(구현자) 2026-10-24 — 디버그 빌드 + scratchpad `ui1z11\data`(ui1z8 사본 + `seed1z11.py` — 상대 3 · 대화 10 · 안 읽음 · 실행 중 · 오류), ego-browser DOM 판독 + 스크린샷
  (`ego\s11_light100.png` · `s11_dark100.png`). 라이트: 바탕 rgb(245, 243, 238) · 절 3 · 절 `zoom` 1 · 둘째 절 위 선 rgb(227, 222, 213) · 머리 타일 없음 · 이름 14px 500(대비 12.73) · 머리 글 6.17 ·
  카드 rgb(251, 250, 247) · 한 색 테두리 · 둥글기 10px · 그림자 · 윤곽선 없음, 안 읽음 = 테두리 rgb(157, 179, 196) + "새 글 2", 유형 칩 바탕 투명 · in rgb(78, 122, 102) · out rgb(74, 102, 136), 제목 13.5px 500,
  바닥 "planner · 08:07 | 완료"(12px · 5.54) · 상태 완료 rgb(70, 107, 82) · 오류 rgb(138, 58, 46), 말풍선 12px / 18px — 내 글 rgb(236, 232, 224) 11.55 · 상대 흰 + 테두리 14.12 · 에이전트 rgb(233, 239, 233) 8.33 ·
  오류 6.36, "+5개 더 보기", `pv-unread` · 바닥 로봇 0개. 크기 125%(저장값) = 2열 441px · 본문 165, 200% = 1열 901px · 본문 264(이름 · 말풍선 글자 그대로), 100% = 3열 288px · 본문 132.
  다크: 바탕 rgb(26, 25, 23) · 카드 rgb(34, 33, 30) · 이름 대비 13.97 · 머리 7.88 · 바닥 5.98 · 말풍선 7.64 ~ 13.69 · 안 읽음 테두리 rgb(92, 122, 146).

## 개정 기록

- 2026-10-24(구현 중 — 설계만, 평가 항목 · EXPECT 그대로): ① 말풍선 사이 간격 `--sp-2` → `--sp-3`(#1z-8 사용자 지시 '말풍선 사이 여백' 행 — 시안 8px 대신).

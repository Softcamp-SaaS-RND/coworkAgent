# stage1 — 요청 개요 · 탭 토글 · 글자 단추

## 목표

plan Stage 1 — '요청' 화면을 받은 요청 · 보낸 요청 개요(머리 상태별 건수)로 열고, 탭 토글 · 메뉴 다시 누름 = 개요, 상태 바꾸기를 본문 바로 아래 글자 단추로(해석 1 ~ 5).

## 산출 파일

- `crates/app/web/view.js` — `reqSummary` · `reqHead` · `REQ_BTN`.
- `crates/app/web/app.js` — `loadRequests`(개요 · 탭, 거르기 없이 500 받아 화면에서 거름) · `reqBoxEl` · `requestEls`(단추 자리) · `textButton` · 탭 토글 ·
  사이드바 '요청' 다시 누름 · `foldEl` 처음 접힘(`shut` — `open:{열쇠}`) · `state.reqBox` 처음 `null`.
- `crates/app/web/app.css` — `.req-box` · `.req-acts` · `.req-notes` · `button.text-btn(.ok/.no)` · `.ico-check` · `.ico-x`.
- `crates/app/web/view.test.mjs` — `req_overview_summary` · `req_overview_head` · `req_overview_buttons`.
- `crates/app/tests/ux_literals.rs` — 1q PRESENT 11 · ABSENT 2.
- `design/mock.py` — 1p mock 복사 + 요청 데이터(받은 요청 대기 · 수용 · 완료 · 반려, 내 요청 수용 · 대기 · 완료).

## 설계 결정

- 접힘 열쇠 `requests:{box}` · 안 `requests:{box}:{상대}`(해석 3) — 옛 `requests:{상대}` 는 버림.
- 머리 숫자 색은 muted, 대기만 `--run`(목록 대기 태그와 같은 색).
- 단추 줄은 링크 아래 여백을 `calc(-1 * var(--sp-2))` 로 당겨 본문 바로 다음 줄로 보이게.

## 테스트 결과

- RED: `req_overview` — `does not provide an export named 'REQ_BTN'`(# pass 0 # fail 1).
- GREEN: `node --test --test-reporter=tap "crates/app/web/*.test.mjs"` → `# pass 115` `# fail 0`.
- `cargo test -p coworkagent-app --test ux_literals --test css_tokens` → `test result: ok. 3 passed` · `test result: ok. 5 passed`.
- mock 눈 확인(다크, 1440×900, Edge headless): 받은 요청 머리 "받은 요청 5 대기 2 수용 1 완료 1 반려 1" · 상대별 절 · 본문 아래 ✓ 수용 ✕ 반려 · ✓ 완료 · 메모 아래 ·
  보낸 요청 처음 접힘 · JS 오류 없음.

## 다음 단계

stage2 — 내 활동(core · agent · API · 화면 탭).

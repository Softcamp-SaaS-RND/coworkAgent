# stage 1 — 개요 미리보기(시안 B) · 보기 기억 · 글 limit

## 목표

intent 결정 1 ~ 8 — 개요 기본 보기 = 상대별 작은 대화 창, "목록 | 미리보기" 기억, 열 수 = 폭, 미리보기는 마지막 글만.

## 산출 파일

- `crates/core/src/run_opts.rs`(`HOME_VIEW_KEY` · `HOME_VIEWS` · `home_view`) · `crates/app/src/notify.rs`(`ui_home_view` 검사 · 쓰기 · 읽기) ·
  `crates/app/src/chat_api.rs`(`TailQuery` · `list_messages` `limit`).
- `crates/app/web/view.js`(`PREVIEW_MSGS` · `previewWindows` · `previewBubbles`) · `app.js`(보기 분기 · `renderHomePreview` · `previewWindowEl` ·
  `pickPreviewTab` · `setHomeView`) · `index.html`(`.seg` · `#home_preview`) · `app.css`(`.pv-*`).
- 시험 `chat_api.rs` · `ux_api.rs` · `ux_literals.rs` · `view.test.mjs`. 문서 — web-ui · an_app · an_core · 사용자 가이드 · PRODUCT.

## 설계 결정

- 열 수는 CSS `auto-fill`(창 최소 360px)만 — 스크립트가 창 수를 세지 않는다. 창 높이 고정 320px, 격자가 세로 스크롤.
- 창 본문 = `flex-end` + `overflow: hidden` — 최신 글이 아래에 붙고 넘친 옛 글은 위에서 잘린다(스크롤 없음, 누르면 대화).
- 다시 읽기 = 필요한 대화만(사용자 "Q3 반영"): 받은 적 없음 · 요약 변화(`previewKey`) · 변경 신호(`previewStaleOf`) · 개요에 들어옴 · 보기 바꿈(`previewAll`) · 탭 고름 ·
  읽기 실패. 서버 `messages_tail` 이 SQL 에서 6개만. 받은 차례(`seq`)가 앞선 응답은 덮지 않는다. 다시 그리기 서명 = 실제 말풍선(R1-2). 처음(요약 변화만, R1-1 지적)
  → R1 반영(그릴 때마다) → Q3(신호 · 요약 · 들어옴)로 바뀌었다 — 이 PC 에서 마지막이 아닌 글만 바뀌면 개요에 다시 들어올 때 보인다.
- 탭 3개 + "+n개"(사용자 "Q2 반영") — 고른 대화가 넷째 이후면 셋째 칸을 그것으로. "+n개" = `pickTile(상대)` → 그 상대 개요(대화마다 창).
- 탭은 본문 링크 밖(`button role=tab`) — 링크 안에 단추를 넣지 않는다. 다시 그린 뒤 고른 탭에 초점.
- 보기 값은 시작할 때만 서버에서 읽고(접힘 `ui_folded` 와 같은 규칙), 그 뒤는 화면이 바꾼 값이 정본.
- `cargo fmt` 는 쓰지 않는다 — 이 저장소는 rustfmt 기본 폭으로 맞춰져 있지 않아 손대지 않은 파일까지 바뀐다(되돌림).

## 테스트 결과

```text
node --test "crates/app/web/*.test.mjs"   → pass 162 · fail 0
cargo test --workspace(PowerShell)         → test result: ok 80개, 실패 0
실제 화면(ego-browser, 디버그 빌드 · 시험 데이터 폴더 · 상대 4 · 대화 6 시드) → E7 EVIDENCE
```

# stage2 — 웹UI: 복사 아이콘 · 편집 모드

> 근거: [plan](../plan.md) Stage 2. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

말풍선 우상단 복사 아이콘(A1), 입력창 위 "편집" · 편집 막대 · 체크박스 · 연필 · 반영 범위 확인 창(A2 · A3 · A4 · A5 · A6). 대상 gate: E1 ~ E3 · E7.

## 산출 파일

- `crates/app/web/index.html` — `.edit-row`(`#edit_btn` "편집", 입력창 위) · `#edit_bar`("0개 선택" · 안내 · 삭제 · 완료) · `<dialog id="scope_dialog">`
  ("내 창에서만" · "상대 창에도" · 취소).
- `crates/app/web/app.css` — 아이콘 · 체크박스 · 편집 막대 · 확인 창, 작업 막대는 아이콘 왼쪽(`right: 48px`).
- `crates/app/web/view.js` — `messageActions` 에서 복사 뺌 · `editedTag` · `keepPicked` · 내 활동 `sync_failed` 라벨 · 서명에 `edited_at` · `editable`.
- `crates/app/web/app.js` — 머리줄 아이콘(`iconButton` — 고정 SVG 상수만 `innerHTML`) · `copyText` · `setEditMode` · `renderEditBar` · `askScope` ·
  `deletePicked` · `editBody` · `redrawMessages`, 대화를 바꾸면 편집 모드 끔, 본문을 고치는 동안 다시 그리기 멈춤.
- 테스트 — `view.test.mjs`(갱신 1 · 새 3) · `app/tests/ux_literals.rs`(`tidy_literals`).

## 설계 결정

- **확인 창 = 네이티브 `<dialog>` + `form method="dialog"`** — 단추 `value` 가 `returnValue`, Esc 는 취소.
- **복사 아이콘은 머리줄(`.msg-meta`) 끝** — 말풍선 우상단이면서 글 흐름 안이라 겹침 계산이 없다. 마우스를 올릴 때 뜨는 작업 막대만 왼쪽으로 비켰다.
- 고른 글은 새로 고칠 때마다 `keepPicked` 로 거른다 — 다른 창에서 지운 글 · 편집 불가로 바뀐 글이 삭제 요청에 섞이지 않게(서버도 거절한다).
- 브라우저 실화면 확인은 하지 않았다(이 PC 에서 실행 중인 앱 PID 7364 를 건드리지 않음) — E8 · 19 TC.

## 테스트 결과

```text
$ node --test --test-reporter=tap "crates/app/web/*.test.mjs" → # pass 50 / # fail 0
$ node --check crates/app/web/app.js → (오류 없음)
$ cargo test -p coworkagent-app --test ux_literals → test result: ok. 11 passed; 0 failed
$ cargo test --workspace --no-fail-fast → 합 passed 344 · failed 0
$ cargo clippy --workspace --all-targets -- -D warnings → Finished
```

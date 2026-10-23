# stage 1 — 상단 바 · 검색 · 미리보기 · 세부정보

## 목표

plan "설계 · stage 1" — 상단 바 버전(A1), 검색 "나간 대화 포함" 스위치(A2) · 유형 칩 S1(A3), 개요 미리보기 2줄 · 여백(A4), 세부정보 '제목' 줄 삭제(A11), 버전 1.3.4.

## 산출 파일

- core `chat.rs` — `closed_sql`(내가 나감 · 상대가 나감 · 끝난 공유 사본) · `search(conn, q, limit, closed)` · `search_messages`.
- app `chat_api.rs` — `SearchQuery.closed`(기본 거짓). `tests/ux_api.rs` `search_closed_switch`. agent `tests/message_tidy.rs` 호출부.
- web `index.html`(wordmark 버전 칸 · 검색 머리 `#search_opts` + `#search_results` · 세부정보 제목 줄 삭제) · `app.js`(`showStatus` 버전 · `showSearch` · `searchMark` · `runSearch` `&closed=` ·
  말풍선 `.pv-text` · `applyHomeZoom` 줄 수 없앰) · `view.js`(`searchGroups(r, peerNames, convs)` `mark` · `tag` · `previewZoom` → `{scale}`) · `app.css`(버전 · 검색 머리 · 칩 · 스위치 · `.pv-text` · 여백 · `--pv-lines` 없앰).
- 시험: `view.test.mjs` `preview_zoom`(줄 수 없음) · `search_groups`(mark · tag) · 새 `search_groups_closed`, `ux_literals.rs`(#1z-8 행 · "이 PC 에만 저장" 없음 · `--pv-lines` 없음 · #1z-6 줄 수 행 개정).
- 버전 네 크레이트 1.3.4.

## 설계 결정

- 메시지 묶음은 `messages_across` 를 넓히지 않고 검색 전용 `search_messages`(같은 열 · 정렬 · `context` 제외)로 — `messages_across` 호출부 8곳을 건드리지 않는다.
- 닫힘 = 웹UI `convClosed` 와 같은 규칙(끝난 공유 사본 포함 — 적대적 조언). 검색 응답의 글 쪽 대화 정보(`conversation_ref`)에는 공유 정보가 없어, 태그는 화면이 가진 같은 대화(`state.convs`)로 계산한다.
- 스위치 값은 체크박스 그 자체(설정 저장 없음 — 결정 11).
- 미리보기 비침: 줄 자르기를 말풍선(안쪽 여백 있음)에서 안쪽 `.pv-text` 로 옮김 — 잘린 3번째 줄이 아래 여백에 보이던 원인.

## 테스트 결과

- `cargo test -p coworkagent-app --test ux_api search_` → `test result: ok. 3 passed; 0 failed`(새 `search_closed_switch` 포함).
- `node --test --test-reporter=tap "crates/app/web/*.test.mjs"` → `# pass 169` · `# fail 0`.
- `cargo test --workspace` → css_tokens 1건 실패(스위치 `border-radius: 10px` 리터럴) → `var(--radius-pill)` 로 고침 → `cargo test -p coworkagent-app --test css_tokens --test ux_literals` 3 · 5 passed.
- `cargo clippy --workspace --all-targets -- -D warnings` → Finished(경고 0).
- 실제 화면은 stage 3 뒤 E8 에서 한 번에(125 % 미리보기 · 검색 스위치 포함).

## 다음 단계

stage 2 — 설정 틀(8 메뉴 · 한 화면 · 스위치 · 2열 타일 · 디버깅 API · 화면).

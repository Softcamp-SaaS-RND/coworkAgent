# stage 1 — 화면(web) · 토큰 · 버전

## 목표

[plan](../plan.md) stage 1 — 받은 대화 입력창 '질의' · '요청' 단추 숨김, 내가 직접 쓴 말풍선 '나' 얼굴 + '나' 타일 색 옅은 면, 요청 의견 칸 안쪽 여백, 버전 1.4.1.

## 산출 파일

- `crates/app/web/js/logic/conv.js` `asksKind` · `logic/author.js` `ownHuman` · `logic/peer.js` `ownTint` — 순수 판정 함수.
- `crates/app/web/js/conv.js` — `showConversation`(받은 대화면 `.ask-kind` 숨김 · 값 "질의"), `messageEl`(`own own-{색}` class · '나' 레일 얼굴 아바타), `messagesSig`(`me_icon` · `me_color`).
- `crates/app/web/js/composer.js` — `request` = `asksKind(state.current) && ask.value === "request"`.
- `crates/app/web/app.css` — 토큰 `--own-{rail-2 · agent-tile · peer-1 ~ 5}` · `--dark-own-*` · 다크 매핑, `.msg.own .bubble` · `.msg.own-{색}`, `.req-note textarea` 안쪽 여백.
- `crates/{core,net,agent,app}/Cargo.toml` · `Cargo.lock` — 1.4.1.
- 시험: `crates/app/web/view.test.mjs`(own_tint · own_human · asks_kind), `crates/app/tests/ux_literals.rs`(PRESENT 11행 · `token_contrast` 14쌍 → 102).

## 설계 결정

- 옅은 면 값 = 타일 색 12% + 흰색(라이트) · 타일 색 30% + `--dark-surface`(다크, 흑연은 `#71717A` 로 섞음 — `--dark-rail-2` 는 다크 면과 거의 같아 구별이 안 됨).
  본문 글자 대비 라이트 14.06 ~ 15.08 · 다크 9.9 ~ 10.89(`token_contrast`).
- 테두리 = `--border-strong`(구현자 해석). 같은 색 옅은 테두리는 `color-mix` 가 필요한데 macOS 12 Safari 15 에 없어 `ux_literals` `literals_absent`(#1l)가 막는다 —
  색별 선 토큰 14개를 더 만드는 대신 단추 테두리 선 하나로. 고른 글(`picked` — article 바탕 `--accent-soft`) 안에서도 말풍선 가장자리가 보인다.
- 숨은 단추가 남긴 "요청" 값이 받은 대화로 새지 않게 보낼 때도 `asksKind` 로 막는다(화면 + 보내기 두 곳).

## 테스트 결과

```text
cargo test --workspace                      → 전부 ok(첫 실행 literals_absent 1 FAILED — color-mix, 테두리 바꾼 뒤 ux_literals 5 passed · css_tokens 3 passed)
cargo clippy --workspace --all-targets -- -D warnings → Finished
node --test --test-reporter=tap "crates/app/web/*.test.mjs" → # pass 177 / # fail 0 (RED 먼저: asksKind export 없음 SyntaxError)
python tools/check_code_tidy.py             → PASS — 제품 원본 116개 · 최대 968줄 · 이력 표식 0
python tools/check_crates.py                → crates=4 edges=6 violations=0
```

## 다음 단계

[stage 2](stage2.md) — 디버그 빌드 실제 화면(두 테마) · 문서(web-ui · 사용자 가이드 · PRODUCT · README).

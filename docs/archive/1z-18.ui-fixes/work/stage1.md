# stage 1 — 화면(web) · 버전

## 목표

[plan](../plan.md) stage 1 — 요청 의견 칸 폭 · 단추 32px, archive 슬라이드 토글, 즐겨찾기 줄 머리 ★ · 세부정보, 에이전트 응답 줄(타일 · 제목 · 시각 · `[이름]`, 1100px · 좁으면 두 줄), 1.4.3.

## 산출 파일

- `crates/app/web/js/logic/conv.js` — `starDetail(c, peerName)` · `agentTag(c)`.
- `crates/app/web/js/sidebar.js` — `convItem` 넷째 인자 `opts`(`star` · `detail` · `agent`), `renderStarred` · `renderAgentConvs` 가 씀(에이전트 응답 서명에 `state.agents` 이름 · 색).
- `crates/app/web/js/conv.js` `messageAct` — 의견 칸을 `.msg-actions` 안이 아니라 `.msg-body` 에(작업 줄 아래).
- `crates/app/web/index.html` — `#req_archive` = `<label class="switch-line">` + `role="switch"`.
- `crates/app/web/app.css` — `.req-note button` 32px · `.req-acts > .req-note { flex: 1 0 100% }` · `.msg-body > .req-note { width: min(100%, 640px) }`,
  `.room-detail` · 줄 머리 `.star-btn` 가운데, `#agent_convs_list` 최대 너비(항목 1100px) · 제목 줄 `flex-wrap` · `.room-title { flex: 1 1 714px }` · `.room-end { flex: 0 0 272px }` · `.room-agent`.
- `crates/{core,net,agent,app}/Cargo.toml` · `Cargo.lock` — 1.4.3.
- (plan 개정) `crates/app/web/js/activity.js` — `savedRowEls` · `unsaveRow`(북마크 목록 줄 머리 해제), `app.css` `.row-mark.lead`.
- 시험: `crates/app/web/view.test.mjs`(star_detail · agent_tag), `crates/app/tests/ux_literals.rs`(#1y archive literal → 스위치, PRESENT 6행).

## 설계 결정

- 두 줄 전환 = flex 줄바꿈(제목 기본 폭 714px + 고정 폭 끝 묶음 272px + 간격 8 = 항목 1100px 의 제목 줄 994px) — container query 는 macOS 12 Safari 15 에 없다.
  끝 묶음을 고정 폭으로 둬 시각 형식 · 이름 길이와 무관하게 전환점이 항목 1100px 이다(이름은 160px 에서 "…").
- 대화 메시지 작업의 의견 칸은 `.msg-actions`(가로 단추 묶음 · 30px 글자 단추 모양) 안에 붙어 같은 결함이었다 — 붙이는 곳을 `.msg-body` 로 옮겨 일반 단추 모양 · 작업 줄 아래.
- 즐겨찾기 ★ 는 대화 머리글과 같은 `starButton` — 누르면 `toggleStar` → `loadConvs` 로 목록에서 빠진다.

## 테스트 결과

```text
node --test → RED 먼저: "does not provide an export named 'agentTag'" (# fail 1) → # pass 180 / # fail 0
cargo test --workspace → test result 84개 전부 ok
cargo clippy --workspace --all-targets -- -D warnings → Finished
python tools/check_code_tidy.py → PASS — 제품 원본 116개 · 최대 968줄 · 이력 표식 0
python tools/check_crates.py → crates=4 edges=6 violations=0
(plan 개정분) ux_literals RED 먼저(1 FAILED — 북마크 PRESENT 3행) → ok 5 · node # pass 181 / # fail 0
```

## 다음 단계

[stage 2](stage2.md) — 실제 화면 · 문서.

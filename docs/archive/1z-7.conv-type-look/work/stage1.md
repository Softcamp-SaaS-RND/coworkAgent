# stage 1 — 대화 유형 색 · 아이콘 + 내 에이전트 상세 정리

## 목표

intent 결정 2 ~ 8 — 대화 유형(`in` 청록 · 로봇 / `out` 파랑 · 사람 / `shared` 회색 점선)을 대화 머리 · 사이드바 줄 · 개요 카드 · 목록 줄에 같은 규칙으로,
설정 '내 에이전트' 상세 한 격자 · 아이콘 색(스키마 26) · 닫기, 버전 1.3.3.

## 산출 파일

- `crates/core/src/agents.rs`(`ICON_COLORS` · `Agent.color` · `set_color`) · `crates/core/src/db.rs`(스키마 26) · `crates/app/src/chat_api.rs`(`AgentPatch.color` 검사 · 저장 · JSON).
- `crates/app/web/view.js`(`AGENT_COLORS` · `agentColor` · `convKindIcon` · `KIND_NAMES` · `PARENT_HINT` · `SHELL_HINT` · `agentPatch` 색) · `app.js`(`kindChip` · `agentTile` ·
  `renderConvHead` · `convItem` · 개요 목록 줄 · `previewCardEl` · `renderAgentList` · `renderAgentsView`) · `index.html`(`#conv_kind`) · `app.css`(유형 토큰 · 규칙 · 상세).
- 시험 `core/tests/paths.rs`(`schema26_agent_color` · 버전 26) · `app/tests/chat_api.rs`(색) · `app/tests/ux_literals.rs`(#1z-7 줄 · 개정 두 줄 · 없음 한 줄) · `web/view.test.mjs`
  (`agent_color` · `conv_kind_icon`) · `agent/tests/*.rs` 의 `Agent { … }` 에 `color`.
- 문서 — web-ui · an_app · an_core · 사용자 가이드 · PRODUCT · README(1.3.3). 버전 네 크레이트 1.3.3.

## 설계 결정

- 색은 둘레의 `data-kind`(= 대화 `dir`)가 `--kind` · `--kind-soft` 를 정하고 줄 · 카드 · 머리 · 칩이 그 값을 쓴다 — 한 규칙이 네 곳에 같은 색을 준다. `:root` 별칭
  `--conv-in` · `--conv-out` · `--conv-shared` 가 기존 `--agent` · `--accent` · `--faint` 를 가리켜 다크 테마는 따라간다(새 색 값 없음 — 시안 A).
- 칩 아이콘 색 = `--surface`(라이트 흰 · 다크 짙은 회색) — 다크의 밝은 청록 · 파랑 면 위에서도 대비(기존 `.msg .avatar.agent-av` 와 같은 방식).
- 안 읽음 카드는 바깥 `outline`, 마우스는 그림자 — 지금의 강조 테두리는 문의(파랑) 테두리와 겹쳐 유형을 가린다. 고른 사이드바 줄은 유형 옅은 바탕 + 안쪽 테두리.
- 에이전트 색은 토큰 열쇠(타일 색 클래스 이름 그대로)로 저장 — 서버가 목록 밖을 400, 화면은 모르는 값을 기본으로. 색 변경은 실행에 닿지 않아 `agents::update`(세대 · 세션
  비움)와 따로 `set_color`. 상대 PC 에 보내는 에이전트 목록(`names`)엔 없다.
- 닫기 = 연필 단추의 `click()`(같은 접기 · `state.open` · `aria-expanded`) + `focus()`(숨은 칸에 초점이 남지 않게 — #05-3 R1 과 같은 이유).

## 테스트 결과

```text
cargo test --workspace                        → 82 묶음 · 499 passed · 0 failed
cargo clippy --workspace --all-targets -- -D warnings → Finished(경고 0)
node --test "crates/app/web/*.test.mjs"       → pass 168 · fail 0
cargo test -p coworkagent-app --test ux_literals --test css_tokens → ok 3 · ok 5 (점선 순서 수정 뒤)
실제 화면(ego-browser, 디버그 빌드 · 시험 데이터 폴더 · 대화 5 시드) → plan E7 EVIDENCE — 개요 목록의 공유 줄 점선 결함 1건 발견 · 수정
```

## 다음 단계

- 적대적 평가(`codex-peer-review`, 라운드 상한 2) → eval · report · 종결.

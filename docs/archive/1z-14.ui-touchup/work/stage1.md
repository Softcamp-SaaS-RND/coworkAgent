# stage 1 — 구현 · 시험 · 문서 · 버전

## 목표

plan stage 1 — 레일 사람 · 사람 그룹 그림(결정 1), 개요 미리보기 카드 테두리 = 유형 색(결정 2 · 3), 사이드바 대화 목록 내 대화 줄 아이콘 없음(결정 4), 버전 1.4.0(결정 5 · 7), 문서.

## 산출 파일

- `crates/app/web/js/core/widgets.js` — `PERSON_SVG` · `GROUP_SVG`(`d86a728` 그림 그대로) · `railFace(icon, group = false)`, `tileFace` 주석(줄 타일 얼굴).
- `crates/app/web/js/rail.js` — 나 · 상대 · 그룹 타일 = `railFace`. `crates/app/web/js/tiles.js` — 타일 편집 미리보기 = `railFace`(그룹이면 그룹 그림).
- `crates/app/web/js/sidebar.js` — `convItem(…, sideList = false)`, `renderConvList` 만 참 → 내 대화(`out`) 줄 유형 아이콘 없음.
- `crates/app/web/app.css` — 토큰 `--card-agent`(= `--agent`) · `--card-mine`(#46607E / 다크 #8AA4C4) · `--card-watch`(= `--run`), `.pv-card[data-kind]` 테두리 셋 ·
  `.pv-card.unseen` 테두리 2px(강조색 덮어쓰기 지움), `.rail-btn .tile-shape svg` 22px · 작은 타일 16px, 레일 주석.
- `crates/app/tests/ux_literals.rs` — 바뀜 4 · 새 PRESENT 7 · 지운 ABSENT 4(사람 그림 셋 · `data-kind="shared"`) · 새 ABSENT 2 · `token_contrast` 82 경우.
- 버전 1.4.0 — `crates/{core,net,agent,app}/Cargo.toml` · `Cargo.lock`, README 1.3.7 줄 → 1.4.0(결정 7).
- [web-ui](../../../manual/dev/web-ui.md)(레일 · 카드 · 사이드바 줄 · 토큰 표) · [사용자 가이드](../../../manual/user/user-guide.md)(레일 · 유형 표시 · 이미지 지우기) · PRODUCT(#1z-14 줄만 스테이징) · README.

## 설계 결정

- 레일 그림은 #1z-13 전 그림 그대로, 바탕만 Graphite 무채(`.tile-shape` = `--avatar-bg` · '나' = `--rail-2`) — 다섯 색은 돌리지 않음(결정 1).
- 카드 테두리 규칙은 `.pv-card:hover` 뒤 — 올려도 유형 색이 남는다. 안 읽음 2px 는 테두리 폭만 바꿔 1px 안쪽으로 자리가 줄어든다(같은 카드 안).
- 사이드바 조건은 `renderConvList` 한 곳 — 즐겨찾기 · 에이전트 응답 화면(유형이 섞임)은 아이콘 그대로.

## 테스트 결과

- RED — `cargo test -p coworkagent-app --test ux_literals` → `token_contrast` · `literals_absent` · `literals_present` FAILED.
- GREEN — `--test ux_literals --test css_tokens` → ok 3 · ok 5, node `# pass 172` · `# fail 0`, `check_code_tidy.py` PASS(116 · 960 · 0), `check_readme.py` readme ok.

## 다음 단계

stage 2 — 실제 화면(레일 · 미리보기 · 사이드바 × 두 테마) · README 사진 셋.

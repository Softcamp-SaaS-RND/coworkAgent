# stage 1 — 토큰 · CSS · 마크업 · 문서

## 목표

plan "설계 · stage 1" — 개요 미리보기를 시안 C "편안함 · 종이 톤" 으로(전용 토큰 `--pv-*` · 다크 `--dark-pv-*`, 절 · 머리 · 카드 · 말풍선 · "+n개 더 보기"), 절 확대 뺌, 매뉴얼 · PRODUCT.

## 산출 파일

- `crates/app/web/app.css` — 첫 `:root` 에 `--pv-*` 색 28 · 크기 · 간격 토큰과 `--dark-pv-*` 28, 다크 대입 28줄, `--pv-zoom` 지움. `.pv-*` 블록 교체, `.home-kind, .pv-kind` → `.home-kind`.
- `crates/app/web/app.js` — `previewSectionEl`(상대 타일 뺌 · `.pv-pres` · "· 안 읽음 n" · `stat pv-wait`), `previewCardEl`(머리 "새 글 n" · 바닥 `.pv-who` + `.pv-state`), "+n개 더 보기".
- `crates/app/tests/ux_literals.rs` — 바뀐 행(카드 폭 · 말풍선 · 이름 · 본문 높이 · 안 읽음 · 카드 테두리) · 새 PRESENT 12 · 새 ABSENT 4(`--pv-zoom` · `pv-unread` · 머리 `rowTile` · 윤곽선).
- 문서 — [web-ui](../../../manual/dev/web-ui.md)(미리보기 절 · 카드 · 말풍선 · 유형 색 닿는 곳) · [사용자 가이드](../../../manual/user/user-guide.md)(개요 미리보기 줄 · 유형 색) · `PRODUCT.md`(#1z-11 줄).

## 설계 결정

- 말풍선 사이 = `--sp-3`(12px) 그대로 — 시안 C 는 8px 이지만 `ux_literals` 의 #1z-8 행 "말풍선 사이 여백"(사용자 지시 "간격 넓게")이 걸렸다. 사용자 지시를 시안 수치보다 앞에 두고
  plan 개정 기록 ① 에 적음(본문 132px 에 2줄 말풍선 둘 + 간격이 들어감).
- 접속 점 색 = 받기 가능 초록 · 집중 황토 · 끊김 붉은 갈색(`.pv-head .st-*` 가 `--st` 를 `--pv-*` 로) — 앱의 `presence()` 글 그대로(시안의 "접속 중" 등은 예시).
- 상태 = `stateText` 글 그대로, 점 · 글 색만 시안 짝(완료 · 실행 중 · 오류 · 답 미전달, 그 밖 회색).
- 다크 = 같은 구조 + 어두운 종이 톤(`--dark-pv-*`) — 실측 대비 5.98 이상.
- 첫 절 위 선은 없음(`.pv-sec + .pv-sec`) — 개요 머리 아래 선이 이미 있다(겹선 방지).

## 테스트 결과

- `cargo test -p coworkagent-app --test ux_literals --test css_tokens` → `ok. 3 passed` · `ok. 5 passed`.
- `node --test --test-reporter=tap crates/app/web/view.test.mjs` → `# pass 172` · `# fail 0`.
- 실제 화면(ego-browser) — plan E6 EVIDENCE.

## 다음 단계

적대적 평가(codex-peer-review 라운드 1/2) → 종결.

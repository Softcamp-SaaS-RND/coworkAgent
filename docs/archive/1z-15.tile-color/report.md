# report — 1z-15.tile-color

종결: 2026-10-26 · 브랜치 `task/1z-15-tile-color`(원 작업 폴더 — worktree 없음) → main(로컬 반영, push 는 사용자 확인 뒤).

## 결과

레일 타일 편집에 '아이콘 색'을 더하고 그룹 그림을 세 사람으로 바꿨다 — 버전 1.4.0 그대로, DB 스키마 28 → 29.

- stage 1 — 저장: `peers.color` · `groups.color`(스키마 29 — 있던 행 · 새 행 = 여섯 중 무작위 한 번 저장, 다시 hello 해도 그대로), '나' = 설정 `me_color`(빈 값 = 흑연),
  `PATCH /api/peers/{id}` · `/api/groups/{id}` `color` · `PUT /api/settings` `me_color`(여섯 밖 400).
- stage 2 — 화면: 타일 편집 '아이콘 색' 단추(에이전트와 같은 6색 · '나' 는 맨 앞 흑연 하나 더), `tileTint` 로 레일 · 미리보기 · 줄 타일(개요 · 연결 · 그룹설정 · 검색) · 말풍선 ·
  요청 의견 · 주고받은 메시지 · 프로필 얼굴에 같은 색(흰 글자), 그룹 그림 = 앞 사람 가운데 + 뒤 좌우 옅은 둘.
- stage 3 — 실제 화면 두 테마 · 저장 흐름 확인(오류 0), README 사진 셋 다시, 매뉴얼 · PRODUCT · README.
- 평가 — check_gates 8/8, codex 라운드 1/2 지적 2(R1-1 그룹설정 목록 색 유실 · R1-2 상대 복합 PATCH 부분 저장 — 둘 다 사실, 고침) · 라운드 2/2 지적 0. 미조치 없음.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 색 골라 저장 · 다시 열기 · '나' 흑연 되돌림 | 충족 | E7 저장 흐름(상대 1 peer-3 → peer-1 · '나' peer-2 → 흑연 — API `me_color` null), `ux_api` `peer_and_me_tile_color` · `groups_api` 이름 바꾸기 시험 |
| A2 새 · 있던 상대 · 그룹 = 여섯 중 한 색 저장 | 충족 | E5 `schema29_tile_color`(있던 행 · 새 30 행 · 다시 hello), E7 시험 데이터 29 로 올림(있던 다섯 행 색) |
| A3 레일 · 머리글자 네모 전부 색 · 대비 | 충족 | E3 `tile_tint` · 그룹설정 목록 색, E4 `token_contrast` 88(`--on-dark` × 여섯 4.5:1 — 두 테마 같은 값), E7 레일 · 연결 · 말풍선 · 주고받은 메시지 · 프로필 |
| A4 그룹 그림 세 사람 | 충족 | E4 그림 행, E7 그룹 타일 원 3 |
| A5 동작 · 문구 · 검증 · 문서 · 1.4.0 | 충족 | E1 · E2 · E6 · E8, 새 문구는 '아이콘 색' 하나(접근 이름 '아이콘 색 {열쇠}' 는 에이전트와 같은 꼴), 매뉴얼 · README 사진 |

- 편차 · 한계:
  - 스키마 29 — 1.4.0 으로 한 번 연 DB 는 이전 실행 파일(1.3.x)이 열지 않는다(올리기 전 `{db}.v28.bak` 사본으로 되돌림 — 기존 규칙). README 1.4.0 줄에 적었다.
  - 무작위 색은 이 PC 표시 — 같은 상대가 두 PC 에서 다른 색일 수 있다(상대에 보내지 않음, 결정 9).
  - 평가 중 다른 `target\debug\coworkAgent.exe`(11:50 시작 — 이 세션이 띄운 것 아님)가 기본 target 을 잡아 셋째 실행부터 `CARGO_TARGET_DIR=target\t15` 로 측정했다.
  - `cargo test --workspace` 간헐 실패 — `os error 10055`(WSAENOBUFS, 로컬 API 연결)로 `ux_api` `home_zoom_setting_persists`(이 Task) · `cli_tx`(#1z-14) 를 잡았다. 단독 재실행은 통과 —
    환경 문제로 본다(추론). #1z-14 report-3 과 같은 건.
  - 미실행: macOS · 두 PC 실연동 · 화면 낭독기 · 서명 빌드 · 옛 스키마(16 이하) 실제 사용자 DB(시험 `store.rs` · `paths.rs` 는 돎).
- 구현자 해석(사용자 미확인):
  - 결정 6 — '나' 색 단추 맨 앞 '흑연'(되돌릴 길).
  - 결정 9 — 색 저장 자리(상대 · 그룹 표 열 · '나' 설정), 무작위 = SQL `abs(random()) % 6`.
  - '나' 의 색은 고르기 전엔 레일만 흑연 · 그 밖 머리글자 네모(프로필 · 주고받은 메시지 '나')는 무채 그대로, 고르면 모두 그 색.
  - plan 승인 질문 없이 구현 — 사용자 지시 + 질문 답 U1 ~ U6 이 설계를 정함.

## 사용자 결정 필요

| 인덱스 | 내용 | 곳 |
|---|---|---|
| report-1 | 1.4.0 릴리스(#1z-13 · #1z-14 · #1z-15 함께) — origin push(squash) · 서명 · gh release | — |
| report-2 | '나' 기본 흑연일 때 프로필 · 주고받은 메시지 '나' 머리글자는 무채(레일만 흑연) — 흑연으로 맞출지 | [intent](intent.md) 결정 4 |
| report-3 | 시험 간헐 실패(`os error 10055`) 별도 Task 여부(#1z-14 report-3 과 같은 건) | [eval](eval.md) 측정 기록 |

## 산출

- core · net · app: `crates/core/src/{db,run_opts,peers,groups}.rs` · `crates/net/src/node.rs`(`set_peer_fields` — 한 트랜잭션) · `crates/app/src/{peers_api,groups_api,notify}.rs`.
- web: `js/logic/peer.js` `tileTint` · `js/logic/rail.js` · `js/core/widgets.js` · `js/tiles.js` · `js/rail.js` · `js/home.js` · `js/peers.js` · `js/search.js` · `js/conv.js` · `js/shell.js` ·
  `js/settings/{conn,groups,prefs}.js` · `index.html` · `app.css`.
- 시험: `crates/core/tests/paths.rs`(`schema29_tile_color`) · `store.rs`(열 수 — 개정 ①) · `crates/app/tests/{ux_api,groups_api,ux_literals}.rs` · `view.test.mjs`.
- 문서: [an_core](../../manual/dev/an_core.md) · [web-ui](../../manual/dev/web-ui.md) · [사용자 가이드](../../manual/user/user-guide.md) · PRODUCT(#1z-15 줄) · README(1.4.0 줄 · 사진 셋) · masterPlan S4 개정.

# stage 3 — 대화 종류 표시(intent U8)

## 목표

plan "설계 · stage 3" — 사람 / 로봇 아이콘을 종류 색 면에 채운 칩과 종류 색 테두리 · 공유 점선 · 대화 머리 종류 선을 대신할 표시를 실제 앱 사진으로 비교하고,
사용자가 고른 표시로 바꾼다. plan 개정 ② · ⑥ 으로 넘어온 대화 줄 카드 · 사이드바 바탕 덮어쓰기 · `.room-av` 도 함께 다룬다.

## 후보 비교(1단계)

- 디버그 빌드 + scratchpad `ui1z12\data` 에 ego-browser 로 CSS · DOM 만 얹었다(제품 코드 그대로). 그룹 '결제' 에 대화 넷을 더해(시험 데이터) 종류가 섞인 사이드바를 만들었다.
- 찍은 곳 다섯 — 사이드바(그룹) · 대화 머리(문의 · 응답) · 개요 목록 · 개요 미리보기 · 검색. 공유 대화는 데이터에 없어 줄 하나를 복제한 '공유 예시(표시만)' 로 보였다.
- 아티팩트 https://claude.ai/artifact/Mueh8o18aq3n6EvLP3Xxvk — (가) 선 아이콘 · (나) 글자 · (다) 상대 머리글자 + stage 2 결과 사진 넷.
- 비교 중 알게 된 것: 사이드바는 절 머리('내 대화' · '에이전트 대화' · '공유 대화')가 이미 종류를 가른다. 종류 표시가 정보로 쓸모 있는 곳은 섞인 목록(개요 · 미리보기 · 검색)과 대화 머리다.
  이것을 '(가) + 사이드바 줄은 표시 없음' 선택지로 함께 물었다.
- 사용자 답(2026-10-25): **(가) 선 아이콘 · 무채** → intent 결정 15 · plan 개정 ⑧.

## 산출 파일

- `crates/app/web/js/logic/conv.js` — `convKindIcon` = 응답 `bot` · 문의 `arrow-up` · 공유 `eye`, `KIND_NAMES.shared` = "공유 대화".
- `crates/app/web/js/sidebar.js` — 대화 줄 유형 칩 class `room-av agent-av` → `room-kind`.
- 주석 맞춤 — `core/widgets.js` `kindChip` · `sidebar.js` · `search.js` `searchMark` · `logic/activity.js` `tone`(stage 1 에서 종류 색을 지운 뒤 남은 설명).
- `crates/app/web/app.css`
  - `.kind-chip` = `--faint` 글자만(색 면 없음), 아이콘 크기 — 사이드바 · 개요 목록 16px, 대화 머리 18px, 미리보기 15px, 검색 16px.
  - 지움 — `--conv-in` · `--conv-out` · `--conv-shared`, `[data-kind]` 색 변수 셋, 종류 테두리(사이드바 줄 · 개요 목록 줄), 고른 줄 종류 바탕 · 안쪽 테두리,
    공유 점선(줄 · 카드 · 머리 · 검색 칩), 대화 머리 종류 2px 선, `.conv-kind` · `.home-kind` 상자, 미리보기 아이콘 종류 색, `--answer-bg`(쓰는 곳이 사라짐 — 라이트 · 다크 · 대입 줄).
  - 대화 줄 납작하게(개정 ②) — 흰 카드(바탕 · 테두리 · 8px 둥글기) · 사이드바 바탕 덮어쓰기(`--subtle`) · 줄 간격 덮어쓰기 · 고른 줄 안쪽 강조 테두리 지움.
    남는 것 = 기본 규칙(`--sidebar` 바탕 · 4px 둥글기 · 올리면 `--surface-2` · 고른 줄 옅은 청록 바탕 + 청록 글자 — P2).
  - `.room-av` · `.room-av.agent-av` 지움(개정 ⑥ — 종류 칩에만 쓰였다).
- `crates/app/tests/ux_literals.rs` — 바뀐 행 2 · 지운 행 8 · 새 PRESENT 2 · 새 ABSENT 7(plan 개정 ⑧).
- `crates/app/web/view.test.mjs` — `conv_kind_icon` 기대값(응답 · 문의 · 공유 · 모르는 유형).
- [intent](../intent.md) — 결정 15, 미해소 없음. [plan](../plan.md) — 개정 ⑧.

## 설계 결정

- 고른 줄 안의 종류 아이콘도 `--faint` 그대로 — 글자만 청록(P2), 아이콘은 상태 · 선택 색을 받지 않는다(결정 5).
- 줄 · 카드 · 머리의 `data-kind` 속성은 둔다 — 개요 목록 줄 배치 규칙(`.rows li[data-kind] > a`)이 쓰고, `ux_literals` 행(#1z-7)이 지킨다.
- 대화 머리 `#conv_kind` 는 공유 대화에서도 보인다(눈) — 지금까지는 공유에 아이콘이 없어 숨었다.

## 테스트 결과

- RED — 시험을 먼저 고친 뒤 `cargo test -p coworkagent-app --test ux_literals --test css_tokens` → `literals_present` · `literals_absent` FAILED.
  node `conv_kind_icon` not ok(# fail 1).
- GREEN — 같은 명령 → `ok. 3 passed` · `ok. 5 passed`(옛 `convKindIcon` PRESENT 행 하나를 더 찾아 지운 뒤). node `# pass 172` · `# fail 0`.
- `cargo test --workspace --no-fail-fast` → 84 묶음 · 521 passed · 1 failed. 실패 1 = an-agent `conv_leave::delete_while_waiting_leaves_no_rows`.
  - stage 2 에서 본 것과 같은 간헐 실패다(left 1 · right 0 — stage2-3, 사용자 답 "원장 기록 · 다음 Task 에서").
  - 이 단계는 웹 원본만 바꿨다.
- `cargo clippy --workspace --all-targets -- -D warnings` → `Finished`(경고 0). `check_code_tidy.py` → PASS(116 · 960 · 0).
- `python {TASK}/design/taste.py` → `taste files=50 checks=9 violations=0`. `labels.py c3568f5` → `labels n=280 missing=0 added=0`.
- 실제 화면 — 디버그 빌드 + scratchpad `ui1z12\data`, ego-browser 1440 × 900, 7 화면 × 라이트 · 다크.
  - 찍은 화면: 그룹 대화 · 응답 대화 머리 · 개요 목록 · 미리보기 · 에이전트 응답 · 즐겨찾기 · 검색.
  - 종류 아이콘: 사이드바 · 개요 · 미리보기 · 검색 · 머리 모두 `--faint`(라이트 rgb(107, 107, 115) · 다크 rgb(142, 142, 151)), 바탕 투명.
  - 점선 0, 대화 줄 높이 38px · 바탕 투명 · 테두리 0, 사이드바 바탕 rgb(246, 246, 247)(다크 rgb(17, 17, 19)).
  - 고른 줄 rgb(231, 240, 239) + 그림자 없음 + 글자 rgb(47, 111, 107)(다크 rgb(26, 43, 41) · rgb(111, 179, 172)).
  - 대화 머리 아래 1px `--border`, 개요 목록 줄 = 아래 선만 · 둥글기 0, 페이지 오류 0.

## 다음 단계

stage 4 — 실제 화면 23 × 라이트 · 다크 사진(E7) · stage2-2(타일 편집 이름 칸 넘침) 고침 · 문서(web-ui · 사용자 가이드 · PRODUCT) · 버전 1.3.7.

# stage1 — 목록 나눔 · 머리 · 말풍선 음영 · 링크 색 · 레일 우클릭

> 근거: [plan](../plan.md) Stage 1(해석 1 ~ 7) + intent 개정 범위 3(레일 우클릭).

## 목표

사이드바 대화방 목록을 내 대화 · agent 대화로 나누고(시안 Ⓐ 머리), 말풍선 음영을 Ⓐ 단계로 낮추고(내 글 링크 = `--sent`), 레일 어디서든 브라우저 우클릭 메뉴를
막는다. 대상 gate: E1 ~ E6(관람자 gate 는 plan 개정 뒤).

## 산출 파일

- `crates/app/web/view.js` — `CONV_GROUPS`(`{dir: "out", title: "내 대화"}` · `{dir: "in", title: "agent 대화"}`) · `splitByDir(rows)` · `splitKey(section, dir)`.
- `crates/app/web/app.js` — `renderConvList` 가 절마다 `splitByDir` → 묶음 둘 이상이면 `.dir-head`(제목 · 개수 `<b>`) + `ul`, 하나면 `ul` 하나. 행 `data-section` =
  `splitKey(절, dir)`. `#rail` `contextmenu` → `preventDefault`(타일 위는 기존 타일 메뉴).
- `crates/app/web/app.css` — `.dir-head`(`--fs-xs` · `--muted` · 양끝), 말풍선: 내 글 = `--surface` + `--border` + 오른쪽 3px `--accent` 띠 + `::before` `--accent`
  0.14, 링크 `--sent` / 상대 글 층 0.12 → 0.07. 흰 글자 전제 덮어쓰기(blockquote · code · pre · table · `.msg-icon` · 흰 호버) 삭제.
- 시험 — node `conv_split_rows` · `conv_split_scopes` · `conv_split_drop` · `conv_split_starred`, `ux_literals` `chat_list_split_literals` · `bubble_contrast`(36 경우) ·
  `icons_settings_literals` 옛 말풍선 literal 두 줄을 새 값으로.

## 설계 결정 · plan 편차

- 묶음 정의를 배열 `CONV_GROUPS` 로 — 관람자 초대(intent 개정)의 세 번째 묶음 "공유 대화" 를 같은 길로 더한다.
- `.dir-head` 여백은 기존 `.side-title` 처럼 px 리터럴(글자 크기 · 색만 토큰).

## 테스트 결과

```text
node --test --test-reporter=tap "crates/app/web/*.test.mjs"  → # pass 108 / # fail 0
node … --test-name-pattern="^conv_split" view.test.mjs       → # pass 4 / # fail 0
cargo test -p coworkagent-app --test ux_literals            → test result: ok. 22 passed; 0 failed
```

## 다음 단계

- 관람자 초대 설계(plan 개정 — 통신 · 저장 · API · 화면) 사용자 승인 → 구현.

# stage 2 — 머리글자 타일 · 아바타

## 목표

plan "설계 · stage 2" — 사람 아이콘 아바타(사람 · 사람 + 그림자 SVG, 사람 선 아이콘 원)를 이름 머리글자 네모로(결정 2 · 12), 상대 · 그룹 타일 다섯 색을 지운다(P4).
에이전트 아바타 = 무채 네모 + 로봇(P1), 내 에이전트 아이콘 색(사용자가 고른 6색)은 그대로.

## 산출 파일

- `crates/app/web/js/logic/peer.js` — `initialOf(name)`(앞뒤 공백을 뺀 첫 글자 묶음 · 로마자 대문자 · 빈 이름 = 빈 글자), `colorOf` 지움.
- `crates/app/web/js/core/widgets.js` — `tileFace(icon, name)` = 그림 또는 `tile-shape` 머리글자, `rowTile(cls, icon, name)`, `PERSON_SVG` · `GROUP_SVG` · `tileColor` 지움.
- 부르는 곳
  - `rail.js` — 나 · 상대 = 표시 이름, 그룹 = 이름, 타일 색 class · `groupColor` 지움.
  - `tiles.js` — 편집 미리보기 = 편집 칸 이름(비면 원래 이름), 색 class 지움(나 = `me` 만).
  - `peers.js` — 절 머리 타일 = 상대 이름, 줄 아바타 = 사람 머리글자 · 에이전트 로봇.
  - `home.js` · `settings/conn.js` · `settings/groups.js` — 줄 타일 = 상대 표시 이름 · 그룹 이름.
  - `search.js` — 사람 = 표시 이름, 상대 에이전트 = 무채 네모 + 로봇.
  - `shell.js` `renderAvatar` — 그림이 없으면 머리글자.
  - `conv.js` — 사람 아바타 = `initialOf`, 색 class · `data-tone` 지움.
- 주석 맞춤 — `logic/message.js` `avatarKind` · `logic/agent.js` `AGENT_COLORS` · `rail.js` · `home.js` · `search.js` · `peers.js` · `shell.js` · `conv.js` · `app.css` 절 머리 넷.
- `crates/app/web/app.css`
  - 새 규칙 — `.tile-shape`(무채 네모, 담는 쪽 둥글기를 따름) · `.rail-btn.me .tile-shape`(흑연 + 흰 글자).
  - 둥글기 `--radius-sm` — `.rail-btn` · `.pa` · `.row-tile` · `.avatar` · `.pm-av`(원 → 네모).
  - `.avatar` = 24px 무채 네모(에이전트도 — P1), 덮던 규칙 셋(`.msg.agent .avatar` · `.msg .avatar` · `.msg .avatar.agent-av`) 지움.
  - `.pm-av` = 28px 무채 네모, 종류 색 면 셋(상대 · 나 = 옅은 청록 · 에이전트 = 아이콘 색) 지움.
  - 프로필 연결 끊김 = 머리글자 `--faint`(사람 그림 흐림 대신).
  - 쓰는 곳이 사라진 SVG 규칙 지움 — `.pa.small svg` · 작은 레일 타일 svg · 절 머리 타일 svg · 프로필 svg · `.pa .tile-shape` · `.rail-btn .tile-shape`(새 `.tile-shape` 가 대신).
- `crates/app/tests/ux_literals.rs`
  - 바뀐 행 4 — 레일 그룹 · 주고받은 메시지 아바타 근거 · 개요 머리 · 미리보기 머리 ABSENT.
  - 지운 행 4 — `data-tone` PRESENT 1, #1j 글자 타일 없음 ABSENT 3(결정 12 가 뒤집음).
  - 새 PRESENT 11 · 새 ABSENT 7.
- `crates/app/web/view.test.mjs` — `color_of` → `initial_of`(11 경우).
- `docs/tasks/1z-13.design-taste/design/taste.py` — `person-icon-avatar` 에 대화 줄 종류 칩의 사람 아이콘을 더함(개정 ⑦).
- [plan](../plan.md) — 개정 ⑤ ~ ⑦.

## 설계 결정

- 첫 글자 = 글자 묶음(`Intl.Segmenter`, 개정 ⑤). 서로게이트 쌍에 더해 결합 문자 · 풀어 쓴 한글(NFD)도 한 글자다.
- 머리글자 색 = 무채(`--avatar-bg` · `--avatar-text`). 레일 '나' 만 흑연(`--rail-2` · `--on-dark`).
  - 시안은 '나' 글자가 `--on-primary` 다. 다크에서는 흑연 위 흑연이 되므로 plan 대로 `--on-dark` 를 쓴다.
- 둥글기 — `.tile-shape` 는 `border-radius: inherit`, 담는 쪽(`.rail-btn` · `.pa`)을 `--radius-sm` 으로. 그림 타일 · 머리글자 타일 · 고른 윤곽선이 같은 모서리가 된다.
- 고른 레일 타일 = 2px `--accent` 윤곽선 그대로(plan). 시안은 흑연 채움이다.
- 상대 에이전트 검색 칩 = `pa small` 안에 `tile-shape` + 로봇. `.pa` 에 바탕을 주면 앞 줄의 `.peer-N` 을 덮어(같은 특이도) 내 에이전트 아이콘 색이 사라진다.
- 주고받은 메시지 아바타(`.pm-av`)는 intent 결정 2 의 '같은 사람 아이콘 원' 에 든다.
  - 사람 = 머리글자 네모, 에이전트 = 로봇 네모(P1), 색 면은 지운다.
  - `role="img"` · 읽는 이름("사람" · "에이전트")은 그대로.
- `conv.js` CLI 글 아바타는 지금처럼 이름의 마지막 `/` 뒤를 쓴다(`상대/CLI` → "C"). 자르기만 `initialOf` 로 바꿨다.
- `.room-av` 는 stage 3 에서 다룬다(개정 ⑥).

## 테스트 결과

- RED — 시험을 먼저 고친 뒤 `cargo test -p coworkagent-app --test ux_literals --test css_tokens` 를 돌렸다.
  - `literals_present` · `literals_absent` FAILED.
  - node `view.test.mjs` 모듈 오류(`initialOf` 없음).
- GREEN — 같은 명령 → `ok. 3 passed` · `ok. 5 passed`. `node --test --test-reporter=tap "crates/app/web/*.test.mjs"` → `# pass 172` · `# fail 0`.
- `cargo test --workspace` — 첫 실행은 an-agent `conv_leave::delete_while_waiting_leaves_no_rows` 1 실패, `--no-fail-fast` 로 다시 돌려 84 묶음 · 522 passed · 0 failed.
  - 실패 문구: "지운 대화의 글은 실행하지 않는다 left: 1 right: 0".
  - 이 단계는 웹 원본만 바꿨다. 단독 실행 5회는 모두 통과했다.
  - 원인 추정(확인 필요): 첫 대화 글이 `running` 이 된 뒤, 실행기 대역이 입력을 기록하기 전에 시험이 `runs` 를 읽어 0 이 됐다.
    끝의 1 은 첫 대화의 실행이고 지운 대화는 실행되지 않았으므로, 제품 동작이 아니라 시험의 경합으로 본다.
- `cargo clippy --workspace --all-targets -- -D warnings` → `Finished`(경고 0).
- `check_code_tidy.py` → `PASS — 제품 원본 116개 · 최대 960줄 · 이력 표식 0`. `check_crates.py` → `crates=4 edges=6 violations=0`.
- `python {TASK}/design/taste.py` → `taste files=50 checks=9 violations=1` — 남은 1 은 개정 ⑦ 종류 칩(stage 3).
- `labels.py c3568f5` → `labels n=280 missing=0 added=0`.
- 실제 화면 — 디버그 빌드 + scratchpad `ui1z12\data`, ego-browser 1440 × 900, 10 화면 × 라이트 · 다크.
  - 찍은 화면: 대화 · 개요 목록 · 미리보기 · 주고받은 메시지 · 연결 · 그룹설정 · 등록된 에이전트 · 타일 편집 · 검색 · 작은 타일.
  - 사람 SVG 0, 빈 머리글자 0, 페이지 오류 0.
  - 레일 타일 40 / 24px · 둥글기 4px. '나' = rgb(39, 39, 42) + 흰 글자(다크 rgb(38, 38, 42)).
  - 머리글자 rgb(228, 228, 231) / rgb(63, 63, 70)(다크 rgb(46, 46, 51) / rgb(212, 212, 216)).
  - 에이전트 아바타 24px 무채, 주고받은 메시지 아바타 무채(나 · 에이전트), 내 에이전트 아이콘 색 그대로 rgb(47, 111, 107), 프로필 얼굴 "나".

## 발견(이 단계 밖 — 사용자 결정)

- stage2-1 접힘 저장 실패(동작 결함, #1q · 0.8.0 부터).
  - 처음 접혀 시작하는 절(주고받은 메시지 상대 절 · 요청 '내 요청' 상자)을 펴면 `foldEl` 이 `open:{열쇠}`(예: `open:peer:1`)를 남긴다.
  - 이 열쇠는 서버 `run_opts::clean_folded` 의 `{화면}:{상대 id | me}` 검사에 걸려 상태 줄에 "접힘 저장 실패: ui_folded 는 …" 이 뜬다.
  - 목록을 통째로 거절하므로 다시 읽기 전까지 다른 절의 접힘 저장도 모두 실패한다.
  - 실측: 주고받은 메시지 절을 펼 때 상태 줄 문구가 떴다.
- stage2-2 타일 편집의 이름 칸이 사이드바 오른쪽 밖으로 넘친다(실측 사진 `tile-edit-*`). #1z-13 이 바꾸지 않은 입력 · 폼 규칙이라 원래 있던 결함으로 본다(확인 필요).
- stage2-3 `conv_leave::delete_while_waiting_leaves_no_rows` 간헐 실패 — 위 테스트 결과(시험 경합 추정).

## 다음 단계

stage 3 — 대화 종류 표시(intent U8): 후보 (가) · (나) · (다)를 실제 앱에 얹어 찍고 사용자가 고른다. `.room-av` · 대화 줄 카드 · 머리 종류 선도 함께 다룬다(개정 ② · ⑥).

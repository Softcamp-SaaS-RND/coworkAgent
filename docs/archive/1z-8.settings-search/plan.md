# plan — 1z-8.settings-search

> 근거: [intent](intent.md)(승인 2026-10-23 — 결정 1 ~ 12). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md)(D35 화면 · D40 나가기 · 삭제 · S4 웹UI).
> 브랜치 `task/1z-8-settings-search`(작업 폴더 `D:\dev\git\coworkAgent-1z-8`). 기준 커밋: `866be6e`(main). plan 승인: 2026-10-23(사용자 "개발 진행해" 로 갈음 — #1z-7 결정 9 와 같은 방식, 구현자 해석).
> 시안: https://claude.ai/artifact/RQFtDzbXq1z6a8512noLpL (설정 = 시안 B · 검색 = S1 · 미리보기 = 제안).

## 목표

상단 바에 버전, 통합 검색에 "나간 대화 포함" 스위치 · 유형 아이콘(S1), 개요 미리보기 말풍선 2줄 · 여백, 대화 세부정보 '제목' 줄 삭제,
설정 = 8 메뉴 · 고른 화면만 · 스위치 · 디버깅 · 연결 / 그룹설정 / 에이전트 목록 · 상세 2단(시안 B). 버전 1.3.4.

### 성공 기준(intent 사본)

- A1: 상단 바 wordmark 뒤 `v` + 실행 중 버전(1.3.4) 표시(시험 · 실제 화면).
- A2: 검색 결과 상자 스위치 "나간 대화 포함" — 끔 = 닫힌 대화의 메시지 · 대화 제목이 빠지고 켬 = 들어옴(태그 포함), "지운 대화는 검색되지 않습니다" 문구, 서버 기본 = 뺌(시험 · 실제 화면).
- A3: 검색 결과 줄의 유형 아이콘 · 색이 S1 규칙(메시지 · 대화 = 대화 유형, 사람 · 상대 에이전트 = 타일 색, 내 에이전트 = 아이콘 색)과 같다(시험 · 실제 화면, 라이트 · 다크).
- A4: 개요 미리보기 말풍선이 크기 100 ~ 200 % 모두 2줄까지, 아래 줄이 비치지 않고, 말풍선 사이 `--sp-3`(시험 · 실제 화면).
- A5: 설정 메뉴 8개(글자 그대로) · 고른 화면만 보임 · 고른 메뉴 = `aria-current` · 빈 `#settings` = 연결이 고른 상태로 시작 · 옛 해시와 진입 경로 전부 제 화면 · 초점(시험 · 실제 화면).
- A6: 설정 화면 안 체크박스 전부 + 검색 스위치가 스위치 모양(`role="switch"`) · 키보드(Space)로 바뀌고 저장 동작 그대로, 설정 밖 체크박스는 그대로(시험 · 실제 화면).
- A7: 연결 · 그룹설정 · 에이전트 = 목록 · 상세 2단 — 줄을 고르면 상세, ✕ · 닫기 = 고름 풀기 · 초점 그 줄, 다른 줄을 골랐다 돌아와도 저장 전 고친 값이 남음, 지금 관리 동작 전부 유지(시험 · 실제 화면).
- A8: 실행 · 화면 · 시작 · 알림 = 2열 타일, 좁은 창에서는 1열 · 목록 · 상세는 위아래로 쌓임(실제 화면).
- A9: 디버깅 — 파일 카드(경로 · 크기 · 복사) · 로그 끝 200줄(상한 넘지 않음 · 수준 거르기 · 새로 고침) · 진단 정보 복사, 디버깅 API 정보 칸 · 진단 정보 복사에 신원 키 · 토큰 · 지문 없음(로그 줄은 그대로)(시험 · 실제 화면).
- A10: 회귀 없음 — PROJECT.md 검증 명령 PASS, 버전 1.3.4, 매뉴얼 · README · PRODUCT · masterPlan 반영.
- A11: 대화 세부정보 '대화 정보' 에 `제목` 줄이 없다(상대 · 에이전트 · 실행 기본값만, 시험 · 실제 화면).

## 설계

### stage 1 — 상단 바 · 검색 · 미리보기 · 세부정보

- 상단 바: `index.html` wordmark 뒤 `<span id="top_version" class="wordmark-ver">`, `showStatus` 가 `v{version}`(실패면 빈 칸).
- 검색 서버: core `chat::search(conn, q, limit, closed: bool)` — 메시지는 `search_messages`(본문 LIKE · `context` 제외 · 최신 `limit`, `closed` 거짓이면 닫힌 대화 제외),
  대화는 `summaries` 중 제목이 맞고 (`closed` 또는 닫히지 않음). 닫힘 = `left_at` · `peer_left_at` · 끝난 공유 사본(`dir = 'shared' AND share_ended = 1`) — 웹UI `convClosed` 와 같다.
  사람 · 에이전트 묶음은 그대로. app `SearchQuery.closed: Option<bool>`(기본 거짓) → `GET /api/search?q=…&closed=true`. 호출부: `chat_api::search`, `agent/tests/message_tidy.rs`.
- 검색 화면: `#search_box` = 고정 머리 `#search_opts`(스위치 `#search_closed` `checkbox role="switch"` "나간 대화 포함" + 한 줄 "지운 대화는 검색되지 않습니다") + `#search_results`(묶음 · 줄을 여기만 다시 그림).
  스위치 값 = 그 체크박스 값(탭이 열린 동안만 — 설정 저장 없음), 바꾸면 다시 조회. `view.js` `searchGroups(r, peerNames)` 줄에 `mark`(`{type:"msg"|"conv", kind: dir}` · `{type:"peer", peer}` ·
  `{type:"agent", owner}`) · `tag`(닫힌 대화 문구 `convClosed`). `app.js` `searchMark(row)` = 칩: msg = `ico-chat` + 유형 색(`data-kind`), conv = `kindChip` 과 같은 칩(공유 사본은 점선 빈 칩),
  peer = `rowTile(small 타일 색, 그림)`, agent = 내 것 `agentTile(state.agents 의 그 이름 색, small)` · 상대 것 = 로봇 + 그 상대 타일 색. 줄 = 칩 · 라벨/부제 · 태그.
- 미리보기: `previewZoom(z)` → `{ scale }`(줄 수 없앰 — 모든 크기 2줄), `--pv-lines` 없앰. 말풍선 = `.pv-msg > .pv-text`(줄 자르기는 안쪽 `.pv-text` 에 — 안쪽 여백에 다음 줄이 비치지 않게),
  `.pv-body` gap `--sp-3`.
- 세부정보: `index.html` `<dt>제목</dt><dd class="muted">이 PC 에만 저장</dd>` 삭제.

### stage 2 — 설정 틀(8 메뉴 · 한 화면 · 스위치 · 2열 타일 · 디버깅)

- `view.js` `settingsTree()` = 평평한 8개 `{id, label, icon}` — `conn` 연결 `link` · `groups` 그룹설정 `users` · `agents` 에이전트 `bot` · `run` 실행 `play` · `screen` 화면 `monitor` ·
  `startup` 시작 `power` · `notify` 알림 `bell` · `debug` 디버깅 `bug`. `SETTINGS_ALIAS = { invite_make: "conn", invite_accept: "conn", peers: "conn", agent_new: "agents" }`.
  `route("#settings[/x]")` → `{ view: "settings", page, sub }` — `page` = 메뉴 id 또는 별칭의 메뉴(모르는 · 빈 = `conn`), `sub` = 별칭이면 그 글자 아니면 null.
- `newMenu()` "에이전트 등록" → `#settings/agent_new`(초점 `agent_name`). 나머지 진입은 그대로(`invite_make` + `window_btn` 누름, `invite_accept` 초점 `invite_text`, 레일 + 사용자).
- `index.html`: `#settings` = `#settings_tree` + `.settings-body`(머리 `#settings_head` — 제목 · 설명 · 머리 단추 자리) + 페이지 8개 `<section class="set-page" id="set_{id}" hidden>`.
  옛 카드 id(`conn` · `peers` · `agents` · `run` · …)는 페이지 id 로 바꾼다 — 화면 조각 id(`window_btn` · `accept_btn` · `invite_*` · `agent_form` · `agent_new` · PREFS 입력 id)는 그대로, 늘 DOM 에.
- `app.js` `renderTree()` — 링크마다 `aria-current="page"`(고른 메뉴만). `showSettingsPage(page, sub)` — 그 페이지만 보임 · 머리 제목/설명 · 트리 현재. `render()` 의 settings 갈래:
  `invite_make` → 초대 생성 상세, `invite_accept` → 초대 수락 상세(`showAccept`), `agent_new` → 새 에이전트 상세, `scrollIntoView` 없앰.
- 스위치: 설정 안 체크박스(알림 6 · 실행 1 · 시작 1 · 에이전트 등록 2 · 상세 2)에 `role="switch"` · CSS `.set-page input[type="checkbox"][role="switch"]`(트랙 · 손잡이, `appearance: none`). 값 · 이벤트는 그대로.
- 실행 · 화면 · 시작 · 알림 = `.set-tiles`(2열 격자) 타일 `.set-tile`(라벨 · 한 줄 설명 왼쪽 / 컨트롤 오른쪽). 960px 아래 1열.
- 디버깅: 서버 `server.rs` 보호 경로(토큰 필요 · `PUBLIC` 아님) `GET /api/debug` → `{version, os, arch, data_dir, files: [{key:"db"|"log", path, size}]}`(경로 = `Paths::under(data_dir)`),
  `GET /api/debug/log` → `{path, lines: [...]}` = 로그 파일 끝 64 KiB 안의 마지막 200줄(앞에서 자른 첫 줄 버림, 없으면 빈 배열). 신원 키 · `instance.json` · 지문 칸 없음.
  화면 = 파일 카드 4(버전 · 데이터 폴더 · DB · 로그 — 경로 · 크기 · 복사) · 로그 상자(수준 전체 / INFO / WARN / ERROR · 새로 고침 · 줄 앞 유닉스 초를 로컬 시각으로) · "진단 정보 복사"
  (`view.js` `diagText(info, me)` = 버전 · OS · 데이터 폴더 · DB · 로그 · 웹UI 포트 · 피어 포트 · 열린 연결 수 — 지문 · 토큰 없음).
- `.settings-body` 의 `max-width: 1000px` 없앰.

### stage 3 — 연결 · 그룹설정 · 에이전트 목록 · 상세(시안 B)

- 공통 `.md`(2열: 목록 360px · 상세) — 목록 = 검색 칸 + 줄 단추(`aria-pressed`, `data-pick`), 상세 = 머리(타일 · 이름 · 부제 · ✕ `aria-label="상세 닫기"`) · 몸 · 아래 단추 줄(닫기 끝).
  고름 = `state.pick[page]`(다시 그려도 유지), ✕ · 닫기 = 고름 풀기 + 초점 그 줄 단추. 아무것도 안 고르면 "{…}를 고르세요". 960px 아래 위아래로 쌓임.
- 연결: 머리 단추 `#window_btn`(+ 초대 생성) · `#accept_btn`(초대 수락). 목록 = 사용자(`groupSettingsList(...).users` — 검색 `#peer_q` 이름 · PC · 지문 · 대화 제목), 줄 = 타일 · 이름 · PC · 상태 점.
  상세 모드: 사용자 id = 모델 · effort(`gsDrafts`) · 상태 · 대화 수 · 지문 · 주소 · 대화 목록 + 저장 · 승인 / 다시 알리기 / 연결 확인 · 내 에이전트 편집 · 삭제 · 닫기,
  `invite` = `#invite_make`(초대 글 · 복사), `accept` = `#invite_accept`(붙여 넣기 폼). `#invite_make` · `#invite_accept` 는 상세 안에 늘 있고 `hidden` 만 바뀐다.
- 그룹설정: 머리 `+ 그룹 추가`(`#peer_group_add` — 만든 그룹을 고름). 목록 = 그룹(검색 `#group_q`), 상세 = 이름 바꾸기(`openGroupRename`) · "이 그룹의 대화"(빼기) · "넣을 수 있는 대화"(사용자별 · 넣기 = `changeGroup`) · 삭제 · 닫기.
  설정 화면 대화 끌기(`settingsConvRow` 손잡이 · `convGrab({join})` · `gsJoin` · `gsDragging` · `.gs-card[data-drag=group]`)는 지운다 — 레일 · 사이드바 끌기 그대로.
- 에이전트: 머리 `#agent_new_btn`(에이전트 등록 = 상세에 `#agent_new` 폼). 목록 = 내 에이전트(로봇 타일 · 이름 · 폴더 · 상위 탐색 · 셸 표시), 상세 = #1z-7 한 격자 + 저장 · 삭제 · 닫기.
  저장 전 고친 값 = `agentDrafts`(id → 폴더 · 추가 지시 · 상위 탐색 · 셸 · 색), 저장 성공이면 지움. `editAgent(id)` = 고름 + 초점 그 줄, 사용자 상세 "내 에이전트 편집" = 에이전트 페이지.
- `manageItem` · `gsToggle` · `gsGroupsEl` · `gsUsersEl` · `settingsConvRow` · `USER_LIST` · `settingsGroupSections` 중 안 쓰게 된 것은 지운다.

### 바꾸는 시험(의도된 변경 — 약화 아님)

- `view.test.mjs`: `settingsTree` 모양(평평한 8개 · 아이콘) · `route` 설정 갈래(`{page, sub}` + 별칭 6 · 빈 · 모르는) · `previewZoom`(줄 수 없음) · `searchGroups`(`mark` · `tag`) ·
  `newMenu` 에이전트 등록 해시 · `settingsGroupSections` / `USER_LIST`(지우면 시험도). 새 시험: `diagText`(지문 · 토큰 없음).
- `ux_literals.rs`: "이 PC 에만 저장"(#17 — 세부정보 줄 삭제), `{ id: "peers", label: "그룹설정"…}` · `<h2>그룹설정<span`(#1z — 메뉴 8개 · 페이지 머리로), `gsGroupsEl/gsUsersEl` ·
  `gs-card data-drag` · `gsToggle(gsu…)` · `manageItem` · `manage-btn` · `if (n.press)…`(트리 하위 없음), `showAccept(r.view === "settings" && r.focus === "invite_accept")`, 초대 ? 문구의 "그룹설정에서 지우세요"
  (사용자 표가 연결로 — "연결에서"), `#1z-7` 상세의 연필 · 닫기 행 → 새 구조 행으로 바꿔 적는다. 행마다 새 출처 `#1z-8`.
- `ux_api.rs` 검색 시험은 그대로 통과해야 한다(닫히지 않은 대화) + 새 `search_closed_switch`.

### 문서

[web-ui](../../manual/dev/web-ui.md) · [an_app](../../manual/dev/an_app.md) · [an_core](../../manual/dev/an_core.md) · [사용자 가이드](../../manual/user/user-guide.md) · PRODUCT · README(1.3.4) ·
masterPlan D35(설정 · 미리보기 2줄) · S4(검색 스위치 · 설정 목록 고르기 — #1r 연필 개정) · 개정 기록(종결). 버전 네 크레이트 1.3.4.

## 평가 항목

- [ ] E1: A10 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A10 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A3·A4·A5·A9 — 웹UI 로직 시험(검색 표지 · 미리보기 크기 · 설정 메뉴 · 해시 별칭 · 진단 글)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A2 — 검색 스위치 서버(기본 = 닫힌 대화 뺌 · closed=true = 포함 · 끝난 공유 사본도 닫힘)
  CHECK: cargo test -p coworkagent-app --test ux_api search_closed
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A9 — 디버깅 API(토큰 필요 · 정보 칸 · 로그 끝 200줄 · 64 KiB 상한 · 지문/토큰/키 칸 없음)
  CHECK: cargo test -p coworkagent-app --test debug_api
  EXPECT: /test result: ok\. [3-9] passed; 0 failed/
  EVIDENCE: pending
- [ ] E6: A1·A5·A6·A7·A8·A11 — 화면 원본 literal(버전 칸 · 검색 머리 · 메뉴 8 · 페이지 · 스위치 · 2열 타일 · 목록 · 상세 · 닫기 · 제목 줄 없음 · 설정 끌기 없음)
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E7: A10 — 문서 규약 · 크레이트 경계 · 버전 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('ver=%d webui=%d app=%d core=%d guide=%d'%(sum(r('crates/%s/Cargo.toml'%c).count('version = \x221.3.4\x22') for c in ('agent','app','core','net')),r('docs/manual/dev/web-ui.md').count('(#1z-8'),r('docs/manual/dev/an_app.md').count('/api/debug'),r('docs/manual/dev/an_core.md').count('closed'),r('docs/manual/user/user-guide.md').count('**디버깅**')))"
  EXPECT: /ver=4 webui=[1-9][0-9]* app=[1-9][0-9]* core=[1-9][0-9]* guide=[1-9][0-9]*/
  EVIDENCE: pending
- [ ] E8: A1 ~ A9 · A11 — 실제 화면: 디버그 빌드(시험 데이터 폴더 — 열린 · 닫힌 대화 시드)에서 상단 바 버전, 검색 스위치 끔/켬 결과 · 칩 색, 미리보기 125 % 2줄 · 비침 없음 · 여백,
  세부정보 제목 줄 없음, 설정 첫 화면 = 연결 고름, 메뉴 8 · 한 화면만, 옛 해시 · 새로 만들기 · 레일 + 진입, 스위치 Space 토글 · 저장, 2열 타일 · 좁은 창 1열,
  연결 · 그룹설정 · 에이전트 고르기 · ✕ · 닫기 초점 · 다른 줄 갔다 와도 초안 · 넣기 / 빼기 · 등록 폼, 디버깅 카드 · 로그 · 복사, 라이트 · 다크
  EVIDENCE: Claude(구현자) 2026-10-23 — `target/debug/coworkAgent.exe start --parent <대기 프로세스>` + `COWORKAGENT_DATA_DIR`=scratchpad `ui1z8\data`(사용자 앱 그대로, 상대 3 · 대화 5 —
  `in` 2 · `out` 3, 나감(`left_at`) 1 · 상대가 나감 1 · 긴 답 · 그룹 '결제'(대화 1) · '빈 그룹' sqlite 시드, 에이전트는 화면에서 등록), ego-browser DOM 판독 · 스크린샷 1440 × 900 · 900 × 900.
  관측: 상단 바 `#top_version` "v1.3.4". 검색 "검색시험" 끔 = 메시지 2 · 대화 2(열린 대화만, 칩 `s-mark kind-chip` `data-kind` in · out, 태그 없음) → 스위치 켬 = 메시지 6 · 대화 4,
  닫힌 줄 태그 "나감" · "상대가 나감", 스위치 크기 36 × 20(처음 96px — 검색 칸 padding 을 물려받음 → 고침). 개요 미리보기 125 % — `.pv-text` `-webkit-line-clamp` 2 · 높이 33px(내용 58px 잘림) ·
  말풍선 사이 12px · 아래 줄 비침 없음(스크린샷). 대화 세부정보 '대화 정보' dt = 상대 · 상대 에이전트 · 실행 기본값(제목 없음). 설정 `#settings` = 화면 conn 만 · 트리 aria-current
  연결 = page · 나머지 false · 상세 "사용자를 고르세요". 사용자 줄 고름 → `aria-pressed` true · 상세 단추 저장 · 연결 확인 · 내 에이전트 편집 · 삭제 · 닫기 → 닫기 = 상세 숨김 · 초점 `md_conn_1`.
  레일 + 사용자 → `#settings/invite_accept` · 초대 수락 칸 열림 · 초점 `invite_text` · `aria-expanded` true. 새로 만들기 → 초대 생성 = `#settings/invite_make` · 초대 생성 칸 · 초대 글 179자 ·
  "초대를 만들었습니다…" → 닫기 = 초점 `window_btn`. `#settings/peers` = 연결. 그룹설정 '결제' 고름 → 이 그룹 1 · 넣을 수 있는 4(나감 · 상대가 나감 태그) → "빌드 검색시험 원인 넣기" = 이 그룹 2 ·
  "그룹 결제 에 넣었습니다" → "배포 일정 검색시험 빼기" = 1(처음 상세 아래 단추 줄이 칸 중간 → 고침). 새로 만들기 → 에이전트 등록 = `#settings/agent_new` · 에이전트 화면만 · 폼 · 초점 `agent_name`
  → 등록 = "등록 완료" · 그 에이전트 고름 · 초점 `md_agents_1` → 추가 지시 "초안 지시" · 색 peer-3 → 실행 화면 → 에이전트 → 다시 고름 = "초안 지시" · peer-3 그대로 → ✕ = 초점 `md_agents_1`.
  알림 `notify_pair` 초점 + Space = 켬 → 끔 · 상태 줄 "설정 저장"(다시 Space 로 되돌림), 설정 안 `role=switch` 없는 체크박스 0. 실행 · 알림 = 2열 타일(496px × 2), 900px 창 = 1열(464px) ·
  연결 목록 아래(목록 bottom 370 < 상세 top 386) 위아래 · 다크 테마 화면(스크린샷). 디버깅 = 카드 4(버전 1.3.4 (windows x86_64) · 데이터 폴더 · DB 152.0 KB · 로그 936 B — 크기 줄바꿈 → nowrap 고침) ·
  로그 8줄 "10-23 11:00:34 WARN …" 꼴. 끝에 `coworkAgent stop`.

## 개정 기록

- 2026-10-23(구현 중 — 설계만, 평가 항목 · EXPECT 그대로): ① stage 2 · 3 을 한 커밋(`work/stage2.md`)으로 — 설정 HTML 을 한 번에 최종 구조(시안 B)로 바꿔 중간 구조를 두 번 만들지 않음.
  ② 에이전트 목록 검색 칸은 두지 않음 — `ux_literals` 의 #1l "에이전트 검색 없음" 결정과 충돌(그 결정 유지, 사용자 · 그룹설정 목록만 검색).
  ③ 설정 화면 머리에 한 줄 설명은 두지 않음 — D35 "내용이 곧 설명인 화면에는 설명 문구를 두지 않는다"(설명은 제목 옆 '?' 그대로, 타일 칸 설명은 둠).
  ④ 검색 스위치 값 = 체크박스 그 자체(`state.searchClosed` 대신). ⑤ `.set-tile-text` → `.set-tile-label`(#1j "글자 타일 없음" 문구 검사와 겹침).

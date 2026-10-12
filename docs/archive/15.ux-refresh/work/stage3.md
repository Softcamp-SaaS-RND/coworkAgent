# stage3 — 웹UI 전면(토큰 · 테마 · 설정 트리 · 표 · 아코디언 · 편집 · 뷰어 · 나/보낸 목록 · 문구)

## 목표

[plan](../plan.md) Stage 3 — 고른 시안 A(토큰 = [stage1.md](stage1.md))로 웹UI 를 다시 입히고 설정 트리 · 그룹 아코디언 · 편집 ·
markdown/HTML 뷰어 · '나' 받은 글 · 보낸 메시지 탭 · 방향 태그 · 테마 · 문구 literal 을 더한다. 대상 gate: E3 · E5 · E7.

## 산출 파일

- `crates/app/web/view.js` — `convLabel` 이 `{title, sub, tag: "보냄" | "받음"}`(화살표 없음) · `route` 에 `#s/{id}`(보낸 메시지) ·
  `#inbox` · `settingsTree()` · `groupFilter` · `groupNames` · `NO_GROUP` · `inboxRows` · `preview` · `isHtmlDoc` · `HTML_CSP` · `htmlSrcdoc` ·
  `inlines` · `markdownBlocks` · `composerMode` · `READ_ONLY_TOOLS` · `EFFORTS`.
- `crates/app/web/index.html` — 레일 '나'(받은 글) · 사이드바(대화 검색 · 목록) · `#inbox` · 대화 머리글 탭(대화 · 보낸 메시지 · 타임라인 — 한 머리글,
  패널만 바뀜) · 작성창(대상 선택 없음) · 설정(트리 + 내 정보 kv 표 · 연결(초대 생성 · 초대 수락 · 연결 목록) · 내 에이전트 · 실행 · 화면 · 알림).
- `crates/app/web/app.css` — `:root` 하나에 시안 A 라이트 토큰 + 다크 원값(`--dark-*`), `:root[data-theme="dark"]` 는 같은 이름을
  `var(--dark-*)` 로 바꾼다(리터럴 없음). 그 밖 색 · 글꼴 크기 리터럴 0(E5).
- `crates/app/web/app.js` — 위 화면 조립. markdown 은 정해진 태그 + `textContent` 로만, HTML 문서 본문은 `<iframe sandbox="">` 의 `srcdoc`(CSP 앞붙임).
- 테스트: `view.test.mjs` 7건 추가(13 → 20, 기존 "대화 이름" 은 태그 기대값으로 개정 · "화면 위치" 에 `#s` · `#inbox`) ·
  `crates/app/tests/ux_literals.rs`(신규 3건).

## 설계 결정

- **설정 트리에 "화면 — 테마" 항목** — plan 의 트리 목록(내 정보 / 연결 / 내 에이전트 / 실행 / 알림)은 개정 2 의 테마(A13)를 넣기 전에 쓰였다.
  A3 "설정 항목이 트리뷰로 전부" 를 지키려고 더했다 — **plan 편차 1**(설계 문구, gate 무관). 실행 항목 라벨은 시안 A 그대로 "실행 — 모델 · effort".
- **사이드바 구성** — 시안 A 대로 머리(고른 PC) · 대화 검색 · 대화 목록만. 이전 사이드바의 "내 에이전트" · "연결 가능한 사용자" 절은 설정의
  내 에이전트 · 연결 목록으로 옮겼다(같은 정보, 편집 가능). **plan 편차 2**(plan 이 정하지 않은 배치 — 고른 시안을 따름).
- 대화 검색은 이 화면이 가진 값(제목 · 상대 · 에이전트)으로 거른다 — 본문 검색은 '나' 받은 글(`/api/messages?q=`)이 맡는다.
- 보낸 메시지 탭 = 그 대화 상대에게 내가 쓴 글 전부(`side=mine&peer=`), 행을 누르면 그 대화. 탭 이름 옆 건수(시안의 "보낸 메시지 1")는 뺐다 —
  대화를 열 때마다 요청이 하나 더 늘어서. 필요하면 건수 API 로 더한다.
- 받은 글 · 보낸 메시지 행의 태그는 그 글이 속한 **대화의 방향**(목록과 같은 규칙, `title="대화 방향"`).
- 원문 보기는 메시지마다 토글. 5초 새로 고침은 메시지 서명(id · 상태 · 길이)이 같으면 다시 그리지 않는다 — 토글 · iframe 상태 유지.
- 편집 / 미리보기 = 작성창(대화 · 새 대화) · 추가 지시(등록 · 편집) — 입력칸 위 버튼, 미리보기는 같은 markdown 뷰어.
- 연결 항목 펼침 = 지문 · 주소(읽기 전용 kv) · 그룹 · 모델(빈 칸 = "기본값(sonnet)" 표시, 목록 제안 sonnet · opus · haiku) · effort(기본값 + 5단계) ·
  저장 · 승인/다시 알리기/연결 확인 · 내 에이전트 도구 편집(내 에이전트 절로 이동) · 삭제. 펼친 항목은 다시 그려도 유지.
- 에이전트 항목 = 이름 kv(읽기 전용) · 폴더 · 도구 체크(D31 5종) · 추가 지시 · 그룹 · 저장 · 삭제. 새 등록은 목록 아래 접이식, 그룹은 등록 뒤 편집에서.
- 다크 테마의 레일 · 사람 아바타 글자는 `--on-dark`(흰색, 두 테마 공통) — `--on-accent` 는 다크에서 어두운 색이라 어두운 바탕 위에서 안 보였다(실화면에서 발견).
- 화면 높이 = 창 높이(`#app` 100vh) — 설정 · 목록 · 메시지가 각자 스크롤하고 레일 · 사이드바는 남는다. 좁은 화면(720px 이하)은 이전처럼 쌓는다.

## 테스트 결과

- `node --test --test-reporter=tap "crates/app/web/*.test.mjs"` — `# pass 20 # fail 0`(E3). 처음 `group_filter_search` 1건 FAIL —
  `localeCompare("ko")` 는 한글을 라틴보다 앞에 둔다, 기대값을 그 규칙으로 고침.
- E5 — `root=1 violations=0`(시작 69). E7 `ux_literals` 3 passed.
- `cargo test --workspace` — `passed=173 failed=0`(Stage 2 170 + ux_literals 3). clippy `Finished` · `cargo fmt --check`(처음 `ux_literals.rs` 줄바꿈 1건 → `cargo fmt`).
- 실화면 확인(임시 미리보기 서버 + Edge headless, 커밋 안 함): 대화 · markdown 표 · 원문 보기 · HTML 본문 sandbox(콘솔: 스크립트 차단
  "allow-scripts permission is not set" · 외부 그림 CSP 차단 "img-src data:") · 미리보기 · 보낸 메시지 · 타임라인 · 받은 글 · 설정 트리 · 연결/에이전트
  펼침 · 다크. 여기서 고친 것: 작성창이 가운데로 쪼그라듦(`form` 의 `align-items: center`) · 레일 버튼이 눌림 스타일에 덮임 · 다크 글자 대비 · 스크롤 시 사이드바가 같이 밀림.
- E11(실화면 수동 · 두 PC)은 종결 때 D28 에 따라 19 TC-K 로.

## 리뷰

분리된 리뷰 에이전트(`0cd5533..b1fb0df`, 읽기 전용) — high 0, medium 3, low 8, nit 2. HTML 주입 경로 0(`innerHTML` 류 0건), markdown 정규식
20만 자 12ms 이하, API 필드 · 설정 키 일치 확인. 코드로 확인 후:

| 지적 | 판단 | 조치 |
|---|---|---|
| 받은 대화 부제 `{상대} → 내 {에이전트}` 에 화살표가 남음(A15) | 사실 | `{상대} 의 요청 · 내 {에이전트}` |
| `direction_tag` 테스트가 부제 가운데 화살표를 못 잡음 | 사실 | 두 방향 부제 · 받은 제목에 `[→←]` 없음 단정 |
| 알림 이벤트마다 연결 목록을 다시 그려 입력 중 값이 사라짐(에이전트 저장 때 다른 폼도) | 사실 | `needsRender` — 값이 같으면 건너뛰고, 그 목록 안에서 입력 중이면 배경 새로 고침을 미룬다. 저장 · 필터는 강제 |
| `+`(초대 수락) · 트리 항목이 다른 화면에서 누르면 스크롤 · 포커스가 안 됨(hashchange 가 늦음) | 사실 | `goSettings` — 화면을 먼저 보이고 스크롤 |
| 뒤로 가기로 레일과 다른 상대의 대화를 열면 보내기가 막힘 | 사실 | 그때 레일을 '나'(전부)로 |
| 타임라인 · 보낸 글 · 받은 글이 5초마다 다시 그려져 iframe 깜박임 | 사실 | 같은 `needsRender` 서명 비교 |
| 새 대화의 상대를 빨리 바꾸면 늦은 에이전트 목록이 덮어씀 | 사실 | 따로 `Latest` 티켓 |
| JSON 아닌 오류 본문이면 "Unexpected token…" | 사실 | 상태 코드 + 본문 앞 200자 |
| HTML 본문 링크를 누르면 프레임이 외부 URL 로 이동(열람 · IP 노출, 스크립트 없음) | 사실 | `srcdoc` 에 `<base target="_top">`(top 이동은 sandbox 가 막음) + 테스트 단정. `<meta refresh>` 자기 이동은 **확인 필요** — CSP `default-src 'none'` 아래 프레임 안 이동이라 읽기 · 실행은 없다 |
| `_` 기울임이 한글 낱말 안(`설정_키_값`)에서 깨짐 | 사실 | 경계를 `\p{L}\p{N}_`(유니코드)로 + 테스트 단정 |
| 트리 id 가 index.html 절과 맞는지 미검증 | 사실 | `settings_tree` 가 index.html 의 `<section id>` 를 확인 |
| 미리보기 중 숨은 `required` 입력이 아무 안내 없이 제출을 막음 | 사실 | `required` 제거, JS 가 빈 글 안내 |
| 도구를 모두 해제하고 저장하면 서버가 말없이 기본값으로 | 사실 | 하나 이상 고르라고 막음 |
| (확인 필요) 보낸 메시지 탭에 받은 대화의 내 에이전트 답도 나옴(`side=mine`) | plan "내가 쓴 글" = API `mine` 그대로 | **미조치 — 사용자 확인 대상**(사람 글만으로 좁힐지) → 사용자 답(2026-10-12): 질의 · 요청 구분 새 기능으로 — 15 는 지금 정의로 종결([intent](../intent.md) 라운드 6) |
| (확인 필요) 상태 문구 `sent` = "보냄" 이 방향 태그 "보냄" 과 겹쳐 보일 수 있음 | 사실(같은 낱말) | 미조치 — 상태 "보냄" 은 16 이전부터의 문구, 바꾸면 범위 밖 |

반영 뒤: `node --test` `# pass 20 # fail 0`(테스트 수 그대로 — 단정 추가) · `passed=173 failed=0` · clippy `Finished` · `cargo fmt --check` · E3 · E5 · E7 ✅.

## 사용자 수정 지시(빌드 확인 뒤, intent 라운드 4 · plan 개정 4)

사용자 원문 "수정 사항. - 기본이 다크테마. - 추가된 사용자 버튼('나', 'sa') 클릭 시 해당 사용자와 대화한 내용만 리스트에 출력안됨. - 환경설정 버튼 선택 시 대화창 옆에 환경설정 항목 출력됨. 전체화면에 환경설정 떠야함. - 모든 대화별 '원문보기'제거. 원문보기 버튼은 하나만 유지하고 해당 버튼 누르면 모든 대화가 원문으로 보임. - 프롬프트 입력창 위의 '편집', '미리보기' 버튼 제거. 프롬프트 입력은 편집모드 상태로 기본 동작. - 대화 목록에 가변 컨트롤 적용." 모호한 3건은 물어서 정함(레일 = "상대=그 상대 글 목록", 가변 = "사이드바 폭 드래그 조절", 편집 버튼 = "모든 입력칸에서 제거").

| 지시 | 조치 |
|---|---|
| 기본 다크 | `run_opts::theme` 기본 `dark` · `index.html` `<html data-theme="dark">`(설정 읽기 전 깜박임 없음) · `applyTheme` 은 `light` 일 때만 라이트 · 테마 선택 순서 다크 먼저. `theme_setting_persists` 기대값 개정 |
| 레일 상대 클릭 → 그 상대 글만 | 원인: 상대 버튼이 사이드바만 걸렀고 메인은 그대로(상대 1명이면 '나' 와 같아 보임). 새 화면 `#p/{id}` — `side=theirs` · `mine`(그 상대) 두 요청을 `view.js` `peerMessages` 로 최신순 합쳐 받은 글과 같은 행으로. 같은 해시를 다시 누르면 `go()` 가 바로 다시 그림. 뒤로 가기로 `#p` · `#inbox` 에 오면 레일도 맞춤 |
| 설정 전체 화면 | `show("settings")` 때 사이드바 · 폭 손잡이 숨김(레일은 이동용으로 남김) |
| 원문 보기 하나 | 메시지마다 버튼 삭제. 대화 머리글 `#raw_btn`(`aria-pressed`) 이 `#conversation.show-raw` 를 바꾸고 CSS 가 뷰어 · 원문 중 하나만 보인다 — 다시 그리지 않아 iframe · 스크롤 유지, 대화 · 타임라인 둘 다, 대화를 옮겨도 유지 |
| 편집 / 미리보기 제거 | `attachPreview` · `composerMode` · `.mode-bar` · `.preview` 삭제(작성창 · 새 대화 · 추가 지시) |
| 대화 목록 가변 | `#side_resize`(`role="separator"`) — 끌기(window pointermove) · ← → 16px, 200 ~ 560px, `localStorage` `coworkagent.side_width`. `#app` 은 넓은 화면에서 줄바꿈 없음(폭을 넓혀도 메인이 아래로 떨어지지 않게), 좁은 화면(720px 이하)은 이전처럼 쌓고 손잡이 숨김 |

- 테스트: `view.test.mjs` `edit_preview_toggle` → `peer_messages`(20건 그대로) · "화면 위치" 에 `#p/2`. `node --test` `# pass 20 # fail 0` ·
  `cargo test --workspace` `passed=173 failed=0` · clippy · `cargo fmt --check` · E5 `root=1 violations=0`.
- 실화면(임시 미리보기 서버 + Edge headless, 커밋 안 함): 첫 화면 `data-theme=dark` · 상대 클릭 `#p/1` 주고받은 글 6행 · '나' 받은 글 3행 ·
  대화 메시지별 버튼 0 · `.mode-bar` 0 · 원문 보기 → 대화 원문 2 / 뷰어 0, 타임라인도 원문 · 설정에서 사이드바 숨김(메인 1336px) ·
  끌기 272 → 419px · ← 키 403px · 새로 고침 뒤 403px 유지 · 콘솔 오류 0. 처음 끌기가 한 칸만 움직임(`setPointerCapture` 를 Playwright 끌기에서 잃음) →
  window 수준 `pointermove` / `pointerup` 으로 바꿔 해결.

## 사용자 수정 지시 2(라운드 4 빌드 확인 뒤, intent 라운드 5 · plan 개정 5)

원문 · 결정은 [intent](../intent.md) 라운드 5. 모호한 4건은 물어서 정함(타일 클릭 = "사이드바 목록 → 메인 메시지", 제목 = "내 화면에만",
편집 버튼 = "대화 머리글 제목 옆", 타일명 = "표시 이름(별칭)").

| 지시 | 조치 |
|---|---|
| 타일 → 오른쪽 대화 목록 → 고르면 메시지 | `pickTile` — 사이드바를 그 타일 대화 목록으로('나' = 전부), 열린 대화가 그 타일 것이면 그대로, 아니면 메인 = 안내(`#`). 라운드 4 의 "상대 = 메인에 주고받은 글" 대신 사이드바 머리 링크(`#side_link`: '나' = 받은 글 `#inbox`, 상대 = 주고받은 글 `#p/{id}`). `go()` 는 빈 해시를 `#` 로 맞춰 같은 화면이면 바로 다시 그림 |
| 새 대화 '제목' | `#chat_title`(선택) → `POST /api/conversations` `title` — 보내기 전에 `run_opts::clean_title`(100자 · 제어 문자 없음) 검증, `Chat::send_via` · `start_relay` 가 대화와 한 트랜잭션으로 `chat::set_title`(리뷰 1). 스키마 10 `conversations.title`, `summaries` 는 붙인 제목 → 첫 사람 글 앞 40자 |
| 제목 편집 버튼 | 머리글 제목 옆 ✎(`#title_edit_btn`) → 그 자리 입력칸(`#title_form`, 저장 · 취소 · Esc). `PATCH /api/conversations/{id}` `{title}`(빈 값 = 지움, 모르는 항목 거절, 없는 대화 404). 대화를 옮기면 편집을 닫는다 |
| '대화 검색' → '제목 검색' | 라벨 literal · 자리 표시 "제목" · 거르는 값도 제목만 |
| 타일명 수정 · 아이콘 등록 | 사이드바 머리 ✎(`#tile_edit_btn`) → `#tile_form`(표시 이름 · 아이콘 고르기 · 아이콘 지우기 · 미리보기). 그림은 캔버스로 가운데 정사각형 80×80 PNG data URL. 상대 = `PATCH /api/peers/{id}` `alias` · `icon`(스키마 10 `peers.alias` · `peers.icon`, `set_peer_tile`), 나 = `PUT /api/settings` `me_alias` · `me_icon`. 검증 `run_opts::clean_alias`(32자) · `clean_icon`(PNG 서명 base64 로 시작 · base64 글자 · 65,536자). 표시 이름은 `displayName` · `tileText`(글자 묶음 앞 두 글자)로 타일 · 사이드바 머리 · 대화 목록 · 발화자 · 상대 고르기에, 설정 연결 목록 · 사이드바 머리 부제엔 원래 이름도. 저장 뒤 열린 대화를 다시 읽어 발화자 이름도 바꾼다 |
| 트레이 '사용자 정보' 제거 | 실제 이름 "내 정보 보기"(원문 이름과 다름 — 정상 모드에서 열기 · 종료 밖의 유일한 항목). `tray.rs` 정상 모드 = "열기" · "종료". 오류 모드 그대로 |
| (발견) 사이드바 폭이 다음 실행에 안 이어짐 | 라운드 4 의 `localStorage` 저장은 웹UI 포트가 실행마다 바뀌어(`server::bind` = `127.0.0.1:0`) 출처가 달라져 이어지지 않는다 → 설정 `ui_side_width`(200 ~ 560, 300ms 몰아 저장) |

- 공통: 전역 `[hidden] { display: none !important; }` 로 `.view` · `.panel` · `#sidebar` 의 개별 규칙을 대신(새 폼 · 제목 줄도 같은 규칙).
  `loadPeers` 가 대화 목록 · 열린 대화 머리글도 다시 그린다(표시 이름). `openConversation` 서명에 상대 표시 이름을 넣어 이름이 바뀌면 다시 그린다.
- 테스트: `view.test.mjs` `tile_text`(20 → 21) · `ux_literals` `title_search_label` · `tray_menu_open_quit_only`(3 → 5) · `ux_api`
  `conversation_title_set_and_edit` · `peer_tile_alias_icon` · `settings_me_tile_side_width`(8 → 11) · `paths` `schema10_title_alias_icon`(11 → 12,
  최신 스키마 단정 9 → 10) · `run_opts` 단위 `alias_title_icon_rules`.
- 실화면(임시 미리보기 서버 + Edge headless, 커밋 안 함): 처음 '나' = 전체 대화 3 · 메인 안내 · 링크 "받은 글 보기" → 상대 타일 = 사이드바 그 상대 ·
  링크 "주고받은 글 보기" · 메인 안내 → 대화 고르면 `#c/3` 메시지 2 → 같은 타일 다시 눌러도 `#c/3` 그대로 → ✎ 제목 "배포 절차 문의" 저장(머리글 ·
  목록) → "제목 검색" "배포 절차" 2건 · "user2" 0건 → 타일 ✎ 표시 이름 "세일즈" + 아이콘(`coworkAgent.ico` → PNG data URL) 저장 = 타일 그림 ·
  사이드바 머리 "세일즈" / 부제 "user2 · PC-b · 페어링됨" · 목록 · 머리글 · 발화자 "세일즈" → '나' 표시 이름 "김팀장" = 타일 "김팀" → 새 대화 제목
  "새 대화 제목" → 폭 끌기 389px 가 설정에 저장 · 새로 고침 뒤 389px → 설정 연결 목록 "세일즈 user2 · PC-b". 콘솔 오류 0.

### 리뷰(분리 리뷰 에이전트, 읽기 전용 — high 0 · medium 1 · low 5 · nit 1 · 테스트 공백)

| 지적 | 판단 | 조치 |
|---|---|---|
| 전송이 실패해도(시간 초과 · 거절) 대화 행은 남는데 제목은 버려짐 — 보낸 뒤에만 저장 | 사실(`send_via` 는 전송 전에 대화를 만든다) | 제목을 대화와 한 트랜잭션으로 보내기 전에 저장(`send_via` · `start_relay` 에 `title`), 보낸 뒤 저장 · `warning` 경로 삭제. 단정: 없는 에이전트(거절 403)여도 제목 · 자동 왕복 대화 제목 |
| `PATCH /api/peers` 가 실행 설정 · 타일을 두 번 나눠 쓰고, 한쪽만 고쳐도 미리 읽은 다른 쪽을 다시 씀 | 사실 | 준 쪽만 검증 · 저장(화면은 따로 보낸다), 준 값만 검증 — 저장된 틀린 값(이전 버전)이 다른 쪽 수정을 막지 않음. 둘 다 주면 차례로(한 트랜잭션 아님, 주석). 단정: 틀린 model 이 저장돼 있어도 타일 · 그룹 수정 200 · 빈 요청 = 그대로 |
| 고른 상대가 목록에서 빠지면(다시 hello) 타일 편집이 그 상대에 묶인 채 남음 · 저장 끝이 그 사이 연 다른 편집을 닫음 | 사실(코드상) | `renderRail` 이 고른 상대를 비울 때 편집을 닫고, 저장 끝은 같은 편집일 때만 닫는다 |
| 표시 이름이 빠진 곳 — 주고받은 글 제목 · 메시지 알림 · 연결 목록 검색 | 사실 | `displayName` · 받은 메시지 알림 이벤트 이름 = 표시 이름(페어링 알림은 원래 이름 — 지문 확인용) · 검색 항목에 `alias`. 단정: roundtrip 의 A 알림 이름 = A 가 붙인 표시 이름, B 에는 없음 |
| `authorLabel` 이 첫 " · " 에서 잘라 표시 이름 "세일즈 · 김팀장" 이 "세일즈" 로 | 사실 | 마지막 " · "(PC 앞)에서 나눈다 + 단정 |
| 제목 편집칸에 첫 글에서 따온 제목(줄바꿈 · 탭)이 그대로 → 탭이면 저장 400 | 사실 | 제어 문자를 공백으로 바꿔 채운다 |
| 폭 저장 300ms 사이 다른 설정 저장 응답이 폭을 되돌림 · 시작 전 `aria-valuenow` 없음 | 사실(nit) | 폭은 시작할 때만 설정에서 읽는다 · `index.html` 에 `aria-valuenow="272"` |
| 테스트 공백 — 타일 판단 · 제목만 거르기가 `app.js` 안, 자동 왕복 + 제목, 폭 경계 허용 | 사실 | `view.js` `tileKeeps` · `convRows` 로 옮겨 기존 "레일 전환" · "레일 거르기" 테스트에 단정(건수 그대로), 자동 왕복 제목 · 경계 200 / 560 단정 |
| (질문) '나' 표시 이름이 내 글의 발화자 "나" 도 바꾸나 | 명세 미결정 | 미조치 — "나" 그대로, 사용자 확인 대상 → 확인 답(2026-10-12) "나" 그대로([intent](../intent.md) 라운드 6) |

반영 뒤: `node --test` `# pass 21 # fail 0` · `cargo test --workspace` `passed=180 failed=0`(새 단정은 기존 테스트 안) · clippy `Finished` ·
`cargo fmt --check` · 실화면 다시 확인(같은 순서, 표시 이름 저장 직후 열린 대화 발화자도 "세일즈", 콘솔 오류 0).

## 다음 단계

적대적 평가(`eval.md`, codex 라운드 상한 2) → 종결(masterPlan D8 · D14 · D21 · D22 · D23 · S4 환류, 19 TC-J `say` 삭제 · TC-K 추가, 매뉴얼 · PRODUCT).

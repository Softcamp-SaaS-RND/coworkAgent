# 웹UI 화면 규칙

> 모듈 매뉴얼 형식(개요 / 공개 계약 / 사용 예)의 **예외** — 화면 규칙 문서다([manual/README](../README.md) 등록 규약). `crates/app/web/`
> (`index.html` · `app.css` · `js/` 모듈 — 아래 [모듈](#모듈) 표)를 고칠 때만 연다. API · 화면 함수 계약은 [an_app](an_app.md), 통신 · 저장 흐름은
> [개발자 가이드](developer-guide.md), 결정 번호(D · S)는 masterPlan. 지켜야 하는 제품 규칙 요약은 루트 `PRODUCT.md` `## 도메인 규칙`.

## 모듈

`crates/app/web/js/` 의 ES 모듈(빌드 없음) — 서버가 `server::SCRIPTS` 표의 이름만 `/js/{이름}` 으로 준다(파일을 더하거나 빼면 표도, 시험 `web_assets` 가 대조).
진입 `js/app.js` 가 맨 먼저 `js/core/token.js` 를 불러(토큰이 없으면 거기서 멈춤) 모든 UI 모듈을 불러 이벤트를 건 뒤 첫 화면을 그린다. 층: `js/core/dom.js` · `js/core/state.js` ·
`js/core/token.js` 는 도메인 모듈을 부르지 않고, `js/logic/*`(DOM 없음 — `view.test.mjs` 가 `node --test` 로 시험)은 UI 모듈을 부르지 않으며 서로 돌지 않는다. 공유 상태는
`js/core/state.js` 의 `state` 하나(다른 모듈은 그 속성만 바꾼다 — 다른 모듈의 `let` 에 대입하지 않는다).

| 모듈 | 담는 것 |
| --- | --- |
| `js/app.js` | 진입 — 토큰 확인 뒤 도메인 모듈을 불러 이벤트를 걸고 첫 화면을 그린다. |
| `js/core/token.js` | 진입 토큰 — 트레이 "열기" 가 준 `/?t=` 를 탭 메모리로 가져가고, 없으면 다시 열기 안내 후 멈춘다. |
| `js/core/api.js` | 이 PC 앱 API 호출 · 연결 끊김 판정. |
| `js/core/dom.js` | DOM 도우미 — 요소 · 아이콘 · 단추 · 칸 · 다시 그릴지 판정. |
| `js/core/widgets.js` | 공용 조각 — 타일 얼굴 · 유형 칩 · 에이전트 타일 · 발화자 아이콘 · 아이콘 단추. |
| `js/core/state.js` | 화면 공유 상태 — 모듈들은 이 객체의 속성만 바꾼다. |
| `js/core/float.js` | 떠 있는 상자(메뉴 · 이름 · 대화 추가) — 열기 · 닫기 · 초점 돌려주기. |
| `js/shell.js` | 셸 — 상태 · 내 정보 · 테마 · 화면 전환(라우터) · 상단 바(+ 새로 만들기 · 프로필 메뉴 · 종). |
| `js/events.js` | 새로 고침 · 변경 신호 스트림 · 알림 소리. |
| `js/rail.js` | 레일 — 상대 · 그룹 타일, 고르기, 그룹 만들기 · 이름 바꾸기, 레일 폭. |
| `js/tiles.js` | 타일 편집(표시 이름 · 아이콘 · 아이콘 색 — 이 PC 에만) · 타일 메뉴. |
| `js/sidebar.js` | 사이드바 — 머리 · 내비 · 대화 목록 · 즐겨찾기 · 에이전트 응답 · 상대별 절(접기) · 폭 · 개수. |
| `js/home.js` | 개요 — 목록 · 미리보기 · 미리보기 크기. |
| `js/conv.js` | 대화방 — 머리 · 글 · 본문 뷰어 · 컨텍스트 · 제목 · 자동 왕복 · 관람자. |
| `js/conv-menu.js` | 대화 메뉴 — 나가기 · 기록 삭제 · 그룹 넣기. |
| `js/composer.js` | 입력창 — 보내기 · 마크다운 단추 · 멘션 후보. |
| `js/detail.js` | 세부정보 — 이 PC 실행 · 진행 · 대화 정보 · 관람자 · 요청 · 북마크. |
| `js/activity.js` | 내 활동 · 북마크 목록. |
| `js/requests.js` | 요청 — 목록 · 보드 · 보관. |
| `js/agents.js` | 에이전트 화면 — 내 에이전트 · 상대 PC 에이전트 목록. |
| `js/peers.js` | 주고받은 메시지. |
| `js/compose.js` | 세션 생성 폼. |
| `js/drag.js` | 끌어 옮기기 — 대화 줄 · 레일 타일 · 순서 저장. 손으로 놓는 순서(끌어 놓기 · Alt + ↑ / ↓)는 같은 묶음(상대 타일 · 그룹 타일 · 같은 절의 대화) 안에서만. |
| `js/search.js` | 통합 검색 — 250 ms 뒤 조회, 묶음(메시지 · 대화 · 사람 · 에이전트)별 결과 상자, 누르면 그곳으로. |
| `js/settings/index.js` | 설정 — 메뉴 나무 · 목록 + 상세 도우미 · 초대 수락 칸. |
| `js/settings/conn.js` | 설정 · 연결 — 사용자 목록 · 상세 · 초대. |
| `js/settings/groups.js` | 설정 · 그룹설정. |
| `js/settings/agents.js` | 설정 · 에이전트 — 목록 · 상세 · 등록. |
| `js/settings/prefs.js` | 설정값 — 실행 · 화면 · 시작 · 알림 칸. |
| `js/settings/debug.js` | 설정 · 디버깅. |
| `js/logic/format.js` | 형식 — 상태 글 · 날짜 · 시각 · 기간 · 조사 · 짧은 이름 · 크기. |
| `js/logic/conv.js` | 대화 — 이름 · 유형 · 목록 줄 · 절 · 메뉴 · 공유 · 대화 정보. |
| `js/logic/message.js` | 글 — 말풍선 쪽 · 작성자 · 시간 줄 · 진행 · 멘션 · 메모 · 실행 단계. |
| `js/logic/context.js` | 컨텍스트 — 크기 · 이름 · 메뉴 · 사유. |
| `js/logic/markdown.js` | 글 본문 — HTML 문서 판별 · 마크다운 블록 · 인라인. |
| `js/logic/peer.js` | 상대 — 접속 · 요약 · 안 읽음 · 색 · 주고받은 메시지. |
| `js/logic/inbox.js` | 주고받은 메시지 — 상대 절 · 줄 · 페이지. |
| `js/logic/author.js` | 글 작성자 — 이름 · 말풍선 쪽. |
| `js/logic/order.js` | 순서 — 위치 정렬 · 옮기기 · 적용. |
| `js/logic/rail.js` | 레일 · 그룹 · 순서 — 타일 · 끌기 · 그룹 소속. |
| `js/logic/home.js` | 개요 미리보기 — 크기 · 절 · 다시 읽기 · 말풍선. |
| `js/logic/request.js` | 요청 — 상태 · 이동 · 보드 · 보관. |
| `js/logic/activity.js` | 내 활동 — 거르기 탭 · 줄. |
| `js/logic/settings.js` | 설정 — 메뉴 나무 · 초안 · 진단 정보 · 모델 · effort. |
| `js/logic/agent.js` | 내 에이전트 — 실행 도구 · 옵션 블록 · 셸 확인 · 색. |
| `js/logic/shell.js` | 셸 — 주소 · 사이드 내비 · 메뉴 · 진입 토큰. |
| `js/logic/sync.js` | 동기화 — 마지막 값만 보내기 · 늦은 응답 버리기 · 한 번에 하나 · 변경 신호(SSE `event: change`) → 다시 읽을 것. |
| `js/logic/search.js` | 통합 검색 — 설정 항목 · 결과 묶음. |

## 화면 구조

D26 · D35 · S4 — 시안 v4(Nocturne) 5영역. 화면은 전부 웹UI 와 tray-icon 으로만(D30 — OS 전용 UI API 없음).

- **상단 바**: wordmark 뒤 실행 중 버전 `v{version}`(#1z-8 — `#top_version`, `showStatus` 가 `/api/status` 의 `version`) · 통합 검색(Ctrl K) · "+ 새로 만들기"(#1z-13 — 테두리 단추, 주 단추 색 아님 · 세션 생성 · 초대 생성 · 초대 수락 · 에이전트 등록 — "자동 왕복 대화" 는 없고 세션 생성 화면 안에서) · 알림 = 내 활동 ·
  프로필 메뉴. 상단 바 집중 칩은 없다.
- **메뉴 아이콘**(#1l · #1r): 사이드바 · 설정 트리 · 새로 만들기 · 타일 · 대화 · 그룹 · 컨텍스트 메뉴 · 프로필 메뉴(상태 셋은 색 점, 그 밖 user · pencil · bell · gear) ·
  내 활동 탭(bell · arrow-down · arrow-up · chat · x) · 요청 탭(arrow-down · arrow-up) 항목마다 `.ico-{이름}`. 바꾸기 · 고치기는 연필(`ico-pencil` · `PENCIL_SVG`).
- **글자 앞 아이콘**(#1z-10 결정 4 — 이미 아이콘 있는 메뉴는 그대로): 라벨 · 절 제목 · 단추 · 스위치 글자 앞 `.ico.lead`(글자색 16px · 한 칸, JS `lead(name)` ·
  `button(label, fn, cls, icon)` · `field(label, control, box, icon)` · `convAct(…, icon)`). 설정 화면 제목 = `settingsTree()` 의 메뉴 아이콘을 `renderTree` 가 붙임 · `--muted`
  (`.set-head h2 .ico.lead` — #1z-13 P4, 옛 강조색). 새 아이콘 = activity · info · clock · folder · terminal · copy · eye · code · help · send · list · grid · resize · user-plus · folder-plus · cpu ·
  sliders · layers · palette · sort · volume · refresh · file · filter · key · globe · save · folder-up · git · message(`app.css` mask SVG). 상단 "+ 새로 만들기" · 레일
  "+ 사용자" · "+ 그룹" 의 `+` 글자는 아이콘(plus · user-plus · folder-plus)으로. 자리 목록 = 아이콘 시안 매핑 표(#1z-10 plan).
- **세션 생성**(#1r · #1x — `#new`): 제목 · 이름 = `js/logic/shell.js` `SESSION_NEW` "세션 생성"(+ 메뉴 `newMenu` · 사이드바 대화 목록 + · 에이전트 줄 단추 · 빈 상대 절 단추 같은 이름).
  줄 라벨마다 아이콘(상대 user · 상대 에이전트 bot · 내 응답 에이전트 swap · 최대 응답 횟수 reset · 제목 title). 메시지 · 메시지 종류 칸은 없다(#1x).
  "내 응답 에이전트"(`chat_relay_agent` → `relay_agent`) 를 고를 때만 하위 줄 "최대 응답 횟수"(`chat_relay_cap` → `relay_cap`, 처음 값 = 설정 `relay_cap_default`)가 보인다.
  상대 칸 처음 값 = `openCompose` → `js/logic/conv.js` `composeDefaults(railPeer, peers)`(레일에서 고른 상대가 페어링됐으면 그 id, '나' · 그룹 · 아니면 "상대 선택").
  제출 "생성" = `POST /api/conversations` `{peer_id, agent, title, relay_agent?, relay_cap?, group?}`(`text` 없음 — 글 없는 대화, [an_app](an_app.md)) → `#c/{id}` · 입력창
  `reply_text` 초점. 첫 메시지(질의 · 요청)는 그 대화 입력창에서 — 상대 PC 는 그때 대화를 받고, 자동 왕복 대화는 그 첫 메시지부터 왕복한다(첫 메시지가 요청이면 400).
- **접이식 들여쓰기**(#1r · #1s): `.peer-fold` 몸(머리 아닌 자식)을 한 단(`--sp-5`, ≤ 720px `--sp-3`) 들인다 — 겹친 절(요청 개요 받은/보낸 → 상대 → 행)은 단계마다
  한 단씩. 세로 안내선은 없다(행 구분선과 격자처럼 보여 #1s 에서 지움).
- **PC 레일**: '나' · 상대 타일 · "+ 사용자" · 구분선 · 그룹 타일 · "+ 그룹" · 설정(#1z-2 — "+" 두 단추 = `.rail-btn.add-text` 글자, 보이는 글자가 접근 이름 · `title` 로 동작). 안 읽음 배지 · 접속 점. 타일 = 색 네모 + 사람(나 · 상대) · 세 사람(그룹 — 앞 가운데 + 뒤 좌우 옅은 둘) 모양(#1z-14 결정 1 · #1z-15 결정 7 — `railFace` · `PERSON_SVG` · `GROUP_SVG`,
  그림이 있으면 그림 — 줄 타일 · 프로필 · 말풍선 · 주고받은 메시지도 같은 얼굴(#1z-15 개정 ③, '나' = `railFace(…, true)` 흑연 `me-tile` — `meTint`)), 이름은 타일 아래.
  타일 색(#1z-15) = 에이전트 아이콘 색과 같은 여섯(`AGENT_COLORS`) — 상대 · 그룹은 처음 무작위로 정해 저장(`color`), '나' 는 기본 흑연(`me_color` 빈 값), 타일 편집 '아이콘 색' 단추로 바꾼다.
  `js/logic/peer.js` `tileTint(color)` = 여섯 중 하나면 `tinted {색}`, 아니면 빈 글자 — 레일 · 타일 편집 미리보기 · 줄 타일(`tileFace` · `rowTile` 의 `color`) ·
  말풍선 상대 아바타(`conv.js` `peerTint`) · 요청 의견 · 주고받은 메시지 사람 아바타(나 = `me_color` · 상대 = 그 상대) · 프로필 얼굴이 같은 규칙(`app.css` `.tinted` — 흰 글자 4.5:1). 타일 순서 설정(사용자 → 그룹 기본 · 그룹 → 사용자, 이 PC). 레일 어디서든 브라우저 우클릭 메뉴는 뜨지 않는다.
  레일 "+ 사용자" = 설정 '연결' 화면 · 상세에 초대 수락 폼 · 초점(#1z-8). 타일 크기(#1x — 설정 `ui_tile_size` `small` 기본 · `normal`, `applyTileSize` →
  `#app.tile-small`): 작게 = 아이콘 24px + 오른쪽 이름 한 줄 말줄임 · 레일 폭 `--rail-w-small`, 넓은 화면(721px 이상)만. 설정 표시 = "크게"(`normal`) · "작게"(`small`).
  레일 끌기(#1z): 레일 오른쪽 손잡이 `#rail_resize`(separator, `aria-valuenow` 72 · 168) — 끄는 동안 `#app` 의 `--rail-w` = 끈 폭(`js/logic/rail.js` `RAIL_W` 64 ~ 240) ·
  `applyTileSize(railSize(w))`(120px 이상 작게), 놓으면 `--rail-w` 를 지우고 `saveTileSize` = 72 · 168 중 그 크기 + 설정 선택 동기화 + `PUT /api/settings`. ← = 크게 · → = 작게. 720px 이하 숨김.
- **사이드바**(레일 범위 메뉴 — D35 세 절): '나' = 개요 / 내 활동 · 요청 / 즐겨찾기 · 북마크 · 에이전트를 상대별 접이식 절로('주고받은 메시지' 메뉴 없음 — 사용자 지시, 내 활동과 겹침 · `#p` 화면은 남음, 접힘 = 설정
  `ui_folded`). '에이전트'(#1v) = 링크 없는 머리 + 들여쓴 하위 "등록된 에이전트"(`#agents` — 에이전트 이름 목록) · "에이전트 응답"(#1z-2 — 옛 "에이전트 대화", **'나' 에만**: `#agents/convs` —
  상대가 연 대화(내 에이전트가 어느 사용자에게 답했는지)를 요청한 상대별 절로(절 머리 = 이름만, 대화 수 없음 — 사용자 지시), 대화 줄 그대로 · 같은 절 안 끌어 옮기기, 메뉴 수 = 그 대화들의 안 읽음 합 · `renderAgentConvs`). 상대 타일 = 개요(`#` — 그 상대 개요, 요청 위 — 사용자 지시) / 요청 / 즐겨찾기 · 북마크 · 에이전트(등록된 에이전트만) 그 상대만,
  첫 화면은 그 상대 개요 — 상대 타일로 `#agents/convs` 를 열면 '나' 로(`js/logic/rail.js` `peerLeaves`), 상대 타일 배지 = `js/logic/peer.js` `tileUnread`(에이전트 대화 `in` 안 읽음 제외). 개요 아코디언(#1z-2) = 머리 맨 앞
  상대 타일(`rowTile` — 레일 얼굴) · 대화 줄 에이전트 이름 앞 로봇(`.home-agent` · `speakerEl("agent")`). 그룹 타일 = 개요(그룹 요약) / 요청 / 즐겨찾기 · 북마크.
  개요 보기(#1z-4) = 머리 "목록 | 미리보기"(`.seg` `#home_view_list` · `#home_view_preview`, 설정 `ui_home_view` — `preview` 기본 · 시작할 때만 읽고 누르면 저장).
  미리보기(#1z-5 2차 시안 A → #1z-11 시안 C "편안함 · 종이 톤" → #1z-13 결정 6 공용 토큰) `#home_preview.pv-list` = 흰 면(`--surface`) 위 상대마다 절 `.pv-sec`(세로 목록, 절 사이 가는 선 `.pv-sec + .pv-sec`,
  넘치면 세로 스크롤) — 머리 `.pv-head` = 이름(14px `--fs-md` · `--fw-medium`) · 접속 `.pv-pres`(점 6px + 글) · "· 대화 n" · "· 안 읽음 n" · "· 대기 요청 n"(`stat pv-wait`) — 상대 타일 없음
  (머리 "{이름} 개요 →" 없음 — 그 상대 개요는 "+n개 더 보기" · 레일 타일), 절 확대 없음(#1z-11 결정 5 — 2026-10-22 "15% 확대" 개정). 대화 없는 상대 = 머리 한 줄 + 오른쪽 "세션 생성"(점선 상자 없음).
  카드 `.pv-card` = `--surface` · 1px 테두리 = 대화 유형 색(#1z-14 결정 2 — `data-kind` 에이전트 대화 `in` `--card-agent` 청록 · 내 대화 `out` `--card-mine` 남색 · 관전 `shared` `--card-watch` 호박, 올려도 그대로) · `--pv-radius` 10px · 그림자 없음, 머리 = 유형 아이콘(`.pv-kind` — 무채 선 아이콘, #1z-13 결정 15) ·
  제목 13.5px 500 · 안 읽음이면 `.pv-new` "새 글 n"(+ 카드 테두리 2px), 바닥 = `.pv-who` "{에이전트} · {시각}" · `.pv-state st-{상태}`(점 + `stateText`). 말풍선 = 12px · 줄간격 1.5 ·
  사람 글 `--subtle` 면 · 에이전트 · 실행 중 `--surface` + 1px `--border` · 오류 `--danger-soft`. 색 = 공용 토큰(#1z-13 결정 6 — 종이 톤 없음), 크기 · 간격 = 첫 `:root` 의 `--pv-*`,
  "미리보기 크기"(#1z-6 — 머리 `#home_zoom_pick` `select#ui_home_zoom`, 미리보기에서만, 설정 `ui_home_zoom` 100 기본 · 125 · 150 · 175 · 200 — 축소 없음, 시작할 때 · 고를 때만 적용(`PREFS` 밖 — 다른 설정 저장의 늦은 응답이 되돌리지 않게, #1z-6 R1), 저장은 `SendLatest`(한 번에 하나 · 마지막 값, R2) ·
  `applyHomeZoom` 이 `js/logic/home.js` `previewZoom` 의 `--pv-scale`(1 ~ 2 — 카드 최소 폭 · 본문 높이 · "+n개" 칸 높이)을 `#home_preview` 에. 말풍선은 모든 크기 2줄(#1z-8 — 사용자 "2줄까지만", #1z-6 의 줄 수 늘림 개정 ·
  `--pv-lines` 없음) — 줄 자르기는 안쪽 `.pv-text`(말풍선 안쪽 여백에 다음 줄이 비치지 않게), 말풍선 사이 `--sp-3`.
  글자 크기는 그대로, 카드가 넓어져 한 줄 열 수가 바뀌면 `refitPreview` 가 다시 잰다 — 사용자 지시로 #1z-6 의 "카드 폭 · 열 수 그대로" 를 바꿈),
  카드 격자 `.pv-cards` = `repeat(auto-fill, minmax(min(100%, calc(256px * var(--pv-scale))), 1fr))`(#1z-11 시안 C 256px — 열 수는 그만큼 준다).
  카드 `a.pv-card`(누르면 그 대화, 안 읽음이면 `.unseen` 유형 색 테두리 2px) = 머리(유형 아이콘 · 제목 · "새 글 n" `.pv-new`) · 본문 `.pv-body`(높이 `132px × --pv-scale`) = 마지막 글 `PREVIEW_MSGS`(12)를 `previewBubbles`
  말풍선 `.pv-msg`(#1z-11 결정 3 — 글자 `--fs-xs` 12px · 줄간격 `--pv-lh-msg` 1.5 · 이름 `--pv-fs-who` 10.5px, 쪽 `bubbleSide`, 이름 `authorLabel`, 글자만, `agent` · `running` · `error` 색, 요약 글 뺌 — 최신이 아래, 넘친 옛 글은 위에서 잘림) ·
  바닥 `.pv-card-foot`("{에이전트} · {시각}" `.pv-who` · 상태 `.pv-state` 점 + 글자). `previewSections(groups, single, cols)` = 카드 최근 활동 순(`last_at` 큰 것부터), '나' · 그룹은 한 줄 —
  넘치면 `cols - 1` 개 + 끝 칸 `.pv-more` "+n개 더 보기"(`pickTile(상대)` = 그 상대 개요), 상대 타일은 전부. 한 줄 열 수 `state.previewCols` = 그린 `.pv-cards` 의 실제 트랙 수
  (`fitPreviewCols` — 바뀌면 다시 그림, `ResizeObserver(#home_preview)` 가 폭 바뀜마다 다시 잼). 대화 없는 페어링 상대 = 머리 한 줄 `.pv-none` + "세션 생성".
  글은 `GET /api/conversations/{id}/messages?limit=12` 으로 보이는 카드의 대화만, 필요할 때만 다시 읽는다 — 받은 적 없음 · 요약 변화(`previewKey`) · 변경 신호가 그 대화
  (`previewStaleOf` → `state.previewStale`, 전체면 `previewAll` — 한 줄 밖 숨은 카드는 `previewStale` 로 남겨 보일 때, #1z-5 R1) · 개요에 들어옴 · 보기 바꿈 · 읽기 실패(5초 새로 고침마다 전부 읽지 않음, 서버 `messages_tail` 이 SQL 에서 자름).
  `state.previewMsgs` = `{key, ms, seq}`(앞선 차례 응답은 덮지 않음), 늦은 응답 = `previewLatest` 로 그리지 않음, 다시 그리기 서명 = 절 · 카드 · 실제 말풍선 · 열 수.
  읽음은 바꾸지 않는다. 이 PC 에서 마지막이 아닌 글만 바뀌면(신호 없음) 개요에 다시 들어올 때.
  그 아래 대화 목록 · 제목 검색. 메뉴 머리 설명은 없다(개요 카드는 안 읽음 · 대기 요청만). 상대 에이전트 줄에 "세션 생성" 단추, 내 에이전트 줄 ✎ → 설정의
  그 에이전트 상세. 사이드바 머리 부제(#1x — `js/logic/shell.js` `sideSub`) = '나' 이름(별칭이면 + 호스트) · 상대 = 호스트(별칭이면 "{이름} · {호스트}") — 지문 · "페어링 됨" ·
  접속 문구 없음(접속 = 레일 점, 레일 타일 `aria-label` 에는 남긴다). 주고받은 메시지(#1q) = 상대별 절 처음 접힘(`open:peer:{상대}` 기억) · 머리 = 타일 · 이름 · "n건" · 마지막 메시지 한 줄 · 시각, 펼치면 서버에서
  20건씩(`side=both` · `before` · `before_at` 복합 커서, 상대마다 한 줄 차례 · 5초 새로 고침은 첫 페이지 + 받은 꼬리 `mergePage`) + 끝 줄 "보인 수 / 전체" · "더 보기", 줄(#1z-2 — 시안 `Peer`, `peerRowEls`) =
  아바타(`js/logic/message.js` `avatarKind` — 사람(상대 · 나) = 레일 얼굴(그림 · 타일 색, 나 = `me_icon` · 흑연) · 에이전트 = 로봇 네모 — #1z-13 결정 12 · P1 · #1z-15 개정 ③) · 메타(이름 · "사람" 또는 `.tag.agent-tag` "에이전트" · 대화 제목 · 요청 태그) ·
  본문 · 오른쪽 시각, 에이전트 줄 바탕 `--agent-soft` · 오른쪽 북마크 아이콘(말풍선과 같은 그림 · 동작). 머리 "기간"(#1y — `#ui_peer_weeks` `.head-pick` · `data-num` → 숫자로 `PUT /api/settings`, `PREFS` 의 하나 · 항목
  `js/logic/peer.js` `peerWeeksOptions`(`PEER_WEEKS` 1 ~ 12 → "{n}주"), 바꾸면 `state.peerPages` 를 비우고 `loadPeerMessages` — 기간 밖 글은 서버가 목록 · 요약에서 뺀다, 대화 화면은 그대로. 받아 둔 꼬리도
  `mergePage(…, since)` 로 기간 안만(`peerSince`), 다시 그릴 때마다 설정을 함께 읽어 다른 창에서 바꾼 기간도 따른다 — #1y 적대적 평가 R1 · R2).
- **요청**(#1q): 메뉴로 열면 개요 — 바깥 절 "받은 요청" · "보낸 요청"(머리 = 전체 수 · 대기 · 수용 · 완료 · 반려 수, 0 은 숨김, 상태 거르기와 무관) · 받은 요청 처음 펼침 ·
  보낸 요청 처음 접힘, 안은 '나' 면 상대별 절. 탭 = 그 목록만, 눌린 탭 · 메뉴 다시 = 개요. 행 = 링크(이름 · 상태 · 시각 · 대화 / 본문) 바로 아래 글자 단추(✓ 수용 ·
  ✓ 완료 `--ok` · ✕ 반려 `--danger` · 다시 알리기 `--accent`, `button.text-btn`), 그 아래 메모 · "요청자에게 알리지 못함".
  보기 "목록 | 보드"(#1v — D39 ①, 고른 값 = `ui_folded` 의 `reqboard:me`, 이 PC): 보드는 탭 한 쪽만(개요에서 고르면 받은 요청 탭, 메뉴로 다시 열어도 탭), 열 = 대기 · 수용 · 반려 · 완료
  (`js/logic/request.js` `boardColumns` — 머리 건수 = 서버 집계), 카드 = 목록 행 그대로(상태 색 띠 없음 — 상태 = 열 제목 · 칩, #1z-13), 상태 거르기는 숨김. 받은 요청 카드를 `canDrop` 열(지금 열 말고 어디로든)에
  놓으면 카드가 그 열로 옮겨진 채 의견 칸(`noteForm`)이 열리고 확인해야 요청자에게 간다(취소면 제자리) — 보낸 요청 · 닫힌 대화는 끌 수 없다. 끌기 = 카드 왼쪽 손잡이
  (`.drag-handle` · `ico-grip`)의 포인터 끌기(`boardGrab` · `boardDrop` — 사본이 따라오고 놓을 열 강조 · 못 놓을 열 흐림). HTML 끌어 놓기(`draggable`)를 쓰지 않는다 — 브라우저
  끌기 확장이 링크 끌기에 반응한다. 카드 링크는 `draggable="false"`. 의견(`req_note` — #1z 용어 — 옛 이름 "메모")은 회색 면 상자(`.req-feedback` — 상태 색 띠 없음 · #1z-13, 줄바꿈 그대로). 의견 칸 = 3줄 textarea(200자 · `noteCount` "n / 200") ·
  취소 · 확인 44px, Enter = 확인 · Shift+Enter = 줄바꿈(`enterSends`). 줄바꿈은 상대에게 U+2028 로 보낸다(`note_to_wire` — 이전 버전 검사 통과, [an_core](an_core.md)). 글자 단추는 카드에도 남는다(키보드 길).
  archive(#1y — 이 PC 에만): 머리 `#req_archive` "archive 항목 보기"(요청 화면에 들어올 때마다 끔 — 기억하지 않음), 켜면 쪽마다 `archived=true` 목록을 더 받아 목록 · 보드
  아래 `#req_archive_list` 에 따로 된 절 "archive — 받은 요청" · "archive — 보낸 요청"(`renderReqArchive` — 접힘 열쇠 `reqarchive:{쪽}`, 처음 펼침). 행 글자 단추
  `js/logic/request.js` `reqArchiveAction`(받은 · 보낸 요청 — 완료 = "archive" · archive 됨 = "archive 해제") → `archiveReq` = `POST /api/messages/{id}/archive` 뒤 `loadRequests`.
  절 머리 `reqHead` 끝 "archive n"(전체 수에 들지 않는다).
- **내 활동**(#1q): 받은 요청 · 내 요청(지금 상태 + 메모, 상태가 바뀌면 맨 위 · 안 본 것) · 받은 답 · 실패만 — 탭 전체 · 받은 요청 · 내 요청 · 받은 답 · 실패 + 본문 검색.
  종류(#1z-3 시안 A → #1z-13 P4 무채): `activityRows` 의 `tone`(`request` · `mine` · `answer` · `failed` — 탭 `ACT_KINDS.kind` 와 같은 키, 실패한 답 · 상대 반영 실패 = `failed`) →
  줄 칩 `.tag.act-{tone}`(무채 — 실패만 `--danger` + `--danger-soft`), 탭 `#act_tabs` = 글자 탭(고른 탭 2px 밑줄, 실패 탭만 `--danger` 글자).
  상대 사람 일반 메시지 · 자동 왕복 기록은 없다.
- **대화 목록**(`js/logic/conv.js` `convListParts`): 나 · 상대 타일 = 내 대화(`dir = out` — 내가 연 대화 · 내가 연 자동 왕복)만 머리 없이 + "공유 대화"(`shared`) 머리,
  에이전트 대화(`in`)는 '나' 의 사이드바 '에이전트 응답' 화면으로(#1v · #1z-2). 그룹 타일 = "내 대화" · "에이전트 대화" · "공유 대화" 머리(제목 · 개수, 하나뿐이면 머리 없음). 끌어 옮기기는 묶음 안에서만(행 `data-section = {절}:{dir}`), 즐겨찾기 화면은 나누지 않는다. 대화 줄 = 한 줄(#1z-2) — 손잡이(#1x — `button.drag-handle` · `ico-grip`, `aria-label`
  "{제목} 옮기기", `convGrab`) · 유형 아이콘(`kindChip` — `.room-kind.kind-chip`, #1z-13 결정 15 · 16: 내 에이전트 응답 `in` = 로봇 · 내가 문의 `out` = 사람 · 공유 `shared` = 눈, 무채 — 사이드바 대화 목록 `renderConvList` 의 내 대화 줄은 아이콘 없음, #1z-14 결정 4 — `convItem` 셋째 인자 `sideList`, 즐겨찾기 · 에이전트 응답 화면은 그대로) · 제목 · 실행 중 점(`spin` — 상태 이름이 접근 이름) · 안 읽음 수 · 시각. 미리보기 · 발화자 · 칩 줄 없음(줄 `title` = 상대 · 에이전트). 줄에 ☆ 없음 — 즐겨찾기는 대화 머리 `star_btn` 만(#1x). 메뉴 '즐겨찾기' = 그 범위 ★ 대화(★ = 대화 하나, 북마크 = 글 하나 — 내부 이름 `saved`).
- **대화**: 메신저형 말풍선 — 이 PC 쪽 글(`mine`) 오른쪽 · 상대 사람 · 에이전트 글 왼쪽(아바타 — 24px 네모, 사람 = 그 상대 레일 얼굴 `peerFace`(공유 대화 원 상대 = 무채 사람) · 에이전트 = 로봇 — · 이름 · 시각) · 날짜 구분선 · 최대 폭 대화 칸의 90%, 방향
  라벨(사람 → 에이전트 · 보냄 · 받음) 없음. 말풍선(#1z-13 결정 12) = 사람 글(내 글 · 상대 사람 글) `--subtle` 면 · 에이전트 글 `--surface` 면 + 1px `--border`, 색 층 없음 ·
  옆 띠 없음(#1p), 둥글기 `--radius-bubble` 8px, 글자 대비 = `ux_literals` `token_contrast`(아래 디자인 토큰 — 78 경우 4.5:1 이상), 링크 = 전역 `a`. 뷰어 = markdown · HTML 은 sandbox iframe.
  대화 머리 = "대화 보기 / 원문 보기" · 컨텍스트 "42k / 1M" · "북마크 n"(세부정보 북마크 절) · "세부정보"(보낸 메시지 · 타임라인 탭 없음). 말풍선 우상단
  아이콘 = 북마크(넣기 · 빼기) · 복사(늘 보임), 아래 작은 단추 = 요청 상태. "다시 보내기" 없음 — 입력창에서 다시 쓴다. 입력창(#1v — 시안 A 모양, #1z 여백) = 창 폭에 맞춰 늘어남(최대 폭 없음 · 좌우 여백 `max(48px, 8%)` = 말풍선 영역과 같음 · 말풍선과의 간격 24px 는 스크롤 영역 밖 —
  `#messages` 아래 패딩 0 · `.composer` 위 여백) · 글 칸 44px 에서 글만큼 200px 까지(`fitComposer`) ·
  1px `--border-strong` · 그림자 없음(#1z-13), 바 = 서식 | 키 안내 · 질의 · 요청 세그먼트(라디오 그대로 — 형제 선택자, `:has()` 없음, 고른 칸 = 1px 고리) · 보내기(#1z-13 결정 3), 안내 문구 없음(placeholder "메시지 입력" — 요청 안내 `#ask_hint` 는 #1x 에서 지움) ·
  @멘션 후보(listbox), 요청 태그 · 받은 요청 완료 작업. 요청 말풍선 위 = "요청 · {상태}" 태그(메모 없음), 대기가 아니면 바로 아래 **의견 말풍선**(`reqFeedback` — 머리 "요청 의견 · {시각}",
  받은 쪽 사람 쪽, 상태 크게 · 의견 본문(줄바꿈 그대로) · 바뀐 시각). 사람 ↔ 에이전트 구분(#1z — 시안 A → #1z-13 P1): 사람 글 = `--subtle` 면,
  에이전트 글 = `--surface` 면 + 1px `--border` + 메타 "에이전트" 강조색 작은 글자(`.tag.agent-tag` — 바탕 없음) + 로봇 아바타(`.avatar.agent-av` — 무채 네모, 좌 · 우 어느 쪽이든). 대화 목록 =
  사이드바 `--sidebar` 바탕 위 납작한 줄(`.side-list li`, #1z-2 한 줄 · #1z-13 카드 · 유형 색 테두리 없음 — 고른 줄 = `--accent-soft` 바탕 + `--accent` 글자, P2), 내 활동 · 북마크 줄은 이름 앞에 발화자 아이콘(`speakerEl` — `js/logic/message.js` `speakerIcon`: 에이전트 = `bot` ·
  사람 = `user`, `inboxRows` · `activityRows` 의 `author`), 주고받은 메시지는 레일 얼굴 · 로봇 아바타(위). 웹UI 편집 모드 없음(#1v — D34 편집 · 삭제는 CLI, "(편집됨)" 표시는 남음).
- **대화 유형 표시**(#1z-7 → #1z-13 결정 15 · 16): 유형 아이콘(`kindChip` · `js/logic/conv.js` `convKindIcon`) = 내 에이전트 응답 `in` 로봇 · 내가 문의 `out` 사람
  (사람 ↔ 에이전트 구분 — 발화자 표시 `speakerIcon` 과 같은 그림) · 공유 `shared` 눈, 모두 무채 `--faint` 선 아이콘(`.kind-chip` — 색 면 · 종류 테두리 · 점선 · 머리 종류 선 없음, `role="img"` 이름
  `KIND_NAMES` — 공유 = "공유 대화"). 닿는 곳: 대화 머리(`#conv_kind` 18px — 제목 앞), 사이드바 대화 줄(`convItem` — 대화 · 즐겨찾기 · 에이전트 응답, 16px), 개요 목록 줄
  (`home-kind` 16px), 개요 미리보기 카드(`pv-kind` 15px), 검색 대화 줄(`s-mark` 16px). 사이드바는 절 머리('내 대화' · '에이전트 대화' · '공유 대화')도 유형을 가른다.
  줄 · 카드의 `data-kind`(= 대화 `dir`)는 남는다(개요 목록 줄 배치). 설정의 대화 관리 줄(그룹설정)과 말풍선은 그대로.
- **세부정보 패널**: 진행 · 대화 정보 · 관람자 · 요청 · 북마크(절 제목마다 아이콘 — #1z-10). 자동 왕복 절 없음(#1z-15 — 멈춤 · 재개는 대화 머리글 `#conv_relay_btn` 만,
  받은 쪽이 멈춘 대화(`relay_blocked`)는 머리글 표시 없음 — `relayText` `null`, 대신 '대화 정보' 에 '자동 왕복' 줄 `RELAY_STOPPED`).
  - **요청**(#1z-15): 최근 글부터 `js/logic/request.js` `reqFold(reqs, open)` — 접혔으면 앞 `REQ_FOLD`(3)건, 넘치면 끝 줄 단추 "더보기" ↔ "접기"(`.req-more`).
    펼침은 `reqView{conv, open}` — 다른 대화로 옮기면 접힌다.
  - **대화 정보**(#1z-10 — 옛 상대 · 에이전트 · 실행 기본값 줄 없음): `<dl id="info_rows">` 를 `renderInfo` 가 `GET /api/conversations/{id}/info` 로 채운다(대화 · 글 수 ·
    마지막 글 상태가 바뀔 때만 다시 읽음, 실패면 다음 그리기에서 다시, 응답은 마지막 읽기 것만 — `infoView.gen`). 줄 = `js/logic/conv.js` `convInfoRows(info, dir)` — 받은 대화 = 기간(clock) · 답하는 도구(bot) · 읽는 폴더(folder) · 컨텍스트(compress — 압축 n회 ·
    마지막, 비우기) · (이 PC 가 자동 왕복을 멈췄으면 — #1z-15) 자동 왕복(swap — `RELAY_STOPPED`, 대화의 `relay_blocked` 를 덧붙여 넘김) · 실행 명령(terminal — 한 줄 말줄임 + **복사** = 전체 명령 · 아래 `INFO_CMD_HINT`), 보낸 대화 · 공유 사본 = 기간 · 컨텍스트.
- **통합 검색 결과**(#1z-8 시안 S1): 상자 머리 `#search_opts` = 스위치 `#search_closed`("나간 대화 포함", `checkbox role="switch"` — 탭이 열린 동안만, 설정 아님) ·
  "지운 대화는 검색되지 않습니다", 결과는 `#search_results` 만 다시 그린다(스위치 초점 유지). 조회 `GET /api/search?q=…&closed={켬}`(서버 기본 = 닫힌 대화 뺌). 줄 = 칩(`searchMark` —
  `js/logic/search.js` `searchGroups` 의 `mark`: 메시지 = 말풍선 · 대화 = 유형 아이콘(둘 다 무채), 사람 = 그 타일(레일 얼굴), 에이전트 = 로봇(내 것 = 고른 아이콘 색 ·
  상대 것 = 무채 네모)) · 라벨 / 부제 · 닫힌 대화 태그(`tag` = `convClosed` — 공유 끝남은 응답에 없어 `state.convs` 의 같은 대화로).
- **설정**(#1z-8 시안 B): 왼쪽 메뉴 8개 평평하게(`js/logic/settings.js` `settingsTree` — 연결 · 그룹설정 · 에이전트 · 실행 · 화면 · 시작 · 알림 · 디버깅, 하위 · 누름 없음), 고른 화면만 보인다
  (`openSettings` — 화면 = `<section id="{메뉴 id}" class="set-page">`, 고른 메뉴 링크 `aria-current="page"`). 해시 `#settings/{화면}` · 별칭 `js/logic/settings.js` `SETTINGS_ALIAS`
  (`invite_make` · `invite_accept` · `peers` → 연결, `agent_new` → 에이전트), 빈 · 모르는 = 연결(사용자 "초기 화면은 연결 매뉴 선택된 상태"). 화면 머리 `.set-head` = 제목 + '?' + 머리 단추.
  본문 폭 제한 없음. 체크박스는 전부 스위치(`checkbox role="switch"` + CSS — 값 · 이벤트 · Space 그대로, 설정 밖 체크박스는 그대로).
  - 연결 · 그룹설정 · 에이전트 = 목록 · 상세 2단(`.md-split` — 목록 360px · 상세, 960px 아래 위아래로). 상세를 고르면 상자 높이 = 내용(#1z-10 결정 8 —
    `.md-detail:has(…)` `align-self: start` · `max-height: 100%`, `.md-body` `flex: 0 1 auto`): 단추 줄(`.md-foot`)이 내용 바로 아래 · 상자 바닥, 칸보다 길면 본문만 스크롤.
    고르기 전 빈 안내는 칸 전체. 고름 = `pick[화면]`(다시 그려도 유지), 줄 = `mdRow` 단추(`aria-pressed` — 고른 줄 = `--accent-soft` 바탕 + `--accent` 글자, 연필 없음 —
    #1r "고치기는 연필" 을 설정 목록에서 개정), 상세 = `mdHead`(타일 · 이름 · 부제 · ✕ "상세 닫기") · 몸 · `mdFoot`(오른쪽 끝 한 묶음 — 작업 · 끝 "닫기", #1z-13 P3). ✕ · 닫기 = `closePick` — 고름 풀기 · 초점 = 연 곳
    (목록 줄 · 머리 단추 "초대 생성" · "초대 수락" · "에이전트 등록").
  - 연결: 머리 `#window_btn`(초대 생성 — 상세에 `#invite_make` 초대 글 · 복사, 한 번에 하나 `inviteBusy`) · `#accept_btn`(초대 수락 — 상세에 `#invite_accept` 붙여 넣기 폼, 진입이 `invite_accept`
    면 열고 다른 진입이면 닫음). 목록 = 사용자(`renderConnPage` — 검색 `#peer_q` = `groupSettingsList` 의 사용자: 이름 · 별칭 · PC · 지문 · 대화 제목), 줄 = 타일 · 이름 · PC · 상태
    (`gsDot` — 접속 상태는 서명 밖, `data-peer-dot` · `data-peer-sub` 만 제자리 갱신), 상세 = `userDetail`(모델 · effort — 저장 전 값 `gsDrafts` · 상태 · 지문 · 주소 · 대화 목록 ·
    저장 · 승인 / 다시 알리기 / 연결 확인 · 내 에이전트 편집 · 삭제 · 닫기).
  - 그룹설정: 머리 `#peer_group_add`(만든 그룹을 고름). 목록 = 그룹(`renderGroupPage` — 검색 `#group_q`), 상세 = `groupDetail` — "이 그룹의 대화"(빼기) · "넣을 수 있는 대화"
    (사용자별 · 넣기 = 복사 — 다른 그룹 · 원래 자리 그대로) · 이름 바꾸기 · 삭제 · 닫기. 설정 화면의 대화 끌기는 없다(레일 · 사이드바 끌기 그대로).
  - 에이전트: 머리 `#agent_new_btn`(상세에 `#agent_new` 등록 폼 — #1z-9 시안 B 2단계: 1단계 `#agent_engine_step` 도구 타일 `#agent_engine_claude` · `#agent_engine_codex`(초점 첫 타일) →
    고르면 2단계 `#agent_form`(도구 칩 `#agent_engine_chip` · "도구 다시 고르기" · 이름 · 폴더 · 좌우 블록 `#agent_claude_opts` / `#agent_codex_opts` — 안 고른 쪽 `fieldset disabled` · 점선 ·
    `js/logic/agent.js` `agentEngineBlocks` 문구 · Codex = 스위치 `agent_skip_git` 기본 켬 · 추가 지시), 등록 = `engine` 함께(도구 없으면 보내지 않음). 등록하면 그 에이전트를 고르고 그 줄에 초점).
    목록 = 내 에이전트(로봇 타일 · 이름 · "{Claude Code|Codex} · 폴더" · "상위 탐색" · "셸" 칩, 검색 칸
    없음 — #1l), 상세 = `agentDetail` 한 격자(#1z-7 — 이름(읽기 전용) · 실행 도구 칩(읽기 전용, #1z-9) · 아이콘 색 `swatches` · 폴더 · 좌우 블록(Claude Code — 상위 한단계 탐색 · 셸 · 도구 / Codex — Git 확인 건너뜀 · 읽기 범위,
    안 쓰는 도구 블록 비활성) · 추가 지시) · 저장 · 삭제 ·
    닫기. 저장 전 값 = `agentDrafts`(id → 주인 `uid` · 폴더 · 추가 지시 · 스위치 · 색 — 다른 줄을 골랐다 와도 남고, 저장 성공 · 삭제면 지움. `gsDrafts` 와 함께 `js/logic/settings.js` `draftOf` 가 주인(에이전트 등록 세대 `uid` — 같은 이름 다시 등록 · 폴더 · 상위 · 셸 저장이면 바뀜, R2 · 상대 지문)이 다르면 버리고 `pruneDrafts` 가 목록에 없는 id 를 버린다 — 지운 뒤 같은 id 를 다시 쓴 새 프로필에 옛 초안이 붙지 않게, #1z-8 R1). 사용자 상세 "내 에이전트 편집" · '등록된 에이전트' ✎(`editAgent`
    — 그 에이전트를 고르고 그 줄에 초점).
  - 실행 · 화면 · 시작 · 알림 = 2열 타일(`.set-tiles` / `.set-tile` — 상자 없이 아래 1px 선(#1z-13), 라벨 · 한 줄 설명 왼쪽, 컨트롤 오른쪽, 960px 아래 1열). 입력 id = 설정 이름(`PREFS` 그대로). 알림 "자동 왕복" '?'
    (`tip_relay` — 위로 여는 `.tip.up`), 화면 "타일 크기"(`ui_tile_size`), 알림 "효과음"(`notify_sound`).
  - 디버깅(`loadDebug` — 화면을 열 때 · 새로 고침): 파일 카드(버전 · 데이터 폴더 · DB · 로그 — 경로 · 크기(`js/logic/format.js` `fileSize`) · 복사), 로그 상자(`GET /api/debug/log` 끝 200줄 —
    `js/logic/settings.js` `logLines` 로 수준 거르기 · 줄 앞 유닉스 초 = 이 PC 시각 "월-일 시:분:초", 끝이 보이게 아래로), "진단 정보 복사"(`js/logic/settings.js` `diagText` — 버전 · OS · 경로 · 크기 · 포트 · 열린 연결 수,
    신원 키 · 토큰 · 지문 없음). 로그 줄은 로그 그대로(자기 지문이 적힌 시작 줄 포함 — '사용자 정보' 에도 보이는 공개 식별자). 내 정보는 프로필 "사용자 정보" 팝업.
- **아이콘**: 메뉴 · 화면 제목 · 설정 트리 · ⋯ 메뉴 · "+ 새로 만들기" 이름 앞 선 아이콘(16px 한 벌).
- **프로필 · 상태**: 프로필 단추 = 24px 네모 얼굴(레일 '나' 와 같은 얼굴 — #1z-15 개정 ③, 알약 · 상태 고리 없음) + 표시 이름(12em 말줄임 · 전체는 툴팁). 얼굴 점 색 = 내 상태(초록 받기 가능 · 주황 집중 · 붉은 앱과 연결 끊김 —
  이 화면의 요청이 앱에 연달아 2번 닿지 못하면 자동, 닿으면 되돌림), 프로필 메뉴에서 받기 가능 · 집중을 고른다. 메뉴 안 켜기 항목(`menuitemcheckbox` — '알림 일시 중지') = 설정 화면 스위치와 같은 막대 + 손잡이(#1z-13 stage4-2 — `::after` 막대 · `::before` 손잡이, 꺼짐 `--faint` · 켬 `--accent`). 머리글 접속 상태.
- **화면 동작**: 다른 PC 를 고르면 열린 대화를 닫는다. 보내기는 해시의 대화 · 레일 상대가 맞을 때만. 읽음 · 내 활동 본 표시는 화면이 받은 범위(id)까지만.
  늦게 온 조회 응답은 버린다. 트레이 메뉴는 "열기" · "종료"(S4).

## 레일 그룹 · 손으로 놓는 자리

D37 · D35 — 그룹 = 이 PC 에만 있는 대화 묶음(상대 PC 로 가는 프레임 없음 — 다자 채널 아님, 저장은 [개발자 가이드](developer-guide.md) 7.2).

- 그룹의 대화 = 넣은 대화만(#1w — D37 개정, 한 대화는 여러 그룹에). 상대 단위 소속은 없다(스키마 21 이 옛 상대 소속을 그 상대의 그때 대화로 옮겼다 — 뒤에 생긴 대화는
  들어오지 않는다). 그룹을 지우면 소속만 지워진다.
- 대화 넣고 빼기 = 대화 ⋯ 메뉴 그룹 체크 · 대화 줄 손잡이를 그룹 타일에 끌어 놓기 · 그룹 "대화 추가" · 설정 그룹설정 상세의 넣기 · 빼기(#1z-8 — 넣기 = 복사).
- 순서는 정렬 기능 없이 손으로 놓은 자리(`pos` — 상대 · 그룹 · 대화, 대화는 대화마다 하나) — 끌어 놓기 · Alt + ↑ / ↓, 같은 묶음 안에서만. 레일 타일 = 포인터 끌기(#1z — `tileGrab`: 포인터 캡처, `js/logic/rail.js` `pastSlop` 5px 넘게 움직여야 끌기 · 그 미만은 누름, 사본 `rail-ghost` · 놓을 자리(#1z-2) =
  포인터가 레일 안이면 같은 묶음 타일 가운데 중 가장 가까운 것의 앞 · 뒤(`js/logic/rail.js` `railSlot` — 틈 · 타일 아래 이름(`pointer-events: none`) · 양 끝 밖도, 옛 `elementFromPoint` 는 그 자리에서
  아무 일이 없었다) → `moveItem`, 끈 뒤 click 은 먹는다, 레일 `user-select: none`, HTML 끌어 놓기 없음),
  사이드바 대화 줄 = 손잡이 포인터 끌기(#1x — `convGrab`: 포인터 캡처 · 줄 사본(`board-ghost`) · `elementFromPoint` → 같은 절 줄 = `moveItem`, 그룹 타일 = `joinGroup`,
  요청 보드 `boardGrab` 꼴 · 줄 링크 `draggable="false"` — 브라우저 끌기 확장이 반응하지 않게). 세션 생성으로 만든 대화는 맨 위,
  새 글이 와도 자리 그대로. 통합 검색 · CLI 목록은 최근 활동순 그대로.
- 타일 편집 폼(`#tile_form`) = 표시 이름(그룹 이름) · 아이콘 그림 · 아이콘 색(`#tile_colors` — `renderTileColors`, 에이전트 상세와 같은 `.swatch`, '나' 는 맨 앞 흑연 `.swatch.rail-2` 하나 더 —
  고르면 미리보기만, 저장 때 바뀐 경우만 `color` · `me_color` 를 보낸다).
- 타일 메뉴(우클릭 · 메뉴 키 · ⋯) = 타일 편집 · 이미지 지우기 · 위로 · 아래로 옮기기 · 그룹 삭제(대화는 남김). 상대 타일의 '그룹' 체크는 없다(#1w).

## 공유 대화(관람자) 화면

D38 — 통신 · 저장은 [개발자 가이드](developer-guide.md) 8.11.

- 관람자 PC 사본(`dir = shared`)은 읽기 전용 — 입력창 · 실행 · 자동 왕복 · 요청 · 다시 공유 없음, 내 활동에서 뺀다. 넣은 사람 쪽 글이 오른쪽.
- 원 상대 화면에는 관람자 수만. 끝(빼기 · 나가기 · 페어링 해제 · 24시간 닿지 않음)이면 사본에 "공유 해제됨".

## 기록 삭제 · 나가기

D40(#1w · #1x 개정) — 통신 · 저장은 [an_app](an_app.md) 경로 · 프레임.

- 대화 줄 ⋯(`.row-more`)은 줄 flex 끝 제자리(`position: static` — 떠 있으면 시각을 가린다, 사용자 지시). '나' 개요(목록 줄 · 미리보기 카드)에서 대화를 열면 `followFromHome` 이 레일을 그 상대로(`pickRail`, 사용자 지시).
- 대화 ⋯ 메뉴(대화 줄 · 대화 머리 같은 함수 `openConvMenu`) 끝 = 구분선 + `js/logic/conv.js` `convMenuItems(conv)`:
  - 공유 사본(`dir = shared`) → "나가기" 만.
  - 닫힌 대화(`convClosed` — 상대가 나감 · 1.0.x 에서 내가 나감) → "기록 삭제"(`ico-trash`, `sub` = "내 글만 삭제") · "나가기".
  - 그 밖 → "기록 삭제"(`sub` = "내 글만 삭제" · "모든 사용자 글 삭제") · "나가기"(`ico-exit`, 붉은 글자, `confirm` = `LEAVE_CONFIRM`).
  - "삭제" · 읽기 전용으로 남기는 나가기 · "이력유지" · "이력삭제" 는 없다(#1x).
- `sub` 는 그 자리에서 펼치는 하위 메뉴 — 머리 `aria-haspopup="menu"` · `aria-expanded` · 끝 "▸", 하위 항목 `button.menu-sub`(→ 펼침 · 첫 항목 초점, ← 접고 머리로).
- 고르면 확인 한 번(`convEndForm` · `END_TEXT` — 확인 단추 위험 색):
  - 나가기 "대화방이 목록에서 삭제됩니다."(단추 "나가기") → `DELETE /api/conversations/{id}`(나가며 이 PC 목록 · 글 삭제 — 1.0.x "삭제").
  - 내 글만 삭제 "이 PC 의 대화 기록을 지웁니다. 대화는 남습니다." · 모든 사용자 글 삭제 "나와 상대 PC 의 대화 기록을 모두 지웁니다. 대화는 남습니다."(단추 "삭제")
    → `POST /api/conversations/{id}/clear {all}`.
  - 지운 대화가 열려 있었으면 '개요'로. 실행 중 · 대기 글이 있으면 서버 409 "실행이 끝난 뒤 지울 수 있습니다" · "실행이 끝난 뒤 나갈 수 있습니다",
    상대 앱이 `Clear` 를 모르면 409 "상대 앱이 이 기능을 모릅니다"(상태 줄 — 누를 때 확인, 메뉴는 비활성으로 그리지 않는다: 상대 기능은 그때 `Ping` 으로만 안다).
- `POST /api/conversations/{id}/leave`(읽기 전용으로 남기는 나가기)는 CLI · API 호환만 쓴다. 세부정보 공유 "나가기"(`share_leave_btn`)도 ⋯ "나가기" 와 같은 확인 · `DELETE`(`convEndForm(conv, "leave")`).
- 닫힌 대화(`js/logic/conv.js` `convClosed` — "나감" · "상대가 나감" · "공유 해제됨")는 읽기 전용 — 입력창 · 컨텍스트 단추 · 자동 왕복 단추 · 관람자 넣기 없음, 받은 요청의 상태 바꾸기 ·
  다시 알리기 · 보드 끌기 없음(`reqMoves(m, closed)` · `reqResend` · `messageActions`). 대화 줄 태그(`convMeta.tag`) · 대화 머리 부제에 그 문구.
- 상대가 나간 대화로 보낸 글의 거절 사유 `left` = "상대가 대화를 나갔습니다"(`refusedText`).

## 컨텍스트

D19 보강 — 세션 처리는 [개발자 가이드](developer-guide.md) 6.3.

- 대화 머리 "42k / 1M" = 답하는 PC 가 답과 함께 보낸 세션 창 사용량(상한 1M 고정, 60% 주황 · 80% 붉음, 0.4.0 상대는 "알 수 없음").
- 단추 → 압축(요약 후 새 세션) · 비우기(세션만). 구분선 글(`author = 'context'`)로 양쪽에 남는다. 실행 · 압축이 걸린 대화에서는 메뉴를 끈다.

## 접속 표시

D17 개정 — 접속 판정 · 집중 거절은 [개발자 가이드](developer-guide.md) 8.5 · 8.8.

- "방금 통신"(75초 안 — 점 · 집중 표시는 이때만) · "n분 · n시간 · n일 전 통신" · "통신 기록 없음".
- 보낸 글이 거절되면 실패 상태일 때만 "상대가 집중 중" · "상대가 거절({code})".

## 알림

S9 · D27 · D29.

- 기본 OFF. 켜면 페어링 요청 · 상대 승인 · 원격 메시지를 웹UI 탭이 열려 있으면 브라우저 알림(SSE → `Notification`, OS 알림 정책 D27), 탭과 무관하게 트레이
  "새 요청" 표시. 웹UI 연결(`POST /api/seen`) · 트레이 열기 성공 · 끄기로 지운다. 켜짐 · 건수는 한 잠금으로 직렬화.
- 종류별(페어링 · 받은 요청 · 받은 답 · 자동 왕복) 켜기 · 끄기와 본문 미리보기 끄기. 자동 왕복은 기본 시작 1 + 끝 1(저절로 멈춘 것만)로 묶고, "메시지마다" ·
  "알리지 않음" 을 고를 수 있다. 자동 왕복 상한 기본값도 설정(1 ~ 50, 처음 10).
- 알림 일시 중지(이 PC) 동안은 브라우저 알림 · 트레이 표시 · 이벤트를 버리고(건수 그대로) 풀면 설정대로.
- **받은 요청 알림**(#1x — B안 · OS 알림 보강, D27 · D30 유지): 서버 알림 kind `request` — 제목 "받은 요청 {대기 수}", 본문 = 최근 대기 요청 최대 3줄 + "외 n건"
  ([an_app](an_app.md) `notify::request_notice`). `js/logic/conv.js` `requestNotice(n)` → tag `coworkagent-request`(같은 tag 라 앞 알림을 바꾼다) · 누르면 `pickRail(null)` · 요청 보드
  (`#requests`), 그 밖 kind 는 tag `coworkagent-{kind}` · 내 활동. 받은 요청 알림은 `silent` — 소리는 웹UI 효과음 `chime()`(WebAudio 두 음 880 Hz → 1320 Hz 각 0.12 초 ·
  음량 0.15, `AudioContext` 는 첫 `pointerdown` · `keydown` 에서 만든다 — autoplay 정책, 그 전에는 소리 없음). 설정 `notify_sound` 켜짐(기본) · 집중 아님일 때만.
  OS 방해 금지는 웹UI 가 알 수 없어 그때도 소리가 난다 — 끄는 길은 "효과음" 설정. 위치 · 모양은 OS 가 정한다.
- **다시 읽기는 알림이 아니라 변경 신호**(D39 ②) — SSE `event: change` 는 알림 설정 · 일시 중지 · 집중과 무관하게 온다(상대 요청 처리 · 메시지 알림 = `conv` + 이 PC
  대화 id, 페어링 = `peer`). `js/logic/sync.js` `loadersFor` 가 자원 → 로더(`CONV_LOADERS` — 연결 · 그룹 · 목록 · 열린 대화(같은 id · id 모름만) · 내 활동 · 요청 ·
  저장 · 상대 메시지 · 실행 · 건수)를 고르고, 빈 · 모르는 종류 · `peer` 면 전체. 50ms 안의 신호는 `mergeChanges` 로 한 번. 알림 처리기는 상태 줄 · 브라우저
  알림만. 5초 전체 새로 고침은 그대로(이 PC 변경 · 시간이 지나 꺼지는 접속 표시용).

## 디자인 토큰

#1z-13 Graphite(결정 12 — 흑연 무채색 + 청록 강조 하나, 옛 #1p 시안 A 먹선 · 쪽빛) — 정본은 `app.css` 첫 `:root`. 색 · 글꼴 · 글꼴 크기 · 굵기 · 줄높이 · 간격 · 둥글기는 `:root` 밖에서 `var(--…)` 만
쓴다 — `tests/css_tokens.rs` 가 막는다(예외 `0` · `auto` · `1px`(테두리만) · `-1px`(`.sr-only` 한 곳) · `50%` · `inherit` · `line-height: 1` · `@font-face` · `url(…)` 안 —
`var(--이름, 대체)` 의 대체 값도 검사, 첫 `:root` 밖 사용자 속성은 `var(…)` · `url(…)` 만). 이름 그대로가 라이트, 다크는 `--dark-*` 를 `:root[data-theme="dark"]` 가 대입한다(기본 테마 라이트 — #1x, `index.html`
`data-theme="light"` · 저장 값이 없으면 `light`).

| 묶음 | 토큰 | 값(라이트 / 다크) |
|---|---|---|
| 바탕 | `--bg` · `--surface` · `--surface-2` · `--sidebar` · `--subtle` · `--border` · `--border-strong` | 흑연 무채 — 바탕 #F6F6F7 / #111113 · 면 #FFFFFF / #18181B · 올림 #F0F0F1 / #26262A · 옅은 면 #F4F4F5 / #1F1F23 · 선 #E4E4E7 / #2E2E33 · 단추 · 입력창 선 #D4D4D8 / #3F3F46 |
| 글자 | `--text` · `--muted` · `--faint` | #18181B · #52525B · #6B6B73 / #E4E4E7 · #A1A1AA · #8E8E97 |
| 주 단추 | `--primary` · `--primary-hover` · `--on-primary` | 흑연 #27272A / 밝은 회색 #E4E4E7 — `button.primary` 만, 강조색이 아니다(#1z-13 결정 5) |
| 강조 | `--accent` · `--accent-hover` · `--accent-soft` · `--on-accent` | 차분한 청록 #2F6F6B / #6FB3AC(채도 45 % 아래) — 링크 · 선택 · 초점 · 안 읽음 · 스위치에만 |
| 상태 | `--agent` · `--run` · `--ok` · `--danger` · `--wait` · `--on-danger` | `--agent` = 강조색을 따름, 그 밖 차분한 색 — 글자/바탕 4.5:1 이상 |
| 미리보기 카드 테두리 | `--card-agent` · `--card-mine` · `--card-watch` | 에이전트 대화 = `--agent` · 내 대화 #46607E / #8AA4C4 · 관전 = `--run`(#1z-14 결정 2 — 강조 하나의 예외, 바탕 대비 3:1 이상) |
| 레일 · 얼굴 | `--rail` · `--rail-2` · `--rail-text` · `--on-dark` · `--avatar-bg` · `--avatar-text` · `--rail-w-small` | 레일은 두 테마 모두 어둡다, 레일 타일 · 얼굴 네모 = 무채(#E4E4E7 · #3F3F46 / #2E2E33 · #D4D4D8) · '나' = `--rail-2` + `--on-dark`, 작은 타일 레일 폭 168px(#1x) |
| 에이전트 · 타일 아이콘 색 | `--agent-tile` · `--peer-1 ~ 5` | 사용자가 고르는 6색(이름이 저장됨 — `AGENT_COLORS`), 흰 아이콘 · 얼굴 4.5:1 이상 — 에이전트 · 나 · 상대 · 그룹 타일(#1z-15 `tileTint`) |
| 글꼴 | `--font` · `--mono` | IBM Plex Sans KR(실행 파일 포함 — `/fonts/`) · 고정폭은 시스템(코드 · 키 · 지문 · 원문 · 숫자만) |
| 크기 · 굵기 · 줄높이 | `--fs-xs ~ xl` · `--fw-regular/medium/bold` · `--lh-body` · `--lh-tight` | 12 · 13 · 14 · 16 · 20 · 400 · 500 · 600 · 1.6 · 1.3 |
| 간격 | `--sp-1 ~ 7` · `--gap` · `--control-h` | 4 · 8 · 12 · 16 · 24 · 32 · 40(4pt) — 그 밖은 `calc` 로 토큰을 더한다 |
| 둥글기 | `--radius-sm` · `--radius` · `--radius-lg` · `--radius-bubble` · `--radius-pill` | 4 · 6 · 8 · 8 · 999 — 얼굴 · 아바타 · 레일 타일 = 4 |
| 그 밖 | `--shadow` · `--backdrop` · `--pv-*` | 작은 그림자(팝오버 · 메뉴 · 끄는 줄만) · 대화 상자 뒤 · 개요 미리보기 크기 · 간격(색 없음) |

- **AI 티로 쓰지 않는 것**(1p `design/tokens.json` `slop` — `gates.py design`): 말풍선 옆 띠 · 선택 안쪽 띠 · 점선 상자(끌어 놓기 표시는 예외) · 숫자 테두리 상자 ·
  절 제목 자간 · 단추의 글자 기호 아이콘(✎ ⋯ → 선 아이콘 `ico-pencil` · `ico-more`). #1z-13(stitch-design-taste 목록 + 사용자 지목 — `design/taste.py` 가 센다):
  파란 강조 · 채도 높은 강조 · 사람 아이콘 아바타(사람 모양 그림 · 사람 선 아이콘 원 — 머리글자 네모로, #1z-15 개정 ③ 이 네모 안 레일 얼굴 `railFace` 로 바꿈) · 강조색으로 채운 주 단추 · 흰 카드 줄(목록 줄은 납작하게 · 선으로) ·
  범주 색(종류 · 상대마다 다른 색 면 · 테두리) · 색 띠(옆 · 위 — 상태 색 포함, 요청 보드 · 의견 상자 stage4-1) · 그라데이션 · 순검정 · 이모지 · 고른 줄 막대.
- **고정폭**은 `.count` · `.fp` · `.kbd` · `.md code` · `.mono` · `.raw` · `.steps .at` · `kbd` 에만(한글이 드는 칩 · 부제 · 에이전트 이름 · 실행 메타 · 안내는 본문 글꼴,
  숫자는 `font-variant-numeric: tabular-nums`). 한글이 든 숫자 표시("안 읽음 n")는 `.stat`.
- **대비** — 글자/바탕 짝(두 테마) · 주 단추 · 얼굴 네모 · 에이전트 아이콘 흰 글자를 `tests/ux_literals.rs` `token_contrast` 가 첫 `:root` 로 계산한다
  (78 경우 4.5:1 — #1z-13, 상태 색은 올림 바탕 · 칩 바탕 위도, 옛 1p `gates.py contrast`). 연결 끊김에도 프로필 얼굴 색은 그대로(상태는 점 색).
- **글꼴 파일** — `crates/app/web/fonts/`(IBM 배포 woff2 그대로 · SIL OFL 1.1 · `LICENSE.txt`), `server.rs` `FONTS` 가 `include_bytes` 로 넣고 토큰 없이 1년 캐시로
  준다. 파일을 바꾸면 이름을 바꾼다.

## 문구 규칙

#1p — 화면 한글 문구(이름 · 설명 · 빈 화면 · 알림)를 더하거나 고칠 때. 1p 는 웹 화면(`crates/app/web/`)만 바꿨다 — 적용 기록은 1p
`plan-copy.md`, CLI · 트레이 · 서버가 만드는 문구(`cli.rs` · `tray.rs` · `notify.rs` · `chat.rs`)는 그대로다.

- **이름**(메뉴 · 단추 · 제목 · 라벨 · 툴팁)은 명사 하나 — "—" · " · " 로 부연을 잇지 않는다. 부연은 설명으로.
- **한 개념 한 이름** — 아래 용어 표.
- **설명**은 한 문장 · 합니다체 · "무엇을 하면 무엇이 된다" 만(기능 나열 · 이유 풀이 없음). 내용이 곧 설명인 메뉴는 설명이 없다. 보안 사실(초대
  자동 등록 · 지문 확인 · 읽기 전용 · 사용량)은 줄여도 남긴다.
- **빈 화면**은 "{무엇} 없음" 한 줄.
- **placeholder** 가 라벨과 같은 말이면 지운다 — 보이는 라벨을 빼면 `aria-label` 로 이름을 남긴다.
- **영어**는 기술 용어(모델 이름 · effort · markdown · PC · CLI · JSON)만.
- **알림 · 확인 문장**은 합니다체("-했다" → "-했습니다"). 상태 줄 · 툴팁의 부연은 " · " 로 잇는다("—" 대신).
- 잠금 — `tests/ux_literals.rs`(잠근 문구 있음 · 바꾼 옛 꼴 없음) · `view.test.mjs`(화면 함수 문구).

## 용어 표

#1p 용어 결정(T1 ~ T7). 화면 문구에 "쓰지 않는 말" 이 있으면 1p `gates.py terms` 가 잡는다(부분 문자열 — 그래서 "글" 하나가 아니라 묶음 말로
적는다). 사용자 가이드 · README 도 "쓰는 말" 을 따르고, 코드 이름 · 개발 문서의 내부 이름(`사람 글` · `saved` 등)은 그대로다.

| 개념 | 쓰는 말 | 쓰지 않는 말 |
|---|---|---|
| 대화 오른쪽 패널 | 세부정보 | 실행 상세 |
| 북마크한 메시지 모음 | 북마크 | 북마크 항목 · 북마크한 글 |
| 상대가 연 대화 묶음 | 에이전트 대화 | agent 대화 |
| 대화 목록 · 그 절 | 대화 | 대화방 |
| 설정 화면 | 설정 | 환경설정 |
| 받는 쪽 알림 켜기 | 알림 받기 | 인바운드 |
| 대화 속 한 건 | 메시지 | 글 종류 · 사람 글 · 주고받은 글 · 보낸 글 · 받은 글 · 내 글 · 상대 글 · 첫 글 · 글마다 |

예외(#1x) — 사용자가 글자 그대로 준 화면 문구는 표보다 앞선다: 나가기 확인 "대화방이 목록에서 삭제됩니다." · 기록 삭제 하위 "내 글만 삭제" · "모든 사용자 글 삭제"(사용자 2026-10-20).
#1y — "기간" · "archive" · "archive 항목 보기" · "archive 해제"(intent 의 사용자 말 그대로 — 영어 "archive" 를 옮기지 않는다).

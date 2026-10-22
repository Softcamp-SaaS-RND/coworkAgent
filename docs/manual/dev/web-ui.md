# 웹UI 화면 규칙

> 모듈 매뉴얼 형식(개요 / 공개 계약 / 사용 예)의 **예외** — 화면 규칙 문서다([manual/README](../README.md) 등록 규약). `crates/app/web/`
> (`index.html` · `app.css` · `app.js` · `view.js`)를 고칠 때만 연다. API · 화면 함수 계약은 [an_app](an_app.md), 통신 · 저장 흐름은
> [개발자 가이드](developer-guide.md), 결정 번호(D · S)는 masterPlan. 지켜야 하는 제품 규칙 요약은 루트 `PRODUCT.md` `## 도메인 규칙`.

## 화면 구조

D26 · D35 · S4 — 시안 v4(Nocturne) 5영역. 화면은 전부 웹UI 와 tray-icon 으로만(D30 — OS 전용 UI API 없음).

- **상단 바**: 통합 검색(Ctrl K) · "+ 새로 만들기"(세션 생성 · 초대 생성 · 초대 수락 · 에이전트 등록 — "자동 왕복 대화" 는 없고 세션 생성 화면 안에서) · 알림 = 내 활동 ·
  프로필 메뉴. 상단 바 집중 칩은 없다.
- **메뉴 아이콘**(#1l · #1r): 사이드바 · 설정 트리 · 새로 만들기 · 타일 · 대화 · 그룹 · 컨텍스트 메뉴 · 프로필 메뉴(상태 셋은 색 점, 그 밖 user · pencil · bell · gear) ·
  내 활동 탭(bell · arrow-down · arrow-up · chat · x) · 요청 탭(arrow-down · arrow-up) 항목마다 `.ico-{이름}`. 바꾸기 · 고치기는 연필(`ico-pencil` · `PENCIL_SVG`).
- **세션 생성**(#1r · #1x — `#new`): 제목 · 이름 = `view.SESSION_NEW` "세션 생성"(+ 메뉴 `newMenu` · 사이드바 대화 목록 + · 에이전트 줄 단추 · 빈 상대 절 단추 같은 이름).
  줄 라벨마다 아이콘(상대 user · 상대 에이전트 bot · 내 응답 에이전트 swap · 최대 응답 횟수 reset · 제목 title). 메시지 · 메시지 종류 칸은 없다(#1x).
  "내 응답 에이전트"(`chat_relay_agent` → `relay_agent`) 를 고를 때만 하위 줄 "최대 응답 횟수"(`chat_relay_cap` → `relay_cap`, 처음 값 = 설정 `relay_cap_default`)가 보인다.
  상대 칸 처음 값 = `openCompose` → `view.composeDefaults(railPeer, peers)`(레일에서 고른 상대가 페어링됐으면 그 id, '나' · 그룹 · 아니면 "상대 선택").
  제출 "생성" = `POST /api/conversations` `{peer_id, agent, title, relay_agent?, relay_cap?, group?}`(`text` 없음 — 글 없는 대화, [an_app](an_app.md)) → `#c/{id}` · 입력창
  `reply_text` 초점. 첫 메시지(질의 · 요청)는 그 대화 입력창에서 — 상대 PC 는 그때 대화를 받고, 자동 왕복 대화는 그 첫 메시지부터 왕복한다(첫 메시지가 요청이면 400).
- **접이식 들여쓰기**(#1r · #1s): `.peer-fold` 몸(머리 아닌 자식)을 한 단(`--sp-5`, ≤ 720px `--sp-3`) 들인다 — 겹친 절(요청 개요 받은/보낸 → 상대 → 행)은 단계마다
  한 단씩. 세로 안내선은 없다(행 구분선과 격자처럼 보여 #1s 에서 지움).
- **PC 레일**: '나' · 상대 타일 · "+ 사용자" · 구분선 · 그룹 타일 · "+ 그룹" · 설정(#1z-2 — "+" 두 단추 = `.rail-btn.add-text` 글자, 보이는 글자가 접근 이름 · `title` 로 동작). 안 읽음 배지 · 접속 점. 타일 A안 = 사람(나 · 상대) · 사람 + 그림자(그룹) 아이콘
  (그림이 있으면 그림), 이름은 타일 아래. 타일 순서 설정(사용자 → 그룹 기본 · 그룹 → 사용자, 이 PC). 레일 어디서든 브라우저 우클릭 메뉴는 뜨지 않는다.
  레일 "+ 사용자" = 설정 '연결' 처럼 페어링 카드 맨 위 · 수락 칸 펼침 · 초점. 타일 크기(#1x — 설정 `ui_tile_size` `small` 기본 · `normal`, `applyTileSize` →
  `#app.tile-small`): 작게 = 아이콘 24px + 오른쪽 이름 한 줄 말줄임 · 레일 폭 `--rail-w-small`, 넓은 화면(721px 이상)만. 설정 표시 = "크게"(`normal`) · "작게"(`small`).
  레일 끌기(#1z): 레일 오른쪽 손잡이 `#rail_resize`(separator, `aria-valuenow` 72 · 168) — 끄는 동안 `#app` 의 `--rail-w` = 끈 폭(`view.js` `RAIL_W` 64 ~ 240) ·
  `applyTileSize(railSize(w))`(120px 이상 작게), 놓으면 `--rail-w` 를 지우고 `saveTileSize` = 72 · 168 중 그 크기 + 설정 선택 동기화 + `PUT /api/settings`. ← = 크게 · → = 작게. 720px 이하 숨김.
- **사이드바**(레일 범위 메뉴 — D35 세 절): '나' = 개요 / 내 활동 · 요청 / 즐겨찾기 · 북마크 · 에이전트를 상대별 접이식 절로('주고받은 메시지' 메뉴 없음 — 사용자 지시, 내 활동과 겹침 · `#p` 화면은 남음, 접힘 = 설정
  `ui_folded`). '에이전트'(#1v) = 링크 없는 머리 + 들여쓴 하위 "등록된 에이전트"(`#agents` — 에이전트 이름 목록) · "에이전트 응답"(#1z-2 — 옛 "에이전트 대화", **'나' 에만**: `#agents/convs` —
  상대가 연 대화(내 에이전트가 어느 사용자에게 답했는지)를 요청한 상대별 절로(절 머리 = 이름만, 대화 수 없음 — 사용자 지시), 대화 줄 그대로 · 같은 절 안 끌어 옮기기, 메뉴 수 = 그 대화들의 안 읽음 합 · `renderAgentConvs`). 상대 타일 = 개요(`#` — 그 상대 개요, 요청 위 — 사용자 지시) / 요청 / 즐겨찾기 · 북마크 · 에이전트(등록된 에이전트만) 그 상대만,
  첫 화면은 그 상대 개요 — 상대 타일로 `#agents/convs` 를 열면 '나' 로(`view.peerLeaves`), 상대 타일 배지 = `view.tileUnread`(에이전트 대화 `in` 안 읽음 제외). 개요 아코디언(#1z-2) = 머리 맨 앞
  사용자 아이콘(`rowTile` — 그 상대 타일 모양 · 색) · 대화 줄 에이전트 이름 앞 로봇(`.home-agent` · `speakerEl("agent")`). 그룹 타일 = 개요(그룹 요약) / 요청 / 즐겨찾기 · 북마크.
  개요 보기(#1z-4) = 머리 "목록 | 미리보기"(`.seg` `#home_view_list` · `#home_view_preview`, 설정 `ui_home_view` — `preview` 기본 · 시작할 때만 읽고 누르면 저장).
  미리보기(#1z-5 2차 시안 A) `#home_preview.pv-list` = 상대마다 절 `.pv-sec`(세로 목록, 넘치면 세로 스크롤) — 머리 `.pv-head` = 아이콘(`rowTile`) · 이름 · 접속 · "· 대화 n" ·
  안 읽음 · 대기 요청(머리 "{이름} 개요 →" 없음 — 그 상대 개요는 "+n개" · 레일 타일), 절마다 `zoom: var(--pv-zoom)`(1.15 — 사용자 "15% 정도 확대"),
  "미리보기 크기"(#1z-6 — 머리 `#home_zoom_pick` `select#ui_home_zoom`, 미리보기에서만, 설정 `ui_home_zoom` 100 기본 · 125 · 150 · 175 · 200 — 축소 없음, 시작할 때 · 고를 때만 적용(`PREFS` 밖 — 다른 설정 저장의 늦은 응답이 되돌리지 않게, #1z-6 R1), 저장은 `SendLatest`(한 번에 하나 · 마지막 값, R2) ·
  `applyHomeZoom` 이 `view.previewZoom` 의 `--pv-scale`(1 ~ 2 — 카드 최소 폭 · 본문 높이 · "+n개" 칸 높이) · `--pv-lines`(2 · 3(125 · 150) · 4(175 · 200) — 말풍선 줄 수)를 `#home_preview` 에.
  글자 크기는 그대로, 카드가 넓어져 한 줄 열 수가 바뀌면 `refitPreview` 가 다시 잰다 — 사용자 지시로 #1z-6 의 "카드 폭 · 열 수 그대로" 를 바꿈),
  카드 격자 `.pv-cards` = `repeat(auto-fill, minmax(min(100%, calc(240px * var(--pv-scale))), 1fr))`(확대된 단위 — 열 수는 그만큼 준다).
  카드 `a.pv-card`(누르면 그 대화, 안 읽음이면 `.unseen` 테두리) = 머리(제목 · 상태 `.pv-state`) · 본문 `.pv-body`(높이 `120px × --pv-scale`) = 마지막 글 `PREVIEW_MSGS`(12)를 `previewBubbles`
  말풍선 `.pv-msg`(글자 `--fs-2xs` 11px · 이름 `--fs-3xs` 10px — 사용자 지시로 축소, 쪽 `bubbleSide`, 이름 `authorLabel`, 글자만 · `--pv-lines` 줄, `agent` · `running` · `error` 색, 요약 글 뺌 — 최신이 아래, 넘친 옛 글은 위에서 잘림) ·
  바닥 `.pv-card-foot`(로봇 · 에이전트 · 시각 · 안 읽음). `previewSections(groups, single, cols)` = 카드 최근 활동 순(`last_at` 큰 것부터), '나' · 그룹은 한 줄 —
  넘치면 `cols - 1` 개 + 끝 칸 `.pv-more` "+n개"(`pickTile(상대)` = 그 상대 개요), 상대 타일은 전부. 한 줄 열 수 `state.previewCols` = 그린 `.pv-cards` 의 실제 트랙 수
  (`fitPreviewCols` — 바뀌면 다시 그림, `ResizeObserver(#home_preview)` 가 폭 바뀜마다 다시 잼). 대화 없는 페어링 상대 = 점선 한 줄 `.pv-none` + "세션 생성".
  글은 `GET /api/conversations/{id}/messages?limit=12` 으로 보이는 카드의 대화만, 필요할 때만 다시 읽는다 — 받은 적 없음 · 요약 변화(`previewKey`) · 변경 신호가 그 대화
  (`previewStaleOf` → `state.previewStale`, 전체면 `previewAll` — 한 줄 밖 숨은 카드는 `previewStale` 로 남겨 보일 때, #1z-5 R1) · 개요에 들어옴 · 보기 바꿈 · 읽기 실패(5초 새로 고침마다 전부 읽지 않음, 서버 `messages_tail` 이 SQL 에서 자름).
  `state.previewMsgs` = `{key, ms, seq}`(앞선 차례 응답은 덮지 않음), 늦은 응답 = `previewLatest` 로 그리지 않음, 다시 그리기 서명 = 절 · 카드 · 실제 말풍선 · 열 수.
  읽음은 바꾸지 않는다. 이 PC 에서 마지막이 아닌 글만 바뀌면(신호 없음) 개요에 다시 들어올 때.
  그 아래 대화 목록 · 제목 검색. 메뉴 머리 설명은 없다(개요 카드는 안 읽음 · 대기 요청만). 상대 에이전트 줄에 "세션 생성" 단추, 내 에이전트 줄 ✎ → 설정의
  그 에이전트 상세. 사이드바 머리 부제(#1x — `view.sideSub`) = '나' 이름(별칭이면 + 호스트) · 상대 = 호스트(별칭이면 "{이름} · {호스트}") — 지문 · "페어링 됨" ·
  접속 문구 없음(접속 = 레일 점, 레일 타일 `aria-label` 에는 남긴다). 주고받은 메시지(#1q) = 상대별 절 처음 접힘(`open:peer:{상대}` 기억) · 머리 = 타일 · 이름 · "n건" · 마지막 메시지 한 줄 · 시각, 펼치면 서버에서
  20건씩(`side=both` · `before` · `before_at` 복합 커서, 상대마다 한 줄 차례 · 5초 새로 고침은 첫 페이지 + 받은 꼬리 `mergePage`) + 끝 줄 "보인 수 / 전체" · "더 보기", 줄(#1z-2 — 시안 `Peer`, `peerRowEls`) =
  원형 아바타(`view.avatarKind` — 상대 사람 `--recv-bg` · 나 `--accent-soft` · 에이전트 `--agent-tile` 로봇) · 메타(이름 · "사람" 또는 `.tag.agent-tag` "에이전트" · 대화 제목 · 요청 태그) ·
  본문 · 오른쪽 시각, 에이전트 줄 바탕 `--agent-soft` · 오른쪽 북마크 아이콘(말풍선과 같은 그림 · 동작). 머리 "기간"(#1y — `#ui_peer_weeks` `.head-pick` · `data-num` → 숫자로 `PUT /api/settings`, `PREFS` 의 하나 · 항목
  `view.peerWeeksOptions`(`PEER_WEEKS` 1 ~ 12 → "{n}주"), 바꾸면 `state.peerPages` 를 비우고 `loadPeerMessages` — 기간 밖 글은 서버가 목록 · 요약에서 뺀다, 대화 화면은 그대로. 받아 둔 꼬리도
  `mergePage(…, since)` 로 기간 안만(`peerSince`), 다시 그릴 때마다 설정을 함께 읽어 다른 창에서 바꾼 기간도 따른다 — #1y 적대적 평가 R1 · R2).
- **요청**(#1q): 메뉴로 열면 개요 — 바깥 절 "받은 요청" · "보낸 요청"(머리 = 전체 수 · 대기 · 수용 · 완료 · 반려 수, 0 은 숨김, 상태 거르기와 무관) · 받은 요청 처음 펼침 ·
  보낸 요청 처음 접힘, 안은 '나' 면 상대별 절. 탭 = 그 목록만, 눌린 탭 · 메뉴 다시 = 개요. 행 = 링크(이름 · 상태 · 시각 · 대화 / 본문) 바로 아래 글자 단추(✓ 수용 ·
  ✓ 완료 `--ok` · ✕ 반려 `--danger` · 다시 알리기 `--accent`, `button.text-btn`), 그 아래 메모 · "요청자에게 알리지 못함".
  보기 "목록 | 보드"(#1v — D39 ①, 고른 값 = `ui_folded` 의 `reqboard:me`, 이 PC): 보드는 탭 한 쪽만(개요에서 고르면 받은 요청 탭, 메뉴로 다시 열어도 탭), 열 = 대기 · 수용 · 반려 · 완료
  (`view.js` `boardColumns` — 머리 건수 = 서버 집계), 카드 = 목록 행 그대로, 상태 거르기는 숨김. 받은 요청 카드를 `canDrop` 열(지금 열 말고 어디로든)에
  놓으면 카드가 그 열로 옮겨진 채 의견 칸(`noteForm`)이 열리고 확인해야 요청자에게 간다(취소면 제자리) — 보낸 요청 · 닫힌 대화는 끌 수 없다. 끌기 = 카드 왼쪽 손잡이
  (`.drag-handle` · `ico-grip`)의 포인터 끌기(`boardGrab` · `boardDrop` — 사본이 따라오고 놓을 열 강조 · 못 놓을 열 흐림). HTML 끌어 놓기(`draggable`)를 쓰지 않는다 — 브라우저
  끌기 확장이 링크 끌기에 반응한다. 카드 링크는 `draggable="false"`. 의견(`req_note` — #1z 용어 — 옛 이름 "메모")은 상태 색 띠 상자(`.req-feedback`, 줄바꿈 그대로). 의견 칸 = 3줄 textarea(200자 · `noteCount` "n / 200") ·
  취소 · 확인 44px, Enter = 확인 · Shift+Enter = 줄바꿈(`enterSends`). 줄바꿈은 상대에게 U+2028 로 보낸다(`note_to_wire` — 이전 버전 검사 통과, [an_core](an_core.md)). 글자 단추는 카드에도 남는다(키보드 길).
  archive(#1y — 이 PC 에만): 머리 `#req_archive` "archive 항목 보기"(요청 화면에 들어올 때마다 끔 — 기억하지 않음), 켜면 쪽마다 `archived=true` 목록을 더 받아 목록 · 보드
  아래 `#req_archive_list` 에 따로 된 절 "archive — 받은 요청" · "archive — 보낸 요청"(`renderReqArchive` — 접힘 열쇠 `reqarchive:{쪽}`, 처음 펼침). 행 글자 단추
  `view.reqArchiveAction`(받은 · 보낸 요청 — 완료 = "archive" · archive 됨 = "archive 해제") → `archiveReq` = `POST /api/messages/{id}/archive` 뒤 `loadRequests`.
  절 머리 `reqHead` 끝 "archive n"(전체 수에 들지 않는다).
- **내 활동**(#1q): 받은 요청 · 내 요청(지금 상태 + 메모, 상태가 바뀌면 맨 위 · 안 본 것) · 받은 답 · 실패만 — 탭 전체 · 받은 요청 · 내 요청 · 받은 답 · 실패 + 본문 검색.
  종류 색(#1z-3 시안 A): `activityRows` 의 `tone`(`request` · `mine` · `answer` · `failed` — 탭 `ACT_KINDS.kind` 와 같은 키, 실패한 답 · 상대 반영 실패 = `failed`) →
  줄 칩 `.tag.act-{tone}`(옅은 바탕 + 진한 글자: 받은 글 `--recv` · 보낸 글 `--sent` · 에이전트 `--agent` + `--answer-bg` · 오류 `--danger` + `--danger-soft`),
  탭 `#act_tabs a[data-kind]` 는 같은 색 글자 · 고른 탭은 그 바탕(전체는 무채색).
  상대 사람 일반 메시지 · 자동 왕복 기록은 없다.
- **대화 목록**(`view.js` `convListParts`): 나 · 상대 타일 = 내 대화(`dir = out` — 내가 연 대화 · 내가 연 자동 왕복)만 머리 없이 + "공유 대화"(`shared`) 머리,
  에이전트 대화(`in`)는 '나' 의 사이드바 '에이전트 응답' 화면으로(#1v · #1z-2). 그룹 타일 = "내 대화" · "에이전트 대화" · "공유 대화" 머리(제목 · 개수, 하나뿐이면 머리 없음). 끌어 옮기기는 묶음 안에서만(행 `data-section = {절}:{dir}`), 즐겨찾기 화면은 나누지 않는다. 대화 줄 = 한 줄(#1z-2) — 손잡이(#1x — `button.drag-handle` · `ico-grip`, `aria-label`
  "{제목} 옮기기", `convGrab`) · 유형 아이콘(`kindChip` — `.room-av.agent-av.kind-chip`, #1z-7: 내 에이전트 응답 `in` = 로봇 · 내가 문의 `out` = 사람 · 공유 사본 = 없음) · 제목 · 실행 중 점(`spin` — 상태 이름이 접근 이름) · 안 읽음 수 · 시각. 미리보기 · 발화자 · 칩 줄 없음(줄 `title` = 상대 · 에이전트). 줄에 ☆ 없음 — 즐겨찾기는 대화 머리 `star_btn` 만(#1x). 메뉴 '즐겨찾기' = 그 범위 ★ 대화(★ = 대화 하나, 북마크 = 글 하나 — 내부 이름 `saved`).
- **대화**: 메신저형 말풍선 — 이 PC 쪽 글(`mine`) 오른쪽 · 상대 사람 · 에이전트 글 왼쪽(아바타 · 이름 · 시각) · 날짜 구분선 · 최대 폭 대화 칸의 90%, 방향
  라벨(사람 → 에이전트 · 보냄 · 받음) 없음. 말풍선 = 가는 선 1px + 색 층(내 글 강조색 · 상대 글 그 상대 타일 색, 불투명도 `--bubble-mine-alpha` ·
  `--bubble-theirs-alpha` — 시안 A 는 0.1 · 0, 0 = 층 없음), 옆 띠 없음(#1p — `color-mix` 는 쓰지 않는다 — macOS 12 Safari 15), 글자 대비 36 경우 모두 4.5:1 이상(`ux_literals`
  `bubble_contrast`), 내 글 링크 `--sent`. 뷰어 = markdown · HTML 은 sandbox iframe.
  대화 머리 = "대화 보기 / 원문 보기" · 컨텍스트 "42k / 1M" · "북마크 n"(세부정보 북마크 절) · "세부정보"(보낸 메시지 · 타임라인 탭 없음). 말풍선 우상단
  아이콘 = 북마크(넣기 · 빼기) · 복사(늘 보임), 아래 작은 단추 = 요청 상태. "다시 보내기" 없음 — 입력창에서 다시 쓴다. 입력창(#1v — 시안 A 모양, #1z 여백) = 창 폭에 맞춰 늘어남(최대 폭 없음 · 좌우 여백 `max(48px, 8%)` = 말풍선 영역과 같음 · 말풍선과의 간격 24px 는 스크롤 영역 밖 —
  `#messages` 아래 패딩 0 · `.composer` 위 여백) · 글 칸 44px 에서 글만큼 200px 까지(`fitComposer`) ·
  옅은 그림자, 바 = 서식 | 질의 · 요청 세그먼트(라디오 그대로 — 형제 선택자, `:has()` 없음) | 키 안내 | 보내기, 안내 문구 없음(placeholder "메시지 입력" — 요청 안내 `#ask_hint` 는 #1x 에서 지움) ·
  @멘션 후보(listbox), 요청 태그 · 받은 요청 완료 작업. 요청 말풍선 위 = "요청 · {상태}" 태그(메모 없음), 대기가 아니면 바로 아래 **의견 말풍선**(`reqFeedback` — 머리 "요청 의견 · {시각}",
  받은 쪽 사람 쪽, 상태 크게 · 의견 본문(줄바꿈 그대로) · 바뀐 시각). 사람 ↔ 에이전트 구분(#1z — 시안 A): 사람 글 = 색 면(내 글 강조색 층 · 상대 글 `--recv-bg`),
  에이전트 글 = `--surface` 면 + `--agent` 1.5px 테두리 + 메타 "에이전트" 칩(`.tag.agent-tag`) + 로봇 아바타(`.avatar.agent-av` — 좌 · 우 어느 쪽이든). 대화 목록 =
  사이드바 `--subtle` 바탕 위 카드 줄(`.side-list li`, #1z-2 한 줄 · #1z-7 유형 아이콘 · 유형 색 테두리), 내 활동 · 북마크 줄은 이름 앞에 발화자 아이콘(`speakerEl` — `view.js` `speakerIcon`: 에이전트 = `bot` ·
  사람 = `user`, `inboxRows` · `activityRows` 의 `author`), 주고받은 메시지는 원형 아바타(위). 웹UI 편집 모드 없음(#1v — D34 편집 · 삭제는 CLI, "(편집됨)" 표시는 남음).
- **대화 유형 색**(#1z-7 시안 A): 대화를 그리는 곳의 둘레에 `data-kind` = 대화 `dir` — `[data-kind]` 가 `--kind`(`in` = `--conv-in` → `--agent` 청록 · `out` = `--conv-out` →
  `--accent` 파랑 · `shared` = `--conv-shared` → `--faint` 회색 점선)와 `--kind-soft`(고른 줄 바탕)를 정하고, 줄 · 카드 · 머리 · 아이콘 칩(`.kind-chip` — 유형 색 면 + `--surface` 아이콘)이 그 값을 쓴다.
  닿는 곳: 대화 머리(`#conv_kind` 28px 칩 — 제목 앞, `role="img"` 이름 `KIND_NAMES` · 머리 아래 2px 선), 사이드바 대화 줄(`convItem` — 대화 · 즐겨찾기 · 에이전트 응답, 줄 테두리 ·
  고른 줄 = 유형 옅은 바탕 + 안쪽 테두리), 개요 미리보기 카드(1.5px 테두리 · 머리 `pv-kind` 칩, 안 읽음 = 바깥 `outline` 강조색 · 마우스 = 그림자), 개요 목록 줄(`home-kind` 칩 ·
  테두리). 설정의 대화 관리 줄(그룹설정)과 말풍선은 그대로.
- **세부정보 패널**: 진행 · 대화 정보 · 자동 왕복 · 관람자 · 요청 · 북마크.
- **설정**: 한 열 전체 폭 트리(페어링 → 그룹설정 → 내 에이전트 → 실행 → 화면 → 시작 → 알림) · 카드 안내는 제목 옆 '?'. "페어링" 카드 = 큰 단추 둘 "초대 생성" ·
  "초대 수락"(붙여 넣기 칸 여닫기 — 제출 "수락"). 트리 · 새로 만들기 "초대 생성" 은 `press: "window_btn"` 으로 초대를 바로 만든다(클릭 때만 — 해시로 와서는 아님,
  #1r). 초대 수락 칸은 라우터가 `#settings/invite_accept`(트리 · 새로 만들기 · 레일 +)면 펼치고 그 밖의 경로에서 접는다. 연결 · 내 에이전트 줄 = 타일 · 이름 · 부제 ·
  연필 단추(`manageItem` — `icon-btn manage-btn`, "{이름} 관리", 상세 여닫기 · 펼치면 강조), 그룹설정 카드(#1z — 옛 "연결 목록", #1z-2 시안 B) = 위 그룹 카드 격자 + 아래 사용자 표(아래 레일 그룹). 내 에이전트는 평평한 목록 · 검색 없음 ·
  "읽기 전용" 표시 없음, 머리 타일 = 로봇(`agentTile` — 고른 아이콘 색 클래스, #1z-7), 상세 = 한 격자(`form-grid` — 이름(읽기 전용 · "등록 뒤 바꿀 수 없음") · 아이콘 색
  `swatches`(6색 `AGENT_COLORS` 단추 `aria-pressed`, 저장 때 바뀐 경우만 `color`) · 폴더 · 상위 한단계 탐색 허용 · 셸 명령 실행 허용(체크박스 옆 한 줄 `check-line` —
  `PARENT_HINT` · `SHELL_HINT`) · 도구 · 추가 지시), 단추 저장 · 삭제 · 닫기(연필 단추와 같은 접기, 초점 → 연필). '등록된 에이전트' 화면의 내 에이전트 줄도 같은 로봇 타일(작게), "에이전트 등록" = 목록 끝 `manage-item new-agent` "새 에이전트" 줄 보이기 · 숨기기(취소 · 등록하면 닫음). 알림 "자동 왕복" 옆 '?'
  (`tip_relay` — 위로 여는 `.tip.up`)가 기능 · 세 항목 설명. 화면 "타일 크기"(`ui_tile_size`) · 알림 "효과음"(`notify_sound` — 받은 요청 아래, #1x). 내 정보는 프로필 "사용자 정보" 팝업.
- **아이콘**: 메뉴 · 화면 제목 · 설정 트리 · ⋯ 메뉴 · "+ 새로 만들기" 이름 앞 선 아이콘(16px 한 벌).
- **프로필 · 상태**: 프로필 단추 = 아이콘 + 표시 이름(12em 말줄임 · 전체는 툴팁). 아이콘 테두리 · 점 색 = 내 상태(초록 받기 가능 · 주황 집중 · 붉은 앱과 연결 끊김 —
  이 화면의 요청이 앱에 연달아 2번 닿지 못하면 자동, 닿으면 되돌림), 프로필 메뉴에서 받기 가능 · 집중을 고른다. 머리글 접속 상태.
- **화면 동작**: 다른 PC 를 고르면 열린 대화를 닫는다. 보내기는 해시의 대화 · 레일 상대가 맞을 때만. 읽음 · 내 활동 본 표시는 화면이 받은 범위(id)까지만.
  늦게 온 조회 응답은 버린다. 트레이 메뉴는 "열기" · "종료"(S4).

## 레일 그룹 · 손으로 놓는 자리

D37 · D35 — 그룹 = 이 PC 에만 있는 대화 묶음(상대 PC 로 가는 프레임 없음 — 다자 채널 아님, 저장은 [개발자 가이드](developer-guide.md) 7.2).

- 그룹의 대화 = 넣은 대화만(#1w — D37 개정, 한 대화는 여러 그룹에). 상대 단위 소속은 없다(스키마 21 이 옛 상대 소속을 그 상대의 그때 대화로 옮겼다 — 뒤에 생긴 대화는
  들어오지 않는다). 그룹을 지우면 소속만 지워진다.
- 대화 넣고 빼기 = 대화 ⋯ 메뉴 그룹 체크 · 대화 줄 손잡이를 그룹 타일에 끌어 놓기 · 그룹 "대화 추가" · 설정 그룹설정.
- 설정 그룹설정(#1z-2 — 시안 B, `renderPeerList`): 검색 = `view.groupSettingsList`(그룹 이름 · 사용자 이름/별칭/PC/지문 · 대화 제목). 위 `gsGroupsEl` = 카드 격자
  (`.gs-cards` — `minmax(220px, 1fr)`, 카드 `.gs-card` `data-drag="group"` = 대화 놓는 자리): 머리 = 펼침 단추 `gsToggle`(그룹 타일 · 이름 · "대화 n") · 연필(이름 바꾸기) ·
  휴지통(삭제), 접힘 = 대화 제목 3개 + "+n 더 보기"(누르면 펼침) · 빈 그룹 "대화 없음 — 끌어 놓으세요", 펼침 = `view.settingsGroupSections` 의 그 그룹 대화 줄(`settingsConvRow` ·
  "그룹에서 빼기"). 아래 `gsUsersEl` = 표(사용자 · PC · 상태(`gsDot` — 점 + 글자, `data-peer-dot` 부분 갱신) · 모델 · effort · 대화 · 관리 연필), 줄(단추 · 입력 밖) · 이름 단추 ·
  연필을 누르면 바로 아래 `tr.gs-more` = `userDetail`(부제 · 지문 · 주소 · 모델 · effort · 단추 + "대화 n" 줄). 펼침 기억 = `state.open`(`gsg{그룹}` · `gsu{상대}` — 5초 새로 고침에도
  그대로, 이 창에만). "+ 그룹 추가" 로 만든 카드는 펼친다. 대화 줄 손잡이(포인터 끌기 — `convGrab(…, { join: true })`: 포인터 아래 가장 가까운 그룹 카드만 대상,
  끄는 동안 `gsDragging` 으로 다시 그리기 미룸, `lostpointercapture` 도 끝 처리)를 카드에 놓으면 **복사**(`gsJoin` · `view.convCopy` — 넣기만, 원래 자리 · 대화 순서 그대로,
  이미 있으면 그대로 · 안내). HTML 끌어 놓기(`draggable`)는 없다.
- 순서는 정렬 기능 없이 손으로 놓은 자리(`pos` — 상대 · 그룹 · 대화, 대화는 대화마다 하나) — 끌어 놓기 · Alt + ↑ / ↓, 같은 묶음 안에서만. 레일 타일 = 포인터 끌기(#1z — `tileGrab`: 포인터 캡처, `view.pastSlop` 5px 넘게 움직여야 끌기 · 그 미만은 누름, 사본 `rail-ghost` · 놓을 자리(#1z-2) =
  포인터가 레일 안이면 같은 묶음 타일 가운데 중 가장 가까운 것의 앞 · 뒤(`view.railSlot` — 틈 · 타일 아래 이름(`pointer-events: none`) · 양 끝 밖도, 옛 `elementFromPoint` 는 그 자리에서
  아무 일이 없었다) → `moveItem`, 끈 뒤 click 은 먹는다, 레일 `user-select: none`, HTML 끌어 놓기 없음),
  사이드바 대화 줄 = 손잡이 포인터 끌기(#1x — `convGrab`: 포인터 캡처 · 줄 사본(`board-ghost`) · `elementFromPoint` → 같은 절 줄 = `moveItem`, 그룹 타일 = `joinGroup`,
  요청 보드 `boardGrab` 꼴 · 줄 링크 `draggable="false"` — 브라우저 끌기 확장이 반응하지 않게). 세션 생성으로 만든 대화는 맨 위,
  새 글이 와도 자리 그대로. 통합 검색 · CLI 목록은 최근 활동순 그대로.
- 타일 메뉴(우클릭 · 메뉴 키 · ⋯) = 타일 편집 · 이미지 지우기 · 위로 · 아래로 옮기기 · 그룹 삭제(대화는 남김). 상대 타일의 '그룹' 체크는 없다(#1w).

## 공유 대화(관람자) 화면

D38 — 통신 · 저장은 [개발자 가이드](developer-guide.md) 8.11.

- 관람자 PC 사본(`dir = shared`)은 읽기 전용 — 입력창 · 실행 · 자동 왕복 · 요청 · 다시 공유 없음, 내 활동에서 뺀다. 넣은 사람 쪽 글이 오른쪽.
- 원 상대 화면에는 관람자 수만. 끝(빼기 · 나가기 · 페어링 해제 · 24시간 닿지 않음)이면 사본에 "공유 해제됨".

## 기록 삭제 · 나가기

D40(#1w · #1x 개정) — 통신 · 저장은 [an_app](an_app.md) 경로 · 프레임.

- 대화 줄 ⋯(`.row-more`)은 줄 flex 끝 제자리(`position: static` — 떠 있으면 시각을 가린다, 사용자 지시). '나' 개요(목록 줄 · 미리보기 카드)에서 대화를 열면 `followFromHome` 이 레일을 그 상대로(`pickRail`, 사용자 지시).
- 대화 ⋯ 메뉴(대화 줄 · 대화 머리 같은 함수 `openConvMenu`) 끝 = 구분선 + `view.convMenuItems(conv)`:
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
- 닫힌 대화(`view.convClosed` — "나감" · "상대가 나감" · "공유 해제됨")는 읽기 전용 — 입력창 · 컨텍스트 단추 · 자동 왕복 단추 · 관람자 넣기 없음, 받은 요청의 상태 바꾸기 ·
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
  ([an_app](an_app.md) `notify::request_notice`). `view.requestNotice(n)` → tag `coworkagent-request`(같은 tag 라 앞 알림을 바꾼다) · 누르면 `pickRail(null)` · 요청 보드
  (`#requests`), 그 밖 kind 는 tag `coworkagent-{kind}` · 내 활동. 받은 요청 알림은 `silent` — 소리는 웹UI 효과음 `chime()`(WebAudio 두 음 880 Hz → 1320 Hz 각 0.12 초 ·
  음량 0.15, `AudioContext` 는 첫 `pointerdown` · `keydown` 에서 만든다 — autoplay 정책, 그 전에는 소리 없음). 설정 `notify_sound` 켜짐(기본) · 집중 아님일 때만.
  OS 방해 금지는 웹UI 가 알 수 없어 그때도 소리가 난다 — 끄는 길은 "효과음" 설정. 위치 · 모양은 OS 가 정한다.
- **다시 읽기는 알림이 아니라 변경 신호**(D39 ②) — SSE `event: change` 는 알림 설정 · 일시 중지 · 집중과 무관하게 온다(상대 요청 처리 · 메시지 알림 = `conv` + 이 PC
  대화 id, 페어링 = `peer`). `view.js` `loadersFor` 가 자원 → 로더(`CONV_LOADERS` — 연결 · 그룹 · 목록 · 열린 대화(같은 id · id 모름만) · 내 활동 · 요청 ·
  저장 · 상대 메시지 · 실행 · 건수)를 고르고, 빈 · 모르는 종류 · `peer` 면 전체. 50ms 안의 신호는 `mergeChanges` 로 한 번. 알림 처리기는 상태 줄 · 브라우저
  알림만. 5초 전체 새로 고침은 그대로(이 PC 변경 · 시간이 지나 꺼지는 접속 표시용).

## 디자인 토큰

#1p 시안 A 먹선 — 정본은 `app.css` 첫 `:root`(= 1p `design/tokens.json`). 색 · 글꼴 · 글꼴 크기 · 굵기 · 줄높이 · 간격 · 둥글기는 `:root` 밖에서 `var(--…)` 만
쓴다 — `tests/css_tokens.rs` 가 막는다(예외 `0` · `auto` · `1px`(테두리만) · `-1px`(`.sr-only` 한 곳) · `50%` · `inherit` · `line-height: 1` · `@font-face` · `url(…)` 안 —
`var(--이름, 대체)` 의 대체 값도 검사, 첫 `:root` 밖 사용자 속성은 `var(…)` · `url(…)` 만). 이름 그대로가 라이트, 다크는 `--dark-*` 를 `:root[data-theme="dark"]` 가 대입한다(기본 테마 라이트 — #1x, `index.html`
`data-theme="light"` · 저장 값이 없으면 `light`).

| 묶음 | 토큰 | 값(라이트 / 다크) |
|---|---|---|
| 바탕 | `--bg` · `--surface` · `--surface-2` · `--sidebar` · `--subtle` · `--border` | 무채 회색 — 바탕 #F4F6F8 / #0F1112 · 면 #FFFFFF / #17181B · 선 #DBDEE2 / #2E3133 |
| 글자 | `--text` · `--muted` · `--faint` | #171A1D · #4E5358 · #686C71 / #E6E8EA · #AAAEB3 · #8F9397 |
| 강조 | `--accent` · `--accent-hover` · `--accent-soft` · `--on-accent` | 쪽빛 #245BAB / #7FACF0 — 주 단추 · 링크 · 선택 · 초점에만 |
| 상태 | `--agent` · `--run` · `--ok` · `--danger` · `--wait` · `--on-danger` | 차분한 색, 글자/바탕 4.5:1 이상 |
| 레일 · 타일 | `--rail` · `--rail-2` · `--rail-text` · `--on-dark` · `--peer-1 ~ 5` · `--agent-tile` · `--rail-w-small` | 레일은 두 테마 모두 어둡다, 타일 5색은 같은 명도 · 채도(흰 글자 4.5:1 이상), 작은 타일 레일 폭 168px(#1x) |
| 글꼴 | `--font` · `--mono` | IBM Plex Sans KR(실행 파일 포함 — `/fonts/`) · 고정폭은 시스템(코드 · 키 · 지문 · 원문 · 숫자만) |
| 크기 · 굵기 · 줄높이 | `--fs-xs ~ xl` · `--fw-regular/medium/bold` · `--lh-body` · `--lh-tight` | 12 · 13 · 14 · 16 · 20 · 400 · 500 · 600 · 1.6 · 1.3 |
| 간격 | `--sp-1 ~ 7` · `--gap` · `--control-h` | 4 · 8 · 12 · 16 · 24 · 32 · 40(4pt) — 그 밖은 `calc` 로 토큰을 더한다 |
| 둥글기 | `--radius-sm` · `--radius` · `--radius-lg` · `--radius-bubble` · `--radius-pill` | 4 · 6 · 8 · 12 · 999 |
| 그 밖 | `--shadow` · `--backdrop` · `--bubble-mine-alpha` · `--bubble-theirs-alpha` | 작은 그림자(팝오버만) · 대화 상자 뒤 · 말풍선 층 0.1 · 0 |

- **AI 티로 쓰지 않는 것**(1p `design/tokens.json` `slop` — `gates.py design`): 말풍선 옆 띠 · 선택 안쪽 띠 · 점선 상자(끌어 놓기 표시는 예외) · 숫자 테두리 상자 ·
  절 제목 자간 · 단추의 글자 기호 아이콘(✎ ⋯ → 선 아이콘 `ico-pencil` · `ico-more`).
- **고정폭**은 `.count` · `.fp` · `.kbd` · `.md code` · `.mono` · `.raw` · `.steps .at` · `kbd` 에만(한글이 드는 칩 · 부제 · 에이전트 이름 · 실행 메타 · 안내는 본문 글꼴,
  숫자는 `font-variant-numeric: tabular-nums`). 한글이 든 숫자 표시("안 읽음 n")는 `.stat`.
- **대비** — 글자/바탕 짝 · 타일 흰 글자 · 말풍선을 1p `gates.py contrast` 가 첫 `:root` 로 계산한다(4.5:1).
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

# stage 2 — 웹 ES 모듈

## 목표

plan "설계 · stage 2" — `app.js`(5,221줄) · `view.js`(1,687줄)를 `crates/app/web/js/` 아래 ES 모듈로 나눈다(옮기기만 — 본문 바이트 그대로), 서버는 `SCRIPTS` 표 · `GET /js/{*file}`.

## 산출 파일

- `crates/app/web/js/`: 진입 `app.js` · `core/`(`token` · `api` · `dom` · `widgets` · `state` · `float`) · UI 도메인 17(`shell` · `events` · `rail` · `tiles` · `sidebar` · `home` · `conv` ·
  `conv-menu` · `composer` · `detail` · `activity` · `requests` · `agents` · `peers` · `compose` · `drag` · `search`) · `settings/`(`index` · `conn` · `groups` · `agents` · `prefs` · `debug`) ·
  DOM 없는 `logic/` 18 — 모두 48개, 가장 긴 파일 `conv.js` 657줄.
- `crates/app/src/server.rs`: `pub const SCRIPTS: &[(&str, &str)]` · `/js/{*file}` · 토큰 없이 주는 판정은 `SCRIPTS` 이름과 정확히 같을 때만 · `/app.js` · `/view.js` 경로 없앰.
  `index.html`: `<script src="/js/app.js" type="module">`.
- 시험: 새 `crates/app/tests/web_assets.rs`, 고친 `web_auth.rs` · `chat_api.rs` · `ux_literals.rs` · `crates/app/web/view.test.mjs`. `cli.rs` 주석 경로 2곳(`view.js` → `logic/author.js`).

## 설계 결정

- 나누는 스크립트(scratchpad `jssplit.py`): 최상위 항목(함수 · `const` · `let` · `class` · 문장 + 바로 위 주석)을 이름 → 모듈 표(이름 · 줄 구간)로 잘라 붙인다. 최상위 선언은 모두 `export`.
- import: UI 모듈은 다른 모듈이 내보낸 이름이 원문에 낱말로 나오면 import(주석 · 문자열 포함 — 빠진 import 없게), `logic/` · `core/` 는 코드 낱말만
  (주석의 `/api/…` 낱말이 `core/state.js` ↔ `core/api.js` 순환을 만들어 `state.js` 가 늦게 평가됐다 — plan 개정 기록 ②). 코드 낱말만 쓴 모듈의 원문 전용 낱말은 모두 주석 · 문자열임을 확인(`rawdiff.py`).
- 진입 `app.js` 의 첫 줄 import = `core/token.js`(줄 끝 주석으로 이유) — 토큰이 없으면 `core/dom.js` · `logic/shell.js` 만 평가된 뒤 멈춘다.
- `keepFocus` → `drag.js`(끌기 항목 `dragInfo` · `dragSel` 을 쓴다 — `core/dom.js` 는 도메인을 import 하지 않는다).
- `railDragging` → `drag.js`(대입은 끌기에서만, `rail.js` 는 읽기만 — 다른 모듈의 `let` 에 대입하지 않는다).
- window 오디오 깨우기(`pointerdown` · `keydown` → `audio ??= new AudioContext()`) → `events.js`(`let audio` 옆). 진입에 두면 import 바인딩 대입(TypeError) —
  window · document 최상위 등록 차례는 그대로다(원본에서 맨 앞, 진입 본문의 document `click` · `keydown` · window `hashchange` 는 모든 import 평가 뒤). plan 개정 기록 ①.
- 구획 주석(`// ---- … ----` 23개)은 다음 항목과 함께 옮겼다(stage 4 에서 정리). `view.js` 첫 줄 머리 주석은 모듈 머리 주석으로 대신하고, `app.js` 머리 3줄은 진입 머리로(`view.js` → `logic/`).
- 모듈마다 머리 한 줄(무엇을 담는지)을 새로 썼다.

## 옮기기만 확인

`jsverify.py` — ① 표의 항목을 차례로 이어 붙이면 원본과 바이트로 같다 ② 모듈 파일 = 머리 · import 줄 · 빈 줄 · 그 모듈 항목(원본 차례, 선언 줄 앞 `export ` 만 더함) ③ 원본(뺀 항목 제외)과 모듈 본문 전체의 줄 multiset 이 같다:

- `view.js`: items 191 · modules 18 · dropped [] · non-blank lines 1512 == 1512 · export added 7 · problems 0
- `app.js`: items 377 · modules 30 · dropped [4](옛 import 블록) · non-blank lines 4748 == 4748 · export added 276 · problems 0
- 검사기 확인: 모듈 하나의 `"click"` 을 `"clack"` 으로 바꾸면 problems 2(본문 다름 · 줄 multiset 다름). 설치한 `crates/app/web/js/` 에 다시 돌려도 위와 같다.

## 모듈 경계 검사

scratchpad 스크립트(층 규칙은 `web_assets` 시험에도):

- 층: `logic/*` 는 `logic/*` 만 import · 순환 없음. `core/dom.js` · `core/state.js` · `core/token.js` 는 `core/` · `logic/` 만. `core/api.js` → `shell.js`(`renderAvatar` — 연결 끊김 표시, plan 층 규칙 밖).
- 다른 모듈 `let` 에 대입: 0(복합 대입 `??=` · `||=` 포함 — 처음 정규식이 `??=` 를 놓쳐 보강, `audio` 건을 찾음).
- 같은 (요소 · 이벤트)를 두 모듈이 등록: 0. 최상위 문장이 같은 id 를 함께 쓰는 모듈은 이벤트가 다르거나(`rail` — `tiles.js` contextmenu · `drag.js` dragstart · keydown) 처리기 안에서 읽기만 한다.
- 원래 비공개였다가 `export` 된 `logic` 이름 7개를 UI 모듈 코드가 쓰는 곳: 0(전역 이름을 가리지 않음). 원본 import 에 별칭(`as`) 없음.
- 문법: 48 모듈 모두 `node --check`(ES 모듈) 통과 — 같은 이름 두 번 선언 · import 충돌 없음.

## 상태 변경 표(A2 리뷰 자료)

모듈마다 `state.X` 에 대입 · 컬렉션 변경하는 속성(주석 제외):

| 모듈 | 바꾸는 `state` 속성 |
| --- | --- |
| `core/api.js` | `netFails` |
| `shell.js` | `current` · `focusAfter` · `focusMsg` · `me` · `previewAll` |
| `events.js` | `previewAll` · `previewStale` · `reqBox` |
| `rail.js` | `current` · `groups` · `peerAgents` · `peerNames` · `peers` · `railGroup` · `railPeer` · `unseen` |
| `sidebar.js` | `convs` · `current` · `folded` · `reqWaiting` · `unseen` · `unseenAll` · `waitingBy` |
| `home.js` | `homeView` · `previewAll` · `previewCols` · `previewMsgs` · `previewStale` |
| `conv.js` | `focusMsg` · `msgs` · `msgsOf` · `pickMsg` |
| `conv-menu.js` | `current` |
| `composer.js` | `mention` · `mentionWarned` · `pickMsg` · `sending` |
| `detail.js` | `detailOpen` · `runs` |
| `activity.js` | `actFresh` · `actKind` · `actOpening` |
| `requests.js` | `reqBox` |
| `peers.js` | `peerHeads` · `peerLoads` · `peerPages` |
| `compose.js` | `focusAfter` |
| `drag.js` | `convs` · `groups` · `peers` |
| `settings/index.js` | `focusAfter` |
| `settings/agents.js` | `agents` · `focusAfter` |
| `settings/prefs.js` | `folded` · `homeView` · `peerPages` · `runs` · `settings` |

다른 도메인 상태를 바꾸는 곳은 도메인을 가로지르는 흐름이다 — 설정값 적용(`settings/prefs.js` `fillPrefs`), 변경 신호로 다시 읽을 것 표시(`events.js`), 순서 저장 뒤 목록 반영(`drag.js`),
타일 · 대화 고르기(`rail.js` · `sidebar.js` · `shell.js` 의 `current`), 다음 화면 초점(`focusAfter`). 옮기기만이라 함수를 쪼개지 않았다 — 리뷰 판단(plan 리뷰 초점 A2).

## 바꾼 시험

- `web_auth.rs`: 토큰 없이 받는 정적 경로 = `/` · `/app.css` · `/favicon.ico` + `SCRIPTS` 전부(`/js/<이름>` 200 · `text/javascript`). 새 `scripts_exact_names_only` —
  `/js/nope.js` · `/js/core/` · `/js/logic/../app.js` · `/app.js` · `/view.js` 는 토큰 없이 401 · 토큰으로 404. Host 403 · 보안 헤더 시험의 경로 `/app.js` → `/js/app.js`.
- `chat_api.rs`: `/view.js` → `/js/logic/message.js`(`export function timeline`, 같은 guard 뒤).
- `ux_literals.rs`: 원본 = `an_app::server::SCRIPTS`(`app` = `logic/` 밖 모듈, `view` = `logic/` 모듈을 이어 붙임), 행의 파일 키 `app.js` → `app`(253곳) · `view.js` → `view`(109곳), 주석 속 경로 1곳.
- `view.test.mjs`: import 166 이름 → `./js/logic/*.js` 18 모듈, 원문 읽기 3곳(`./app.js` 둘 · `["./index.html", "./app.js", "./view.js"]` 반복) → `source(false | true)`(`js/` 모듈을 이어 붙인 글) — plan 개정 기록 ③.
- 새 `web_assets.rs`: ① `web/js` 파일 = `SCRIPTS` 이름 · `index.html` 진입 태그 ② 상대 import 대상이 표에 있고 가져오는 이름을 대상이 `export` ③ 진입 첫 import = `core/token.js` · 층 규칙.
  검사기 확인: `core/float.js` 에 없는 이름 import · `logic/format.js` 에 `core/state.js` import 를 넣으면 각각 실패(되돌림).

## 테스트 결과

- `node --test --test-reporter=tap "crates/app/web/*.test.mjs"` → `# tests 172` · `# pass 172` · `# fail 0`(HEAD 원본을 scratchpad 에 꺼내 돌려도 172).
- `cargo clippy --workspace --all-targets -- -D warnings` → `Finished`.
- `cargo test --workspace` → `test result: ok` 84줄(stage 1 의 83 + `web_assets`) · `FAILED` · `panicked` 0 · 통과 521.

## 실제 화면(E8)

디버그 빌드 + 시험 데이터(1z-11 사본) · ego-browser 1440 × 900, 페이지보다 먼저 오류 수집기(`error` · `unhandledrejection` · `console.error` · `console.warn`)를 심고 57 단계:

- 라이트 · 다크(설정 '화면' 의 테마로 바꿈) 각각 개요(미리보기 · 목록) · 대화 · 세부정보 · 내 활동 · 요청(목록 · 보드) · 북마크 · 즐겨찾기 · 에이전트 응답 · 등록된 에이전트 · 세션 생성 ·
  설정 8 메뉴 · 검색 · 새로 만들기 · 프로필 메뉴 · 레일 타일 메뉴 — 단계마다 화면이 바뀌고 수집 오류 0, `/js/` 48개 모두 200.
- 실패 응답은 `/api/peers/{1,2,3}/ping` · `/api/peers/{n}/agents` 502 뿐(시험 데이터의 상대가 꺼져 있다 — 원격 왕복, 원본과 같음).
- 레일 타일 메뉴: 우클릭(`contextmenu`)은 ego-browser 가 문서 캡처 단계에서 전파를 멈춘다(진단 — 문서 캡처 리스너만 불리고 타일 · 레일 리스너는 안 불림, 앱에 캡처 리스너 없음).
  같은 `openTileMenu` 를 여는 사이드바 머리 ⋯ 로 상대 타일 메뉴를 열었다 — 타일 편집 · 위로 옮기기 · 아래로 옮기기.
- 토큰 없이 `/`(새 탭): 다시 열기 안내 · `/api/*` 요청 0 · `#new_menu` 자식 0 · `#claude_effort` 옵션 0(다른 모듈 최상위 코드가 돌지 않음) · 오류는 `토큰 없음` 하나(원본과 같음).

## 배정 표

`view.js → js/logic/` (191 항목)

| 모듈 | 항목 |
| --- | --- |
| `logic/format.js` | `STATE_TEXT` · `stateText` · `dayKey` · `dayLabel` · `dayBreaks` · `displayName` · `shortName` · `formatDuration` · `formatTime` · `formatClock` · `josa` · `preview` · `fileSize` |
| `logic/conv.js` | `convLabel` · `roomPreview` · `convRows` · `filterByPeer` · `keepsConversation` · `convSections` · `MAX_OBSERVERS` · `shareHead` · `shareCandidates` · `CONV_GROUPS` · `splitByDir` · `visibleConvRows` · `convListParts` · `agentConvSections` · `convMeta` · `convClosed` · `LEAVE_CONFIRM` · `convMenuItems` · `composeDefaults` · `requestNotice` · `INFO_CMD_HINT` · `convInfoRows` · `convKindIcon` · `KIND_NAMES` |
| `logic/message.js` | `relayText` · `runMeta` · `linkReplies` · `timeline` · `unlinkedReplies` · `refusedText` · `progressText` · `stepText` · `outboxText` · `messagesSigOf` · `mentionQuery` · `mentionToken` · `hasMention` · `mentionTarget` · `messageActions` · `editedTag` · `insertMarkdown` · `speakerIcon` · `avatarKind` · `NOTE_MAX` · `noteCount` · `enterSends` · `runSteps` · `lastRun` |
| `logic/context.js` | `CONTEXT_LIMIT` · `contextSize` · `contextLabel` · `contextBusy` · `ctxMenu` · `contextWhy` · `contextNote` |
| `logic/markdown.js` | `isHtmlDoc` · `HTML_CSP` · `htmlSrcdoc` · `inlines` · `ROW` · `RULE` · `ITEM` · `cells` · `markdownBlocks` |
| `logic/peer.js` | `peerState` · `presenceOf` · `presence` · `PEER_WEEKS` · `peerWeeksOptions` · `peerGroups` · `waitingByPeer` · `peerSummary` · `colorOf` · `unreadByPeer` · `tileUnread` |
| `logic/inbox.js` | `inboxRows` · `PEER_PAGE` · `peerSummaryRows` · `peerMsgsSig` · `mergePage` · `pageMore` |
| `logic/author.js` | `bubbleSide` · `authorLabel` · `sharedAuthor` · `CLI_LABEL` |
| `logic/order.js` | `sortByPos` · `moveInOrder` · `orderToApply` · `withOrder` · `movedText` |
| `logic/rail.js` | `tileKeeps` · `groupSettingsList` · `DRAG_SLOP` · `pastSlop` · `railSlot` · `splitKey` · `groupLeaves` · `peerLeaves` · `groupUnread` · `stepTarget` · `dropKind` · `inGroup` · `groupPeers` · `scopeQuery` · `groupNameError` · `membershipDiff` · `RAIL_W` · `railSize` |
| `logic/home.js` | `PREVIEW_MSGS` · `HOME_ZOOMS` · `previewZoom` · `previewSections` · `previewKey` · `previewStaleOf` · `previewBubbles` |
| `logic/request.js` | `REQ_STATES` · `askTag` · `reqMoves` · `reqResend` · `requestRows` · `reqArchiveAction` · `reqBoxes` · `reqHead` · `REQ_BTN` · `BOARD_COLS` · `boardColumns` · `canDrop` · `reqFeedback` |
| `logic/activity.js` | `ACT_KINDS` · `activityRows` |
| `logic/settings.js` | `settingsTree` · `SETTINGS_ALIAS` · `portText` · `draftOf` · `pruneDrafts` · `logLines` · `diagText` · `MODELS` · `EFFORTS` |
| `logic/agent.js` | `READ_ONLY_TOOLS` · `WEB_TOOLS` · `EDIT_TOOLS` · `TOOLS_NOTE` · `SKIP_GIT_HINT` · `CODEX_READ_NOTE` · `ENGINES` · `engineLabel` · `agentEngineBlocks` · `PARENT_NOTE` · `SHELL_NOTE` · `SHELL_WARN` · `PARENT_HINT` · `SHELL_HINT` · `SHELL_ON` · `shellConfirmNeeded` · `agentPatch` · `AGENT_COLORS` · `agentColor` |
| `logic/shell.js` | `route` · `holdsRender` · `sideNav` · `SESSION_NEW` · `newMenu` · `profileMenu` · `starredSections` · `sideSub` · `entryToken` · `DISCONNECTED` |
| `logic/sync.js` | `SendLatest` · `Latest` · `CONV_LOADERS` · `loadersFor` · `mergeChanges` · `unionLoaders` · `OneAtATime` · `runsSummary` |
| `logic/search.js` | `searchItems` · `searchGroups` |

`app.js → js/` (377 항목, 뺀 것 L4 = 옛 import 블록)

| 모듈 | 항목 |
| --- | --- |
| `app.js` | `문장 L5058` · `문장 L5073` · `문장 L5211` · `문장 L5212` · `문장 L5213` · `문장 L5214` · `문장 L5216` · `문장 L5217` · `문장 L5218` · `문장 L5219` · `문장 L5221` |
| `core/token.js` | `TOKEN_KEY` · `entry` · `문장 L164` · `문장 L165` · `token` · `disconnected` · `문장 L173` |
| `core/api.js` | `api` · `OFFLINE_AFTER` · `netResult` · `appOffline` |
| `core/dom.js` | `$` · `ico` · `lead` · `el` · `button` · `act` · `uid` · `field` · `unchanged` · `needsRender` |
| `core/widgets.js` | `kindChip` · `agentTile` · `PERSON_SVG` · `GROUP_SVG` · `tileFace` · `tileName` · `tileColor` · `rowTile` · `speakerEl` · `svgButton` · `iconButton` · `textButton` |
| `core/state.js` | `state` · `latest` · `previewLatest` |
| `core/float.js` | `floatFrom` · `openFloat` · `closeFloat` · `focusAfterFloat` |
| `shell.js` | `showStatus` · `showMe` · `applyTheme` · `applyTileSize` · `go` · `renderAvatar` · `VIEW_EL` · `show` · `applyFocus` · `render` · `MENUS` · `closeMenus` · `toggleMenu` · `문장 L4979` · `문장 L4998` · `setFocus` · `renderProfileMenu` · `profileAct` · `문장 L5053` · `문장 L5196` |
| `events.js` | `refresher` · `refresh` · `refreshOnce` · `changeQ` · `onChange` · `refreshFailed` · `PAIRING_KINDS` · `audio` · `문장 L4785` · `chime` · `listen` |
| `rail.js` | `applyRailOrder` · `loadPeers` · `renderRail` · `drawRailTiles` · `groupColor` · `loadGroups` · `railGroupOf` · `scopedConvs` · `pickGroup` · `pingPeers` · `pickRail` · `pickTile` · `문장 L670` · `loadPeerAgents` · `openGroupNew` · `문장 L4025` · `openGroupRename` · `changeGroup` · `saveTileSize` · `문장 L4918` · `문장 L4949` |
| `tiles.js` | `ICON_PX` · `tileEdit` · `currentTile` · `tileNow` · `openTileEdit` · `closeTileEdit` · `renderTilePreview` · `iconFromFile` · `문장 L825` · `문장 L826` · `문장 L827` · `문장 L829` · `tileMenuEvent` · `selectTile` · `openTileMenu` · `groupDeleteForm` · `문장 L938` · `문장 L939` · `문장 L940` · `문장 L946` · `문장 L960` |
| `sidebar.js` | `renderSideHead` · `renderSideNav` · `loadConvs` · `STAR_SVG` · `starButton` · `toggleStar` · `convItem` · `renderConvList` · `renderStarred` · `renderAgentConvs` · `pairedIds` · `foldEl` · `foldSaver` · `setFolded` · `grouped` · `문장 L2156` · `loadCounts` · `SIDE_W` · `sideSave` · `setSideWidth` · `saveSideWidth` · `문장 L4884` · `문장 L4899` |
| `home.js` | `followFromHome` · `refreshHome` · `renderHome` · `renderHomePreview` · `fitPreviewCols` · `refitPreview` · `문장 L2062` · `previewSectionEl` · `previewCardEl` · `applyHomeZoom` · `homeZoomSaver` · `문장 L2140` · `setHomeView` · `문장 L2154` |
| `conv.js` | `inlineEls` · `markdownEl` · `bodyEl` · `bodyPair` · `showRaw` · `문장 L371` · `문장 L372` · `findConv` · `renderConvHead` · `renderCtx` · `openCtxMenu` · `ctxConfirm` · `문장 L2406` · `showConversation` · `elapsed` · `messageEl` · `messageAct` · `bookmarkButton` · `toggleBookmark` · `COPY_SVG` · `BOOKMARK_SVG` · `TRASH_SVG` · `PENCIL_SVG` · `copyText` · `redrawMessages` · `messageEls` · `feedbackEl` · `contextEl` · `noteForm` · `resendState` · `messagesSig` · `focusMessage` · `setConvMessages` · `openConversation` · `editTitle` · `문장 L2779` · `문장 L2780` · `문장 L2781` · `문장 L2784` · `showRelay` · `switchRelay` · `renderShare` · `josaName` · `addObserver` · `removeObserver` · `문장 L2881` · `문장 L2883` · `문장 L2889` |
| `conv-menu.js` | `openConvMenu` · `END_TEXT` · `convEndForm` · `문장 L4216` · `openGroupPicker` |
| `composer.js` | `sendText` · `문장 L3083` · `문장 L3121` · `문장 L3131` · `문장 L3155` · `fitComposer` · `문장 L3162` · `moveMention` · `문장 L3167` · `문장 L3168` · `문장 L3171` · `문장 L3173` · `mentionNames` · `updateMention` · `closeMention` · `renderMention` · `pickMention` |
| `detail.js` | `showDetail` · `loadRuns` · `renderRuns` · `stepEl` · `renderDetail` · `infoView` · `renderInfo` · `drawInfo` · `copyLine` · `detailItem` · `문장 L3042` · `문장 L5202` · `문장 L5206` |
| `activity.js` | `rowEls` · `activityEls` · `loadActivity` · `actTimer` · `문장 L3329` · `renderActTabs` · `loadSaved` |
| `requests.js` | `loadRequests` · `renderReqArchive` · `archiveReq` · `REQ_BOARD` · `reqBoard` · `boardEls` · `boardGrab` · `boardDrop` · `문장 L3545` · `reqBoxEl` · `requestEls` · `requestItem` · `문장 L3607` · `문장 L3613` · `문장 L3614` |
| `agents.js` | `loadAgentsView` · `renderAgentsView` · `editAgent` |
| `peers.js` | `loadPeerMessages` · `loadPeerPage` · `peerSince` · `renderPeerMessages` · `moreRow` · `peerRowEls` · `markRow` |
| `compose.js` | `newChat` · `fillPeerSelects` · `agentsAsk` · `loadChatAgents` · `문장 L3896` · `syncRelayCap` · `문장 L3904` · `openCompose` · `문장 L3916` |
| `drag.js` | `keepFocus` · `railDragging` · `dragInfo` · `orderOf` · `orderSaves` · `orderSave` · `orderStamp` · `keepOrder` · `moveItem` · `clearDrop` · `joinGroup` · `convGrab` · `tileGrab` · `문장 L4460` · `문장 L4461` · `dragSel` · `stepMove` |
| `search.js` | `searchAsk` · `searchTimer` · `closeSearch` · `showSearch` · `searchMark` · `runSearch` · `문장 L5171` · `searchGo` · `문장 L5181` · `문장 L5185` · `문장 L5188` |
| `settings/index.js` | `goSettings` · `문장 L998` · `문장 L999` · `pick` · `setPick` · `mdRowId` · `closePick` · `mdRow` · `mdHead` · `mdFoot` · `문장 L1058` · `showAccept` · `문장 L1065` · `renderTree` · `openSettings` |
| `settings/conn.js` | `peerSub` · `convSig` · `renderPeerList` · `renderConnPage` · `gsDot` · `convLine` · `convAct` · `gsDrafts` · `userDetail` · `문장 L1257` · `inviteBusy` · `문장 L1367` · `문장 L1390` · `문장 L1401` |
| `settings/groups.js` | `renderGroupPage` · `groupDetail` · `문장 L1355` · `문장 L1356` |
| `settings/agents.js` | `loadAgents` · `agentDrafts` · `renderAgentList` · `agentDetail` · `showAgentNew` · `문장 L1604` · `newEngine` · `showNewEngine` · `문장 L1630` · `문장 L1635` · `문장 L1640` · `문장 L1641` · `문장 L1643` · `문장 L1645` |
| `settings/prefs.js` | `PREFS` · `fillPrefs` · `문장 L4640` · `문장 L4641` · `문장 L4642` · `문장 L4644` · `문장 L4670` · `showSettings` |
| `settings/debug.js` | `debugInfo` · `loadDebug` · `debugLog` · `renderDebugLog` · `debugCopy` · `문장 L4592` · `문장 L4593` · `문장 L4595` |

## 다음 단계

stage 3 — 죽은 코드 · 불필요 로직.

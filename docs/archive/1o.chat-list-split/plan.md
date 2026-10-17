# plan — 1o.chat-list-split

> 근거: [intent](intent.md)(승인 2026-10-17 — 시안 Ⓐ · 미해소 2 ~ 6 권장안). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `8966221`(task/1o-chat-list-split — intent 승인). plan 승인: 2026-10-17(사용자 "의견 수용 진행. 디자인 시안은 `1o Ⓐ 섹션 머리 그룹 · 음영 약하게`로 채택." — 해석 6 = (가) `--sent`, 그 밖 해석 그대로).

## 목표

사이드바 대화방 목록을 **내 대화**(`dir = out` — 내가 연 대화 · 내가 연 자동 왕복)와 **agent 대화**(`dir = in` — 상대 PC 사용자가 연 대화에 내 에이전트가 답)로
나눠 보이고, 말풍선 음영 대비를 시안 Ⓐ 단계로 낮춘다. 서버 · 저장 · 통신은 바꾸지 않는다(화면이 이미 `dir` 를 안다).

### 성공 기준(intent 사본)

- A1: 사이드바 대화방 목록이 내 대화 · agent 대화로 나뉘어 보이고(고른 시안 모양), 내가 연 자동 왕복 대화는 내 대화에 있다.
- A2: 상대 타일 · 그룹 · 레일 "나" 의 목록이 모두 같은 규칙으로 나뉜다(미해소 2 · 3 확정안대로).
- A3: 말풍선 음영이 고른 시안 단계로 낮아지고, 내 글 · 상대 글 구분과 글자 대비(4.5:1 이상)가 다크 · 라이트 모두 유지된다.
- A4: 기존 목록 기능(☆ · 안 읽음 · 실행 중 · 자동 왕복 칩 · 검색 · 끌어 옮기기 · 접기) 회귀 없음.
- A5: 검증 명령(`cargo test --workspace` · clippy · node 화면 시험 · `check_gates` · `check_ddd_docs`) 통과.

### intent 결정(사본)

- 구분 기준 = 대화의 `dir`(`out` = 내 대화, `in` = agent 대화). 자동 왕복은 연 쪽 기준 — 내가 연 자동 왕복 = 내 대화.
- 모양 · 음영 = **시안 Ⓐ** — 섹션 머리 "내 대화 n" · "agent 대화 n"(개수 포함). 말풍선 내 글 = 강조색 층 14% + 오른쪽 띠, 상대 글 = 타일 색 층 7%.
- 레일 "나" = 상대별 절 **안에서** 나눈다(상대별 접기 그대로). 즐겨찾기 화면 = 그대로. 끌어 옮기기 = 같은 묶음 안에서만. 빈 묶음 = 머리를 숨긴다(한쪽만 있으면
  지금처럼 한 목록). @멘션 대화(#18) = 그 대화의 `dir` 를 따른다.
- 제외: 서버 · 저장 · 통신 변경, 말풍선 밖 색, 목록 정렬 규칙 변경(손으로 놓은 순서 그대로).

## 코드 사실(실측, 기준 `8966221`)

- 대화 `dir` — `an_core::chat` `Dir::Out | Dir::In`(`core/src/chat.rs` 91 · 194), `/api/conversations` 가 `"dir": c.dir.as_str()` 로 준다
  (`app/src/chat_api.rs` 354). 두 값뿐이라 모든 대화가 둘 중 한 묶음에 든다.
- 사이드바 목록 `app.js` `renderConvList`(1409 ~ 1455) — `convRows(sortByPos(scopedConvs()), …, q)`(검색 = 제목) → `view.convSections(rows, paired, peerNames)`.
  `convSections`(`view.js` 642): `paired == null`(상대 타일 · 그룹)이면 절 하나 `{key: "convs", title: "대화방"}`, '나' 면 `peerGroups`(654) 로 상대별 절
  `{key: "peer:{id}", peer}`. 그리기: 상대별 절 = `` foldEl(`convs:${s.peer}`, …) ``(접힘 = 설정 `ui_folded`), 그 밖 = `section.side-section > h2.side-title`
  ("대화방" + `+`). 그룹이면 그 위에 머리 "그룹 대화 · n" + "대화 추가". 빈 목록 = 안내 행 "대화 없음" · "맞는 대화 없음" · 그룹 안내.
- 대화 줄 `convItem({c, label}, section)`(1365) — `li[data-drag="conv"][data-id][data-section=section]` + ☆ · 아바타 · 제목 · 미리보기 · 시각 · 실행 중 · 자동 왕복 칩
  (`convMeta` 688) · 안 읽음 · ⋯.
- 즐겨찾기 화면 `renderStarred`(1458) → `view.starredSections`(461) = `convSections(★ 행, …)` 를 그대로 부른다 → **`convSections` 의 반환을 바꾸면 즐겨찾기
  화면도 바뀐다**. `` convItem(r, `starred:${s.key}`) ``.
- 끌어 옮기기(#1i) — `dragInfo`(3178)가 `data-section` 을 읽고 `view.dropKind`(757)가 대화는 `drag.section === target.section` 일 때만 `"move"`, 대화 → 그룹 타일은
  `"join"`(그룹에 넣기). Alt + ↑ / ↓ `stepMove`(3308)는 같은 `data-section` 행만 이웃으로 본다. 옮긴 순서는 전체 순서(`moveInOrder` → `PUT /api/order`)에 끼운다.
- 말풍선 CSS(`app.css`) — 바탕 규칙 `.bubble`(531: `border: 1px solid var(--border)` · `background: var(--surface-2)`). 676 ~ 689(#1l):
  - 상대 글 `.msg.side-theirs .bubble` = `var(--surface)` + 왼쪽 3px `var(--tone)` 띠, `::before` 층 = `var(--tone)` · `opacity: 0.12`(타일 색 `data-tone` → `--tone`).
  - 내 글 `.msg.side-mine .bubble` = `background: var(--accent); color: var(--on-accent)`, 그 위에 흰 글자 전제의 덮어쓰기 — `a` · `blockquote` 글자색 상속(685),
    `code` · `pre` · `table` → `var(--text)`(686), `pre code` 바탕 없음(687), `.msg-icon` 상속 · 호버 `rgb(255 255 255 / 0.18)`(688 ~ 689).
  - 테마 = `:root[data-theme="dark"]` 이 `--dark-*` 를 대입(78). `--peer-1 ~ 5` 는 두 테마 같은 값. `color-mix` 는 쓰지 않는다(macOS 12 Safari 15, #1l 라운드 2).
- 시안 Ⓐ CSS(`design/shots.mjs` `CSS.A`) — 내 글 = `background: var(--surface); color: var(--text); border: 1px solid var(--border); border-right: 3px solid
  var(--accent)` + `::before` 층 `var(--accent)` · `opacity: .14`, 내 글 링크 `var(--accent)`, 상대 글 층 `opacity: .07`. 머리 = 목록 안 `li.grp-head`
  `<span>내 대화</span><b>n</b>`(양끝 정렬, `font-size: 11px` · `letter-spacing` 리터럴). 순서 = 내 대화 → agent 대화.
- 글자 대비(WCAG, 층 합성 = `tone × α + surface × (1 − α)`, 토큰 값 그대로 계산 — 2026-10-17):

  | 바탕 | 라이트 글자 / muted / 링크(`--accent`) | 다크 글자 / muted / 링크(`--accent`) |
  |---|---|---|
  | 지금 내 글(강조색 꽉 채움, 글자 `--on-accent`) | 5.79(글자) | 6.02(글자) |
  | **Ⓐ 내 글**(accent 14%) | 14.36 / 6.31 / 4.74 | 11.69 / 5.51 / **4.35** |
  | Ⓐ 상대 글(tone 7%, peer-1 ~ 5 최소) | 16.10 / 7.08 / 5.32 | 13.08 / 6.16 / 4.87 |
  | 지금 상대 글(tone 12%, 최소) | 15.13 / 6.65 / 5.00 | 12.19 / 5.74 / 4.54 |
  | Ⓐ 내 글 링크 대안 `--sent` | 5.95 | 6.77 |
  | Ⓐ 내 글 링크 대안 `--accent-hover` | 5.95 | 5.88 |

  → **시안 Ⓐ 그대로면 다크 내 글 링크가 4.35 로 A3(4.5:1) 미달**(해석 6).
- 시험 — node `crates/app/web/view.test.mjs` 104건 통과(2026-10-17 실측 `# pass 104`). 이름 첫머리 ASCII(`conv_sections — …`), `--test-name-pattern` 으로 골라
  돌릴 수 있다(`--test-isolation=none` 이어야 시험 단위로 센다 — 없으면 파일 하나 = 1건). `crates/app/tests/ux_literals.rs` 20건 — `icons_settings_literals`
  (536)가 **옛 말풍선 규칙 literal 을 단정**(593 `opacity: 0.12` · 594 `background: var(--accent); color: var(--on-accent);`) · 600 `color-mix(` 없음.
  CSS 는 `include_str!("../web/app.css")` 로 이미 읽는다.
- 문서 — 사용자 가이드 54 · 66 · 75 ~ 76(사이드바 · 대화방 줄 · 즐겨찾기) · 196 ~ 197(말풍선 색 "내 글은 강조색(진한 파랑) 바탕"), README 29("내 글은 오른쪽
  강조색"), PRODUCT 91 ~ 94 · 102(말풍선 색). 캡처 `docs/images/home.png`('나' 사이드바 상대별 절) · `group.png`(그룹 사이드바) · `workspace.png`(말풍선, README 6).

## intent 대비 해석 (plan 승인 대상)

1. **나누는 자리 — 사이드바 목록만.** `view.js` 에 새 export `splitByDir(rows)` → `[{dir: "out", title: "내 대화", rows}, {dir: "in", title: "agent 대화", rows}]`
   중 행 있는 것만(순서 = 내 대화 → agent 대화, 각 묶음 안 순서 = 들어온 순서 그대로). `renderConvList` 가 `convSections` 의 **각 절 안에서** 부른다 —
   상대 타일("대화방" 절) · 그룹("그룹 대화 · n" 아래) · 레일 "나"(상대별 접는 절 안) 모두 같은 한 길(A2). `convSections` · `starredSections` 는 바꾸지
   않는다(즐겨찾기 화면 그대로). 상대 개요(`peerSummary`) · 그룹 "대화 추가" 고르기 상자도 그대로.
2. **머리 모양(시안 Ⓐ)** — 묶음마다 머리 한 줄: 왼쪽 "내 대화" · "agent 대화"(글자 그대로 — `agent` 소문자), 오른쪽 개수(시안의 `<b>n</b>` 양끝 정렬). 기존
   관례 "그룹 대화 · n" 꼴(가운뎃점)이 아니라 **시안 literal** 을 따른다. 행이 있는 묶음이 **둘 다일 때만** 머리를 보인다(한쪽만 있으면 머리 없이 지금처럼 한 목록).
   글자 크기는 `--fs-xs`(12px) 토큰 — 시안의 `11px` · `letter-spacing` 리터럴은 `app.css` 규약("`:root` 밖 리터럴 금지")에 맞춰 바꾼다(편차, 승인 대상).
3. **개수 n = 지금 보이는 행 수**(검색으로 거른 뒤). 검색으로 한쪽이 0 이 되면 그 머리와 남은 쪽 머리도 숨는다(해석 2 규칙 그대로). 대안: 거르기 전 전체 수.
4. **끌어 옮기기 · Alt + ↑ / ↓** — 행의 `data-section` = `{절 key}:{dir}`(예 `peer:3:out` · `convs:in`). `dropKind` · `stepMove` 는 바꾸지 않아도 다른 묶음으로
   놓기 · 넘기기가 막힌다(같은 묶음 안에서만). 옮긴 순서는 지금처럼 전체 순서에 끼운다(정렬 규칙 변경 없음). **대화 → 그룹 타일 넣기(`"join"`)는 순서 옮기기가 아니라
   두 묶음 모두에서 그대로 된다.** 접힘 열쇠 `convs:{상대}` 는 그대로(나눈 뒤에도 상대별 절 하나를 통째로 접는다).
5. **말풍선 음영(시안 Ⓐ)** — 내 글: `isolation: isolate; background: var(--surface); color: var(--text); border: 1px solid var(--border); border-right: 3px
   solid var(--accent)` + `::before` 층(`var(--accent)`, `opacity: 0.14`, 상대 글 층과 같은 꼴). 상대 글: `::before` `opacity: 0.12` → `0.07`(띠 · 바탕은 그대로).
   흰 글자 전제의 내 글 덮어쓰기(지금 685 ~ 689 — `blockquote` 상속 · `code` / `pre` / `table` → `--text` · `pre code` · `.msg-icon` 상속 · 흰 호버)는 없애
   상대 글과 같은 기본 규칙을 따르게 한다(blockquote = `--muted` 5.51 이상). `color-mix` 쓰지 않음.
6. **내 글 링크 색 — 시안과 A3 충돌.** 시안 Ⓐ 의 `var(--accent)` 는 다크 4.35 로 4.5:1 미달. **결정: (가) `var(--sent)`**(2026-10-17). 고른 것:
   - (가) **`var(--sent)`**(라이트 `#3E45C8` 5.95 · 다크 `#A9AEFF` 6.77) — "보낸 글 글자" 토큰, 강조색 계열 유지. **권장.**
   - (나) `var(--accent-hover)`(5.95 · 5.88) — 링크 호버 때 색이 안 바뀐다.
   - (다) 글자색 상속 + 밑줄(14.36 · 11.69) — 지금 규칙 유지, 링크가 강조색이 아니다.
   - (라) 시안 그대로 `var(--accent)` — A3 미달을 받아들인다(A3 개정 필요).
7. **내 글 · 상대 글 구분(A3)** — 쪽(오른쪽 · 왼쪽) · 띠 쪽(오른쪽 강조색 · 왼쪽 타일 색) · 층 색(강조색 · 타일 색)으로 가른다. peer-1(`#5B6CF0`)은 강조색과 같은
   파랑 계열이라 그 상대의 대화에서는 층 색 차이가 작다(라이트 `#E6E7FA` 대 `#F4F5FE`) — 쪽 · 띠로 가른다(시안 Ⓐ 그대로, 바꾸지 않음).
8. **수동 화면 확인** — 루트 `PROJECT.md` "수동 테스트 시점"(masterPlan D28)대로 수동 gate(E10)는 `ABANDON`(사유 D28) + 19.prototype-acceptance TC 로 넘긴다.
   구현 중 mock(가상 데이터, `design/shots.mjs` 꼴 · 다크 · 라이트) 확인은 stage 기록(`work/stage2.md` 테스트 결과)에 남긴다 — gate 증거가 아니다.
9. **캡처 다시 찍기** — `home.png`('나' 상대별 절 안 나눔) · `group.png`(그룹 목록 나눔) · `workspace.png`(새 말풍선 음영), 전부 가상 데이터 · 기존 크기 · 테마 그대로.
10. 바뀌지 않는 것 — 서버 · API · 스키마, 즐겨찾기 화면, 상대 개요, 대화 줄 모양(☆ · 안 읽음 · 실행 중 · 자동 왕복 칩), 검색 방식(제목), 접힘, 손으로 놓은 순서,
    말풍선 밖 색(머리 · 입력창 · 실행 상세). @멘션 대화는 별도 처리 없음(그 대화의 `dir` 대로).

## 개정 — 관람자 초대 · 레일 우클릭 · 설명 지우기(intent 개정 범위 3 · 4)

코드 사실(2026-10-17 조사, worktree 기준 `3550dd5`):
- 프레임 = 4바이트 길이 + JSON, 연결 1개 = 요청 1 · 응답 1(`net/src/proto.rs` 1). `Request` 48 ~ 210(`#[serde(tag="type")]`), 새 필드는 `#[serde(default)]`.
  **`MAX_FRAME` 64KB**(42) · 글 본문 `MAX_TEXT` 16KB(44) — 이력을 한 프레임에 묶을 수 없다. 새 종류는 `node.rs` 492 ~ 505(처리기) · 628 ~ 638 · `proto.rs`
  `ids_ok` 326 ~ 349 세 곳에 등록. 능력 = `Pong { focus, ctx }`(227 ~ 233) — 구버전 거절 틀 `knows_context` · `deliver()` CTX 확인(agent `chat.rs` 911 ~ 924).
- outbox(`db.rs` 240 ~ 256) — 상대별 줄 `lane()`(chat.rs 868 ~ 893), 24시간 포기 `backoff()`(816 ~ 847, `CTX_DONE` 만 예외), `has_before` 는 **대화 단위**(outbox.rs 96).
  `stage` · `enqueue` 는 `conv.peer_id` 고정(chat.rs 687 · 713 · 763). `for_message`(outbox.rs 224)는 `msg_id` 로 "그 글을 나르는 행" 을 찾는다(`tidy_ok` 가 쓴다).
- 스키마 **19**(db.rs 5). `conv_row` 는 모르는 `dir` 이면 `Err`(core chat.rs 254 ~ 258). `(Dir, bool)` 전수 match(2452 ~ 2460 · 2496 ~ 2512 · 3525). `messages.uuid`
  UNIQUE(db.rs 72). 대화 삭제 기능은 없다. `chat::observe()`(core chat.rs 1087 — 램포트 시계)가 이미 있어 코드 이름은 `Share*`.
- 레일 우클릭 — 타일 `tileMenuEvent` 는 이미 `preventDefault`(app.js 720) — 빈칸 · 레일 바탕은 막지 않았다.

해석(승인 대상):

11. **이름** — 화면 "공유 대화"(사용자 literal) · 역할 "관람자". 코드 `Share*` · 대화 `dir = "shared"`(`Dir::Shared` — 전수 match 가 빠진 곳을 컴파일러로 잡고,
    `dir != In` · `== Out` 검사가 사본을 자동으로 거절한다). 기각: 별도 열만(모든 질의에 손으로 조건 — 보통 `Post` · `Edit` 가 사본을 건드리는 길이 남는다).
12. **프레임**(A = 초대한 PC · B = 원 상대 · C = 관람자, 모두 페어링된 상대만 · `ids_ok` 검사):
    `Share { conv, title, agent, owner_dir, other }`(A→C 시작 · 다시 초대 — C 는 사본 글을 비우고 다시 받음) · `SharePost { conv, msg }`(A→C 새 글 · 같은 uuid 면 편집) ·
    `ShareDelete { conv, msg }` · `ShareEnd { conv }`(A→C 빼기 · C→A 나가기) · `ShareNotice { conv, n }`(A→B 관람자 수 — **이름은 보내지 않는다**).
    **이력은 글 하나 = 프레임 하나**로 줄에 넣는다(16KB < 64KB). ponytail: 글 1000개 = 연결 1000번 — 느리면 크기를 재서 묶는다.
13. **저장(스키마 20)** — 새 표 `shares(conversation_id, peer_id, sent_upto, created_at)`(A — 관람자 · 보낸 커서), `conversations` 에 `share_other` · `share_dir` ·
    `share_ended`(C 사본) · `watchers`(B — 관람자 수), `messages.owner`(C — A 쪽 글). C 사본은 `peer_id = A` — A 타일 아래에 저절로 든다.
14. **퍼뜨리기** — `Chat::fan_share(conv)` 가 관람자마다 `id > sent_upto` 인 글(컨텍스트 구분선 제외)을 `SharePost` 행으로 넣고 커서를 올린다(한 트랜잭션 · 몇 번
    불러도 두 번 가지 않음). 부르는 자리 = `dispatch()` · `enqueue()` · `handle()` 끝. 편집 · 삭제는 `edit_message` · `delete_messages` · `inbound_edit` ·
    `inbound_delete` 에서 `id <= sent_upto` 글만 바로. 행은 `msg_id` NULL(편집 막힘 · 보관 표시 오염 방지) · `has_before` 예외 · **포기하지 않음**(C 가 오래
    꺼져 있으면 쌓인다 — 빼기 · 페어링 해제 때 지움). C 가 `not_paired` · `unsupported` 로 답하면 그 관람자를 빼고 A 화면에 표시.
15. **사본 막기** — C 저장은 전용 `share::upsert`(`mine=0 · exec=0 · request=0` — 실행 안 깨움), 같은 uuid 가 이 사본 것일 때만 덮어씀(아니면 `exists`).
    글쓰기 · 컨텍스트 · 자동 왕복 · 요청 상태 · CLI 보내기는 `Shared` 면 거절, 내 활동 · 안 본 활동 집계에서 뺀다. 받는 처리기는 `find(conv, A)` 가 **Shared 사본**인지
    본다(A 가 C↔A 자기 대화에 끼어들지 못하게).
16. **원 상대 알림(A8)** — agent 대화는 B 에게 `ShareNotice`(B 화면 대화 머리 "관람자 n" · 알림 1회). **내 대화도 같은 알림을 보낸다**(B 에이전트의 답이
    제3자에게 가므로 — intent 는 agent 대화만 적었다, 확인 필요). B 가 구버전이면 알림만 조용히 버린다.
17. **API** — `POST /api/conversations/{id}/observers {peer_id}`(거절: 사본 · B 본인 · 페어링 안 됨 · 능력 없음 409 "상대 앱 업데이트 필요") ·
    `DELETE /api/conversations/{id}/observers/{peer_id}` · `POST /api/conversations/{id}/leave`(C, 사본만). 대화 JSON — `dir:"shared"` · `observers` · `watchers` ·
    `share:{other, owner_dir, ended}`, 글 JSON `owner`.
18. **화면** — `CONV_GROUPS` 셋째 `{dir: "shared", title: "공유 대화"}`. 사본: 입력창 · 편집 모드 · 컨텍스트 · 자동 왕복 단추 숨김, 머리 "공유 · 읽기 전용", 빠졌으면 태그
    **"공유 해제됨"**(목록 줄 · 머리), 말풍선은 공유한 사람 쪽 글(`owner`) 오른쪽 · 원 상대 쪽 왼쪽, 발화자 = `owner` 면 A 이름 · 아니면 `share.other`. A: 대화 상세(⋯)에 "관람자" — 목록 · 추가(페어링된 상대 고르기) · 빼기, 머리
    "관람자 n". C: 상세에 "나가기". 사본에서는 다시 공유 불가.
19. **구버전 상대(0.5.0)** — `Pong.share`(`#[serde(default)]`). 초대 API 가 C 에 Ping → 모르면 409. 보내는 줄에서도 CTX 처럼 먼저 확인.
20. **페어링 해제** — C 를 지우면 그 관람(shares 행 · 줄의 share 행)도 지운다(C 사본은 남는다). A↔B 를 지워도 관람은 그대로(새 글이 없을 뿐).
21. **상한 · 정하지 않는 것** — 관람자 수 상한 **대화당 5명**(넘으면 400 "관람자는 5명까지") · 관람자 PC 는 보기만(편집 · 삭제 · 사본 지우기 없음 — 대화 삭제 기능 자체가 없다) · A 가 붙인 제목은 그대로 C 에 간다 · C 의 CLI `history` 는 사본도
    읽는다(발화자 표시만 고침).
22. **레일 우클릭** — `#rail` 에 `contextmenu` → `preventDefault`(타일 위는 기존 타일 메뉴). 브라우저 기본 메뉴는 자동 시험으로 볼 수 없다 — literal 단정 + E16 실화면.
23. **설명 지우기**(사용자 지시) — 개요 머리 설명 · 개요 카드의 접속 · "대화 n" · "최근 …"(안 읽음 · 대기 요청 · 페어링 해제 표시는 남김) · 북마크 머리 설명 · 빈 목록
    "— 메시지 작업 '북마크' 로 모읍니다" · 즐겨찾기 · 요청 머리 설명 지움, 에이전트(상대) 줄 = 이름 + 오른쪽 `button.small` "새 대화"(줄을 눌러도 같다), 타일 편집
    안내 "이 PC 에서만 표시". 시험 `ux_literals` `hint_cleanup_literals`.

## 설계

- `view.js` — `export function splitByDir(rows)`(해석 1 · 3; 파일 전용 함수가 아니라 기존 규약대로 export). 시험(`view.test.mjs`, 이름 첫머리 `conv_split`):
  - `conv_split_rows` — `out` · `in` 섞인 행 → 두 묶음 · 제목 literal · 순서(내 대화 먼저) · 묶음 안 순서 유지, 내가 연 자동 왕복(`dir: "out"`, `relay_agent`) =
    내 대화, 한쪽만 / 빈 행 → 머리 숨김 조건(행 있는 묶음 수), 검색으로 거른 행의 개수.
  - `conv_split_scopes` — 상대 타일(`convSections(rows)`) · 그룹 · '나'(`convSections(rows, paired)`) 각 절에 같은 규칙, 행 `data-section` 열쇠 꼴 `{절}:{dir}`
    (열쇠 만드는 함수도 `view.js` export — 예 `splitKey(section, dir)`).
  - `conv_split_drop` — `dropKind` 로 같은 절 · 다른 묶음 → `null`, 같은 묶음 → `"move"`, 두 묶음 모두 그룹 타일 → `"join"`.
  - `conv_split_starred` — `starredSections` 결과가 묶음으로 나뉘지 않음(기존 열쇠 · 제목 그대로).
- `app.js` `renderConvList` — 절마다 `splitByDir(s.rows)` → 묶음이 둘이면 묶음마다 머리(`.dir-head`: 제목 · 개수) + `ul.side-list`, 하나면 지금처럼 `ul` 하나.
  `convItem(r, splitKey(s.key, dir))`. 빈 목록 안내 · 그룹 머리 · `+` · 접힘은 그대로. `renderStarred` 그대로.
- `app.css` — `.dir-head`(`.side-title` 꼴 · `--fs-xs` · `--muted`, 양끝 정렬) · 말풍선 해석 5 · 6. 리터럴은 `opacity` 값뿐.
- `ux_literals.rs`:
  - 새 `chat_list_split_literals` — VIEW `"내 대화"` · `"agent 대화"` · `export function splitByDir(`, APP `splitByDir(` · `` foldEl(`convs:${s.peer}` ``(접힘 열쇠
    그대로) · `starredSections(`(즐겨찾기 그대로), CSS 내 글 `::before` `opacity: 0.14;` · `border-right: 3px solid var(--accent)` · 상대 글 `opacity: 0.07;` ·
    `.dir-head`, 없어진 것 0 — `.msg.side-mine .bubble { background: var(--accent); color: var(--on-accent);` · 상대 글 `opacity: 0.12;` · `color-mix(`.
  - 새 `bubble_contrast` — CSS 에서 토큰(`--surface` · `--text` · `--muted` · `--accent` · 해석 6 링크 토큰 · `--peer-1 ~ 5`, 라이트 = 이름 그대로 · 다크 = `--dark-*`)과
    두 `::before` 의 `opacity` 를 읽어 층을 합성하고, 2 테마 × (내 글 1 + 상대 글 5) × 글자 3(본문 · muted · 링크) = **36 경우**를 세어(`assert_eq!(36)`) 모두
    4.5:1 이상을 단정. 계약 수치도 절대값으로(`0.14` · `0.07`).
  - 고침 `icons_settings_literals` — 옛 말풍선 literal 두 줄(593 · 594)을 새 규칙으로 바꾼다(지우면 1l 계약이 빠진다 — 새 값으로 단정). `color-mix(` 없음은 그대로.
- 문서 — 사용자 가이드(사이드바 표 · 대화방 줄 · 말풍선 색 · 캡처 3장) · README 29 · an_app 매뉴얼(목록 나눔 · `splitByDir`) · PRODUCT(종결 때).

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 목록 나눔(`splitByDir` · `renderConvList` · 열쇠) · 머리 · 말풍선 음영 · 링크 색 · 레일 우클릭 | node · Rust 문구 · 대비 시험 | E1 ~ E6 |
| 2 | 관람자 — 통신(프레임 · Pong) · 스키마 20 · core `share` · 퍼뜨리기 · 사본 막기 · 페어링 해제 정리 | 세 앱 통합 시험 · 저장 시험 | E11 · E12 · E15 |
| 3 | 관람자 — API · 화면(공유 대화 묶음 · 읽기 전용 · 관람자 목록 · 나가기 · 알림) | API · node 시험 | E13 · E14 |
| 4 | 문서 · 캡처 · mock 확인(가상 데이터, 다크 · 라이트 — stage 기록) | 문서 · 이미지 | E7 ~ E10 · E16 |

## 평가 항목

- [ ] E1: A4·A5 — 워크스페이스 테스트 전부 통과(문구 · 대비 · 기존 시험 회귀 없음)
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A5 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2·A4·A5 — 화면 로직 node 테스트 108건 이상 · 실패 0(기존 104 + `conv_split` 4 — 검색 · 별 · 칩 · 끌기 기존 시험 포함)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (10[8-9]|1[1-9][0-9])\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A4 — 나눔 시험만 따로 4건 이상(내 대화 · agent 대화 · 자동 왕복 = 내 대화 · 빈 묶음 머리 숨김 · 상대 타일 / 그룹 / '나' 같은 규칙 · 다른 묶음 놓기 불가 · 즐겨찾기 그대로)
  CHECK: node --test --test-isolation=none --test-reporter=tap --test-name-pattern="^conv_split" crates/app/web/view.test.mjs
  EXPECT: /# pass [4-9]\s+# fail 0/
  EVIDENCE: pending
- [ ] E5: A1·A3·A4 — 문구 · 규칙 자리(머리 문구 · 접힘 열쇠 · 즐겨찾기 그대로 · 말풍선 층 0.14 / 0.07 · 오른쪽 띠, 없어진 것 0 — 강조색 꽉 채운 내 글 · 층 0.12 · color-mix)
  CHECK: cargo test -p coworkagent-app --test ux_literals chat_list_split
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A3 — 말풍선 글자 대비 36 경우(2 테마 × 내 글 · 상대 글 5색 × 본문 · muted · 링크) 모두 4.5:1 이상
  CHECK: cargo test -p coworkagent-app --test ux_literals bubble_contrast
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A5 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: pending
- [ ] E8: A5 — README 사용자 관점
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: pending
- [ ] E9: A5 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: pending
- [ ] E10: A1·A2·A3·A4 — 실화면 확인(실제 상대 대화, 다크 · 라이트): 상대 타일 · 그룹 · '나'(상대별 절 안) 목록에 "내 대화 n" · "agent 대화 n" 머리, 한쪽만이면 머리 없음 ·
  내가 연 자동 왕복이 내 대화에 · 끌기 / Alt + ↑ / ↓ 가 묶음 안에서만 · 접기 · 검색 · ☆ · 안 읽음 · 실행 중 · 칩 그대로 · 즐겨찾기 화면 그대로 · 말풍선 내 글 옅은
  강조색 + 오른쪽 띠 · 상대 글 옅은 타일 색 + 왼쪽 띠 · 링크 읽힘 · JS 오류 없음.
  EVIDENCE: pending

- [ ] E11: A7·A9·A10 — 세 앱 통합 시험(이력 · 새 글 · 답 · 편집 · 삭제 따라감 · 꺼진 관람자 보관 · 빼기 · 나가기 뒤 안 감 · 사본에 실행 없음 · 위조 Share 거절 · 구버전 거절)
  CHECK: cargo test -p an-agent --test share
  EXPECT: /test result: ok\. ([6-9]|[1-9][0-9]) passed; 0 failed/
  EVIDENCE: pending
- [ ] E12: A8 — 원 상대 알림(관람자 수 · 이름 없음)
  CHECK: cargo test -p an-agent --test share notice
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E13: A7·A9·A10 — API(관람자 추가 · 빼기 · 나가기 · 사본 거절 · B 본인 거절 · 구버전 409 · 대화 JSON 필드)
  CHECK: cargo test -p coworkagent-app --test chat_api share
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E14: A7·A10 — 화면 로직(공유 대화 묶음 · 읽기 전용 · 발화자 · 자동 왕복 표시 없음) node 3건 이상
  CHECK: node --test --test-isolation=none --test-reporter=tap --test-name-pattern="^share_" crates/app/web/view.test.mjs
  EXPECT: /# pass [3-9]\s+# fail 0/
  EVIDENCE: pending
- [ ] E15: A10 — 스키마 20 올리기(19 → 20 · 기존 대화 dir 그대로)
  CHECK: cargo test -p an-core --test store share
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E16: A6·A7·A8·A9 — 실화면 확인(실제 세 PC): 레일 빈칸 · 타일 우클릭에 브라우저 메뉴 없음 · 관람자 추가 → 관람자 PC "공유 대화" 에 이력 · 새 글 따라옴 · 입력창 없음 ·
  원 상대 "관람자 n" 알림 · 빼기 · 나가기 뒤 읽기 전용으로 남음.
  EVIDENCE: pending

A5 의 `check_gates` 는 구현 완료 시점에 `eval.md` 평가 원장을 만드는 것으로 충족한다(자기 원장을 읽는 CHECK 는 두지 않는다 — eval-protocol 규약 2).

## 종결 시 할 일

- E10 · E16 은 D28 에 따라 `ABANDON`(사유: D28) + 19.prototype-acceptance 에 TC 추가(위 화면 확인 항목 — 실제 상대 · 두 · 세 PC 대화로).
- masterPlan: D26 · D35 · S4 보강(사이드바 목록 내 대화 · agent 대화 나눔 · 말풍선 음영 Ⓐ) · 관람자 초대(새 D — 공유 대화 · 읽기 전용 · 원 상대 알림) · D7(새 프레임) ·
  D32(스키마 20) · 개정 기록. PRODUCT 91 ~ 94 · 102 말풍선 · 목록 갱신. 원장 없음(예정).
- 이 PC 반영(릴리스 빌드). GitHub 반영 · 릴리스는 사용자 확인 뒤.

## 개정 기록

- 2026-10-17 · intent 개정(레일 우클릭 · 관람자 초대 — 사용자 지시 1016-cowork.md · 인터뷰 답)에 따라 "개정 — 관람자 초대 · 레일 우클릭" 절(해석 11 ~ 22) · Stage 2 · 3
  추가(옛 Stage 2 → 4) · 평가 항목 E11 ~ E16 추가 — 사유: intent 성공 기준 A6 ~ A10 · 승인 2026-10-17(사용자 "관람자 수 상한 : 5명. 관람자 PC 는 view 만 제공. 편집, 삭제 없음. '공유 해제됨' 태그만 추가. UI 추가수정 사항. 내용이 곧 설명이되는 매뉴에 대해서는 매뉴에 대한 설명을 제거한다." — 상한 5 · 보기만 · '공유 해제됨' 반영)
- 2026-10-17 · 해석 22 레일 우클릭은 Stage 1 에서 먼저 구현했다(`3550dd5`) — 사유: 화면 한 줄 · 관람자와 독립 · 승인 2026-10-17(같은 지시)
- 2026-10-17 · 해석 23 설명 지우기 추가(Stage 1) — 사유: 사용자 지시(intent 결정 둘째 지시) · 승인 2026-10-17(사용자 지시 자체)
- 2026-10-17 · 해석 18 공유 대화 말풍선 — 공유한 사람 쪽 글 오른쪽(처음 구현은 모두 왼쪽) — 사유: 사용자 "공유한 사람의 글을 오른쪽에 둠. 대화가 오가는 순서를 알기 쉽게 하기 위함." · 승인 2026-10-17(사용자 지시 자체)
- 2026-10-17 · 해석 14 "포기하지 않음" → 관람자에 24시간 닿지 않으면 그 관람을 끝낸다(공유 행 지움 · 관람자에서 뺌 · 빼기 표시 한 행은 남겨 돌아오면 "공유 해제됨" · 원 상대 알림 행은 버림), 보내기 바로 앞 관람자 확인 — intent A9 "꺼진 관람자에게는 보관했다 보낸다" 를 24시간으로 한정 — 사유: 적대적 평가 라운드 1 · 2(꺼진 관람자 몫이 끝없이 쌓임 · 빼기와 보내던 중 글이 겹치면 뺀 뒤에도 감) · 승인 2026-10-17(사용자 "관람자에 한해서는 24시간 한정 승인")

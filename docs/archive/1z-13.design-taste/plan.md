# plan — 1z-13.design-taste

> 근거: [intent](intent.md)(승인 2026-10-25 — 결정 1 ~ 14, U8 은 이 plan 의 stage 3). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md)(D26 · D35 · S4).
> 브랜치 `task/1z-13-design-taste`(원 작업 폴더 — worktree 없음). 기준 커밋: `c3568f5`(main — 이 브랜치의 코드 기준, 그 뒤 커밋은 문서만). plan 승인: 2026-10-25(사용자 질문 답 "승인 · 구현 시작" — P1 ~ P3 권장안 · P4 포함).
> 고른 시안: Superdesign ① Graphite `78afba99` v3(지금 화면 재현 `e0cb3783` v1 과 비교) — https://superdesign.dev/teams/be614f5b-4a18-4a10-b849-a7384eb0ecc3/projects/da9a49e6-a34f-44d5-8806-98b3e90d2ca0

## 목표

앱 전체를 Graphite 언어로 바꾼다 — 흑연 무채색 + 청록 강조 하나, 주 단추 = 흑연, 사람 아이콘 아바타 → 머리글자 네모, 카드 대신 선, 다크도 같은 언어로 다시 칠함.
큰 틀(상단 바 · 레일 · 사이드바 · 본문 · 세부정보) · 기능 · 화면 문구 · HTTP API · DB 스키마는 그대로. 버전 1.3.7.

### 성공 기준(intent 사본)

- A1: AI 티 0 — taste 목록(이모지 · 보라 / 파랑 네온 · 그라데이션 · 과한 그림자 · 같은 카드 3열 · 순검정 · 채도 높은 강조 · 상투 문구)이 화면에 없고, 사용자가 짚은
  파란 강조 · 사람 아이콘 아바타 · 단추 색 · 배치가 고른 시안대로 바뀐다.
- A2: 강조색 하나 — 주 단추는 무채색, 강조색은 상태 · 선택에만(토큰 표는 plan).
- A3: 동작 · 문구 그대로 — 기존 시험 통과(바뀌는 시험은 시안이 바꾼 마크업 · 토큰 행만 — 행마다 근거).
- A4: 실제 화면 — 모든 화면 · 라이트 · 다크가 고른 시안과 같은 언어, 콘솔 오류 0.
- A5: 회귀 없음 — PROJECT.md 검증 명령 PASS, 매뉴얼(web-ui 화면 규칙) 반영.

## 설계

### 공통 규칙

- 값은 `app.css` 첫 `:root` 토큰만(`tests/css_tokens.rs`). 새 토큰 = 라이트 이름 + `--dark-*` + `:root[data-theme="dark"]` 대입 줄, 세 곳을 함께.
- 제품 원본(`app.css` · `index.html` · `js/`) 주석에 Task 번호 · 시안 · 결정 번호 · 날짜를 쓰지 않는다(`tools/check_code_tidy.py`). 주석은 왜만.
- 새 함수는 둘레 규약대로(모듈 `export` · 시험 파일의 기존 함수 옆) — 파일 전용 static 함수를 새로 만들지 않는다.
- 동작 · 모양 설명이 바뀌면 같은 개념을 말하는 주석 · 매뉴얼을 저장소 전체에서 찾아 같이 고친다.
- TDD — 시험(`ux_literals.rs` 행 · `view.test.mjs`)을 먼저 고쳐 RED 를 본 뒤 구현.

### 토큰 표

| 토큰 | 라이트 | 다크 | 쓰임 |
|---|---|---|---|
| `--bg` · `--sidebar` | #F6F6F7 | #111113 | 앱 바탕 · 사이드바 · 세부정보 · 설정 트리 · 개요 미리보기 바탕 |
| `--topbar` | #FFFFFF | #0C0C0E | 상단 바 |
| `--surface` | #FFFFFF | #18181B | 본문(메인 바탕도 이것 — 지금 `--bg`) · 면 |
| `--surface-2` | #F0F0F1 | #26262A | 올림(마우스) · 무채 고른 칸 |
| `--subtle` | #F4F4F5 | #1F1F23 | 사람 글 면 · 코드 · 읽기 전용 칸 · 세그먼트 바탕 |
| `--border` | #E4E4E7 | #2E2E33 | 가는 선 |
| `--border-strong`(새) | #D4D4D8 | #3F3F46 | 단추 · 입력창 테두리 |
| `--text` · `--muted` · `--faint` | #18181B · #52525B · #6B6B73 | #E4E4E7 · #A1A1AA · #8E8E97 | 글자(`--faint` 는 시안 #71717A 보다 진하게 — 올림 바탕 대비 4.5:1) |
| `--primary` · `--primary-hover` · `--on-primary`(새) | #27272A · #3F3F46 · #FAFAFA | #E4E4E7 · #D4D4D8 · #18181B | 주 단추 |
| `--accent` · `--accent-hover` · `--accent-soft` · `--on-accent` | #2F6F6B · #255955 · #E7F0EF · #FFFFFF | #6FB3AC · #8CC5BF · #1A2B29 · #0B1413 | 강조 하나(채도 40.5 % · 30.9 %) — 고른 것 · 안 읽음 · 켬 · 초점 · 링크 · 에이전트 글자(P1) |
| `--agent` · `--agent-soft` | `var(--accent)` · `var(--accent-soft)` | (따라감 — `--dark-agent*` 지움) | 에이전트 표시 |
| `--run` · `--ok` · `--danger` · `--wait` | #8A6A2F · #3F7A52 · #A34A40 · #6B6B73 | #D6AE6B · #7FB894 · #E08A7E · #8E8E97 | 상태(차분한 색) |
| `--sent-bg` · `--recv-bg` / `--sent` · `--recv` | #F4F4F5 / #3F3F46 | #1F1F23 / #D4D4D8 | 범주 칩 · 내 활동 탭 → 무채색(P4) |
| `--answer-bg` · `--danger-soft` | #E7F0EF · #F6E9E7 | #1A2B29 · #3A1F1C | 받은 답 · 실패 칩 바탕 |
| `--rail` · `--rail-2` · `--rail-text` | #18181B · #27272A · #A1A1AA | #0C0C0E · #26262A · #A1A1AA | 레일(두 테마 모두 어둡다) |
| `--avatar-bg` · `--avatar-text`(새) | #E4E4E7 · #3F3F46 | #2E2E33 · #D4D4D8 | 머리글자 네모 |
| `--agent-tile` · `--peer-1 ~ 5` | #2F6F6B · #496A9A · #2F6B62 · #7B598B · #905A3A · #2F6577 | 같음 | 에이전트 아이콘 색 6(사용자가 고름 — 기능이라 둔다). 채도 45 % 아래 · 흰 아이콘 4.5:1 |
| `--shadow` · `--shadow-sm` | 0 4px 16px rgba(24,24,27,.10) · 0 1px 2px rgba(24,24,27,.10) | 지금 값 | 팝오버 · 메뉴 / 스위치 손잡이 · 끄는 줄만(멈춘 면에 그림자 없음) |
| `--radius-bubble` | 8px | | 말풍선(지금 12px) |
| 지움 | `--pv-*` 색 28 · `--dark-pv-*` 28 · 그 대입 28줄 · `--bubble-mine-alpha` · `--bubble-theirs-alpha` · `--dark-agent` · `--dark-agent-soft` | | 미리보기는 공용 토큰, 말풍선 층 없음 |
| `--conv-in` · `--conv-out` · `--conv-shared` | stage 3 에서 정함 | | 대화 종류 |

### stage 1 — 토큰 · 공통 부품 · 단추 자리

- 토큰 = 위 표(라이트 · 다크 · 대입 줄). `--pv-*` 크기 · 간격 토큰(`--pv-fs-*` · `--pv-*-pad` · `--pv-radius` …)은 둔다(#1z-11 크기 그대로).
- 단추: 기본 = `--surface` + 1px `--border-strong`, 올리면 `--surface-2` 바탕(지금 강조색 테두리), `.primary` = `--primary` · `--on-primary`(올리면 `--primary-hover`), `.danger` 그대로.
  상단 '새로 만들기'(`#new_btn`)는 `primary` 를 빼 기본 단추(테두리 · 흑연 글자) — 자리는 지금처럼 종 왼쪽(시안).
- 입력: 테두리 `--border`, 초점 2px `--accent` 그대로. 스위치 켬 · 메뉴 체크 = `--accent` 그대로.
- 선택 표시(P2 권장): 사이드바 메뉴 · 대화 줄 · 설정 트리 · 목록 줄 = `--accent-soft` 바탕 + `--accent` 글자, 막대 · 안쪽 테두리 없음.
- 사이드바: 바탕 `--sidebar`(`#sidebar { background: var(--subtle) }` 덮어쓰기 지움). 대화 줄 = 카드 없는 납작한 줄(흰 면 · 테두리 · 8px 둥글기 · 유형 테두리 지움) — 줄 높이 36px 안팎,
  올리면 `--surface-2`, 손잡이 · ⋯ 자리 그대로.
- 대화 머리: `.seg`(대화 보기 · 원문 보기) = 글자 탭 — 테두리 · 바탕 없음, 켠 쪽 `--text` · 500 + 아래 2px `--text` 밑줄. `.tabs`(내 활동 · 요청 목록 탭)도 같은 글자 탭.
  `.tool-btn`(컨텍스트 · 북마크 · 세부정보) = 테두리 없는 단추(`--muted` 글자, 올리면 `--surface-2` · `--text`, 켬 = `--accent` 글자). 머리 아래 유형 색 2px 선 지움(1px `--border` 그대로).
- 말풍선: 둥글기 8px, 사람 글(내 글 · 상대 사람 글) = `--subtle` 면 · 테두리 없음, 에이전트 글 = `--surface` 면 + 1px `--border`(지금 1.5px 에이전트 색).
  '에이전트' 표시(`.tag.agent-tag`) = 바탕 없는 `--accent` 작은 글자(P1). 말풍선 색 층(`.msg[data-tone]` · `::before` 둘 · 내 글 링크 `--sent`) 지움 — 링크는 전역 `a`.
- 입력창: 1px `--border-strong` · 그림자 없음, 초점 = `--accent` 테두리(시안의 회색 고리 대신 — 초점 = 강조색 규칙). 바 순서 = 서식(B · 목록 · 코드) | 키 안내 · 질의 / 요청 · 보내기(`index.html` 에서
  `kbd-hint` 를 `ask-kind` 앞으로). 질의 / 요청 세그먼트 = 바탕 `--subtle`, 고른 칸 = `--surface` + 1px `--border-strong` 고리(그림자 없음).
  글자 줄바꿈 뿌리: `.ask-kind span` 이 안쪽 아이콘 `span` 에도 걸려 아이콘이 `0 12px` 여백을 받는다 → 선택자를 `.ask-kind label > span` 으로, 단추 · 칸 글자 `white-space: nowrap`.
- 세부정보: 절 제목 아이콘 `--faint`. 대화 정보(`.info`) = 상자 없음 — 항목마다 아래 1px `--border`. 북마크 목록(`.saved-list a`) = 상자 없는 줄 + 아래 선. 자동 왕복 상자(`.relay-box`)는 둔다(묶음 하나).
- 설정: 2열 타일(`.set-tile`) = 상자 없음(바탕 · 테두리 · 둥글기 지우고 아래 1px 선, 2열 격자 그대로). 목록 · 상세 2단(`.md-list` · `.md-detail`)은 상자 그대로(묶음). 제목 아이콘(`.set-head h2 .ico.lead`) =
  `--muted`(지금 강조색 — P4). 상세 아래 단추 줄(P3 권장) = 오른쪽 끝 한 묶음, 순서 그대로 — `mdFoot` 은 빈칸(`grow`)을 앞으로, `index.html` 정적 줄 셋(초대 생성 · 초대 수락 · 에이전트 2단계)도
  빈칸을 앞으로(에이전트 1단계는 왼쪽 안내 글 그대로).
- 레일: 타일 = 머리글자 네모(stage 2) — `--avatar-bg` · `--avatar-text`, 나 = `--rail-2` · `--on-dark`, 고른 타일 = 2px `--accent` 윤곽선 그대로, 안 읽음 배지 `--accent`, 접속 점 오른쪽 아래.
- 프로필 단추: 알약 테두리 · 상태 고리 없음 — 24px 머리글자 네모(그림이면 그림) + 상태 점 + 이름(지금 글자 그대로).
- 메인 바탕 `--surface`. 개요 미리보기(`.pv-*`) = 공용 토큰 — 바탕 `--bg`, 카드 `--surface` + 1px `--border`, 글 `--text` · `--muted` · `--faint`, 링크 · 새 글 점 · 안 읽음 테두리 `--accent`,
  사람 글 `--subtle`, 에이전트 · 실행 중 글 `--surface` + 1px `--border` · 이름 `--accent`, 오류 `--danger-soft` · `--danger`, 상태 점 `--run` · `--ok` · `--danger` · `--wait`, "+n개" 점선 상자 = 1px 실선 `--border`.
- 범주 색 무채색(P4): 내 활동 탭 · 칩(받은 요청 · 내 요청 · 받은 답)은 `--sent-bg` · `--sent` 무채색 + 지금 아이콘, 실패만 `--danger`. 개요 목록 줄 유형 테두리(`.rows li[data-kind]`) 지움.
- 그림자: 팝오버 · 메뉴 · 끄는 줄만 `--shadow`, 입력창 · 세그먼트 그림자 지움.

### stage 2 — 머리글자 타일 · 아바타(결정 12)

- `logic/peer.js` `initialOf(name)` — 앞뒤 공백을 뺀 첫 글자(서로게이트 쌍도 한 글자), 로마자는 대문자, 빈 이름 = 빈 글자. `colorOf` 지움(쓰는 곳이 없어진다).
- `core/widgets.js`: `tileFace(icon, name)` = 그림이 있으면 그림, 없으면 `tile-shape` 에 `initialOf(name)` — 사람 · 그룹 모양 SVG(`PERSON_SVG` · `GROUP_SVG`) · `tileColor` 지움,
  `rowTile(cls, icon, name)`. 에이전트 타일(`agentTile`)은 로봇 아이콘 + 고른 색 그대로.
- 부르는 곳: `rail.js`(나 = 표시 이름 · 상대 = 표시 이름 · 그룹 = 이름, 타일 class 의 상대 색 지움), `tiles.js`(편집 미리보기 = 편집 칸 이름 — 비면 원래 이름, 이름 칸 입력마다 다시 그림 · 색 class 지움),
  `peers.js` · `home.js` · `settings/conn.js` · `settings/groups.js` · `search.js`(상대 에이전트 타일 = 무채 `pa` + 로봇), `shell.js` `renderAvatar`(그림 없으면 머리글자),
  `conv.js`(사람 아바타 = `initialOf` · 상대 색 class · `data-tone` 지움, 에이전트 아바타 = 로봇 그대로 — P1).
- CSS: `.tile-shape` · `.pa` · `.room-av` · `.msg .avatar` · `.avatar-face` = `--avatar-bg` · `--avatar-text` · 500 · `--radius-sm`(원 → 네모), 크기 레일 작은 타일 24 · 큰 타일 40 · 메시지 24 · 프로필 24 · 목록 26 / 36.
  `.peer-N` 규칙은 에이전트 아이콘 색으로만 남는다.

### stage 3 — 대화 종류 표시(intent U8)

지금 = 사람(내가 문의) · 로봇(내 에이전트 응답) 아이콘을 종류 색 면에 채운 칩 + 사이드바 줄 · 개요 목록 줄 테두리 · 대화 머리 아래 선 = 종류 색, 공유는 점선. 사람 아이콘을 빼면 표시가 사라지므로 대체를 고른다.

1. 후보 셋을 실제 앱(디버그 빌드 + 시험 데이터)에 ego-browser 로 CSS · DOM 만 얹어(제품 코드 안 바꿈) 사이드바 줄 · 대화 머리 · 개요 목록 · 미리보기 카드 · 검색 결과를 찍어 아티팩트로 보인다.
   - (가) 선 아이콘만 · 색 없음(`--faint`) — 응답 = 로봇 · 문의 = 위 화살표(내 활동 '내 요청' 과 같은 뜻) · 공유 = 눈, 채운 칩 · 종류 테두리 · 점선 없음. **권장**(아이콘 유지 · 색 하나).
   - (나) 글자만 — 줄 끝 시각 앞 · 머리 제목 뒤 "문의" · "응답" · "공유"(12px `--faint`), 아이콘 없음.
   - (다) 시안 그대로 — 상대 머리글자 네모, 종류 표시 없음(비교용 — 정보가 사라진다).
2. 사용자가 고르면 `kindChip` · `convKindIcon` · `[data-kind]` 규칙 · `--conv-*` 를 그 표시로 바꾸고 고른 표시 · 바뀌는 시험 행을 이 plan 개정 기록에 적는다(설계만 — 평가 항목 · EXPECT 그대로).

### stage 4 — 실제 화면 · 문서 · 버전

- 실제 화면(E7, 결정 13): 디버그 빌드 + 시험 데이터(scratchpad 사본 + 시드 — 상대 3 · 그룹 2 · 대화 · 안 읽음 · 실행 중 · 오류 · 요청 · 북마크 · 공유), ego-browser 1440 × 900,
  화면 23 × 라이트 · 다크 사진 — 개요(미리보기 · 목록) · 내 활동 · 요청(목록 · 보드) · 즐겨찾기 · 북마크 · 등록된 에이전트 · 에이전트 응답 · 상대 대화 목록 · 대화(세부정보 · 원문) ·
  주고받은 메시지 · 새로 만들기 메뉴 · 세션 생성 · 검색 · 프로필 메뉴 · 설정 8(연결 · 그룹설정 · 에이전트 · 실행 · 화면 · 시작 · 알림 · 디버깅). 콘솔 오류 0, DOM 판독(사람 SVG 0 · 주 단추 바탕 =
  `--primary` 계산값 · 타일 머리글자 · 대비). 사진은 아티팩트로 사용자에게 보이고 고칠 곳을 받아 반영한다.
- 문서: [web-ui](../../manual/dev/web-ui.md)(디자인 토큰 표 · AI 티 목록 — 파란 강조 · 사람 아이콘 아바타 · 채운 주 단추 · 카드 줄 · 범주 색을 더함 · 레일 타일 · 프로필 단추 · 대화 줄 ·
  머리 · 입력창 · 단추 줄 · 대화 종류) · [사용자 가이드](../../manual/user/user-guide.md)(타일 · 색 설명 문단) · PRODUCT(#1z-13 줄 — 다른 세션의 미커밋 2줄과 섞지 않고 이 줄만 스테이징) ·
  masterPlan 환류(종결 — D26 토큰 · D35 / S4 레일 타일 · 대화 줄 · 미리보기 · 대화 종류 · AI 티 목록) · 개정 기록.
- 버전 1.3.7 — `crates/{core,net,agent,app}/Cargo.toml` · `Cargo.lock`.

### 시안과 다른 곳(근거)

- `--faint` #71717A → #6B6B73: 올림 바탕(#F0F0F1) 대비 4.24 → 4.64.
- 입력창 초점: 시안 회색 고리 → `--accent` 테두리(초점 = 강조색, 다른 입력과 같게).
- 말풍선 작업 아이콘(북마크 · 복사): 시안의 말풍선 위 떠 있는 막대 대신 지금 자리(말풍선 안 오른쪽 위 · 올릴 때) — 시안 화면에 보이지 않는 상태라 바꾸지 않는다.
- 선택 줄 · 에이전트 아바타 · 단추 줄 · 범주 색은 P1 ~ P4.

### plan 해석 — 승인 때 확인(P1 ~ P4)

2026-10-25 사용자 답: P1 "로봇 아이콘 + 청록 글자" · P2 "막대 없이 옅은 청록 바탕" · P3 "오른쪽 한 묶음 · 순서 그대로" · P4 승인에 포함.


- P1 에이전트 표시 — 시안은 에이전트 아바타 = 머리글자("p") · '에이전트' = 청록 글자. 결정 11(아이콘 유지)과 결정 5(강조는 상태 · 선택에만)에 걸린다.
  권장: 아바타 = 로봇 아이콘(무채 네모), '에이전트' = 청록 작은 글자(시안) — 사람 ↔ 에이전트 구분이 이 앱의 핵심 정보(#1z-7).
- P2 선택 줄 — 시안은 왼쪽 2px 청록 막대 + 옅은 회색. 매뉴얼 'AI 티로 쓰지 않는 것'(#1p — 선택 안쪽 띠)과 부딪친다. 권장: 막대 없이 `--accent-soft` 바탕 + `--accent` 글자(지금 방식, 색만 바꿈).
- P3 설정 · 상세 단추 줄 — 지금 = [주 단추 · 작업] ······ [닫기](양 끝으로 벌어짐). 권장: 오른쪽 끝 한 묶음, 순서 그대로([수락][닫기] · [이름 바꾸기][삭제][닫기]). 대안: macOS 순서([닫기][주 단추]).
- P4 결정 5 로 바뀌는 이전 색 규칙 — 사람 · 그룹 타일 5색(#1j · S4 '타일 색 = 상대 id') → 한 색 머리글자, 내 활동 종류 색(#1q) → 무채색 + 아이콘(실패만 위험 색),
  대화 종류 색(#1z-7) → stage 3, 상대 사람 글 주황 면(#1o) → 사람 글 = 옅은 회색 면, 설정 제목 아이콘 강조색(#1z-10) → 무채색, 말풍선 색 층(#1p) → 없음,
  글자 타일 없음(#1j) → 머리글자 네모(결정 12), 미리보기 종이 톤(#1z-11) → 공용 토큰(결정 6). 에이전트 아이콘 색 6(사용자가 고름)은 둔다.

### 바꾸는 시험(의도된 변경 — 약화 아님)

- `ux_literals.rs`:
  - 바뀜: `tileFace(g.icon, true)` → `tileFace(g.icon, g.name)`(머리글자) · 내 글 / 상대 글 바탕(360 · 387 · 389) → 사람 글 `--subtle` · 에이전트 글 `--surface` + 선 · 설정 제목 아이콘(499) → `--muted`.
  - 지움 → ABSENT: 말풍선 층(358 · 359 · 388) · 내 글 링크 `--sent`(390) · 머리 아래 종류 선(454) · 줄 종류 테두리(455).
  - stage 3 이 정함: 184 · 448 · 452 · 458 · 460(종류 칩 · 문의 색 · 미리보기 종류 색).
  - ABSENT 546 · 547 · 549(#1j 글자 타일 없음) 지움 — 결정 12 로 되돌림.
  - 새 PRESENT: `button.primary { border: 0; background: var(--primary); color: var(--on-primary);` · `--border-strong` · `--avatar-bg` · `initialOf` · `.ask-kind label > span` · 바 순서(키 안내 → 질의 / 요청) ·
    `mdFoot` 빈칸 앞. 새 ABSENT: `id="new_btn" class="primary"` · `PERSON_SVG` · `GROUP_SVG` · `colorOf(` · `data-tone` · `--bubble-mine-alpha` · `--pv-bg` · `#sidebar { background: var(--subtle); }` ·
    `.side-list li:not(.hint) { background: var(--surface); border:` · `.ask-kind span { display: inline-flex; align-items: center; height: 24px; padding: 0 var(--sp-3);`.
  - `bubble_contrast`(말풍선 층 36 경우) → `token_contrast`: 두 테마 × 32 짝(글자 3 × 바탕 4 ~ 5 · 강조 글자 · 주 단추 · 배지 · 레일 · 머리글자 · 상태 · 위험 · 범주 칩) + 에이전트 아이콘 6색 흰 아이콘 = 70 경우 4.5:1 이상.
- `view.test.mjs`: `colorOf` 줄 → `initialOf`(한글 · 로마자 대문자 · 공백 · 서로게이트 쌍 · 빈 이름), `convKindIcon` 세 줄은 stage 3, `AGENT_COLORS` 그대로.
  `mdFoot` 단추 순서는 `ux_literals` PRESENT 행(`settings/index.js` 는 DOM 을 쓰는 모듈이라 node 시험 밖 — 개정 ①).
- `css_tokens.rs`: 그대로(바뀐 CSS 도 규칙 안).

### 리뷰 초점

- 큰 틀 · 기능 · 문구가 그대로인가(라벨 비교 E6, 숨은 기능 손실 — 타일 그림 · 에이전트 색 · 상태 점 · 안 읽음 · 공유 표시).
- 강조색이 상태 · 선택 · 초점 · 링크 · 에이전트 글자 밖에 쓰였나, 멈춘 면 그림자 · 카드 줄이 남았나.
- 다크 대비 · 다크 대입 누락(새 토큰 셋 자리), 지운 토큰을 아직 쓰는 곳.

## 평가 항목

- [ ] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A3 — 웹UI 로직 시험(머리글자 · 단추 줄 순서 · 대화 종류)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A3 — 화면 원본 literal(주 단추 흑연 · 머리글자 · 입력창 순서 · 지운 층 · 종류 표시) · 토큰 대비 70 경우 · `:root` 밖 리터럴 0
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A1·A2 — AI 티 검사: 웹UI 원본 50개에서 그라데이션 · 순검정 · 이모지 · 사람 아이콘 아바타 · 옛 파랑 · 강조 채도 45 % 이상 · 주 단추가 흑연 아님 · 옆 띠 / 자간 / 글자 기호 단추 · 선택 줄 막대 0
  CHECK: python {TASK}/design/taste.py
  EXPECT: taste files=50 checks=9 violations=0
  EVIDENCE: pending
- [ ] E6: A3 — 문구 그대로: `index.html` 한글 글(글 마디 · aria-label · title · placeholder)이 기준 커밋 `c3568f5` 와 같다
  CHECK: python {TASK}/design/labels.py c3568f5
  EXPECT: /labels n=[1-9][0-9]* missing=0 added=0/
  EVIDENCE: pending
- [ ] E7: A1·A4 — 실제 화면: 디버그 빌드 + 시험 데이터 · ego-browser 1440 × 900 — 화면 23 × 라이트 · 다크가 Graphite 언어(흑연 주 단추 · 청록 강조 하나 · 머리글자 네모 · 카드 대신 선 ·
  고른 대화 종류 표시), 콘솔 오류 0, 사람 SVG 0, 사용자가 사진(아티팩트)을 보고 확인
  EVIDENCE: pending
- [ ] E8: A5 — 문서 규약 · 크레이트 경계 · 코드 정리 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_code_tidy.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('web=%d guide=%d product=%d'%(r('docs/manual/dev/web-ui.md').count('--primary'),r('docs/manual/user/user-guide.md').count('머리글자'),r('PRODUCT.md').count('#1z-13')))"
  EXPECT: /web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]*/
  EVIDENCE: pending
- [ ] E9: A5 — 버전 1.3.7(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v137=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.3.7' for t in v)))"
  EXPECT: crates=4 v137=4
  EVIDENCE: pending

## 개정 기록

- 2026-10-25(승인과 같은 커밋 — 설계만, 평가 항목 · EXPECT 그대로): ① `mdFoot` 단추 순서 시험 = `view.test.mjs` → `ux_literals` 행(`view.test.mjs` 는 `js/logic/` 만 — DOM 없음).
- 2026-10-25(stage 1 — 설계만, 평가 항목 · EXPECT 그대로): ② 대화 줄(카드 · 종류 테두리 · 점선) · `#sidebar` 바탕 덮어쓰기 · 대화 머리 종류 선은 stage 3 으로 — 줄 테두리를 먼저 지우면
  공유 줄 `border-style: dashed` 가 기본 굵기 테두리로 살아난다. ③ 개요 미리보기 목록 바탕 = `--surface`(표의 `--bg` 대신 — 메인이 흰 면, 사용자 지시 "더 연하게" 그대로).
  ④ 단추 올림 = 테두리 `--faint`(바탕 `--surface-2` 대신 — 색 바탕 단추가 올릴 때 제 색을 잃지 않게).
- 2026-10-25(stage 2 — 설계만, 평가 항목 · EXPECT 그대로): ⑤ `initialOf` 첫 글자 = 글자 묶음(`Intl.Segmenter`) — 서로게이트 쌍에 더해 결합 문자 · 풀어 쓴 한글(NFD)도
  한 글자(`Array.from` 은 'e + ◌́' · 'ᄀ + ᅵ + ᆷ' 의 첫 코드 포인트만 남긴다). ⑥ `.room-av` 는 대화 종류 칩(`kindChip`)에만 쓰여 stage 3(intent U8)으로 —
  지금 바꾸면 후보 비교 사진의 바탕이 흔들린다. ⑦ AI 티 검사 `person-icon-avatar` 에 대화 줄 종류 칩의 사람 아이콘(`"out" ? "user"`)을 더함 — intent 표의
  '대화 줄 · 레일 타일 · 사이드바 머리가 같은 사람 아이콘' 을 E5 가 센다(stage 2 뒤 1 — stage 3 이 0 으로).
- 2026-10-25(stage 3 — 설계만, 평가 항목 · EXPECT 그대로): ⑧ 대화 종류 표시 = (가) 무채 선 아이콘(intent 결정 15 — 사용자 질문 답, 실제 앱 비교 아티팩트).
  `convKindIcon` = 응답 `bot` · 문의 `arrow-up` · 공유 `eye`, `KIND_NAMES.shared` = "공유 대화"(사이드바 절 머리와 같은 말), `.kind-chip` = `--faint` 글자만,
  `--conv-*` · `[data-kind]` 색 변수 · 종류 테두리 · 점선 · 머리 종류 선 · `--answer-bg`(쓰는 곳이 사라짐) 지움, 사이드바 줄 클래스 `room-av agent-av` → `room-kind`.
  바뀌는 시험 행: `ux_literals` 바뀜 2(사이드바 `kindChip` · `KIND_NAMES`) · 지움 8(옛 `convKindIcon` · `--conv-*` 셋 · 머리 선 둘 · 줄 테두리 · 미리보기 아이콘 색) ·
  새 PRESENT 2 · 새 ABSENT 7, `view.test.mjs` `conv_kind_icon` 기대값.
- 2026-10-25(stage 4 E7 답 — 설계만, 평가 항목 · EXPECT 그대로): ⑨ 실제 화면 = 목록대로 25곳(stage 4 의 "화면 23" 은 같은 줄 목록을 센 수와 다름 — 개요 · 요청 · 대화가
  둘씩), '상대 대화 목록' = 상대 타일 + 공유 대화를 연 화면. 사용자 답(https://claude.ai/artifact/Ndz7qpArMaEjgDwf845CfT): 대화 종류 문의(`out`) = 사람 선 아이콘(intent 결정 16 — `convKindIcon` `"user"`),
  stage4-1 요청 보드 칸 위 3px · 카드 왼쪽 4px · 의견 상자 왼쪽 3px 상태 색 띠 지움(상태 = 칸 제목 · 칩) · AI 티 검사 '옆 띠' 를 위 띠 · 상태 색(`--run` · `--ok` · `--danger` ·
  `--wait` · `--muted`)까지 넓힘(E5 가 `--tone` · `--accent` 만 찾아 놓쳤다), stage4-2 메뉴 안 켜기 항목(프로필 '알림 일시 중지') = 설정 화면 스위치 모양(36 × 20 · 손잡이),
  stage4-3 내 활동 발화자 사람 아이콘 그대로("사람은 사람 아이콘으로 사용"). 개정 ⑦ 되돌림 — `person-icon-avatar` 에서 대화 종류 칩의 사람 아이콘을 뺀다(아바타가 아닌
  종류 표시 · 무채 선, 결정 16). 바뀌는 시험 행: `ux_literals` 바뀜 1(`convKindIcon`) · 새 PRESENT 1(메뉴 스위치 손잡이) · 새 ABSENT 4(띠 셋 · 손잡이 없는 메뉴 알약),
  `view.test.mjs` `conv_kind_icon` 기대값.
- 2026-10-26(stage 4 — 설계만, 평가 항목 · EXPECT 그대로): ⑩ stage2-3(`conv_leave::delete_while_waiting_leaves_no_rows` 간헐 실패 — 전체 실행 4번 중 3번)을 이 Task 에서
  고친다 — 사용자 답 "이 Task 에서 시험만 고침"(앞 답 "다음 Task" 개정, E1 을 막음). 원인 = 글 상태(실행 중)가 실행기 호출보다 먼저 바뀌어, 시험이 앞 대화 실행이 실행기에
  닿기 전에 실행 수를 쟀다. 고침 = 실행기 기록 1건을 기다린 뒤 세고, 지운 대화 글이 실행기에 닿지 않았음을 따로 확인(제품 코드 그대로).
- 2026-10-26(평가 R1-1 — 설계만, 평가 항목 · EXPECT 그대로): ⑪ 실제 쓰는 대비 짝이 `token_contrast` 밖이었다 — 연결 끊김 프로필 머리글자 `--faint` / `--avatar-bg`
  4.16 · 4.16, 컨텍스트 경고 글자 + 도구 단추 올림 `--run` / `--surface-2` 라이트 4.41. 고침 = 연결 끊김 머리글자 흐림 지움(`--avatar-text` 그대로 — 상태는 점 색) ·
  라이트 `--run` #8A6A2F → #80632B(위 토큰 표 값 개정), 짝 넷 더함(`run`/`surface-2` · `danger`/`surface-2` · `run`/`subtle` · `ok`/`subtle` — 요청 태그 칩) → 두 테마 × 36 짝 + 6 = 78 경우
  (E4 제목의 '70 경우' 는 승인 때 수 — 시험은 늘기만 함), `ux_literals` 새 ABSENT 1(연결 끊김 머리글자 흐림).

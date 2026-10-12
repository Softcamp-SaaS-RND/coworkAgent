# plan — 17.workspace-renewal

> 근거: [intent](intent.md)(승인 2026-10-12 — 공유 이해 확인, 라운드 2). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `8db4db4`(main — Merge Task #15, `task/17-workspace-renewal` 분기점). plan 승인: 2026-10-12(사용자 "plan 승인. 해석 2 승인.").
> 건수 EXPECT 는 개정 1 로 `8db4db4` 기준(ux_api 16 · roundtrip 12, 그 밖은 `8eb8aab` 와 같음).
> 시안: [검토 아티팩트](https://claude.ai/artifact/REnpPvZSZLnMHVcrAEc4ob) · Superdesign draft `d4b76d08-f1a7-4dd2-89c3-ab0d5bfd714b` v4(목업 HTML 은 리포 밖 `.superdesign/`).

## 목표

메인 작업 화면을 시안 v4(Nocturne)의 5영역(상단 바 · 레일 · 사이드바 · 대화 · 실행 상세)으로 다시 짜고, 이 PC 안에서 끝나는 기능 — 통합 검색 ·
내 활동 · ★ 즐겨찾기 · 저장된 항목 · 메시지 작업 · 입력창 서식 · 실행 상세 · "+ 새로 만들기" · 프로필 메뉴 · 알림 일시 중지 — 를 더한다. 집중 · @멘션은 18.

### 성공 기준(intent 사본)

- A1: 메인 화면이 상단 바 · 레일 · 사이드바 · 대화 · 실행 상세 5영역으로 그려지고, 영역별 라벨이 시안 v4 와 글자 그대로 같다(18 몫인 상태 칩 ·
  @멘션 제외, 라벨 목록은 plan 에 고정).
- A2: 통합 검색(Ctrl K)이 메시지 본문 · 대화 제목 · 사람(PC 표시 이름) · 에이전트 이름을 찾고, 결과를 누르면 그 대화(메시지면 그 위치)로 간다.
  사이드바 '제목 검색' 은 지금처럼 제목으로만 거른다.
- A3: 내비 '내 활동' 이 '받은 글' 을 대체한다 — 상대 쪽 글 · 자동 왕복 시작 · 끝 · 실패가 최신순으로 보이고 안 본 수가 배지로 보인다. 화면에
  '받은 글' 라벨이 없다.
- A4: 대화 머리글 ★ 로 즐겨찾기를 켜고 끄며, 켠 대화는 사이드바 '★ 즐겨찾기' 섹션에 보인다. 이 PC 에만 저장하고 상대에게 보내지 않는다.
- A5: 메시지 작업 '저장' 으로 저장한 메시지가 '저장된 항목'(사이드바 내비 · 실행 상세)에 모이고 해제된다. 이 PC 에만 저장한다.
- A6: 메시지 작업 '복사' 는 본문을 클립보드로, '다시 보내기' 는 내 글을 같은 대화에 새 요청으로 보낸다. 메시지별 편집 · 삭제 · 원문 버튼은 없다.
- A7: 입력창의 굵게 · 목록 · 코드 버튼이 markdown 표기를 넣고, Enter 는 보내기 · Shift+Enter 는 줄바꿈이며 한글 조합 중 Enter 는 보내지 않는다.
- A8: 실행 상세 패널에 고른 요청의 진행(보냄 → 대기열 → 실행(model · effort · 턴 · 시간) → 응답) · 대화 정보 · 자동 왕복 진행과 멈춤 · 저장된 항목이
  보인다. '타임라인' 탭은 없다.
- A9: "+ 새로 만들기" 메뉴의 다섯 항목(새 대화 · 자동 왕복 대화 · 초대 생성 · 초대 수락 · 에이전트 등록)이 각 화면으로 간다.
- A10: 프로필 메뉴에 표시 이름 · 아이콘 · 알림 일시 중지 · 환경설정이 있다. 알림 일시 중지 동안 브라우저 알림 · 트레이 표시가 나가지 않고 해제하면
  다시 나간다. 이 PC 에만 저장한다.
- A11: 15 에서 정한 결정이 유지된다 — intent `## 결정` 유지 목록(기존 테스트 green + 라벨 literal 테스트).

### intent 결정(사본)

- 시안 1안(Nocturne v4)이 화면 기준. '받은 글' → '내 활동'(자동 왕복 시작 · 끝 · 실패 포함). ★ · 저장된 항목 · 알림 일시 중지는 이 PC 에만.
  타임라인 탭 제거 → 실행 상세 패널(D26 개정 — 종결 시 환류). 집중 · @멘션은 18.
- 유지(15): 다크 기본 · 원문 보기 하나(머리글) · 입력 편집 모드 고정 · 타일 → 사이드바 목록 → 메인 메시지 · 제목 ✎ · 새 대화 제목 칸 · 제목 검색 ·
  사이드바 폭 조절 · 타일 표시 이름 · 아이콘 · 설정 전체 폭 · 트레이 열기 · 종료 · 새 글 = 대화 에이전트 실행 요청.
- 라운드 2: '주고받은 글' 은 상대 타일일 때 내비 행 · 사이드바 내비는 고른 타일 범위('나' = 전체) · 통합 검색 = 이 PC 의 모든 대화 본문 ·
  다시 보내기 = 같은 대화 · 같은 대상 새 요청 · 내 활동 안 본 수 = 상대별 마지막 열람 시각(해석 2 — 승인 2026-10-12) · 구현은 15 merge 뒤 `main` 에서 분기.
- 호환성 없음(D32): 이전 버전 상대 대비를 두지 않는다. SQLite 이력은 순방향 마이그레이션으로 보존.
- 제외: 집중 · @멘션(18) · 검토 제외 9건 · 메시지 편집 · 삭제 · 15 결정 재검토 · 새 UI 프레임워크(D10) · macOS 실동작(D25) · 수동 gate 실행(D28).

## intent 대비 해석 (plan 승인 대상)

코드 실측(HEAD `8eb8aab`)으로 시안 · 결정의 전제를 확인한 결과 — 없는 데이터는 만들고, 만들 수 없는 것은 덜어 낸다.

1. **시각 스타일** — 시안 Nocturne 값을 다크 토큰으로, 같은 이름의 라이트 짝을 만든다(15 의 다크 기본 · 라이트 선택 유지). 글꼴은 **내려받지 않는다**:
   `'Noto Sans KR'` · `'JetBrains Mono'` 가 설치돼 있으면 쓰고 없으면 지금 글꼴(`Malgun Gothic` · `Apple SD Gothic Neo` · `Consolas` · `Menlo`). 이유: 로컬
   웹UI 는 외부 요청이 없고(D9), 실행 파일 하나(D2)에 한글 글꼴을 넣으면 수 MB 가 는다.
2. **내 활동 안 본 수 — 결정의 전제가 사실과 다르다.** `POST /api/seen` 은 트레이 카운터만 지운다(`notify.rs` `Notifier::clear`) — 항목별 읽음
   표시는 없다. 대체: 상대마다 "내 활동을 마지막으로 본 시점"(이 PC)을 두고 그 뒤 항목을 안 본 것으로 센다. 내 활동을 열면 그 범위(상대 하나 ·
   '나' = 전부)의 본 시점을 화면이 받은 마지막 항목으로. **개정 2**: 본 시점 = 시각(`activity_seen_at`, 초) → 본 마지막 글 · 자동 왕복 기록 id
   (`peers.activity_seen_msg` · `activity_seen_event`) — 같은 초에 온 것 · 받은 뒤 본 표시 전에 온 것이 본 것으로 숨지 않게(적대적 평가 라운드 1).
3. **대화 행 '안 읽음' · 레일 배지** — 지금 없다. 대화마다 읽은 마지막 메시지 id(`conversations.read_id`, 이 PC)를 두고, 대화를 열면 그 대화의 마지막
   id 로. 안 읽음 = 상대 쪽 글(`mine = 0`) 중 그 id 보다 큰 것. 레일 배지 = 그 상대 대화들의 안 읽음 합. 이전(스키마 11)은 기존 대화를 다 읽은 것으로 둔다.
4. **자동 왕복 시작 · 끝 기록** — 지금은 알림으로만 흘러가고 남지 않는다. 시작 · 끝 알림을 내는 자리에서 **알림 설정과 무관하게** `relay_events` 에
   남긴다(끝 사유는 지금 문구 그대로 — "상한 도달" · "상대가 멈춤" 등). **개정 3**: 같은 표에 내 글이 실패로 바뀐 순간(`failed` · `message_id`)도
   남겨, 내 활동을 본 뒤 실패로 바뀐 글(이미 본 id)을 다시 안 본 것으로 센다 — 목록에는 그 글로만 보인다(적대적 평가 라운드 2).
5. **model · effort 표시** — 지금 어디에도 남지 않고 `Reply` 에도 없다. 실행한 쪽이 답에 model · effort 를 저장하고(`messages.model` · `effort`),
   `Reply` · `Post` 에 실어 보낸 쪽도 같은 값을 저장한다 — 피어 프레임에 필드 2개 추가(상대 동작 변화 없음, D32 로 호환 장치 없음).
6. **대기열 시각** — 대기 시작 · 끝 시각은 남지 않는다. 진행 단계는 상태로만 보이고 시각은 보낸 시각 · 응답 받은 시각(초까지) · 실행 시간(`duration_ms`)만.
7. **온라인 표시**(시안의 초록 점 · '온라인') — 접속 상태를 재는 곳이 없다(주기 ping 없음). 18 의 집중 표시(`Ping` · `Pong`)와 함께 18 로 넘긴다.
   17 의 타일 · 사이드바 머리는 지금처럼 페어링 상태만.
8. **에이전트 부제**(시안 "결제 · 배포") — 프로필에 설명이 없다. 이름만 보인다.
9. **알림 일시 중지** — 켜고 끄는 설정 하나(`notify_paused`, 시간 선택 없음). 켜 있는 동안 알림 이벤트를 버린다(트레이 · 브라우저 알림 · SSE
   모두 — `notify_inbound` 를 끈 것과 같은 경로) — 화면 갱신은 5초 폴링. `notify_inbound` 와 별개 키라 해제하면 그 설정대로 돌아온다.
10. **Enter 동작이 바뀐다** — 지금은 Enter = 줄바꿈 · 버튼으로만 보낸다. 시안대로 Enter = 보내기, Shift+Enter = 줄바꿈, IME 조합 중(`isComposing` ·
    `keyCode 229`) Enter 는 무시.
11. **통합 검색** — 서버에서 `LIKE` 부분 일치(이스케이프, 기존 `messages_across` 방식) — 한글 부분 문자열이 그대로 맞는다. 그룹별 상한 메시지 20 ·
    나머지 10. 결과 이동: 메시지 = `#c/{대화}/m/{메시지}`(그 글로 스크롤 · 강조) · 대화 = `#c/{id}` · 사람 = 그 타일 선택 · 에이전트 = 내 것은
    `#settings/agents`, 상대 것은 새 대화(`#new`)에 상대 · 에이전트를 채워서.
12. **'받은 글' 화면 → '내 활동'** — `#inbox` 를 `#activity` 로 바꾸고 기존 거르기(상대 · 종류 · 본문 검색)를 잇는다. 종류에 자동 왕복 · 실패를 더한다.
    실패 = 내 쪽 메시지 중 `state = 'error'`(내 요청 · 내 에이전트 답). 상대 에이전트의 실패 답은 상대 쪽 글로 이미 들어오고 실패 상태로 보인다.
13. **타임라인 탭 제거** — `#tab_timeline` · `#panel_timeline` · `#t/{id}` 를 지우고, 실행 상세가 기존 `view.js` `timeline()` 을 쓴다.
14. **알림 클릭 이동** — 지금은 메시지 밖 종류가 모두 `#settings` 로 간다. 페어링 종류는 설정, 나머지(메시지 · 자동 왕복)는 `#activity`.
    SSE 를 받으면 종류와 무관하게 다시 그린다(지금은 `message` 만).
15. **배치 수치** — 사이드바 폭 기본 272 · 범위 200 ~ 560 유지(15 결정, 시안 264). 실행 상세 패널 320px — 창 폭 1280 이상이면 열고 시작, 열림 상태는
    저장하지 않는다. ★ 대화는 '★ 즐겨찾기' 섹션에만 보인다(대화 섹션에서 빠진다).

## 설계

### 저장(스키마 11, `an-core`)

```sql
ALTER TABLE conversations ADD COLUMN starred_at INTEGER;    -- ★ 켠 시각, NULL = 아님
ALTER TABLE conversations ADD COLUMN read_id INTEGER;       -- 읽은 마지막 메시지 id
ALTER TABLE messages      ADD COLUMN saved_at INTEGER;      -- 저장 시각, NULL = 아님
ALTER TABLE messages      ADD COLUMN model TEXT;            -- 답: 실행 model
ALTER TABLE messages      ADD COLUMN effort TEXT;           -- 답: 실행 effort
ALTER TABLE peers         ADD COLUMN activity_seen_msg INTEGER;   -- 내 활동: 본 마지막 글 id(개정 2)
ALTER TABLE peers         ADD COLUMN activity_seen_event INTEGER; -- 내 활동: 본 마지막 자동 왕복 기록 id(개정 2)
CREATE TABLE IF NOT EXISTS relay_events (
    id INTEGER PRIMARY KEY, conversation_id INTEGER NOT NULL,
    kind TEXT NOT NULL,            -- 'start' | 'end' | 'failed'(개정 3 — 내 글이 실패로 바뀐 순간)
    detail TEXT NOT NULL DEFAULT '', created_at INTEGER NOT NULL,
    message_id INTEGER             -- 'failed' 의 글(개정 3)
);
CREATE INDEX IF NOT EXISTS messages_by_conv ON messages(conversation_id);         -- 개정 2
CREATE INDEX IF NOT EXISTS relay_events_by_conv ON relay_events(conversation_id); -- 개정 2
CREATE INDEX IF NOT EXISTS relay_events_by_message ON relay_events(message_id);   -- 개정 3
-- 이전 채움: read_id = 대화별 max(messages.id) · activity_seen_msg · activity_seen_event = 올린 때 max(id)
```

- 기존 규칙 그대로: `if v < 11` 한 트랜잭션 · 있는 열 건너뛰기 · `{db}.v10.bak` · `SCHEMA_VERSION = 11`. `core/tests/paths.rs` 의 `10` 리터럴 9곳을 11 로.
- `an_core::chat` 추가 — `set_starred(id, on)` · `mark_read(id, upto)` · `set_saved(msg, on)` · `saved(peer?, conversation?, limit)` ·
  `add_relay_event(conv, kind, detail)` · `activity(peer?, kind?, q?, limit)`(상대 쪽 글 + 실패 + `relay_events`, 최신순) · `activity_unseen(peer?)` ·
  `activity_upto()` · `mark_activity_seen(peer?, (message, event))` · `search(q, limit)`. 읽음 · 본 표시는 받은 범위까지만 · 뒤로 가지 않는다(개정 2). `messages_across` 는 `mine: Option<bool>`(None = 양쪽)로 넓힌다. `Summary` 에 `starred` · `unread`.
  `add_reply` 에 `model` · `effort`.

### 피어 프레임 · 실행(`an-net` · `an-agent`)

- `Request::Reply` · `Request::Post` 에 `model: Option<String>` · `effort: Option<String>`(64자 · `run_opts` 검증과 같은 글자 규칙, 틀리면 버림).
- `execute()` 가 `run_opts::resolve` 로 정한 값을 `record()` 까지 넘겨 답에 저장하고, 답을 보낼 때 싣는다. 받은 쪽은 `inbound_reply` · `inbound_post` 에서 저장.
- 자동 왕복 시작(`start_relay` · `mark_relayed`) · 끝(`relay_end` · `inbound_relay`)에서 `add_relay_event` — 알림 호출(`notify`) 바로 앞, 설정과 무관.

### 로컬 API(`an-app`)

- `GET /api/search?q=<글>&limit=<n>` — `q` 앞뒤 공백 정리 1 ~ 100자(밖이면 400) → `{messages[], conversations[], peers[], agents[]}`. 메시지 =
  message_json + `conversation{id, peer_id, agent, dir, title}`, 대화 = 요약 JSON, 사람 = PeerView(이름 · 표시 이름 · 호스트 일치), 에이전트 =
  `{name, owner: "me" | <peer_id>}`(내 에이전트 이름 · 폴더 + 대화에 나온 상대 에이전트 이름).
- `GET /api/activity?peer=&kind=all|human|agent|relay|failed&q=&limit=` → `{items[], unseen, unseen_all, upto{message, event}}` — 항목 `{kind: "message"|
  "failed"|"relay_start"|"relay_end", at, conversation{…, title}, message?, detail?, unseen}`, `limit=0` = 목록 없이 수만(5초 새로 고침).
  `POST /api/activity/seen {peer?, message, event}`(`message` · `event` = 그 화면이 받은 `upto`, 지금 마지막 항목을 넘으면 거기까지) → 204(`peer` 없음 =
  전부, 범위 없는 몸은 거절 — 개정 2).
- `PATCH /api/messages/{id} {saved: bool}` → `{id, saved}` · `GET /api/saved?peer=&conversation=&limit=` → 저장 순 최신부터.
- `PATCH /api/conversations/{id}` 에 `starred: bool`(선택 — `title` 도 선택, 둘 다 없으면 400, 모르는 항목 거절) → `{id, title, starred}`.
  `POST /api/conversations/{id}/read {upto}`(`upto` = 화면이 받은 마지막 글 id, 그 대화 글이 아니면 400 — 개정 2) → 204. 요약 JSON 에 `starred` · `unread`, message_json 에 `model` · `effort` · `saved`(필드 추가만 — CLI 가 읽는 필드는 그대로).
- 설정 `notify_paused`(bool, 기본 false) — `GET/PUT /api/settings`. `Prefs` 에 넣고 `Notifier::on_event` 가 켜져 있으면 버린다(건수 · 트레이 · SSE 모두).

### 웹UI(`web/`, 빌드 없는 ES 모듈 유지 — `createElement` · `textContent` 만, `innerHTML` 없음)

- **토큰**(`app.css` 첫 `:root` 가 유일한 리터럴 자리 — 15 규칙 그대로, 다크 블록은 `var(--dark-*)` 대입):

  | 토큰 | 라이트 | 다크(Nocturne) |
  |---|---|---|
  | `bg` · `topbar` · `sidebar` | `#F4F5F9` · `#FFFFFF` · `#EEF0F6` | `#0E1117` · `#0B0E14` · `#121620` |
  | `surface` · `surface-2` · `subtle` | `#FFFFFF` · `#F7F8FB` · `#ECEEF4` | `#171C28` · `#1D2331` · `#0B0E14` |
  | `border` · `text` · `muted` · `faint` | `#D7DBE5` · `#151925` · `#4B5365` · `#5F687B` | `#262D3D` · `#E8EBF2` · `#9BA3B4` · `#7D8699` |
  | `accent` · `accent-hover` · `accent-soft` · `on-accent` | `#4E55D9` · `#3E45C8` · `#ECEDFD` · `#FFFFFF` | `#7C83FF` · `#9AA0FF` · `rgba(124,131,255,.14)` · `#0B0E14` |
  | `agent` · `run` · `ok` · `danger` · `wait` | `#0A7562` · `#8A5700` · `#167A47` · `#C0392B` · `#5D6578` | `#3DD6B5` · `#F5B83D` · `#3DD68C` · `#FF6B6B` · `#8B93A7` |
  | `sent-bg` · `sent` · `recv-bg` · `recv` | `#ECEDFD` · `#3E45C8` · `#FFF4E5` · `#8A5700` | `rgba(124,131,255,.14)` · `#A9AEFF` · `#3B2F17` · `#F2B95C` |
  | `rail` · `peer-1 ~ 5` | `#1B1F2A` · `#5B6CF0` `#2F9E7A` `#9A5CD6` `#C96A3A` `#2A8FB5` | `#090B10` · 같은 다섯 색 |

  글꼴 `--font: 'Noto Sans KR', 'IBM Plex Sans KR', 'Malgun Gothic', 'Apple SD Gothic Neo', sans-serif` · `--mono: 'JetBrains Mono', 'IBM Plex Mono', Consolas,
  Menlo, monospace`(해석 1). 크기 12 · 13 · 14 · 16 · 20, 반지름 8(컨트롤) · 10(카드 · 패널) · 6(태그), 컨트롤 높이 32.
- **배치**: `#app` 그리드 — 1행 상단 바(48px), 2행 레일(60px) · 사이드바(`--side-w`) · 끌기 손잡이 · `main` · 실행 상세(320px, 접힘). 설정 화면은 사이드바 ·
  실행 상세를 숨긴 전체 폭(15).
- **상단 바**: 워드마크 "coworkAgent" · 통합 검색 입력(placeholder "검색 — 메시지 · 대화 · 사람 · 에이전트", 힌트 "Ctrl K", Ctrl+K 로 초점, 250 ms 뒤 조회,
  그룹별 결과 상자 · Esc 닫기) · "+ 새로 만들기"(메뉴: 새 대화 → `#new` · 자동 왕복 대화 → `#new` 에서 `#chat_relay_agent` 에 초점 · 초대 생성 →
  `#settings/invite_make` · 초대 수락 → `#settings/invite_accept` · 에이전트 등록 → `#settings/agents`) · 종(`title="알림"` → `#activity`, 안 본 활동이 있으면 점) ·
  내 아바타(`title="프로필 — 표시 이름 · 알림 일시 중지 · 환경설정"` — 메뉴: 표시 이름 · 아이콘 = '나' 타일 편집 폼 · 알림 일시 중지 켜기/끄기 · 환경설정 → `#settings`).
- **레일**: 지금 그대로('나' · 상대 타일 · + 초대 수락 · 설정) + 타일 안 읽음 배지(해석 3).
- **사이드바**: 머리(표시 이름 ✎ · 호스트) · 내비 `view.js` `sideNav(tile)` — 대화 · 내 활동(안 본 수) · 저장된 항목 · 에이전트 · (상대 타일만) 주고받은 글 ·
  '제목 검색'(15 그대로) · 섹션 "★ 즐겨찾기" · "대화"(+ `title="새 대화"`) · "{이름}의 에이전트"('나' = "내 에이전트" — 상대는 `GET /api/peers/{id}/agents`
  로, 실패하면 "목록을 받지 못했습니다") · 대화 행(방향 태그 · 제목 · `{에이전트} · {상태}` · 실행 중 표시 · "왕복 {n}/{상한}" 칩 · 안 읽음 굵게 + 점).
- **경로**(`view.js` `route`): `#activity` · `#saved` · `#agents` · `#c/{id}/m/{msg}` · `#settings/{절}` 추가, `#inbox` · `#t/{id}` 삭제. 내 활동 · 저장된 항목 ·
  에이전트 화면은 고른 타일 범위(해석 2 · 결정).
- **대화**: 머리글 — 방향 태그 · 제목 · ★(`title="즐겨찾기"`) · ✎(`title="제목 편집"`) · 부제 `{상대} 의 {에이전트} · {model} · {effort}`(마지막 답의 값, 없으면
  뺌) · 탭 "대화" · "보낸 메시지" · 아이콘 버튼 "원문 보기"(`#raw_btn`, 하나 — 15) · "저장된 항목"(실행 상세의 그 절로) · "세부정보"(패널 열고 닫기).
  메시지 — 작업 막대 `view.js` `messageActions(msg)`: 모두 "저장"(저장됨이면 "저장 해제") · "복사", 내 실행 요청만 "다시 보내기". 실행 중 = "실행 중 · {n}초".
  입력창 — placeholder "{에이전트} 에게 보내기 — markdown 가능", 버튼 "굵게" · "목록" · "코드"(`view.js` `insertMarkdown`), 힌트 "Enter 보내기 · Shift+Enter 줄바꿈",
  "보내기", Enter 판정 `view.js` `enterSends(ev)`(해석 10). 대화를 열면 `POST …/read {upto: 받은 글 최대 id}`.
- **실행 상세**(`aside#detail`): "실행 상세" · "닫기" · "진행"(`view.js` `runSteps(step)` — "요청 보냄" · "대기열" · "실행 완료" | "실행 중" | "실패" · "응답 받음",
  시각은 `HH:MM:SS`, 실행 = `{model} · {effort} · {n}턴 · {시간}`) · "대화 정보"(상대 · 상대 에이전트 · 실행 기본값 · 제목 = "이 PC 에만 저장") · "자동 왕복"
  (진행 `{n}/{상한}` · "멈춤" — 기존 relay 버튼과 같은 API) · "저장된 항목"(이 대화). 고른 요청 = 기본 마지막 요청, 메시지를 누르면 그 요청.
- **내 활동**(`#activity`): `view.js` `activityRows(items)` — 종류별 라벨("자동 왕복 시작" · "자동 왕복 끝 — {사유}" · "실패") · 안 본 항목 강조, 거르기 상대 · 종류
  (전체 · 사람 글 · 에이전트 답 · 자동 왕복 · 실패) · 본문 검색. 열면 `POST /api/activity/seen`.
- **저장된 항목**(`#saved`) · **에이전트**(`#agents`): 범위 목록 · 누르면 그 대화 / 상대 에이전트는 새 대화 채움 · 내 에이전트는 `#settings/agents`.
- **문구 literal**(A1 — `ux_literals` 가 index.html · app.js · view.js 에서 찾는다): "검색 — 메시지 · 대화 · 사람 · 에이전트" · "+ 새로 만들기" · "새 대화" ·
  "자동 왕복 대화" · "초대 생성" · "초대 수락" · "에이전트 등록" · "표시 이름 · 아이콘" · "알림 일시 중지" · "환경설정" · "내 활동" · "저장된 항목" · "에이전트" ·
  "주고받은 글" · "제목 검색" · "★ 즐겨찾기" · "즐겨찾기" · "제목 편집" · "보낸 메시지" · "원문 보기" · "세부정보" · "저장" · "복사" · "다시 보내기" · "굵게" ·
  "목록" · "코드" · "Enter 보내기 · Shift+Enter 줄바꿈" · "보내기" · " 에게 보내기 — markdown 가능" · "실행 상세" · "닫기" · "진행" · "요청 보냄" · "대기열" ·
  "실행 완료" · "응답 받음" · "대화 정보" · "상대 에이전트" · "실행 기본값" · "이 PC 에만 저장" · "자동 왕복" · "멈춤".

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 스키마 11 · 피어 프레임 model/effort · 자동 왕복 기록 · API(search · activity · saved · starred · read · notify_paused) | 테스트(paths · roundtrip · relay · notifier · ux_api) | E6 ~ E10 |
| 2 | 웹UI 뼈대 — 토큰 · 5영역 · 상단 바 · 메뉴 · 사이드바 내비/섹션 · 경로 · 레일 배지 · 받은 글 · 타임라인 탭 제거 | 테스트(view.test · ux_literals) | E3 · E4 · E5 |
| 3 | 대화 · 실행 상세 · 메시지 작업 · 입력창 · 내 활동 · 저장된 항목 · 에이전트 · 검색 결과 | 테스트(view.test) | E3 · E11 |
| 공통 | 검증 명령 · 실화면 | — | E1 · E2 · E12 |

## 평가 항목

- [ ] E1: 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2·A3·A4·A6·A7·A8·A9·A10 — 화면 로직 node 테스트 31건(기존 21 + side_nav · new_menu · profile_menu · insert_markdown · enter_sends · message_actions · run_steps · search_groups · activity_rows · conv_sections, 기존 "화면 위치" 는 새 경로로 고침)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass 31\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A3·A8·A11 — 화면 문구 literal(ux_literals 8건: 기존 5 + workspace_labels(설계 "문구 literal" 목록 전부 · `id="raw_btn"` 1회) · no_received_label('주고받은 글' 밖의 '받은 글' 0) · no_timeline_tab(`tab_timeline` · `panel_timeline` · `#t/` · "타임라인" 0))
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: test result: ok. 8 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A1 — 색 · 글꼴 크기 리터럴은 `app.css` 첫 `:root` 안에만이고, 다크 토큰이 Nocturne 값 · 글꼴 첫 이름이 Noto Sans KR
  CHECK: python -c "import re;c=open('crates/app/web/app.css',encoding='utf-8').read();j=open('crates/app/web/app.js',encoding='utf-8').read();m=re.search(r':root\s*\{[^}]*\}',c);r=m.group(0) if m else '';rest=c.replace(r,'');v=len(re.findall(r'#[0-9a-fA-F]{3,8}\b|rgba?\(',rest))+len(re.findall(r'font-size:\s*[0-9.]+(?:px|rem|em)',rest))+len(re.findall(r'#[0-9a-fA-F]{6}\b',j));need=['--dark-bg: #0E1117','--dark-surface: #171C28','--dark-border: #262D3D','--dark-text: #E8EBF2','--dark-accent: #7C83FF','--dark-agent: #3DD6B5','--dark-run: #F5B83D','--dark-ok: #3DD68C','--dark-danger: #FF6B6B','--font: '+chr(39)+'Noto Sans KR'+chr(39)];n=sum(x in r for x in need);print(f'root={int(bool(m))} violations={v} nocturne={n}')"
  EXPECT: root=1 violations=0 nocturne=10
  EVIDENCE: pending
- [ ] E6: A2·A3·A4·A5·A10 — 로컬 API(ux_api 24건: 기존 16 + search_groups_messages_titles_peers_agents · search_treats_like_wildcards_literally · activity_merges_theirs_failures_relay_events · activity_unseen_by_peer_and_mark_seen · star_toggles_local_only · saved_toggle_and_list_scoped · conversation_read_clears_unread · settings_notify_paused_roundtrip)
  CHECK: cargo test -p coworkagent-app --test ux_api
  EXPECT: test result: ok. 24 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A3·A4·A5·A8 — 스키마 11 이전(paths 13건: 기존 12 + schema11_workspace — 새 열 · `relay_events` · read_id · 내 활동 본 표시 id 채움 · 대화별 색인 · `v10.bak`)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 13 passed; 0 failed
  EVIDENCE: pending
- [ ] E8: A8 — 실행 model · effort 가 양쪽 답에 남는다(roundtrip 13건: 기존 12 + reply_records_model_effort_both_sides)
  CHECK: cargo test -p an-agent --test roundtrip
  EXPECT: test result: ok. 13 passed; 0 failed
  EVIDENCE: pending
- [ ] E9: A3 — 자동 왕복 시작 · 끝(상한 도달)이 알림 설정과 무관하게 기록된다(relay 10건: 기존 9 + relay_start_end_events_recorded)
  CHECK: cargo test -p an-agent --test relay
  EXPECT: test result: ok. 10 passed; 0 failed
  EVIDENCE: pending
- [ ] E10: A10 — 알림 일시 중지(notifier 11건: 기존 9 + paused_drops_notices_and_tray · resume_after_pause_notifies_again)
  CHECK: cargo test -p coworkagent-app --test notifier
  EXPECT: test result: ok. 11 passed; 0 failed
  EVIDENCE: pending
- [ ] E11: A11 — 15 결정을 지키는 테스트 29건이 이름 그대로 남아 있다(ux_api 16 · ux_literals 5 · view 8 — 개정 1 에서 15 적대적 평가 회귀 5건 추가)
  CHECK: python -c "fs={'crates/app/tests/ux_api.rs':['favicon_served','peer_patch_group_model_effort','agent_patch_edits_but_not_name','settings_claude_defaults','inbox_lists_their_messages_with_search','outbox_lists_my_messages_to_peer','theme_setting_persists','to_human_rejected','conversation_title_set_and_edit','peer_tile_alias_icon','settings_me_tile_side_width','direct_add_with_code_pairs_both_sides','direct_readd_known_key_waits_for_approval','new_conversation_to_human_rejected','inbox_kind_filter_applies_before_limit','settings_read_error_is_500'],'crates/app/tests/ux_literals.rs':['turn_label_literal','invite_labels_without_messenger','favicon_link_present','title_search_label','tray_menu_open_quit_only'],'crates/app/web/view.test.mjs':['settings_tree','markdown_blocks_safe','html_doc_detect','group_filter_search','inbox_rows','peer_messages','direction_tag','tile_text']};n=sum(len(v) for v in fs.values());m=sum(((chr(34)+t+' ') if f.endswith('.mjs') else ('fn '+t+'(')) not in open(f,encoding='utf-8').read() for f,v in fs.items() for t in v);print(f'targets={n} missing={m}')"
  EXPECT: targets=29 missing=0
  EVIDENCE: pending
- [ ] E12: A1·A2·A3·A4·A5·A6·A7·A8·A9·A10 — 실화면(수동): 5영역 · 다크/라이트 · 통합 검색 · 내 활동 · ★ · 저장 · 메시지 작업 · 입력창 · 실행 상세 · + 메뉴 · 프로필 · 알림 일시 중지 · 두 PC 왕복의 model · effort
  EVIDENCE: pending

## 종결 시 할 일

- E12 는 D28 에 따라 `ABANDON`(사유: D28) + 19.prototype-acceptance 에 TC 추가(위 실화면 항목).
- masterPlan D26 개정 환류(C 요청 타임라인 화면 → 실행 상세 패널) · S4 보강(5영역 · 통합 검색 · 내 활동 · 즐겨찾기 · 저장) · D29 보강(알림 일시 중지) ·
  D32 적용 기록(피어 프레임 `Reply` · `Post` 의 model · effort).
- PRODUCT 도메인 규칙 "웹UI 구조" · "인바운드 알림" · "스키마 올리기" 갱신, 매뉴얼 `an_core` · `an_net` · `an_agent` · `an_app`.

## 개정 기록

- 개정 1(2026-10-12, 착수 — **평가 항목 변경 · 사용자 승인 — 위임**(masterPlan "자동 진행 위임" · 진행 목표 원문 "Goal set: task 15,17,18 까지
  완료. 사용자 테스트는 task 18 완료 후 일괄 검증.")): 기준 커밋 `8db4db4`. 15 종결로 늘어난 테스트에 맞춰 E6 19 → 24(기존 11 → 16) · E8 12 → 13
  (기존 11 → 12) · E11 24 → 29(15 적대적 평가 회귀 5건 — direct_add_with_code_pairs_both_sides · direct_readd_known_key_waits_for_approval ·
  new_conversation_to_human_rejected · inbox_kind_filter_applies_before_limit · settings_read_error_is_500). stage 사용자 검증은 진행 목표대로 18 종결 뒤
  일괄(E12 는 D28 그대로).
- 개정 2(2026-10-12, 적대적 평가 라운드 1 지적 — **설계 변경 · 사용자 승인 — 위임**(masterPlan "자동 진행 위임")): 읽음 · 내 활동 본 표시를 화면이
  받은 범위까지만 — `POST …/read {upto}` · `GET /api/activity` 의 `upto` · `POST /api/activity/seen {peer?, message, event}`(빈 몸 = 전부 폐지). 해석 2 의
  본 시점을 초 단위 시각(`activity_seen_at`)에서 id(`activity_seen_msg` · `activity_seen_event`)로 — 같은 결정(상대별 · 이 PC · 본 뒤 항목 = 안 본 것)의
  수단 교체. 대화별 글 · 기록 색인 · `limit=0`(수만). 스키마 11 은 배포 전이라 같은 단계에서 고친다(D32). 평가 항목 수는 그대로(E6 · E7 은 기존
  테스트 보강 — conversation_read_clears_unread · activity_unseen_by_peer_and_mark_seen · schema11_workspace), E7 문구만.
- 개정 3(2026-10-12, 적대적 평가 라운드 2 지적 — **설계 변경 · 사용자 승인 — 위임**(masterPlan "자동 진행 위임")): 내 글이 실패로 바뀌는 순간을
  `relay_events`(`failed` · `message_id`)에 남겨(`chat::set_state` · `set_flags` 한 곳) 본 뒤 실패로 바뀐 글을 다시 센다 — 해석 4 보강. 자동 왕복
  목록 · 수는 `start` · `end` 만. 평가 항목 수 그대로(activity_unseen_by_peer_and_mark_seen · schema11_workspace 보강).

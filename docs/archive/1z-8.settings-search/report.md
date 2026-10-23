# report — 1z-8.settings-search

종결: 2026-10-23 · 브랜치 `task/1z-8-settings-search` → main(squash — push 는 사용자 승인 뒤)

## 결과

UI/UX 수정 1.3.4 — seed `1021-cowork.md` 와 추가 지시("대화 정보 `제목` 제거병행" · "환경설정 초기 화면은 `연결` 매뉴 선택된 상태").

- 상단 바: 이름 **coworkAgent** 뒤 실행 중 버전 `v1.3.4`.
- 통합 검색: 상자 머리 스위치 **"나간 대화 포함"**(기본 끔 — 나감 · 상대가 나감 · 끝난 공유 사본의 글 · 제목을 뺌, 서버 `GET /api/search?closed=` 기본 거짓) · "지운 대화는 검색되지 않습니다",
  결과 줄 앞 유형 칩(시안 S1 — 기존 색 재사용) · 닫힌 대화 태그.
- 개요 미리보기: 말풍선은 어느 크기든 2줄, 말풍선 사이 `--sp-3`. 대화 세부정보 '대화 정보' 의 `제목` 줄 삭제.
- 설정(시안 B — 사용자 선택, 권장안 A 아님): 왼쪽 메뉴 8개 **연결 · 그룹설정 · 에이전트 · 실행 · 화면 · 시작 · 알림 · 디버깅**, 고른 화면만, 처음 = 연결(고른 상태).
  연결 · 그룹설정 · 에이전트 = 목록 · 상세 2단(줄 고름 · ✕ · **"닫기"**, 초점 = 연 곳), 그룹 넣기 · 빼기 단추, 초대 생성 · 수락 · 에이전트 등록 = 상세 칸 폼. 실행 · 화면 · 시작 · 알림 = 2열 타일.
  체크박스는 전부 스위치(`role="switch"`). 디버깅 = 파일 카드(버전 · 데이터 폴더 · DB · 로그 경로 · 크기 · 복사) · 로그 끝 200줄(수준 거르기) · 진단 정보 복사(`GET /api/debug` · `/api/debug/log`).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 상단 바 버전 | 충족 | E6(`ux_literals`) · E8(v1.3.4) |
| A2 검색 스위치 · 서버 기본 뺌 | 충족 | E4(`search_closed_switch`) · E3 · E8(끔 2 · 2 / 켬 6 · 4) |
| A3 검색 유형 칩 S1 | 충족 | E3(`search_groups` · `search_groups_closed`) · E8(라이트 · 다크) |
| A4 미리보기 2줄 · 여백 | 충족 | E3(`preview_zoom`) · E8(125 % 2줄 · 비침 없음 · 12px) |
| A5 메뉴 8개 · 한 화면 · 처음 연결 · 옛 해시 | 충족 | E3(`settings_tree` · `settings_route`) · E8 |
| A6 스위치 · Space · 저장 | 충족 | E6 · E8(`notify_pair` Space 저장, 설정 안 비스위치 0) |
| A7 목록 · 상세 · 닫기 초점 · 저장 전 값 · 관리 동작 | 충족 | E8 · 적대적 평가 R1 · R2 반영(저장 전 값 주인 — 아래) |
| A8 2열 · 좁은 창 | 충족 | E8(900px 1열 · 목록 위 상세 아래) |
| A9 디버깅 · 키 · 토큰 · 지문 없음 | 충족 | E5(`debug_api` 5) · E3(`debug_text`) · E8 |
| A10 회귀 없음 · 1.3.4 · 문서 | 충족 | E1 · E2 · E7 |
| A11 대화 정보 제목 줄 없음 | 충족 | E6 · E8 |

- 적대적 평가 2 라운드(상한, Orca 가 라운드마다 새로 띄운 Codex) — R1 high 1(지운 에이전트의 저장 전 값이 같은 id 를 다시 쓴 새 프로필에 붙음 → 주인 · 목록 밖 버림 · 삭제면 지움, 같은 꼴의 연결
  `gsDrafts` 도), R2 medium 1(같은 이름 다시 등록이면 주인 = 이름이 막지 못함 → 에이전트 JSON 등록 세대 `uid` 를 주인으로) 반영. 미조치 없음 — R2 수정은 리뷰어가 보지 않았다(상한)([eval](eval.md)).
- 실제 화면 확인 중 결함 3건을 스스로 찾아 고침 — 그룹 상세 아래 줄이 가운데에 뜸 · 검색 스위치 폭 96px(검색 칸 여백 상속) · 디버깅 크기 글자 줄바꿈.
- 구현자 해석(사용자 미확인 — intent 결정 11 · plan 개정 기록):
  - 검색 스위치 값은 탭이 열린 동안만(새로 열면 끔), 좁은 창 기준 폭 960px, 로그 API 상한 200줄 · 64 KiB, 디버깅 "버전" = `/api/status` 와 같은 값.
  - 승인 절차 = 인터뷰 답과 "개발 진행해" 로 intent · plan 승인 갈음.
  - 에이전트 목록 검색 칸 없음(#1l 결정 유지 — plan 개정 ②), 설정 화면 머리 한 줄 설명 없음(D35 — ③).
  - 디버깅 로그 끝 줄은 로그 그대로 — 시작 줄의 이 PC 자기 지문이 보인다('사용자 정보' 에 이미 보이는 공개 식별자, A9 문구).
  - 다른 탭이 같은 에이전트의 폴더 · 상위 · 셸을 저장하면(새 `uid`) 이 탭의 저장 전 값은 버려진다(R2 수정의 부수 효과 — 옛 값으로 덮지 않는 쪽).
- 동작 변화(사용자가 알고 고름 — U2): 통합 검색은 지금까지 닫힌 대화도 늘 보였으나 이제 기본으로 뺀다. `chat::search` 호출자는 `/api/search` 하나(코드 확인 — 시험 `message_tidy` 는 `closed = true`).

## 산출

- core: `chat::closed_sql` · `search(conn, q, limit, closed)`. app: `SearchQuery.closed`, `GET /api/debug` · `/api/debug/log`(`server::tail_lines` · `LOG_TAIL_LINES` · `LOG_TAIL_BYTES`), 에이전트 JSON `uid`.
- web: `index.html` 설정 `section.set-page` 8개 · `#top_version` · `#search_opts` / `#search_closed` · `#search_results`, `view.js` `settingsTree`(평평 8개) · `SETTINGS_ALIAS` · `searchGroups` `mark` · `tag` ·
  `previewZoom` · `fileSize` · `logLines` · `diagText` · `draftOf` · `pruneDrafts`, `app.js` `pick` · `setPick` · `closePick` · `mdRow` · `mdHead` · `mdFoot` · `renderConnPage` · `renderGroupPage` ·
  `userDetail` · `groupDetail` · `agentDetail` · `loadDebug` · `debugCopy`, `app.css` 스위치 · `.set-*` · `.md*` · `.debug-*` · `.pv-text`.
- 시험: app `debug_api`(5) · `ux_api` `search_closed_switch` · `chat_api` `agents_crud_api`(같은 이름 다시 등록 → 다른 `uid`) · `ux_literals`(#1z-8 줄) · `view.test.mjs`(설정 · 검색 · 미리보기 · 디버깅 · `draft_owner`).
- 문서: web-ui · an_app · an_core · 사용자 가이드 · PRODUCT · README(1.3.4) · masterPlan S4 · D35 · D37 · D40 · D26 · 종결 목록 · 개정 기록. 시안 캔버스 https://claude.ai/artifact/RQFtDzbXq1z6a8512noLpL

## 남은 것

- main squash push · 1.3.4 서명 릴리스 여부 — 사용자 결정.
- macOS 화면(스위치 · 2열 타일 · 디버깅 경로) · 실제 두 PC 연동(초대 생성 · 수락을 상세 칸에서) — 19 acceptance · Mac 실기 조건 대기(D28).
- 후보: `agents` 표 `AUTOINCREMENT`(id 다시 쓰기 자체를 없앰 — 스키마 변경이라 이번 범위 밖, 저장 전 값은 `uid` 로 이미 막음).

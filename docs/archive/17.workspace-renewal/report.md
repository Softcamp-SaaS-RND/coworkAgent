# report — 17.workspace-renewal

종결: 2026-10-12 · 브랜치 `task/17-workspace-renewal` → main

## 결과

메인 작업 화면을 시안 v4(Nocturne)의 5영역 — 상단 바 · 레일 · 사이드바 · 대화 · 실행 상세 — 으로 다시 짰다. 상단 바에 통합 검색(Ctrl K — 메시지 ·
대화 · 사람 · 에이전트) · "+ 새로 만들기" 메뉴 · 알림(내 활동) · 프로필 메뉴(표시 이름 · 아이콘 · 알림 일시 중지 · 환경설정)가 있고, 사이드바는 고른
타일 범위의 내비(대화 · 내 활동 · 저장된 항목 · 에이전트 · 주고받은 글) · ★ 즐겨찾기 · 대화 섹션을 보인다. '받은 글' 은 '내 활동'(상대 쪽 글 ·
자동 왕복 시작 · 끝 · 실패, 안 본 수 배지)으로 바뀌었고, 타임라인 탭은 실행 상세 패널(진행 · 대화 정보 · 자동 왕복 · 저장된 항목)로 대체됐다(D26
개정). 메시지는 작업 막대(저장 · 복사 · 다시 보내기)를, 입력창은 서식 버튼 · Enter 보내기를 갖는다. 답의 model · effort 가 양쪽 이력에 남는다.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 5영역 · 시안 라벨 literal | 충족(literal · 토큰) · 실화면 ABANDON | E4 `workspace_labels` · E5 `root=1 violations=0 nocturne=10` · E3 · E12(D28) |
| A2 통합 검색 · 결과 이동 · 제목 검색 유지 | 충족 · 실화면 ABANDON | E6 `search_groups_messages_titles_peers_agents` · `search_treats_like_wildcards_literally` · E3 `search_groups` · E12 |
| A3 내 활동(받은 글 대체) · 안 본 수 | 충족 · 실화면 ABANDON | E6 `activity_merges_theirs_failures_relay_events` · `activity_unseen_by_peer_and_mark_seen` · E9 · E4 `no_received_label` · E3 `activity_rows` · E12 |
| A4 ★ 즐겨찾기(이 PC) | 충족 · 실화면 ABANDON | E6 `star_toggles_local_only` · E3 `conv_sections` · E7 · E12 |
| A5 저장된 항목(이 PC) | 충족 · 실화면 ABANDON | E6 `saved_toggle_and_list_scoped` · E7 · E12 |
| A6 복사 · 다시 보내기 · 편집/삭제 없음 | 충족 · 실화면 ABANDON | E3 `message_actions` · E12 |
| A7 입력창 서식 · Enter · 한글 조합 | 충족 · 실화면 ABANDON | E3 `insert_markdown` · `enter_sends` · E12 |
| A8 실행 상세(진행 · 정보 · 자동 왕복 · 저장) · 타임라인 탭 없음 | 충족 · 실PC ABANDON | E3 `run_steps` · E4 `no_timeline_tab` · E8 `reply_records_model_effort_both_sides` · E7 · E12 |
| A9 "+ 새로 만들기" 다섯 항목 | 충족 · 실화면 ABANDON | E3 `new_menu` · E4 · E12 |
| A10 프로필 메뉴 · 알림 일시 중지(이 PC) | 충족 · 실화면 ABANDON | E10 `paused_drops_notices_and_tray` · `resume_after_pause_notifies_again` · E6 `settings_notify_paused_roundtrip` · E3 `profile_menu` · E12 |
| A11 15 결정 유지 | 충족 | E11 `targets=29 missing=0` · E1 |

- 적대적 평가 2 라운드(상한) — 라운드 1: high 2 · medium 1(읽음 경쟁 · 초 단위 활동 워터마크 — 사실 · 고침 — 받은 범위 id 로, 5초 폴링 전수 스캔 — 일부 사실 · 수만 조회 ·
  색인), 라운드 2: high 1 · medium 1(본 뒤 실패로 바뀐 내 글 — 사실 · 고침 — 실패 전환 기록, 자동 왕복 기록이 알림 기준 — 설계대로 · 미조치).
  미조치 8건은 [eval.md](eval.md) → 원장.
- stage 리뷰 3회(서브에이전트, 읽기 전용) — Stage 1: 자동 왕복 상태 저장 실패 때 잠금 재진입 교착 · 지운 상대의 안 본 수 · 상한 마지막 답의
  실행 정보 · 빈 몸 열람, Stage 2: Enter 연타 · 내 활동 열람 범위 · 새로 고침 초점 · 실패 답 표시 · 상태 줄 높이, Stage 3: 실행 상세 대화 불일치 ·
  키보드 작업 막대 · 검색 재열림 · 글 위치 · 마지막 실행 출처 — 반영.
- plan 개정 3건(모두 위임): 1 기준 커밋 · 평가 항목 수, 2 읽음 · 내 활동 본 표시를 받은 범위까지(id) — 해석 2(사용자 승인)의 "본 시각" 을 "본
  마지막 항목 id" 로 바꾼 편차(같은 결정의 수단 교체), 3 실패 전환 기록.
- 스키마 11 은 배포 전이라 라운드 1 · 2 에서 같은 단계 안을 고쳤다 — 그 전 빌드로 11 이 된 개발 DB 는 지우고 다시 만든다.

## 산출

- `an-core`: 스키마 11(`conversations.starred_at` · `read_id` · `messages.saved_at` · `model` · `effort` · `peers.activity_seen_msg` · `activity_seen_event` ·
  `relay_events`(+ 실패 전환 `failed` · `message_id`) · 색인 셋) · `chat` — `set_starred` · `marks` · `mark_read(id, upto)` · `set_saved` · `saved` · `add_relay_event` ·
  `activity` · `activity_unseen` · `activity_upto` · `mark_activity_seen(peer, 범위)` · `search` · `set_run` · `messages_across(mine: Option)`.
- `an-net`: `Reply` · `Post` 의 `model` · `effort`(글자 규칙 밖이면 버림 — D32 호환 대비 없음).
- `an-agent`: 답에 실행 model · effort 기록 · 전달 · 자동 왕복 시작 · 끝 기록(알림 설정과 무관) · 자동 왕복 상태 저장 실패 처리(잠금 순서).
- `coworkagent-app`: `GET /api/search` · `GET /api/activity`(`upto` · `limit=0`) · `POST /api/activity/seen {peer?, message, event}` ·
  `GET /api/saved` · `PATCH /api/messages/{id}` · `PATCH /api/conversations/{id}`(`starred`) · `POST /api/conversations/{id}/read {upto}` · 설정
  `notify_paused` · 웹UI 전면(Nocturne 토큰 · 5영역 · 상단 바 메뉴 · 사이드바 내비 · 섹션 · 레일 배지 · 내 활동 · 저장된 항목 · 에이전트 화면 ·
  메시지 작업 · 입력창 · 실행 상세 · 통합 검색 결과 · 타임라인 탭 제거).

## 남은 것

- 실화면 · 실PC 확인(E12) — `19.prototype-acceptance` TC-L(D28). 사용자 일괄 검증은 Task 18 뒤(사용자 지시 2026-10-12).
- 미조치 지적 — [eval.md](eval.md) `## 미조치 지적` → 원장.
- 집중 · @멘션 · 질의/요청 · 접속 상태(온라인 점) — 18.focus-mention.

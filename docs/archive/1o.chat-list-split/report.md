# report — 1o.chat-list-split

종결: 2026-10-17 · 브랜치 `task/1o-chat-list-split` → main

## 결과

사이드바 대화방 목록을 **"내 대화"**(내가 연 대화 · 내가 연 자동 왕복) · **"agent 대화"**(상대가 연 대화 — 내 에이전트가 답) · **"공유 대화"** 머리로 나누고
(시안 Ⓐ — '나' 는 상대별 절 안, 끌어 옮기기는 묶음 안), 말풍선 음영을 낮췄다(내 글 강조색 14% + 오른쪽 띠 · 상대 글 타일 색 7% — 글자 대비 36 경우 4.5:1
이상). 진행 중 사용자 지시로 **관람자 초대(공유 대화)** — 대화를 페어링된 상대(5명까지)에게 읽기 전용으로 나눔(이력 · 이후 글 · 편집 · 삭제 따라감, 원 상대에는
관람자 수만, 빼기 · 나가기 · 24시간 닿지 않음 → "공유 해제됨") — 과 레일 우클릭 브라우저 메뉴 막기 · 메뉴 설명 지우기 · 에이전트 "새 대화" 단추를 더했다.
통신 프레임 5종 · 스키마 20.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 내 대화 · agent 대화 · 내가 연 자동 왕복 = 내 대화 | 충족 | E3 · E4 `conv_split_rows` · mock(stage 4) |
| A2 상대 타일 · 그룹 · '나' 같은 규칙 | 충족 | E4 `conv_split_scopes` · mock |
| A3 말풍선 음영 Ⓐ · 구분 · 대비 4.5:1 | 충족 | E5 · E6 `bubble_contrast`(36 경우) |
| A4 기존 목록 기능 회귀 없음 | 충족 | E1 · E3(node 112) · E4 `conv_split_drop` · `conv_split_starred` |
| A5 검증 명령 | 충족 | `check_gates` 14 충족 · 2 ABANDON(D28) |
| A6 레일 우클릭 브라우저 메뉴 없음 | 충족(자동 = `contextmenu` 막힘까지) | E5 literal · mock `defaultPrevented` — 실제 브라우저 메뉴는 원장 `#1o PC` |
| A7 관람자 넣기 → 공유 대화 · 이력 · 따라감 · 입력창 없음 | 충족 | E11 · E13 · E14 · mock |
| A8 원 상대 알림 | 충족 | E12 `notice_tells_origin_watcher_count` · E13 |
| A9 빼기 · 나가기 · 사본 읽기 전용 · 꺼진 관람자 보관(24시간) | 충족 | E11(`removed_or_left…` · `kept_while_viewer_offline` · `removed_mid_flight…` · `unreachable_viewer_dropped…`) |
| A10 공유 대화는 실행 · 자동 왕복 · 요청 · CLI 에 안 끼어듦 · 구버전 거절 | 충족 | E11(`history…` 사본 막기 · `old_viewer_rejected` · `forged_share_rejected`) · E13 |

- 적대적 평가 2 라운드(상한, codex-review 세션 · 워크트리) — 라운드 1 high 3(재사용 id → **반박**(리뷰어 확인) · 빼기 ↔ 보내던 중 · 꺼진 관람자 몫 무제한
  → 고침), 라운드 2 high 2(확인 ~ 보내기 창 → 줄임 · 24시간 포기 뒤 사본 방치 → 빼기 표시 남김). 미조치 3 · A9 24시간 한정은 사용자 승인([eval](eval.md),
  원장 `#1o R1-*` · `R2-*`).
- plan 편차: intent 개정 2회(사용자 지시 1016-cowork.md — 레일 우클릭 · 관람자 초대 · 둘째 지시 — 상한 5 · 보기만 · "공유 해제됨" · 설명 지우기), plan 개정
  기록 6줄 모두 승인. 공유 대화 말풍선은 사용자 지시로 넣은 사람 쪽 글 오른쪽. Stage 2(통신 · 저장)는 별도 에이전트가 구현 · 이 세션이 검토.
- 1n 과 합치기 — main 을 이 브랜치에 merge(충돌: masterPlan D13 · D14 · 로드맵 · 개정 기록 = main, 개발자 가이드 8.10 + 8.11 · `ux_literals` 둘 다 살림).

## 산출

- 통신 — `Request::Share` · `SharePost{ShareMsg}` · `ShareDelete` · `ShareEnd` · `ShareNotice` · `Pong.share` · `knows_share` · `MessageKind::Share`.
- 저장 — 스키마 20(`shares` · `conversations.share_other` · `share_dir` · `share_ended` · `watchers` · `messages.owner`), `Dir::Shared`, core `share` 모듈.
- 대화 엔진 — `share_add` · `share_remove` · `share_leave` · `fan_share`(커서) · 받는 처리기(사본 소유 확인) · 사본 쓰기 거절 · 보내기 직전 관람자 확인 ·
  24시간 포기 = 관람 끝(빼기 표시 남김).
- API — `POST/DELETE /api/conversations/{id}/observers[/{peer}]` · `POST …/leave`, 대화 JSON `observers` · `watchers` · `share`, 글 `owner`. CLI 공유 발화자.
- 웹 — `CONV_GROUPS` · `splitByDir` · `splitKey` · `.dir-head` · 말풍선 Ⓐ · `sharedAuthor` · `shareHead` · `shareCandidates` · 세부정보 "관람자" · 사본 읽기 전용 ·
  `bubbleSide` owner · `#rail` 우클릭 막기 · 설명 지우기 · 에이전트 "새 대화" 단추.
- 시험 — `agent/tests/share.rs` 9 · `chat_api` `share_observer_api` · `store` `share` · `remote_input` `share_frames_ids_checked` · node `conv_split_*` 4 ·
  `share_*` 4 · `ux_literals` `chat_list_split_literals` · `bubble_contrast` · `hint_cleanup_literals` · cli `share_author_label`.
- 문서 — 사용자 가이드 4 · 7.2 · **7.7** · 16장 · 개발자 가이드 5.2 · 7.1 · 7.2 · **8.11** · 9장 · 매뉴얼 4 · README · PRODUCT · 캡처 3장(가상 데이터) ·
  masterPlan D26 · D35 · S4 보강 · **D38** · 원장 5행.

## 남은 것

- 19 인수 TC — 실제 두 · 세 PC: 목록 나눔 · 말풍선 · 레일 우클릭 브라우저 메뉴 · 관람자 이력 · 원 상대 알림 · 빼기 · 나가기(원장 `#1o PC`).
- 공유 글 따로 보내는 줄 · 커서 재생(원장 `#1o R1-F3`) — 다음 Task 후보.
- 릴리스(통신 형식 · 스키마 20 — 0.6.0 후보) · GitHub 반영은 사용자 확인 뒤.

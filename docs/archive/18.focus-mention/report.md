# report — 18.focus-mention

종결: 2026-10-12 · 브랜치 `task/18-focus-mention` → main

## 결과

사용자가 켜는 **집중**(이 PC)으로 상대 PC 의 새 실행 요청을 받는 순간 거부하고(새 오류 코드 `focus` — 기록 · 실행 · 알림 없음), 보낸 쪽은
"상대가 집중 중" 으로 보이며 다시 보내기로 재시도한다. 진행 중 자동 왕복은 다음 차례에서 멈추고 집중 동안 알림도 멈춘다. 상대의 접속 · 집중을
30초 `Ping` · `Pong{focus}` 로 재서 레일 점 · 사이드바 머리 · 대화 머리글에 보인다. 사람 글마다 **질의 · 요청**(기본 질의)을 고르고 양쪽 이력에
남기며, 받은 요청은 받은 쪽만 **완료 표시**해 쓴 쪽에 보낸다(새 프레임 `Done`). 요청은 실행 상세 '요청' · 내 활동 종류 "요청"(내게 요청된 목록) ·
보낸 메시지 종류로 모아 본다. 내가 연 대화에서 **@멘션**으로 상대 PC 의 다른 에이전트를 부르면 받은 쪽이 그 에이전트를 대화 × 에이전트 세션으로
실행하고 양쪽 요청 · 답에 이름이 남는다(CLI `ask --conv --agent` 도).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 집중 상태 바꾸기 · 이 PC 저장 | 충족 · 실화면 ABANDON | E6 `focus_setting_and_peer_presence` · E3 `profile_menu` · E4 `focus_mention_labels` · E14(D28) |
| A2 집중 동안 새 요청 거부(실행 · 기록 · 알림 없음) · 끄면 다시 받음 | 충족 · 실PC ABANDON | E8 `focus_refuses_new_request_unrecorded` · E10 `focus_pauses_notices` · E11 `pong_carries_focus_and_presence` · E14 |
| A3 보낸 쪽 실패 + "상대가 집중 중" · 다시 보내기 | 충족 · 실화면 ABANDON | E8 · E6 · E3 `presence`(실패 상태일 때만 사유) · E14 |
| A4 진행 중 자동 왕복 다음 차례에서 멈춤 | 충족 · 실PC ABANDON | E9 `focus_stops_relay_both_sides`(받은 쪽 · 연 쪽 · 재개 뒤 사유 지움 · 시작 거절이면 켜짐 거둠) · E14 |
| A5 상대 집중 · 접속이 레일 · 머리글에 | 충족 · 실화면 ABANDON | E11 · E6 · E3 `presence` · E7 `schema12_focus_mention`(`Peer::online` 경계 단위 테스트) · E14 |
| A6 @ 후보 · 부른 에이전트가 답 · 답에 이름 | 충족 · 실PC ABANDON | E8 `mention_runs_named_agent_own_session` · E6 `mention_api_rules` · E12 `ask_mention_in_conversation` · E3 `mention` · E9 `mention_rejected_in_relay` · E14 |
| A7 같은 에이전트를 다시 부르면 이전 맥락(대화 × 에이전트 세션) | 충족 · 실PC ABANDON | E8(세션 New → Resume · 대화 에이전트 세션 그대로) · E7(`agent_sessions` · 폴더 변경 · 지울 때 비움) · E14 |
| A8 15 · 17 결정 유지 | 충족 | E13 `targets=55 missing=0` · E1 |
| A9 질의 · 요청 고르기 · 양쪽 이력 · 거르기 · 모아 보기 | 충족 · 실화면 ABANDON | E8 `ask_kind_and_done_sync_both_sides` · E6 `ask_kind_recorded_and_filtered` · E3 `ask_marks` · E4 · E14 |
| A10 받은 쪽만 완료 표시 | 충족 · 실PC ABANDON | E8(조작 `Done` 프레임 거절 포함) · E6 `done_only_by_receiver`(둘 다 준 PATCH 부분 적용 없음) · E3 · E14 |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1: high 2(목록을 못 받은 대화의 `@이름` 이 대화 에이전트로 조용히 감 — 사실 · 고침 — 보내지
  않고 목록 다시 받기, 실행 중 지우고 같은 이름 · 폴더로 다시 등록하면 지운 세션이 되살아남 — 사실 · 고침 — 에이전트 세대 `agents.uid`), 라운드 2:
  high 1(후보와 맞지 않는 멘션 꼴이 대화 에이전트로 감 — 사실 · 고침 — 한 번 막고 알림 · 다시 보내면 멘션 없이). 라운드 1 첫 실행은 codex 모델
  용량 오류로 실패해 같은 명령 재실행을 정본으로 했다(라운드로 세지 않음). 미조치 8건은 [eval.md](eval.md) → 원장.
- stage 리뷰 3회(서브에이전트, 읽기 전용) — Stage 1(M 2 · L 7 · N 7): 재개 뒤 남던 거절 사유 · 새로 고침 초점 회귀 · 시작 거절 뒤 켜짐 · 이미 받은 글의
  재전송 · 놓친 Ping 몰아 보내기 · 거절 코드 꼴 · 온라인 경계 — 반영. Stage 2(L 7): 완료 · 저장 원자성 · 완료 경합 · 제자리 갱신(초점 · aria-live) ·
  취소 경고 문구 · `Done` 조작 테스트 — 반영. Stage 3(M 1 · L 4): 첫 글이 닿지 않은 대화의 멘션이 받은 쪽 대화 에이전트를 바꾸던 것(프레임
  `mention` 분리) · 후보 넣기 자리 · 목록 다시 묻기 · 에이전트 지울 때 세션 · 키 처리 — 반영.
- plan 개정 6건(모두 위임, 평가 항목 수 그대로): 1 설계 문구 정합(`set_presence` · ping 204 · `set_done(bool)` · 경고 문구), 2 해석 7 보강(재전송은
  집중이어도 받음), 3 PATCH 응답 · 한 트랜잭션 · 새 대화 라디오 · E3 문구, 4 멘션 프레임 필드 · 지울 때 세션, 5 목록 없는 멘션 막기 · 에이전트 세대,
  6 맞지 않는 멘션 한 번 막기.
- 스키마 12 는 한 번에 올렸다(D32 — 배포 전, 이전 버전 대비 없음). 라운드 1 에서 같은 12 안에 `agents.uid` 를 더했다 — 이 Task 개발 중 빌드로 12 가
  된 DB 는 지우고 다시 만든다(사용자 PC 는 #17 빌드 DB 라 해당 없음).

## 산출

- `an-core`: 스키마 12(`messages.request` · `done_at` · `agent` · `refused` · `peers.seen_at` · `focus` · `agent_sessions`) · `settings::focus` ·
  `peers::set_presence` · `Peer::online(now)` · `ONLINE_WITHIN` · `chat::set_request` · `set_done` · `set_agent` · `set_refused(Option)` · `message_by_id` ·
  `agent_session` · `set_agent_session` · `ActivityFilter::Query` · `Request` · `messages_across(…, request, …)` · `agents::forget_sessions`(update · remove) ·
  `Agent.uid`(세대).
- `an-net`: `Pong{focus}` · 오류 코드 `focus` · `clean_code` · `Message` · `Post` 의 `request` · `Message.mention` · 새 `Done` · `Node::ping -> 집중` ·
  `watch_presence`(30초, 놓친 주기 건너뜀).
- `an-agent`: 집중 거부(재전송 예외 — `resent`) · 연 쪽 자동 왕복 멈춤 · 보낸 쪽 거절 기록(`refused_by_peer`) · `Compose` · `send_with` · `post_with` ·
  `start_relay_with` · `mark_done(id, done, saved)` · `inbound_done` · 대화 차례 `turn` · 멘션 실행 대상 · 세션 · 답 이름.
- `coworkagent-app`: 설정 `focus`(알림 멈춤) · 피어 JSON `online` · `focus` · `request` · `agent`(멘션) 본문 · `PATCH /api/messages/{id} {saved?, done?}` ·
  `ask` 거르기 · 활동 종류 `query` · `request` · CLI 멘션 · 웹UI(집중 칩 · 메뉴 · 레일 점 · 머리글 · 입력창 종류 · 요청 태그 · 완료 작업 · 실행 상세
  '요청' · 보낸 메시지 종류 · @ 후보 listbox · 목록 없음 · 맞지 않는 멘션 막기).

## 남은 것

- 실화면 · 두 PC 확인(E14) — `19.prototype-acceptance` TC-M(D28, `9f8580c`). 사용자 일괄 검증은 지금(Task 18 종결 뒤 — 사용자 지시 2026-10-12).
- 미조치 지적 — [eval.md](eval.md) `## 미조치 지적` → 원장.
- 위임으로 정한 것 중 뒤집기 쉬운 것 — 입력창 글 종류 = 라디오 두 값 · 멘션은 내가 연 대화만 · 요청 다시 보내기 = 요청 · 멘션 후보에 대화 에이전트도.

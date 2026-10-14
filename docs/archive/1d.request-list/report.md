# report — 1d.request-list

종결: 2026-10-13 · 브랜치 `task/1d-request-list` → main

## 결과

에이전트 설정을 단순하게 하고, 고치기 · 만들기 같은 일을 **요청 목록**으로 다룬다. 모델은 `opus` · `sonnet` · `haiku` 드롭다운(전역 · 상대별 "기본값"),
목록 밖 값은 저장 거부, 예전 값은 스키마 14 가 기본값으로. 에이전트 도구 칸을 없애고 모든 실행에 읽기 전용 5개(`Read` · `Grep` · `Glob` · `LS` ·
`NotebookRead`)를 늘 넣는다. 입력창에서 고른 **요청**은 받는 PC 에이전트를 **실행하지 않고** 받는 PC 요청 목록에 '대기' 로 남는다(집중 중에도). 받은
사람만 대기 → 수용 · 반려, 수용 → 완료로 바꾸고(한 줄 메모) 상태와 보내기 행을 한 트랜잭션으로 적어 `ReqState` 프레임이 outbox 로 요청자에게 간다.
요청자가 오래 꺼져 있거나 거절하면 그 요청에 "요청자에게 상태를 알리지 못함" · "다시 알리기", 같은 상태 재전송은 요청자 쪽에서 시각 · 알림을 바꾸지
않는다. 왼쪽 메뉴 '요청' 화면(받은 · 보낸 요청 · 상태 거르기 · 대기 수 배지)이 생기고 '내 활동' 의 질의 · 요청 거르기는 없앴다.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 모델 드롭다운 · 목록 밖 거부 · 예전 값 기본값 | 충족 | E6 `models_tools_model_list_only` · E4 `schema14_moves_done_and_models` · E3 `models_tools` · E8 |
| A2 도구 칸 없음 · 실행 인자 읽기 전용 5개 | 충족 | E7(runner 8) · E6 `models_tools_no_tools_field` · E8 |
| A3 요청은 실행 없이 '대기' · 집중 중에도 받음 | 충족 | E5 `new_conversation_request_not_run` · `continued_and_inbound…` · `focus_refuses_query_but_takes_request` · E8 |
| A4 받은 쪽만 상태 + 메모 · 요청자에 도착(꺼져 있어도) | 충족 | E5 `receiver_moves_state_with_note` · `state_reaches_requester_after_it_returns` · `state_give_up_marked_and_sent_again` · `forged…` · E4 · E8 |
| A5 '요청' 화면 · 대화로 이동 · 내 활동 요청 거르기 없음 | 충족 | E3 `request_rows` · `requests_route` · `side_nav` · E6 `models_tools_request_filter` · `…activity_has_no_ask_kinds` · E8 |
| A6 이전 완료 표시 → 완료 · 나머지 → 대기 | 충족 | E4 `schema14_moves_done_and_models` |
| A7 검증 명령 | 충족 | E1(316) · E2 · E3(47) · `check_readme` · `check_crates` · `check_ddd_docs` |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1: high 1(스키마 14 가 못 보낸 옛 완료 알림을 지움) · medium 1(상태 알림 포기가 안 보임) —
  사실 · 고침(plan 개정 3). 라운드 2: high 1(요청자 거절 뒤 복구 경로 없음) · medium 1(같은 상태 재전송이 시각 · 알림을 다시 만듦) — 사실 · 고침
  (plan 개정 4). 미조치 0.
- plan 개정 1(코드 대조 — 목록 밖 모델은 기본값 · 도구 입력 422) · 2(상태 보내기 행을 합치지 않음).
- 관찰: gate 체커 중 E1 이 간헐적으로 rc 101(같은 시각 다른 cargo 작업과 겹친 때) — 실패 시험 이름은 잡지 못함, 단독 실행은 모두 통과([eval](eval.md)).
- masterPlan S1 개정 · D17 · D21 · D22 · D31 보강 · D32 적용 — D 추가 없음.

## 산출

- `an-core`: 스키마 14(`req_state` · `req_note` · `req_at` · 목록 밖 모델 정리 · 옛 완료 알림 프레임 변환) · `ReqState` · `AskFilter` · `set_req_state` ·
  `clean_req_note` · `run_opts::MODELS` · `agents` 도구 정리.
- `an-net`: `Request::ReqState`(`Done` 대체) · `MessageKind::Ask` · `AskState`.
- `an-agent`: 요청 = 실행 없음(`request_ok` · `inbound_ask` · `post_with` 요청 미러) · `set_req_state` · `inbound_req_state`(멱등) · 포기 · 거절 표시 ·
  runner 도구 고정.
- `coworkagent-app`: `PATCH /api/messages/{id} {req_state, note}` · 메시지 JSON `req_state` · `req_note` · `req_at` · `GET /api/messages` `req` · 활동 종류
  정리 · 에이전트 `tools` 422 · 모델 검증 · 알림 `Ask` · `AskState` · 웹UI(모델 드롭다운 · 도구 안내 · '요청' 화면 · 메모 칸 · 다시 알리기 · 입력창 안내).
- 시험: `core/tests/requests.rs`(4) · `agent/tests/requests.rs`(7) · `app/tests/models_tools.rs`(4) · `view.test.mjs` +5 · `ux_literals` +1 · 기존 시험
  새 규칙으로(`roundtrip` 완료 표시 시험 삭제 → `requests.rs`, `ux_api` 둘, `runner`, `store`, `paths`, `remote_input`).
- 문서: README(요청 목록 행) · user-guide(4 · 6 · 7.3 · 7.5 · 9 · 13) · developer-guide(6.4 · 표 · 8.6) · 모듈 4 · PRODUCT 도메인 규칙 · masterPlan.

## 남은 것

- 19 인수 TC — 요청 보내기 · 수용 · 반려 · 완료 · 메모 · 꺼진 요청자 · 다시 알리기 · 모델 드롭다운 · 도구 칸 없음.
- E1 간헐 실패 — 종결 뒤 원인 식별 · 조치([부하에서 간헐 실패](../../troubleshootings/2026-10-13-flaky-tests-under-load.md)).
- GitHub 반영(push)은 사용자 확인 뒤.

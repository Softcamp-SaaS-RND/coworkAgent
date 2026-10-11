# report — 11.two-sided-chat

종결: 2026-10-11 · 브랜치 `task/11-two-sided-chat` → main

## 결과

두 PC 양쪽 사람이 같은 대화에 글마다 대상("에이전트에게" · "사람에게")을 골라 쓴다(D18 · D23). 사람 글은 상대에 미러되고, 실행 요청은
그 PC 의 대화 에이전트가 실행하며, 지난 실행 뒤 사람 글이 발화자와 함께 실행 입력 앞에 붙는다. 양쪽 순서는 램포트 시계 `lc`(스키마 6).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 "사람에게" 양방향 미러 | 충족 · 실화면 ABANDON | E4 `human_note_both_ways` · E6 · E8(D28) |
| A2 받은 쪽 "에이전트에게" · 양쪽 같은 순서 | 충족 · 실화면 ABANDON | E4 `receiver_asks_own_agent` · `concurrent_notes_same_order_both_sides` · `late_queued_request_not_skipped` · E8 |
| A3 실행 입력 앞붙임 | 충족 | E4 `notes_prepended_to_run` |
| A4 소유 대화만 · 재전송 1회 · 답 미러 조건 | 충족 | E4 `post_refused_when_foreign` · E5 |
| A5 이전 이력 `mine` · `exec` | 충족 | E3 `schema6_fills_mine_exec` |
| A6 검증 명령 | 충족 | E1 · E2 · E7 |

- 적대적 평가 2 라운드(상한) — high 4 전부 코드 반영: 양쪽 공통 순서(램포트 `lc`) · 실행 수락 후 미러 실패는 경고(라운드 1), 실행 대기열
  커서 제거 · 원격 `lc` 상한(라운드 2). 미조치 2건은 [eval.md](eval.md) → 원장.

## 산출

- `an-core`: 스키마 6(`mine` · `exec` · `lc`) · `add_human` · `stamp` · `observe` · `notes_between` · `last_exec_before` · `pending`.
- `an-net`: `Request::Post` · `message` · `reply` 의 `lc`. `an-agent`: `Chat::post` · `inbound_post` · 맥락 앞붙임.
- `agentnetwork-app`: `POST /api/conversations/{id}/messages {text, to_agent}` · 양쪽 입력창 · 발화자 이름 · 실행 글만 타임라인.

## 남은 것

- 실화면 확인 — `19.prototype-acceptance`(D28). 자동 왕복 — `12.auto-relay`.

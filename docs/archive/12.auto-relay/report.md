# report — 12.auto-relay

종결: 2026-10-11 · 브랜치 `task/12-auto-relay` → main

## 결과

대화를 여는 쪽이 내 에이전트를 함께 고르면 두 에이전트가 상한(기본 10, 1 ~ 50 왕복)까지 자동으로 주고받는다(D24). 연 쪽이 구동하고,
어느 쪽 사람이든 쓰거나 멈추면 멈추며, 재개는 연 쪽이 카운트 0 에서 한다. 받은 쪽이 멈추면 에이전트발 요청을 `stopped` 로 거절한다.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 상한까지 자동 왕복 · 양쪽 같은 순서 | 충족 · 실화면 ABANDON | E3 `relay_runs_until_cap` · E7(D28) |
| A2 양쪽 사람 멈춤 · 받은 쪽 거절 | 충족 · 실화면 ABANDON | E3 `human_write_stops_relay` · `receiver_stop_refuses_agent_requests` · E5 · E7 |
| A3 연 쪽 재개(카운트 0 · 거절 해제) | 충족 · 실화면 ABANDON | E3 `resume_restarts_count` · `stop_during_run_then_resume` · E5 · E7 |
| A4 페어링 · 대화 소유 · 가드 | 충족 | E3 · roundtrip 미페어링 거부(`by_agent` · `Relay`) |
| A5 검증 명령 | 충족 | E1 · E2 · E6 |

- 적대적 평가 2 라운드(상한) — high 4 · medium 1 전부 코드 반영: 실패 답 멈춤 · 전송 오류는 `sent` 유지 · 자동 전달 등록 원자화(라운드 1),
  상한에서 상대 재실행 없음 · 재개 때 넘기지 못한 내 답 다시 넘김(라운드 2). 미조치 3건은 [eval.md](eval.md) → 원장.

## 산출

- `an-core`: 스키마 7(`relay_*`) · `Relay` · `set_relay` · `relay_on` · `relay_step` · `set_blocked` · `add_agent_request` · `set_flags`.
- `an-net`: `Message.by_agent` · `Request::Relay`. `an-agent`: `start_relay` · `relay` · 연 쪽 구동 · 멈춤 · 재개.
- `agentnetwork-app`: `POST /api/conversations {relay_agent, relay_cap}` · `POST /api/conversations/{id}/relay` · 새 대화 자동 왕복 ·
  머리글 표시 · 멈춤 · 재개(`relayText`).

## 남은 것

- 실화면 · 실제 claude 확인 — `19.prototype-acceptance`(D28).
- 자동 왕복 매 회 받은 쪽 인바운드 알림(상한 50 이면 최대 50건) — 알림 묶기는 후속 검토.

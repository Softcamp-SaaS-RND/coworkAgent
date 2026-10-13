# report — 1c.durable-messaging

종결: 2026-10-13 · 브랜치 `task/1c-durable-messaging` → main

## 결과

보내는 PC 가 상대에게 가는 대화 프레임을 **SQLite outbox** 에 먼저 적고 상대별 줄로 보낸다(RabbitMQ 같은 브로커는 단일 실행 파일 · 서버 없음과 충돌해
기각). 상대가 꺼져 있으면 글은 "보내는 중" 으로 보관됐다가 상대가 돌아오면(받기 · Ping · 시작 알림) 바로, 아니면 5초부터 두 배 · 최대 5분 간격으로
다시 가고, 앱이 켜진 동안 24시간 닿지 않으면 "보내지 못함"(다시 보내기). 같은 uuid 라 상대에서 한 번만 실행하고, 받는 쪽 내부 오류는 `retry` 코드로
다시 보내게 한다. 사람 요청 · 미러 · 답 · 자동 왕복 넘김은 **글과 보내기 행을 한 DB 트랜잭션**으로 적는다(적대적 평가 반영). **CLI 트랜잭션** —
`ask --id <키>`(같은 키는 새로 보내지 않고 그 결과, 동시 호출도 한 번) · `wait <tx>`(끊긴 CLI 가 이어 받음) · `status <tx>`, 기다리는 동안 앱이 60초마다
상대에 `Status` 를 묻고 10분 닿지 않으면 종료 코드 7. **접속 상태는 통신할 때만** — 30초 주기 확인을 없애고 마지막 통신 시각 `seen_at`(보내기 성공 ·
받기 · Ping), 시작 알림 · 웹UI 열 때 · 타일 고를 때 확인(60초 안 중복 없음), 화면 "방금 통신 · n분 전 통신 · 통신 기록 없음". **처리경과** — 받는 쪽이
`claude -p --output-format stream-json` 이벤트에서 턴 · 도구 이름별 횟수(`mcp__*` → MCP) · 초를 세어 10초에 한 번 이하로 알린다 → "처리 중 — 2턴 ·
Read 3 · Grep 1 · 41초"(웹UI · CLI). README 는 사용자 관점(웹 서버 없음 · AI API 비용 없음 · 페어링 코워크 · 대화가 서버에 쌓이지 않음 · 기능 한 줄 +
매뉴얼 링크)으로 다시 쓰고 기술 절은 개발자 가이드로 옮겼다.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 꺼진 상대에 보낸 요청 → 켜지면 실행 · 답 | 충족 | E5 `request_to_offline_peer_runs_when_it_returns` · E12 원자성 · E11 "보내는 중" |
| A2 묻는 쪽이 꺼졌을 때 답 → 켜지면 도착 | 충족 | E5 `reply_reaches_asker…` · E12 `reply_not_recorded_without_outbox_row` |
| A3 재시작 뒤 이어 보냄 · 한 번만 실행 | 충족 | E5 재시작 · 한 번만 실행 |
| A4 한 상대가 막혀도 다른 상대 즉시 | 충족 | E5 상대 격리 |
| A5 한도 넘으면 "보내지 못함" · 상태 질의 | 충족 | E4 · E5 포기 · `Status` · E3 `outboxText` · `refusedText` · E11 |
| A6 `--id` 중복 없음 · `wait` 이어 받기 | 충족 | E6(동시 같은 키 포함 6건) |
| A7 `status` · 대기 · 실행 동안 기다림 · 닿지 않으면 종료 7 | 충족 | E6 |
| A8 통신할 때만 접속 시각 · 유휴 연결 0 | 충족 | E7 · E3 presence · E11 |
| A9 시작 알림으로 바뀐 포트를 따라감 · 60초 중복 없음 | 충족 | E7 |
| A10 처리경과 단계 요약(입력 없음) · 실물 claude | 충족 | E8 · E9(2.1.296 haiku — `Read 1` · 2턴 · 결과 그대로) · E11 |
| A11 기존 테스트 · 웹UI · 경고 0 | 충족 | E1 · E2 · E3(42) |
| A12 README 사용자 관점 · 기술 절 이동 | 충족 | E10 `readme ok` · stage5 |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1: high 2(글 기록과 outbox 사이 꺼짐 → 유실 · `--id` check-then-act) — 사실 · 고침(plan
  개정 1). 라운드 2: high 2(자동 왕복 넘김 원자성 · 포기가 보내는 중인 행을 덮음) — 사실 · 고침(plan 개정 2). 미조치 4건은 [eval.md](eval.md) → 원장.
- 단계 리뷰 1회(Stage 1, 서브에이전트) — future drop 때 `inflight` 고착 · 기다린 시간 부풀림 · 상대 N개 × 24시간 · 일시 오류 `bad_request` · 줄 작업
  panic · 실패 행 누적 — 반영(stage1).
- plan 대비 편차 4건(코드가 정본): 포기 때 `mirror` 는 로그만(내 실행 결과를 덮지 않음) · `seen_ago` → `seen_at` · 메시지 `outbox` 에 `next_at` 없음 ·
  `/api/tx` 에 `key` · `stalled` 추가 — [stage5](work/stage5.md).
- masterPlan D7 실현 · D14 보강 · D24 보강 · D32 적용 — D 추가 없음.

## 산출

- `an-core`: 스키마 13 — `outbox` · `cli_tx` 표, `outbox`(add · head · has_before · failing · retry · queued_for · hurry · fail · pending_peers ·
  prune_failed …) · `cli_tx`(create · by_key · by_message · due · unreachable · resume …) · `peers::touch`.
- `an-net`: `Request::Status` · `Response::Status` · `Progress.detail`(`StepDetail`) · 응답 코드 `retry` · `Node::announce` · `refresh` · `on_seen` ·
  보내기 · 받기 접속 시각(`watch_presence` 삭제).
- `an-agent`: `chat` 의 보내기 줄(`stage` · `dispatch` · `ship` · `enqueue` · `lane` · `settle` · `give_up` · `peer_back` · `recover`) · 상태 질의 ·
  CLI 트랜잭션(`tx_*` · `keyed` · `watch_tx`) · 단계 요약 합침(`step_every`), `runner` 의 stream-json · `run_process_lines` · `count_step` · `StepSink`.
- `coworkagent-app`: `/api/tx/{id}` · `/resume` · 메시지 JSON `outbox` · 진행 `detail` · 피어 `seen_at` · ping 60초 · CLI `--id` · `wait` · `status` ·
  종료 7 · 웹UI(`outboxText` · `stepText` · `presence` 마지막 통신 · `pingPeers`).
- 도구: `tools/check_readme.py`(PROJECT 검증 표 등록). 시험: `core/tests/outbox.rs`(5) · `agent/tests/durable.rs`(6) · `atomic.rs`(4) · `steps.rs`(4) ·
  `steps_live.rs`(1, ignore) · `app/tests/cli_tx.rs`(6) · `net/tests/presence.rs`(4) · `view.test.mjs` 3.
- 문서: README 개편 · 매뉴얼(user-guide · developer-guide · 모듈 4 · 라우팅) · PRODUCT 도메인 규칙(보내기 보장 · 접속 상태 · 처리경과 · CLI 트랜잭션) ·
  masterPlan.

## 남은 것

- 미조치 — [eval.md](eval.md) `## 미조치 지적` → 원장(완료 · 멈춤 · 마지막 기록 원자성 · 자동 왕복 답 기록과 넘김 사이 · 받은 대화 `queued` 표시 ·
  지운 상대 24시간 재시도).
- 개발자 가이드 `comm-query.png` 는 1c 전 흐름 — 다시 그릴지 사용자 결정(정본은 8.2 mermaid).
- GitHub 반영(README 포함 push)은 사용자 확인 뒤.
- 19 인수 TC — 꺼진 상대 · 보내지 못함 · CLI `--id` · `wait` · 종료 7 · 마지막 통신 · 처리 중 단계.

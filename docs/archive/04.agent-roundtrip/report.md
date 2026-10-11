# report — 04.agent-roundtrip

종결: 2026-10-11 · 브랜치 `task/04-agent-roundtrip` → main

## 결과

사람 → 원격 에이전트(`claude -p`) 1왕복 · 양쪽 이력 · 이어 쓰기(`--resume`) · 원격 메시지 알림. 새 domain 크레이트 `an-agent`(실행기 ·
조율), 스키마 4(에이전트 프로필 · 대화 · 메시지), 피어 요청 `agents` · `message` · `reply`, 웹UI "내 에이전트" · "대화".

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 프로필 등록 · 조회 · 삭제, 기본 도구 읽기 전용 | 충족 | E3 · E7 |
| A2 페어링된 상대만 · 없는 에이전트 거부 | 충족 | E5 `unpaired_or_unknown_agent_refused` · `forged_or_duplicate_reply_refused` |
| A3 D22 인자 · 표준 입력 | 자동 충족 · 실동작 ABANDON | E4 · E8(D28) |
| A4 왕복 · 양쪽 이력 · resume | 자동 충족 · 실동작 ABANDON | E5 · E7 · E8(D28) |
| A5 대화별 직렬 · 30분 종료 · 수신 즉시 응답 | 충족(오류 답 경로 일부) | E4(자손까지 종료) · E5(직렬 · 즉시 응답 · 재시작 · 기록 실패 오류 답). 실행 오류(`TimedOut`) → 오류 답은 같은 `Err` 경로 — 그 경로만 따로 시험하지는 않았다 |
| A6 받은 요청 · 답 알림 | 충족 | E5(이벤트) · E7(`Notifier` `message`) |
| A7 테스트 · 경고 0 · 의존 경계 | 충족 | E1 · E2 · E6 `crates=4 edges=6 violations=0` |

- 적대적 평가 2 라운드(상한) — high 6 · medium 1 전부 코드 반영: DB 대기열 · 시작 복구 · 기록 트랜잭션 · `reply_to` 검증 · 자손
  프로세스 종료(라운드 1), 대화별 보내기 직렬화 · 전송 오류는 `sent` 유지 · 웹UI 늦은 응답 폐기(라운드 2). 미조치(일부 조치 ·
  테스트 공백 · macOS 확인)는 [eval.md](eval.md) → 원장.
- plan 개정 2회(gate 건수 E4 4 → 5 · E5 4 → 9, 설계 본문 동기화).

## 산출

- `an-core`: 스키마 4 · `agents` · `chat`(DB 대기열 `next_pending` · `pending_conversations`, 상태 `running`) · `busy_timeout`.
- `an-net`: `Request::{Agents, Message, Reply}` · `Response::Agents` · `MAX_TEXT` · `PeerEvent::Message` · `Node::on_request` · `emit`.
- `an-agent`(신규): `runner`(`claude_args` · `parse_output` · `run_process` + `ProcessTree` · `find_claude` · `run_claude`) · `chat::Chat`.
- `agentnetwork-app`: `chat_api` · `main`(Chat 연결 · 시작 복구) · 웹UI 섹션 · `notify` 의 `message` 알림.
- 의존: an-agent 에 `windows-sys`(Windows 전용, Job Object — 프로세스 그룹과 같은 동작이라 D30 의 OS 연동 조건).

## 남은 것

- 실제 `claude -p` · PC 2대 왕복 · 맥락 유지 · 읽기 전용 거부 확인 — `19.prototype-acceptance`(D28).
- 같은 uuid 재전송 · 오프라인 재전송 — Phase 2. 자동 왕복(에이전트 ↔ 에이전트) — Phase 1.
- 위험 수용(그대로): D24 개정 — 원격 반복 요청이 수신 PC 사용량을 시간 제한 외 한도 없이 쓸 수 있다. 허용 도구에 `Bash` 등을 넣으면
  원격 상대가 그 PC 에서 명령을 실행할 수 있다 — 웹UI 경고 문구는 Phase 1 UI 에서.

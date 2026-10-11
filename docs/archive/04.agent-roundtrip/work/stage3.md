# stage3 — 프로토콜 · 조율 · 왕복

## 목표

[plan](../plan.md) Stage 3 — `an-net` 요청 `agents` · `message` · `reply` 와 `an-agent::chat::Chat` 조율. 대상 gate: E5(E6 은
앱이 `an-agent` 를 물어야 4/6/0 이 되므로 Stage 4 에서 측정).

## 산출 파일

- `crates/net/src/{proto.rs,node.rs}` — `Request::{Agents,Message,Reply}` · `Response::Agents` · `MAX_TEXT`(16 KiB) ·
  `PeerEvent::Message` · `Node::on_request` · `Node::emit`
- `crates/core/src/{chat.rs,db.rs}` — `has_message`(재전송 판별) · `busy_timeout(5s)`(노드 · 조율 · 앱이 같은 DB 파일을 다른 연결로 쓴다)
- `crates/agent/src/{lib.rs,chat.rs}` · `crates/agent/tests/{common/mod.rs,roundtrip.rs}`
- `crates/app/src/notify.rs` — `PeerEvent::Message` → 알림 `message`(D29 대상)
- `Cargo.lock` — Stage 2 에서 빠진 `an-agent` 항목

## 설계 결정

- 대화 요청은 `Node::answer` 가 먼저 **페어링 확인**(아니면 `not_paired`) 뒤 처리기로 넘긴다. 처리기가 없으면 `unsupported`.
- 받는 쪽(`message`): 본문 1 ~ `MAX_TEXT` 바이트 · 에이전트 존재(`unknown_agent`) · 대화 uuid 가 그 상대 소유(다른 상대 uuid →
  `bad_request`) · 같은 메시지 uuid 재전송은 `ok` 만(한 번만 실행). 사람 메시지 `queued` 저장 → 대화별 대기열 → 바로 `ok`.
- 대화별 직렬 작업(D19): 대화 id 마다 tokio 작업 하나가 순서대로 `spawn_blocking(run)`. 첫 실행은 `--session-id <새 UUID v4>`,
  이후 저장된 세션으로 `--resume`. 결과 → 세션 저장 · 사람 메시지 `done/error` · 에이전트 메시지 저장 → `reply` 를 상대 주소 ·
  키 고정으로 보낸다. 못 보내면 에이전트 메시지 `undelivered`.
- 답 본문은 `MAX_TEXT` 안으로 글자 경계에서 자르고 `…(잘림)` 을 붙인다.
- 보내는 쪽(`send`): 길이를 **보내기 전에** 확인(`InvalidInput`), 상대 거절이면 메시지 `error` + 거절 코드 포함 오류.
- `reply` 는 내가 연 `out` 대화(같은 uuid · 같은 상대)에만 붙는다 — 아니면 `unknown`.
- ponytail: 대화마다 작업 하나를 계속 둔다 — 대화가 아주 많아지면 유휴 작업 정리가 필요하다.

## 테스트 결과

- roundtrip 4 통과, an-agent 전체 8, workspace 전체 통과, clippy `-D warnings` 통과, `check_crates` `crates=4 edges=5 violations=0`.
- mutation(각각 되돌림):
  - `unknown_agent` 확인 제거 → unpaired_or_unknown_agent_refused FAIL
  - 대화별 대기열 대신 매번 새 작업 → inbound_does_not_wait_for_run FAIL(순서 · 재개)
  - 재개 대신 매번 새 세션 → followup_resumes_session · inbound_does_not_wait_for_run FAIL
  - 받는 쪽이 실행을 기다림(0.8초) → inbound_does_not_wait_for_run FAIL
  - 보내는 쪽 길이 확인 제거 → 처음엔 **생존**(상대 `bad_request` 로도 실패하므로). 오류 종류 `InvalidInput` 단정으로 강화 → FAIL.

## 다음 단계

Stage 4 — 앱 API(에이전트 CRUD · 원격 에이전트 · 대화 · 메시지) · 웹UI · `main` 연결 · chat_api 테스트 · E6.

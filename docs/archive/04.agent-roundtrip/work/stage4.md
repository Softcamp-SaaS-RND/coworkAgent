# stage4 — API · 웹UI · 알림

## 목표

[plan](../plan.md) Stage 4 — 앱 API(에이전트 CRUD · 원격 에이전트 · 대화 · 메시지) · 웹UI 섹션 · `main` 연결. 대상 gate: E1 · E2 · E6 · E7
(E8 수동은 D28 에 따라 인수 Task 로).

## 산출 파일

- `crates/app/src/chat_api.rs`(신규) · `crates/app/src/{lib.rs,main.rs}` · `crates/app/Cargo.toml`(`an-agent` 의존)
- `crates/app/web/{index.html,app.js,app.css}` — "내 에이전트" · "대화" 섹션
- `crates/app/tests/chat_api.rs`(2건)
- `crates/agent/src/chat.rs`(`Chat::db` 공개) · `crates/core/src/chat.rs`(`Dir::as_str` 공개)

## 설계 결정

- 경로는 plan 그대로: `GET/POST /api/agents` · `DELETE /api/agents/{id}` · `GET /api/peers/{id}/agents` · `GET/POST /api/conversations` ·
  `GET/POST /api/conversations/{id}/messages`. 모두 기존 guard 뒤(토큰 · Host · Origin).
- 프로필 등록: 이름 1 ~ 64자 · 제어 문자 없음, 폴더는 **있는 절대 경로**만(상대 경로는 앱 현재 폴더 기준이 되어 의도와 어긋난다),
  같은 이름 409, 도구가 비면 읽기 전용 기본값.
- 이어 쓰기는 내가 연(`out`) 대화만 — 받은 대화(`in`)는 400. `Chat::send` 도 `out` · 같은 상대만 받아 이중으로 막힌다.
- 오류 코드: 없음 404 · 입력 400 · 중복 409 · 페어링 아님 · 상대 거절 403 · 그 밖(네트워크) 502. 상대 거절 코드는 본문 `error` 에 남는다.
- `main`: `Chat::new(db, node, |input| run_claude(&find_claude(), &input))` — 실행 파일은 실행마다 찾는다(앱 실행 뒤 설치해도 된다).
  `PeerEvent::Message` 는 기존 `node.on_event → Notifier` 경로로 알림이 된다.
- 웹UI: 상대 선택 → 원격 에이전트 목록 → 새 대화, 대화 목록(→ 보낸 · ← 받은) → 메시지(사람 · 에이전트 · 상태) → 이어 쓰기.
  전부 `textContent`. 알림이 꺼져 있으면 이벤트가 안 오므로 5초마다 다시 읽는다(ponytail 표시).

## 테스트 결과

- chat_api 2 통과, workspace 전체(35 묶음) 통과, clippy `-D warnings` 통과, `check_crates` `crates=4 edges=6 violations=0`,
  `node --check`(ES 모듈) 통과.
- mutation(각각 되돌림):
  - 폴더 절대 경로 확인 제거 → agents_crud_api FAIL
  - 이어 쓰기가 새 대화를 엶 → conversation_api_roundtrip FAIL
  - 이어 쓰기의 `out` 확인 제거 → **생존** — `Chat::send` 가 `out` 아닌 대화를 404 로 막아 결과가 같다(이중 방어, 기대대로).

## 다음 단계

평가 — gate 원장(E8 ABANDON → D28) · 적대적 평가(BASE d5fadb4).

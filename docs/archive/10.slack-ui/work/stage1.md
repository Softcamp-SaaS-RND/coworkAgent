# stage1 — 스키마 5 · 실행 정보 · 원 요청 대응

## 목표

[plan](../plan.md) Stage 1 — 답에 실행 정보 · 원 요청 대응을 싣고 양쪽에 저장, 대화 요약. 대상 gate: E3 · E4 · E5.

## 산출 파일

- `crates/core/src/{db.rs,chat.rs}` — 스키마 5(`messages` 에 `reply_to` · `turns` · `duration_ms`), `Message` 에 `created_at` · `reply_to` ·
  `turns` · `duration_ms`, `add_reply` · `summaries` · `Summary` · `TITLE_CHARS`(40)
- `crates/net/src/proto.rs` — `Request::Reply` 에 `turns` · `duration_ms`(없으면 `None`)
- `crates/agent/src/{runner.rs,chat.rs}` — `RunOutput` 실행 정보(`num_turns` · `duration_ms`), `record` · `inbound_reply` 가 `add_reply`
- 테스트: `store`(+`reply_meta_and_summaries`) · `runner`(`parse_claude_json` 보강) · `roundtrip`(+`reply_carries_run_meta` ·
  `reply_without_meta_accepted`) · `paths`(스키마 5)

## 설계 결정

- 원 요청 대응은 **로컬 id**(`reply_to`) — 보내는 쪽은 `inbound_reply` 가 찾은 `asked.id`, 받는 쪽은 `process` 의 `asked.id`.
- `inbound_reply` 는 인자가 많아 `Request` 를 통째로 받아 안에서 푼다(clippy `too_many_arguments`).
- 요약 정렬: 마지막 메시지 시각 → 마지막 메시지 id → 대화 id 내림차순(같은 초 안에서도 결정적).
- `#[serde(default)]` 는 `Option` 이라 없어도 같은 동작 — 이전 버전 호환을 코드에 드러내려고 남긴다(mutation 으로 제거해도 생존, 기대대로).

## 테스트 결과

- an-core store 4 · an-agent runner 5 · roundtrip 11 통과, workspace 전체 · clippy `-D warnings` 통과.
- mutation: Reply 에 `turns` 안 실음 → `reply_carries_run_meta` FAIL · 제목 길이 40 → 45 → `reply_meta_and_summaries` FAIL.

## 다음 단계

Stage 2 — API 확장 · `/view.js` · 화면 로직 모듈(`node --test`).

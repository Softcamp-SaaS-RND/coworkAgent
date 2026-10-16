# eval.md — 1m.context-compact 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: fa036152cd4ea6ea64fcf2f8066e391cccd5418c

- [x] E1: A1·A3·A4·A5·A6·A7·A8·A9 — 워크스페이스 테스트 전부 통과(usage 읽기 · 압축 · 비우기 · 요약 주입 · 꺼진 상대 보관 · unsupported · 자동 압축 · 설정 · 연동 · API 400)
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A9 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

- [x] E3: A1·A2·A6·A9 — 화면 로직 node 테스트 103건 이상 · 실패 0(`contextSize` · `contextLabel` 단계 · `ctxMenu` 꺼짐)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (10[3-9]|1[1-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 117.6815

- [x] E4: A1·A2·A4 — 웹UI 문구 · 자리(`#ctx_btn` · 압축 · 비우기 메뉴 문구 · 구분선 문구 · 설정 `auto_compact`)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E5: A9 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: PASS — DDD 문서 규약 위반 없음

- [x] E6: A9 — README 사용자 관점
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: readme ok

- [x] E7: A9 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: crates=4 edges=6 violations=0

- [x] E8: A1·A2·A4·A6 — mock 화면(가상 데이터, 다크 · 라이트): 머리 "컨텍스트 n / 1M" · 60 · 80% 색 · 알 수 없음 · 메뉴 · 확인 · 실행 중 꺼짐 · 압축 · 비움 구분선 · 요약
  EVIDENCE: 2026-10-16 mock7(scratchpad m1/m1.mjs) 다크 13/13 · 라이트 13/13 — 42k / 1M ok · 650k warn(다크 rgb(245,184,61) · 라이트 rgb(138,87,0)) · 830k high(rgb(255,107,107) · rgb(192,57,43)) · "컨텍스트 알 수 없음" · 0.4.0 거절 구분선 "상대가 0.5.0 이상이어야 합니다" · 자동 압축 구분선 뒤 35k · 요약 펼침 · 비움 구분선 뒤 0 / 1M · 메뉴 2항목 아이콘 · 확인 "컨텍스트 압축" · 실행 → "요약 중…" · 진행 중 · 실행 중 · 답 없음 꺼짐 · 설정 자동 압축 체크 켜짐 · JS 오류 · 4xx 0

- [x] E9: A3 — 실제 claude 1회(이 PC · 임시 폴더 에이전트 · 비파괴 — 실제 상대 · 이력은 건드리지 않음): 실행 → 크기 읽힘 → 요약 실행 → 새 세션 첫 실행에 요약이 들어가
  EVIDENCE: 2026-10-16 `cargo test -p an-agent --test context_live -- --ignored` 1 passed(Claude Code 2.1.299 · haiku · low · 13.3초, 시험용 두 앱 · 임시 폴더) — 답 "노을-42" · 창 6157 → 압축 Done(요약에 요청 · 답 "노을-42") → 새 세션(Session::New, 입력에 <대화 요약>) "파일을 다시 읽지 말고" 물음에 "노을-42" · 창 6484


WEAK: E8 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)
WEAK: E9 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

- 2026-10-16 · `/codex:adversarial-review --base db1d1ab --scope branch` · 세션 codex-review · HEAD `13567ab`(요청과 일치) · job `review-mvadm6n8-l9wser` —
  Verdict needs-attention(high 3).
- **[high] 압축 seed가 폴더 변경·재등록 뒤 새 에이전트에 유출됨**(`core/chat.rs` 972-996) — 판단: **사실**. `ctx_seeds` 가 이름으로만 묶여 `forget_sessions`
  (폴더 변경 · 삭제)가 지우지 않았다. **고침** `e8a2258` — `forget_sessions` 가 그 에이전트의 대화 · 멘션 요약을 함께 지우고, 압축 끝 트랜잭션은 요약을 만든 에이전트의
  세대(`uid`)가 지금과 같을 때만 남긴다. 시험 `seed_dropped_when_agent_folder_changes`.
- **[high] 자동 왕복의 원격 압축 실패가 성공으로 표시됨**(`agent/chat.rs` 3318-3320) — 판단: **사실**. 이 PC 도 처리한 글(`exec`)은 상대 `failed` · 거절 ·
  포기를 버렸다. **고침** `e8a2258` — 그 글에 `refused = peer_{코드}` 를 남겨 구분선이 "상대 PC 쪽은 실패(사유) — 다시 누르면 다시 요청" 으로 보인다(이 PC 쪽
  처리 결과 · 상태는 그대로 — 이 PC 세션은 이미 비워졌으므로 실패로 바꾸면 그쪽이 거짓). 권고의 local/remote 상태 열 분리 대신 기존 `refused` 열로(스키마
  더하지 않음). 시험 `relay_peer_failure_not_hidden` · node `context_note`.
- **[high] 처리 완료와 결과 전송이 원자적이지 않아 요청자가 영구 대기함**(`agent/chat.rs` 3393-3411) — 판단: **사실**. **고침** `e8a2258` — `run_context` 가 세션
  비우기 · 요약 남기기 · 구분선 끝 · 회신 줄(`context_done`, `pending`)을 한 트랜잭션으로(압축 요약 실행은 그 앞, 기록 없이). 사이에 꺼지면 구분선은
  `running` 으로 남아 다시 켤 때 실패 · 회신(세션 그대로)으로 끝난다. 꺼짐 창 재현 시험은 없음(트랜잭션 구성으로 창을 없앰).

## 적대적 평가 — 라운드 2/2

- 2026-10-16 · `/codex:adversarial-review --base db1d1ab --scope branch` · 세션 codex-review(라운드 1 과 같음) · HEAD `9438294`(요청과 일치) · job
  `review-mvae93v0-3gimjt` — Verdict needs-attention(high 1 · medium 2).
- **[high] `context_done` 재시도 포기 후 요청이 영구 고착된다**(`agent/chat.rs` 1056-1064) — 판단: **사실**. 요청자가 포기 한도(24시간)보다 오래 닿지 않으면 처리
  결과 줄이 실패 보관돼 요청자 구분선이 `sent` 에 묶이고 수동 압축도 막혔다. **고침** `108f3a6` — 포기 때 `context_done` 줄은 건너뛴다(돌아오면 보낸다). 시험
  `done_kept_while_requester_away_past_give_up`(포기 한도 1초 · 9초 단절 — 고치기 전 코드로 돌리면 실패 확인).
- **[medium] 컨텍스트 작업이 더 오래된 실행 요청을 추월한다**(`core/chat.rs` 1206-1214) — 판단: **사실**. **고침** `108f3a6` — 줄 앞자리는 `auto_compact` 만
  (`coalesce(ctx_op = 'auto_compact', 0) DESC, id`), 수동 압축 · 비우기는 실행 대기 · 실행 중 글이 있으면 409 거절. 받은 쪽은 요청이 닿은 순서(id)대로. 시험
  `manual_waits_for_runs_and_only_auto_jumps_queue`.
- **[medium] 자동 왕복 부분 실패 뒤 남은 컨텍스트가 `0 / 1M` 으로 표시된다**(`view.js` 1087-1099) — 판단: **사실**. **고침** `108f3a6` — `contextSize` 가
  `peer_*` 부분 처리 구분선에서는 이 PC 쪽(`mine`) 세션 값만 지우고 상대 세션 값은 남긴다. node `context_size` 단정 2개.
- 상한 도달 — 재리뷰하지 않는다.

## 미조치 지적

| 지적 원문(요약) | 판단 | 미조치 사유 |
|---|---|---|
| 라운드 1 권고 — 자동 왕복 구분선에 local/remote 결과를 별도 상태로 영속화 | 사실(권고) | 기존 `refused` 열에 `peer_{코드}` 로 남겨 화면 · 크기 계산이 부분 처리를 구분한다 — 스키마를 더 늘리지 않았다. 상대 쪽만 다시 처리하는 단추는 없다(다시 누르면 양쪽 다시 요청) |
| 라운드 1 권고 — 처리 끝 · 회신 사이 꺼짐 창을 재현하는 재시작 시험 | 사실(권고) | 한 트랜잭션으로 묶어 창 자체를 없앴다. 프로세스를 그 순간 죽이는 시험 장치가 없어 재현 시험은 두지 않았다 |
| 라운드 2 권고 — 재접속 시 구분선 uuid 로 상태 재조회 · 재전송 프로토콜 | 사실(권고) | 같은 결과를 "처리 결과 줄은 포기하지 않음"으로 얻었다(줄이 남아 돌아오면 보냄). 받은 쪽 DB 를 잃은 경우의 재조회는 없음 — 19 인수 TC 관찰 |
| E9 범위 밖 — 0.4.0 상대에 `Context` 거절 · 실제 두 PC(상대 압축 · 꺼진 상대 · 자동 왕복 연동) | 사실(확인 필요) | 이 PC 하나로는 0.4.0 노드 · 두 PC 를 띄울 수 없다 — `Pong` · 답 프레임 옛 꼴 읽기만 시험. 원장 `#1m R2-PC` 조건 대기 · 19 인수 TC |

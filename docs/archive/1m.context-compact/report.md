# report — 1m.context-compact

종결: 2026-10-16 · 브랜치 `task/1m-context-compact` → main

## 결과

대화 머리에 그 대화 Claude 세션의 창 사용량("42k / 1M", 상한 1M 고정 · 60% 주황 · 80% 붉음 · 0.4.0 상대는 "알 수 없음")을 보이고, 그 단추에서 압축(세션마다
요약 실행 → 세션 비움 → 다음 새 세션 첫 실행에 요약 한 번) · 비우기(세션만, 이력 그대로)를 고른다. 결과는 양쪽 대화에 구분선(압축은 "요약 보기")으로 남는다.
답하는 PC 가 실행 뒤 800k 를 넘으면 자동 압축(설정 → 실행, 기본 켜짐). 상대 쪽 세션은 새 피어 프레임 `Context`(outbox — 꺼져 있으면 보관, 보내기 전
`Pong.ctx` 로 0.4.0 거절), 자동 왕복은 두 PC 세션 함께 · 받은 요청은 다시 넘기지 않는다. 스키마 19.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 크기 표시 · 알 수 없음 | 충족 | E1 `compact_from_asker…`(답 ctx 기록) · E3 `context_size` · `context_label` · E8 |
| A2 메뉴 · 확인 · 실행 중 꺼짐 | 충족 | E3 `ctx_menu` · E4 · E8 · E1 `manual_waits_for_runs…`(서버 409) |
| A3 압축 → 요약 새 세션 · 양쪽 요약 | 충족 | E1 `compact_from_asker…` · E9 실제 claude("노을-42" 기억) |
| A4 비우기 · 이력 그대로 · 구분선 | 충족 | E1 `clear_drops_sessions…` · E8 |
| A5 0.4.0 상대 | 충족(시험 앱 범위) | E1 `old_pong_has_no_context` — 실제 0.4.0 노드 확인은 원장 `#1m R2-PC` |
| A6 자동 압축 · 색 | 충족 | E1 `auto_compact_over_80_percent_unless_off` · E8(다크 · 라이트 색) |
| A7 꺼진 상대 보관 | 충족 | E1 `request_kept_while_peer_offline` · `done_kept_while_requester_away…` |
| A8 자동 왕복 연동 | 충족 | E1 `relay_compacts_both_sessions_once` · `relay_peer_failure_not_hidden` |
| A9 검증 명령 | 충족 | `check_gates` 9/9 |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1 high 3(요약 세대 경계 · 자동 왕복 상대 실패 숨김 · 처리 끝과 회신 비원자) · 라운드 2 high 1 · medium 2
  (회신 포기 · 수동 압축 추월 · 부분 처리 크기 0) 모두 고침. 미조치 4(권고 대안 3 · 두 PC 실제 확인 — [eval](eval.md), 원장 `#1m R2-PC`).
- plan 편차: 없음(해석 1 ~ 13 그대로). 진행 중 거절에 "실행 대기 · 실행 중 글"(라운드 2) · 줄 앞자리는 자동 압축만(라운드 2) — plan 해석 7 을 좁힘.
- 1M 실측(Claude Code 2.1.299): `sonnet` · `haiku` · `opus` 모두 `contextWindow` 1000000 — `[1m]` 지정 불필요.

## 산출

- 통신 — `Request::Context` · `CTX_OPS` · `Reply.ctx` · `Post.ctx` · `Pong.ctx` · `Node::knows_context`.
- 저장 — 스키마 19(`messages.ctx_tokens` · `ctx_op` · `ctx_seeds`) · `Author::Context` · 컨텍스트 함수들 · `agents::forget_sessions` 가 요약도 지움 ·
  설정 `auto_compact`.
- 실행 — `RunOutput.ctx_tokens`(`usage.iterations` 마지막) · `SUMMARY_PROMPT` · `Chat::context` · `run_context` · `compact` · 요약 주입 · 자동 압축 · `context` /
  `context_done` 줄.
- 웹 — `#ctx_btn` · 메뉴 · 확인 · 구분선 · 설정 체크, `view.contextSize` · `contextLabel` · `contextBusy` · `ctxMenu` · `contextNote`. API
  `POST /api/conversations/{id}/context`.
- 시험 — `context.rs` 10 · `context_live.rs`(실제 claude, `--ignored`) · `runner` `parse_context_tokens` · `ux_api` 1 · `ux_literals` 20 · node 104.
- 문서 — 사용자 가이드 7.4 · 13장 · developer-guide · 모듈 매뉴얼 4 · PRODUCT · README · 캡처 `workspace` · masterPlan D19 보강 · 원장 1행.

## 남은 것

- 19 인수 TC — 실제 두 PC 압축 · 비우기 · 꺼진 상대 · 자동 왕복 연동, 0.4.0 상대 거절 안내(원장 `#1m R2-PC`).
- 릴리스 0.5.0(통신 형식 · 스키마 19)은 사용자 확인 뒤.

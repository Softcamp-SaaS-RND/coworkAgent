# report — 1u.scoped-refresh

종결: 2026-10-19 · 브랜치 `task/1u-scoped-refresh` → main

## 결과

웹UI 가 알림마다 전체를 다시 읽던 것을, 알림과 별개인 **변경 신호**(SSE `event: change`)가 실은 자원만 다시 읽게 바꿨다(D39 ② — 문구 개정 사용자 승인).
신호는 알림 설정 · 일시 중지 · 집중과 무관하게 간다 — 알림을 끈 사용자도 상대 글 · 편집 · 삭제 · 진행이 5초를 기다리지 않고 보인다. 5초 새로 고침은 그대로.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 대화 변경 → 소비자 전부 · 설정 제외 · 다른 대화면 본문 제외 | 충족 | E4 node(`loadersFor`) |
| A2 빈 · 모르는 종류 · `peer` → 전체, 합치기 | 충족 | E4 node(`mergeChanges`) |
| A3 원천별 자원 키(상대 요청 처리 · 메시지 알림 · 페어링) | 충족 | E5(`change_keys_by_source`) · E7(`inbound_edit_signals_change`) |
| A4 알림 끔 · 일시 중지 · 집중에도 신호, 건수 없음 | 충족 | E5(`change_ignores_prefs` · `change_not_counted`) |
| A5 뒤처지면 전체 신호, SSE `event: change` | 충족 | E5(`change_lagged_is_full`) · E6 |
| A6 회귀 없음 · 5초 그대로 | 충족 | E1 · E2 · E3(node 133) · E8 · E9 |
| A7 실화면 일관(5초 끔 · 두 PC) | 19 인수로 이월 | E10 ABANDON(D28) — 사용자 "인수테스트에서 확인하고 종결 처리"(2026-10-19), 원장 `#1u R1-H1` 조건 대기 |

- 적대적 평가 2 라운드(상한, `codex-review`) — 라운드 1 high 1(A7 이월 — 승인 사항) · medium 1(다시 읽기 겹침 → `OneAtATime`), 라운드 2 medium 1(실패 시 쌓인 요청 유실 →
  묶음 단위). 미조치 3([eval](eval.md)) — 사용자 수용, 원장 `#1u R1-H1` · `R2-N1` · `R2-N2`.
- 구현자 해석(plan 승인 때 함께 승인): 결정 2 "원격 이벤트" = 상대 요청 처리 전부(`chat.rs` `handle`) + 페어링 + 메시지 알림. 대화 변경은 연결 로더도 다시 읽는다(`seen_at`).

## 산출

- an-net `PeerEvent::Changed` · `Message.conv` · `Request::conv()`, an-agent `handle` 변경 신호, an-app `Change` · `Res` · `subscribe_changes` · `next_change` · SSE 합치기.
- 웹UI `view.js` `CONV_LOADERS` · `loadersFor` · `mergeChanges` · `unionLoaders` · `OneAtATime`, `app.js` `refresh(only)` · `onChange` · `refreshFailed`.
- 시험: Rust 6(notifier 4 · events_api 1 · message_tidy 1) + 기존 1 보정, node 9.
- 문서: masterPlan D39 ② · Phase 1 후보 줄(5초 축소로 교체) · 개정 기록, PRODUCT "웹UI 다시 읽기", web-ui "알림", an_app `notify::routes`, 원장 5행.

## 남은 것

- 19 인수 TC: 1u E10(5초 끈 빌드 · 두 PC · 알림 끔 · 집중).
- 다음 후보: 5초 새로 고침 축소 — 전제 E10 확인 + 이 PC 변경 신호 · 시간 경과 접속 표시(masterPlan Phase 1 후보 줄).
- 저장소가 `cargo fmt` 기준과 다르다(이 Task 밖 파일 수십 개) — 손대지 않음, 필요하면 별도 정리.

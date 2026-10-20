# report — 1y.data-update

종결: 2026-10-20 · 브랜치 `task/1y-data-update` → main · 1.1.0 사전 배포

## 결과

주고받은 메시지 "기간" 1 ~ 12주(기본 4주 — 보기만), 요청 archive(완료 7일 뒤 자동 · 손으로 · 기본 숨김 · "archive 항목 보기" 로 따로 · 이 PC 에만), 시작 때 DB VACUUM,
받은 요청 "대기" 되돌리기 서버 보완. 저장 스키마 23(`messages.req_archived`). 자동 업데이트(S6)는 만들었다가 사용자 지시로 기능째 걷어냈다
("서명 관리 기능은 rollback" → "git release 에 신규파일 등록 시 자동 배포되는 기능은 삭제.") — D12 · D36 · S6 그대로(TODO).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 기간 1 ~ 12주 · 기본 4 · 목록 · 요약 거름 · 남음 · 400 | 충족 | E4(`peer_weeks_setting_persists` · `peer_messages_within_weeks`) · E9 · E11(CDP) · node `merge_page_since` |
| A2 완료 7일 뒤 자동 · 손으로 archive | 충족 | E5(`request_archive_auto` · `request_archive_manual` · `request_archive_state_change`) |
| A3 기본 숨김 · "archive 항목 보기" 로 따로 | 충족 | E5 · E9 · E11(CDP) |
| A4 이 PC 에만 | 충족 | E5 `request_archive_local_only` |
| A5 VACUUM · 실패해도 뜬다 | 충족 | E6(`vacuum_sparse` · `vacuum_small_skip` · `?` 아님) |
| A6 자동 업데이트 코드 · 의존 없음 | 충족 | E7 |
| A7 릴리스 절차(사전 배포) 문서 · 사용자 문서에 자동 업데이트 없음 | 충족 | E8 |
| A8 1.1.0 사전 배포 · 자산 SHA-256 | 충족 | E12(릴리스 뒤 `gh release view`) |
| A9 회귀 없음 | 충족 | E1 · E2 · E3(node 151) · E10 |

- 적대적 평가 2 라운드(상한, Orca Codex 세션 `review2` — `codex-peer-review`) — 라운드 1 high 2 · medium 1, 라운드 2 high 2 · medium 1. 기간 지적(R1-M1 · R2-M1 — 받아 둔 꼬리 ·
  다른 창 기간)은 반영(`mergePage` `since` · 설정 다시 읽기). 업데이트 지적(기동 실패 되돌리기 · 교체 중단 창 · 잠금 인계)은 반영했다가 기능째 걷어내 대상 없음. 미조치 없음([eval](eval.md)).
- 사용자 결정(2026-10-20): 서명 관리 되돌림(회사 인증서) → 서명 없음 위험 수용 → 자동 배포 기능 삭제 · 병합 · 릴리스 등록.
- 구현자 해석 4 ~ 7(사용자 승인 — plan 승인과 함께), 1 · 2 · 3 · 8 은 자동 업데이트와 함께 효력 없음.
- plan 밖 변경: 기존 시험 전제(`requests.rs` 완료 501건 `req_at` · `store.rs` · `paths.rs` 스키마 번호), 1.1.0 릴리스 = #1x + #1y(1.0.1 뒤 첫 릴리스) — 정식이 아닌 사전 배포(D36 그대로,
  intent 결정 22 "정식(Latest)" 은 업데이터와 함께 무효).

## 산출

- an-core: 스키마 23 `req_archived`, `db::vacuum_if_sparse`, `chat` `ARCHIVE_DAYS` · `Message::archived` · `archived_sql` · `set_req_archived` · `unix_now` · `AskFilter::Request(_, archived)` ·
  `req_counts.archived` · `messages_page` / `peer_message_counts` `since`, `run_opts::peer_weeks`.
- an-app: `POST /api/messages/{id}/archive` · `archived` 질의 · 기간 거르기 · `ui_peer_weeks` · `PATCH` `wait`, 시작 VACUUM.
- 웹UI: "기간" 고르기 · "archive 항목 보기" · archive 단추 · 머리 "archive n" · 받아 둔 꼬리 기간 거르기.
- 시험: Rust `ux_api` 새 8 · `store` 새 3, node 새 4(151), `ux_literals` 5줄.
- 문서: masterPlan D35 보강 · S6 주석 · 개정 기록 · 종결 목록, PRODUCT, README(1.1.0), 사용자 가이드, 개발자 가이드(2.4 릴리스 · 4.1 · 7.2 · 7.5 · 8.6), web-ui · an_app · an_core, 원장 2행.

## 남은 것

- 자동 업데이트(S6) · 코드 서명은 D36 TODO 그대로 — 회사 인증서와 함께.
- 장기 데이터 중 대화 글 · 내 활동 보관 기한 · 스키마 사본 정리는 검토만(intent "장기 데이터 검토" 표).

# report — 1x.session-ui

종결: 2026-10-20 · 브랜치 `task/1x-session-ui` → main

## 결과

"+ 세션 생성" = 글 없이 대화만 만들고 첫 글은 그 대화방 입력창에서(자동 왕복은 첫 글부터), ⋯ 메뉴 = "기록 삭제 ›(내 글만 삭제 · 모든 사용자 글 삭제)" · "나가기"(확인 뒤 목록에서 삭제),
사이드바 대화 줄 손잡이 끌기(☆ 없음), 타일 머리 부제 정리 · 타일 크기 보통 / 작게, 기본 테마 라이트, 받은 요청 알림 시안 B · 효과음(D40 개정 · D26 · D35 · S4 · D27).
저장 스키마 22 · 프레임 `Clear{conv, id}` · `Pong.clear` · 오류 코드 `CLEAR_*`. 클립보드 이미지 붙여넣기는 검토만(masterPlan 후보 줄).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 세션 생성 화면(제목 · 칸 없음 · "생성" · 상대 미리 선택) | 충족 | E7 node(`compose_defaults` · `session_label`) · E8 · CDP(상대 미리 선택) |
| A2 빈 대화 생성 · 첫 글 상대 도달 · 첫 글 자동 왕복 | 충족 | E4 `session_open` 4 · E6 `create_session_without_text` |
| A3 "새 대화" 라벨 0 · "세션 생성" | 충족 | E8 · E9 |
| A4 요청 안내 문구 없음 | 충족 | E8 `hint=0` |
| A5 내 글만 삭제 — 대화 · 그룹 · 즐겨찾기 그대로, 이 PC 글 · 실행 · 세션 0, 상대 사본 그대로 | 충족 | E5 `conv_clear` |
| A6 모든 사용자 글 삭제 — 양쪽 · 관람자 글 0, 참여 상대 것만 받음, 실행 · 보내는 중 막음, 이전 버전 거절 | 충족 | E5(12 — 멱등 · 순서 · 재실행 없음 · 늦은 이전 버전 · 보내는 중 포함) · E6 `clear_route` |
| A13 ⋯ 메뉴(기록 삭제 › · 나가기 · 나간 대화 · 관람자) · 나가기 확인 | 충족 | E7 `conv_menu_items_1x` · CDP(하위 펼침 · 확인 문구) |
| A7 대화 줄 ☆ 없음 · 손잡이 끌기 · HTML5 끌기 없음 | 충족(자동 · CDP) · 같은 절 옮기기 실화면 이월 | E8 `star=0 html5=0 grip=1` · CDP 그룹 타일 넣기 `dragIntercepted false` · E12 |
| A8 머리 부제 지문 · 접속 문구 없음 | 충족 | E7 `side_sub` · CDP |
| A9 타일 크기 저장 · 400 · 작게 = 가로 줄 | 충족 | E6 `tile_size_setting_persists` · CDP(레일 168px) |
| A10 기본 테마 라이트 | 충족 | E6 `theme_default_light` |
| A11 클립보드 이미지 검토 후보 줄 | 충족 | E10 |
| A14 받은 요청 알림 글 · 효과음 | 충족(자동) · 실화면 이월 | E6 `request_notice_text` · `notify_sound_setting` · E7 `request_notice` · E12 ABANDON(D28) |
| A12 회귀 없음 | 충족 | E1(78 바이너리 468 passed · 0 failed) · E2 · E3(node 147) · E11 |

- 적대적 평가 2 라운드(상한, Orca Codex 세션 `review` — `codex-peer-review`) — 라운드 1 high 3 · medium 1(`Clear` 비멱등 → `Clear{conv, id}` · `gone_msgs`, 순서 우회 → 대화 줄,
  지운 글 재실행 → 지운 글 uuid + **글 번호 재사용 결함**(새 글이 실행되지 않음) → `messages.id` AUTOINCREMENT 스키마 22, 늦은 이전 버전 → 내 활동 "반영하지 못함"),
  라운드 2 high 1(보내는 중 추월 → `CLEAR_SENDING`). 미조치 없음([eval](eval.md)), 원장 `#1x R2-D1`(barrier 시험 없음 — 수용).
- 사용자 승인(종결 승인 2026-10-20): plan 개정 E5 EXPECT `(1[0-9]|[7-9])`, 스키마 22(1.0.1 이하는 올린 DB 를 열지 않음 — 되돌릴 때 `coworkAgent.db.v21.bak`),
  `CLEAR_SENDING` 막음, 구현자 해석 — [stage1](work/stage1.md) ① 늦게 확인된 이전 버전 상대(라운드 1 로 내 활동 기록으로 바뀜), [stage2](work/stage2.md) ① 세부정보 공유
  "나가기" 도 목록에서 삭제 ② 용어 표 예외("대화방이 목록에서 삭제됩니다." · "내 글만 삭제" 글자 그대로) ③ 기록 삭제 알림 제목 "대화 기록 삭제 — {에이전트}".
- plan 밖 변경: 옛 전제 시험(`share.rs` · `message_tidy.rs` 글 번호 재사용, `paths.rs` 스키마 번호), Pong 리터럴(`context.rs` · `remote_input.rs`), E8 CHECK 보정(plan 개정 기록).

## 산출

- an-core: 스키마 22(`messages` AUTOINCREMENT · `gone_msgs`), `chat` `clear_conversation` · `busy` · `empty` · `gone` · `mark_gone` · `waiting_requests`, `outbox::CLEAR` · `sending`,
  `run_opts` 기본 테마 `light` · `ui_tile_size`.
- an-net: `Request::Clear{conv, id}` · `Pong.clear` · `knows_clear`.
- an-agent: `open_session` · 빈 대화 첫 글 자동 왕복, `clear_conversation` · `inbound_clear` · `share_head`, 받기 경로 `gone` 확인, `deliver` · `note_sync_failed` clear.
- an-app: `POST /api/conversations` 빈 본문 → 201 · `POST …/clear {all}`, 설정 `notify_sound` · `ui_tile_size`, 받은 요청 알림 글 · 기록 삭제 알림.
- 웹UI: 세션 생성 화면 · ⋯ 하위 메뉴 · `convEndForm` · 대화 줄 손잡이(`convGrab`) · 작은 타일 · 라이트 기본 · `requestNotice` · 효과음(`chime`).
- 시험: Rust `session_open.rs` 4 · `conv_clear.rs` 13 · `ux_api` 새 6 · `schema22_messages_autoincrement`, node 147, `ux_literals` 표.
- 문서: masterPlan D40 개정 · D26 · D35 · S4 · D27 · 후보 줄 · 개정 기록 · 종결 목록, PRODUCT, README, web-ui · an_core · an_net · an_agent · an_app · 개발자 가이드(7.2 · 8.12) ·
  사용자 가이드, 원장 7행.

## 남은 것

- 19 인수 TC: 1x E12(두 PC — 받은 요청 OS 알림 모양 · 효과음 · 누르면 요청 보드, 사이드바 같은 절 줄 옮기기, 모든 사용자 글 삭제 양쪽 · 관람자).
- 클립보드 이미지 붙여넣기 — masterPlan 후보 줄(별도 Task).
- 지운 글 uuid(`gone_msgs`)는 대화를 지울 때만 지운다(보관 기한 없음 — 장기 데이터 관리 검토 대상, 다음 Task).

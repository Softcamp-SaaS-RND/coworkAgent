# report — 15.ux-refresh

종결: 2026-10-12 · 브랜치 `task/15-ux-refresh` → main

## 결과

사용자 개선 요청을 고른 시안 하나(A Slate Pro — 다크 기본 · 라이트 선택)로 다시 입힌 웹UI 와 함께 반영했다. 초대는 '초대 생성' → '초대 수락'
두 단계로 줄었고(생성 측 승인 생략 — D8 개정, 위험 수용), 연결 · 에이전트 목록은 그룹 아코디언 · 필터 · 검색 · 편집을 갖는다. `claude -p` 는
`--model` · `--effort`(전역 기본 `sonnet` · `high` + 상대별 덮어쓰기)로 돈다. 메시지는 markdown · HTML(sandbox) 뷰어로 보이고, '나' · 상대 타일은
사이드바에 그 대화 목록과 받은 글 · 주고받은 글 목록을 연다. 새 글은 모두 대화 에이전트에게 실행 요청이다(D23 개정 — CLI `say` 삭제).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 시안 5개 · 사용자 선택 기록 | 충족 | E4(수동 — 시안 A, intent 라운드 3) |
| A2 공통 토큰 · 키/값 · 편집/읽기 구분 | 충족(토큰) · 실화면 ABANDON | E5 `root=1 violations=0` · E11(D28) |
| A3 설정 트리뷰 | 충족 · 실화면 ABANDON | E3 `settings_tree` · E11 |
| A4 markdown · html 뷰어 | 충족 · 실화면 ABANDON | E3 `markdown_blocks_safe` · `html_doc_detect` · E11 |
| A5 favicon | 충족 | E7 `favicon_link_present` · E8 `favicon_served` |
| A6 초대 생성 측 자동 등록 · 알림 · 바로 삭제 | 충족 · 실PC ABANDON | E6 auto_pair 2건 · E8 직접 추가 2건 · E11 |
| A7 초대 수락 안내 문구 | 충족 | E7 `invite_labels_without_messenger` |
| A8 연결 목록 아코디언 · 그룹 필터 · 도구 편집 | 충족 · 실화면 ABANDON | E3 `group_filter_search` · E8 `peer_patch_group_model_effort` · E10 · E11 |
| A9 `--model` · `--effort` 기본 · 상대별 | 충족 | E8 `settings_claude_defaults` · `settings_read_error_is_500` · E9 run_options 3건 · E10 |
| A10 내 에이전트 목록 · 편집 · 가이드 | 충족 · 실화면 ABANDON | E8 `agent_patch_edits_but_not_name` · E10 · E11 |
| A11 알림 화면 크기 · 상한 라벨 | 충족(라벨) · 실화면 ABANDON | E7 `turn_label_literal` · E11 |
| A12 '나' 받은 글 · 상대 주고받은 글 · 보낸 메시지 탭 | 충족 · 실화면 ABANDON | E3 `inbox_rows` · `peer_messages` · E8(받은 글 · 보낸 글 · 종류 거르기) · E11 |
| A13 테마 다크(기본) · 라이트 저장 | 충족 · 실화면 ABANDON | E8 `theme_setting_persists` · E11 |
| A14 새 글은 모두 에이전트에게 | 충족 | E8 `to_human_rejected` · `new_conversation_to_human_rejected` · E12 cli 15건 |
| A15 보냄 · 받음 태그 | 충족 · 실화면 ABANDON | E3 `direction_tag` · E11 |
| A16 대화 제목(이 PC 에만) | 충족 · 실화면 ABANDON | E7 `title_search_label` · E8 `conversation_title_set_and_edit` · E10 · E11 |
| A17 타일 표시 이름 · 아이콘(이 PC 에만) | 충족 · 실화면 ABANDON | E3 `tile_text` · E8 `peer_tile_alias_icon` · `settings_me_tile_side_width` · E10 · E11 |
| A18 트레이 메뉴 "열기" · "종료" | 충족(literal) · 실화면 ABANDON | E7 `tray_menu_open_quit_only` · E11 |

- 적대적 평가 2 라운드(상한) — 라운드 1: high 2(직접 추가가 페어링되지 않음 — 사실 · 고침, `--effort` 최소 버전 — 오탐 · 공식 CHANGELOG 근거),
  라운드 2: high 2 · medium 2(새 대화 `to` 검증 · 실행 중 폴더 변경 세션 · 받은 글 종류 거르기 제한 전 · 설정 DB 읽기 오류 노출 — 모두 사실 · 고침).
  미조치 2건(E11 · 재개 실패 세션 정리)은 [eval.md](eval.md) → 원장.
- stage 리뷰 3회(분리된 리뷰 에이전트) — HTML 주입 경로 0 · markdown 정화 · 초대 자동 승인 경쟁(Hello 응답 `approved`, 개정 3) 등 반영.
- 사용자 확인 라운드 6회(intent 라운드 1 ~ 6) — 시안 결정 · 빌드 확인 뒤 수정 지시 2회(라운드 4 · 5) 포함.
- plan 개정 7건: 1 · 3 설계 문구, 2 · 4 · 5 · 6 · 7 평가 항목(사용자 승인 — E3 21 · E7 5 · E8 16 · E9 3 · E10 12 · E12 신설).

## 산출

- `an-core`: 스키마 9(`peers.grp` · `model` · `effort` · `agents.grp`) · 10(`conversations.title` · `peers.alias` · `icon`) · `run_opts`(model ·
  effort · 그룹 · 별칭 · 제목 · 아이콘 · 테마 · 사이드바 폭 검증과 기본값 — 설정 읽기 오류는 그대로 낸다) · `peers::set_meta` · `set_tile` ·
  `agents::update`(폴더가 바뀌면 그 에이전트 대화의 세션을 지움) · `chat::set_title` · `messages_across`(종류 · 상대 · 본문 거르기 뒤 제한).
- `an-net`: 초대 생성 측 자동 등록(처음 보는 키 + 코드 일치 → 내 승인 · Hello 응답 `approved`) · `PeerEvent::AutoPaired` ·
  `Node::set_peer_meta` · `set_peer_tile` · 초대 텍스트 "초대 수락".
- `an-agent`: `RunInput.model` · `effort` · `claude_args(…, model, effort)` · 실행 직전 상대별 → 전역 → 기본값(못 읽으면 실행 실패) · 실행 도중
  폴더가 바뀌면 세션을 남기지 않음 · `default_relay_cap` 오류 전파.
- `coworkagent-app`: `PATCH /api/peers/{id}`(그룹 · model · effort · 별칭 · 아이콘) · `PATCH /api/agents/{id}` · `PATCH /api/conversations/{id}`
  (제목) · `GET /api/messages`(`side` · `peer` · `kind` · `q` · `limit`) · 설정 `claude_model` · `claude_effort` · `ui_theme` · `ui_side_width` ·
  `me_alias` · `me_icon`(DB 읽기 오류 500) · `GET /favicon.ico` · 새 대화 · 이어 쓰기 `to` 는 `agent` 만 · 직접 추가도 양쪽 페어링 · CLI `say`
  삭제 · 트레이 "열기" · "종료" · 웹UI 전면(시안 A 토큰 · 다크/라이트 · 타일 → 사이드바 대화 목록 · 받은 글 · 주고받은 글 · 보낸 메시지 탭 ·
  뷰어 · 원문 보기 · 설정 트리 · 사이드바 폭).

## 남은 것

- 실화면 · 실PC 확인(E11) — `19.prototype-acceptance` TC-K(D28). 사용자 일괄 검증은 Task 18 뒤(사용자 지시 2026-10-12).
- 재개(`--resume`) 실패 시 세션 정리 — 원장 #15 R2-F2r(측정: 없는 세션이면 종료 1 · JSON 없음 → 같은 대화가 계속 실패).
- 화면 배치 리뉴얼(5영역 · 내 활동 · 실행 상세) — 17.workspace-renewal. 집중 · @멘션 · 질의/요청 — 18.focus-mention.

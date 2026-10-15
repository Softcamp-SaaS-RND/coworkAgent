# report — 1i.rail-group

종결: 2026-10-15 · 브랜치 `task/1i-rail-group` → main

## 결과

레일에 **그룹**을 더했다 — 프로젝트마다 여러 상대와의 대화를 묶어 보는, 이 PC 에만 있는 묶음(한 대화는 여러 그룹에, 상대에게 보이지 않고 글은 지금처럼
1:1). 레일 상대 타일 아래 구분선 뒤 그룹 타일 · "그룹 +"(이름 1 ~ 40자 · 같은 이름 금지). 그룹 타일 = 메뉴 4개(대화(그룹 요약 — 대화 있는 상대만) · 요청 ·
저장된 항목 · 주고받은 글), 사이드바 "대화 추가" · ★(그룹 안) · 상대별 절. 넣고 빼기 = 대화 ⋯ "그룹" 체크 · "대화 추가"(검색 + 체크) · 대화를 그룹 타일에 끌어
놓기 · "그룹에서 빼기", 그룹 화면에서 만든 새 대화는 자동 소속. 레일 타일마다 **타일 메뉴**(우클릭 · 메뉴 키 · 사이드 머리 ⋯) — 타일 편집(이름 · 그림) ·
이미지 지우기 · 그룹 삭제(대화는 남김). 순서는 정렬 기능 없이 **손으로 놓은 자리** — 상대 타일 · 그룹 타일 · 같은 절 대화를 끌어 놓거나 Alt + ↑ / ↓, 대화는
대화마다 순서 하나(어느 화면에서 옮겨도 같은 앞뒤), 새 대화는 맨 위 · 새 글이 와도 자리 그대로. 타일 색은 상대 id 기준.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 "그룹 +" · 이름 바꾸기 · 삭제(대화 남음) | 충족 | E3 `create_name_rules` · `rename_and_icon` · `delete_keeps_conversations` · E6 · E7 |
| A2 여러 그룹 소속 · 대화 ⋯ 체크 | 충족 | E3 `membership_many_groups` · E7(⋯ 체크) |
| A3 "대화 추가" 검색 + 체크 · "그룹에서 빼기" | 충족 | E5 `membership_diff` · E6 · E7 |
| A4 이 PC 에만 · 다시 켜도 남음 | 충족 | E3 `local_only_and_survives_reopen`(보관 줄 0) · E4 · E7 단계 b |
| A5 타일 메뉴 — 이름 · 이미지 · 지우기 · 그룹 삭제 | 충족 | E6 · E7(우클릭 · Shift+F10 · ⋯ · 그림 넣고 지움) |
| A6 그룹 메뉴 4 · 그 그룹 것만 · 상대별 절 · 새 대화 소속 | 충족 | E5 `side_nav_group` · `group_summary` · `group_sections` · E3 `lists_by_group` · `requests_by_group` · `new_conversation_top_and_stays` · E7 |
| A7 이름 1 ~ 40자 · 같은 이름 금지 · 문구 | 충족 | E3 · E5 `group_name_error`(그림 문자 포함) · E7 |
| A8 끌어 놓기 · Alt + ↑ / ↓ · 묶음 경계 · 다시 켜도 남음 | 충족 | E5 `move_in_order` · `step_target` · `drop_kind` · E7(그룹 · 대화) · mock `s3`(상대 타일 · 묶음 사이 놓기 없음) |
| A9 대화마다 순서 하나 · 새 글에 자리 그대로 · 새 대화 맨 위 · 상대 타일 = 절 순서 | 충족 | E3 `new_conversation_top_and_stays` · `order_*` · E4 · E5 `order_one_for_all` · `order_to_apply` |
| A10 대화를 그룹 타일에 놓아 넣기 | 충족 | E5 `drop_kind` · E7 |
| A11 지운 상대의 대화도 그룹에 남음 | 충족 | E3 `removed_peer_conversation_stays_in_group` · E5 `group_summary` |
| A12 검증 명령 | 충족 | E1 · E2 · `check_ddd_docs` · `check_gates` 7/7 |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1: high 2(그룹 삭제 경쟁에서 그룹 없이 전송 · 소속 실패를 성공 문구로) · medium 3(순서 저장
  겹침 · HTML `maxlength` 가 UTF-16 단위 · 상자 닫힘 초점) — 모두 사실 · 고침. 라운드 2: medium 3(순서 id 1만 개 상한 · 바깥 누름 초점 · 빼기의 없는 대화가
  200) — 모두 사실 · 고침, 재리뷰 없음(상한). 미조치 지적 없음. 여러 창의 마지막 저장 우선은 plan 해석 16 대로 수용(ledger `#1i R1-M1`).
- 실제 앱 확인에서 기존 결함 1건을 찾아 고쳤다 — 타일 아이콘 그림을 객체 URL(`blob:`)로 읽어 CSP `img-src` 에 막혔다(#17 부터, mock 은 CSP 가 없어
  통과). data URL 로 읽는다.
- 이 PC 반영: 릴리스 빌드(`aaea9bf`)로 앱 다시 띄움, 스키마 15 → 16(사본 `.v15.bak` · 실행 전 `coworkAgent.db.pre-1i.bak`). 화면 확인 27/27(비파괴 —
  시험 그룹은 지우고 대화 순서는 원래대로).
- masterPlan: **D37 추가**(레일 그룹) · **D35 개정**(손으로 놓는 자리 · 타일 색 · 타일 메뉴) · S4 보강 · 오버플로 1i(1h 앞).

## 산출

- `an-core`: 스키마 16(`peers.pos` · `conversations.pos` · `groups` · `group_convs`) · `groups` 모듈 · `chat::Scope` · `Summary.pos` · 새 대화 맨 위 · 새
  상대 맨 뒤 · `peers::list` 자리 순. `an-agent`: `Compose.group`(없는 그룹이면 트랜잭션을 버림). `coworkagent-app`: `groups_api`(그룹 · 소속 · 순서) ·
  목록 `group` 거르기 · 새 대화 `group`.
- 웹UI: 그룹 타일 · "그룹 +" · 그룹 사이드바 · 그룹 요약 · 떠 있는 상자(`#float`) · 대화 ⋯ 메뉴 · "대화 추가" · 타일 메뉴 · 끌어 놓기 · Alt + ↑ / ↓ ·
  순서 저장 직렬화, `view.js` 새 함수 15개.
- 시험: `groups_api.rs`(13) · `store.rs` `schema16_groups_and_order` · `reorder_more_than_ten_thousand` · `view.test.mjs`(57 → 76) · `ux_literals.rs`
  `rail_group_literals`.
- 문서: README · user-guide §4(4.1 그룹 · 4.2 타일 메뉴 · 4.3 자리 옮기기) · developer-guide(스키마 16) · an_app · an_core · PRODUCT · masterPlan · ledger.

## 남은 것

- README · 사용자 가이드 화면 캡처는 그룹 이전 레일이다(갱신 여부 사용자 결정).
- 19 인수 TC — 상대 둘 이상에서 상대 타일 끌기 · 상대별 절 순서, 그룹 여러 개 · 대화 많은 목록에서 끌기 · 화면 읽기 프로그램 알림.
- GitHub 반영(push)은 사용자 확인 뒤.

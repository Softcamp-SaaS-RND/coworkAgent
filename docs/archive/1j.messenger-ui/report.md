# report — 1j.messenger-ui

종결: 2026-10-15 · 브랜치 `task/1j-messenger-ui` → main

## 결과

화면 개편 — 디자인 캔버스 3 라운드로 정한 메신저형 화면. **대화**: 말풍선(이 PC 쪽 오른쪽 · 상대 쪽 왼쪽 아바타 · 이름) · 날짜 구분선 · 방향 라벨
("사람 → 에이전트" · 보냄 / 받음) 없앰 · 머리 "대화 보기 / 원문 보기" · "북마크 n" · "세부정보"("보낸 메시지" 탭 없앰) · 말풍선 우상단 북마크 · 복사 아이콘 ·
최대 폭 90%(사용자 지시). **대화방 줄**: ☆ · 아바타 · 제목 · 마지막 글 미리보기 · 시각 · 안 읽음. **메뉴**: 즐겨찾기 화면 · 상대 타일 첫 화면 = 그 상대 개요 ·
주고받은 글은 '나' 에만. **상단 바**: "받기 가능" 칩 없앰 · 프로필 아이콘 상태 색(받기 가능 초록 · 집중 주황 · 앱과 연결 끊김 붉음) · 프로필 메뉴 상태 고르기 ·
이름 한 번. **레일**: A안 사람 / 사람 + 그림자 모양 + 아래 이름(72px) · 다시 그려도 초점 유지(누른 뒤 Alt + ↑ / ↓) · 타일 메뉴 위로 · 아래로 옮기기.
**상대 단위 그룹**: 그룹에 상대를 넣으면 그 상대의 대화 전부(뒤에 생긴 것도) — 스키마 18 `group_peers` · 옛 자유 입력 그룹 이름표는 같은 이름 레일 그룹으로 ·
타일 메뉴 그룹 체크 · 연결 목록 그룹 절(끌어 옮기기 · "그룹 ▾" · 추가 · 이름 바꾸기 · 삭제). **설정**: "페어링" 카드 · 초대 두 단추 같은 모양 · 직접 추가 ·
읽기 전용 · 에이전트 그룹 칸 없앰.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 말풍선 · 날짜 · 방향 라벨 없음 | 충족 | E3 `bubble_side` · `day_label · day_breaks` · `inbox_rows` · `activity_rows` · E4 `messenger_view_literals` · E10(mine 2 / theirs 2 · 구분선 2) |
| A2 머리 대화 보기 · 원문 보기 · 북마크 n · 세부정보 | 충족 | E4 · E10(머리 4 단추 · 원문 보기 · `#s/` → `#c/`) |
| A3 말풍선 북마크 · 복사 아이콘 | 충족 | E3 `message_actions_no_bookmark` · E10(넣고 빼기 → 머리 수 ""→1→"") |
| A4 대화방 줄 ☆ · 보냄 / 받음 없음 | 충족 | E3 `conv_sections` · `room_preview` · `room_preview_cli` · E10(☆ 넣고 빼기) |
| A5 메뉴 세 범위 · 즐겨찾기 화면 | 충족 | E3 `side_nav` · `side_nav_starred` · `side_nav_exchanged_me_only` · `starred_sections` · E10('나' 7 · 상대 4 · 상대 개요) |
| A6 칩 없음 · 상태 색 · 이름 한 번 | 충족(끊김 붉음은 mock) | E3 `presence_of` · `profile_menu_radio` · E4 · E10(`st-online` · radio 2) · stage 2 mock(요청을 막자 `st-offline`) |
| A7 레일 A안 | 충족 | E4 `menu_status_rail_literals` · E10(72px · 모양 4 · 이름 11px) |
| A8 상대 그룹 · 연결 목록 · 레일 반영 | 충족 | E5 `groups_api` 17 · E6 `schema18_*` · E3 `peer_sections` · `peer_move*` · `in_group_peers` · E10(새 그룹에 넣기 · 레일 반영 · 그룹 ▾ 빼기 · 삭제) |
| A9 누른 뒤 Alt + ↑ / ↓ · 타일 메뉴 옮기기 | 충족 | E10(누른 뒤 초점 = 타일 · Alt + ↓ 뒤 유지 — 이 PC 상대 1 이라 자리 바뀜은 mock: 2,3,4 → 3,2,4 · 메뉴 위로 · 아래로) |
| A10 설정 정리 | 충족 | E4 `peer_group_settings_literals` · E10(페어링 · 같은 class · 읽기 전용 · 직접 추가 · 에이전트 그룹 칸 없음) |
| A11 검증 명령 | 충족 | E1 · E2 · E3 · E7 `check_ddd_docs` · E8 `check_readme` · E9 `check_crates` 모두 gate(원장 `#1h R2-F2` 닫힘) · `check_gates` 10/10 |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1: medium 2(대화방 미리보기가 로컬 `id` 순 · 상대 그룹 옮기기가 두 요청) — 공통 순서 `(lc, uuid)` ·
  `PUT /api/peers/{id}/groups` 한 트랜잭션. 라운드 2: medium 2(요약 마지막 상태 · 시각이 다른 글 · 미리보기 CLI 출처) — 같은 글 · `last.via`. 미조치 없음,
  재리뷰 없음(상한 — 라운드 2 수정분은 리뷰되지 않음).
- plan 편차(work 문서 · plan 개정 기록): `last` 한 객체 · 미리보기 120자 · 보기 전환 `role=group` · 말풍선 90%(사용자 지시) · 끊김 판정 = `api()` fetch 실패 2번 ·
  메뉴 "즐겨찾기 n" · 수락 단추 "초대 수락" · 페어링 안내에서 "메신저로" 뺌 · 이미지 지우기 안내 "사람 모양".
- 이 PC 반영: 릴리스 빌드(라운드 1 이전 코드)로 다시 띄움, 스키마 17 → 18(사본 `.v17.bak`) · 화면 27/27(비파괴). 종결 뒤 최종 빌드로 다시 띄운다.
- masterPlan: D35 개정(메뉴 · 상대 첫 화면 · 레일 A안) · D37 개정(상대 단위 그룹 — 1i 기각안 뒤집음) · S4 보강 · 개정 기록.

## 산출

- `an-core`: 스키마 18(`group_peers` · `peers.grp` 옮기기) · `groups::{fit_name, unknown_peers, change_peers, change_peer_groups}` · `Group.peers` ·
  `chat::{LastMessage, PREVIEW_CHARS}` · `Summary.last` · `Scope::Group` 상대 포함. `coworkagent-app`: `PUT /api/groups/{id}/peers` · `PUT /api/peers/{id}/groups` ·
  그룹 JSON `peers` · 대화 JSON `last`.
- 웹UI: 말풍선 · 날짜 구분 · 머리 · 아이콘 · 대화방 줄 · 즐겨찾기 화면 · 상대 개요 · 프로필 상태 · 레일 A안 · 초점 유지 · 타일 메뉴 · 연결 목록 그룹 절 · 설정 정리.
- 시험: `view.test.mjs` 82 → 96 · `ux_literals.rs` 14 → 17 · `groups_api.rs` 13 → 17 · `store.rs` `schema18_*` · `summary_last_follows_canonical_order` ·
  `paths.rs` 18 · `chat_api.rs` `last`.
- 문서: README · 사용자 · 개발자 가이드 · an_core · an_app · PRODUCT · masterPlan · 원장 · 캡처 5장(가상 데이터).

## 남은 것

- 19 인수 TC — 여러 상대에서 Alt + ↑ / ↓ 자리 바꿈 · 앱과 연결 끊김(붉음)을 실제 앱으로 · 옛 자유 입력 그룹이 있는 DB 의 스키마 18 옮기기 · 연결 목록 끌어 옮기기.
- 캡처 중 `activity` · `search` · `relay` · `new-conversation` · `comm-*` 은 1j 이전 레일 · 상단 바 모습(내용은 맞음) — 다음 캡처 갱신 때.
- 통계 메뉴(별도 Task — 보여줄 숫자 미정).
- `graphify --update`(PROJECT.md 종결 절차) — 이번에도 하지 않았다(그래프 2026-10-13 기준).
- GitHub 반영(push) · 릴리스(1h · 1j 함께)는 사용자 확인 뒤.

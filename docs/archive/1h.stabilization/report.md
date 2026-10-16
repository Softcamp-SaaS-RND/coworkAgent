# report — 1h.stabilization

종결: 2026-10-15 · 브랜치 `task/1h-stabilization` → main

## 결과

19 인수 전 안정화 — 사용자가 0.3.0 에서 본 화면 문제 5건과 앞 Task 들이 남긴 결함 3건을 고쳤다. **화면**: 레일 타일 글자는 표시 이름 앞부분을 폭에 맞춰
(넓이 예산 4 — 영문 · 숫자 4자, 한글 · 한자 · 그림 문자 · 국기 · 키캡 2자, 세 글자 이상은 작은 글씨) · 그룹 타일은 겹친 카드 모양 · 사이드바 메뉴는 세
절과 구분선(상단 **개요**("대화" 이름 바꿈) / 중단 내 활동 · 요청 · 주고받은 글 / 하단 **북마크 항목** · 에이전트) · "저장된 항목" → "북마크 항목", 메시지
작업 "북마크" / "북마크 해제"(★ = 대화 하나, 북마크 = 글 하나 — 중복 아님) · 우상단 프로필 단추 = 아이콘 + 표시 이름 알약(12em 말줄임 · 툴팁 전체).
**결함**: 트레이 아이콘을 못 만들면 1 · 2 · 4 · 8 · 15 · 30초 뒤 다시(그동안 온 새 요청 수도 보관) · 이어 쓰던 Claude 세션이 사라졌으면 새 세션으로 한 번
다시 실행해 답하고 그 뒤로 새 세션을 잇는다 · 상대 번호를 다시 쓰지 않는다(스키마 17 `peers.id` AUTOINCREMENT, 순번 시작 = 남은 대화 · 보낼 글 · 접힘
열쇠의 번호까지). 사용자 가이드에 방화벽 프로필(Public) 확인 · 끄는 방법.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 타일 글자 넓이 예산 4 · 타일 안 | 충족 | E3 `tile_text_latin` · `tile_text_wide` · `tile_text_budget` · `tile_text_keycap` · E9(넓이 [2, 4, 3, 2] · 타일 안) |
| A2 그룹 겹친 카드 | 충족 | E4 `stabilization_literals` · E9(그룹 2 · 상대 타일 카드 없음) |
| A3 메뉴 세 절 · 개요 | 충족 | E3 `side_nav` · `side_nav_group` · `side_nav_sections` · E4 · E9('나' · 상대 · 그룹) |
| A4 북마크(편집 · 설정 "저장" 그대로) | 충족 | E3 `message_actions` · `message_actions_bookmark` · E4("저장된 항목" 0 · "고친 본문 저장" 남음) · E9 |
| A5 프로필 알약 | 충족 | E4 · mock(17자 이름 208px · 말줄임 · 툴팁) · E9 |
| A6 트레이 다시 시도 | 충족(간격) · 실제 로그인 직후는 19 TC | E7 `retry_after_schedule` · 라운드 1 attention 보관 |
| A7 사라진 세션 → 새 세션 | 충족(대역 실행기 · 실측 출력 판별) · 실제 claude 는 19 TC | E6 `resume_missing` 5건(2.1.299 결과 줄 · 2.1.296 stderr · 다른 실패는 다시 실행 안 함) |
| A8 상대 번호 다시 쓰지 않음(올린 DB 포함) | 충족 — 해석 확인 대기 | E5 `schema17_*`(지운 번호 · 남은 대화 · 보낼 글 · 접힘 열쇠 · 다시 열어도) · E9(이 PC 스키마 17 · 번호 그대로). 스키마 16 이하에서 흔적 없이 지운 번호는 되살릴 수 없다(원장 `#1h R1-M1`) |
| A9 방화벽 안내 | 충족 | E8 `check=2 off=1` |
| A10 검증 명령 | 충족 — gate 편입 결정 대기 | E1 · E2 · E3 + 종결 측정 `check_ddd_docs` PASS · `check_readme` ok · `check_crates` violations=0 · `check_gates` 9/9(원장 `#1h R2-F2`) |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1: high 1(A10 gate 가 PROJECT.md 명령 일부만) · medium 3(올린 DB 의 흔적 없는 지운 번호 ·
  트레이 다시 시도 중 새 요청 표시 유실 · 키캡 넓이 1) — 트레이 · 키캡 고침, A10 은 전부 실행 기록, 지운 번호는 실해 없음. 라운드 2: high 2(A8 해석 ·
  A10 gate) · medium 1(STATE 재개 좌표) — 접힘 열쇠 번호를 순번에 넣음 · STATE 갱신, A8 해석 · A10 gate 편입은 **사용자 결정 대기**(eval "미조치 지적",
  원장 `#1h R1-M1` · `#1h R2-F2`). 재리뷰 없음(상한).
- 원장 닫힘: `#15 R2-F2r`(사라진 세션) · `#17 S1-P`(상대 번호 재사용). 1e 남은 것(로그인 직후 트레이 생성 실패)은 다시 시도로 대신했다.
- 이 PC 반영: 릴리스 빌드(라운드 1 이전 `50f1d63` 코드)로 앱 다시 띄움, 스키마 16 → 17(사본 `.v16.bak`). 화면 확인 16/16(비파괴). 종결 뒤 최종 빌드로 다시
  띄운다(이 PC 접힘 열쇠는 `me` · 상대 1 뿐이라 순번은 그대로).
- masterPlan: **D35 개정**(메뉴 세 절 · 개요 · 북마크 · 타일 글자 · 그룹 모양 · 프로필) · 통계 메뉴 별도 Task 후보 · 개정 기록.

## 산출

- `an-core`: 스키마 17(`peers` AUTOINCREMENT 다시 만들기 · 순번 시작). `an-agent`: `runner::SessionMissing` · `session_gone` · `session_missing` · 새 세션
  한 번 다시 실행. `coworkagent-app`: `tray::retry_after` · 다시 시도 · 새 요청 수 보관.
- 웹UI: `view.tileText(name, budget)` · `sideNav` 세 절 · 북마크 글자 · `tileFace` 긴 글자 · 프로필 알약 · 구분선 · 겹친 카드.
- 시험: `view.test.mjs`(76 → 83) · `ux_literals.rs` `stabilization_literals` · `store.rs` `schema17_*` 2 · `resume_missing.rs`(5) · `tray.rs`
  `retry_after_schedule` · `paths.rs` 스키마 번호.
- 문서: README · user-guide(3 · 4 · 7.4 · 7.5 · 17장) · developer-guide · an_core · an_agent · an_app · PRODUCT · masterPlan · ledger · 캡처 3장(가상 데이터).

## 남은 것

- 사용자 결정: A8 해석(흔적 없는 지운 번호) · A10 gate 편입(다음 Task plan 틀).
- 19 인수 TC — 로그인 직후 트레이(다시 시도 로그 "트레이 아이콘을 만들 수 없다(n번째)") · 세션 파일을 지운 대화에 질문(새 세션으로 답) · 상대 지우고
  다시 이은 뒤 새 번호 · 여러 상대의 타일 글자.
- 통계 메뉴(별도 Task — 보여줄 숫자 미정).
- `graphify --update`(PROJECT.md 종결 절차) — 1i 와 같이 이번에도 하지 않았다(그래프 2026-10-13 기준).
- GitHub 반영(push) · 릴리스는 사용자 확인 뒤.

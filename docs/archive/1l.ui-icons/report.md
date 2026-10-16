# report — 1l.ui-icons

종결: 2026-10-16 · 브랜치 `task/1l-ui-icons` → main

## 결과

사이드바 메뉴 · 화면 머리 제목 · 설정 트리 · 대화 · 타일 ⋯ 메뉴 · "+ 새로 만들기" 이름 앞에 뜻에 맞는 선 아이콘을 붙였다(`app.css` `.ico` 한 벌, 16px 하나 —
기존 15px 단추 아이콘도 맞춤). 말풍선은 내 글 = 강조색, 상대 글 = 그 상대 타일 색 연한 바탕 + 띠. 설정은 한 열 · 카드 안내는 제목 옆 '?' · 내 정보는 프로필
"사용자 정보" 팝업 · 내 에이전트 검색 없앰 · "에이전트 등록" 이 등록 칸을 보이고 숨김 · "추가 지시" 사용 예는 '?'. 레일 타일 순서를 설정에서 고른다(사용자 →
그룹 기본 · 그룹 → 사용자). "+ 새로 만들기" 에서 "자동 왕복 대화" 를 뺐다(새 대화 화면 안에서 그대로).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 메뉴 · 제목 · 트리 · ⋯ 아이콘 | 충족 | E3 `menu_icons` · `icons_defined` · E4 · E8(모두 16×16, 그룹 타일 ⋯ 포함) |
| A2 말풍선 색 | 충족 | E4 · E8(내 글 `--accent` · 상대 글 타일 색 층 0.12 · 띠 3px) · mock 다크 · 라이트. macOS 12 Safari 15 실기는 확인 필요(원장 `#1l R2-SAF`) |
| A3 설정 '?' 도움말 | 충족 | E4 · E8(늘 보이는 안내 0 · 올리면 뜸) · mock(초점이면 뜸) |
| A4 내 정보 → 사용자 정보 팝업 | 충족 | E3 `settings_tree` · `profile_menu` · E4 · E8(팝업 · Esc) |
| A5 한 열 · 검색 없음 · 등록 칸 · 추가 지시 '?' | 충족 | E3 · E4 · E8(카드 7 한 열 · 보이기 초점 `agent_name` · 숨기기) |
| A6 검증 명령 | 충족 | E1 · E2 · E3 · E5 · E6 · E7 · `check_gates` 8/8 |
| A7 타일 순서 | 충족 | E1 `rail_order_setting_persists` · E8(레일 · Tab 순서 그룹 먼저 · 새로 고침 유지 · 되돌림) |
| A8 새로 만들기 자동 왕복 없음 · 아이콘 | 충족 | E3 `new_menu` · E4 · E8 |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1: high 1(그룹 타일 ⋯ "그룹 삭제" 호출이 옛 인자라 메뉴가 예외로 안 열림) — 인자 · 휴지통 아이콘 ·
  node "`item(` 첫 인자 = 아이콘" 검사 · E8 그룹 타일 ⋯. 라운드 2: medium 1(`color-mix()` 가 macOS 12 Safari 15 에 없음) — 타일 색 층(`::before` 불투명도)으로.
  상한 도달 — 재리뷰 없음, 미조치 1(Safari 15 실기 확인 — 원장 조건 대기, [eval](eval.md)).
- plan 편차(stage1): 아이콘 크기 일관성(승인 때 추가) — 텍스트와 함께 쓰는 아이콘만 16px 로, 아이콘만 있는 단추 · 레일 타일 그림은 그대로. '?' 는 `:focus` 로
  (Esc 닫기 없음). 페어링 '?' 에 카드 안내 + 초대 생성 안내 합침.
- 이 PC 반영: 릴리스 빌드(라운드 2 수정 포함)로 화면 확인 15/15(비파괴 — 타일 순서는 바꿨다가 되돌림). 종결 뒤 main 최종 빌드로 다시 띄운다.
- 서버: 설정 열쇠 `ui_rail_order` 하나(스키마 그대로 · 틀린 값 400).
- masterPlan: D35 개정 · S4 보강 · 개정 기록. 원장 1행.

## 산출

- 웹UI: `ico` · `.ico` 22 + trash · `sideNav` · `settingsTree` · `newMenu` `icon` · `profileMenu` "사용자 정보" · `me_dialog` · `showAgentNew` · `applyRailOrder` ·
  `data-tone` 말풍선 · `.help` · `.tip` · 설정 한 열.
- 서버: `run_opts::RAIL_ORDER_KEY` · `RAIL_ORDERS` · `rail_order` · `SettingsPatch.ui_rail_order`.
- 시험: `view.test.mjs` 98 → 100 · `ux_literals.rs` 18 → 19 · `ux_api` `rail_order_setting_persists`.
- 문서: 사용자 가이드 · an_app · developer-guide · PRODUCT · README · masterPlan · 캡처 4장(home · workspace · pairing · agents, 가상 데이터).

## 남은 것

- 19 인수 TC — macOS 12 Safari 15 에서 상대 말풍선 색(원장 `#1l R2-SAF`) · 타일 순서 그룹 → 사용자에서 끌어 옮기기 · '?' 도움말 키보드.
- 디자인 캔버스(1j)는 1k · 1l 화면과 다르다 — 다음 시안 작업 때 갱신.
- GitHub 반영(push) · 릴리스(1h · 1j · 1k · 1l 함께)는 사용자 확인 뒤.

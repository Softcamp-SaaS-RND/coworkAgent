# report — 1g.rail-scope

종결: 2026-10-14 · 브랜치 `task/1g-rail-scope` → main

## 결과

레일 타일마다 겹치던 메뉴를 정리했다. **'나'** = 모든 상대를 상대별로 묶어 보는 자리 — 메뉴 6개(대화 · 내 활동 · 요청 · 저장된 항목 · 주고받은 글 ·
에이전트), 각 목록은 상대별 접이식 절(제목 = 상대 이름 · 건수, 레일 순서, 글 없는 상대 숨김), '대화' 는 메인에 상대별 요약(대화 수 · 안 읽음 · 대기
요청 · 최근 활동 + 그 상대 대화), 사이드바 대화 목록은 ★ 즐겨찾기 아래 상대별 절. 접은 절은 이 PC 설정 `ui_folded` 에 남아 다시 열어도 접혀 있다.
**상대 타일** = 그 상대만 — 메뉴 4개(요청 · 저장된 항목 · 주고받은 글 · 에이전트), 첫 화면 그 상대 주고받은 글, '대화' 메뉴 · 내 활동 없음(상대 타일인
채 `#` 는 그 상대 주고받은 글로, 내 활동은 '나' 로). 사이드바 '내 에이전트' · '{상대}의 에이전트' 절은 없앴다(에이전트 화면만).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 '나' 메뉴 6개 | 충족 | E3 `side_nav` · E4 `rail_scope_literals` · E5 실측 |
| A2 상대 타일 메뉴 4개 · 첫 화면 주고받은 글 · 그 상대만 | 충족 | E3 `side_nav` · E5 실측(첫 화면 `#p/{id}` · 묶음 없음 · 사이드바 그 상대만) |
| A3 상대별 접이식 절 · 내 에이전트 맨 앞 · 접힘 기억 | 충족 | E3 `peer_groups` · `row_peer` · `send_latest` · E6 `settings_folded` · E5 실측(새로 고침 뒤 접힘 유지) |
| A4 '대화' 상대별 요약 · 사이드바 ★ 아래 상대별 절 | 충족(대기 요청 수는 500건 상한 — 미조치) | E3 `peer_summary` · `waiting_by_peer` · `conv_sections_by_peer` · E5 실측 |
| A5 사이드바 에이전트 절 없음 | 충족 | E4 · E5 실측 |
| A6 검증 명령 | 충족 | E1 · E2 · E3(57) · `check_readme` · `check_crates` · `check_ddd_docs` |
| A7 상대 타일인 채 내 활동 → '나' · `#` → 주고받은 글 · '상대' 거르기 칸 없음 | 충족 | E3 `side_nav`(경로) · E4 · E5 실측 |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1: high 1(절 머리 `summary` 에 초점이 남으면 5초 새로 고침이 목록을 멈춤) · medium 1(접힘
  저장 동시 전송 역전) — 둘 다 사실 · 고침(plan 개정 1, `holdsRender` · `SendLatest`). 라운드 2: medium 1(상대별 대기 요청 수 500건 상한) · low 1(여러 창
  접힘 덮어쓰기) — 사실 · 미조치(사용자 결정 대기 — eval "미조치 지적", ledger `#1g R2-F1` · `#1g R1·R2-F2`).
- 화면 사진 검토로 절 클래스가 설정 화면 그룹(`.group`)과 겹치던 것을 찾아 `peer-fold` 로 바꿨다(stage2).
- 이 PC 반영: 릴리스 빌드(`daed561`)로 감시 + 앱 다시 띄움, DB 사본 `coworkAgent.db.pre-1g.bak`. 화면 확인 27/27(비파괴).
- masterPlan: **D35 추가**(레일 범위) · S4 보강 · Phase 1 오버플로 1g.

## 산출

- `an-core`: `run_opts::FOLDED_KEY` · `clean_folded` · `folded`. `coworkagent-app`: 설정 `ui_folded`(`SettingsPatch` · 설정 JSON).
- 웹UI: `view.sideNav` · `route(#p)` · `peerGroups` · `peerSummary` · `waitingByPeer` · `convSections(rows, paired)` · `holdsRender` · `SendLatest`, `app.js`
  `foldEl` · `grouped` · `renderHome` · 주소 바꿈, `index.html` 홈 요약 · 사이드바 에이전트 절 · 내 활동 상대 칸 제거, `app.css` `.peer-fold`.
- 시험: `view.test.mjs`(+7, 갱신 1 → 57) · `ux_api.rs` `settings_folded` · `ux_literals.rs` `rail_scope_literals`.
- 문서: README · user-guide §4 · an_app · developer-guide · PRODUCT · masterPlan.

## 남은 것

- 미조치 2건 사용자 결정: 상대별 대기 요청 정확한 수(서버 집계 API — intent 제외 개정 필요) · 여러 창 접힘 덮어쓰기(키 단위 설정 연산).
- 19 인수 TC — 상대가 둘 이상인 PC 에서 '나' 상대별 절 순서(레일 순서) · 풀린 상대 절 · 요청 · 저장된 항목이 있을 때 묶음 · 접힘이 앱 다시 실행 뒤에도
  남음.
- GitHub 반영(push)은 사용자 확인 뒤.

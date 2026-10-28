# report — 1z-18.ui-fixes

종결: 2026-10-28 · 브랜치 `task/1z-18-ui-fixes`(원 작업 폴더 — worktree 없음) → main squash(1.4.3 — 릴리스 · 서명은 별도 요청). #1z-17 리뷰 수정(`a2bbe55` · `650e8b7`)도 함께 실림.

## 결과

요청 의견 칸을 카드 폭에 맞추고(단추 오른쪽 · 32px), archive 칸을 슬라이드 토글로, 즐겨찾기 줄에 머리 ★ · 둘째 줄 세부정보를, 에이전트 응답 줄에 에이전트 타일 · 제목 · 시각 ·
`[이름]`(항목 1100px, 좁으면 두 줄)을, 북마크 목록 줄에 머리 해제 단추를 줬다. 추가 요청으로 트레이 표시를 "확인할 일" — 받은 요청 · 연결 요청 · 연결 승인 · 내 요청 상태 변경만,
알림 설정과 무관 — 으로 바꾸고 설정 스위치 이름을 "브라우저 알림" 으로. 버전 1.4.2 → 1.4.3, DB 스키마 그대로.

- stage 1 — `starDetail` · `agentTag` · `convItem` `opts` · `noteForm` 자리(`.msg-body`) · archive `role="switch"` · `savedRowEls` · `unsaveRow` · CSS · 1.4.3.
- stage 2 — 디버그 빌드 · 시험 데이터 사본 실제 화면(두 테마 · 1100 ~ 1920px) · 매뉴얼 · PRODUCT · README.
- 추가 요청(리뷰 상한 뒤) — `notify::tray_marks` · 트레이 툴팁 · 설정 "브라우저 알림" · 권한 없음 안내 문구(결정 10 ~ 14).
- 평가 — check_gates 8/8(최종 측정 `b8ff6a1`), codex 라운드 1/2 needs-attention 지적 2 → 고침(`104b75d`), 라운드 2/2 needs-attention 지적 1 → 고침(`5f2f816`, 상한 — 재리뷰 없음).
  미조치 없음. 결정 10 ~ 14 변경은 상한 뒤라 재리뷰 없음.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 의견 칸 카드 폭 · 단추 오른쪽 32px(목록 · 보드 · 대화) | 충족 | E4, E6(폼 981 = 안쪽 981 · 198 = 198 · 대화 238) |
| A2 archive 슬라이드 토글 · 동작 그대로 | 충족 | E4, E6 |
| A3 즐겨찾기 줄 머리 ★ 해제 · 둘째 줄 | 충족 | E3 star_detail, E4, E6 |
| A4 에이전트 응답 줄 · 1100px · 좁으면 두 줄 | 충족 | E3 agent_tag, E4, E6(1100 한 줄 · 1088 이하 두 줄) |
| A5 기존 동작 · 검증 · 문서 · 1.4.3 | 충족 | E1 · E2 · E5 · E7 |
| A6 북마크 목록 해제(열린 대화 · 사본도) | 충족 | E4, E6(r07), 리뷰 R1 · R2 수정 화면(r08 · r09) |
| A7 트레이 확인할 일 = 요청 · 연결 요청 · 연결 승인 · 내 요청 상태 변경, 설정 무관 | 충족(함수 시험) | E8 — 실제 트레이 화면(Windows · macOS)은 확인 안 함 |
| A8 설정 "브라우저 알림" · 상태 문구 | 충족 | E4 PRESENT |

- 편차 · 한계:
  - 리뷰 R1 지적 1 — 북마크 목록에서 해제해도 열린 대화 말풍선 아이콘이 켜진 채(대화 서명에 북마크 없음) → `markSaved` 공통 함수.
  - 리뷰 R1 지적 2 — 해제 뒤 목록 다시 읽기가 그 사이 연 화면의 조회를 버림(공유 순번) → 북마크 화면일 때만 다시 읽기.
  - 리뷰 R2 지적 1 — 해제 중 연 대화의 늦은 응답이 켜진 아이콘을 남김 → 새로 고침 때 북마크 단추만 서버 값에 맞춤(실제 화면 재현 · 고친 뒤 3 회 확인).
  - 트레이는 함수 시험(`notifier` · `events_api`)으로만 확인 — `tray.rs` 는 툴팁 글만 바뀜, 두 OS 공통 `tray-icon`. 릴리스 전 실기기 확인 권장.
  - 추가 요청 결정 10 ~ 14 는 코드리뷰 2/2 상한 뒤 — 재리뷰 없음.
  - 커밋 중 다른 세션 미커밋 줄(PRODUCT security-posture 2 줄)이 한 번 섞여 로컬에서 amend 로 뺐다(push 전) · an_app 매뉴얼 중복 줄 한 번 생겨 `e14b366` 에서 정리.

## 사용자 결정 필요

> **2026-10-28 사용자 결정 — 종결**: 사용자 "U2: 종결 승인." · "U3: squash push 까지만 진행. 서명추가 및 gh release 는 별도 요청 예정."

- intent-3 · 8 — ★ 자리 = 줄 머리 해석: 사용자가 고치지 않고 진행 · 종결 — 그대로.
- intent-10 ~ 14 — 확인 답으로 닫힘(연결 승인 넣음 · "확인할 일 N건" · 권한 없음 문구 · "브라우저 알림").
- 구현자 해석: plan 승인 질문 없이 구현(사용자 "개발 진행해"), 받은 요청 = `Ask` · 연결 요청 = `PairRequest`, 초대 자동 등록(`AutoPaired`) · 사람 글은 트레이에서 뺌.

## 산출

- 코드: `crates/app/web/js/{sidebar,conv,activity,shell,events}.js` · `js/logic/{conv,message}.js` · `js/settings/prefs.js` · `index.html` · `app.css` ·
  `crates/app/src/{notify,tray}.rs` · `crates/*/Cargo.toml` · `Cargo.lock`.
- 시험: `view.test.mjs`(star_detail · agent_tag) · `ux_literals.rs` · `tests/notifier.rs` · `tests/events_api.rs` · `tray.rs` 단위 시험.
- 문서: [web-ui](../../manual/dev/web-ui.md) · [an_app](../../manual/dev/an_app.md) · [개발자 가이드](../../manual/dev/developer-guide.md) · [사용자 가이드](../../manual/user/user-guide.md) ·
  PRODUCT · README · masterPlan(S4 · D29 · 개정 기록).
- 문서 체인: [intent](intent.md) · [plan](plan.md) · [stage1](work/stage1.md) · [stage2](work/stage2.md) · [eval](eval.md).

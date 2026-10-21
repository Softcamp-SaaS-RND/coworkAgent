# report — 30.auto-update

종결: 2026-10-22 · 브랜치 `task/30-auto-update` · `task/30-release-130` → main(squash)

## 결과

Windows **자동 업데이트**(트레이 모드 감시가 앱 전에 GitHub `Softcamp-SaaS-RND/coworkAgent` 최신 정식 릴리스 확인 → 받기 → 회사 인증서 Authenticode · 서명자 CN
`SOFTCAMP CO.,LTD.` · 제품 이름 · 파일 버전 = 태그 확인 → 묻지 않고 교체 → 새 버전으로 뜸, 뜨지 못하면 되돌림)과 서명 정식 릴리스 v1.1.1 · v1.2.0 · v1.3.0.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 서명 정식 릴리스 · `releases/latest` · SHA-256 | 충족 | v1.1.1 · v1.2.0(2026-10-21) · v1.3.0(2026-10-22, Latest — SHA-256 `5C9D5D97…BC2DE11` 받은 파일 = 등록 파일) · E4 |
| A2 감시가 앱 전에 확인 · 교체 · 실패는 지금 버전 | 충족 | E3(로컬 HTTP) · E5 · E7 |
| A3 서명 없음 · 고친 사본 · 다른 서명자 · 낮은 버전 거절, `.new` 안 남김 | 충족 | E3 · E4(`real_signed_release` — `v99.0.0` 태그 · 다른 제품 이름 거절) |
| A4 업데이터가 든 낮은 버전이 실제로 최신 서명 릴리스로 바뀌어 뜸 | 충족 | E7 — 1.1.9 → v1.3.0(plan 개정 — 사용자 "의견수용"), 로그 `Updated("v1.3.0")` · `.old` 삭제 |
| A5 릴리스 절차 · 자동 업데이트 안내 | 충족 | E6(개발자 가이드 2.4 · 사용자 가이드 · README) |
| A6 회귀 없음 | 충족 | E1 · E2 · E6 |

- 적대적 평가 2 라운드(상한) — R1 high(서명된 옛 버전을 높은 태그에 붙여 내려 깔기) · R1 medium(`Updated` 결과 덮어쓰기) · R2 medium(`.old` 삭제 실패로 정상 버전 되돌림)
  반영. 미조치 2행 — 원장 `#30 R2-1` · `#30 (#1y 이어받음)` 조건 대기([eval](eval.md)).
- E7 실측 중 관측: 시험 사본(scratchpad)이 릴리스 빌드라 HKCU `Run` 을 제 경로로 고쳐 썼고, 원래 앱을 다시 띄우자 원래 경로로 되돌아왔다(D13 — 시작마다 맞춤).
- 실제 데이터 DB 는 E7 전에 사용자가 띄운 1.3.0 이 스키마 24 로 올렸다(`coworkAgent.db.v23.bak` 2026-10-22 04:19).
- plan 밖: 1.3.0 버전 올림 · README 1.3.0 줄(#1z-3 · #05-2 를 담은 릴리스) — 사용자 "의견수용".

## 산출

- app: `update`(확인 · 받기 · `verify` = Authenticode + `identity` · `swap` · `settled` · `clean`) · 감시 연결(`main.rs`) · `update.txt` 덧붙임 · 시험 `update`(10 + 실제 배포처 2 `#[ignore]`).
- 릴리스: v1.1.1 · v1.2.0(Windows exe + macOS zip) · v1.3.0(Windows exe — macOS zip 은 보유한 Mac 에서 첨부 몫).
- 문서: an_app · 개발자 가이드 2.4(빌드 → 서명 → 확인 → 등록) · 사용자 가이드 · README · PRODUCT · masterPlan D12 · D36 · S6 · Phase 3 표 · 개정 기록.

## 남은 것

- 미조치 2행(원장 조건 대기). v1.3.0 macOS zip 첨부(보유한 Mac). 상대 버전 차이 표시 · 파이프라인 · macOS 자동 업데이트는 TODO(D36).

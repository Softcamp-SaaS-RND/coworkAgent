# report — 1z-19.tray-guide

종결: 2026-11-03 · 브랜치 `task/1z-19-tray-guide`(원 작업 폴더 — worktree 없음) → main squash(`d3c1456` v1.4.4 — 사용자 "병합 및 push 진행." · 릴리스 · 서명은 별도 요청).

## 결과

정상 모드 트레이 메뉴에 "매뉴얼"(처음 "사용자 매뉴얼" — 사용자 지시로 개정)을 더해 GitHub main 의 사용자 가이드를 기본 브라우저로 연다. 사용자 가이드는 579줄 · 17장에서
기능별 따라 하기 10장 · 캡처 11장(웹 10 · 트레이 1)으로 다시 썼고, README 의 가이드 장 번호(`§n`) 16개를 새 장으로 맞췄다. 버전 1.4.3 → 1.4.4, DB 스키마 그대로.

- stage 1 — `tray::USER_GUIDE` · 메뉴 항목 · `ux_literals` PRESENT · COUNTS 4 → 5 · 1.4.4.
- stage 2 — mock(가상 데이터) + ego-browser 캡처 10장, 실제 앱 트레이 메뉴 캡처 · 클릭 확인.
- stage 3 — 가이드 재작성 · README · manual README · 개발 문서 3곳 · PRODUCT.
- 개정(stage 3 뒤) — 메뉴 글자 `매뉴얼`(intent 결정 1 라운드 2), 릴리스 빌드에서 `tray-menu.png` 다시 찍고 E3 다시 확인.
- 평가 — check_gates 11/11(측정 `42d323b`), codex 라운드 1/2 needs-attention 지적 2(가이드 재작성 때 빠진 보안 안내) → 복원. 라운드 2 없음. 미조치 없음.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 트레이 "매뉴얼" → 기본 브라우저에 GitHub 가이드(두 OS 같은 코드 길) | 충족(Windows 실기) | E1 · E2(HTTP 200) · E3(릴리스 빌드 실기) — macOS 실기는 확인 안 함(`browser::open` 공통 길) |
| A2 기능별 따라 하기(단계 + 캡처) · 화면 글자 앱 그대로 | 충족 | E8 · E10(화면 글자 78개 원본 대조) |
| A3 캡처 = 지금 화면 · 가상 데이터만 | 충족 | E8(11장 실재) · E9 |
| A4 링크 깨짐 없음 · 문서 규약 | 충족 | E11(`readme ok` · `missing=0` · `check_ddd_docs` PASS) |

- 편차 · 한계:
  - 리뷰 R1 지적 1 — 초기화 뒤 남는 Claude Code 세션 기록 안내가 빠짐 → 가이드 10.2 ⚠ 줄 복원.
  - 리뷰 R1 지적 2 — "모든 사용자 글 삭제" 때 상대가 꺼져 있으면 상대 PC 는 켜질 때 지운다는 안내가 빠짐 → 가이드 4.8 복원.
  - 두 지적의 원인은 결정 4("내부 세부는 결과만") 를 보안 · 반출 안내까지 넓혀 적용한 것 — 보안 literal 7개 검사(E8)는 이 두 문구를 세지 않았다.
  - main push(`d3c1456`)가 리뷰보다 먼저였다(사용자 지시 순서) — 리뷰 수정은 종결 push 에 실린다.
  - 트레이 "매뉴얼" 클릭 확인 때 기본 브라우저에 같은 가이드 탭이 이전 시험분과 함께 있어, 새 탭이 이번 클릭분인지는 탭 수로 가르지 않았다(로그 실패 0줄).
  - 캡처는 1440×900(ego 스크린샷이 `deviceScaleFactor` 2 를 무시).

## 사용자 결정 필요

> **2026-11-03 사용자 결정 — 종결**: 사용자 "병합 및 push 진행." · "종결 진행".

- intent-1 — 메뉴 글자 `사용자 매뉴얼` → `매뉴얼`(라운드 2 — 사용자 지시로 닫힘).
- 릴리스(서명 · gh release)는 별도 요청.

## 산출

- 코드: `crates/app/src/tray.rs` · `crates/*/Cargo.toml` · `Cargo.lock`.
- 시험: `crates/app/tests/ux_literals.rs` · `tray.rs` 단위 시험 `user_guide_url`.
- 문서: [사용자 가이드](../../manual/user/user-guide.md) · `docs/images/` 11장 · [an_app](../../manual/dev/an_app.md) · [개발자 가이드](../../manual/dev/developer-guide.md) ·
  [web-ui](../../manual/dev/web-ui.md) · [manual README](../../manual/README.md) · README · PRODUCT · masterPlan(S4 · 개정 기록).
- 캡처 도구: [design/mock.py](design/mock.py) · [design/shots-ego.mjs](design/shots-ego.mjs).
- 문서 체인: [intent](intent.md) · [plan](plan.md) · [stage1](work/stage1.md) · [stage2](work/stage2.md) · [stage3](work/stage3.md) · [eval](eval.md).

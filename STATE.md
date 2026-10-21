# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-10-21 / task/30-auto-update — Task #30 리뷰 2/2 끝, 1.2.0 서명 대기

## 현재 작업

Task `30.auto-update`(브랜치 `task/30-auto-update`, main 미반영) — v1.1.1 서명 정식 릴리스 등록 끝(`Softcamp-SaaS-RND/coworkAgent`), 업데이터 되살림 ·
Authenticode + 서명자 · 버전 리소스 확인 · 리뷰 2/2 반영. 열린 Task = `19.prototype-acceptance`(사용자 일괄 수동 시험 대기).

- **중단 지점**: `target/release/coworkAgent.exe`(1.2.0, 서명 없음 — SHA-256 `7508e882…873875`)에 선생님 서명 대기. E7 시험용 1.1.9 빌드 = scratchpad `e7\coworkAgent.exe`.
- **재개 순서**: 서명 확인(Valid · CN · 속성 1.2.0) → squash push → `gh release create v1.2.0 … --target <SHA> --latest` → E7(떠 있는 F:\coworkAgent.exe 끄고 1.1.9 사본 트레이 실행 →
  v1.2.0 교체 · 로그 "자동 업데이트:" · `.old` 삭제 → 사본 끄고 원래 앱 다시) → masterPlan 환류(작업 폴더 초안 있음) · report · archive · 종결 squash push.
- PRODUCT.md 의 다른 세션 수정(security-posture 링크 2줄) · docs/tech/security-posture.md 는 그대로(미커밋).

## 다음 행동

1. 19 acceptance TC(사용자) — 19 문서는 `task/19-prototype-acceptance` 브랜치에(main 병합 — 이후 main 은 일반 merge). 19 원장 재측정 · `tools/check_acceptance.py` 번호 형식(1a ~ 1z) 수정 필요.
2. v1.0.0 · v1.0.1 · v1.1.0 macOS zip 첨부(보유한 Mac).

- **블로커 / 미해소**: 19 acceptance TC(사용자). 그룹 타일 묶음 머리 "에이전트 대화" vs 메뉴 "에이전트 응답"(#1z-2 report — 사용자 확인 필요). 구현자 해석(사용자 미확인 — 각 report): 1q 결정 7 · 15,
  1r plan 해석 1 · 3 · 결정 3 · 11, 1s 결정 1 · 2, 1z-2 결정 2. D34 개정 문구 "편집 · 삭제는 CLI(· API)" 와 달리 CLI 에 편집 · 삭제 명령 없음(확인 필요).

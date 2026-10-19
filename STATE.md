# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-10-19 16:40 / main — 1.0.0 배포(squash push · v1.0.0 사전 배포)

## 현재 작업

| 항목 | 값 |
|------|-----|
| Task | 없음 — 1w.conv-leave 종결([report](docs/archive/1w.conv-leave/report.md)) |
| 단계 | 다음 Task 착수 전 · v1.0.0 macOS 파일 첨부 대기 |
| 재개 좌표 | [masterPlan](docs/masterPlan.md) Phase 1 오버플로 "남음" 후보 |

- **직전에 한 일**: 1w 종결 — 대화 ⋯ 나가기 · 삭제(D40 — `Leave` · `Pong.leave` · `left` · 스키마 21 묘비 · 번호 재사용 없음), 그룹 = 대화만 · 설정 "사용자 목록"(D37 개정),
  적대적 평가 2/2 반영, 원장 10행, archive. 적대적 평가 수단을 `codex-peer-review` 스킬로 바꿈(PROJECT.md · 전역 지침).
  graphify 전면 갱신(3950 노드 · 212 커뮤니티, `.graphifyignore` 로 `arapaima/` 등 제외) · 규칙 = Task 종결 시 최소 AST 갱신(PROJECT.md).
- **중단 지점**: 없음. 작업 폴더의 PRODUCT.md 수정(security-posture 링크) · docs/tech/security-posture.md 는 다른 세션 작업 — 사용자 지시로 그대로(미커밋).

## 다음 행동

1. v1.0.0 릴리스에 macOS `coworkAgent-macos.zip` 첨부(보유한 Mac 빌드 — D11 · D25 보강) · 다음 Task 고르기(masterPlan 후보) · 이 PC 반영 — `python tools/rebuild_run.py`(사용자 확인 뒤, 스키마 21 로 오른다 — `.v20.bak` 사본).
2. 19 acceptance TC 일괄(사용자 — 1u E10 · 1v E12 · 1w E12 포함). 다음 적대적 평가 전 Orca 에 `review` 이름의 Codex 세션 준비(`codex-peer-review`).

- **블로커 / 미해소**: 19 acceptance TC(사용자). 구현자 해석(사용자 미확인 — 각 report): 1q 결정 7 · 15,
  1r plan 해석 1 · 3 · 결정 3 · 11, 1s 결정 1 · 2. D34 개정 문구 "편집 · 삭제는 CLI(· API)" 와 달리 CLI 에 편집 · 삭제 명령 없음(확인 필요).

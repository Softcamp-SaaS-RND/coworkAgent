# stage3 — README · 매뉴얼 · masterPlan

> 근거: [plan](../plan.md) Stage 3.

## 목표

사용자 · 개발 문서와 masterPlan 에 레일 범위를 반영한다. 대상 gate: E1 ~ E6.

## 산출 파일

- `README.md` — 주요 기능 "나 · 상대별 보기"(user-guide §4).
- [user-guide.md](../../../manual/user/user-guide.md) — §4 레일 · 사이드바 행 고침 + "레일 범위" 표('나' · 상대 타일 — 메뉴 · 목록 · 첫 화면 · 사이드바 대화) · 접힘 기억 ·
  내 활동은 '나' 에만 · 이 PC 에만 남는 것에 절 접힘.
- [an_app.md](../../../manual/dev/an_app.md) — 웹UI 레일 범위(`sideNav` · `peerGroups` · `peerSummary` · `waitingByPeer` · `convSections(rows, paired)` · 주소 바꿈 ·
  `ui_folded`) · 설정 키 `ui_folded`. [developer-guide.md](../../../manual/dev/developer-guide.md) — 설정 키 목록.
- `PRODUCT.md` — 웹UI 구조의 사이드바(D35).
- [masterPlan.md](../../../masterPlan.md) — D35 추가 · S4 보강 · Phase 1 오버플로 `1g.rail-scope` · 개정 기록 두 줄(사용자 지시 · Task #1g).

## 테스트 결과

```text
$ python tools/check_ddd_docs.py → PASS
$ python tools/check_readme.py → readme ok
```

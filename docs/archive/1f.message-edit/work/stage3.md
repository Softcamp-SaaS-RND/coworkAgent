# stage3 — README · 매뉴얼 · masterPlan

> 근거: [plan](../plan.md) Stage 3. 사용자 요청 "완료 후 readme , 매뉴얼 문서 업데이트".

## 목표

복사 아이콘 · 편집 모드 · 상대 창 반영을 사용자 · 개발자 문서에 반영하고 masterPlan 비목표를 개정한다(D34). 대상 gate: E1 ~ E3 · E8.

## 산출 파일

- `README.md` — 주요 기능 표 "복사 · 이력 정리" 행.
- [user-guide.md](../../../manual/user/user-guide.md) — 7.5(복사 아이콘 · 작업 막대) · 7.6 편집 모드(새) · 내 활동 '상대 반영 실패' · 16장 문제 해결 1행.
- [developer-guide.md](../../../manual/dev/developer-guide.md) — 5.2 프레임 목록(`ReqState` · `Edit` · `Delete`, 낡은 `Done` 정리) · 6.6 포기 표 · 7.2 스키마 15 ·
  7.3 양쪽 · 이 PC · 8.9 이력 정리 순서 그림(새).
- [an_core.md](../../../manual/dev/an_core.md) · [an_net.md](../../../manual/dev/an_net.md) · [an_agent.md](../../../manual/dev/an_agent.md) · [an_app.md](../../../manual/dev/an_app.md) — #1f 계약.
- `PRODUCT.md` — 도메인 규칙 "이력 정리". [masterPlan.md](../../../masterPlan.md) — 비목표 개정 · D34 · S3 보강 · Phase 1 오버플로 · 개정 기록.

## 설계 결정

- developer-guide 머리말 · 7.2 의 "스키마 13" 은 #1d(14)부터 낡아 있던 값 — 이번에 15 로 맞췄다. 5.2 의 `Done`(#1d 에서 `ReqState` 로 대체)도 정리.
- 새 순서 그림은 8.9 로 붙였다 — 기존 8.7 · 8.8 번호를 바꾸지 않는다(다른 문서의 절 번호 참조 보존).

## 테스트 결과

```text
$ python tools/check_readme.py → readme ok
$ python tools/check_ddd_docs.py → PASS — DDD 문서 규약 위반 없음
```

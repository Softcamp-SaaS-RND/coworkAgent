# stage3 — 문서 · 방화벽 안내 · 이 PC 실측 · 캡처

> 근거: [plan](../plan.md) Stage 3. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

해석 10(방화벽 안내) · README · 매뉴얼 · 캡처 · 이 PC 실측(E9). 대상 gate: E1 ~ E9.

## 산출 파일

- [user-guide](../../../manual/user/user-guide.md) — 4장 표(프로필 단추 · 타일 글자 · 겹친 카드 · 메뉴 세 절 · 첫 화면 '개요' · 북마크 항목) · 3장 트레이 다시 시도 · 7.4
  사라진 세션 · 7.5 북마크(★ 와의 차이) · 17장 방화벽 프로필 확인 · Public 끄기(확인 필요 표기) · 그림 설명.
- `README.md` — 북마크 항목. `PRODUCT.md` — 메뉴 세 절 · 그룹 겹친 카드 · 안정화 절(#1h).
- [developer-guide](../../../manual/dev/developer-guide.md) — 스키마 17 · `peers` AUTOINCREMENT · 사라진 세션 표 행. `an_core.md` 스키마 17 · `an_agent.md` `SessionMissing` ·
  `an_app.md` 메뉴 세 절 · 북마크 · 프로필 · 타일 글자 · 트레이 다시 시도.
- `docs/images/{workspace,home,group}.png` — 가상 데이터 mock(scratchpad `readme3/shot3.mjs`)으로 다시 찍음(1440 × 900 @2x).

## 설계 결정

- **방화벽 끄는 명령은 확인하지 않음** — 이 PC 방화벽을 바꾸지 않는다(plan 해석 10). 가이드에 "확인 필요" 를 적었다.

## 검증 — E9(이 PC 실제 앱 · 비파괴)

- `python tools/rebuild_run.py` → 스키마 16 → 17: `user_version` 17 · 사본 `coworkAgent.db.v16.bak` · `peers` 정의 AUTOINCREMENT · 상대 번호 `[1]` 그대로 ·
  순번 1 · 열 19 · 그룹 2 · 대화 2 · 글 6(올리기 전과 같음).
- 화면(scratchpad `h1/e9.mjs`, 실데이터 캡처는 공개하지 않음) 16/16 PASS — 타일 글자 넓이 [2, 4, 3, 2] ≤ 4 · 타일 안 · 그룹 타일 2 겹친 카드 · 상대 타일 카드
  없음 · '나' 메뉴 개요 | 내 활동 · 요청 · 주고받은 글 | 북마크 항목 · 에이전트 · 구분선 `role=separator` · 첫 화면 "개요" · 상대 타일 두 절 · 그룹 타일 세 절 ·
  "저장된 항목" 없음 · 북마크 항목 제목 · 메시지 작업 "북마크" 4 · 실행 상세 절 · 프로필 이름 안 잘림 · JS 오류 0.
- 트레이 다시 시도(로그인 직후) · 사라진 세션의 실제 claude 실행은 19 인수 TC 로 넘긴다(plan E9).

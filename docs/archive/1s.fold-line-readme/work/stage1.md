# stage1 — 세로선 제거 · README · 캡처

## 목표

plan Stage 1 — 접이식 하위 줄 세로 안내선 제거(들여쓰기 유지), README 를 1q · 1r 화면에 맞춤, 캡처 5장 새로.

## 산출 파일

- `crates/app/web/app.css` — `.peer-fold > :not(summary) { margin-left: var(--sp-5); }`(세로선 지움).
- `crates/app/tests/ux_literals.rs` — 1r 들여쓰기 잠금 행을 새 규칙으로, ABSENT `#1s 세로선 제거`(먼저 실패 2건 확인 → 고친 뒤 통과).
- `README.md` — 기능 표 5행(복사 · 이력 정리 "✎ 편집" · 요청 목록 개요 · 글자 단추 · 자동 왕복 내 응답 에이전트 · 최대 응답 횟수 · 나 · 상대별 보기 접이식 들여쓰기 ·
  주고받은 메시지 20건씩 · 검색 · 알림 내 활동 범위 · '?'), 시작하기 2 ~ 4(메뉴 초대 생성 = 바로 만들기 · 새 에이전트 줄 · ✎ · 내 응답 에이전트 "안 함").
- `docs/images/{workspace,pairing,agents,home,group}.png` — 1q mock + 1p 캡처 스크립트 `doc` 모드(2x).
- [web-ui.md](../../../manual/dev/web-ui.md) · [masterPlan.md](../../../masterPlan.md) D35 · 개정 기록.

## 검증

- mock(`#agents` 다크 · `#requests` 라이트) — 세로선 없음 · 단계 들여쓰기 유지 · 행 구분선은 들인 자리에서 시작.
- 캡처: `agents.png` 연필 · "새 에이전트" 줄 · 취소 / 등록, `pairing.png` 연결 목록 연필, `home.png` 개요 상대 절 들여쓰기(세로선 없음). 캡처 스크립트 `ERRORS []`.
- `python tools/check_readme.py` → `readme ok`.

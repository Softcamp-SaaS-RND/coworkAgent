# stage4 — 문서 · mock 확인

## 목표

plan Stage 4 — 손댄 화면 · API 의 매뉴얼 · 사용자 가이드 갱신, mock(가상 데이터) 눈 확인.

## 산출 파일

- [사용자 가이드](../../../manual/user/user-guide.md) — 4장 주고받은 메시지(접힘 요약 · 20건씩 · 행 북마크) · 7장 내 활동(받은 요청 · 내 요청 · 받은 답 · 실패, 탭) ·
  9장 요청(개요 · 탭 토글 · 글자 단추) · 10장 자동 왕복 끝 사유(알림으로, 내 활동에 없음) · 14장 `coworkAgent /?`(stage5).
- [웹UI 화면 규칙](../../../manual/dev/web-ui.md) 화면 구조 — 주고받은 메시지 · 요청 · 내 활동.
- [an_app](../../../manual/dev/an_app.md) — #1q API(`/api/activity` kind · `side=both` · `before` · `/api/messages/peers`) · CLI `/?`.
- [개발자 가이드](../../../manual/dev/developer-guide.md) 저장 표 — `relay_events.kind = req_state`.
- `design/mock.py` — 요청 · 내 활동 · 주고받은 메시지 가상 데이터 · API.

## 설계 결정

- 자동 왕복 끝 사유는 내 활동에서 빠져 알림("자동 왕복" 알림을 켰을 때)으로만 남는다 — 사용자 결정 3("자동 왕복은 제거")의 결과, 가이드에 적었다.
- README 는 stage5 에서 바꿨다(연동 절 · 기능 표). 요청 · 내 활동 화면 설명은 README 에 없어 그대로.

## 테스트 결과

- `python tools/check_ddd_docs.py` → `PASS`. `python tools/check_readme.py` → `readme ok`.
- mock 눈 확인(Edge headless 1440×900, `design/mock.py`): 요청 개요 다크 · 라이트(머리 건수 · 글자 단추 ✓ 수용 ✕ 반려 ✓ 완료 ·
  메모 아래) · 내 활동 다크(탭 · 라벨 태그 · 메모) · 주고받은 메시지 다크(접힘 머리 요약 · 펼침 · 행 북마크) — 모두 `pageerror` · 콘솔 오류 없음.
  실제 두 PC 확인(E11)은 D28 에 따라 19 인수로.

## 다음 단계

eval — `tools/check_gates.py` 평가 원장 · 적대적 평가(라운드 상한 2).

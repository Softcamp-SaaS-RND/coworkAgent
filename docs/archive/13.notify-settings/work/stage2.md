# stage2 — 웹UI 설정

## 목표

[plan](../plan.md) Stage 2. 대상 gate: E7 · E8.

## 산출 파일

- `crates/app/web/index.html` — 설정 → 알림: 페어링 · 받은 요청 · 받은 답 체크, 자동 왕복 알림(시작 · 끝만 / 글마다 / 알리지 않음), 글 앞부분
  보이기, 새 자동 왕복 대화의 상한 기본값.
- `crates/app/web/app.js` — `PREFS` · `fillPrefs`: 바꾼 항목만 `PUT`, 실패하면 서버 값으로 되돌림. 새 대화 상한 칸 처음 값 = 기본값.

## 설계 결정

- 입력 id = 설정 이름 — 저장 · 채우기가 한 목록(`PREFS`)으로 돈다. 값은 `value` · `checked` 로만(HTML 주입 없음).

## 테스트 결과

- node `# fail 0`(화면 조립은 자동 테스트 대상 밖 — E8 수동, D28).

## 다음 단계

평가(gate · 적대적 평가 2 라운드) · 종결.

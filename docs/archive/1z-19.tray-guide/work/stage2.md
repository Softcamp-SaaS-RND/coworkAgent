# stage 2 — 캡처 도구 · 캡처

## 목표

[plan](../plan.md) stage 2 — 사용자 가이드용 캡처 11장(웹 10 · 트레이 1), 실제 앱 트레이 메뉴 확인(E3). 캡처 수단은 ego-browser(plan 개정 — 사용자 지시).

## 산출 파일

- `design/mock.py` — 1q mock 사본. 정적 파일 `/js/**`(ES 모듈 진입 `js/app.js`), `GET /api/conversations/{id}/info`(세부정보 '대화 정보'), `/api/status` 버전 1.4.4,
  그룹 "결제 프로젝트"(대화 1 · 6), 없는 경로 stderr `404 …`.
- `design/shots-ego.mjs` — ego-browser 캡처 순서(10장).
- `docs/images/` — `home` · `group` · `pairing` · `agents` · `new-conv` · `workspace` · `requests` · `relay` · `notify` · `settings`(새로 · 다시, 1440×900 라이트) · `tray-menu`(새로, 210×108).

## 설계 결정

- 화면 = 해시 주소로 옮긴다(`#` · `#settings/conn` · `#settings/agents` · `#new` · `#c/1` · `#requests` · `#settings/notify` · `#settings/run`), 그룹은 레일 그룹 타일 클릭.
  `pairing` = "초대 생성" 펼침, `agents` = review 상세, `new-conv` = 상대 이준호 · guide, `relay` = 같은 폼에 "내 응답 에이전트" review(최대 응답 횟수 보임),
  `workspace` = 대화 1 + 입력창 `@` 후보, `settings` = 실행(모델 · effort · 동시 실행 상한 · 자동 압축).
- 브라우저 확장이 페이지에 넣는 요소(파란 구름 단추 — `#ds365-convert-panel-host`, `#crx-mouse-redesign-content-root`)는 DOM 에서 지워도 다시 붙어 CSS 로 숨긴다.
- `deviceScaleFactor: 2` 는 ego-browser 캡처에 반영되지 않아 1440×900 그대로 둔다(글자 판독 가능).
- 트레이 메뉴: 사용자의 실 인스턴스를 끄지 않으려고 `rebuild_run.py` 대신 디버그 빌드를 시험 데이터 루트(scratchpad `trayapp`)로 띄웠다(피어 포트는 임의 포트로).
  넘침 창 아이콘을 UI Automation 으로 찾아 우클릭 → 화면 캡처 → 메뉴 상자만 잘라 회색 여백을 둠. 사용자 캡처 요청은 필요 없었다.

## 테스트 결과

- mock 마지막 실행 stderr `404` 0줄(첫 실행에서 `/api/conversations/1/info` · `/3/info` 404 → 엔드포인트 추가), `#disconnected` 없음.
- UI Automation: `coworkAgent / 피어 포트 47100 사용 중 — … 임의 포트 64993 로 받는다 | 2142,1135,40,40`(디버그) · `coworkAgent | 2218,1392`(사용자 실 인스턴스 — 손대지 않음).
- 메뉴 캡처 = "열기 · 사용자 매뉴얼 · 종료". "사용자 매뉴얼" 클릭 5초 뒤 `Get-Process` 창 제목 `coworkAgent/docs/manual/user/user-guide.md at main · Softcamp-SaaS-RND/coworkAgent - ego`(기본 브라우저).
- 디버그 앱 로그: `시작 — 127.0.0.1:64992` … `종료 요청(로컬 API)` · `종료`(열기 실패 줄 없음), `coworkAgent stop` → `종료함` rc 0, 사용자 실 인스턴스 PID 47568 · 48728 그대로.
- 11장 눈 대조: 머리 `coworkAgent v1.4.4`, 가상 인물 · PC · 경로만.

## 다음 단계

stage 3 — 사용자 가이드 다시 쓰기(옛 장 → 새 장 표) · README `§n` · 같은 개념 문서(an_app · developer-guide · web-ui) · PRODUCT 줄.

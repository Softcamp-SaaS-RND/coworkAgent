# stage4 — 문서 · 캡처 · mock 확인

> 근거: [plan](../plan.md) Stage 4(해석 8 · 9).

## 목표

목록 나눔 · 말풍선 음영 · 관람자 초대 · 레일 우클릭 · 설명 지우기를 문서에 싣고, 캡처 3장을 다시 찍고, 가상 데이터 mock 으로 화면을 확인한다. 대상 gate: E7 ~ E10 · E16.

## 산출 파일

- 사용자 가이드 — 4장(레일 우클릭 · 실행 상세 관람자 · 개요 요약 · 에이전트 "새 대화" · 대화방 목록 묶음) · 7.2 말풍선 색 · **7.7 관람자 · 공유 대화** · 16장 두 줄.
- 개발자 가이드 — 5.2 공유 프레임 · `Pong.share` · outbox, 7.1 · 7.2 스키마 20(`shares` · 사본 열 · `owner`), **8.11 공유 대화**(순서 그림 · 퍼뜨리기 · 막기 · API ·
  ponytail · 남은 것), 9장 공유 대화 경계 줄.
- 매뉴얼 — `an_net`(프레임 · `Pong.share` · `knows_share`) · `an_core`(`Dir::Shared` · 스키마 20 · `share` 모듈 · `SCHEMA_VERSION` 18 → 20 옛 숫자도 고침) ·
  `an_agent`(`share_add` · `share_remove` · `share_leave` · `fan_share` · 오류 종류) · `an_app`(API · 화면 함수 · CLI).
- README — 메신저형 대화 줄(음영 · 목록 묶음) · 새 줄 "공유 대화(관람자)".
- 캡처(가상 데이터 mock8 — 인물 · PC · 대화 모두 지어낸 것, 다크 1440×900 @2x) — `docs/images/home.png`('나' 상대별 절 안 내 대화 · agent 대화 · 공유 대화) ·
  `group.png`(그룹 "결제 모듈" · 대화 추가) · `workspace.png`(말풍선 Ⓐ · 머리 "관람자 1" · 세부정보 관람자 절).

## 설계 결정

- 공유 대화 세부정보 "대화 정보" 는 "대화 에이전트" · 실행 기본값 = 마지막 실행 값(이 PC 가 실행하지 않는다), 요청 빈 안내는 "요청 없음" 만 — mock 확인에서 잡아 고침.
- 관람자 안내 "{이름}이/가 공유한 대화" 는 `josa` 로.

## 테스트 결과

mock(scratchpad `o1/mock8.py` · `o1/o1doc.mjs`, Edge headless) — `RESULT 8/8`, `ERRORS []`:

```text
PASS A2 '나' 상대별 절 안 머리 ["내 대화3","agent 대화2","내 대화1","agent 대화1","공유 대화2"]
PASS 설명 지움 — 개요 머리 null
PASS A7 머리 관람자 1
PASS A7 공유 대화 읽기 전용 {"reply":true,"edit":true,"ctx":true,"sub":"공유 · 최유나 ↔ mspark · 읽기 전용"}
PASS A7 발화자 — 공유한 사람 · 원 상대 · 원 상대 에이전트
PASS A9 공유 해제됨 태그
PASS A6 레일 바탕 우클릭 기본 메뉴 막힘(contextmenu defaultPrevented)
PASS 에이전트 새 대화 단추
```

- 라이트 테마 공유 대화 · 대화 캡처도 눈으로 확인(scratchpad `o1/chk/*_light.png` — 문서에 넣지 않음). mock 은 gate 증거가 아니다(plan 해석 8).
- 브라우저 기본 우클릭 메뉴가 실제로 안 뜨는지는 자동으로 볼 수 없다 — E16 실화면.

```text
python tools/check_readme.py   → readme ok
python tools/check_ddd_docs.py → PASS
```

## 다음 단계

- eval — `check_gates`(E1 ~ E9 · E11 ~ E15) · E10 · E16 은 D28 에 따라 ABANDON + 19 TC · 적대적 리뷰(≤ 2 라운드).

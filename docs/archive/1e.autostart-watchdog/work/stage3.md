# stage3 — 매뉴얼 · PRODUCT · 실측

> 근거: [plan](../plan.md) Stage 3. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

감시 프로세스 · 자동 시작 · 패닉 기록을 사용자 · 개발자 문서에 반영하고, Windows 릴리스 실측(E7)을 한다. 대상 gate: E1 ~ E3 · E7.

## 산출 파일

- [user-guide.md](../../../../docs/manual/user/user-guide.md) — 3장(로그인 자동 실행 · 비정상 종료면 다시 켜짐 · 3번이면 붉은 아이콘) · 15장(자동 실행 등록 위치 — 데이터
  폴더 밖) · 16장 문제 해결 3행(프로세스 둘 · 로그인해도 안 켜짐 · 3번 비정상 종료).
- [developer-guide.md](../../../../docs/manual/dev/developer-guide.md) — 3.1 프로세스 표(감시 프로세스 행) · 4장 서비스 표(감시 · 자동 시작) · 4.1 시작 순서(감시 분기 · 패닉 훅 ·
  자동 시작 등록).
- [an_app.md](../../../../docs/manual/dev/an_app.md) — `watchdog` · `autostart` · 설정 `autostart` 훅. [an_core.md](../../../../docs/manual/dev/an_core.md) — 패닉 훅 · `AUTOSTART_KEY`.
- `PRODUCT.md` — 기술 스택(windows-sys 레지스트리) · 도메인 규칙 "실행 구조 · 자동 시작".
- `README.md` — 시작하기 1단계에 자동 실행 · 다시 켜짐 한 줄.

## 설계 결정

- **E7 실측은 하지 않았다 → 19 TC.** 이 PC 에 coworkAgent 프로세스가 실행 중(PID 7364, 2026-10-13 12:33 시작)이다. 실측하려면 같은 데이터 폴더 ·
  `instance.lock` · `HKCU\…\Run` 을 쓰게 되고(격리 수단 없음 — 데이터 루트는 OS 알려진 폴더), 새 실행은 그 앱에 "열기" 를 넘기고 끝난다. plan
  승인 조건("실행 중인 앱이 있으면 끄지 않고 19 TC 로 넘김")대로 `ABANDON` 으로 드러낸다.
- 자동 시험이 덮는 범위: 감시 규칙 전부(가짜 자식) + 실제 실행 파일 자식의 비정상 종료 3번 멈춤 · CLI 우회(E4), 레지스트리 등록 · 고침 · 해제는
  시험 키로 실제 API(E6), 설정 API(E6). 덮지 못하는 것 = 릴리스 배선(`main.rs` 의 `cfg!(debug_assertions)` 분기 · 실제 Run 키 · 트레이 오류 모드
  화면) — 19 TC 항목.

## 테스트 결과

```text
$ python tools/check_readme.py → readme ok
$ python tools/check_ddd_docs.py → PASS — DDD 문서 규약 위반 없음
```

코드는 이 stage 에서 바꾸지 않았다(Stage 2 결과 그대로 — 327 passed · clippy Finished · node 47).

## 다음 단계

`eval.md` — gate 기계 판정 · E7(19 TC) · E8(D25) `ABANDON` · 적대적 평가 라운드 1/2.

# stage3 — app 배선 · API · 웹UI · 트레이

## 목표

[plan](../plan.md) Stage 3 — 시작 시 신원 · 피어 수신 · `Node` 배선, `/api/me` · `/api/peers*`, 웹UI 내 정보 · 피어 섹션,
트레이 "내 정보 보기". 대상 gate: E1 · E2 · E7 · E8 · E10 · E11.

## 산출 파일

- `crates/app/src/{peers_api,server,main,tray,lib}.rs` · `crates/app/web/{index.html,app.js,app.css}`
- `crates/app/tests/peers_api.rs`(2) · `crates/app/tests/common/mod.rs`(`send_body` 추가)

## 설계 결정

- `server::router_with(…, extra)` — 상태·핸드오프 경로와 `extra` 를 합친 뒤 guard 를 한 번에 씌운다. `router` 는
  `router_with(…, Router::new())` 그대로(00 테스트 시그니처 유지).
- 오류 매핑: `NotFound` 404 · `InvalidInput`(주소 형식 · 자기 자신) 400 · 그 밖(연결 실패 · 상대 거절 · 시간 초과) 502,
  본문 `{"error": …}`.
- JSON 요청은 axum `Json` 추출기 — `Content-Type: application/json` 이 아니면 415(폼 형태 교차 요청 차단, Origin 검사에 더한 층).
- `/api/me` 의 `addrs` = 기본 경로 IPv4(UDP `connect` 로 경로만 고름, 패킷 없음) + 피어 포트. 못 구하면 빈 배열(웹UI 는 포트만).
- 트레이 "내 정보 보기" = 진입 URL + `#me` — 진입 303 의 `Location: /` 에 조각이 없으면 브라우저가 원래 조각을 잇는다.
- 시작 순서: 경로 → 로그 → 잠금 → DB(스키마 2) → 신원 키(손상 시 시작 실패 · 메시지 상자) → 런타임 → 웹UI · 피어 포트 → 트레이.
- 이름 = `USERNAME`/`USER`, PC 이름 = `COMPUTERNAME`, 없으면 `hostname` 명령.

## 테스트 결과

- 워크스페이스 49 통과 · clippy 경고 0 · `crates=3 edges=3 violations=0` · 릴리스 exe 1개(`agentNetwork.exe`).
- mutation: `extra` 를 guard 밖에 merge 하면 `me_shows_fingerprint_and_port`(토큰 없이 401 단정) FAIL 확인.
- 실제 exe 실행 스모크는 하지 않았다 — `0.0.0.0` 수신이 Windows 방화벽 허용 대화상자를 사용자 데스크톱에 띄운다. E10 · E11
  수동 확인 때 첫 실행에 "개인 네트워크 허용" 이 필요하다.

## 다음 단계

gate 원장(`check_gates.py`) → 적대적 평가(BASE `b6d6647`, 최대 2 라운드) → E10 · E11 사용자 수동 확인 → 종결.

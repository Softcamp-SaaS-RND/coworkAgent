# stage2 — 로컬 웹UI 서버 + 상태 페이지

## 목표

[plan](../plan.md) Stage 2 — `crates/app`(`agentnetwork-app`, lib `an_app`)에 로컬 웹UI 서버(127.0.0.1 · 토큰 · Host ·
Origin), 상태 페이지, 단일 인스턴스 핸드오프 클라이언트. 대상 gate: E4 · E8 · E12.

## 산출 파일

- `crates/app/Cargo.toml` — `an-core`(path) · axum 0.8.9 · tokio 1.53.2 · getrandom 0.4.3 · log 0.4.34
- `crates/app/src/{lib,server,handoff}.rs`
- `crates/app/web/{index.html,app.js,app.css}` — `include_str!` 로 임베드
- `crates/app/tests/` — `local_ui.rs`(6) · `handoff.rs`(2) · 개수 gate 밖: `entry.rs`(4) · `guard_extra.rs`(5) · `handoff_deadline.rs`(1) · `common/mod.rs`(실서버 기동 · 원시 HTTP)
- `Cargo.lock` 갱신

## 설계 결정

- 실행 파일(`agentNetwork` 실행 파일 타깃(Cargo bin))은 stage 3 에서 붙인다 — 이번 stage 는 lib 와 테스트만.
- 검사 순서 Host → Origin → 토큰. 토큰은 쿠키 `an_token` 또는 `Authorization: Bearer`(핸드오프용). Bearer 가 있으면
  그것만 본다(쿠키로 덮지 않는다). 비교는 길이 같으면 일정 시간(`ct_eq`).
- 진입 `/?t={token}` → 303 + `Set-Cookie: an_token=…; HttpOnly; SameSite=Strict; Path=/` + `Location: /`.
- `/api/status` JSON 은 serde 없이 직접 이스케이프(`"` · `\` · 제어 문자) — plan 의 외부 의존 8종을 늘리지 않으려고.
  axum 기본 기능이 serde_json 을 끌어오지만 직접 쓰지 않는다.
- 브라우저 열기는 `router` 인자로 주입 — 테스트는 호출 횟수를 세는 대역, 운영(stage 3)은 OS 열기.
- 테스트는 운영과 같은 `server::bind` · `server::router` 로 실제 TCP 서버를 띄우고, 헤더를 통제하려고 원시 HTTP/1.1 로
  요청한다(내부 함수 직접 호출 없음).

## 테스트 결과

RED(스텁 `todo!()`): entry 3 · handoff 2 · local_ui 6 전부 FAILED.

GREEN:

```text
$ cargo test -p agentnetwork-app --test local_ui
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
$ cargo test -p agentnetwork-app --test handoff
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
$ cargo test --workspace
test result: ok. … — 12개 테스트 바이너리 전부 ok, FAILED 0(리뷰 반영 후)
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.54s
$ python tools/check_crates.py
edge agentnetwork-app -> an-core
crates=2 edges=1 violations=0
```

중간 실패 1건: `valid_200` 의 기대 문자열이 JSON 이스케이프 전 경로(`C:\tmp\a`)였다 — 서버 출력(`C:\\tmp\\a`)이 맞고
테스트 쪽 오타. 테스트를 고쳤다.

## 리뷰

별도 리뷰어(구현 세션과 분리된 서브 에이전트, 작업 트리 vs `688c282`) — 결함 3건 · 테스트 공백 5건.

| 지적 | 판단 | 조치 |
| --- | --- | --- |
| [High] 쿠키는 포트를 구분하지 않아 같은 PC 의 다른 OS 사용자가 127.0.0.1 다른 포트로 `an_token` 을 받을 수 있다 | 사실 | 사용자 결정 "쿠키 유지" — plan 신뢰 경계 개정(개정 기록 1). 종결 시 미조치 원장에 `수용` |
| [Low] 쿠키가 이미 있으면 `/?t=` 가 리다이렉트되지 않아 토큰이 주소창에 남는다 | 사실 | 진입 분기를 쿠키 검사 앞으로. 회귀 테스트 `token_query_redirects_even_with_cookie` RED → GREEN |
| [Low] 핸드오프 읽기 시간 제한이 read 1회 단위라 조금씩 흘리는 상대에 끝없이 대기 | 사실 | 전체 5초 상한 + 상태 줄만 읽기. 테스트 `slow_drip_peer_times_out_within_deadline`, 상한 무력화 변이로 FAILED 확인 |
| 테스트 공백(쿠키 이름 접두 · 쿠키 여러 개 · 정적 자산 무토큰 · 같은 출처 Origin · `localhost` Host) | 사실(결함 아님) | `tests/guard_extra.rs` 5건 추가 — 처음부터 GREEN(이미 맞게 동작) |

## 다음 단계

[plan](../plan.md) Stage 3 — `agentNetwork` 실행 파일 타깃(Cargo bin) · tao 이벤트 루프 · tray-icon("열기" · "종료") · 단일 인스턴스 배선 ·
파일 로거 · Windows 릴리스 빌드(E1 · E2 · E3) · Windows 실동작(E10, 수동).

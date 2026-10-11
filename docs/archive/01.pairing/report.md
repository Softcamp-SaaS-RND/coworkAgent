# report — 01.pairing

종결: 2026-10-11 · 브랜치 `task/01-pairing` → main

## 결과

설치별 ed25519 신원 · 트레이 "내 정보 보기" · TLS 1.3 raw public key 상호 인증 · 초대(메신저 텍스트) 기반 페어링 · 양측 승인 ·
미페어링 거부를 세웠다. 02 이후의 메시지는 `Node::answer` 의 "페어링된 피어만" 관문 뒤로 붙는다.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 신원 키 · 지문 유지 · 손상 시 정지 | 충족 | E3 · E9 |
| A2 트레이 "내 정보 보기" | 자동 근거만 | E7(`/api/me` · 초대 API). E10(수동) ABANDON — D28 |
| A3 RPK 상호 인증 · 키 고정 | 충족 | E4 · E13 · E14(지문 고정) |
| A4 양측 승인 | 충족 | E5 · E12 · E13 · E14 |
| A5 미페어링 거부 | 충족 | E5 |
| A6 원격 입력 상한 | 충족 | E6 · E12 · E13 |
| A7 PC 2대 실동작 | ABANDON | E11 — D28(프로토타입 인수 Task) |
| A8 테스트 · 경고 0 · 의존 경계 | 충족 | E1 · E2 · E8 |

- 적대적 평가 2 라운드(상한) — 지적 5건 전부 반영(페어링 받기 창 → 초대 코드 · 재추가 시 승인 초기화 · 삭제 통지 결과 ·
  페어링 세대 · 수신 출처별 4 · 3초). 미조치 3건은 [eval.md](eval.md) → 원장.
- **개정 3(초대 텍스트 UX · 지문 고정 연결)은 라운드 상한 뒤라 적대적 평가를 거치지 않았다** — 자동 테스트(E14 · E7)와
  mutation(지문 고정 무시 · 조각 초과 허용 FAIL 확인)만. 다음 Task 의 적대적 평가 범위에 `crates/net/src/invite.rs` ·
  `tls::Pin::Fingerprint` · `POST /api/peers {invite}` 를 포함한다.

## 산출

- 신규 크레이트 `an-net`(comm) — `identity` · `tls` · `proto` · `node` · `invite`. `an-core` 스키마 2(`peers`) · `Paths::identity_file`.
- `agentnetwork-app` — `peers_api`(`/api/me` · `/api/peers*` · `/api/pairing-window`) · `server::router_with` · 트레이 "내 정보 보기" ·
  웹UI 내 정보 · 피어 · 초대 섹션.
- 매뉴얼 [manual/dev/an_net.md](../../manual/dev/an_net.md) 신규, `an_core` · `an_app` 갱신.

## 남은 것

- 수동 확인(A2 · A7)은 Phase 1 끝 `19.prototype-acceptance`(D28).
- 웹UI 는 상태 페이지에 섹션을 더한 임시 형태 — 슬랙형(D26)은 Phase 1.

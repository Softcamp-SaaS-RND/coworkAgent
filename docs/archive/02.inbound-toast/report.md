# report — 02.inbound-toast

종결: 2026-10-11 · 브랜치 `task/02-inbound-toast` → main

## 결과

인바운드 이벤트(페어링 요청 · 상대 승인)를 OS 알림으로 띄우는 옵션 — 기본 OFF, 웹UI 토글, Windows 토스트(WinRT · AUMID
`agentNetwork`), macOS `osascript`. 표시는 OS 알림 시스템이 맡아 집중 지원 · 방해 금지를 따른다(D27).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 이벤트 1회 · 거절 시 없음 | 충족 | E3(재통지 단정 포함) |
| A2 기본 OFF · 유지 · 끄면 표시 없음 | 충족 | E4 · E5(끈 뒤 대기분 폐기 포함) |
| A3 원격 문자열 XML 이스케이프 | 충족 | E4 |
| A4 실제 표시 · 클릭 · OS 억제 | ABANDON | E7 — D28(프로토타입 인수 Task) |
| A5 테스트 · 경고 0 · 의존 경계 | 충족 | E1 · E2 · E6 |

- 적대적 평가 2 라운드(상한) — 지적 5건 전부 코드 반영(전용 표시 스레드 · 유한 대기열 · osascript 5초 kill · approve 재통지
  idempotent · 끈 뒤 대기분 폐기 · 포화 시 로그 없음 · UI 실패 되돌림). 미조치 3건(테스트 공백)은 [eval.md](eval.md) → 원장.
- 01 개정 3(초대 텍스트 · 지문 고정)을 이 평가 범위에 함께 실었다 — 라운드 2 판정 "blocker 없음".

## 산출

- `an-net` `PeerEvent` · `Node::on_event`. `an-core` 스키마 3 `settings`. `agentnetwork-app` `notify`(`Notifier` · `toast_xml` ·
  `show_os` · `/api/settings`) · 웹UI 알림 토글.
- 매뉴얼 `an_core` · `an_net` · `an_app` 갱신.

## 남은 것

- 실제 토스트 표시(AUMID 레지스트리 방식이 설치 없는 exe 에서 출처 이름으로 뜨는지) · 집중 지원 억제 확인은 `19.prototype-acceptance`.
- 원격 메시지 알림은 03 에서 `PeerEvent` 에 더한다.

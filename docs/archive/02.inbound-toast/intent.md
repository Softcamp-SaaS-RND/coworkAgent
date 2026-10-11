# intent — 02.inbound-toast

승인: 2026-10-11

## 왜
인바운드 요청(처음 보는 PC 의 페어링 요청 · 상대 승인)이 와도 사용자가 웹UI 를 열어 보기 전에는 모른다. 사용자 지시 원문 "특히
인바운드 요청에 대해서 윈도우 토스트 형태로 알림기능 옵션 제공" · "해당툴의 기능은 머신 상태를 준수하도록 구현" — 출처:
[masterPlan](../../masterPlan.md) S9 · D27 · D29 · D28(수동 테스트는 프로토타입 인수 Task) · 자동 진행 위임.

## 범위
- 피어 노드의 인바운드 이벤트 — 페어링 요청(처음 보는 키 · 다시 추가) · 상대 승인
- 알림 설정(기본 OFF) — SQLite 저장 · 웹UI 토글 · API
- Windows 토스트(Windows 알림 시스템 — 집중 지원 · 방해 금지는 OS 가 결정, D27). 클릭하면 웹UI 피어 화면
- macOS 는 OS 알림(`osascript display notification`) 코드를 함께 작성(D25 — 확인은 Phase 4)

## 제외
- 원격 메시지 알림 — 03.agent-roundtrip 에서 같은 이벤트 경로에 더한다
- 잠금 · 절전 · 유휴 직접 감지(D27 기각안) · 알림 내 버튼(승인/거절 액션) · 알림 소리 설정
- 수동 확인(토스트 실제 표시 · 집중 지원 억제) — D28 `19.prototype-acceptance`

## 성공 기준
- A1: 처음 보는 PC 의 페어링 요청이 받아들여지거나 아는 PC 가 다시 추가하면 "페어링 요청" 이벤트가, 상대 승인이 반영되면 "승인"
  이벤트가 한 번 난다. 거절된 요청(closed · busy · stale · unknown)은 이벤트가 없다.
- A2: 알림은 기본 OFF 이고, 웹UI 에서 켜고 끈 값이 재시작 뒤에도 유지된다. 꺼져 있으면 아무것도 표시하지 않는다.
- A3: 토스트 내용의 원격 문자열(이름 · 호스트)은 XML 이스케이프된다 — 원격 입력이 토스트 구조를 바꾸지 못한다.
- A4: 켜져 있으면 Windows 토스트가 Windows 알림 시스템으로 표시되고, 클릭하면 웹UI 피어 화면이 열린다. 집중 지원 · 방해 금지 중에는
  OS 정책대로 억제된다(수동 — D28).
- A5: 루트 `PROJECT.md` 검증 명령의 테스트 · 경고 0 · 의존 경계 행이 통과한다.

## 결정
자동 진행 위임 + 사용자 확인 답(2026-10-11) — 권장안으로 정했다.

- N1 표시는 `windows` 0.62.2(이미 의존 트리에 있음)의 WinRT `ToastNotificationManager` — AUMID `agentNetwork` 를
  `HKCU\Software\Classes\AppUserModelId\agentNetwork`(DisplayName)에 등록하고 그 ID 로 보낸다. 기각: `tauri-winrt-notification`
  (같은 API 의 얇은 래퍼 — 새 크레이트) · PowerShell AUMID 빌려 쓰기(출처가 "Windows PowerShell" 로 보인다).
- N2 클릭 = 프로토콜 활성화(`activationType="protocol"`, launch = 진입 URL `/?t=…#peers`) — COM 활성화기 없이 브라우저가 연다.
  진입 토큰은 알림 센터 기록(사용자 프로필)에 남는다 — `instance.json` 과 같은 신뢰 경계.
- N3 이벤트 경로: `an_net::Node` 가 이벤트를 내고(콜백 1개), app 의 `Notifier` 가 설정을 보고 표시한다. comm 은 UI 를 모른다.
- N4 설정은 SQLite `settings` 표(스키마 3) `notify_inbound` = `0`/`1`, 기본 `0`.
- N5 레지스트리 등록은 알림을 켤 때만(꺼진 채로는 시스템에 아무것도 쓰지 않는다). 표시 실패는 로그만 — 앱 동작을 막지 않는다.

## 미해소
- 없음

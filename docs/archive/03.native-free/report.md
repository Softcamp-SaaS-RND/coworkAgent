# report — 03.native-free

종결: 2026-10-11 · 브랜치 `task/03-native-free` → main

## 결과

D30(네이티브 종속 없음 — 두 OS 같은 기능 · UI/UX)에 맞춰 02 의 WinRT 토스트 · AUMID 레지스트리 · AppleScript 알림과 00 의 메시지
상자를 걷어 냈다. 인바운드 알림 = 브라우저 알림(SSE → `Notification`) + 트레이 "새 요청" 표시, 시작 실패 = 트레이 오류 모드.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 켜면 스트림 · 트레이 건수, 끄면 없음 | 충족 | E3 · E4 |
| A2 보면 지움 · 끄면 지움 | 충족 | E3(끄기 경쟁 포함) · E4(`POST /api/seen`) |
| A3 느린 웹UI 가 수신을 막지 않음 | 충족 | E3 `slow_subscriber_does_not_block` |
| A4 시작 실패 파일 · 트레이 오류 모드 | 파일 충족 · 트레이 ABANDON | E5 · E8(D28) |
| A5 OS 전용 UI API 없음 | 충족 | E6 `native=none` |
| A6 테스트 · 경고 0 · 의존 경계 | 충족 | E1 · E2 · E7 |

- 적대적 평가 2 라운드(상한) — 지적 9건 중 코드 지적 전부 반영(건수 · 표시 순서화 → 켜짐까지 한 잠금, SSE GET 무부작용 +
  `POST /api/seen`, 쿠키 상태 변경 Origin 필수, 열기 성공 때만 소거, 트레이 생성 실패 표시, 권한 거부 안내). 미조치 4건(테스트
  공백 · 트레이 재생성)은 [eval.md](eval.md) → 원장.
- 쿠키 상태 변경 Origin 필수는 01 · 02 의 모든 쿠키 API 에도 걸린다 — #00 개정 1(쿠키 포트 비격리 수용) 위험의 상태 변경 경로를 막는다.

## 산출

- `notify`(Notifier 재작성 · SSE · `/api/seen`) · `tray`(`Tray` · `Attention` · `run_error` · `write_start_error`) · `main`(트레이 먼저,
  실패 시 오류 모드) · `server` guard(Origin 필수) · 웹UI(EventSource · Notification).
- 의존: `windows` 제거, `windows-sys` 는 `ShellExecuteW` 만, `futures-util` 직접.

## 남은 것

- 브라우저 알림 실표시 · 트레이 표시 · 오류 모드 확인 — `19.prototype-acceptance`(D28).

# report — 13.notify-settings

종결: 2026-10-11 · 브랜치 `task/13-notify-settings` → main

## 결과

자동 왕복 알림을 양쪽 PC 모두 시작 1건 + 저절로 멈춤 1건으로 묶었고(기본), 알림을 종류별로 켜고 끄며, 미리보기를 끄고, 자동 왕복 상한
기본값을 정하는 설정을 더했다(D29 보강).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 자동 왕복 시작 1 · 끝 1(기본) · 글마다 · 끄기 | 충족 · 실화면 ABANDON | E3 `relay_modes` · E5 `relay_events_start_step_end` · `relay_events_first_run_fails` · E8(D28) |
| A2 종류별 · 미리보기 끄기 · 재시작 유지 | 충족 · 실화면 ABANDON | E3 `kinds_filtered` · `preview_off` · E4 · E8 |
| A3 상한 기본값 · 범위 밖 400 | 충족 | E4 · E6 `relay_cap_default_from_settings` |
| A4 검증 명령 | 충족 | E1 · E2 · E7 |

- 적대적 평가 2 라운드(상한) — high 2 · medium 1 전부 코드 반영: 양쪽 시작 1건(`relay` 표시) · 설정 PATCH 한 트랜잭션(라운드 1), 받은 쪽
  자동 왕복 기록 · 첫 실행 실패 끝 알림(라운드 2). 미조치 2건은 [eval.md](eval.md) → 원장.

## 산출

- `an-net`: `PeerEvent::Message.kind` · `MessageKind` · `Message.relay`. `an-core`: `mark_relayed`.
- `an-agent`: 종류별 이벤트 · 자동 멈춤 끝 알림(`relay_end`) · `RELAY_CAP_KEY` · `default_relay_cap`.
- `agentnetwork-app`: `Prefs` 거르기 · `SettingsPatch` · `Notifier::update` · `GET/PUT /api/settings` 확장 · 상한 기본값, 웹UI 설정 → 알림.

## 남은 것

- 실화면 확인 — `19.prototype-acceptance` TC-I(D28).

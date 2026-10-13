# stage3 — 실측 · 매뉴얼

> 근거: [plan](../plan.md) Stage 3. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

포트 풀림 · 잡은 프로세스 표시를 실제 릴리스 앱으로 재고(A6 · A8), 1b 화면을 실행 중 앱에서 보고(A3 · A5 · A9), 매뉴얼을 맞춘다. 대상 gate:
E9 ~ E11.

## 산출 파일

- `tools/port_check.py` — 떠 있는 앱 강제 종료 → 탐색기로 릴리스 앱 시작 → (기본) 강제 종료 뒤 5초 안 웹UI · 47100 풀림 · 남은 `instance.json` ·
  다시 띄워 47100 을 새 PID 가 엶 / (`--occupy`) 47100 을 먼저 잡고 띄워 `/api/me` 의 `peer_port_fixed` · 잡은 프로세스 → 정상으로 다시 띄움.
  끝나면 앱을 띄워 둔다.
- 매뉴얼 — `an_agent`(실행 문 · 진행) · `an_net`(`Progress` · `Ping{port}` · 잡은 프로세스 · 열린 연결 수 · 주소 갱신) · `an_app`(`/api/runs` ·
  `/api/me` 포트 필드 · 메시지 `progress` · CLI 진행 줄 · 기다림 멈춤 · 툴팁) · user-guide(대기 안내 · 동시 실행 상한 · 이 PC 실행 · CLI · 포트 문제
  해결) · developer-guide(실행 문 · Ping 주소 갱신 · 포트 · `port_check.py`).

## 실측

```text
cargo build --release -p coworkagent-app                  → Finished (09:00 빌드, e695947 + 작업 트리 manual 만 다름)
python tools/port_check.py                                → freed=1 relisten=1 instance_left=1 (rc 0)
python tools/port_check.py --occupy                       → fixed=0 holder=python (rc 0)
끝난 뒤                                                   → coworkAgent.exe 1개 · 0.0.0.0:47100 LISTENING 그 PID
```

- 첫 실행은 옛 릴리스 exe(08:28 — 앱이 떠 있어 다시 링크되지 않음)로 돌아 `/api/me` 에 새 필드가 없었다 → 앱을 끄고 다시 빌드한 뒤 두 실측을 다시
  돌렸다(위 값).

E11 실화면(헤드리스 Edge — playwright-core msedge 채널 · 실행 중 앱, 스크립트는 세션 scratchpad `e11_check.mjs`):

- 내 정보: 웹UI 포트 = instance.json 의 포트 · 피어 포트 "47100(고정)" · 열린 피어 연결 "0".
- 설정 → 실행: 동시 실행 상한 입력 처음 "2" → 3 으로 바꾸면 `GET /api/runs` cap 3 → 시험 뒤 2 로 되돌림(확인).
- 대화(보낸 대화): 글 목록 응답을 가로채 첫 내 글을 `sent` + `progress{queued, 1}` 로 바꿔 그리게 함 → 메타 줄 "상대 PC 에 처리 중인 작업이 많아
  응답을 기다립니다(앞에 1건)". 실제 두 PC 대기는 `capacity` · `capacity_api` 시험이 재고, 사람 눈 확인은 19 TC.
- 실행 상세 맨 위 "이 PC 실행 — 실행 중 0 · 대기 0" · "실행 · 대기 없음".
- 콘솔 오류 · pageerror 0.
- 트레이 툴팁(피어 포트 문구 둘째 줄)은 헤드리스로 못 본다 — `tray::tip` 은 코드 경로만, 사람 확인은 19 TC.
- 화면 캡처에는 실제 상대 이름이 있어 저장소에 넣지 않았다(세션 scratchpad `e11/`).

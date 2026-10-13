# stage5 — README 개편 · 매뉴얼 · 실화면

> 근거: [plan](../plan.md) Stage 5 · 해석 11. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

README 를 사용자 관점(목적 · 주요 기능 한 줄 + 매뉴얼 링크 · 시작하기)으로 다시 쓰고 기술 절은 개발자 가이드로 옮긴다(A12). 매뉴얼에 1c 를 반영하고
실행 중 앱 화면을 확인한다. 대상 gate: E10 · E11.

## 산출 파일

- `README.md` — "왜 coworkAgent 인가"(웹 서버 없음 · AI API 비용 없음 · 페어링으로 코워크 · 대화가 서버에 쌓이지 않음) · 주요 기능 10줄(각 줄 사용자
  가이드 절 링크) · 시작하기 4단계 · 알아 둘 점 · 문서. 통신 메커니즘 · 피어 프로토콜 · 요구 사항(빌드) · 빌드 · 실행 · 사용 방법 상세 · 개발 절은 뺐다.
- `tools/check_readme.py` — 상대 링크 존재 · 기술 절 제목 없음 · 목적 4문구(자체 검사 `assert` 포함).
- 매뉴얼 — `developer-guide.md`(README 에서 옮김: 통신 구간 표 · 그림 → §5.1, 피어 프로토콜 → §5.2(`Status` · `Progress` · `retry` 추가),
  순서 그림 → §5.3, 빌드 · 실행 → §2.2, 검증 명령 → §2.3, 경로 표 → §2.1 에 합침 / 1c: 스키마 13 · 보내기 줄 · 상태 질의 · stream-json · 8.2 · 8.7 · 8.8
  "접속 상태 — 통신할 때만"), `user-guide.md`(7.2 보내는 중 · 보내지 못함 · 처리 중 단계 · 11 마지막 통신 · 14 `--id` · `wait` · `status` · 종료 7 ·
  16 문제 해결), 모듈 4(`an_core` · `an_net` · `an_agent` · `an_app` — 새 API 추가 · `watch_presence` · `PRESENCE_EVERY` · `wait_reply` 삭제),
  `manual/README.md` 라우팅 요약.

## 설계 결정

- **기능 링크는 파일 단위 + 절 번호 표기**(`[사용자 가이드 §7](…/user-guide.md)`) — 제목의 "·" 가 GitHub 앵커에서 어떻게 바뀌는지에 기대지 않는다
  (`check_readme.py` 가 파일 존재로 검사할 수 있게).
- **설치 · 빌드는 README 에서 "직접 빌드는 개발자 가이드" 한 줄** — 배포 위치(릴리스 페이지 등)는 아직 없다(확인 필요 — 배포 Task 에서).
- **`comm-query.png` 는 1c 전 흐름 그대로** — 개발자 가이드 §5.3 에 "기본 흐름만, 정본은 8.2 mermaid" 로 적었다(다시 그리기는 미정 — 사용자 결정).
- **README 화면 그림은 기존 가상 데이터 그림만** — 실행 중 앱의 화면(실제 이름)은 저장소에 넣지 않는다.

## plan 대비 편차(코드가 정본 — 매뉴얼은 코드에 맞춤)

| plan | 산출 | 판단 |
|---|---|---|
| 해석 4 — 포기 때 `mirror` 도 글 `error` + `unreachable` | `give_up` 은 `mirror` 를 로그만 | 의도 유지 — `mirror` 는 내 에이전트에게 보낸 사람 글의 미러라 글 상태는 내 실행이 정한다(1b `own_run_not_failed_by_mirror` 와 같은 규칙). 미러 실패로 내 실행 결과를 덮지 않는다 |
| 설계 `peers_api.rs` — `seen_ago` | 피어 JSON `seen_at`(유닉스 초) | 화면이 "n분 전" 을 직접 센다(stage3 설계 결정) |
| 메시지 JSON `outbox` 에 `next_at` | `{state, tries, error}` | 화면 · CLI 가 쓰지 않아 뺐다 |
| `GET /api/tx/{id}` 필드 | `key` · `stalled`(멈춘 기록 유무) 추가 | 정보 필드 추가뿐 — CLI 판정은 `state` 만 쓴다(`stalled` 상태 → 종료 7) |

## 실화면(E11)

`python tools/rebuild_run.py`(릴리스 빌드 `c58f1d1` + 작업 트리 문서만 다름 · 앱 재시작) → 헤드리스 Edge(playwright-core msedge 채널, 스크래치 스크립트)가
실행 중 앱에 붙음. 대화 글 목록 응답을 가로채 마지막 글 3개를 보관 중(`outbox{pending, tries 2}`) · 처리 중(`progress{running, detail{2턴 · Read 3 ·
Grep 1 · 41초}}`) · 보내지 못함(`error` + `refused unreachable`) 으로 바꿔 그리기 경로를 본다.

```text
레일 타일 접속 문구            → "방금 통신"(시작 알림 announce 뒤)
대화 머리글 접속 문구          → "방금 통신"
GET /api/peers                 → 모든 피어에 seen_at 키
보관 중 글 메타                → "보내는 중 — 상대에 닿지 않아 다시 보냄(2회)"
처리 중 글 메타                → "처리 중 — 2턴 · Grep 1 · Read 3 · 41초"
보내지 못한 글 메타            → "오류 · 보내지 못함 — 상대에 오래 닿지 않았다(다시 보내기로 다시 보낼 수 있다)"
콘솔 · 페이지 오류             → 0
```

## 테스트 결과

```text
python tools/check_readme.py   → readme ok
python tools/check_ddd_docs.py → PASS — DDD 문서 규약 위반 없음
```

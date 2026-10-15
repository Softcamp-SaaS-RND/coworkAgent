# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> 누적 기록은 여기 없다: Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`),
> 미조치는 [overview/ledger.md](docs/overview/ledger.md). 가드레일·검증 명령은 루트 `PROJECT.md`,
> 제품 구조는 루트 `PRODUCT.md`, 절차는 [etc/ddd-workflow.md](docs/etc/ddd-workflow.md).

- **갱신**: 2026-10-15 / main — 0.3.0 GitHub 릴리스(v0.3.0 · pre-release) · 1i squash push · README · 사용자 가이드 캡처 갱신

## 현재 작업

| 항목 | 값 |
|------|-----|
| Task | 다음 = 1h.stabilization(UI · UX · 버그 수정 안정화) — 착수 인터뷰 전 |
| 단계 | 1i 종결 직후 |
| 상태 | 1i 종결 · GitHub push(squash) · GitHub 릴리스 v0.3.0(사전 배포) |
| 재개 좌표 | [1i report](docs/archive/1i.rail-group/report.md) "남은 것" · 아래 다음 행동 1 |

- **직전에 한 일**: 1i — 레일 그룹(이 PC 에만 있는 대화 묶음, 한 대화 여러 그룹 · 그룹 타일 메뉴 4 · "대화 추가" · 대화 ⋯ "그룹" · 그룹 타일에 끌어
  놓기) · 타일 메뉴(우클릭 · 메뉴 키 · ⋯ — 타일 편집 · 이미지 지우기 · 그룹 삭제) · 손으로 놓는 자리(끌어 놓기 · Alt+↑/↓, 대화마다 순서 하나 · 새 대화
  맨 위) · 스키마 16 · 타일 아이콘 그림 CSP 결함 고침. masterPlan D37 · D35 개정. 적대적 평가 2/2 라운드 지적 8건 모두 고침.
  그 전: 외부 공격 노출 검토 문서([overview/security-review.md](docs/overview/security-review.md) — 이 PC 방화벽 규칙이 Public 프로필도 허용, 확인 필요) ·
  D36(서명 · 배포 · 자동 업데이트 → Azure DevOps TODO) · 실행 파일 속성 버전 · 1g.
- **중단 지점**: 1h 착수 인터뷰 대기. GitHub main 은 로컬 main 과 트리가 같다(squash — 실행 파일 속성 버전 · D36 · 보안 검토 · 1i · 0.3.0).
  README · 사용자 가이드 캡처(`workspace` · `home` · 새 `group`)는 가상 데이터로 그룹 레일을 다시 찍었고, 받는 곳(GitHub 릴리스) 안내를 넣었다.
- **이 PC 반영**: `task/1i-rail-group` `aaea9bf` 릴리스 빌드로 실행 중(자동 시작 = `target\release` 경로). 스키마 16(사본 `.v15.bak` · 실행 전
  `coworkAgent.db.pre-1i.bak`). 1i 화면 27/27([1i eval](docs/archive/1i.rail-group/eval.md) E7).

## 다음 행동

1. 1h 착수 인터뷰 — 범위 후보: 원장 열림(#15 R2-F2r · #17 S1-P · #1g R2-F1 · #1g R1·R2-F2) · UI 시안 차이 8건 · 1e 트레이 생성 실패 위험 · 1f
   "지운 뒤 정산" 순서 · 사용자 관찰 버그 · 방화벽 Public 프로필(보안 검토 7장).
2. 19 acceptance TC 일괄(사용자) — 1a ~ 1i TC 추가분(1i: 상대 둘 이상에서 상대 타일 끌기 · 상대별 절 순서, 그룹 여러 개 · 긴 목록 끌기 · 화면 읽기 알림).

- **블로커 / 미해소**: 1c 원장 미조치 4건(19 TC 뒤 판단). 1e 남은 것(로그인 직후 트레이 생성 실패 위험 — 19 TC). 1f 범위 밖 관찰(기존 outbox 종류의
  "지운 뒤 정산" 순서). 이전 대기 그대로 — UI 시안 차이 8건 · 매뉴얼의 미구현 코드 제안 7건 후속 여부 · `agentNetwork-t19` 워크트리 prunable.

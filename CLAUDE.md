# CLAUDE.md

저장소 권위 문서의 **부트로더**. 절차·명령·계약은 여기 적지 않는다. 프로젝트 무관 뼈대 — 고유 요소는 루트
`PROJECT.md`(일하는 방식) · `PRODUCT.md`(만든 것) · `STATE.md`(지금 상태)가 나눠 갖는다. 정본과 다르면 정본을 따른다.

## 매 세션 필독

1. `STATE.md` — 재개 좌표. **가장 먼저.**
2. `PROJECT.md` — 가드레일 · 개발 환경 · 검증 명령.
3. [etc/ddd-workflow.md](docs/etc/ddd-workflow.md) — DDD 절차 핵심(문서 체인 · 개발 Flow · 평가 계약 · Git).

## 필요할 때 여는 문서

| 언제 | 여는 문서 |
| --- | --- |
| Task 를 열거나 닫을 때 · D·S 를 인용할 때 | [masterPlan.md](docs/masterPlan.md) |
| 코드를 고치기 전 | `PRODUCT.md` 의 손댈 절 → 거기서 건 매뉴얼 절 |
| intent 를 쓸 때 | [etc/intent-protocol.md](docs/etc/intent-protocol.md) |
| plan 평가 항목을 쓸 때 · 평가할 때 | [etc/eval-protocol.md](docs/etc/eval-protocol.md) |
| 문서를 만들거나 옮길 때 · 루트 문서 절을 고칠 때 | [etc/doc-conventions.md](docs/etc/doc-conventions.md) |
| 매뉴얼을 찾을 때 | [manual/README.md](docs/manual/README.md) |
| 종결 · 미조치를 볼 때 | [overview/ledger.md](docs/overview/ledger.md) |
| Task 목록 · 상태 | `python tools/check_ddd_docs.py --tasks`(폴더가 SSOT) |

## 검증

커밋·종결 전 PASS 필수 명령의 정본은 `PROJECT.md` "검증 명령" 표. 빌드·테스트 green 과 **별개로** 문서 규약 체커
(`tools/check_ddd_docs.py`)와 gate 체커(`tools/check_gates.py`)가 통과해야 종결한다. 완료 선언에는 **명령과 출력**을 첨부한다.

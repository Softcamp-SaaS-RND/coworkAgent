# CLAUDE.md

저장소의 권위 문서를 찾기 위한 **짧은 부트로더**다. 절차·명령·계약을 여기 중복 기록하지 않는다.
프로젝트 무관 뼈대이므로 어느 프로젝트에나 그대로 복사한다 — 고유 요소는 루트 `PROJECT.md`(일하는
방식) · `PRODUCT.md`(만든 것) · `STATE.md`(지금 상태)가 나눠 갖는다.

## 로딩 순서

1. 루트 `STATE.md` — 재개 좌표(덮어쓰는 문서). **세션 시작 시 가장 먼저.**
2. 루트 `PROJECT.md` — 가드레일 · 개발 환경 · 검증 명령. **코드·문서 작업 착수 전.**
   손댈 영역의 구조·규칙은 루트 `PRODUCT.md`.
3. [etc/ddd-workflow.md](docs/etc/ddd-workflow.md) — **DDD 절차 정본. 착수 전 필독**(작업 성격과 무관하게).
4. [masterPlan.md](docs/masterPlan.md) 로드맵 → 해당 Task 폴더 `docs/tasks/{NN.기능명}/` 의 문서 체인.

이 파일과 정본이 다르면 **정본을 따른다**.

## 방법론 — DDD (Document-Driven Development / Hyper-Waterfall)

문서가 구현을 선행·구속한다. 계획을 문서로 확정(사용자 승인) → 구현은 그 문서가 계약 → 평가·보고까지
문서로. 체인은 `intent.md` → `plan.md` → `work/stage*.md` → `eval.md` → `report.md`, **각 단계 사용자
검증 후 다음 단계**. **세션·담당자가 바뀌어도 문서만으로 재개**된다.

## 주제별 정본

| 무엇을 하려면 | 여는 문서 |
| --- | --- |
| 개발 Flow · 도구 전제 · 문서 체계 · 루트 3문서 계약 · Git · 공통 원칙 | [etc/ddd-workflow.md](docs/etc/ddd-workflow.md) |
| Task 착수 인터뷰 · `intent.md` 형식 · 종결 환류 | [etc/intent-protocol.md](docs/etc/intent-protocol.md) |
| 평가 항목 gate 작성 · 적대적 평가 절차 · 임계값 | [etc/eval-protocol.md](docs/etc/eval-protocol.md) |
| Task 폴더 명 · 문서 링크 규약 | [etc/doc-conventions.md](docs/etc/doc-conventions.md) |
| 모듈 매뉴얼 라우팅 · 등록 규약 | [manual/README.md](docs/manual/README.md) |
| 전체 의도 · 확정 방향(D) · 기능 명세(S) · Phase 로드맵 | [masterPlan.md](docs/masterPlan.md) |
| 이 프로젝트의 가드레일 · 개발 환경 · 검증 명령 | 루트 `PROJECT.md` |
| 제품 구조 — 기술 스택 · 모듈 Layer · 도메인 규칙 · 모듈 매뉴얼 · 설계 문서 | 루트 `PRODUCT.md` |
| 지금 상태 — 재개 좌표 | 루트 `STATE.md` |
| Task 목록 · 상태 | 폴더가 SSOT — `python tools/check_ddd_docs.py --tasks` |
| 적대적 평가 미조치 추적 | [overview/ledger.md](docs/overview/ledger.md) |

## 검증

커밋·종결 전 PASS 필수 명령의 정본은 루트 `PROJECT.md` "검증 명령" 표다. 빌드·테스트 green 과
**별개로** 문서 규약 체커(`tools/check_ddd_docs.py`)와 gate 체커(`tools/check_gates.py`)가 통과해야
종결한다. 완료 선언에는 **명령과 출력**을 첨부한다.

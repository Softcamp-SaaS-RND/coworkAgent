# PROJECT.md — coworkAgent

**어떻게 일하는가**의 단일 위치 — 가드레일 · 개발 환경 · 검증. 제품 구조·규칙은 루트 `PRODUCT.md`,
진행 현황·미조치는 루트 `STATE.md`. 계약은 [etc/ddd-workflow.md](docs/etc/ddd-workflow.md)
"PROJECT.md — 필수 5절". **빈 절은 없다** — 실내용이 없으면 `없음`, 미확정이면 `미확정` + TODO 주석.

## 치명 가드레일

<!-- 3~7줄. 🔴 "무엇이 변하면 안 되는가" 로 쓴다 — "어느 파일을 건드리지 마라" 는 경로가 사라지는 순간
     검사도 사라진다(영원히 초록). 줄 수·자리별 키워드는 `tools/check_ddd_docs.py` 의 `GUARDRAIL_BULLETS`·
     `GUARDRAIL_KEYWORDS` 와 함께 고친다. -->
<!-- 출처: 1~5행 의도 문서 "방법"·"기능" · 6행 인터뷰 Q3, 문구는 Q21(masterPlan D22) -->

- 배포 산출물은 OS 별 **단일 실행 파일** 하나다 — 보조 실행 파일을 함께 배포하지 않는다.
- **Rust** 단일 코드베이스가 Windows·macOS 양쪽에서 빌드·동작한다 — 한 OS 에서만 되는 기능은 완료가 아니다.
- 서버·클라이언트를 **같은 실행 파일**이 겸한다 — 서버 전용·클라이언트 전용 빌드로 나누지 않는다.
- 화면은 전부 **웹UI** 로 만든다 — 트레이 메뉴 외에 네이티브 창을 두지 않는다.
- 대화·작업 이력은 사용자 데이터 경로(AppData 류) 아래 **SQLite** 에 둔다 — 실행 파일 옆·시스템 경로에 쓰지 않는다.
- 원격 메시지는 **신뢰하지 않는 입력**이다 — 에이전트에 허용하지 않은 동작으로 이어지지 않는다.

## 프로젝트 개요

- **목표**: 해당 프로세스를 실행하고 있는 PC 간 Claude 간 통신할 수 있는 툴(coworkAgent — 2026-10-11 개명, 구 agentNetwork)을 만든다.
- **성격**: 신규 — 빈 리포에서 시작. 레거시·외부 자산 없음.
- 전체 의도·범위·제외·성공 기준은 [masterPlan.md](docs/masterPlan.md) `## 의도` 가 정본(복제 금지).
- 제품이 무엇인지는 루트 `PRODUCT.md`.

## 개발 환경

- **도구 전제**(ddd-workflow 역할 표의 수단):
  - 인터뷰 — `grill-me`.
  - 의존 그래프 — `cargo tree`(크레이트 간).
  - 구조 탐색 — `graphify`(코드 · 문서를 만들기 전에 그래프로 기존 구조 · 호출 관계를 먼저 본다). 산출물 `graphify-out/` 은
    로컬 캐시라 **git 에서 관리하지 않는다**(`.gitignore`) — 리포에 그래프를 커밋하지 않고, 오래됐으면 다시 만들어 쓴다.
    **Task 종결 시**(개발 Flow 6 의 archive 이동 뒤) `/graphify --update` 로 변경분만 재추출한다 — 그래프가 없으면 생략.
    그래프가 답하지 못하는 질의만 grep · 파일 읽기로 보충한다.
  - TDD · stage 리뷰 · 완료 증거 — `superpowers:test-driven-development` · `superpowers:requesting-code-review` ·
    `superpowers:verification-before-completion`.
  - 적대적 평가자 — `/codex:adversarial-review` 를 구현 세션과 분리된 codex 리뷰 세션에 요청(세션 선택·요청
    절차는 사용자 전역 지침). **라운드 상한 2** — 사용자 전역 지침이
    [etc/eval-protocol.md](docs/etc/eval-protocol.md) 의 "최대 3 라운드" 보다 우선한다.
- **Git 브랜치**: Task 별 `task/NN-이름` 브랜치 → main 으로 merge. rebase 는 사용자 요청 시에만.
- **수동 테스트 시점**: 프로토타입(Phase 1 끝)까지는 함수 테스트 · TDD 만 — 각 Task 의 수동 gate 는 ABANDON 으로 드러내고
  프로토타입 인수 Task 에서 일괄 진행한다(masterPlan D28).
- **개발 PC 재실행**: `python tools/rebuild_run.py` — 떠 있는 coworkAgent 강제 종료 → 릴리스 빌드 → 실행(실행 파일 잠금 때문에
  끄고 빌드한다). 실행 중이던 요청은 다음 시작 때 실패 답이 되고, 빌드가 실패하면 앱은 꺼진 채로 남는다. 앱은 탐색기로
  띄워 부른 셸의 환경 변수가 앱 · `claude -p` 로 새지 않는다 — Claude Code 세션에서 앱을 띄울 때는 `Start-Process` 대신 이것을 쓴다
  (세션 환경엔 `CLAUDE_CODE_MESSAGING_TOKEN` · `CLAUDE_EFFORT` 등이 있다).

## 검증 명령

커밋·종결 전 PASS 필수. **정본은 본 표.** 🔴 쓰지 않기로 한 체커의 행은 지운다 — 영원히 "도입 예정" 인
행은 이 표를 지켜지지 않는 표로 만든다.

| 검사 | 명령 | 시점 |
|------|------|------|
| 테스트 | `cargo test --workspace` | 매 stage · 종결 |
| 경고 0 | `cargo clippy --workspace --all-targets -- -D warnings` | 매 stage · 종결 |
| 웹UI 화면 로직 | `node --test --test-reporter=tap "crates/app/web/*.test.mjs"` — Node 22+(검증 환경 25.8.0), 개발 PC 에만 필요 | 매 stage · 종결 |
| 문서 규약 | `python tools/check_ddd_docs.py` | 매 커밋 |
| 평가 항목 gate | `python tools/check_gates.py docs/tasks/{key}` | 구현 완료 시점 · 이후 코드 수정마다 |
| 의존 경계 (`check_layers.py` 대체 — 비-Python) | `python tools/check_crates.py` — `cargo metadata` 로 워크스페이스 크레이트 · 내부 간선 실측, Layer 역방향 · 미분류 크레이트를 위반으로 센다 | 매 종결 |
| 임계값 경계 (`check_thresholds.py` 대체 — 비-Python) | 경계 정확값·±1 단위 테스트(`cargo test --workspace` 안). `cargo-mutants` 도입은 첫 임계값 Task 에서 판단 | 도입 예정 — 첫 임계값 Task 부터 매 종결 |

## 참조 문서

- 루트 `PRODUCT.md` — 제품 개요 · 기술 스택 · 모듈 Layer 구조 · 도메인 규칙 · 모듈 매뉴얼
- 루트 `STATE.md` — 재개 좌표(덮어쓴다). Task 목록은 폴더가 SSOT(`check_ddd_docs.py --tasks`)
- [overview/ledger.md](docs/overview/ledger.md) — 적대적 평가 미조치 원장(누적)
- [masterPlan.md](docs/masterPlan.md) — 전체 의도 · 확정 방향(D) · 기능 명세(S) · Phase 로드맵
- [etc/ddd-workflow.md](docs/etc/ddd-workflow.md) — DDD 절차 정본(개발 Flow · 문서 체계 · 계약)
- [overview/bootstrap-interview.md](docs/overview/bootstrap-interview.md) — 부트스트랩 인터뷰(질문 · 답 · 조사 사실). 본 문서 TODO 의 `Qn` 출처
- `docs/tasks/*/intent.md` · `docs/archive/*/intent.md` — Task 별 의도. 전부 grep 하면 의도가 모인다
- `docs/archive/*/report.md` — 종결 Task 보고서 · `docs/feedback/` — 사람 작성 피드백
- 의도 문서(seed, 리포 밖) — `%APPDATA%\orca\floating-workspace\aitray.md`. 내용은 masterPlan 으로 옮겼다(정본은 masterPlan)

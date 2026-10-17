# plan-docs — 1p 문서 정리(Stage 1)

> 근거: [plan](plan.md) 해석 1 ~ 6 · 13. 기준 커밋 `ceaaa6f`. 크기는 git 의 LF 바이트(`design/gates.py size`).
> **옮김 = 대상 절에 없는 사실만 옮기고 원래 자리에서 지운다**(복사 금지). 사실이 남았는지는 E2(`gates.py moved ceaaa6f`)가 지운 줄의
> 코드 이름 · 단위 붙은 수를 살아 있는 문서(archive · tasks 밖)에서 찾아 기계로 본다.

## 1. 크기 — 지금 → 목표

| 문서 | 지금 | 목표 | 어떻게 |
|------|------|------|--------|
| 루트 `CLAUDE.md` | 3.0KB | ≈ 2.0KB | "매 세션 필독" 셋 + "필요할 때 여는 문서" 표(2 절) |
| 루트 `STATE.md` | 2.6KB | ≈ 1.8KB | 낡은 블로커를 제자리로(8 절) |
| 루트 `PROJECT.md` | 6.9KB | ≈ 5.5KB | 5 절 |
| [ddd-workflow](../../etc/ddd-workflow.md) | 14.8KB | ≈ 9.4KB | 3 절 — 문서 체계 · PROJECT/PRODUCT 계약 · 원장 계약을 doc-conventions 로 |
| **필독 4문서 합** | **27.2KB**(체인 전체 60.1KB) | **≤ 20KB**(E1) | masterPlan · PRODUCT 는 필독에서 빠진다 |
| 루트 `PRODUCT.md` | 31.7KB | ≤ 15KB(E1) — 어림 12KB | 6 절 |

## 2. 루트 CLAUDE.md — 새 모양(전문)

프로젝트 무관 뼈대 그대로(이 프로젝트 고유 문서 이름을 넣지 않는다). E1 이 `## 매 세션 필독` 절의 문서 집합을 기계로 본다.

```markdown
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
```

- 지금 `CLAUDE.md` 의 "방법론 — DDD" 단락은 ddd-workflow `## 문서 체인` 과 같은 말이라 지운다("Hyper-Waterfall" 이름 · "문서만으로 재개" 는
  ddd-workflow 머리로).

## 3. ddd-workflow 쪼개기

| 절 | 크기 | 처리 |
|----|------|------|
| 머리 | 0.3KB | 남김 — "매 세션 필독 절차 핵심, 문서 구조 계약은 doc-conventions" 한 줄 보탬 |
| `## 문서 체인` | 1.3KB | 남김 |
| `## 도구 전제` | 1.2KB | 남김(intent · eval-protocol 이 이 이름으로 건다) |
| `## 적대적 평가자` | 1.2KB | 남김(eval-protocol 규약 3 이 이 이름으로 건다) |
| `## 개발 Flow` | 2.9KB | 남김 — 7 의 "문서 체계 트리" 는 doc-conventions 링크로 |
| `## 문서 체계` | 2.5KB | → doc-conventions `## 문서 체계` |
| `## 루트 3문서 계약` 머리 · 표 | 0.9KB | → doc-conventions |
| `### PROJECT.md — 필수 5절` | 0.8KB | → doc-conventions(+ 루트 `PROJECT.md` 가드레일 위 주석의 작성 규칙 — 줄 수 · 키워드 · 체커 연동) |
| `### PRODUCT.md — 필수 6절` | 0.8KB | → doc-conventions |
| `### STATE.md — 필수 2절` | 1.2KB | **남김** — 단계 전환마다 쓴다(개발 Flow 8 아래로) |
| `### 미조치 원장` | 0.5KB | → doc-conventions |
| `## Git / 커밋 규칙` · `## 공통 원칙` | 1.3KB | 남김 |

doc-conventions 머리는 "문서를 만들거나 옮길 때 · 루트 문서(PROJECT · PRODUCT) 절이나 원장을 고칠 때 여는 문서" 로 바꾼다.

## 4. 바깥 참조 — 같은 커밋에서 고친다(결정 3 — 체커 동시 개정)

- 루트 `CLAUDE.md` 27행(주제별 정본 표 — 2 절 표로 대체).
- 루트 `PROJECT.md` 머리("PROJECT.md — 필수 5절") · 루트 `PRODUCT.md` 머리("PRODUCT.md — 필수 6절") → doc-conventions.
- [doc-conventions](../../etc/doc-conventions.md) 머리("문서 체계" → 같은 문서 안).
- [manual/README](../../manual/README.md) "먼저 읽는다" 줄.
- `tools/check_ddd_docs.py` — docstring 1행 · 상수 주석 125 · 134 · 169 · 오류 문구 820 ~ 865 의 출처 문서 이름(STATE 2절은 ddd-workflow 그대로).
- 이름이 그대로라 고치지 않는 곳: intent-protocol · eval-protocol 의 "도구 전제" · "적대적 평가자" · "개발 Flow" — ledger 의 "개발 Flow 6".

## 5. 루트 PROJECT.md 줄이기

- 가드레일 위 작성 규칙 주석(3 ~ 7줄 · 체커 연동) → doc-conventions PROJECT 5절. 출처 주석 한 줄은 남긴다. 🔴 가드레일 6줄 본문은 그대로(체커가
  줄 수 · 자리별 키워드를 잡는다).
- 개발 환경: graphify 줄을 두 줄로(쓰임 · `graphify-out/` 은 git 밖 · 종결 때 `/graphify --update`). `tools/rebuild_run.py` 줄은 쓰는 법 한 줄 +
  세부(실행 중 요청 · 빌드 실패 · 환경 변수 `CLAUDE_CODE_MESSAGING_TOKEN` · `CLAUDE_EFFORT` 가 새지 않는 이유)는 [developer-guide](../../manual/dev/developer-guide.md) 2.2 로.
- 참조 문서: `CLAUDE.md` 표와 겹치는 줄(PRODUCT · STATE · ledger · masterPlan · ddd-workflow)을 지우고, 거기 없는 것(부트스트랩 인터뷰 · intent 모으기 ·
  report · feedback · 의도 seed 위치)만 남긴다.

## 6. 루트 PRODUCT.md `## 도메인 규칙` 처리

남는 줄은 **지켜야 하는 규칙 한 줄 + 정본 절 링크**. 흐름은 [developer-guide](../../manual/dev/developer-guide.md), 화면은 새 web-ui.md(7 절).

| # | 블록(크기) | 처리 | 옮길 곳 |
|---|------------|------|---------|
| 1 | 신원(0.3KB) | 남김 | — |
| 2 | 페어링(0.8KB) | 줄여 남김 — 위험 수용(5분 · 1건 · LAN MITM) 문장 유지 | 흐름은 개발자 가이드 8.1 |
| 3 | 두 OS 같은 UI(0.2KB) | 남김 | — |
| 4 | 인바운드 알림(0.8KB) | 한 줄 | web-ui.md 알림 |
| 5 | 실행 구조 · 자동 시작(1.1KB) | 한 줄 | 개발자 가이드 3.1 |
| 6 | 시작 실패(0.4KB) | 한 줄 | 개발자 가이드 3.1 · 10 |
| 7 | 웹UI 구조(1.5KB) | 한 줄 | web-ui.md 화면 구조 |
| 8 | 레일 그룹 · 손으로 놓는 자리(1.5KB) | 한 줄(이 PC 에만 · 상대에 안 보냄) | 화면 → web-ui.md · 저장(`group_convs` · `group_peers` · 스키마 18) → 개발자 가이드 7.2 |
| 9 | 안정화 #1h(0.6KB) | 지움 | 화면 → web-ui.md · `--resume` 사라짐 → 6.6 · `peers.id` → 7.2 · 트레이 다시 시도 → 4.1 |
| 10 | 메신저형 화면(1.4KB) | 지움 | web-ui.md |
| 11 | 다듬기 #1k(0.8KB) | 지움 | web-ui.md |
| 12 | 아이콘 · 설정 정리 #1l(0.7KB) | 지움 | web-ui.md |
| 13 | 목록 나눔 · 음영 · 공유 대화(1.4KB) | 한 줄(공유 = 읽기 전용 · 5명 · 24시간) | 화면 → web-ui.md · 통신 → 8.11 |
| 14 | 컨텍스트(0.6KB) | 한 줄 | 화면 → web-ui.md · 세션 → 6.3 |
| 15 | 집중 · 접속 상태(1.1KB) | 한 줄 | 8.5 · 8.8 |
| 16 | 질의 · 요청 목록(1.0KB) | 한 줄(요청 = 실행 없음) | 8.6 |
| 17 | 이력 정리(0.7KB) | 한 줄(끝난 내 사람 글만) | 8.9 |
| 18 | @멘션(1.2KB) | 한 줄 | 8.4 |
| 19 ~ 22 | 로컬 웹UI 인증 · 데이터 폴더 권한 · 원격 실패 · 원격 메시지(1.2KB) | 남김(보안) | — |
| 23 | Claude 실행(1.2KB) | 줄여 남김 — 읽기 전용 5도구 · `dontAsk` · 표준 입력만 · 30분 · 모델 순서 | 세부 → 6.5 |
| 24 | 실행 한도(0.9KB) | 한 줄 | 3.2 |
| 25 | 피어 주소(0.4KB) | 한 줄 | 5.1 |
| 26 | 보내기 보장(0.9KB) | 한 줄 | 6.6 |
| 27 | 대화 왕복(0.6KB) | 한 줄 | 8.2 |
| 28 ~ 30 | 문의 응답 전용 · Claude 자격 증명 · 실행 파라미터(0.8KB) | 남김 | — |
| 31 | AI 에이전트용 CLI(1.1KB) | 한 줄(종료 코드 표는 정본 한 곳) | 8.7 |
| 32 | 부모 도구 메뉴(0.8KB) | 한 줄 | 8.10 |
| 33 | 양쪽 발화(0.5KB) | 줄여 남김 | — |
| 34 | 자동 왕복(0.7KB) | 한 줄 | 8.3 |
| 35 | 스키마 올리기(0.3KB) | 한 줄 | 7.5 |
| 36 | 저장(0.5KB) | 남김(이 PC 에만 · 양쪽에 남는 것) | — |

- `## 설계 문서` 에 web-ui.md 한 줄을 보탠다. `## 제품 개요` · `## 기술 스택` · `## 모듈 Layer 구조` · 모듈 표는 그대로.

## 7. docs/manual/dev/web-ui.md — 새 lazy 문서(골격)

모듈 매뉴얼 형식의 예외(서두 명시 — [manual/README](../../manual/README.md) 등록 규약). 화면(`crates/app/web/`)을 고칠 때만 연다.

- `## 화면 구조` — PRODUCT 7 · 8 · 9 · 10 · 11 · 12 · 13 · 14 의 화면 규칙(Stage 1).
- `## 알림` — PRODUCT 4(Stage 1).
- `## 디자인 토큰` — Nocturne 값 · `:root` 밖 리터럴 0 · mono 허용 목록 · `css_tokens` 시험(Stage 3).
- `## 문구 규칙` — R1 ~ R7(Stage 4, [plan-copy](plan-copy.md)).
- `## 용어 표` — `| 개념 | 쓰는 말 | 쓰지 않는 말 |`, T1 ~ T7 확정분(Stage 4 — E7 이 읽는다).
- [manual/README](../../manual/README.md) `dev/` 목록 · 루트 `PRODUCT.md` `## 설계 문서` 에 링크.

## 8. 루트 STATE.md 낡은 항목 — 지우지 않고 제자리로

| 항목 | 사실 | 처리 |
|------|------|------|
| 1c 원장 미조치 4건 | [ledger](../../overview/ledger.md) `#1c` 행에 있다 | STATE 에서 지움(원장이 SSOT) |
| #1g R2-F1 · R1·R2-F2 | 원장 `열림` 행에 있다 | STATE 에서 지움 |
| 1f 범위 밖 관찰("지운 뒤 정산" 순서) | 1f report · eval 에만 있다 | 원장에 `#1f` 행(`열림`)으로 올리고 STATE 에서 지움 |
| 캡처 일부 1j 이전 | 이 Task Stage 4 가 다시 찍는다 | Stage 4 뒤 지움 |
| UI 시안 차이 8건 | 2026-10-12(#17 · #18 무렵) 사용자 결정 대기 — 원본 목록이 문서에 없고 그 뒤 1j · 1k · 1l · 1o 가 화면을 다시 짰다 | 폐기 — 사유를 stage1.md 에(사용자 확인 대상) |
| 매뉴얼 미구현 코드 제안 7건 | 2026-10-12 매뉴얼 리뷰 무렵 — 원본 목록이 문서에 없다. 확인되는 것: resume 실패 시 새 세션 = #1h 구현, LAN 검사 없음 = 매뉴얼에 한계로 기록 | 폐기 — 사유를 stage1.md 에(사용자 확인 대상) |
| `agentNetwork-t19` 워크트리 prunable | `git worktree list` 에 없다 | 지움 |

## 9. 보관 · 삭제 후보(사용자 결정 — 권장: 전부 유지)

필독이 아니라 맥락 비용이 없고, archive · 매뉴얼이 링크로 걸고 있어 지우면 이력이 끊긴다.

| 문서 | 크기 | 거는 곳 | 권장 |
|------|------|---------|------|
| [overview/bootstrap-interview.md](../../overview/bootstrap-interview.md) | 30.6KB | PROJECT · masterPlan · archive 14 | 유지 — 가드레일 · D 의 Q 번호 출처 |
| [overview/security-review.md](../../overview/security-review.md) | 12.3KB | 개발자 · 사용자 가이드 | 유지 |
| [overview/ui-mockups.md](../../overview/ui-mockups.md) | 3.3KB | masterPlan · archive 10 · 15 | 유지 — #10 시안 선택 기록 |
| [feedback/2026-10-12-improve-coworker.md](../../feedback/2026-10-12-improve-coworker.md) | 3.0KB | archive 15 · 16 | 유지 — 사람 작성 피드백(이번 "폰트/크기 · 톤/매너" 요청의 앞선 출처) |
| `docs/images` 참조 없는 캡처 4장(activity · new-conversation · relay · search) | 1.0MB | 없음 | **지움**(Stage 4 — E9) |
| `.remember/today-*.done.md` 6개 | 94KB | 플러그인 기록 · 주입 안 됨 | 범위 밖(결정 2) |

## 10. 로컬 삭제(결정 9ⓐ — 리포 밖 · 되돌릴 수 없음)

- `target-t12/`(9.0GB, `.git/info/exclude`) — `.cargo/config.toml` · `tools/` 에 참조 없음(확인함). 빈 `.claude/worktrees/agent-abd1086a96c70114e/`.
- `arapaima/`(잠긴 워크트리 — GitHub squash 반영용)는 건드리지 않는다.

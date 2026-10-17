# stage1 — 필독 문서 줄이기 · lazy 이관 · STATE 정리 · 로컬 캐시

> 근거: [plan](../plan.md) Stage 1(해석 1 ~ 7 · 17) · [plan-docs](../plan-docs.md) 2 ~ 10.

## 목표

매 세션 필독을 루트 `CLAUDE.md` · `STATE.md` · `PROJECT.md` · ddd-workflow 로 줄이고(≤ 20KB), 나머지는 "필요할 때 여는 문서" 로 옮긴다.
`PRODUCT.md` 는 지켜야 하는 규칙 한 줄 + 정본 절로(≤ 15KB). 옮긴 사실은 살아 있는 문서에 남긴다. 대상 gate: E1 · E2 · E9(일부) · E12.

## 산출 파일

- 루트 `CLAUDE.md` — `## 매 세션 필독`(STATE · PROJECT · ddd-workflow) · `## 필요할 때 여는 문서` 표 · `## 검증`. "방법론 — DDD" 단락은 ddd-workflow 머리로.
- [ddd-workflow](../../../etc/ddd-workflow.md) — 핵심만(문서 체인 · 도구 전제 · 적대적 평가자 · 개발 Flow + `STATE.md` 2절 · Git · 공통 원칙).
- [doc-conventions](../../../etc/doc-conventions.md) — 문서 체계 · 루트 3문서 계약(PROJECT 5절 + 가드레일 작성 규칙 · PRODUCT 6절) · 미조치 원장 계약을 받음.
- 루트 `PROJECT.md` — 가드레일 위 작성 규칙 주석 → doc-conventions, 개발 환경 줄임(`rebuild_run.py` 세부 → 개발자 가이드 2.2), 참조 문서는
  `CLAUDE.md` 표에 없는 것만.
- 루트 `PRODUCT.md` — `## 도메인 규칙` 을 신원 · 보안 / Claude 실행 / 대화 · 통신 / 실행 · 배포 / 저장 다섯 절로. 불변 · 보안 블록(신원 · 페어링 ·
  로컬 웹UI 인증 · 데이터 폴더 권한 · 원격 실패 · 원격 메시지 · 두 OS 같은 UI · Claude 실행 · 문의 응답 전용 · Claude 자격 증명 · 실행 파라미터 ·
  양쪽 발화 · 저장)은 원문 그대로, 흐름 블록은 한 줄 + 개발자 가이드 절. 머리 계약 링크 → doc-conventions, `## 설계 문서` 에 web-ui.md.
- [개발자 가이드](../../../manual/dev/developer-guide.md) — 흐름 블록 원문을 절 끝으로 옮김: 3.1(실행 구조 · 자동 시작 · 시작 실패) · 3.2(실행 한도) ·
  4.1(트레이 다시 시도) · 5.1(피어 주소) · 6.3(컨텍스트 규칙) · 6.4(대화 왕복) · 6.6(보내기 보장 · 사라진 세션) · 7.2(레일 그룹 저장 · 상대 번호) ·
  7.5(스키마 올리기) · 8.3(자동 왕복) · 8.4(@멘션 + `messages.agent`) · 8.5(집중 · 접속) · 8.6(질의 · 요청) · 8.7(CLI + `via = 'cli'`) · 8.9(이력 정리) ·
  8.10(부모 도구 메뉴) · 8.11(공유 대화 규칙) · 2.2(`rebuild_run.py`).
- [web-ui.md](../../../manual/dev/web-ui.md)(새) — 화면 구조 · 레일 그룹 화면 · 공유 대화 화면 · 컨텍스트 · 접속 표시 · 알림 · 색 · 글꼴 · 크기.
  [manual/README](../../../manual/README.md) 에 등록, "먼저 읽는다" 줄을 `CLAUDE.md` 기준으로.
- `tools/check_ddd_docs.py` — 옮긴 계약의 출처 문서 이름(docstring · 상수 주석 · 오류 문구 — PROJECT 5절 · PRODUCT 6절 · 원장 · 가드레일 → doc-conventions).
  검사 로직은 그대로.
- 루트 `STATE.md` — 낡은 항목 정리(아래). [masterPlan](../../../masterPlan.md) Phase 1 오버플로 후보에 "outbox 정산 순서 점검" 한 줄.
- 로컬(리포 밖): `target-t12/` 9.0GB 삭제.

## 설계 결정

- **옮김 = 복사 아님**: 흐름 블록은 원문 그대로 개발자 가이드 절 끝에 붙이고 `PRODUCT.md` 에서는 한 줄 규칙만 남겼다. 한글 규칙 문장은 E2(백틱 ·
  단위 붙은 수만 봄)가 잡지 못해, 문장 단위로 옮겼다 — 개발자 가이드가 77.6KB → 90.0KB 로 커졌지만 필요할 때 그 절만 연다.
- **STATE 2절은 ddd-workflow 에 남김**: 단계 전환마다 쓰는 규칙이라 필독 쪽. PROJECT · PRODUCT 계약 · 원장 계약 · 문서 트리는 doc-conventions.
- **plan-docs 8 편차 — 1f 관찰은 원장이 아니라 masterPlan 후보로**: 원장 머리 규약이 "'라운드가 다루지 않은 영역'·'후속 안건' 은 지적이 아니다(넣으면
  원장이 백로그가 된다)" 라서, 1f 평가의 범위 밖 관찰("행을 지운 뒤 정산" 순서)은 masterPlan Phase 1 오버플로 "후보" 줄로 옮겼다.
- **STATE 낡은 항목 처리**:
  - 1c 원장 미조치 4건 · #1g R2-F1 · R1·R2-F2 — 원장에 행이 있어 STATE 에서만 지움.
  - `agentNetwork-t19` 워크트리 — `git worktree list` 에 없음, 지움.
  - **폐기(해석 7 — 사용자 승인 2026-10-17)**: "UI 시안 차이 8건(원본 목록 확인 필요)" — 2026-10-12(#17 · #18 무렵) 결정 대기, 원본 목록이 문서에 없고
    그 뒤 1j · 1k · 1l · 1o 가 화면을 다시 짰다. "매뉴얼의 미구현 코드 제안 7건" — 같은 날 매뉴얼 리뷰 무렵, 목록이 문서에 없다. 확인되는 것: resume
    실패 시 새 세션 = #1h 구현(개발자 가이드 6.6), LAN 검사 없음 = 매뉴얼에 한계로 기록(`5b491ca`).
  - 캡처 일부 1j 이전 — Stage 4 가 다시 찍거나 지운 뒤 지운다.
- **빈 `.claude/worktrees/agent-abd1086a96c70114e/` 는 못 지움**: `rmdir` → "Device or resource busy" — 다른 프로세스가 작업 폴더로 쥐고 있다(다른 세션
  추정). 강제로 지우지 않고 종결 전 다시 시도한다(E9).
- **하위 에이전트 워크트리 사고(되돌림)**: Stage 2 를 격리 워크트리 하위 에이전트로 병행하려 했으나, 그 워크트리가 `origin/main`(GitHub squash 줄기
  `2a9b059`)에서 떠 1p 폴더가 없었다 — 즉시 멈췄고 변경 없이 정리됐다. 로컬 이력과 squash 줄기는 섞지 않는다 — Stage 2 는 이 브랜치에서 직접 한다.

## 테스트 결과

```text
python docs/tasks/1p.project-tidy/design/gates.py size
files=4 bytes=19115 under20k=True must_read_ok=True product=15066 under15k=True
python docs/tasks/1p.project-tidy/design/gates.py moved ceaaa6f
targets=167 missing=0
python docs/tasks/1p.project-tidy/design/gates.py extra ceaaa6f
t12=False wt=True state_stale=0 shots=5 stale_shots=5 orphans=4      ← wt · 캡처는 종결 전 / Stage 4
python tools/check_ddd_docs.py   → PASS — DDD 문서 규약 위반 없음
python tools/check_readme.py     → readme ok
```

| 문서 | 전(LF 바이트) | 후 |
|------|------|------|
| 루트 `CLAUDE.md` | 2,957 | 1,713 |
| 루트 `STATE.md` | 2,227 | 1,791 |
| 루트 `PROJECT.md` | 6,910 | 5,757 |
| ddd-workflow | 14,834 | 9,854 |
| **필독 합** | **27,173**(masterPlan 포함 체인 60.1KB) | **19,115** |
| 루트 `PRODUCT.md` | 31,673 | 15,066 |

## 다음 단계

- ▶ 다음: tasks/1p.project-tidy/work/stage2.md — `notice_for` 삭제 · `db_err` 한 곳 · `an-agent` `serde` · `ux_literals.rs` 표.

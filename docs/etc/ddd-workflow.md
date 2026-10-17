# DDD Workflow — 절차 정본(핵심)

> 루트 `CLAUDE.md` 가 **매 세션 필독**으로 거는 절차 핵심 — DDD(Document-Driven Development · Hyper-Waterfall: 문서가 구현을
> 선행·구속하고, 세션·담당자가 바뀌어도 문서만으로 재개한다). 프로젝트 무관 뼈대다 — 고유 요소는 루트 3문서가 나눠 갖는다:
> 일하는 방식은 `PROJECT.md`, 만든 것은 `PRODUCT.md`, 지금 상태는 `STATE.md`. 여기에 프로젝트 고유 내용을 쓰지 않는다.
> 문서 트리 · 루트 3문서 계약(PROJECT · PRODUCT 절) · 원장 계약 · Task 폴더 명 · 링크는 [doc-conventions.md](doc-conventions.md) —
> 문서를 만들거나 옮길 때 연다.

## 문서 체인

`intent.md`(**인터뷰로 확정한 의도**) → `plan.md`(**+ 평가 항목**) → `work/stage*.md` →
**`eval.md`(적대적 평가)** → `report.md`. 전부 `docs/tasks/{NN.기능명}/` **한 폴더**.

- 의도(무엇·왜)와 방법(어떻게)을 가른다 — **의도가 확정되기 전에 방법을 쓰지 않는다.**
- **역할 간 인계도 문서 경로로.** 맥락을 복사하지 않고 Task 문서 경로를 준다(*읽고 → 네 몫을 하고 →
  결과를 거기 append → 요약만 반환*). 복사한 맥락은 반드시 원본과 어긋난다.
- **진행 보고는 문서명 기준** — 단계 번호는 Task 마다 의미가 달라 좌표가 못 된다.

  ```text
  ✅ 완료: tasks/{NN.기능명}/work/stage1.md  (한 줄 요약)
  ▶ 다음: tasks/{NN.기능명}/work/stage2.md  (한 줄 요약)
  ```

- **코드 사실 우선.** 계획서에는 호출 그래프·로그·실 출력으로 확인된 흐름만 — 의심되는 흐름은
  입증한다. 코드가 있으면 plan 전에 의존 그래프를 실측한다.
- **테스트 동반 개발.** 모듈 추가·변경 시 동급 테스트 동시 추가, 녹색 유지, **우리 코드 경고 0**
  (외부 의존 경고는 루트 `PRODUCT.md` 기술 스택 "테스트" 에 baseline 고정).

## 도구 전제 — 이름이 아니라 **역할**이 계약이다

이름을 부르는 보조 도구는 선택이다. 없으면 같은 역할을 다른 수단으로 하고, 계약(분리·증거·재현)은
그대로다. 쓰는 수단은 루트 `PROJECT.md` "개발 환경" 에 한 번 적어 고정한다.

| 이름(예) | 역할 | 없을 때 |
|-----------|------|---------|
| `grill-me` | Task 착수 **인터뷰** — 의도 확정 | 직접 묻는다 — 규약은 [intent-protocol.md](intent-protocol.md) |
| `graphify` | 의존 그래프 실측 | 언어의 정적 분석 도구. 없으면 의존 훑기 결과를 `docs/tech/` 에 |
| `superpowers:test-driven-development` | RED → GREEN → REFACTOR | 절차를 그대로 수행(먼저 실패를 눈으로 본다) |
| `superpowers:requesting-code-review` · `verification-before-completion` | stage 리뷰 · 완료 증거 | 리뷰어를 지정하고, 완료 선언에 **명령·출력** 첨부 |
| `/codex:adversarial-review` | **구현 세션과 분리된** 적대적 평가자 | 다른 모델·세션·사람. **분리**만 지키면 된다 |

🔴 도구 이름을 지운 자리를 비워 두는 것만 금지한다. 역할이 빠지면 계약이 사라진다.

## 적대적 평가자 (Adversarial Evaluator)

**목적**: 구현자가 기능을 자의적으로 축소·생략하지 않고 **명세 전부를 구현**하게 만든다.
**테스트 녹색은 완전성의 증거가 아니다.** 계약 4줄 — 실행 세부는 [eval-protocol.md](eval-protocol.md).

1. **평가 항목은 plan 단계에서 고정.** `plan.md` 에 **`## 평가 항목`** 절 필수, **gate 형식**
   (`- [ ] E1: …` + `CHECK:`/`EXPECT:`/`EVIDENCE:`). EXPECT 가 **정확값**, CHECK 가 **판정 방법**.
2. **평가 주체 분리.** 구현 완료 시점에 `tools/check_gates.py` 로 gate 를 기계 판정해 `eval.md`
   **`## 평가 원장`** 을 채운 뒤, **구현 세션이 아닌 평가자**가 본다. **박스는 명령이 뒤집는다** —
   손으로 체크한 박스는 증거가 아니다.
3. **항목 변경은 plan 개정 + 사용자 승인으로만.** 개정 사실을 `eval.md` 에 적어 drift 를 드러낸다.
   `CHECK` 는 보정 가능, **EXPECT 변경은 항목 변경**이다.
4. **미조치는 원장으로.** 종결 시 [overview/ledger.md](../overview/ledger.md) 에 *반박·조건부·
   닫힘도 함께* 올린다. 흩어 두면 다음 Task 가 조용히 갚아도 모른다.

## 개발 Flow

0. **세션 시작**: 루트 `STATE.md` 의 좌표에서 재개한다.
1. **착수 전** 루트 `PROJECT.md`(가드레일·검증 명령)와 손댈 영역의 `PRODUCT.md` 절을 읽는다. 순서는
   [masterPlan.md](../masterPlan.md) Phase 로드맵.
2. **Task 개설 → 의도 확정**: `docs/tasks/{NN.기능명}/`(번호 규약은
   [doc-conventions.md](doc-conventions.md)) + `intent.md`. **목록 표에 행을 만들지 않는다** —
   폴더가 Task 의 SSOT 다(목록은 `python tools/check_ddd_docs.py --tasks`).
   사람이 seed 를 쓰고 AI 가 **인터뷰**로 확정한다 — 사실은 AI 가 찾고 **결정만 사용자에게** 묻는다.
   **사용자의 "공유 이해" 확인 = intent 승인**(`승인: {날짜}`). 규약·골격은
   [intent-protocol.md](intent-protocol.md).
3. **plan 작성**(AI): **`## 평가 항목`** 절이 intent 의 **성공 기준을 전부 덮는다**. plan 은
   **자족적**으로 — 구현자가 intent 를 다시 열지 않아도 되게(구현 세션에는 plan 경로만 건넨다).
   작성 규칙은 [eval-protocol.md](eval-protocol.md), 형식 확인은 gate 명령에 `--list`. 부속 자료는
   `plan-{X}.md`. **사용자 검토·승인 후 다음 단계.**
4. **stage 별 구현 + work 기록**(AI): `work/stage{M}.md` — 필수 섹션 **목표 / 산출 파일 / 설계 결정 /
   테스트 결과 / 다음 단계**, stage 완료 커밋에 **반드시** 동봉. 구현은 **TDD**. stage 완료 시
   **리뷰를 받고**, 완료 선언은 **증거(명령·출력)와 함께**. 매 stage 사용자 검증.
5. **적대적 평가**(필수, 구현 완료 시점): `eval.md`. **먼저** gate 를 기계 판정해 **`## 평가 원장`** 을
   채우고(미충족이 남으면 그것부터), 그 뒤 **분리된 평가자**가 intent·plan 과 대조한다. 실 환경 검증·
   회귀 시나리오는 수동 gate 의 EVIDENCE 또는 `work/stage*.md` 테스트 결과에 둔다 — 별도 문서를
   만들지 않는다.
6. **종결**: `report.md` → 손댄 모듈의 `docs/manual/` 등록·갱신([manual/README.md](../manual/README.md))
   → [overview/ledger.md](../overview/ledger.md) 미조치 갱신 · `PRODUCT.md` 모듈 표·layer 표 갱신
   (상태 `✅` 는 archive 이동이 곧 기록이다 — 표에 적지 않는다) → intent 의 결정·명세 변경분을
   [masterPlan.md](../masterPlan.md) 로 **환류**(링크가 아니라 내용을 옮긴다 — intent-protocol 5) →
   검증 명령 표에서 **시점이 온 전 항목 PASS** → `git mv docs/tasks/{key} docs/archive/{key}` +
   인바운드 링크 갱신.
7. **횡단 산출물**(매뉴얼·보고서·기술 노트·장애 분석·피드백)은 [doc-conventions.md](doc-conventions.md) "문서 체계" 트리의 제자리에 둔다.
8. **단계 전환마다 루트 `STATE.md` 갱신**(시점·형식은 아래 "STATE.md — 필수 2절").

### STATE.md — 필수 2절

**작업 상태의 단일 좌표.** `STATE.md` → 루트 `CLAUDE.md` · `PROJECT.md`(+ 손댈 영역의 `PRODUCT.md`) ·
해당 Task 폴더만 읽으면 재개 가능해야 한다. 체커가 heading 존재 + 본문 비어 있지 않음을 검사한다.

- `## 현재 작업` — Task·단계·상태·**재개 좌표(열어야 할 문서 경로)** + 직전에 한 일 + 중단 지점.
  진행 중인 Task 가 없으면 `없음` 이라고 적는다(빈 본문 금지).
- `## 다음 행동` — 다음 한 걸음(실행할 명령 또는 열 문서) + 블로커 / 미해소.

**갱신 시점(필수)**: 문서 체인 단계가 전환될 때마다(intent → plan → 각 stage → eval → report → 종결)
갱신하고 **그 단계 커밋에 동봉**한다. **승인 대기·블로커로 멈출 때도.** 갱신 줄에 **갱신 시각·커밋·
브랜치**를 적는다. 🔴 **전부 덮어쓴다** — 누적 기록을 여기 두지 않는다: 경과는 git log 와 문서 체인에,
Task 목록은 폴더에, 미조치는 원장 문서에 있다. `PROJECT.md` 검증 명령·`PRODUCT.md` 모듈 표도 복제 금지.

## Git / 커밋 규칙

- **브랜치 전략은 프로젝트 결정.** 루트 `PROJECT.md` "개발 환경" 에 기록·유지.
- 커밋 메시지: `Task #{NN} Stage {M}: {설명}` / `Task #{NN}: {설명}` / 최종 `Task #{NN}: 최종 보고서`.
  `docs:`·`chore:`·`fix:` 접두어 활용.
- **history 는 merge 로 합친다.** non-fast-forward 거부 시 `git pull --no-rebase` 후 push.
  rebase 는 사용자가 명시적으로 요청할 때만.
- **커밋·push 는 항상 검증을 통과시켜 수행한다**: 훅 통과(`--no-verify` 없이) · 서명 유지 ·
  `main`/`master` 는 fast-forward push 만 · 시크릿·서명 키는 리포 밖 · 비대화형 명령.

## 공통 원칙

- **외부·레거시 자산은 검증 후에만 도입.** AST·실행으로 입증한 흐름만, 프로젝트 코드로 다시 써서
  포팅한다.
- **시크릿·서명 키·접속 정보는 env 또는 분리 파일(gitignored)로만.**
- **설정은 단일 지점에서만 해석.** 같은 정보는 한 파일에만, 새 노브는 **실제로 읽는 코드까지 배선**
  한다(死 노브 0).
- **실패는 드러낸다.** 설정 오타·미지 키·배선 실패는 warning 이상으로 노출하고, 예외를 삼킬 때는
  근거를 코드에 남긴다(`except: pass` 위장 대신).

# 문서 규약 — Task 폴더 명 · 링크

> **Task 폴더나 문서 링크를 새로 만들 때만 여는 문서.** 문서 트리는 [ddd-workflow.md](ddd-workflow.md)
> "문서 체계", 종결 절차는 그 문서 "개발 Flow" 6. 여기는 그 세부 규약이며 `tools/check_ddd_docs.py` 가 기계로 잡는다.

## Task 폴더 명

- canonical key = `{NN}.{기능명}`. NN 버퍼는 [masterPlan.md](../masterPlan.md) 기준(Phase 0 = `00~09`,
  Phase 1 = `10~19`, …). 번호는 Task 의 **주소**다 — 같은 번호 둘은 위반(`24` 와 `24-2` 는 다른 주소,
  `-n` 은 같은 번호의 하위 트랙).
- 🔴 **앞자리 0 은 주소가 아니다** — `#00`·`#0`·`000` 은 **같은 번호**다(체커가 정규화한다). 자리수를
  늘려 새 주소를 만들 수 없다. `000.무엇` 같은 세 자리 이름은 `00.무엇` 과 충돌하고 번호 중복으로
  FAIL 한다. 버퍼가 모자라면 자리수가 아니라 오버플로(아래 소문자 한 글자)를 쓴다.
- **NN 은 진짜 신규 작업에만.** 검증·리뷰·하우스키핑처럼 새 산출물이 없는 작업은 Task 폴더를 만들지
  않고 대상 Task 문서나 `docs/tech/`·`docs/troubleshootings/` 에 남긴다.
- **버퍼는 Phase 당 10개.** 소진 시 오버플로(`0a~0z` — 숫자 뒤 **소문자 한 글자**, 체커가 읽는 유일한
  확장)를 [masterPlan.md](../masterPlan.md) 에 **미리** 적어 둔다. 11번째에서 임기응변하면 canonical key 가
  흔들린다.

## 링크 (표준 마크다운)

- `docs/` 내부 참조는 **표준 마크다운 상대 경로**(`[텍스트](../etc/eval-protocol.md)`). 외부 URL 은
  `[text](https://…)`. 이중 대괄호 표기(wikilink)는 금지 — 백틱·울타리 안이라도 위반이다.
- 같은 Task 폴더 내 형제 참조는 **파일명만**(`[plan](plan.md)`, `[stage1](work/stage1.md)`). archive 로
  옮겨도 상대 관계가 유지된다.
- **다른 Task 를 가리킬 때**는 상대 경로(`../04.someTask/report.md`). **종결 시 인바운드 링크를
  `archive/{key}/…` 로 함께 갱신**한다. 가능하면 `report.md` 를 가리켜라 — 종결 후에도 의미가 유지되는
  유일한 문서다.
- 🔴 **링크는 휘발성 → 안정 방향으로만.** 안정 문서(`tech/`·`manual/`·`masterPlan.md`)는 휘발성 문서
  (`tasks/`)를 **가리키지 않는다** — 종결마다 갱신 부담이 생기고 놓치면 dead link 다. 남길 값이 있으면
  링크가 아니라 **그 문서로 내용을 옮긴다**. 활성 Task 목록을 링크 표로 만들지 않는다 — 폴더가
  SSOT 이고 목록은 `python tools/check_ddd_docs.py --tasks` 로 파생한다.
- **현존 문서는 링크로.** 백틱 경로는 코드 파일과 삭제된 역사 경로에만.
- **루트 문서**(`CLAUDE.md`·`PROJECT.md`·`PRODUCT.md`·`STATE.md`)는 `docs/` 밖이다. `docs/` 안에서는 백틱으로
  **루트 `PROJECT.md`** 처럼 쓴다. 코드 docstring·문서 밖 인용은 **주제명으로** — 섹션 번호는 개정 때
  전량 어긋난다.

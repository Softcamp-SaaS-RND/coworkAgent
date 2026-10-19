# eval.md — 1t.notion-ux 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 17c97befcfa966920fdd0dcb6d31087a5d3faf78

- [x] E1: A1 — intent 결정에 후보 10행(U1~U6 · B1~B4)이 모두 채택 · 보류 · 제외 중 하나로 판정됨
  CHECK: python -c "import re;s=open('{TASK}/intent.md',encoding='utf-8').read().split('\n## \uacb0\uc815\n')[1].split('\n## \ubbf8\ud574\uc18c')[0];ids=['U1','U2','U3','U4','U5','U6','B1','B2','B3','B4'];w='\ucc44\ud0dd|\ubcf4\ub958|\uc81c\uc678';ok=[i for i in ids if re.search('- '+i+' \u2014 ('+w+')',s)];print(f'rows={len(ids)} judged={len(ok)}')"
  EXPECT: rows=10 judged=10
  EVIDENCE: rows=10 judged=10

- [x] E2: A2 — masterPlan 후보 줄 2개 — "U1+U2"(그룹 · 정렬 · 거르기 · S1 · S4 · D35) · "B1"(모든 화면 · 배지 · D26)
  CHECK: python -c "t=open('docs/masterPlan.md',encoding='utf-8').read();sec=t.split('Phase 1 \uc624\ubc84\ud50c\ub85c')[1].split('## \uac1c\uc815 \uae30\ub85d')[0];c=[l for l in sec.splitlines() if l.startswith('- \ud6c4\ubcf4(') and '(D39 ' in l];a=sum(all(k in l for k in ['1t U1+U2','\ubcf4\ub4dc','\uc0c1\ud0dc \uc5f4','\uc5f4 \uba38\ub9ac \uac74\uc218','\uadf8\ub8f9 \u00b7 \uc815\ub82c \u00b7 \uac70\ub974\uae30','S1 \u00b7 S4 \u00b7 D35']) for l in c);b=sum(all(k in l for k in ['1t B1','\ubaa8\ub4e0','\ubc30\uc9c0','D26']) for l in c);print(f'candidates={len(c)} u1u2={a} b1={b}')"
  EXPECT: candidates=2 u1u2=1 b1=1
  EVIDENCE: candidates=2 u1u2=1 b1=1

- [x] E3: A3 — masterPlan D39 가 하나 · 결정 literal 7종(Notion · ① · ② · 보류 · 기각 · D34 · 모든) · 개정 기록 #1t · D39 줄 하나 · intent 가 D39 인용
  CHECK: python -c "t=open('docs/masterPlan.md',encoding='utf-8').read();s=t.splitlines();d=sum(l.startswith('- **D39**') for l in s);blk=t.split('- **D39**')[1].split('\n- **')[0];k=['Notion','\u2460','\u2461','\ubcf4\ub958','\uae30\uac01','D34','\ubaa8\ub4e0'];n=sum(x in blk for x in k);r=sum(l.startswith('- 2026-10-19 \u00b7 Task #1t \u00b7 D39') for l in s);i=int('D39' in open('{TASK}/intent.md',encoding='utf-8').read());print(f'd39={d} keys={n}/{len(k)} log={r} cited={i}')"
  EXPECT: d39=1 keys=7/7 log=1 cited=1
  EVIDENCE: d39=1 keys=7/7 log=1 cited=1

- [x] E4: A4 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: PASS — DDD 문서 규약 위반 없음

- [x] E5: 코드 무변경(결정 2) — 기준 커밋 뒤 바뀐 파일이 허용 목록(STATE · masterPlan · 원장 · 1t 폴더 `.md`) 안에만 있고, 필수 5개(STATE · masterPlan · intent · plan · work/stage1)가 모두 바뀜
  CHECK: python -c "import re,os,subprocess as p;g=p.run(['git','diff','--name-only','--diff-filter=ACMR','a74bdea'],capture_output=True,text=True);f=g.stdout.split();ok=re.compile(r'^(STATE\.md|docs/masterPlan\.md|docs/overview/ledger\.md|docs/(tasks|archive)/1t\.notion-ux/.+\.md)$');u=[x for x in f if not ok.match(x)];n=[x for x in f if not x.endswith('.md')];q=['STATE.md','docs/masterPlan.md','intent.md','plan.md','work/stage1.md'];r=sum(any((x==y or x.endswith('1t.notion-ux/'+y)) and os.path.exists(x) for x in f) for y in q)*(g.returncode==0);print(f'required={r}/5 unexpected={len(u)} non_md={len(n)}')"
  EXPECT: required=5/5 unexpected=0 non_md=0
  EVIDENCE: required=5/5 unexpected=0 non_md=0

## 적대적 평가 — 라운드 1/2

평가자: `codex-review` 세션 · `/codex:adversarial-review --base a74bdea --scope branch "1t: …"`(job `review-mvegrz4d-qadund`, 실행 HEAD `6eb30c5` =
요청 일치, worktree clean) · Verdict needs-attention(high 2 · medium 1). 원문:

```text
출하 보류. B1·B2 코드 사실과 STATE 재개 경로는 맞지만, A2 산출 추적성이 끊겼고 D39의 부분 refresh 계약은 stale UI를 허용한다. gate 5/5도 이를 검출하지 못한다.

- [high] 채택된 U2가 A2 산출물에서 독립적으로 추적되지 않는다 (docs/masterPlan.md:258)
  A2는 채택 행마다 이름·범위·닿는 D·S를 후보에 남기도록 요구한다. U2는 채택됐지만 U1 후보에 암묵적으로 합쳐졌고, 후보 줄은 U2 식별자와 원래 매핑인 S4를 누락한 채 `S1 · D35`만 기록한다. 후속 구현자가 U2의 그룹·정렬·거르기 저장 범위를 누락해도 산출물만으로 발견하기 어렵다.
  Recommendation: U2를 별도 후보로 만들거나 해당 줄을 `U1+U2`로 명시하고 그룹·정렬·거르기 저장 범위 및 `S1 · S4 · D35`를 기록한다.
- [high] E2·E3·E5가 성공 기준보다 약해 허위 5/5를 만든다 (docs/tasks/1t.notion-ux/plan.md:39-53)
  E2는 `(D39 `가 포함된 후보 줄 수만 세므로 이름·범위·D/S 매핑 누락을 통과시킨다. E3도 D39·개정 기록·인용의 존재만 검사해 승인된 결정 1~4 내용은 검증하지 않는다. E5의 `docs=min(len(d),1)`은 양수 개수를 전부 1로 축약하고 확장자만 검사하므로 예상 밖 Markdown 변경이나 산출 파일 누락에도 통과한다. 문서 Task의 핵심 산출 정확성을 gate가 증명하지 못한다.
  Recommendation: E2는 U1+U2/B1의 정확한 이름·범위·D/S literal을, E3는 D39의 채택·보류·기각 핵심 literal을 검사한다. E5는 기준 커밋 대비 허용 파일 6개의 정렬된 정확 목록과 `unexpected=0 non_md=0`을 단정한다.
- [medium] D39의 '그 화면만' refresh는 교차 화면 상태를 stale하게 만들 수 있다 (docs/masterPlan.md:196-197)
  현재 `refresh()`는 peers·groups·conversations·현재 route 데이터·counts를 함께 다시 읽고, `listen()` 주석도 한 이벤트가 연결·대화·내 활동 수를 함께 바꾼다고 명시한다(`app.js` 3773~3785, 3804~3805). 그런데 D39는 알림마다 '그 화면만' 읽도록 계약한다. 코드 fan-out상 추론하면 대화 변경 뒤 사이드바 목록·배지·내 활동 수가 갱신되지 않을 수 있고, 같은 결정이 5초 polling을 축소하면 stale 상태가 오래 유지된다.
  Recommendation: 화면 단위가 아니라 변경 resource의 모든 consumer를 invalidation 대상으로 정의한다. polling을 끈 상태에서 대화 본문·목록·배지·내 활동·연결 상태가 한 SSE 이벤트 후 일관되는지를 후속 Task 성공 기준에 추가한다.
```

| 지적(요지) | 판단 | 조치 |
|---|---|---|
| [high] U2 가 후보 줄에서 추적 안 됨 · S4 누락 | 사실 — 묶은 해석은 결정 3 에 있으나 후보 줄 자체엔 U2 · S4 가 없었다 | 후보 줄을 `D39 ① · 1t U1+U2` 로, U2 범위(그룹 · 정렬 · 거르기) · `S1 · S4 · D35` 명시. B1 줄도 `1t B1` 표기. intent 결정 3 U2 문구 맞춤 |
| [high] E2 · E3 · E5 가 기준보다 약함 | 사실 — 개수 · 존재만 단정, E5 는 모집단 축약 | plan 개정(`## 개정 기록` — **사용자 미승인**): E2 literal(U1+U2 · 그룹 · 정렬 · 거르기 · S1 · S4 · D35 / B1 · 모든 · D26), E3 D39 결정 literal 7종, E5 허용 목록 정규식 + 필수 5개 + `unexpected=0 non_md=0`. 허용 목록은 "정확 6개" 대신 패턴 — 종결 때 archive 이동 · report · 원장이 더해져 정확 목록은 종결 순간 깨진다 |
| [medium] D39 ② "그 화면만" → 교차 화면 stale | 사실 — `refresh()`(`app.js` 3773~3785)가 연결 · 그룹 · 대화 · 현재 화면 · 수를 함께 읽는다 | D39 ② 를 자원 단위 무효화(그 자원을 쓰는 **모든** 화면 · 배지)로 개정, 5초 새로 고침 축소는 "새로 고침 없이 한 알림 뒤 일관" 시험 뒤로. 후보 줄 성공 기준에 같은 문구. intent B1 권장 칸 맞춤 |

재측정: `python tools/check_gates.py docs/tasks/1t.notion-ux` → 5/5 충족(위 원장).

## 적대적 평가 — 라운드 2/2

평가자: `codex-review` 세션(라운드 1 과 같은 세션) · `/codex:adversarial-review --base a74bdea --scope branch "1t 라운드 2: …"`(job `review-mveh109w-dycngz`,
실행 HEAD `97e169f` = 요청 일치, worktree clean) · Verdict needs-attention(high 3). **라운드 상한 도달 — 반영 뒤 재리뷰 없음.** 원문:

```text
선생님, 출하 보류. U2 추적과 자원 단위 무효화 문구는 반영됐지만, 평가 계약 승인·재현성·gate 소실 방지가 아직 불충분합니다.

- [high] 미승인 EXPECT 개정을 통과 계약으로 사용함 (docs/tasks/1t.notion-ux/plan.md:56-60)
  E2·E3·E5의 CHECK뿐 아니라 EXPECT도 변경했으나 개정 기록은 `사용자 미승인`입니다. 저장소 규약상 EXPECT 변경은 사용자 승인이 필요한 항목 변경입니다. 그런데 eval은 변경된 계약을 이미 5/5 충족으로 기록합니다. 사용자에게 승인받지 않은 기준으로 완료 판정할 수 있습니다.
  Recommendation: 개정된 E2·E3·E5의 명시적 사용자 승인을 받은 뒤 승인일·사유를 개정 기록에 남기고 gate를 다시 실행하십시오.
- [high] 평가 원장의 측정 커밋이 실제 검사 내용과 다름 (docs/tasks/1t.notion-ux/eval.md:10)
  원장은 측정 커밋을 `6eb30c5`로 기록하지만, 해당 커밋에는 강화 전 E2·E3·E5와 U2/S4 누락·화면 단위 refresh 문구가 있습니다. 현재 원장의 강화된 출력은 이후 working tree에서만 얻을 수 있으므로 기록된 SHA를 checkout해 재현할 수 없습니다. 미커밋 변경이나 후속 drift를 숨기는 증거가 됩니다.
  Recommendation: plan·intent·masterPlan 수정이 포함된 커밋에서 `check_gates.py`를 다시 실행하고, 그 커밋 SHA를 측정 커밋으로 갖는 eval 갱신을 별도 커밋하십시오.
- [high] 강화된 gate도 산출물 삭제와 범위 이탈을 허위 통과시킴 (docs/tasks/1t.notion-ux/plan.md:39-53)
  E2는 Phase 1 오버플로 절 위치, U1의 보드·상태 열·건수, B1의 `배지`를 검사하지 않습니다. E3는 D39 블록 안 단어 7개의 공존만 검사해 결정별 의미 매핑이 사라져도 통과합니다. 특히 E5는 `git diff --name-only`를 사용해 삭제된 `work/stage1.md`도 필수 변경으로 세고, 임의의 1t `.md`와 ledger 변경을 모두 허용합니다. 따라서 검사 대상이 없어지거나 예상 밖 문서가 추가돼도 `required=5/5 unexpected=0 non_md=0`이 가능합니다.
  Recommendation: E2·E3는 정확한 절을 파싱해 각 결정의 필수 literal과 매핑을 검증하십시오. E5는 `git diff --name-status --diff-filter=ACMR`의 단계별 정확 파일 집합, 필수 파일의 현재 존재, git 명령 return code를 함께 단정하십시오.
```

| 지적(요지) | 판단 | 조치 |
|---|---|---|
| [high] 미승인 EXPECT 로 통과 판정 | 사실(리뷰 시점) | 사용자 승인 2026-10-19 "plan 승인." → plan `## 개정 기록` 승인 표기(`fa40814`) |
| [high] 원장 측정 커밋 `6eb30c5` ≠ 검사 내용 | 사실 — 라운드 1 수정을 커밋하기 전에 원장을 만들었다 | 수정 커밋 `17c97be`(clean tree)에서 `check_gates.py` 재실행 → 측정 커밋 = `17c97be`, 5/5. 원장 갱신은 별도 커밋 |
| [high] 강화 gate 도 삭제 · 범위 이탈 허위 통과 | 일부 사실 | E2 · E5 CHECK 보정(EXPECT 그대로): E2 = 오버플로 절 안 후보만 · U1 보드 · 상태 열 · 열 머리 건수 · B1 배지, E5 = `--diff-filter=ACMR` · 필수 파일 현재 존재 · git rc. E3 결정별 매핑 · E5 정확 파일 집합은 미조치(아래) |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| 라운드 2 [high] — E3 가 D39 블록 안 literal 7개의 공존만 보고 결정별 의미 매핑(① ↔ 보드, ② ↔ 자원 단위 등)은 검증 안 함 | 사실 | 문서 의미 매핑은 문자열 gate 로 끝까지 잡기 어렵다 — D39 문구는 종결 전 사용자 확인(STATE 블로커)이 판정한다. 강화하려면 EXPECT 변경 = 다시 plan 개정 · 승인 |
| 라운드 2 [high] — E5 가 단계별 정확 파일 집합이 아니라 허용 패턴(1t 폴더 `.md` · 원장)이라 예상 밖 1t 문서 추가를 통과 | 사실 · 의도 | 종결 때 archive 이동 · report · 원장이 더해져 정확 집합은 단계마다 바뀐다(EXPECT 를 둘로 쪼개야 함 — 항목 변경). 코드 무변경(결정 2)의 판정 목적은 `non_md=0` · 허용 밖 0 으로 충족 |

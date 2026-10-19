# plan — 1t.notion-ux

> 근거: [intent](intent.md)(승인 2026-10-19 — 결정 1 ~ 4). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `a74bdea`(main — 0.8.0 뒤 macOS 빌드). plan 승인: 2026-10-19(사용자 "권장 방안으로 진행" + masterPlan "자동 진행 위임").

## 목표

Notion 웹 앱 REA 분석(`D:\dev\git\notion\rea\analysis.md`)에서 반영할 것을 행별로 확정하고, masterPlan 에 차용 범위(D39)와
채택 항목의 후속 Task 후보를 남긴다. **코드 변경 없음.**

### 성공 기준(intent 사본)

- A1: 위 후보 표의 모든 행(U1~U6 · B1~B4)이 채택 · 보류 · 제외 중 하나로 사용자 확정되어 `## 결정` 에 기록된다.
- A2: 채택 행마다 후속 Task 후보(이름 · 범위 한 줄 · 닿는 D·S)가 masterPlan Phase 1 오버플로 후보 줄에 있다.
- A3: masterPlan 에 Notion 요소 차용의 범위를 정한 D 가 추가(또는 D26 · S4 개정)되고 `## 개정 기록` 에 #1t 줄이 있다.
- A4: 문서 체커 `python tools/check_ddd_docs.py` PASS.

## 설계

- 행별 판정 — intent `## 결정` 3(채택 U1 · U2 · B1 / 보류 U3 · U5 · B2 · B3 · B4 / 제외 U4 · U6).
- masterPlan
  - `## 확정 방향` 에 **D39**(골격 유지 + 채택 ① 요청 보드 보기 ② 바뀐 범위만 다시 그리기 · 보류 조건 · 기각) — intent 승인 커밋에 함께(intent-protocol 5: 인용 전에 추가).
  - Phase 1 오버플로 "남음" 에 후보 2줄 — 채택 U1+U2 → "요청 보드 보기(D39 ①)", B1 → "바뀐 범위만 다시 그리기(D39 ②)". 번호는 착수 때 매긴다(기존 후보 줄 형식).
  - `## 개정 기록` 에 `2026-10-19 · Task #1t · D39 …` 한 줄(종결 환류).
- 코드 · PRODUCT 모듈 표 · 매뉴얼 — 손대지 않는다(결정 2).

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | masterPlan 후보 2줄 · 개정 기록 · 평가 | masterPlan · work/stage1.md · eval.md | E1 ~ E5 |

## 평가 항목

- [ ] E1: A1 — intent 결정에 후보 10행(U1~U6 · B1~B4)이 모두 채택 · 보류 · 제외 중 하나로 판정됨
  CHECK: python -c "import re;s=open('{TASK}/intent.md',encoding='utf-8').read().split('\n## \uacb0\uc815\n')[1].split('\n## \ubbf8\ud574\uc18c')[0];ids=['U1','U2','U3','U4','U5','U6','B1','B2','B3','B4'];w='\ucc44\ud0dd|\ubcf4\ub958|\uc81c\uc678';ok=[i for i in ids if re.search('- '+i+' \u2014 ('+w+')',s)];print(f'rows={len(ids)} judged={len(ok)}')"
  EXPECT: rows=10 judged=10
  EVIDENCE: pending
- [ ] E2: A2 — masterPlan 후보 줄 2개 — "U1+U2"(그룹 · 정렬 · 거르기 · S1 · S4 · D35) · "B1"(모든 화면 · 배지 · D26)
  CHECK: python -c "t=open('docs/masterPlan.md',encoding='utf-8').read();sec=t.split('Phase 1 \uc624\ubc84\ud50c\ub85c')[1].split('## \uac1c\uc815 \uae30\ub85d')[0];c=[l for l in sec.splitlines() if l.startswith('- \ud6c4\ubcf4(') and '(D39 ' in l];a=sum(all(k in l for k in ['1t U1+U2','\ubcf4\ub4dc','\uc0c1\ud0dc \uc5f4','\uc5f4 \uba38\ub9ac \uac74\uc218','\uadf8\ub8f9 \u00b7 \uc815\ub82c \u00b7 \uac70\ub974\uae30','S1 \u00b7 S4 \u00b7 D35']) for l in c);b=sum(all(k in l for k in ['1t B1','\ubaa8\ub4e0','\ubc30\uc9c0','D26']) for l in c);print(f'candidates={len(c)} u1u2={a} b1={b}')"
  EXPECT: candidates=2 u1u2=1 b1=1
  EVIDENCE: pending
- [ ] E3: A3 — masterPlan D39 가 하나 · 결정 literal 7종(Notion · ① · ② · 보류 · 기각 · D34 · 모든) · 개정 기록 #1t · D39 줄 하나 · intent 가 D39 인용
  CHECK: python -c "t=open('docs/masterPlan.md',encoding='utf-8').read();s=t.splitlines();d=sum(l.startswith('- **D39**') for l in s);blk=t.split('- **D39**')[1].split('\n- **')[0];k=['Notion','\u2460','\u2461','\ubcf4\ub958','\uae30\uac01','D34','\ubaa8\ub4e0'];n=sum(x in blk for x in k);r=sum(l.startswith('- 2026-10-19 \u00b7 Task #1t \u00b7 D39') for l in s);i=int('D39' in open('{TASK}/intent.md',encoding='utf-8').read());print(f'd39={d} keys={n}/{len(k)} log={r} cited={i}')"
  EXPECT: d39=1 keys=7/7 log=1 cited=1
  EVIDENCE: pending
- [ ] E4: A4 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: pending
- [ ] E5: 코드 무변경(결정 2) — 기준 커밋 뒤 바뀐 파일이 허용 목록(STATE · masterPlan · 원장 · 1t 폴더 `.md`) 안에만 있고, 필수 5개(STATE · masterPlan · intent · plan · work/stage1)가 모두 바뀜
  CHECK: python -c "import re,os,subprocess as p;g=p.run(['git','diff','--name-only','--diff-filter=ACMR','a74bdea'],capture_output=True,text=True);f=g.stdout.split();ok=re.compile(r'^(STATE\.md|docs/masterPlan\.md|docs/overview/ledger\.md|docs/(tasks|archive)/1t\.notion-ux/.+\.md)$');u=[x for x in f if not ok.match(x)];n=[x for x in f if not x.endswith('.md')];q=['STATE.md','docs/masterPlan.md','intent.md','plan.md','work/stage1.md'];r=sum(any((x==y or x.endswith('1t.notion-ux/'+y)) and os.path.exists(x) for x in f) for y in q)*(g.returncode==0);print(f'required={r}/5 unexpected={len(u)} non_md={len(n)}')"
  EXPECT: required=5/5 unexpected=0 non_md=0
  EVIDENCE: pending

## 개정 기록

- 2026-10-19 · 적대적 평가 라운드 1 지적 2(gate 가 성공 기준보다 약함)에 따라 E2 · E3 · E5 의 CHECK · EXPECT 강화 — E2 `candidates=2` →
  `candidates=2 u1u2=1 b1=1`, E3 `d39=1 log=1 cited=1` → `d39=1 keys=7/7 log=1 cited=1`, E5 `docs=1 code=0` → `required=5/5 unexpected=0 non_md=0`.
  승인 2026-10-19(사용자 "plan 승인.").
- 2026-10-19 · 적대적 평가 라운드 2 지적 3 에 따라 E2 · E5 **CHECK 보정**(EXPECT 그대로 — 승인 대상 아님): E2 = Phase 1 오버플로 절 안 후보만 · U1 보드 · 상태 열 · 열 머리 건수, B1 배지 literal 추가. E5 = `--diff-filter=ACMR`(지운 파일 제외) · 필수 파일 현재 존재 · git rc 0 단정.

# eval.md — 1z-5.home-thumb 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 3360dfd642f9ed9a72c427811a26dab7f237abc4

- [x] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.25s

- [x] E3: A1·A2·A4 — 웹UI 로직 시험(절 · 카드 순서 · 한 줄 + 숨은 수 · 상대 타일 전부 · 말풍선 · 다시 읽을 때)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A1·A2·A4 — 절 목록 · 카드 격자 · 열 재기 · "+n개" · 마지막 글만 실재, B 창 · 탭 없음, 글자만
  CHECK: python -c "import pathlib as P;c=P.Path('crates/app/web/app.css').read_text(encoding='utf-8');j=P.Path('crates/app/web/app.js').read_text(encoding='utf-8');h=P.Path('crates/app/web/index.html').read_text(encoding='utf-8');v=P.Path('crates/app/web/view.js').read_text(encoding='utf-8');print('box=%d grid=%d fit=%d more=%d tail=%d old=%d html=%d'%(h.count('<div id=\x22home_preview\x22 class=\x22pv-list\x22></div>'),c.count('.pv-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(100%, 240px), 1fr));'),j.count('}).observe($(\x22home_preview\x22));'),j.count('text: `+${s.more}개` })'),j.count('/messages?limit=${PREVIEW_MSGS}`'),sum(x.count(k) for x in (c,j,v,h) for k in ('pv-win','pv-tab','previewTab','previewWindows')),j[j.index('function previewSectionEl'):j.index('/// 개요 보기 바꾸기')].count('innerHTML')))"
  EXPECT: box=1 grid=1 fit=1 more=1 tail=1 old=0 html=0
  EVIDENCE: box=1 grid=1 fit=1 more=1 tail=1 old=0 html=0

- [x] E5: A5 — 문서 규약 · 크레이트 경계 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;w=P.Path('docs/manual/dev/web-ui.md').read_text(encoding='utf-8');g=P.Path('docs/manual/user/user-guide.md').read_text(encoding='utf-8');print('webui=%d guide=%d'%(w.count('#home_preview.pv-list'),g.count('| \x27개요\x27 미리보기(기본) | 상대마다 절')))"
  EXPECT: webui=1 guide=1
  EVIDENCE: webui=1 guide=1

- [x] E6: A1·A2·A3·A4 — 실제 화면: 디버그 빌드(시험 데이터 폴더, #1z-4 시드 — 상대 4 · 대화 8)에서 기본 미리보기 · 절 · 카드 · 폭별 열 수 · 한 줄 + "+n개" · 상대 개요 전부 · 빈 상대 점선 · 글자만 · 안 읽음 그대로
  EVIDENCE: Claude(구현자) 2026-10-22 — `target/debug/coworkAgent.exe start --parent <PID>` + `COWORKAGENT_DATA_DIR`=scratchpad `ui1z5\data`(사용자 앱 그대로, #1z-4 시드 사본 — 상대 4 · 대화 8), ego-browser DOM 판독. 관측: 처음 `#home_view_preview` pressed · `#home_summary` hidden · `#home_intro` hidden → 절 4: 김도윤 "2시간 전 통신 · 대화 5 · 안 읽음 3 · 김도윤 개요 →" 카드 2 + "+3개"(자연 폭 3열, 첫 카드 `#c/1` `.unseen` · 머리 "배포 절차 점검 완료" · 말풍선 6 · 바닥 "review · 06:31 안 읽음 3", 둘째 `#c/3` error 말풍선), 박서연 카드 1(`<b>굵게</b>` 글자 그대로 · 태그 0), 이준호 카드 2, 최하은 `.pv-none` "3일 전 통신 · 대화 없음 · 세션 생성" → 격자 폭 380/700/1000/1400 = 1/2/3/5열 · 김도윤 카드 1+"+4개"(380 은 "+n개" 둘째 줄) / 1+"+4개" / 2+"+3개" / 5(넘침 없음) · 이준호 380 = 1+"+1개" · `overflow-y: auto` · 카드 본문 120px → 안 읽음 전후 `배포 절차 점검 3` 그대로 → "+3개" 누름 = "개요 · 김도윤" 절 1 · 카드 5 · 넘침 없음. R1 재확인: 1400 → 380 → 숨은 대화(`#c/2`) 첫 글 DB 수정(신호 없음) → 내 활동 갔다 개요로(전체 무효화, 보이는 카드 3만 읽음) → 1400 = `#c/2` 포함 숨었던 5개 다시 읽음 · "숨은 동안 고친 첫 글". 끝에 `coworkAgent stop`.


WEAK: E6 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_62e1895b-701c-4d6f-8964-629d8fcd82f2`.
Run `run_c0ecaf48f9dd` · Task `task_00e1343184cc` · Dispatch `ctx_a006981c2ddd`, worker_done(`msg_158359b947fc`). 범위 `4dd486c..2e3f74e`(self-collect),
메타 `요청 1z-5-home-thumb-r1 · revision 2e3f74ec216ae36a9a0bac3beede21357a60ef67 · 라운드 1/2`. 리뷰 동안 대상 작업 폴더 HEAD 2e3f74e 그대로(그 사이 report 초안 · check_gates 첫 실행의 eval.md — 커밋 밖, 리뷰 범위 밖). 리뷰어 Dispatch `worker-release` → `released` · `captured`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against 4dd486c
Verdict: needs-attention

고정 revision diff와 결정 1~7·이어받은 재읽기 규칙을 읽기 전용으로 검토하고 실제 함수 추출 재현을 수행했다. 전체 무효화가 숨은 카드 캐시에 남지 않아 SSE 재연결 후 오래된 글을 표시하는 medium 결함 1건을 확인했다. 숨은 캐시 무효화와 회귀 검증이 남았으며 브라우저 레이아웃 반복은 이번 실행으로 검증하지 않았다.

Findings:
- [medium] 전체 무효화를 한 줄 밖 카드에도 유지해야 합니다 (crates/app/web/app.js:1773-1777)
  대화 5개를 4열에서 읽어 캐시한 뒤 2열로 줄이면 카드 1개만 보입니다. SSE 단절 중 숨은 대화의 마지막이 아닌 글이 변경되고 재연결되면 onopen은 previewAll=true로 복구하지만, 여기서는 플래그를 즉시 지우고 보이는 카드만 갱신합니다. 숨은 캐시는 previewStale에도 들어가지 않아 4열로 넓혀 다시 표시해도 previewKey가 같으므로 이전 글을 계속 표시합니다. 대상 revision의 previewSections·previewKey·renderHomePreview를 추출한 Node 재현에서 폭 확장 후 요청 0회와 숨었던 카드의 old 캐시 유지를 확인했습니다(레이아웃·SSE 자체는 stub). 기존 B의 탭 선택은 previewStale.add(conv)로 새로 읽었지만 A의 폭 확장에는 대응 경로가 없어 #1z-4 재연결 복구 규칙이 깨집니다.
  Recommendation: previewAll을 소비할 때 모든 기존 캐시를 무효화하거나 모든 캐시 대화를 previewStale에 추가하고, 보이지 않아 아직 읽지 않은 대화의 표시를 유지하십시오. 캐시 생성 → 폭 축소 → 숨은 대화의 비마지막 글 변경 → 재연결 전체 무효화 → 폭 확장 시 해당 대화를 다시 GET하는 회귀 검증을 추가하십시오.

Next steps:
- 전체 무효화를 보이지 않는 카드에도 보존한 뒤 재연결·폭 확장 회귀를 검증한다.
- 수정 revision으로 라운드 2/2를 수행한다.
```

판정 · 조치:

| 지적 | 판정 | 조치 |
|---|---|---|
| R1-1 medium 전체 무효화가 한 줄 밖 숨은 카드에 남지 않음 | 사실 — `previewAll` 을 바로 지우고 보이는 카드만 읽어, 숨은 카드의 받아 둔 글은 `previewKey` 가 같으면 넓혀도 그대로 | `renderHomePreview` 에서 `all` 이면 이번에 안 읽은 받아 둔 대화를 `previewStale` 로(보일 때 읽음) · `ux_literals` 한 줄 · 실제 화면 재확인(E6 R1 재확인) |

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex, handle `term_1199e2c3-bf5e-4f9a-826a-d4c91744a154`. Run `run_c0ecaf48f9dd` · Task `task_b9fcb9b3290d` · Dispatch `ctx_7360e92dac5b`,
worker_done(`msg_f0cfb5087277`) 발신 handle 일치. 범위 `4dd486c..e954a7a`(self-collect), 메타 `요청 1z-5-home-thumb-r2 · revision e954a7a9adf07adf1d91c7385f2c1e3e0f019eb8 · 라운드 2/2`. 리뷰 동안 대상 작업 폴더 편집 없음(HEAD e954a7a 그대로).
리뷰어 Dispatch `worker-release` → `released` · `captured`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against 4dd486c
Verdict: approve

고정 범위 diff와 명세·이어받은 규칙을 읽기 전용으로 대조했습니다. R1 숨은 카드 무효화·재노출 읽기·실패 재시도·응답 역전·열 수별 카드 선택을 해당 revision 코드의 메모리 내 실행으로 확인했으며, 차단할 실질 결함은 발견하지 못했습니다. 실제 브라우저의 ResizeObserver·레이아웃·접근성 동작은 이번 리뷰에서 재실행하지 않았습니다.

No material findings.
```

판정: 지적 없음(approve) — 상한 2/2 도달, 재리뷰 없음. 리뷰어가 밝힌 미실행(실제 브라우저 `ResizeObserver` · 레이아웃 · 접근성)은 E6 실제 화면이 덮는다(접근성은 판독 안 함).

## 미조치 지적

없음

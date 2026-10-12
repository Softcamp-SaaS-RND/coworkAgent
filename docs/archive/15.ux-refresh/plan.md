# plan — 15.ux-refresh

> 근거: [intent](intent.md)(승인 2026-10-12 — 공유 이해 도달, 라운드 2 결정 · A9 개정 포함). **구현자는 이 문서만 보고 일한다.**
> D·S 는 [masterPlan](../../masterPlan.md). 기준 커밋: `5bb31b5`(intent 라운드 2). plan 승인: 2026-10-12(사용자 "plan 승인").

## 목표

웹UI 를 하나의 디자인(사용자가 고른 시안)으로 다시 입히고, 초대 단계 축소 · 목록 그룹/필터/검색 · 등록 항목 편집 · model/effort 설정 ·
메시지 뷰어 · '나'/상대별 메시지 목록 · favicon 을 더한다.

### 성공 기준(intent 사본)

- A1: 디자인 시안 5개(실제 데이터의 구조 · 분량 유지, 이름 · IP · 지문 · 경로 익명화)를 아티팩트로 게시하고 사용자가 1개를 고른 결정이 intent `## 결정` 에 기록된다.
- A2: 고른 시안 기준으로 전 화면의 폰트 · 크기 · 색이 공통 토큰(CSS 변수) 하나에서 나오고, 키 · 값과 편집 · 읽기 전용이 시각적으로 구분된다.
- A3: 설정 항목이 트리뷰로 전부 보인다.
- A4: markdown · html 내용이 뷰어로 보이고 편집 · 읽기 모드를 버튼으로 바꾼다.
- A5: 웹UI 탭 favicon 이 `coworkAgent.ico` 와 같은 그림이다.
- A6: 초대 생성 측은 그 초대(5분 창 · 1회용 코드)로 들어온 요청을 추가 승인 없이 등록하고, 등록 즉시 알림(D29)을 띄우며 연결 목록에서 바로 삭제할 수 있다.
- A7: 초대 수락 측 안내 문구가 '초대 생성'에서 생성된 메시지 입력 기준이고 "메신저로 받은" 표시가 없다.
- A8: 연결 목록이 아코디언으로 보이고 그룹 필터가 있으며, 목록에서 내 에이전트 프로필의 도구를 D31 읽기 전용 목록 안에서 편집할 수 있다.
- A9: `claude -p` 실행이 기본 `--model sonnet` · `--effort high` 를 붙이고, 설정의 기본값과 등록한 상대방별 덮어쓰기(비우면 기본값)를 바꾸면
  그 상대 요청의 다음 실행에 반영된다.
- A10: 내 에이전트 — 공유 목록 우선 출력 · 그룹 · 필터 · 검색 · 등록된 경로 · 도구 편집 · 추가 지시 사용 예제 가이드.
- A11: 알림 화면 텍스트 · 컨트롤 크기가 같고, `relay_cap_default` 라벨이 대화 턴에 대한 상한 기본값 문구다.
- A12: 메인 화면 — '나' 선택 시 나에게 들어온 메시지 목록(필터 · 검색), 등록 사용자 선택 시 그 사용자와의 대화 화면(D26) 위에 내가 보낸 메시지 목록 탭.
- A13: 설정에서 화면 테마를 라이트(기본) · 다크로 고르고, 고른 값이 이 PC 설정에 저장돼 다시 열어도 유지된다.
- A14: 새 글은 모두 대화 에이전트에게 실행 요청이다 — 웹UI 에 대상 선택이 없고, CLI `say` 는 없으며(모르는 명령 2), 로컬 API `to: "human"` 은 400.
  기존 이력의 사람 글(실행 없음)은 그대로 보인다.
- A15: 대화 목록 · 머리글의 방향 표시가 화살표(→ ←) 대신 "보냄" · "받음" 태그다.

### intent 결정(사본)

- 배치 Phase 1(19 앞) · 한 Task(gate 20 · 인터뷰 5라운드 닿으면 쪼갬) · 시안 결정이 첫 stage.
- **D8 개정(위험 수용)**: 초대 생성 측 승인 생략 — 5분 창 · 1회용 초대 코드가 유일한 관문. 보완: 등록 즉시 알림 · 연결 목록 바로 삭제.
  직접 추가(지문 미고정) 경로의 LAN MITM 자동 등록도 수용(2026-10-12, 사내망 전용 — intent D8 "추가 수용").
- 도구 변경 = 연결 목록에서 **내 에이전트 프로필** 도구 편집(상대별 차이 없음, D31 목록 안).
- 메인 화면: 대화 화면(D26 A + C)은 유지, 보낸 메시지 목록은 탭으로 더한다.
- 시안 데이터: 구조 · 분량 유지, 이름 · IP · 지문 · 경로 익명화.
- 그룹화 = 사용자 지정 그룹 이름(피어 · 에이전트마다 1개) + 그룹별 아코디언 · 그룹 필터 · 검색.
- 뷰어: 메시지 본문 = markdown 뷰어(읽기 전용), 대화 머리글 "원문 보기" 버튼 하나로 전체 전환. 입력칸은 늘 편집(개정 4).
- HTML 본문 = `<iframe sandbox srcdoc>`(스크립트 · 폼 · 같은 출처 차단, 외부 리소스 CSP 차단).
- model · effort = 전역 기본값(sonnet · high) + 등록한 상대방별 덮어쓰기.
- '나' 목록 = 모든 대화에서 상대 쪽이 쓴 글 전부, 최신순, 누르면 그 대화로(개정 5: 사이드바 머리 링크로 연다).
- 타일 클릭 = 사이드바에 그 타일의 대화 목록 → 고르면 메인에 메시지(개정 5). 대화 제목 · 타일 표시 이름 · 아이콘은 이 PC 에만(개정 5).
  트레이 메뉴 = "열기" · "종료"(개정 5, S4 개정).
- 시안 A 기본(토큰 = [work/stage1.md](work/stage1.md)) · 방향 = "보냄"/"받음" 태그 · 사람에게 보내기 제거(웹UI · CLI `say` · API, D23 개정) ·
  다크 테마 = 다크 기본 + 설정에서 라이트 선택(개정 4).
- 제외: macOS 실동작(D25) · 새 UI 프레임워크 · 외부 라이브러리(현재 정적 ES 모듈 유지 — D10) · D31 밖 도구 · 19 TC 실행.

## intent 대비 해석 (plan 승인 대상)

1. **화면 라벨 literal** — 원문 항목 이름을 그대로 쓴다: "페어링 받기" → **"초대 생성"**, "초대 붙여넣기" → **"초대 수락"**, 알림 상한
   라벨 → **"대화 턴에 대한 상한 기본값"**(원문 "대화 턴에 대한 상한 기본값으로 문장 변경"). 초대 텍스트(`invite.rs` `to_text`) 둘째 줄의
   `"초대 붙여넣기"` 도 `"초대 수락"` 으로. 초대 수락 입력 안내 = "'초대 생성' 에서 만든 초대 메시지를 통째로 붙여 넣으세요".
   초대 생성 측 안내("복사해서 상대에게 보내세요")는 원문이 바꾸라 하지 않았지만 "메신저" 낱말은 같이 뺀다(수락 측과 짝).
2. **자동 등록 범위** — 처음 보는 키의 hello 가 받기 창의 초대 코드와 맞으면(= 초대 텍스트 붙여넣기 · 주소+코드 직접 추가 둘 다) 내 승인을
   자동으로 하고 상대에 approve 를 보낸다. 아는 키의 재 hello(다시 추가 — 코드 없음)는 지금처럼 사람 승인.
3. **에이전트 편집 범위** — 폴더 · 도구 · 추가 지시 · 그룹. **이름은 읽기 전용** — 대화가 에이전트를 이름으로 가리키고 상대 PC 도 그
   이름으로 부른다(바꾸면 진행 중 대화가 `unknown_agent`). 이름을 바꾸려면 새로 등록.
4. **상대별 model · effort 가 쓰이는 실행** — 그 상대가 보낸 요청(받은 대화)과 그 상대와의 자동 왕복에서 내 에이전트 실행. 비우면 전역 기본값.
5. **effort 지원 확인** — 설치 검증 환경 `claude` 2.1.296 `--help` 에 `--model` · `--effort <level>`(low · medium · high · xhigh · max) 있음.
   `MIN_CLAUDE`(2.1.267)의 `--effort` 지원은 확인 필요 — Stage 2 에서 `claude --help` 실측, 없으면 `MIN_CLAUDE` 를 올리고 기록.

## 설계

### 저장(스키마 9, `an-core`)

```sql
ALTER TABLE peers  ADD COLUMN grp TEXT;     -- 그룹 이름, NULL = 그룹 없음
ALTER TABLE peers  ADD COLUMN model TEXT;   -- 이 상대 요청 실행의 model 덮어쓰기, NULL = 기본값
ALTER TABLE peers  ADD COLUMN effort TEXT;  -- 이 상대 요청 실행의 effort 덮어쓰기, NULL = 기본값
ALTER TABLE agents ADD COLUMN grp TEXT;
```

- 이전은 기존 규칙(한 트랜잭션 · 있는 열 건너뛰기 · `{db}.v8.bak`). `Peer` · `Agent` 에 필드, `peers::set_meta(id, grp, model, effort)` ·
  `agents::update(id, folder, tools, prompt, grp)`.
- 전역 기본값은 `settings` 키 `claude_model`(기본 `sonnet`) · `claude_effort`(기본 `high`). 값 검증 한 곳(`an-core` `run_opts`):
  model = 1 ~ 64자 `[A-Za-z0-9._:\[\]-]`, `-` 로 시작 금지 · effort ∈ {low, medium, high, xhigh, max}. 그룹 이름 = 공백 정리 0 ~ 32자.

### 실행 인자(`an-agent`)

- `RunInput` 에 `model` · `effort`. `Chat` 이 실행 직전 정한다: 그 대화 상대의 덮어쓰기 → 전역 설정 → `sonnet` · `high`.
  설정을 못 읽으면(DB 오류) 기본값으로 돌리지 않고 실행 실패 답(개정 7).
- `claude_args` 끝에 `--model <m> --effort <e>`(가드 · 도구 규칙 그대로).
- 실행 도중 그 에이전트의 폴더가 바뀌면(편집이 세션을 지운다) 끝난 실행의 세션을 남기지 않는다 — 기록 트랜잭션에서 실행 때 폴더와 지금 폴더를
  대조(개정 7).

### 초대 자동 등록(`an-net`)

- `answer` 의 hello: 처음 보는 키 + 초대 코드 일치면 `upsert` 뒤 `set_my_ok`, 응답 뒤 그 상대에 `Approve{round}` 를 보낸다(응답을 막지 않게
  작업으로). 이벤트 `PeerEvent::AutoPaired{name, host, fingerprint}` — 알림(D29)은 `notify_pair` 설정을 따르고 문구 "새 연결 등록".
- 붙인 쪽(`POST /api/peers`): 초대 붙여넣기는 추가 뒤 승인. 주소+코드 직접 추가도 코드를 담아 보냈고 상대가 승인했으면(`their_ok`) 승인까지 한 번에 —
  아는 키 다시 추가(상대 미승인)는 승인 대기(개정 6).

### 로컬 API(`an-app`)

- `GET /favicon.ico` — `crates/app/assets/coworkAgent.ico`(`include_bytes!`), `image/x-icon`. `index.html` 에 `<link rel="icon" href="/favicon.ico">`.
- `PATCH /api/peers/{id} {group?, model?, effort?}`(빈 문자열 = 지움, 틀린 값 400) · 피어 JSON 에 `group` · `model` · `effort`.
- `PATCH /api/agents/{id} {folder?, tools?, prompt?, group?}`(이름 없음 — 주면 400) · 에이전트 JSON 에 `group`.
- `GET/PUT /api/settings` 에 `claude_model` · `claude_effort`(같은 검증). 설정 DB 읽기 오류는 기본값으로 감추지 않고 500(개정 7).
- `GET /api/messages?side=theirs|mine&peer=<id>&kind=human|agent&q=<글>&limit=<n>` — 대화를 가로지른 글 목록, 최신순(기본 200, 최대 500). `theirs` =
  `mine = false`(상대 쪽이 쓴 글), `mine` = 내가 쓴 글. `kind` 는 글 종류(사람 글 · 에이전트 답)로 `LIMIT` 전에 거른다(개정 7, 그 밖 400).
  `q` 는 본문 부분 일치(`LIKE` 이스케이프). 항목 = 메시지 JSON + `conversation{id, peer_id, agent, dir}`.

### 웹UI(`web/`, 빌드 없는 ES 모듈 유지)

- **디자인 토큰**: `app.css` `:root` 에 색 · 글꼴 · 크기 · 간격 변수 — 그 밖에서는 색 리터럴 · 글꼴 크기 리터럴을 쓰지 않는다(`app.js`
  의 `RAIL_COLORS` 도 변수로). 값은 Stage 1 에서 고른 시안.
- **키 · 값 / 편집 · 읽기 전용**: 설정의 정보는 2열 표(`<table class="kv">`), 읽기 전용 값은 잠금 표시 · 편집 가능 값은 입력 컨트롤.
- **설정 트리뷰**: 설정 화면 왼쪽 `<nav>` 트리 — 내 정보 / 연결(초대 생성 · 초대 수락 · 연결 목록) / 내 에이전트 / 실행(model · effort) / 알림.
  항목을 누르면 그 절로 스크롤. 트리 항목 목록은 `view.js` `settingsTree()` 가 정본.
- **연결 목록**: 그룹별 `<details>` 아코디언 + 그룹 필터(`<select>`) + 검색(이름 · 호스트 · 지문). 항목 펼침 = 상태 · 지문 · 그룹 · model ·
  effort 편집 · 삭제 · 내 에이전트 도구 편집 바로가기.
- **내 에이전트**: 등록 목록을 먼저(그룹 아코디언 · 필터 · 검색 — 이름 · 폴더), 항목별 편집(폴더 · 도구 체크박스 · 추가 지시 · 그룹), 그 아래
  새 등록 폼. 추가 지시 사용 예제 가이드(접이식, 예: "답하기 전에 docs/ 의 설계 문서를 먼저 읽고 근거 파일 경로를 함께 적는다").
- **뷰어**: 메시지 본문 — HTML 문서로 보이면(`<!doctype html` · `<html` 로 시작) sandbox iframe(`sandbox=""`, `srcdoc` 앞에 CSP
  `default-src 'none'; style-src 'unsafe-inline'; img-src data:`), 아니면 markdown 뷰어. 원문 보기 = 머리글 버튼 하나(개정 4). markdown 은 `view.js`
  `markdownBlocks(text)`(제목 · 목록 · 코드 블록 · 인용 · 표 · 문단 · 인라인 코드 · 굵게 · 기울임 — 링크는 글자로, 개정 1)가 블록 목록을 만들고
  `app.js` 가 `createElement` · `textContent` 로만 조립(`innerHTML` 없음). 입력칸은 편집 / 미리보기 버튼 없이 늘 편집(개정 4).
- **알림**: 컨트롤 크기를 토큰으로 맞추고 라벨 literal(해석 1).
- **테마**: `:root` 라이트 토큰 + `:root[data-theme="dark"]` 다크 토큰(같은 이름). 설정 `ui_theme`(`dark` 기본 · `light`, 그 밖 400 — 개정 4)을
  `GET/PUT /api/settings` 로, `app.js` 가 시작 때 `document.documentElement.dataset.theme` 에 건다.
- **보내기 대상 없음**: 작성창 · 새 대화에 대상 선택 없음 — 항상 에이전트에게. `POST /api/conversations/{id}/messages` · 새 대화
  `POST /api/conversations`(개정 7) 의 `to: "human"` 은 400(`to` 생략 · `"agent"` 만). CLI `say` 삭제 — `is_command` 에서 빼서 모르는 명령(2). 받기(`inbound_post` 사람 글)는 유지.
- **방향 태그**: `view.js` `convLabel` 이 화살표 대신 `{ tag: "보냄" | "받음" }` 을 내고 목록 · 머리글이 태그로 그린다.
- **메인 화면**(개정 5): 타일(레일 '나' · 상대)을 누르면 사이드바 = 그 타일 대화 목록('나' = 전부), 메인 = 안내(`#`) — 열린 대화가 그 타일
  것이면 그대로. 목록에서 고르면 `#c/{id}`. 사이드바 머리 링크: '나' = 받은 글 `#inbox`(필터 상대 · 종류 · 검색), 상대 = 주고받은 글 `#p/{id}`
  (`side=theirs` · `mine` 을 `peerMessages` 로 합침). 대화 화면 머리에 "보낸 메시지" 탭(그 상대에게 내가 쓴 글 목록). 설정 화면은 사이드바 숨김
  (전체 폭), 사이드바 폭은 경계 끌기 · ← → 키(200 ~ 560px) — 설정 `ui_side_width`(웹UI 포트가 실행마다 바뀌어 `localStorage` 는 이어지지 않는다).
- **대화 제목**(개정 5): 스키마 10 `conversations.title`(NULL = 첫 사람 글 앞 40자). `POST /api/conversations` 의 `title`(선택 — 보내기 전에
  검증하고 대화와 한 트랜잭션으로 저장 — 전송이 실패해도 남는 대화에 붙는다, `Chat::send_via` · `start_relay` 의 `title`) · `PATCH /api/conversations/{id}` `{title}`(빈 값 = 지움, 100자 · 제어 문자 없음 — `run_opts::clean_title`,
  모르는 항목 거절). 상대에 보내지 않는다. 머리글 제목 옆 ✎ → 입력칸. 사이드바 "제목 검색" 은 제목으로만.
- **타일 표시 이름 · 아이콘**(개정 5): 상대 = 스키마 10 `peers.alias` · `peers.icon`(`PATCH /api/peers/{id}` 의 `alias` · `icon`, 빈 값 = 지움),
  나 = 설정 `me_alias` · `me_icon`(`PUT /api/settings`). 검증 `run_opts::clean_alias`(32자) · `clean_icon`(`data:image/png;base64,iVBORw0KGgo`
  로 시작 · base64 글자 · 65,536자까지). 웹UI 가 고른 그림을 캔버스로 가운데 정사각형 80×80 PNG 로 줄여 보낸다. 표시 이름은 `view.js`
  `displayName` · `tileText`(글자 묶음 단위 앞 두 글자)로 타일 · 사이드바 머리 · 대화 목록 · 발화자 · 상대 고르기 · 연결 검색에, 아이콘은 `<img src>` 로
  타일에. 받은 메시지 알림 이벤트의 상대 이름도 표시 이름(`an_agent::chat` `notify`), 페어링 알림은 원래 이름. `PATCH /api/peers` 는 준 쪽(실행 설정 ·
  타일)만 검증 · 저장한다.
- **트레이**(개정 5): 정상 모드 메뉴 "열기" · "종료"("내 정보 보기" 삭제 — S4 개정). 오류 모드 그대로.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 시안 5개(익명화 실데이터) 아티팩트 게시 → 사용자 선택 → intent `## 결정` 기록 · 디자인 토큰 값 확정 | 아티팩트 링크 · 토큰 표 | E4 |
| 2 | 스키마 9 · 실행 인자 · 초대 자동 등록 · API(favicon · PATCH · settings · messages) | 테스트(paths · run_options · auto_pair · ux_api) | E6 · E8 · E9 · E10 |
| 3 | 웹UI 전면(토큰 · 트리 · kv 표 · 아코디언 · 편집 · 뷰어 · 나/보낸 목록 · 문구) | 테스트(view.test · ux_literals) | E3 · E5 · E7 |
| 공통 | 검증 명령 · 실화면 | — | E1 · E2 · E11 |

## 평가 항목

- [ ] E1: 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A3·A4·A8·A10·A12·A15·A17 — 화면 로직 node 테스트 21건(기존 13 + settings_tree · markdown_blocks_safe · html_doc_detect · group_filter_search · inbox_rows · peer_messages · direction_tag · tile_text — 개정 4 에서 edit_preview_toggle 대체, 개정 5 에서 tile_text 추가)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass 21\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A1 — 시안 5개 아티팩트 게시 · 사용자 선택이 intent `## 결정` 에 기록(수동: 아티팩트 링크 · 고른 안 · 날짜)
  EVIDENCE: pending
- [ ] E5: A2 — 색 · 글꼴 크기 리터럴은 `app.css` `:root` 안에만(`app.js` 색 리터럴 0)
  CHECK: python -c "import re;c=open('crates/app/web/app.css',encoding='utf-8').read();j=open('crates/app/web/app.js',encoding='utf-8').read();m=re.search(r':root\s*\{[^}]*\}',c);rest=c.replace(m.group(0),'') if m else c;v=len(re.findall(r'#[0-9a-fA-F]{3,8}\b|rgba?\(',rest))+len(re.findall(r'font-size:\s*[0-9.]+(?:px|rem|em)',rest))+len(re.findall(r'#[0-9a-fA-F]{6}\b',j));print(f'root={int(bool(m))} violations={v}')"
  EXPECT: root=1 violations=0
  EVIDENCE: pending
- [ ] E6: A6 — 초대 자동 등록(auto_pair 2건: invite_hello_auto_pairs_both_sides · rehello_without_code_needs_approval)
  CHECK: cargo test -p an-net --test auto_pair
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A5·A7·A11·A16·A18 — 화면 문구 · favicon 링크 · 트레이 메뉴 literal(ux_literals 5건: turn_label_literal · invite_labels_without_messenger · favicon_link_present · title_search_label · tray_menu_open_quit_only — 개정 5 에서 2건 추가)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: pending
- [ ] E8: A5·A6·A8·A9·A10·A12·A13·A14·A16·A17 — 로컬 API(ux_api 16건: favicon_served · peer_patch_group_model_effort · agent_patch_edits_but_not_name · settings_claude_defaults · inbox_lists_their_messages_with_search · outbox_lists_my_messages_to_peer · theme_setting_persists · to_human_rejected · conversation_title_set_and_edit · peer_tile_alias_icon · settings_me_tile_side_width · direct_add_with_code_pairs_both_sides · direct_readd_known_key_waits_for_approval · new_conversation_to_human_rejected · inbox_kind_filter_applies_before_limit · settings_read_error_is_500 — 개정 5 에서 3건, 개정 6 에서 2건, 개정 7 에서 3건 추가)
  CHECK: cargo test -p coworkagent-app --test ux_api
  EXPECT: test result: ok. 16 passed; 0 failed
  EVIDENCE: pending
- [ ] E9: A9 — 실행 인자 model · effort(run_options 3건: claude_args_carry_model_effort · peer_override_then_default · settings_read_error_fails_run — 개정 7 에서 1건 추가)
  CHECK: cargo test -p an-agent --test run_options
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: pending
- [ ] E10: A8·A9·A10·A16·A17 — 스키마 9 · 10 이전(paths 10 → 12건 — 개정 5 에서 schema10_title_alias_icon 추가)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 12 passed; 0 failed
  EVIDENCE: pending
- [ ] E11: A2·A3·A4·A6·A8·A10·A11·A12·A13·A15·A16·A17·A18 — 실화면(수동): 고른 시안대로 전 화면 · 트리 · 아코디언 · 편집 · 뷰어 · 나/보낸 목록 · 알림 크기 · 두 PC 초대 자동 등록 · 타일 클릭 · 제목 · 타일 표시 이름 · 아이콘 · 트레이 메뉴(개정 5)
  EVIDENCE: pending
- [ ] E12: A14 — CLI `say` 삭제(cli 15건: Task #16 의 16건 − `say_*` 2건 + say_is_unknown_command)
  CHECK: cargo test -p coworkagent-app --test cli
  EXPECT: test result: ok. 15 passed; 0 failed
  EVIDENCE: pending

## 종결 시 할 일

- E11 은 D28 에 따라 `ABANDON`(사유: D28) + 19.prototype-acceptance 에 TC-K 추가.
- masterPlan D23 개정(사람에게 보내기 제거) · D14 개정(CLI `say` 삭제) · 19 TC-J 에서 `say` 단계 삭제 · 원장 #16 관련 행 확인.
- masterPlan D8 개정 환류(초대 생성 측 자동 등록 · 위험 수용 — 직접 추가 경로 MITM 포함) · D21 보강(그룹 · 편집) · D22 보강(model · effort 전역 + 상대별) · S4 보강
  (트레이 "내 정보 보기" 삭제 — 개정 5) · 매뉴얼 `an_app.md` 의 트레이 메뉴 줄 · 19 acceptance.md 26행(트레이 "내 정보 보기" 로 지문 확인)을
  "열기 → 설정 → 내 정보" 로.
- 매뉴얼 `an_core` · `an_net` · `an_agent` · `an_app` · PRODUCT(페어링 규칙 · 실행 인자 · 웹UI 구조).

## 개정 기록

- 개정 7(2026-10-12, 적대적 평가 라운드 2/2 반영 — **평가 항목 변경 · 사용자 승인 2026-10-12**("개정 7 승인")): E8 13 → 16(new_conversation_to_human_rejected ·
  inbox_kind_filter_applies_before_limit · settings_read_error_is_500) · E9 2 → 3(settings_read_error_fails_run). 설계: 새 대화도 `to` 검증 ·
  `GET /api/messages` 의 `kind`(제한 전 거르기) · 설정 DB 읽기 오류는 500 · 실행 실패 답 · 실행 도중 폴더가 바뀌면 그 실행의 세션을 남기지 않음
  (roundtrip 11 → 12 — folder_change_during_run_drops_stale_session 은 E1 로 잰다).

- 개정 6(2026-10-12, 적대적 평가 라운드 1/2 반영 — **평가 항목 변경 · 사용자 승인 2026-10-12**("개정 6 승인")): E8 11 → 13(direct_add_with_code_pairs_both_sides ·
  direct_readd_known_key_waits_for_approval). 설계 "초대 자동 등록" 에 붙인 쪽 승인 — 주소+코드 직접 추가도 코드가 맞아 상대가 승인하면 붙인 쪽 승인까지
  한 번에(초대 붙여넣기와 같다), 아는 키 다시 추가는 승인 대기.

- 개정 5(2026-10-12, 라운드 4 빌드 확인 뒤 사용자 수정 지시 — intent 라운드 5 · A12 개정 · A16 ~ A18 신설, **평가 항목 변경 · 사용자 승인 2026-10-12**("개정 4·5 승인")):
  E3 20 → 21(tile_text) · E7 3 → 5(title_search_label · tray_menu_open_quit_only) · E8 8 → 11(conversation_title_set_and_edit · peer_tile_alias_icon ·
  settings_me_tile_side_width) · E10 11 → 12(schema10_title_alias_icon) · E11 에 A16 ~ A18. 설계에 타일 클릭 · 대화 제목 · 타일 표시 이름 · 아이콘 ·
  트레이 메뉴 · 사이드바 폭 설정 저장(개정 4 의 `localStorage` 정정). 스키마 10.

- 개정 4(2026-10-12, Stage 3 빌드 확인 뒤 사용자 수정 지시 — intent 라운드 4, **평가 항목 변경 · 사용자 승인 2026-10-12**("개정 4·5 승인")): E3 의 `edit_preview_toggle`
  → `peer_messages`(편집 / 미리보기 제거로 대상 없음, 건수 20 그대로). theme_setting_persists 기대값 기본 `dark`. 설계에 다크 기본 · 레일 상대
  `#p/{id}` · 설정 전체 폭 · 원문 보기 하나 · 사이드바 폭 조절.

- 개정 3(2026-10-12, Stage 2 — 설계 문구만, 평가 항목 · EXPECT 변경 없음): 초대 자동 등록의 승인 전달을 "응답 뒤 `Approve{round}` 작업" 대신
  **Hello 응답의 `approved`**(`#[serde(default)]`)로 — 수락 측이 피어를 저장하기 전에 Approve 가 닿는 경쟁을 없앤다. 근거 · 이전 버전 상대 영향은
  [work/stage2.md](work/stage2.md) "설계 결정".
- 개정 2(2026-10-12, 시안 결정 — intent 개정 A13 ~ A15, **평가 항목 변경 · 사용자 승인 2026-10-12**("plan 개정 승인")): E3 19 → 20(direction_tag) · E8 6 → 8
  (theme_setting_persists · to_human_rejected) · E12 신설(CLI `say` 삭제 — cli 16 → 15) · E11 에 A13 · A15. 설계에 테마 · 보내기 대상 없음 · 방향 태그.
- 개정 1(2026-10-12, Stage 1 — 설계 문구만, 평가 항목 · EXPECT 변경 없음): markdown 뷰어 범위에 **표**(`|` 머리 · 구분 · 행)를 더한다 — 실데이터의 긴 답에 표 7줄이 있다.

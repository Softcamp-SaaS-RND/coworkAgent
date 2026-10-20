# eval.md — 1x.session-ui 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 5dfdd0437e37ea6d0410da39ff6d53ac51830332

- [x] E1: A12 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A12 — 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.24s

- [x] E3: A12 — 화면 로직 node 테스트 · 실패 0 · 147 이상
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (14[7-9]|1[5-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 153.8217

- [x] E4: A2 — 빈 대화 생성 · 첫 글 상대 도달 · 첫 글 자동 왕복(두 PC)
  CHECK: cargo test -p an-agent --test session_open session_
  EXPECT: /test result: ok\. [4-9] passed; 0 failed/
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.55s

- [x] E5: A5·A6·A13 — 기록 삭제 내 글 · 모든 사용자 · 관람자 · 거절 · 이전 버전 · 실행 중(두 PC)
  CHECK: cargo test -p an-agent --test conv_clear clear_
  EXPECT: /test result: ok\. (1[0-9]|[7-9]) passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 6.27s

- [x] E6: A2·A9·A10·A14 — 서버 경로 · 설정 · 알림 글
  CHECK: cargo test -p coworkagent-app --test ux_api -- theme_default_light tile_size_setting_persists notify_sound_setting request_notice_text create_session_without_text clear_route
  EXPECT: /test result: ok\. 6 passed; 0 failed/
  EVIDENCE: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 32 filtered out; finished in 0.29s

- [x] E7: A1·A6·A8·A13·A14 — 메뉴 · 머리 부제 · 세션 생성 기본값 · 요청 알림 node 시험
  CHECK: node --test --test-reporter=tap --test-name-pattern="conv_menu_items_1x|side_sub|compose_defaults|request_notice|session_label" crates/app/web/view.test.mjs
  EXPECT: /# pass [5-9]\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 78.7839

- [x] E8: A1·A3·A4·A7 — 화면 원문: "새 대화" · 메시지 종류 · 요청 안내 · 줄 ☆ · 대화 줄 HTML5 끌기 0, 세션 생성 · 손잡이 있음
  CHECK: python -c "import pathlib,re;h=pathlib.Path('crates/app/web/index.html').read_text(encoding='utf-8');j=pathlib.Path('crates/app/web/app.js').read_text(encoding='utf-8');v=pathlib.Path('crates/app/web/view.js').read_text(encoding='utf-8');lit=lambda s,w:len(re.findall(r'[\x22\x27\x60>][^\x22\x27\x60<\n]*'+w,s));ci=j[j.index('function convItem'):j.index('function renderConvList')];print('old=%d kind=%d hint=%d star=%d html5=%d new=%d grip=%d'%(lit(h+j+v,'새 대화'),h[h.index('chat_form'):h.index('</form>',h.index('chat_form'))].count('메시지 종류'),h.count('실행하지 않고'),ci.count('starButton'),ci.count('draggable: \x22true\x22'),min((h+v).count('세션 생성'),1),min(ci.count('drag-handle'),1)))"
  EXPECT: old=0 kind=0 hint=0 star=0 html5=0 new=1 grip=1
  EVIDENCE: old=0 kind=0 hint=0 star=0 html5=0 new=1 grip=1

- [x] E9: A3 — 사용자 문서의 "새 대화" 0 · "세션 생성" 있음
  CHECK: python -c "import pathlib;fs=['docs/manual/user/user-guide.md','README.md'];t=''.join(pathlib.Path(f).read_text(encoding='utf-8') for f in fs);print('targets=%d old=%d new=%d'%(len(fs),t.count('새 대화'),min(t.count('세션 생성'),1)))"
  EXPECT: targets=2 old=0 new=1
  EVIDENCE: targets=2 old=0 new=1

- [x] E10: A11 — masterPlan 후보 줄에 클립보드 이미지 검토(조각 전송 · 저장 위치 · 바이트 확인 · 읽기 실측)
  CHECK: python -c "import pathlib;t=[l for l in pathlib.Path('docs/masterPlan.md').read_text(encoding='utf-8').splitlines() if l.startswith('- 후보') and '클립보드 이미지' in l];print('lines=%d frame=%d dir=%d magic=%d add_dir=%d'%(len(t),sum('MAX_FRAME' in l for l in t),sum('데이터 폴더' in l for l in t),sum('PNG' in l for l in t),sum('--add-dir' in l for l in t)))"
  EXPECT: lines=1 frame=1 dir=1 magic=1 add_dir=1
  EVIDENCE: lines=1 frame=1 dir=1 magic=1 add_dir=1

- [x] E11: A12 — 문서 규약 · 의존 경계 · 화면 문구 표
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && cargo test -p coworkagent-app --test ux_literals
  EXPECT: crates=4 edges=6 violations=0
  EVIDENCE: crates=4 edges=6 violations=0

- [ ] E12: A7·A14 — 실화면: 사이드바 대화 줄 손잡이 끌기(같은 절 옮기기 · 그룹 타일 넣기)를 CDP 로 — `Input.dragIntercepted` 없음 · 순서 바뀜 확인(이 PC), 받은 요청 OS 알림 모양 · 효과음 · 누르면 요청 보드(두 PC — 프로토타입 인수 Task(D28)로 이월).
  EVIDENCE: pending

ABANDON: E12 실화면 중 받은 요청 OS 알림 모양 · 효과음 · 누르면 요청 보드(두 PC) — 수동 테스트는 프로토타입 인수 Task 에서 일괄(masterPlan D28), 19 TC 로 넘김. 이 PC 몫(사이드바 대화 줄 손잡이 끌기)은 CDP 로 확인했다 — 그룹 타일 넣기 `dragIntercepted false` · `PUT /api/groups/12/conversations {"add":[1]}` · ⋯ 하위 펼침 · 나가기 확인 문구 · 세션 생성 상대 미리 선택 · 작은 타일(stage2 테스트 결과). 같은 절 줄 옮기기는 데이터가 없어 못 봄(판정은 E7 · 기존 dropKind 시험)

## 적대적 평가 — 라운드 1/2

평가자: Orca 기존 Codex 세션 `review`(title `review | skills`, handle `term_b99297f6-be8f-4ea5-9a09-227db963ade4`, agentIdentity codex, 직전 Dispatch 완료 · Ready 확인) — `codex-peer-review` 스킬,
Run `run_9bccb8d352a8` · Task `task_73023e5dfd94` · Dispatch `ctx_1bb942dac76f`(`worker-start --terminal` 이 `agent_unconfigured`, `dispatch --inject` 가 `no_agent_detected` 로 거절돼
`dispatch`(주입 없음) + preamble · spec 파일을 `terminal send` 로 전달 — 결과는 worker_done 으로 회수, payload taskId · dispatchId 일치). 범위 `fea9fb1..674496c`(self-collect), 메타
`요청 1x-session-ui-R1 · revision 674496c · 라운드 1/2`. 리뷰 뒤 작업 폴더 HEAD 674496c 그대로(다른 세션 PRODUCT.md · security-posture.md 외 변경 없음). 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against fea9fb1
Verdict: needs-attention

출하 보류: fea9fb1..674496c의 intent·plan·구현·시험을 읽기 전용으로 대조했다. Clear 재전송의 추가 삭제, Clear보다 새 글이 먼저 전송되는 순서 결함, 기록 삭제 후 기존 요청 재실행, 상대 삭제 실패의 사용자 통지 누락을 확인했다. 실행 재현·테스트는 수행하지 않았으며 해당 결함 수정과 손실·재전송 회귀 검증이 남았다.

Findings:
- [high] Clear를 멱등 처리하지 않아 재전송이 삭제 이후의 새 기록까지 지운다 (crates/agent/src/chat.rs:4264-4271)
  Clear에는 conv만 있고 삭제 작업 ID·세대·삭제 경계가 없다. inbound_clear는 수신할 때마다 4269행에서 현재 기록 전체를 다시 지운다. B가 Clear를 적용한 뒤 응답이 유실되거나 A의 acked DB 정산이 실패하면 A는 같은 프레임을 재전송한다. 그 사이 B에서 새 글을 작성한 경우 재전송된 Clear가 새 글·세션까지 삭제하며 관람자 사본도 다시 비운다. 이는 정상 재시도 경로만으로 발생 가능한 비멱등 삭제다. 실행 재현은 하지 않았다.
  Recommendation: Clear에 고유 작업 ID와 삭제 경계를 싣고 적용 여부를 삭제와 같은 트랜잭션으로 영속화하라. 같은 작업의 재수신은 Ok만 반환하고, 경계 이후 새 기록은 보존하라. 적용 뒤 ACK 유실→새 글→동일 Clear 재전송을 검증하라.
- [high] conv_id 없는 Clear가 새 메시지의 순서 검사를 우회한다 (crates/agent/src/chat.rs:4218-4225)
  4223행은 Clear를 conv_id=NULL인 pending 행으로 넣는다. 그러나 dispatch(737행)의 선행 검사 outbox::has_before는 conv_id=conv.id인 행만 찾고, failing은 tries>0인 pending만 찾는다. 따라서 방금 큐에 넣은 Clear(tries=0) 또는 전송 중인 Clear는 새 글의 즉시 전송을 막지 않는다. lane이 Clear를 보내기 전·knows_clear Ping을 기다리는 동안 사용자가 새 글을 보내면 새 Message/Post가 먼저 도착할 수 있고, 뒤따른 Clear가 새 글을 지운다. 상대가 실행한 새 요청의 기록까지 없어질 수 있다. 코드 경로로 확인했으며 실행 재현은 하지 않았다.
  Recommendation: 삭제 후에도 Clear 행을 보존하려는 저장 수명과 대화별 전송 순서를 별도로 관리하라. pending·inflight Clear를 같은 UUID 대화의 장벽으로 삼고 이후 글은 ACK 뒤 전송하라. capability Ping을 지연한 상태에서 삭제 직후 새 글 전송을 검증하라.
- [high] 기록과 함께 수신 UUID 중복 방지 정보도 삭제해 옛 요청이 다시 실행된다 (crates/core/src/chat.rs:628-636)
  clear_conversation은 대화 UUID를 유지하면서 messages를 물리 삭제하고 별도 수신 tombstone을 남기지 않는다. inbound_message의 중복 방지는 message_by_uuid(2025행)에 의존하며, 없으면 같은 요청을 State::Queued로 다시 넣고 wake한다. B가 요청을 실행했지만 A가 ACK를 받지 못한 상태에서 B가 실행 완료 후 내 글만 삭제를 하면 busy 검사도 통과한다. 이후 A의 outbox가 옛 Message를 재전송하면 B는 새 요청으로 받아 에이전트를 재실행한다. 삭제한 Ask/Post도 늦은 재전송으로 복원될 수 있다. 정상 전송 실패에서 가능한 코드 경로이며 실행 재현은 하지 않았다.
  Recommendation: 본문·실행 이력과 별개로 처리한 메시지 UUID 또는 대화 세대·수신 경계를 보존하라. 삭제 이전 프레임 재수신은 기록 복원·재실행 없이 ACK하도록 하고, ACK 유실→실행 완료→기록 삭제→원 Message 재전송에서 실행 횟수 불변을 검증하라.
- [medium] 늦게 확인된 미지원 상대의 삭제 실패를 성공처럼 정산하고 로그만 남긴다 (crates/agent/src/chat.rs:1024-1030)
  상대가 꺼져 있을 때 all 삭제는 로컬을 비우고 Clear를 보관한다. 나중에 상대가 Pong.clear=false로 확인되면 여기서는 warn 로그만 남기고 Response::Ok로 outbox를 제거한다. 사용자는 모든 사용자 글 삭제를 요청했지만 상대 기록이 남았다는 화면·내 활동 통지를 받지 못한다. plan의 deliver 계약은 '상대 쪽 기록은 지우지 못했습니다' 내 활동 기록을 요구하며 stage1도 이 누락을 사용자 미확인 해석으로 인정한다. 기능 미지원이 사용자에게 드러나지 않는 삭제 계약 위반이다.
  Recommendation: 미지원·영구 거절을 사용자에게 보이는 영속 실패 상태와 알림으로 남기고 해당 대화에서 확인할 수 있게 하라. 오프라인 all 삭제 후 이전 버전 상대가 복귀하는 회귀 시험에서 실패 표시까지 검증하라.

Next steps:
- Clear 작업 ID·삭제 경계·전송 장벽과 수신 UUID 보존을 설계해 ACK 유실·지연·재전송에도 새 기록과 실행 횟수를 보존한다.
- 미지원 상대의 삭제 실패를 사용자에게 표시하고 관련 회귀 시험을 추가한다.
- 수정 후 라운드 2/2에서 이번 지적의 폐쇄와 수정으로 생긴 결함을 우선 검토한다.
```

raw JSON: 위와 같은 내용(`worker_done` 본문 둘째 줄) — 보관 `scratchpad/review_r1.json`.

### 판정 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [high] Clear 비멱등 — 응답 유실 뒤 재전송이 새 기록까지 지운다 | 사실(`inbound_clear` 가 받을 때마다 비움) | `Clear{conv, id}` — 받는 쪽이 적용한 `id` 를 `gone_msgs` 에 남기고 같은 `id` 는 `Ok` 만(같은 트랜잭션). 시험 `clear_resend_idempotent` |
| [high] conv_id 없는 Clear 가 새 글의 순서 검사를 우회 | 사실(`has_before` 는 같은 `conv_id` 행만 본다) | Clear 행을 그 대화 줄(`conv_id`)에 — 뒤에 쓴 글은 그 뒤. 나가기는 기다리는 Clear 를 지우지 않고(`kind <> clear`), 삭제는 떼어 남긴다(`conv_id` NULL). 시험 `clear_before_later_posts`(줄 막음 단정) · `clear_then_leave_still_clears_peer` |
| [high] 지운 글 uuid 가 없어 늦은 재전송이 되살리거나 다시 실행 | 사실 + 더 깊은 원인 발견 | `gone_msgs`(스키마 22) — `clear_conversation` 이 지운 글 uuid 를 남기고, 받은 `Message` · `Post` · `Ask` · `Reply` 가 그 uuid 면 `Ok` 만. 고치며 **글 번호 재사용 결함**을 찾았다: 비운 뒤 새 글이 지운 번호를 받으면 대화 작업(`wake` 의 꺼낸 번호 집합)이 새 글을 실행하지 않는다 → `messages.id` AUTOINCREMENT(스키마 22). 시험 `cleared_message_resend_not_rerun` · `schema22_messages_autoincrement`, 옛 전제 시험(`share.rs` 같은 id 재사용) 고침 |
| [medium] 늦게 확인된 이전 버전 상대 — 로그만 | 사실(stage1 구현자 해석 1) | `acked` 의 반영 실패 기록에 `clear` 를 넣어 내 활동 "반영하지 못함 — 기록 삭제"(`sync_failed`), 대화를 지운 뒤면(떼어 낸 행) 남길 자리가 없어 넘어간다. 시험 `clear_late_old_peer_noted` |

물기 확인 — `inbound_clear` 의 `gone(id)` 확인 · Clear 행 `conv_id` · `inbound_message` 의 `gone(msg)` 확인을 끈 빌드에서 `clear_resend_idempotent` · `clear_before_later_posts` ·
`cleared_message_resend_not_rerun` · `clear_late_old_peer_noted` FAILED → 되돌림. 수정 뒤 `cargo test --workspace` 78 바이너리 467 passed · 0 failed, clippy 0.

plan 개정(사용자 미승인): E5 EXPECT `[7-9]` → `(1[0-9]|[7-9])`(시험 4개 늘어 11) — plan 개정 기록.


## 적대적 평가 — 라운드 2/2

평가자: 같은 Codex 세션 `review`(handle `term_b99297f6-…`) — Run `run_9bccb8d352a8` · Task `task_96393cb4008c` · Dispatch `ctx_5173fe1e06a7`(라운드 1 과 같은 전달 방법), payload taskId · dispatchId 일치.
범위 `fea9fb1..672134e`, 메타 `요청 1x-session-ui-R2 · revision 672134e · 라운드 2/2`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against fea9fb1
Verdict: needs-attention

출하 보류: fea9fb1..672134e를 읽기 전용으로 검토했고 R1의 동일 Clear 재수신·이미 수신한 UUID 재실행·뒤에 쓴 글 순서·미지원 상대 실패 기록에 대한 조치를 확인했다. 다만 이미 직접 전송 중인 옛 요청을 Clear가 추월해 삭제 뒤 다시 실행·복원시키는 순서 경합 1건이 남았다. SQL 행 삭제·busy 판정은 메모리 DB로 확인했지만 네트워크 순서 역전은 코드 기반 추론이며 전체 테스트·두 PC 실화면은 실행하지 않았다.

Findings:
- [high] 진행 중인 직접 전송을 Clear가 추월할 수 있어 삭제한 요청이 뒤늦게 실행된다 (crates/agent/src/chat.rs:4227-4234)
  clear_conversation은 send_human/post가 쓰는 대화 turn 잠금을 잡지 않고, busy도 running/queued만 검사해 로컬 Sent 글은 막지 않는다. 여기서 core clear_conversation을 호출하면 657행 SQL이 그 글의 inflight outbox 행까지 삭제하지만 dispatch(749~761행)가 이미 띄운 send_to 작업과 req 사본은 취소되지 않는다. 그러면 lane은 새 Clear를 앞 행으로 보고 별도 연결로 보낼 수 있다. 기존 대화에서 A의 옛 Message 연결/프레임만 지연되고 Clear가 먼저 B에 적용된 뒤 옛 Message가 도착하는 경우, B는 그 UUID를 아직 수신하지 않았으므로 gone_msgs에 없고 inbound_message가 다시 queued로 넣어 실행한다. A는 지워졌는데 B에는 삭제 이전 요청·답이 생긴다. Clear id 멱등성과 새 Clear 행의 conv_id는 이 선행 inflight 프레임의 순서를 보장하지 않는다. SQL상 Sent가 busy=false이고 inflight 행이 제거되는 것은 메모리 DB로 확인했으며, 이 네트워크 실행 순서는 별도 연결을 병렬 처리하는 코드에 근거한 추론으로 실전 재현은 미수행이다.
  Recommendation: Clear를 직접 전송과 재전송 lane을 포함한 동일 대화 순서에 넣고 이미 전송 중인 프레임이 정산되기 전 그 장벽을 제거하지 마라. 지연 프레임까지 확실히 거절해야 한다면 Clear에 삭제 세대·경계 또는 삭제한 송신 UUID를 전달해 수신자가 아직 못 본 옛 요청도 억제하게 하라. 첫 Message만 수신 직전에 정지→all Clear 완료→Message 해제 시 기록 0·추가 실행 0을 검증하라.

Next steps:
- 삭제와 직접 전송·재전송의 순서를 보장하고 지연된 삭제 이전 요청의 실행을 막는 회귀 시험을 추가한다.
- E5 EXPECT 개정의 사용자 미승인 상태와 이번 지적의 판단·조치 여부를 기록한다.
- 라운드 2/2 상한에 따라 추가 적대적 재리뷰 없이 종료하고 두 PC 실화면은 D28 인수에서 확인한다.
```

raw JSON: 보관 `scratchpad/review_r2.json`.

### 판정 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [high] 진행 중인 직접 전송을 Clear 가 추월 — 지운 요청이 상대에서 뒤늦게 실행 | 사실(코드 경로 — `busy` 는 실행 중 · 대기만, `clear_conversation` 이 `inflight` 행을 지워도 떠난 `send_to` 는 그대로) | `clear_conversation(all)` 이 그 대화 · 그 상대의 `inflight` 행이 있으면 409 `CLEAR_SENDING`("보내는 중인 글이 있습니다 — 잠시 뒤 다시 지우세요", 같은 쓰기 트랜잭션 — `outbox::sending`). 기다리던(`pending`) 행은 아직 떠나지 않아 지워도 된다. 시험 `clear_all_waits_inflight`(막음을 끈 빌드에서 FAILED 확인) |

라운드 2/2 상한 — 위 수정 뒤 재리뷰하지 않고 종료(전역 지침). 수정 뒤 `cargo test --workspace` 78 바이너리 468 passed · 0 failed, clippy 0.

두 라운드 지적 5건 모두 반영. 남은 것: 두 PC 실화면(E12 — D28 인수), 네트워크 순서 역전의 실전 재현(시험은 막음 조건만 본다 — 추론 기반 지적, 리뷰어도 실전 재현 미수행 —
원장 `#1x R2-D1` 수용).

## 미조치 지적

없음

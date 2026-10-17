# stage2 — 관람자 초대(공유 대화): 통신 · 저장 · 퍼뜨리기 · 사본 막기

> 근거: [plan](../plan.md) "개정 — 관람자 초대 · 레일 우클릭 · 설명 지우기" 해석 11 ~ 22 · Stage 2. 구현은 별도 에이전트(이 세션이 검토 · 커밋).

## 목표

A(대화를 가진 PC)가 페어링된 C 를 관람자로 넣으면 C 에 사본(`dir = shared`)을 만들고 이력 · 이후 글 · 답 · 편집 · 삭제를 보낸다. 원 상대 B 에는 관람자 수만.
대상 gate: E11 · E12 · E15.

## 산출 파일

- `net` — `proto.rs` `Request::Share` · `SharePost { conv, msg: ShareMsg }` · `ShareDelete` · `ShareEnd` · `ShareNotice`, `ids_ok` 등록, `Pong.share`(`#[serde(default)]`).
  `node.rs` 처리기 목록 두 곳 · `share: true` · `knows_share` · `MessageKind::Share`.
- `core` — 새 `share.rs`(관람자 표 · 사본 열기 · `upsert`(mine 0 · exec 0 · request 0, 이 사본 글만 덮어씀) · 지움 · 끝 · 관람자 수 · `MAX_OBSERVERS` 5),
  `db.rs` 스키마 20(`shares` 표 · `conversations.share_other` · `share_dir` · `share_ended` · `watchers` · `messages.owner`, `MINE_BY_DIR` `'shared' → 0`),
  `chat.rs` `Dir::Shared` · `Message.owner` · `find_own` · 활동 집계에서 사본 제외 · 끝 글을 지우면 `shares.sent_upto` 를 낮춤(SQLite id 재사용), `outbox.rs`
  `SHARE` · `SHARE_NOTICE`, `peers.rs` 페어링 해제 때 그 상대의 관람 · share 행 지움.
- `agent` — `chat.rs` `share_add`(사본 · 원 상대 · 페어링 안 됨 · 5명 · 이미 관람자 · 구버전 거절, 관람자 행 · Share 머리 · 이력을 한 트랜잭션) · `share_remove` ·
  `share_leave` · `fan_share`(커서 `sent_upto`, `msg_id` NULL, `has_before` 예외, 포기 없음) · 편집 · 삭제 반영 · 받는 처리기(사본 소유 확인 · 위조 거절) · 원 상대
  알림(바뀔 때만) · 사본 쓰기 거절("공유 대화는 읽기 전용").
- `app` — `notify.rs` `MessageKind::Share`(제목 "관람자 — {agent}", 본문 "관람자 n", "요청" 알림 묶음).
- 시험 — 새 `agent/tests/share.rs` 7건 · `core/tests/store.rs` `share` 1건 · `net/tests/remote_input.rs` `share_frames_ids_checked`, 기존 `paths.rs` 스키마 숫자 ·
  `context.rs` Pong 리터럴.

## 설계 결정 · plan 편차

- share 행을 보내기 전마다 Ping 으로 `Pong.share` 확인(plan 해석 19 문구 그대로) — 연결 수가 두 배. ponytail 주석: 느리면 상대별 능력을 기억하거나 묶는다.
- 이미 관람자인 상대를 다시 넣으면 `AlreadyExists`(다시 보내지 않음). `Share.other` = 원 상대가 스스로 정한 이름(A 의 별칭은 보내지 않음).
- 빠진 사본에 오는 SharePost · ShareDelete 는 `unknown`. 관람자를 빼는 응답은 `not_paired` · `unsupported` 뿐.
- 프레임이 64KB 를 넘을 것 같으면 본문을 4KB 로 줄인다(포기하지 않는 줄을 한 프레임이 막지 않게).
- 남은 것 — 페어링 해제로 관람자가 지워질 때 원 상대의 관람자 수는 갱신하지 않는다 · 사본 글 상태는 처음 보낼 때 값(이후 실패 전환은 안 따라감) · CLI `history`
  발화자(app 범위, stage 3).

## 테스트 결과

```text
cargo test -p an-agent --test share           → test result: ok. 7 passed; 0 failed
cargo test -p an-agent --test share notice    → test result: ok. 1 passed; 0 failed; 6 filtered out
cargo test -p an-core --test store share      → test result: ok. 1 passed; 0 failed; 13 filtered out
cargo test --workspace                        → test result: ok 73건 · 실패 0
cargo clippy --workspace --all-targets -- -D warnings → Finished
```

## 다음 단계

- stage 3 — API(`/api/conversations/{id}/observers` · `leave`, 대화 JSON `observers` · `watchers` · `share`, 글 `owner`) · 화면 연결 · CLI 발화자.

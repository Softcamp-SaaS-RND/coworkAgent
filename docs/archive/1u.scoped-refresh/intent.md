# intent — 1u.scoped-refresh

승인: 2026-10-19

## 왜

masterPlan **D39 ②** "바뀐 범위만 다시 그리기" 의 첫 후속 Task(Phase 1 후보 줄 `D39 ② · 1t B1`). 지금 웹UI 는 SSE 알림이 오면 종류와 무관하게
전체 `refresh()` 를 하고(`crates/app/web/app.js` `listen()` · `refresh()`), 그와 별도로 5초마다 같은 `refresh()` 를 돈다(`app.js` `setInterval(refresh, 5000)`).
화면이 늘수록(요청 보드 D39 ① 등) 알림 한 건의 비용이 커진다.

- 출처: masterPlan D39 ② · Phase 1 후보 줄 · 원장 #17 S3-L, 근거 보강 `D:/dev/git/notion/rea/porting-guide-cowrkAgent.md` R1
  (Notion 은 `versions/{id}:{table}` 버전 신호만 보내고, 로컬 version ≥ 알림 version 이면 다시 읽지 않는다 — 로그인 캡처 [관찰-런타임]).
- 이 Task 가 없으면: 5초 새로 고침을 줄일 전제(D39 — "한 알림 뒤 그 화면 · 배지 일관" 시험)가 생기지 않는다.

### 코드 사실 (2026-10-19 확인)

| 사실 | 위치 |
|---|---|
| SSE 알림 = `Notice {kind, title, body}` — 자원 정보 없음 | `crates/app/src/notify.rs` `struct Notice` |
| 알림은 **원격 이벤트(`PeerEvent`)에서만** 나온다 — 페어링 3종 + `Message`(종류 8) | `crates/net/src/node.rs` `enum PeerEvent` · `MessageKind` |
| 알림은 **알림 설정이 허락할 때만** SSE 로 간다 — 전체 끔 · 일시 중지 · 집중 · 종류별 끔이면 보내지 않는다 | `notify.rs` `Notifier::on_event` · `Prefs::allows` |
| 따라서 알림을 끈 사용자는 **5초 새로 고침만으로** 화면이 맞춰진다. 이 PC 에서 생긴 변경(내가 보낸 글 · 실행 끝 · outbox 상태)은 SSE 신호가 없다 | 위 + `app.js` 동작 후 다시 그리기 |
| 메시지 알림을 내는 곳은 한 함수 — 대화 id(`conversation_id`)를 이미 쥐고 있다 | `crates/agent/src/chat.rs` `fn notify`(→ `self.node.emit(PeerEvent::Message …)`) |
| 구독자가 뒤처지면 놓친 알림을 건너뛴다(`RecvError::Lagged` → `continue`) | `notify.rs` `async fn events` |
| `refresh()` = 연결 · 그룹 · 대화 목록 + 현재 화면(대화 · 내 활동 · 요청 · 저장 · 상대 메시지 · 실행 · 설정) + 건수 | `app.js` `refresh()` |

## 범위

- 서버 → 웹UI SSE 에 알림과 별개인 **변경 신호**(이벤트 이름 `change`, 본문 `{res: [{kind, id?}]}`)를 둔다 — 알림 설정 · 일시 중지 · 집중과 무관하게 항상 보낸다(결정 1).
- 신호 원천(결정 2): 상대의 요청 처리(대화가 있는 요청 전부 — 글 · 답 · 요청 상태 · 진행 · 편집 · 삭제 · 컨텍스트 · 자동 왕복 · 공유) → `conv`(이 PC 대화 id),
  메시지 알림 이벤트 → `conv`, 페어링 이벤트 → `peer`.
- 웹UI `view.js` 순수 함수 **자원 → 로더 소비자 표** · 신호 합치기: `conv` 면 그 자원을 쓰는 **모든** 로더(열린 대화는 같은 id · id 모름일 때만),
  빈 목록 · 모르는 종류 · `peer` 면 지금처럼 전체 `refresh()`.
- 뒤처짐(`Lagged`)으로 놓친 신호가 있으면 빈 `res` 변경 신호 한 번 = 전체 다시 읽기(결정 5).
- 알림(`Notice`)은 브라우저 알림 · 상태 줄 문구만 — 다시 읽기는 변경 신호가 맡는다.
- 단위 시험: Rust(원천별 자원 키 · 설정 무관 · 뒤처짐 · SSE `event: change`) · node(소비자 표 · 합치기 · 전체 폴백).

## 제외

- **5초 새로 고침 축소 · 제거** — A7 을 실화면으로 확인한 뒤 별도 Task(D39).
- 이 PC 에서 생긴 변경(내가 보낸 글 · 실행 끝 · outbox 전달 상태)의 전용 신호 — 5초 축소 Task 에서(결정 2). 단 지금 알림을 내는 이 PC 원천(예: 자동 왕복 끝)은 메시지 알림 이벤트라 함께 신호가 간다.
- 시간이 지나 꺼짐으로 바뀌는 접속 표시(D17 — 통신이 없어 생기는 변화라 신호가 없다) — 5초 새로 고침이 그대로 메운다. 상대 요청으로 켜지는 쪽은 `conv` 신호의 연결 로더가 맡는다.
- 버전 건너뛰기(`ver`) — 결정 3.
- 피어 프로토콜(`crates/net/src/proto.rs` 프레임) 변경 — SSE 는 이 PC 웹UI 전용.
- 요청 보드 보기(D39 ① — 다음 Task).
- 브라우저 쪽 캐시 · 저장소(가드레일 — 이력은 서버 SQLite).

## 성공 기준

- A1: `conv` 변경은 연결(접속 표시 — D17 마지막 통신 시각) · 그룹 · 대화 목록 · 열린 대화 본문 · 내 활동 · 요청 · 저장 · 상대 메시지 · 실행 · 건수 로더를 **모두** 고르고, 설정은 고르지 않으며, 다른 대화만 바뀌면 열린 대화 본문은 고르지 않는다(node 시험).
- A2: 빈 `res` · 모르는 종류 · `peer` 는 전체 `refresh()`, 짧은 간격에 온 신호는 한 번으로 합쳐지고 하나라도 전체면 전체다(node 시험).
- A3: 상대 요청 처리 · 메시지 알림 · 페어링이 각각 맞는 자원 키(`conv` + 이 PC 대화 id / `peer`)의 변경 신호를 낸다(Rust 시험).
- A4: 알림이 꺼짐 · 일시 중지 · 집중이어도 변경 신호는 간다, 변경 신호는 트레이 건수를 올리지 않는다(Rust 시험).
- A5: 변경 신호 구독자가 뒤처지면 빈 `res`(전체 다시 읽기) 신호를 받는다, SSE 가 `event: change` 로 흘린다(Rust 시험).
- A6: 회귀 없음 — PROJECT.md 검증 명령 PASS, 5초 새로 고침은 그대로.
- A7: 5초 새로 고침을 끈 상태에서 변경 신호 한 번 뒤 대화 본문 · 목록 · 배지 · 내 활동 · 연결 상태가 일관된다(실화면 — masterPlan Phase 1 후보 줄 성공 기준).

## 결정

라운드 1 — 사용자 "권장안으로 진행, D39 ② 개정 승인"(2026-10-19). 미해소 1 ~ 5 를 모두 권장안으로 확정 = 공유 이해 도달(intent 승인).

1. **신호 채널 = 알림과 별개인 변경 신호**(SSE `event: change`) — 알림 설정 · 집중과 무관하게 항상. masterPlan D39 ② 문구 개정("알림이" → "변경 신호가", 사용자 승인).
   기각: `Notice` 에 `res` 추가(알림을 끈 사용자에게 신호가 없다). 근거: Notion 도 레코드 변경(`versions/*`)과 사용자 알림(`notification/{userId}`)을
   같은 연결의 다른 구독 키로 가른다(`D:/dev/git/notion/rea/features/sync-storage.md` · `collaboration.md` [관찰-런타임], notion 세션 재확인).
   Notion 알림 설정이 데이터 신호까지 막는지는 확인 필요(설정 변경 캡처 없음). 집중(D17)은 상대의 새 실행 요청만 막고 화면 갱신은 막지 않는다.
2. **원천 = 원격 이벤트만**, 이 PC 변경 전용 신호는 5초 축소 Task 에서. 구현자 해석(사용자 미확인): "원격 이벤트" = 페어링된 상대 요청 처리
   (`crates/agent/src/chat.rs` `fn handle` — 대화가 있는 요청 전부, 알림을 내지 않는 편집 · 삭제 · 진행 포함) + 페어링 + 메시지 알림 이벤트.
3. **버전 건너뛰기 없음** — 중복은 웹UI 합치기 · `Latest` 로. 측정으로 중복 다시 읽기가 보이면 `ver`(선택 필드라 나중에 붙여도 호환).
4. **Task = `1u.scoped-refresh`**(브랜치 `task/1u-scoped-refresh`).
5. **뒤처짐 포함** — 놓치면 빈 `res` 변경 신호 한 번 = 전체 다시 읽기.

## 미해소

- 없음

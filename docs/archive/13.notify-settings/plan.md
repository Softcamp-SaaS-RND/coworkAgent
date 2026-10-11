# plan — 13.notify-settings

> 근거: [intent](intent.md)(승인 2026-10-11 — 사용자 지시 + 확인 답 2건). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).

## 목표

자동 왕복 알림 묶기(시작 · 끝) · 종류별 알림 · 미리보기 끄기 · 자동 왕복 상한 기본값 설정.

### 성공 기준(intent 사본)

- A1: 자동 왕복 대화에서 양쪽 PC 알림은 시작 1건 · 자동 멈춤 1건뿐이다(기본). 모드 "매번" 이면 글마다, "끄기" 면 없다.
- A2: 종류별 끄기 · 미리보기 끄기가 알림 · 트레이 건수에 반영되고, 재시작 뒤에도 유지된다.
- A3: 상한을 주지 않은 자동 왕복 대화는 설정한 기본값을 쓴다. 범위 밖 값은 저장되지 않는다(400).
- A4: 루트 `PROJECT.md` 검증 명령이 통과한다.

## 설계

### 이벤트(`an-net`)

- `PeerEvent::Message` 에 `kind: MessageKind{Request, Reply, RelayStart, RelayStep, RelayEnd}`. `RelayEnd` 의 `preview` 는 멈춘 사유(이 PC 가
  만든 문구).

### 내는 곳(`an-agent`)

- 받은 쪽: 사람 요청 `Request`, 에이전트발 요청 — 대화의 첫 에이전트발 요청이면 `RelayStart`, 아니면 `RelayStep`. 재개 알림(`relay{on:true}`)
  `RelayStart`. 상한 기록(`post` agent) `RelayEnd`("상한 도달"). 사람 글 미러 `Request`.
- 연 쪽: 사람 요청의 답 `Reply`, 내 에이전트 답에 온 답 `RelayStep`. 자동 멈춤 `RelayEnd` — 상한 도달 · 상대 실행 실패 · 내 에이전트 실행
  실패 · 전달 거절 · 전달 확인 안 됨 · 상대가 멈춤. 사람이 멈춘 것(멈춤 · 글쓰기)은 내지 않는다. 에이전트 답 미러 `Reply`.

### 거르기 · 설정(`agentnetwork-app` `notify`)

- 설정 키(`settings` 표): `notify_inbound`(기존) · `notify_pair` · `notify_request` · `notify_reply`(기본 `1`) · `notify_relay`
  (`start_end` 기본 · `every` · `off`) · `notify_preview`(기본 `1`) · `relay_cap_default`(1 ~ 50, 기본 10).
- `Notifier` 가 설정을 들고 `on_event` 에서 거른다(걸러진 것은 건수에도 안 든다). `RelayStep` 은 `every` 일 때만, `RelayStart` · `RelayEnd` 는
  `off` 가 아닐 때. 미리보기 끄면 `message` 본문 = 이름 · PC 만.
- `GET /api/settings` → 전부. `PUT /api/settings` — 모든 항목 선택(준 것만 바꿈), 하나라도 틀리면 400 · 아무것도 안 바꿈, 한 트랜잭션.
- `POST /api/conversations` 의 `relay_cap` 이 없으면 `relay_cap_default`(`an_agent::chat::default_relay_cap(conn)`).

### 웹UI

- 설정 → 알림: 전체(기존) 아래 페어링 · 받은 요청 · 받은 답 체크, 자동 왕복 알림 선택(시작·끝 / 매번 / 끄기), 미리보기 체크, 자동 왕복 상한
  기본값(숫자). 저장 실패면 서버 값으로 되돌림. 새 대화의 상한 칸 처음 값 = 기본값.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 이벤트 종류 · 내는 곳 · 거르기 · 설정 API · 상한 기본값 | 테스트(notifier · settings_api · relay · chat_api) | E1 ~ E6 |
| 2 | 웹UI 설정 | node 회귀 | E7 · E8 |

## 평가 항목

- [ ] E1: A4 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A4 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2 — 거르기(notifier 9건: 기존 6 + kinds_filtered · relay_modes · preview_off)
  CHECK: cargo test -p agentnetwork-app --test notifier
  EXPECT: test result: ok. 9 passed; 0 failed
  EVIDENCE: pending
- [ ] E4: A2·A3 — 설정 API(settings_api 3건: 기존 2 + prefs_patch_validated_and_persisted)
  CHECK: cargo test -p agentnetwork-app --test settings_api
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A1 — 자동 왕복 알림 종류(relay 9건: 기존 7 + relay_events_start_step_end · relay_events_first_run_fails)
  CHECK: cargo test -p an-agent --test relay
  EXPECT: test result: ok. 9 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A3 — 상한 기본값(chat_api 6건: 기존 5 + relay_cap_default_from_settings)
  CHECK: cargo test -p agentnetwork-app --test chat_api
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A4 — 화면 로직 node 테스트 회귀 없음
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E8: A1·A2·A3 — 실화면(수동): 설정 → 알림 항목 · 자동 왕복 시작 · 끝 알림 · 미리보기 끄기 · 상한 기본값
  EVIDENCE: pending

## 종결 시 할 일

- 매뉴얼 `an_net` · `an_agent` · `an_app` · PRODUCT(알림) · masterPlan D29 보강 · `#13` · 19 acceptance 에 E8 TC.

## 개정 기록

- 개정 1(2026-10-11, 적대적 평가 라운드 1 반영): 자동 왕복을 여는 사람 요청에 `message.relay = true` — 받은 쪽은 그 요청을 `RelayStart`,
  뒤 에이전트발 요청은 모두 `RelayStep`. 연 쪽은 자동 왕복 중 사람 요청에 온 첫 답을 `RelayStart`. 받은 쪽 내 에이전트가 에이전트발 요청에
  실패하면 `RelayEnd`. 설정 PATCH 는 `notify_inbound` 까지 한 트랜잭션 · 한 상태 잠금. E5 8 → 9건.
- 개정 2(2026-10-11, 적대적 평가 라운드 2 반영): `message.relay` 는 연 쪽 에이전트 이름(`Option`). 받은 쪽은 `relay` · `by_agent` 를 받으면
  그 대화를 자동 왕복으로 기록(`mark_relayed` — `relay_agent` 만)하고, 자동 왕복을 연 첫 사람 요청의 실행 실패에도 `RelayEnd`. 웹UI 는 받은 쪽
  상대 에이전트 이름을 보인다.

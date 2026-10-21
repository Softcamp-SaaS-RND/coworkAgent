# an_net

## 개요

`an-net`(`crates/net`) — comm Layer. 설치 신원(ed25519) · TLS 1.3 raw public key 상호 인증 · 피어 프레임 · 페어링 노드 · 초대
텍스트. UI · HTTP 를 모른다. 의존: `an-core`(피어 표).

## 공개 계약

- `identity::Identity::load_or_create(path)` — 없으면 생성(임시 파일 → 이름 바꾸기), 손상이면 `InvalidData`(재생성 안 함).
  `spki()` · `fingerprint()`(SHA-256(SPKI) 앞 16바이트, `7F3A 19C2 …`). `identity::fingerprint(spki)`.
- `tls::server_config(id)` — 클라이언트 키 필수 · 아무 키나(인가는 연결 뒤). `tls::client_config(id, Option<spki>)` ·
  `client_config_pin(id, Pin::{Any, Key, Fingerprint})`. TLS 1.3 전용, 서명 스킴 ED25519 하나.
- `proto` — 프레임 = 4바이트 BE 길이 + JSON, `MAX_FRAME` 64 KiB. `Request::{Hello{name,host,port,round,code}, Approve{round},
  Remove, Ping{port}, Agents, Message{conv,msg,agent,text}, Reply{conv,msg,reply_to,text,ok,turns?,duration_ms?,lc?}, Post{conv,msg,author,text,exec,reply_to?,ok?,turns?,duration_ms?,lc?}}`
  (`Message` 에도 `lc?` — 양쪽 공통 순서 · `by_agent?` — 자동 왕복의 에이전트발 실행 요청, `relay?` — 자동 왕복을 여는 사람 요청(연 쪽 에이전트 이름 — 받은 쪽 기록 · 알림 묶기), `Relay{conv, on}` — 자동 왕복 멈춤 · 재개 알림, `Post` = 실행 없는 사람 글 · 받은 쪽 실행 요청과 그 답의 미러, `Message` · `Post` 의 `via?` — 글의 출처 `cli`, 이전 버전은 모르는 필드라 무시)(대화 요청은
  페어링된 상대만, `Reply` 실행 정보는 없으면 `None`. `Reply` · `Post` 의 `model?` · `effort?` — 답을 실행한 값, 받은 쪽은 `run_opts` 글자 규칙에 맞는
  것만 남긴다 · D32 이전 버전 대비 없음. #18: `Message` · `Post` 의 `request?`(사람 글 종류 — 요청), `Message.mention?`(멘션 — 부른 받는 PC 에이전트,
  `agent` 는 늘 대화 에이전트), `ReqState{conv, msg, state, note?}`(#1d — `Done` 대체) — 받은 요청의 상태(`wait` · `accepted` · `rejected` ·
  `done`) · 의견(여러 줄 — 줄바꿈은 U+2028 로 실어 보냄, #1z)을 쓴 쪽에, 페어링된 상대만. #1f: `Edit{conv, msg, text}` · `Delete{conv, msg}` — 쓴 쪽 사람이 자기 사람 글을 고쳤다 ·
  지웠다(이력 정리 — 받는 쪽은 그 상대가 쓴 사람 글에만, 실행 · 알림 없음), 페어링된 상대만 · `ids_ok` 로 uuid 꼴 검사 · D32 이전 버전 대비 없음. #1b: `Progress{conv, msg, state, ahead}` — 받는 쪽이 상대 실행 요청의
  차례(`queued` · `running`)를 보낸 쪽에, 페어링된 상대만 · `Ping.port` — 보낸 쪽 피어 수신 포트, 0 = 주소 바꾸지 않음. #1c: `Progress.detail?` —
  `StepDetail{turns, tools: BTreeMap<이름, 횟수>, secs}` 실행 중 단계 요약(턴 · 도구 이름별 횟수 · 경과 초만 — 입력 · 경로 · 모델 글 없음),
  `StepDetail::cleaned()`(받은 쪽 — 이름 `clean_name` 뒤 `MAX_STEP_NAME` 24자 · 빈 이름 버림 · `MAX_STEP_TOOLS` 16개), `Status{conv, msg}` — 보낸 쪽이
  자기 실행 요청 글의 지금 상태를 묻는다, 페어링된 상대만). #1m: `Context{conv, msg, op, text}` — 컨텍스트 압축 · 비우기(`op` = `CTX_OPS` `compact` · `auto_compact` ·
  `clear` 요청 · `done`(요약) · `failed`(사유 코드) 결과, `ids_ok` 대상) · `Reply.ctx?` · `Post.ctx?`(답 세션의 창 사용량, 없으면 싣지 않음) · #1o 공유: `Share{conv, title, agent, owner_dir, other}` · `SharePost{conv, msg: ShareMsg{uuid, author, text, lc, at, owner, via, agent, ok, edited}}` · `ShareDelete{conv, msg}` · `ShareEnd{conv}` · `ShareNotice{conv, n}`(모두 `ids_ok` 대상 — `SharePost` 는 `msg.uuid`) · #1w `Leave{conv}`(대화 나가기 — `ids_ok` · `conv()` 대상) · #1x `Clear{conv, id}`(대화 기록 모두 지우기 — `id` = 그 지우기의 uuid, 받는 쪽이 적용한 `id` 를 남겨 다시 온 같은 지우기는 `Ok` 만, `ids_ok`(둘 다 uuid) · `conv()` 대상). `Response::{Hello{name, host, approved}, Ok, Pong{focus, ctx, share, leave, clear}, Agents{names}, Status{state,
  ahead}, Error{code}}`(`Pong.focus` — 답한 PC 의 집중, 설정을 못 읽으면 거짓. `Pong.ctx` — `Context` 를 앎(#1m, 0.4.0 은 없음 = 거짓). `Pong.share` — 공유 프레임을 앎(#1o, 0.5.0 은 없음 = 거짓). `Pong.leave` — `Leave` 를 앎(#1w, 0.8.0 은 없음 = 거짓). `Pong.clear` — `Clear` 를 앎(#1x, 1.0.1 은 없음 = 거짓). `Status.state` = `queued`(`ahead` = 앞 사람 수) · `running` · `done` ·
  `unknown`)(`approved` = 초대 생성 측이 이 hello 를 초대 코드로 자동 승인 — 보낸 쪽은
  같은 세대의 상대 승인으로 기록)(`not_paired` · `unknown` · `stale` · `busy` · `closed` ·
  `bad_request` · `unknown_agent` · `unsupported` — 받는 쪽에 처리기가 없음 · `focus` — 받는 PC 가 집중 중 · `invalid` — 대화 요청 id 가 UUID 꼴 아님 · `left` — 받는 쪽이 그 대화를 나갔거나 지웠다(#1w) ·
  `retry` — 받는 쪽이 처리 중 오류로 잠시 처리하지 못함, 보낸 쪽은 거절이 아니라 다시 보낸다(#1c)). `clean_name` — 제어 문자 · 서식 문자(Cf, `is_format`)
  제거 · 64자. `is_uuid` · `Request::ids_ok()`(Message · Done · Progress · Status · Reply · Post · Relay 의 `conv` · `msg` · `reply_to`, #1a). `clean_code(code)` — 상대가 준 오류 코드는 영소문자 · `_`
  1 ~ 32 자만, 그 밖 `unknown`(저장 · 표시 전에).
- `node::Node::new(identity, conn, me)` · `serve(listener)`(출처 IP 당 4 · 전체 32 · 연결당 3초) · `open_window(d) -> 코드`(새 창마다 IP 별 틀린
  초대 코드 셈을 비운다 — 한 창에서 `MAX_CODE_MISSES` 10회 틀린 IP 는 그 창 동안 `closed`, #1a) ·
  `open_invite(ip, d) -> Invite` · `add(addr, code)` · `add_invite(&invite)`(지문 고정) · `approve(id)` · `remove(id) -> 통지 여부` ·
  `ping(id) -> 상대 집중`(받은 `Pong` 을 접속 상태로 — `set_presence`) · `knows_context(id) -> 상대가 Context 를 앎`(#1m — 같은 Ping) · `knows_share(id)`(#1o) · `knows_leave(id)`(#1w) · `knows_clear(id)`(#1x) · `MessageKind::Share`(원 상대 관람자 알림) · `MessageKind::Clear`(상대가 대화 기록을 모두 지웠다 — #1x) · `refresh(id, within) -> 상대 집중`(#1c — 마지막 통신이 `within` 초 안이면 보내지
  않고 저장된 집중, 아니면 `ping`) · `announce()`(#1c — 시작 때 페어링된 상대마다 `ping` 한 번 · 상대별 작업 · 실패는 넘어감. 주기 확인은 없다) ·
  `peers()` · `set_peer_meta(id, group, model, effort)` · `set_peer_tile(id, alias, icon)` · `send(addr, pinned, req)`(응답을 받으면 그 키의 `peers::touch`
  — 기록 실패는 보내기를 실패로 만들지 않음) · `on_request(f)`(대화 요청 처리기) · `on_seen(f)`(#1c — 페어링된 상대의 `Ping` 을 받을 때 상대 id 로
  부름 · 한 번만 설정 · 막히면 안 된다) · `on_event(f)` —
  `PeerEvent::{PairRequest{name,host,fingerprint}, AutoPaired{name,host,fingerprint}, Approved{name,host}, Message{name,host,agent,preview,kind}}`
  (`MessageKind::{Request, Reply, RelayStart, RelayStep, RelayEnd, Ask, AskState}` — 알림 거르기용, `RelayEnd` 의 `preview` 는 멈춘 사유, `Ask` =
  받은 요청 · `AskState` = 내 요청의 상태가 바뀜 — #1d)
  를 응답 전에 부른다(콜백은 막히면 안 된다 — 느린 일은 큐로). `bind_peer_port() -> (listener, Option<Holder>)` — `0.0.0.0:47100`, 사용 중이면
  임의 포트 + 잡은 프로세스(`Holder{name, pid}`) · `bind_peer_port_on(port)` · `port_holder(port)`(Windows `netstat -ano -p TCP` + `tasklist`, Unix
  `lsof` — 못 찾으면 `None`, 부르는 쪽은 "알 수 없음" · PID 0) · `set_peer_holder(h)` · `peer_holder()` · `peer_port_note()`("피어 포트 47100 사용 중 —
  {이름}(PID {n}) · 임의 포트 {p} 로 받는다") · `open_connections()`(피어 수신 중인 연결 수 — 전체 32 중 쓰는 수, #1b).
- 인가 규칙(`Node::answer`): 미페어링 상대는 hello · approve · remove 만. 처음 보는 키의 hello 는 받기 창의 초대 코드가 맞을 때
  1건 — 내 승인을 자동으로 하고(D8 개정 — 초대 생성 측 승인 생략, 위험 수용) `AutoPaired` 를 낸다. 아는 키의 hello 는 양쪽 승인 초기화 +
  새 세대(사람 승인 — `PairRequest`). approve 는 같은 세대만. ping 은 페어링(my_ok ∧ their_ok)만 — `port` 가 0 이 아니고 (연결 출처 IP, `port`) 가 저장 주소와 다르면 `peers::set_addr` 로
  바꾼다(승인 · 세대 그대로, #1b — 임의 포트 · 바뀐 IP 를 따라감). `ping()` 은 내 수신 포트를 싣는다. 접속 시각(#1c — 통신할 때만): 페어링된
  상대의 대화 요청 · `Ping` 을 받을 때 `peers::touch`(기록 실패는 받기를 막지 않음), `Ping` 이면 `on_seen` 도 부른다.
- `invite::Invite::{to_text, parse}` — 토큰 `coworkagent-invite:v1/{주소}/{코드}/{지문 hex32}`, 메신저 머리말 · 줄바꿈을 견딘다. 안내 줄은
  웹UI "초대 수락" 에 붙여 넣으라고 한다.

## 사용 예

```rust
let node = Node::new(Identity::load_or_create(&paths.identity_file())?, db::open(&paths.db_file())?, me);
tokio::spawn(node.clone().serve(bind_peer_port().await?));
// 초대 생성 측: 초대 텍스트를 상대에게(그 초대로 온 처음 보는 키는 자동 승인)
let text = node.open_invite(lan_ip, PAIRING_WINDOW).to_text(&me.name, &me.host);
// 초대 수락 측: 붙여 넣은 텍스트로 추가 + 내 승인 → 양쪽 페어링
let p = node.add_invite(&Invite::parse(&pasted).unwrap()).await?;
node.approve(p.id).await?;
```

## 금지 사항

- 비밀키 · 초대 코드를 로그 · 원격 응답에 싣지 않는다. 원격 이름은 `clean_name` 을 거쳐 저장한다.

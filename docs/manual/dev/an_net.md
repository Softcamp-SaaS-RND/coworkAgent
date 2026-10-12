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
  Remove, Ping, Agents, Message{conv,msg,agent,text}, Reply{conv,msg,reply_to,text,ok,turns?,duration_ms?,lc?}, Post{conv,msg,author,text,exec,reply_to?,ok?,turns?,duration_ms?,lc?}}`
  (`Message` 에도 `lc?` — 양쪽 공통 순서 · `by_agent?` — 자동 왕복의 에이전트발 실행 요청, `relay?` — 자동 왕복을 여는 사람 요청(연 쪽 에이전트 이름 — 받은 쪽 기록 · 알림 묶기), `Relay{conv, on}` — 자동 왕복 멈춤 · 재개 알림, `Post` = 실행 없는 사람 글 · 받은 쪽 실행 요청과 그 답의 미러, `Message` · `Post` 의 `via?` — 글의 출처 `cli`, 이전 버전은 모르는 필드라 무시)(대화 요청은
  페어링된 상대만, `Reply` 실행 정보는 없으면 `None`. `Reply` · `Post` 의 `model?` · `effort?` — 답을 실행한 값, 받은 쪽은 `run_opts` 글자 규칙에 맞는
  것만 남긴다 · D32 이전 버전 대비 없음. #18: `Message` · `Post` 의 `request?`(사람 글 종류 — 요청), `Message.mention?`(멘션 — 부른 받는 PC 에이전트,
  `agent` 는 늘 대화 에이전트), `Done{conv, msg, done}` — 받은 요청의 완료 표시를 쓴 쪽에, 페어링된 상대만) · `Response::{Hello{name, host, approved}, Ok, Pong{focus}, Error{code}}`(`Pong.focus` — 답한 PC 의 집중, 설정을 못
  읽으면 거짓)(`approved` = 초대 생성 측이 이 hello 를 초대 코드로 자동 승인 — 보낸 쪽은
  같은 세대의 상대 승인으로 기록)(`not_paired` · `unknown` · `stale` · `busy` · `closed` ·
  `bad_request` · `focus` — 받는 PC 가 집중 중). `clean_name` — 제어 문자 제거 · 64자. `clean_code(code)` — 상대가 준 오류 코드는 영소문자 · `_`
  1 ~ 32 자만, 그 밖 `unknown`(저장 · 표시 전에).
- `node::Node::new(identity, conn, me)` · `serve(listener)`(출처 IP 당 4 · 전체 32 · 연결당 3초) · `open_window(d) -> 코드` ·
  `open_invite(ip, d) -> Invite` · `add(addr, code)` · `add_invite(&invite)`(지문 고정) · `approve(id)` · `remove(id) -> 통지 여부` ·
  `ping(id) -> 상대 집중`(받은 `Pong` 을 접속 상태로) · `watch_presence()`(시작 직후와 `PRESENCE_EVERY` 30초마다 페어링된 상대 모두에 ping — 놓친
  주기는 몰아 보내지 않는다) · `peers()` · `set_peer_meta(id, group, model, effort)` · `set_peer_tile(id, alias, icon)` · `send(addr, pinned, req)` · `on_event(f)` —
  `PeerEvent::{PairRequest{name,host,fingerprint}, AutoPaired{name,host,fingerprint}, Approved{name,host}, Message{name,host,agent,preview,kind}}`
  (`MessageKind::{Request, Reply, RelayStart, RelayStep, RelayEnd}` — 알림 거르기용, `RelayEnd` 의 `preview` 는 멈춘 사유)
  를 응답 전에 부른다(콜백은 막히면 안 된다 — 느린 일은 큐로). `bind_peer_port()` — `0.0.0.0:47100`, 사용 중이면 임의 포트.
- 인가 규칙(`Node::answer`): 미페어링 상대는 hello · approve · remove 만. 처음 보는 키의 hello 는 받기 창의 초대 코드가 맞을 때
  1건 — 내 승인을 자동으로 하고(D8 개정 — 초대 생성 측 승인 생략, 위험 수용) `AutoPaired` 를 낸다. 아는 키의 hello 는 양쪽 승인 초기화 +
  새 세대(사람 승인 — `PairRequest`). approve 는 같은 세대만. ping 은 페어링(my_ok ∧ their_ok)만.
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

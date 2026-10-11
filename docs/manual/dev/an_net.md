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
  (`Message` 에도 `lc?` — 양쪽 공통 순서 · `by_agent?` — 자동 왕복의 에이전트발 실행 요청, `relay?` — 자동 왕복을 여는 사람 요청(연 쪽 에이전트 이름 — 받은 쪽 기록 · 알림 묶기), `Relay{conv, on}` — 자동 왕복 멈춤 · 재개 알림, `Post` = 실행 없는 사람 글 · 받은 쪽 실행 요청과 그 답의 미러)(대화 요청은
  페어링된 상대만, `Reply` 실행 정보는 없으면 `None` — 이전 버전 호환) · `Response::{Hello, Ok, Pong, Error{code}}`(`not_paired` · `unknown` · `stale` · `busy` · `closed` ·
  `bad_request`). `clean_name` — 제어 문자 제거 · 64자.
- `node::Node::new(identity, conn, me)` · `serve(listener)`(출처 IP 당 4 · 전체 32 · 연결당 3초) · `open_window(d) -> 코드` ·
  `open_invite(ip, d) -> Invite` · `add(addr, code)` · `add_invite(&invite)`(지문 고정) · `approve(id)` · `remove(id) -> 통지 여부` ·
  `ping(id)` · `peers()` · `send(addr, pinned, req)` · `on_event(f)` — `PeerEvent::{PairRequest{name,host,fingerprint}, Approved{name,host}, Message{name,host,agent,preview,kind}}`
  (`MessageKind::{Request, Reply, RelayStart, RelayStep, RelayEnd}` — 알림 거르기용, `RelayEnd` 의 `preview` 는 멈춘 사유)
  를 응답 전에 부른다(콜백은 막히면 안 된다 — 느린 일은 큐로). `bind_peer_port()` — `0.0.0.0:47100`, 사용 중이면 임의 포트.
- 인가 규칙(`Node::answer`): 미페어링 상대는 hello · approve · remove 만. 처음 보는 키의 hello 는 받기 창의 초대 코드가 맞을 때
  1건. 아는 키의 hello 는 양쪽 승인 초기화 + 새 세대. approve 는 같은 세대만. ping 은 페어링(my_ok ∧ their_ok)만.
- `invite::Invite::{to_text, parse}` — 토큰 `coworkagent-invite:v1/{주소}/{코드}/{지문 hex32}`, 메신저 머리말 · 줄바꿈을 견딘다.

## 사용 예

```rust
let node = Node::new(Identity::load_or_create(&paths.identity_file())?, db::open(&paths.db_file())?, me);
tokio::spawn(node.clone().serve(bind_peer_port().await?));
// 받는 쪽: 초대 텍스트를 메신저로
let text = node.open_invite(lan_ip, PAIRING_WINDOW).to_text(&me.name, &me.host);
// 보내는 쪽: 붙여 넣은 텍스트로 추가 + 내 승인
let p = node.add_invite(&Invite::parse(&pasted).unwrap()).await?;
node.approve(p.id).await?;
```

## 금지 사항

- 비밀키 · 초대 코드를 로그 · 원격 응답에 싣지 않는다. 원격 이름은 `clean_name` 을 거쳐 저장한다.

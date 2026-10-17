"""웹UI 눈 확인 · 문서 캡처용 가짜 API — 정적 파일(crates/app/web) + 지어낸 데이터(가상 인물 · PC · 대화). 실제 데이터 디렉터리는 건드리지 않는다.
python mock.py <web dir> <port> [dark|light]   — 캡처는 shots.mjs. #1o mock8 을 #1p 에서 리포로 옮김(글꼴 · 아이콘 경로 추가)."""
import json
import sys
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlparse, parse_qs

WEB = Path(sys.argv[1])
PORT = int(sys.argv[2])
NOW = int(time.time())


PEERS = [
    {"id": 2, "name": "jhlee", "alias": "이준호", "host": "QA-LAB-02", "fingerprint": "AB12 CD34 EF56 7890", "addr": "192.168.0.12:47100",
     "paired": True, "my_ok": True, "their_ok": True, "group": "QA", "model": None, "effort": None, "icon": None, "online": True, "focus": False, "seen_at": NOW - 5, "pos": 0},
    {"id": 3, "name": "yuna", "alias": "최유나", "host": "DEV-PC-07", "fingerprint": "1122 3344 5566 7788", "addr": "192.168.0.15:47100",
     "paired": True, "my_ok": True, "their_ok": True, "group": None, "model": None, "effort": None, "icon": None, "online": False, "focus": False, "seen_at": NOW - 3600, "pos": 1},
    {"id": 4, "name": "build", "alias": None, "host": "BUILD-SRV", "fingerprint": "9A9B 9C9D 9E9F 9091", "addr": "192.168.0.20:47100",
     "paired": True, "my_ok": True, "their_ok": True, "group": None, "model": None, "effort": None, "icon": None, "online": True, "focus": False, "seen_at": NOW - 40, "pos": 2},
]
AGENTS = [{"id": 1, "name": "review", "folder": "D:\\work\\shop-api", "tools": "Read,Grep,Glob", "prompt": "", "group": None}]
PEER_AGENTS = ["guide", "qa-check"]


def conv(id, peer, agent, dir, title, state, unread=0, starred=False, relay=None, at=0):
    c = {"id": id, "uuid": f"u{id}", "peer_id": peer, "agent": agent, "dir": dir, "relay_agent": None, "relay_cap": None,
         "relay_count": None, "relay_on": False, "relay_blocked": False, "title": title, "last_state": state,
         "last_at": NOW - at, "starred": starred, "unread": unread, "pos": -id}
    if relay:
        c.update(relay_agent=relay[0], relay_cap=relay[1], relay_count=relay[2], relay_on=True)
    return c


CONVS = [
    conv(1, 2, "guide", "out", "결제 모듈 배포 절차", "running", starred=True, at=12),
    conv(2, 2, "review", "in", "로그인 API 오류 코드 정리", "running", at=60),
    conv(3, 2, "guide", "out", "릴리스 노트 초안 검토", "running", relay=("review", 10, 3), at=120),
    conv(4, 2, "guide", "out", "HTML 성능 보고서", "done", at=300),
    conv(5, 2, "review", "in", "테스트 케이스 누락 확인", "done", unread=1, at=900),
    conv(6, 3, "review", "in", "주문 API 응답 필드 추가", "done", unread=1, at=1500),
    conv(7, 3, "front", "out", "화면 테마 토큰 위치", "done", at=4000),
    conv(8, 4, "ci", "out", "야간 빌드 실패 원인", "done", at=7200),
    conv(9, 3, "front", "shared", "디자인 토큰 이름 규칙", "done", at=2500),
    conv(10, 3, "front", "shared", "아이콘 크기 기준", "done", at=9000),
]
# #1o 공유 — 1 에 관람자(최유나), 2 는 원 상대가 관람자 1명에게 공유, 9 · 10 = 최유나가 공유한 대화(10 은 공유 해제됨).
CONVS[0]["observers"] = [3]
CONVS[1]["watchers"] = 1
CONVS[8]["share"] = {"other": "mspark", "owner_dir": "out", "ended": None}
CONVS[9]["share"] = {"other": "mspark", "owner_dir": "out", "ended": True}


def msg(id, author, mine, text, state="done", at=0, reply_to=None, **kw):
    m = {"id": id, "author": author, "text": text, "state": state, "created_at": NOW - at, "reply_to": reply_to, "turns": None,
         "duration_ms": None, "mine": mine, "exec": author == "human", "via": None, "model": None, "effort": None, "saved": False,
         "request": False, "req_state": None, "req_note": None, "req_at": None, "agent": None, "refused": None,
         "edited_at": None, "ctx_tokens": None, "ctx_op": None, "progress": None, "outbox": None, "editable": False}
    m.update(kw)
    if m["request"] and m["req_state"] is None:
        m["req_state"] = "wait"
    return m


ANSWER = """## 결제 모듈 배포 절차

결제 모듈은 **스테이징 검증 뒤** 운영에 올립니다.

1. 릴리스 브랜치 생성
2. 스테이징 배포
3. 승인 뒤 운영 배포

| 환경 | 브랜치 | 승인 |
|---|---|---|
| 스테이징 | `release/*` | 팀 리드 |
| 운영 | `main` | 승인자 |

### 롤백

- 문제가 발생하면 이전 태그로 즉시 롤백합니다.

```
git revert <tag>
cargo build --release
```"""

MESSAGES = {
    9: [
        msg(91, "human", False, "버튼 색 토큰 이름은 어떤 규칙으로 지어?", at=2700, owner=True),
        msg(92, "agent", False, "`--{역할}-{상태}` 꼴입니다 — 예 `--accent-hover`. 테마별 값은 `--dark-*` 로 따로 둡니다.", at=2600, owner=False, turns=2, duration_ms=12000),
        msg(93, "human", False, "글자 크기 토큰도 같은 규칙이야?", at=2550, owner=True),
        msg(94, "agent", False, "글자 크기는 `--fs-{크기}`(xs · sm · md · lg)입니다.", at=2500, owner=False, turns=1, duration_ms=7000),
    ],
    10: [msg(101, "human", False, "아이콘은 몇 px 기준이야?", at=9100, owner=True),
         msg(102, "agent", False, "16px 한 벌입니다.", at=9000, owner=False, turns=1, duration_ms=5000)],
    1: [
        msg(11, "human", True, "결제 모듈을 운영에 배포하는 절차와 롤백 방법을 정리해 줘. release 브랜치 기준으로.", at=90000),
        msg(12, "agent", False, ANSWER, at=89800, reply_to=11, turns=3, duration_ms=41000, model="sonnet", effort="high", saved=True, ctx_tokens=42300),
        msg(13, "human", True, "@guide 승인자는 누구로 지정돼 있어?", state="running", at=12),
    ],
    2: [msg(21, "human", False, "401 · 403 · 429 응답 코드를 표로 정리해 주세요", state="running", at=60, request=True)],
    3: [
        msg(31, "human", True, "릴리스 노트 초안을 검토해 줘", at=400),
        msg(32, "agent", False, "초안의 변경 항목 세 개를 확인했습니다.", at=300, reply_to=31, turns=2, duration_ms=18000, model="sonnet", effort="high"),
    ],
    4: [
        msg(41, "human", True, "HTML 성능 보고서를 만들어 줘", at=700),
        msg(42, "agent", False, "<!doctype html><html><body><h1>성능 보고서</h1><p>p95 120ms</p></body></html>", at=650, reply_to=41,
            turns=4, duration_ms=52000, model="sonnet", effort="high", saved=True, ctx_tokens=650000),
    ],
    6: [msg(61, "human", False, "주문 조회 API 응답에 배송 예정일 필드를 추가해 주세요", at=1500, request=True),
        msg(62, "context", True, "", at=1400, ctx_op="clear")],
    7: [msg(71, "human", True, "다크 테마 색 토큰은 어디서 정의돼?", at=4100),
        msg(72, "agent", False, "`web/app.css` 맨 위 `:root` 블록입니다.", at=4000, reply_to=71, turns=1, duration_ms=9000, model="sonnet", effort="high", saved=True),
        msg(73, "context", True, "", state="error", at=3900, ctx_op="compact", refused="unsupported")],
    8: [msg(81, "human", True, "어젯밤 야간 빌드가 왜 실패했어?", at=7300),
        msg(82, "agent", False, "통합 테스트 1건이 시간 초과로 실패했습니다.", at=7200, reply_to=81, turns=2, duration_ms=15000, model="sonnet", effort="high", ctx_tokens=830000)],
    5: [msg(51, "human", False, "테스트 케이스 누락이 있는지 확인해 주세요", at=1000),
        msg(52, "agent", True, "누락 2건을 찾았습니다.", at=950, reply_to=51, turns=2, duration_ms=22000, model="sonnet", effort="high", ctx_tokens=120000),
        msg(53, "context", True, "테스트 케이스 누락 확인 — 결제 · 환불 경계값 2건 누락(`tests/refund.rs`), 확인한 파일 tests/*.rs 12개. 남은 질문: 부분 환불 시험 범위.", at=900, ctx_op="auto_compact"),
        msg(54, "human", False, "부분 환불 시험도 봐 주세요", at=800),
        msg(55, "agent", True, "부분 환불 시험은 1건 있습니다 — 경계값을 더하면 좋겠습니다.", at=780, reply_to=54, turns=1, duration_ms=8000, model="sonnet", effort="high", ctx_tokens=35000)],
}

ACTIVITY = {
    "items": [
        {"kind": "message", "at": NOW - 60, "unseen": True, "conversation": {"id": 2, "peer_id": 2, "agent": "review", "dir": "in", "title": "로그인 API 오류 코드 정리"},
         "message": msg(21, "human", False, "401 · 403 · 429 응답 코드를 표로 정리해 주세요", request=True)},
        {"kind": "relay_start", "at": NOW - 400, "unseen": True, "conversation": {"id": 3, "peer_id": 2, "agent": "guide", "dir": "out", "title": "릴리스 노트 초안 검토"}, "detail": "릴리스 노트 초안을 검토해 줘"},
        {"kind": "message", "at": NOW - 1000, "unseen": True, "conversation": {"id": 5, "peer_id": 2, "agent": "review", "dir": "in", "title": "테스트 케이스 누락 확인"},
         "message": msg(51, "human", False, "테스트 케이스 누락이 있는지 확인해 주세요")},
        {"kind": "message", "at": NOW - 1500, "unseen": True, "conversation": {"id": 6, "peer_id": 3, "agent": "review", "dir": "in", "title": "주문 API 응답 필드 추가"},
         "message": msg(61, "human", False, "주문 조회 API 응답에 배송 예정일 필드를 추가해 주세요", request=True)},
    ],
    "unseen": 4,
    "unseen_all": 4,
    "upto": NOW,
}

SETTINGS = {"ui_rail_order": "peers_first", "notify_inbound": True, "notify_pair": True, "notify_request": True, "notify_reply": True, "notify_relay": "start_end",
            "notify_preview": True, "notify_paused": False, "relay_cap_default": 10, "claude_model": "sonnet", "claude_effort": "high",
            "ui_theme": sys.argv[3] if len(sys.argv) > 3 else "dark", "ui_side_width": 272, "me_alias": "박지훈", "me_icon": None, "focus": False,
            "ui_folded": [], "autostart": True, "auto_compact": True}


class H(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def send(self, code, body=None, ctype="application/json"):
        data = b"" if body is None else (body if isinstance(body, bytes) else json.dumps(body, ensure_ascii=False).encode())
        self.send_response(code)
        if data:
            self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        u = urlparse(self.path)
        p = u.path
        q = parse_qs(u.query)
        if p == "/":
            return self.send(200, (WEB / "index.html").read_bytes(), "text/html; charset=utf-8")
        if p in ("/app.js", "/view.js"):
            return self.send(200, (WEB / p[1:]).read_bytes(), "text/javascript; charset=utf-8")
        if p == "/app.css":
            return self.send(200, (WEB / "app.css").read_bytes(), "text/css; charset=utf-8")
        if p.startswith("/fonts/") and (WEB / "fonts" / p[7:]).is_file() and "/" not in p[7:]:
            ctype = "font/woff2" if p.endswith(".woff2") else "text/plain; charset=utf-8"
            return self.send(200, (WEB / "fonts" / p[7:]).read_bytes(), ctype)
        if p == "/favicon.ico":
            return self.send(200, (WEB.parent / "assets" / "coworkAgent.ico").read_bytes(), "image/x-icon")
        if p == "/api/status":
            return self.send(200, {"version": "0.1.0", "data_dir": "C:\\Users\\jhpark\\AppData\\Local\\coworkAgent", "db": "ok (스키마 12)"})
        if p == "/api/me":
            return self.send(200, {"name": "jhpark", "host": "PARK-PC", "fingerprint": "9F8E 7D6C 5B4A 3210", "addrs": ["192.168.0.10:47100"], "port": 47100})
        if p == "/api/peers":
            return self.send(200, PEERS)
        if p == "/api/agents":
            return self.send(200, AGENTS)
        if p == "/api/conversations":
            return self.send(200, [dict(c, last=last_of(c)) for c in CONVS])
        if p == "/api/groups":
            return self.send(200, sorted(GROUPS, key=lambda g: (g["pos"], g["id"])))
        if p == "/api/settings":
            return self.send(200, SETTINGS)
        if p == "/api/activity":
            return self.send(200, ACTIVITY)
        if p.startswith("/api/conversations/") and p.endswith("/messages"):
            return self.send(200, MESSAGES.get(int(p.split("/")[3]), []))
        if p == "/api/runs":
            return self.send(200, {"cap": 2, "running": [], "waiting": []})
        if p == "/api/messages":
            side, ask, req, peer, grp = (q.get(k, [""])[0] for k in ("side", "ask", "req", "peer", "group"))
            gconvs = gconv_ids(grp)
            ms = [dict(m, conversation=conv_ref(cid))
                  for cid, lst in MESSAGES.items() for m in lst
                  if (side != "mine" or m["mine"]) and (side != "theirs" or not m["mine"])
                  and (not ask or (m["author"] == "human" and m["request"] == (ask == "request")))
                  and (not req or m["req_state"] == req) and (not peer or conv_ref(cid)["peer_id"] == int(peer))
                  and (gconvs is None or cid in gconvs)]
            return self.send(200, sorted(ms, key=lambda m: -m["created_at"]))
        if p.startswith("/api/peers/") and p.endswith("/agents"):
            return self.send(200, {"names": PEER_AGENTS})
        if p == "/api/search":
            qq = q.get("q", [""])[0]
            ms = [dict(m, conversation={"id": cid, "peer_id": 2, "agent": "guide", "dir": "out", "title": "결제 모듈 배포 절차"})
                  for cid, lst in MESSAGES.items() for m in lst if qq in m["text"]]
            return self.send(200, {"messages": ms, "conversations": [c for c in CONVS if qq in c["title"]],
                                   "peers": [p for p in PEERS if qq in (p["alias"] or "") + p["name"] + p["host"]],
                                   "agents": [{"name": "guide", "owner": 2}] if qq in "guide" else []})
        if p == "/api/saved":
            peer = q.get("peer", [""])[0]
            grp = q.get("group", [""])[0]
            gconvs = gconv_ids(grp)
            ms = [dict(m, conversation=conv_ref(cid))
                  for cid, lst in MESSAGES.items() for m in lst if m["saved"] and (not peer or conv_ref(cid)["peer_id"] == int(peer))
                  and (gconvs is None or cid in gconvs)]
            return self.send(200, ms)
        if p == "/api/sent-log":
            return self.send(200, SENT)
        if p == "/api/events":
            return self.send(200, b": mock\n\n", "text/event-stream")
        return self.send(404, {"error": "mock 없음 " + p})

    def body(self):
        n = int(self.headers.get("Content-Length") or 0)
        raw = self.rfile.read(n)
        return json.loads(raw) if raw else {}

    def do_POST(self):
        b = self.body()
        p = urlparse(self.path).path
        SENT.append((p, b))
        if p == "/api/pairing-window":
            inv = ("coworkAgent 페어링 초대 — jhpark · PARK-PC\n"
                   "5분 안에 coworkAgent 웹UI \"초대 수락\" 에 이 메시지를 통째로 붙여 넣으세요.\n"
                   "coworkagent-invite:v1/192.168.0.10:47100/K7QM-3XRA/9F8E7D6C5B4A32100123456789ABCDEF")
            return self.send(200, {"invite": inv, "seconds": 300})
        if p == "/api/groups":
            name = (b.get("name") or "").strip()
            if not name or len(name) > 40:
                return self.send(400, {"error": "그룹 이름은 1 ~ 40자, 제어 문자 없이"})
            if any(g["name"] == name for g in GROUPS):
                return self.send(409, {"error": "같은 이름의 그룹이 있습니다"})
            g = {"id": NEXT_GID[0], "name": name, "icon": None, "pos": len(GROUPS), "convs": [], "peers": []}
            NEXT_GID[0] += 1
            GROUPS.append(g)
            return self.send(201, g)
        if p.startswith("/api/conversations/") and (p.endswith("/observers") or p.endswith("/leave")):
            return self.send(204)
        if p.startswith("/api/conversations/") and p.endswith("/context"):
            cid = int(p.split("/")[3])
            c = next(c for c in CONVS if c["id"] == cid)
            m = msg(950 + len(SENT), "context", True, "", state="queued" if c["dir"] == "in" else "sent", ctx_op=b.get("op"))
            MESSAGES.setdefault(cid, []).append(m)
            return self.send(200, m)
        if p.startswith("/api/conversations/") and p.endswith("/messages"):
            cid = int(p.split("/")[3])
            m = msg(900 + len(SENT), "human", True, b.get("text", ""), state="sent", request=b.get("request", False), agent=b.get("agent"))
            MESSAGES.setdefault(cid, []).append(m)
            return self.send(201, {"conversation": next(c for c in CONVS if c["id"] == cid), "message": m})
        return self.send(204)

    def do_DELETE(self):
        if "/observers/" in self.path:
            return self.send(204)
        return self._delete()

    def _delete(self):
        p = urlparse(self.path).path
        SENT.append((p, None))
        if p.startswith("/api/groups/"):
            gid = int(p.split("/")[3])
            GROUPS[:] = [g for g in GROUPS if g["id"] != gid]
            return self.send(204)
        return self.send(404, {"error": "mock 없음"})

    def do_PUT(self):
        b = self.body()
        p = urlparse(self.path).path
        SENT.append((p, b))
        if p.startswith("/api/groups/") and p.endswith("/peers"):
            g = next(g for g in GROUPS if g["id"] == int(p.split("/")[3]))
            g["peers"] = sorted((set(g["peers"]) | set(b.get("add", []))) - set(b.get("remove", [])))
            return self.send(200, g)
        if p.startswith("/api/groups/") and p.endswith("/conversations"):
            g = next(g for g in GROUPS if g["id"] == int(p.split("/")[3]))
            g["convs"] = sorted((set(g["convs"]) | set(b.get("add", []))) - set(b.get("remove", [])))
            return self.send(200, g)
        if p == "/api/order":
            items = {"peers": PEERS, "groups": GROUPS, "conversations": CONVS}[b["kind"]]
            for i, id in enumerate(b["ids"]):
                next(x for x in items if x["id"] == id)["pos"] = i
            if b["kind"] == "peers":
                PEERS.sort(key=lambda x: (x.get("pos", 0), x["id"]))
            return self.send(204)
        SETTINGS.update(b)
        return self.send(200, SETTINGS)

    def do_PATCH(self):
        b = self.body()
        p = urlparse(self.path).path
        SENT.append((p, b))
        parts = p.split("/")
        if p.startswith("/api/conversations/"):
            c = next(c for c in CONVS if c["id"] == int(parts[3]))
            if "starred" in b:
                c["starred"] = b["starred"]
            return self.send(200, {"id": c["id"], "title": c["title"], "starred": c["starred"]})
        if p.startswith("/api/groups/"):
            g = next(g for g in GROUPS if g["id"] == int(parts[3]))
            if "name" in b:
                n = b["name"].strip()
                if any(x["name"] == n and x["id"] != g["id"] for x in GROUPS):
                    return self.send(409, {"error": "같은 이름의 그룹이 있습니다"})
                g["name"] = n
            if "icon" in b:
                g["icon"] = b["icon"] or None
            return self.send(200, g)
        if p.startswith("/api/peers/"):
            pr = next(x for x in PEERS if x["id"] == int(parts[3]))
            if "alias" in b:
                pr["alias"] = b["alias"] or None
            if "icon" in b:
                pr["icon"] = b["icon"] or None
            return self.send(200, pr)
        if p.startswith("/api/messages/"):
            for ms in MESSAGES.values():
                for m in ms:
                    if m["id"] == int(parts[3]):
                        m.update(b)
            return self.send(200, dict(b, id=int(parts[3])))
        return self.send(204)


GROUPS = []


def gconv_ids(grp):
    if not grp:
        return None
    g = next((g for g in GROUPS if g["id"] == int(grp)), None)
    if not g:
        return set()
    return set(g["convs"]) | {c["id"] for c in CONVS if c["peer_id"] in g.get("peers", [])}
NEXT_GID = [1]


def conv_ref(cid):
    c = next(c for c in CONVS if c["id"] == cid)
    return {"id": c["id"], "peer_id": c["peer_id"], "agent": c["agent"], "dir": c["dir"], "title": c["title"]}


FAKE_LAST = {
    2: ("human", False, "로그인 실패 시 401 과 403 을 어떻게 나눌지 정리해 줘"),
    4: ("agent", False, "보고서 첫 장 — 첫 렌더 1.8초, 차단 스크립트 2개"),
    5: ("agent", True, "누락 3건 — 환불 · 부분 취소 · 쿠폰 중복"),
    6: ("human", False, "주문 응답에 배송 예정일 필드를 넣어 줘"),
    8: ("agent", False, "의존 캐시 만료로 링크 단계 실패 — 캐시 키 갱신 필요"),
}


def last_of(c):
    ms = MESSAGES.get(c["id"])
    if ms:
        m = ms[-1]
        return {"text": m["text"][:120], "author": m["author"], "mine": m["mine"], "agent": m.get("agent"), "owner": m.get("owner")}
    if c["id"] in FAKE_LAST:
        au, mine, text = FAKE_LAST[c["id"]]
        return {"text": text, "author": au, "mine": mine, "agent": None}
    return None


SENT = []
AGENT_CALLS = []
ThreadingHTTPServer(("127.0.0.1", PORT), H).serve_forever()

// 화면 로직 — DOM 없음. app.js 가 그리고, view.test.mjs 가 `node --test` 로 시험한다.

export const STATE_TEXT = {
  sent: "보냄",
  queued: "실행 대기",
  running: "실행 중",
  done: "완료",
  error: "오류",
  undelivered: "답 미전달",
};

/// 메시지 상태 → 문구(모르는 값은 그대로).
export function stateText(state) {
  return STATE_TEXT[state] ?? state ?? "";
}

/// 피어 승인 상태 → 문구.
export function peerState(p) {
  if (p.paired) return "페어링됨";
  if (!p.my_ok) return "내 승인 대기";
  return "상대 승인 대기";
}

/// 사이드바 · 머리글의 대화(대화방) 이름. 제목이 없으면(메시지 없음) 에이전트로. 보냄 · 받음 표시는 없다(#1j — 대화방 하나에 주고받은 글이 이어진다).
export function convLabel(conv, peerNames) {
  const who = peerNames.get(conv.peer_id) ?? "알 수 없는 상대";
  // 공유 대화(#1o) — 초대한 사람(`peer_id`)과 원 상대(`share.other`)의 대화를 보기만 한다.
  if (conv.dir === "shared") return { title: conv.title || `${conv.agent} 대화`, sub: `공유 · ${shortName(who)} ↔ ${conv.share?.other ?? "상대"}` };
  const out = conv.dir === "out";
  const title = conv.title || (out ? `${conv.agent} 에게` : `내 ${conv.agent} 가 받은 요청`);
  const sub = out ? `${who} 의 ${conv.agent}` : `${who} 의 요청 · 내 ${conv.agent}`;
  return { title, sub };
}

/// 대화방 줄 미리보기(#1j) — 마지막 글 `{말한 이}: {본문 한 줄}`(서버 `last`), 글이 없으면 빈 글자.
export function roomPreview(conv, peerName) {
  const l = conv.last;
  if (!l) return "";
  return `${authorLabel(conv, l, peerName)}: ${preview(l.text)}`;
}

/// 말풍선 쪽(#1j) — 이 PC 쪽 글(`mine` — 보낸 대화의 내 글 · 받은 대화의 내 에이전트 답)은 오른쪽, 그 밖은 왼쪽.
/// 공유 대화(#1o)는 공유한 사람 쪽 글(`owner`)이 오른쪽 — 대화가 오가는 순서를 알기 쉽게.
export function bubbleSide(m) {
  return m.mine === true || m.owner === true ? "mine" : "theirs";
}

/// 날짜 구분선 열쇠 — 지역 날짜(`YYYY-M-D`).
export function dayKey(ts) {
  const d = new Date(ts * 1000);
  return `${d.getFullYear()}-${d.getMonth() + 1}-${d.getDate()}`;
}

/// 날짜 구분선 글자 — 오늘 · 어제 · `M.D`(올해) · `YYYY.M.D`.
export function dayLabel(ts, now = Date.now() / 1000) {
  const d = new Date(ts * 1000);
  const today = new Date(now * 1000);
  if (dayKey(ts) === dayKey(now)) return "오늘";
  if (dayKey(ts) === dayKey(now - 86400)) return "어제";
  return d.getFullYear() === today.getFullYear() ? `${d.getMonth() + 1}.${d.getDate()}` : `${d.getFullYear()}.${d.getMonth() + 1}.${d.getDate()}`;
}

/// 말풍선 목록 사이 날짜 구분선 자리 — 글마다 `{day}`(새 날짜면 그 글자, 아니면 `null`).
export function dayBreaks(ms, now = Date.now() / 1000) {
  let prev = null;
  return ms.map((m) => {
    const k = dayKey(m.created_at);
    const day = k === prev ? null : dayLabel(m.created_at, now);
    prev = k;
    return day;
  });
}

/// 이 PC 에서 부르는 상대 이름 — 표시 이름(별칭)이 있으면 그것.
export function displayName(p) {
  return p.alias || p.name;
}

/// 사이드바 대화 목록 — 레일에서 고른 타일의 대화만, `q` 가 있으면 제목으로만 거른다(대소문자 무시).
export function convRows(convs, peerNames, railPeer, q = "") {
  const needle = q.trim().toLowerCase();
  return filterByPeer(convs, railPeer)
    .map((c) => ({ c, label: convLabel(c, peerNames) }))
    .filter(({ label }) => !needle || label.title.toLowerCase().includes(needle));
}

/// 타일을 눌러도 열린 대화를 그대로 둘까 — 대화 화면(옛 `#s/` 포함)이고 그 대화가 그 타일 것일 때만.
export function tileKeeps(view, conv, railPeer) {
  return ["conversation", "sent"].includes(view) && keepsConversation(conv, railPeer);
}

/// 레일에서 고른 상대의 대화만(`null` = 내 PC = 전부).
export function filterByPeer(convs, peerId) {
  return peerId == null ? convs : convs.filter((c) => c.peer_id === peerId);
}

/// 발화자 이름(D23) — 사람: 내 글 "나", 상대 글은 상대 이름. 에이전트: 대화 에이전트의 주인 쪽(보낸 대화 = 상대 PC, 받은 대화 = 내 PC) —
/// 멘션한 요청의 답이면 실행한 에이전트(`m.agent`). 자동 왕복(D24)이면 반대쪽 에이전트 글도 있다 — 보낸 대화의 내 글 = 내 자동 왕복 에이전트,
/// 받은 대화의 상대 글 = 상대 에이전트.
export function authorLabel(conv, m, peerName) {
  const peer = shortName(peerName);
  if (conv.dir === "shared") return sharedAuthor(conv, m, peer);
  if (m.author === "agent") {
    const agent = m.agent ?? conv.agent;
    if (conv.dir === "out") return m.mine === true ? `내 ${conv.relay_agent ?? "에이전트"}` : `${peer}/${agent}`;
    return m.mine === false ? `${peer}/${conv.relay_agent ?? "에이전트"}` : `내 ${agent}`;
  }
  if (m.via === "cli") return m.mine ? `내 ${CLI_LABEL}` : `${peer}/${CLI_LABEL}`;
  return m.mine ? "나" : peer;
}

/// 공유 대화(#1o)의 발화자 — `owner` = 초대한 사람(`peer`) 쪽 글, 아니면 원 상대(`share.other`). 에이전트는 그 쪽 사람의 것 — 대화 에이전트는
/// 초대한 사람이 연 대화(`owner_dir = "out"`)면 원 상대 것, 받은 대화면 초대한 사람 것.
export function sharedAuthor(conv, m, peer) {
  const other = conv.share?.other ?? "상대";
  const who = m.owner === true ? peer : other;
  if (m.author === "agent") {
    const ownsAgent = (conv.share?.owner_dir === "out") !== (m.owner === true);
    return `${who}/${m.agent ?? (ownsAgent ? conv.agent : "에이전트")}`;
  }
  if (m.via === "cli") return `${who}/${CLI_LABEL}`;
  return who;
}

/// "이름 · PC" 의 이름 — 마지막 구분 앞까지(표시 이름에 " · " 가 있어도 자르지 않는다). 모르는 상대 = "상대".
export function shortName(full) {
  const s = full ?? "상대";
  const cut = s.lastIndexOf(" · ");
  return cut < 0 ? s : s.slice(0, cut);
}

/// CLI 로 쓴 글(`via = "cli"`)의 발화자 — 사용자의 AI 에이전트.
const CLI_LABEL = "AI 에이전트(CLI)";

/// 대화 머리글의 자동 왕복 표시(D24)와 누를 수 있는 동작 — 보낸 쪽: 멈춤 · 재개, 받은 쪽: 멈춤만(재개는 연 쪽). 아니면 `null`.
export function relayText(conv, messages) {
  if (conv.dir === "shared") return null;
  if (conv.dir === "out") {
    if (!conv.relay_agent) return null;
    const n = `자동 왕복 ${conv.relay_count ?? 0} / ${conv.relay_cap}`;
    return conv.relay_on ? { text: `${n} · 진행 중`, action: "stop" } : { text: `${n} · 멈춤`, action: "resume" };
  }
  if (conv.relay_blocked) return { text: "자동 왕복 거절 중", action: null };
  const relayed = (messages ?? []).some((m) => m.author === "agent" && m.mine === false);
  return relayed ? { text: "자동 왕복 받는 중", action: "stop" } : null;
}

/// 열린 대화가 레일에서 고른 상대와 맞나 — 아니면 그 대화를 닫는다(다른 PC 를 고른 화면에서 이전 상대에게 보내지 않게).
export function keepsConversation(conv, railPeer) {
  return conv != null && (railPeer == null || conv.peer_id === railPeer);
}

/// 실행 시간 표기.
export function formatDuration(ms) {
  if (ms == null) return "";
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}초`;
  const m = Math.floor(ms / 60000);
  const s = Math.round((ms % 60000) / 1000);
  return `${m}분 ${s}초`;
}

/// 유닉스 초 → 지역 시각 "HH:MM".
export function formatTime(unix) {
  if (unix == null) return "";
  const d = new Date(unix * 1000);
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}

/// 유닉스 초 → 지역 시각 "HH:MM:SS"(실행 상세 진행).
export function formatClock(unix) {
  if (unix == null) return "";
  const d = new Date(unix * 1000);
  return [d.getHours(), d.getMinutes(), d.getSeconds()].map((n) => String(n).padStart(2, "0")).join(":");
}

function runMeta(reply) {
  const parts = [];
  if (reply.turns != null) parts.push(`${reply.turns}턴`);
  if (reply.duration_ms != null) parts.push(formatDuration(reply.duration_ms));
  return parts.length ? parts.join(" · ") : "실행 정보 없음";
}

/// 답 ↔ 요청 연결. `reply_to` 가 있으면 그것으로, 없으면(스키마 5 이전 이력) 답을 기다리는 요청이 **하나뿐**일 때만 그 요청에 —
/// 여럿이면 어느 요청의 답인지 모르므로 연결하지 않는다(`unlinked`).
export function linkReplies(messages) {
  const linked = new Map();
  const unlinked = [];
  const pending = [];
  for (const m of messages) {
    if (m.author === "human") {
      if (m.exec !== false) pending.push(m); // 실행 없는 사람 글은 답을 기다리지 않는다
      continue;
    }
    if (m.reply_to != null) {
      if (!linked.has(m.reply_to)) linked.set(m.reply_to, m);
      const at = pending.findIndex((h) => h.id === m.reply_to);
      if (at >= 0) pending.splice(at, 1);
    } else if (pending.length === 1) {
      linked.set(pending.pop().id, m);
    } else {
      unlinked.push(m);
    }
  }
  return { linked, unlinked };
}

/// 요청 흐름 — 사람 메시지(요청)마다 요청 → 실행 → 응답 단계(실행 상세 '진행' 이 쓴다). 연결 못 한 답은 `unlinkedReplies` 로 따로.
export function timeline(messages) {
  const { linked } = linkReplies(messages);
  return messages
    .filter((m) => m.author === "human" && m.exec !== false)
    .map((h) => {
      const reply = linked.get(h.id) ?? null;
      const steps = [
        { kind: "request", title: h.via === "cli" ? `요청 · ${CLI_LABEL}` : "요청", at: h.created_at, text: h.text, state: h.state },
        reply
          ? { kind: "run", title: "실행", meta: runMeta(reply), state: reply.state }
          : { kind: "run", title: "실행", meta: stateText(h.state), state: h.state },
        reply
          ? {
              kind: "reply",
              title: reply.state === "done" ? "응답" : `응답 · ${stateText(reply.state)}`,
              at: reply.created_at,
              text: reply.text,
              state: reply.state,
            }
          : { kind: "reply", title: "응답 대기", at: null, text: "", state: h.state },
      ];
      return { request: h, reply, steps };
    });
}

/// 어느 요청의 답인지 모르는 이전 이력 답(시간순).
export function unlinkedReplies(messages) {
  return linkReplies(messages).unlinked;
}

/// 화면 위치(해시) — `#c/{id}` 대화(`/m/{글}` = 그 글 위치) · `#s/{id}` 그 대화 상대에게 보낸 글 · `#p/{id}` 상대와 주고받은 글 ·
/// `#activity` 내 활동 · `#requests` 요청 목록 · `#starred` 즐겨찾기(#1j) · `#saved` 북마크 항목 · `#agents` 에이전트 · `#settings`(`/{절}` = 그 절, 모르는 절은 처음) ·
/// `#new` 세션 생성(#1x). 그 밖은 안내 화면.
export function route(hash) {
  const h = (hash ?? "").replace(/^#/, "");
  let m = /^c\/(\d+)(?:\/m\/(\d+))?$/.exec(h);
  if (m) return m[2] ? { view: "conversation", id: Number(m[1]), msg: Number(m[2]) } : { view: "conversation", id: Number(m[1]) };
  m = /^s\/(\d+)$/.exec(h);
  if (m) return { view: "sent", id: Number(m[1]) };
  m = /^p(?:\/(\d+))?$/.exec(h);
  if (m) return { view: "peer", id: m[1] ? Number(m[1]) : null };
  if (h === "agents/convs") return { view: "agentConvs" };
  if (["activity", "requests", "starred", "saved", "agents", "new"].includes(h)) return { view: h };
  m = /^settings(?:\/(\w+))?$/.exec(h);
  if (m) {
    const known = settingsTree().some((n) => n.id === m[1] || (n.children ?? []).some((c) => c.id === m[1]));
    return { view: "settings", focus: known ? m[1] : null };
  }
  return { view: "home" };
}

/// 배경 새로 고침을 미룰 초점 — 입력칸에서 쓰는 중일 때만(`tag` = 초점 요소 `tagName`). 상대별 절 머리(`summary`) · 단추에 초점이 남아도 다시
/// 그린다(#1g 적대적 평가 라운드 1 — 절을 접은 뒤 목록이 멈췄다).
export function holdsRender(tag) {
  return ["INPUT", "TEXTAREA", "SELECT"].includes(tag);
}

/// 마지막 값만 보내기(#1g 적대적 평가 라운드 1) — 한 번에 하나만 보내고, 보내는 동안 들어온 값은 마지막 것만 이어서 보낸다(동시에 보낸 옛 값이 늦게
/// 닿아 새 값을 덮지 않게). `send` 의 실패는 `send` 가 알린다 — 여기서는 다음 값을 막지 않는다.
export class SendLatest {
  constructor(send) {
    this.send = send;
    this.busy = false;
    this.waiting = false;
    this.next = undefined;
  }
  push(value) {
    if (this.busy) {
      this.waiting = true;
      this.next = value;
      return;
    }
    this.run(value);
  }
  async run(value) {
    this.busy = true;
    try {
      await this.send(value);
    } catch {
      // `send` 가 알린다
    }
    this.busy = false;
    if (this.waiting) {
      this.waiting = false;
      this.run(this.next);
    }
  }
}

/// 늦게 온 응답 판별 — 요청마다 `next()` 로 번호를 받고, 응답 때 `is(번호)` 가 거짓이면(그 뒤 다른 요청이 있었다) 버린다.
export class Latest {
  constructor() {
    this.n = 0;
  }
  next() {
    this.n += 1;
    return this.n;
  }
  is(ticket) {
    return ticket === this.n;
  }
}

// ---- 변경 신호(SSE `event: change` — D39 ②) ----

/// 대화 변경 한 건이 닿는 로더 전부(자원 단위 — 화면 단위가 아니다). 상대 요청은 접속 표시(마지막 통신 시각)도 바꿔 연결도 넣는다.
export const CONV_LOADERS = ["peers", "groups", "convs", "current", "activity", "requests", "saved", "peer", "runs", "counts"];

/// 바뀐 자원 → 다시 읽을 로더 집합, `null` = 전체(빈 목록 · 모르는 종류 · 연결 변경). 열린 대화 본문은 그 id · id 모름일 때만.
export function loadersFor(res, currentId) {
  if (!Array.isArray(res) || !res.length || res.some((r) => r?.kind !== "conv")) return null;
  const out = new Set(CONV_LOADERS);
  if (currentId == null || !res.some((r) => r.id == null || r.id === currentId)) out.delete("current");
  return out;
}

/// 짧은 간격에 온 신호 합치기 — 처음이면 그대로, 하나라도 전체(빈 · 배열 아님)면 전체(`[]`).
export function mergeChanges(q, res) {
  if (q === undefined) return res;
  if (!Array.isArray(q) || !q.length || !Array.isArray(res) || !res.length) return [];
  return [...q, ...res];
}

/// 도는 중 쌓인 다시 읽기 요청 합치기 — `undefined` = 아직 없음, `null` = 전부(하나라도 전부면 전부), `Set` = 합집합.
export function unionLoaders(pending, only) {
  if (pending === undefined) return only;
  if (pending === null || only === null) return null;
  return new Set([...pending, ...only]);
}

/// 다시 읽기를 한 번에 하나만 돌린다 — 도는 중 온 요청은 `unionLoaders` 로 합쳐 끝난 뒤 한 묶음으로 한 번 더. `call` 이 돌려주는 약속은
/// 자기 요청이 든 묶음의 결과다 — 앞 묶음이 실패해도 쌓인 묶음은 돈다.
export class OneAtATime {
  constructor(run) {
    this.run = run;
    this.busy = null;
    this.queued = null; // { only, promise, resolve, reject }
  }
  call(only) {
    if (!this.busy) return this.start(only);
    if (this.queued) {
      this.queued.only = unionLoaders(this.queued.only, only);
    } else {
      const q = { only };
      q.promise = new Promise((resolve, reject) => Object.assign(q, { resolve, reject }));
      this.queued = q;
    }
    return this.queued.promise;
  }
  start(only) {
    const p = (async () => {
      try {
        await this.run(only);
      } finally {
        this.busy = null;
        const q = this.queued;
        this.queued = null;
        if (q) this.start(q.only).then(q.resolve, q.reject);
      }
    })();
    this.busy = p;
    return p;
  }
}

// ---- 설정 트리 ----

/// 설정 화면 왼쪽 트리 — 항목 `id` = 그 절의 요소 id(누르면 그 절로), `icon` = `.ico-{이름}`(#1l). 내 정보는 프로필 메뉴 "사용자 정보" 팝업(#1l).
/// `press` = 누르면 그 절로 간 뒤 함께 누를 단추 id(#1r — "초대 생성" 은 초대를 만든다, 해시로 다시 올 때는 누르지 않는다).
export function settingsTree() {
  return [
    {
      id: "conn",
      label: "연결",
      icon: "link",
      children: [
        { id: "invite_make", label: "초대 생성", icon: "plus", press: "window_btn" },
        { id: "invite_accept", label: "초대 수락", icon: "enter" },
        { id: "peers", label: "그룹설정", icon: "users" },
      ],
    },
    { id: "agents", label: "내 에이전트", icon: "bot" },
    { id: "run", label: "실행", icon: "play" },
    { id: "screen", label: "화면", icon: "monitor" },
    { id: "startup", label: "시작", icon: "power" },
    { id: "notify", label: "알림", icon: "bell" },
  ];
}

// ---- 검색 · 그룹설정의 그룹(#1j) ----

/// 검색 — `fields` 값에 부분 일치(대소문자 · 앞뒤 공백 무시), 빈 검색이면 전부(순서 그대로).
export function searchItems(items, q, fields) {
  const needle = (q ?? "").trim().toLowerCase();
  return needle ? items.filter((it) => fields.some((f) => String(it[f] ?? "").toLowerCase().includes(needle))) : items;
}

/// 설정 그룹설정의 끝 절 이름(#1w — 사용자 목록: 관리 줄 + 그 사용자와의 대화 트리).
export const USER_LIST = "사용자 목록";

/// 설정 그룹설정 절(#1w — 그룹 = 대화만, D37 개정) — 레일 그룹 순서대로 그 그룹 대화 줄(`{id, peer, title, sub}` — `convs` 순서, 비어도 둔다 — 놓을 자리),
/// 끝에 "사용자 목록"(줄은 부르는 쪽이 상대마다 그린다).
export function settingsGroupSections(groups, convs, peerNames) {
  return [
    ...groups.map((g) => ({
      group: g.id,
      title: g.name,
      rows: inGroup(convs, g).map((c) => ({ id: c.id, peer: c.peer_id, ...convLabel(c, peerNames) })),
    })),
    { group: null, title: USER_LIST },
  ];
}

/// 그룹설정 왼쪽 목록(#1z — 시안 A) — 검색(`q`)에 맞는 그룹 · 사용자와 각 대화 수. 사용자 = 이름 · 별칭 · PC · 지문 또는 그 사용자 대화 제목이 맞으면,
/// 그룹 = 이름이 맞거나 그 그룹 대화(제목 · 사용자)가 맞으면. 빈 검색이면 전부(순서 그대로). 그룹 `convs` = 오른쪽 상세에 보일 대화 id —
/// 이름이 맞으면(또는 빈 검색) 그 그룹 대화 전부, 아니면 맞는 대화만(적대적 리뷰 R1 — 이름으로 찾은 그룹의 상세가 비지 않게).
export function groupSettingsList(groups, peers, convs, q, peerNames) {
  const needle = (q ?? "").trim().toLowerCase();
  const hit = (s) => String(s ?? "").toLowerCase().includes(needle);
  const userHit = (p) => !!p && ["alias", "name", "host", "fingerprint"].some((f) => hit(p[f]));
  const convHit = (c) => hit(convLabel(c, peerNames).title) || userHit(peers.find((p) => p.id === c.peer_id));
  const mine = (p) => convs.filter((c) => c.peer_id === p.id);
  return {
    groups: groups
      .filter((g) => !needle || hit(g.name) || inGroup(convs, g).some(convHit))
      .map((g) => {
        const all = inGroup(convs, g);
        const shown = !needle || hit(g.name) ? all : all.filter(convHit);
        return { id: g.id, name: g.name, icon: g.icon, count: all.length, convs: shown.map((c) => c.id) };
      }),
    users: peers.filter((p) => !needle || userHit(p) || mine(p).some(convHit)).map((p) => ({ id: p.id, count: mine(p).length })),
  };
}

/// 끌기 시작 거리(#1z — 레일 타일) — 누른 채 이만큼(px) 움직여야 끌기, 그 미만은 그냥 누름(타일 고르기).
export const DRAG_SLOP = 5;

export function pastSlop(dx, dy) {
  return dx * dx + dy * dy >= DRAG_SLOP * DRAG_SLOP;
}

/// 레일 타일을 놓을 자리(#1z-2) — 같은 묶음 타일 가운데(`centers`, 보이는 순서 · 세로 레일 = Y, 가로 레일 = X)에서 포인터(`pos`)와 가장 가까운 타일과
/// 그 앞 / 뒤(가운데를 넘으면 뒤). 틈 · 타일 아래 이름 · 양 끝 밖에 놓아도 자리가 정해진다 — 놓는 점 아래 요소만 보면 거기서는 아무 일이 없었다. 타일이 없으면 `null`.
export function railSlot(centers, pos) {
  if (!centers.length) return null;
  let i = 0;
  for (let k = 1; k < centers.length; k++) if (Math.abs(centers[k] - pos) < Math.abs(centers[i] - pos)) i = k;
  return { index: i, after: pos > centers[i] };
}

/// 대화 줄을 그룹 절에 놓기(#1w) — 복사: 그 그룹에 넣기만(원래 자리 — 사용자 목록 · 다른 그룹 — 그대로). 이미 있거나 모르는 그룹이면 `null`.
export function convCopy(convId, to, groups) {
  const g = groups.find((x) => x.id === to);
  return g && !(g.convs ?? []).includes(convId) ? { add: [convId] } : null;
}

/// 받침에 맞춘 조사 — 한글로 끝나면 받침 있음 `withB` · 없음 `withoutB`, 그 밖(영문 · 숫자 등)은 `withB(withoutB)`.
export function josa(word, withB, withoutB) {
  const last = (word ?? "").at(-1) ?? "";
  const code = last.charCodeAt(0) - 0xac00;
  if (!(code >= 0 && code <= 11171)) return `${withB}(${withoutB})`; // 빈 글자(NaN)도
  return code % 28 ? withB : withoutB;
}

// ---- 주고받은 글 · 보낸 글 ----

/// `GET /api/messages` 항목 → 목록 행. `kind` = `""`(전체) · `human`(사람 글) · `agent`(에이전트 답). 누르면 그 대화로.
export function inboxRows(messages, peerNames, { kind = "" } = {}) {
  return messages
    .filter((m) => m.conversation && (!kind || m.author === kind))
    .map((m) => {
      const conv = m.conversation;
      // 방향 라벨(보냄 · 받음 등)은 없다(#1j) — 이름 · 요청 칩 · 대화방 제목.
      return {
        id: m.id,
        who: authorLabel(conv, m, peerNames.get(conv.peer_id)),
        // 어느 에이전트와의 대화인지(#1k — 주고받은 글 칩): 보낸 대화 = 상대 에이전트, 받은 대화 = 내 에이전트.
        agent: conv.dir === "in" ? `내 ${conv.agent}` : conv.agent,
        kind: askTag(m),
        saved: m.saved === true,
        room: conv.title || "",
        preview: preview(m.text),
        at: m.created_at,
        href: `#c/${conv.id}`,
        peer: conv.peer_id,
        author: m.author,
        mine: m.mine === true,
      };
    });
}

/// 주고받은 메시지 한 번에 받는 건수(#1q).
export const PEER_PAGE = 20;

/// 주고받은 메시지 상대 절 머리(#1q, `GET /api/messages/peers`) — 레일 순서(`paired`), 그 밖(페어링 풀린 상대)은 id 순으로 뒤. 이름 · 건수 · 마지막 메시지
/// 한 줄 · 시각.
export function peerSummaryRows(items, paired, peerNames) {
  const rank = (id) => (paired.includes(id) ? paired.indexOf(id) : paired.length + id);
  return [...items]
    .sort((a, b) => rank(a.peer) - rank(b.peer))
    .map((x) => ({ peer: x.peer, name: shortName(peerNames.get(x.peer)), count: `${x.count}건`, total: x.count, last: preview(x.last?.text), at: x.last?.created_at ?? 0 }));
}

/// 주고받은 메시지 다시 그리기 서명(#1q 적대적 평가 R1) — 행에 보이는 값 전부(본문 · 요청 상태 · 메모 · 편집 · 상태 · 북마크 · 대화 제목 · 에이전트).
export function peerMsgsSig(heads, pages) {
  const msg = (m) => [
    m.id,
    m.text,
    m.saved,
    m.req_state,
    m.req_note,
    m.edited_at,
    m.state,
    m.created_at,
    // 발화자 표시(`authorLabel`)가 기대는 값 — 공유 사본은 같은 글의 이것만 바뀔 수 있다(R2).
    m.author,
    m.mine,
    m.via,
    m.agent,
    m.owner,
    m.conversation?.title,
    m.conversation?.agent,
    m.conversation?.dir,
  ];
  return JSON.stringify([heads, heads.map((h) => (pages.get(h.peer) ?? []).map(msg))]);
}

/// 펼친 절 새로 고침(#1q 적대적 평가 R2) — 새로 받은 첫 페이지(`fresh`, 최신순) + 이미 받은 글 중 그 페이지 마지막보다 앞(`(created_at, id)`)의 꼬리.
/// 첫 페이지가 다 차지 않으면(그 상대 글 전부) 꼬리는 없다 — "더 보기" 로 받은 쪽을 5초 새로 고침이 지우지 않는다.
export function mergePage(fresh, have, page = PEER_PAGE, since = -Infinity) {
  // 받아 둔 꼬리도 보기 기간(#1y — `since` 이후에 쓴 글)만 — 시간이 지나 기간 밖이 된 글이 남지 않게.
  have = have.filter((m) => m.created_at >= since);
  if (fresh.length < page) return fresh;
  const end = fresh.at(-1);
  const older = (m) => m.created_at < end.created_at || (m.created_at === end.created_at && m.id < end.id);
  const ids = new Set(fresh.map((m) => m.id));
  return [...fresh, ...have.filter((m) => older(m) && !ids.has(m.id))];
}

/// 펼친 상대 절 끝 줄(#1q) — "{보인 수} / {전체}", 남았을 때만 더 보기.
export function pageMore(shown, total) {
  return { text: `${shown} / ${total}`, more: shown < total };
}

/// 목록에 보일 본문 앞부분 — 줄바꿈은 공백으로, 120자까지.
export function preview(text) {
  const one = (text ?? "").replace(/\s+/g, " ").trim();
  return one.length > 120 ? one.slice(0, 120) + "…" : one;
}

// ---- 작업 화면 — 상단 바 · 사이드바 · 내 활동 · 대화 · 실행 상세 · 통합 검색 ----

/// 사이드바 내비(#1g) — '나'(`tile` = `null`) = 전부를 상대별로 묶어 보는 6개, 상대 타일 = 그 상대만 4개(개요는 아래 목록, 내 활동은 '나' 에만).
/// 그룹 타일(`tile` = `"group"`, #1i) = 개요(그룹 요약) · 요청 · 북마크 항목 · 주고받은 글 4개 — 내 활동('나' 에만) · 에이전트(대화 단위 아님) 없음.
/// 세 절(#1h) — 상단 개요 / 중단 내 활동 · 요청 · 주고받은 글 / 하단 북마크 항목 · 에이전트(사용자가 모은 · 등록한 것), 절 사이 `{ sep: true }`
/// (빈 절은 건너뛴다). 내 활동은 안 본 수(`count`), 요청은 받은 요청 중 대기 수(`waiting`).
/// '에이전트'(#1v) = 링크 없는 머리(`head`) + 들여쓴 하위(`sub`) — 등록된 에이전트 · 에이전트 응답(#1z-2 — '나' 에만, 상대가 연 대화, 수 = 안 읽음 `agentUnread`).
export function sideNav(tile, unseen = 0, waiting = 0, starred = 0, agentUnread = 0) {
  const me = tile == null;
  const group = tile === "group";
  // 상대 타일의 첫 화면이 그 상대 개요라(#1j) 상대에는 '개요' 메뉴가 없다. '주고받은 메시지' 는 '나' 에만 — 상대 · 그룹은 대화방 목록이 같은 글을 보인다.
  // `icon` = `app.css` `.ico-{이름}`(#1l — 화면 머리 제목과 같은 아이콘).
  const top = me || group ? [{ label: "개요", href: "#", icon: "home" }] : [];
  const mid = [
    ...(me ? [{ label: "내 활동", href: "#activity", count: unseen, icon: "bell" }] : []),
    { label: "요청", href: "#requests", count: waiting, icon: "inbox" },
    ...(me ? [{ label: "주고받은 메시지", href: "#p", icon: "swap" }] : []),
  ];
  const end = [
    { label: "즐겨찾기", href: "#starred", count: starred, icon: "star" },
    { label: "북마크", href: "#saved", icon: "bookmark" },
    ...(group
      ? []
      : [
          { label: "에이전트", icon: "bot", head: true },
          { label: "등록된 에이전트", href: "#agents", icon: "bot", sub: true },
          // 에이전트 응답(#1z-2 — 옛 "에이전트 대화") = 내 에이전트가 어느 사용자에게 답했는지 — '나' 에만.
          ...(me ? [{ label: "에이전트 응답", href: "#agents/convs", icon: "chat", sub: true, count: agentUnread }] : []),
        ]),
  ];
  return [top, mid, end].filter((s) => s.length).flatMap((s, i) => (i ? [{ sep: true }, ...s] : s));
}

/// 세션 생성 이름(#1x — 사용자 2026-10-20, 옛 이름 새 대화) — 상단 + 메뉴 · 화면 제목 · 사이드바 · 에이전트 줄 단추가 쓴다.
export const SESSION_NEW = "세션 생성";

/// 상단 바 "+ 새로 만들기" 메뉴 — 각 화면과 그 화면에서 초점을 줄 칸 · 아이콘 · 함께 누를 단추(`press`, #1r). 자동 왕복은 세션 생성 화면 안에서(#1l — 메뉴에서 뺌).
export function newMenu() {
  return [
    { label: SESSION_NEW, href: "#new", focus: "chat_peer", icon: "chat" },
    { label: "초대 생성", href: "#settings/invite_make", focus: "window_btn", icon: "plus", press: "window_btn" },
    { label: "초대 수락", href: "#settings/invite_accept", focus: "invite_text", icon: "enter" },
    { label: "에이전트 등록", href: "#settings/agents", focus: "agent_name", icon: "bot" },
  ];
}

/// 상단 바 프로필 메뉴(#1j) — 상태 고르기(받기 가능 · 집중 — 하나만 켜짐) · 앱과 연결 끊김(자동 표시, 고를 수 없음) | 사용자 정보(팝업 — #1l) · 표시 이름 · 아이콘('나' 타일 편집) ·
/// 알림 일시 중지(켜짐 표시) · 환경설정. `state` = 그 상태의 색(`presenceOf` 열쇠), 상태가 아닌 항목은 `icon`(#1r).
export function profileMenu(paused, focus, offline = false) {
  return [
    { key: "avail", label: "받기 가능", hint: "새 요청을 받습니다", state: "online", radio: true, checked: focus !== true },
    { key: "focus", label: "집중", hint: "새 요청을 거절합니다", state: "focus", radio: true, checked: focus === true },
    { key: "offline", label: "앱과 연결 끊김", hint: "자동 표시", state: "offline", info: true, current: offline === true },
    { sep: true },
    { key: "me", label: "사용자 정보", icon: "user" },
    { key: "tile", label: "프로필 편집", icon: "pencil" },
    { key: "pause", label: "알림 일시 중지", icon: "bell", checked: paused === true },
    { key: "settings", label: "설정", icon: "gear", href: "#settings" },
  ];
}

/// 내 상태(#1j — 프로필 아이콘 테두리 · 점) — 앱과 연결 끊김(붉음)이 먼저, 그다음 집중(주황), 그 밖은 받기 가능(초록).
export function presenceOf(focus, offline) {
  if (offline === true) return { key: "offline", text: "앱과 연결 끊김" };
  return focus === true ? { key: "focus", text: "집중" } : { key: "online", text: "받기 가능" };
}

/// 즐겨찾기 화면(#1j) — ★ 대화방만, 대화방 절 그대로(`paired` 를 주면 상대별 절). 빈 절은 뺀다 — 하나도 없으면 빈 배열.
export function starredSections(convs, peerNames, paired = null) {
  return convSections(
    convRows(convs.filter((c) => c.starred === true), peerNames, null),
    paired,
    peerNames,
  ).filter((s) => s.rows.length);
}

/// 상대 접속 상태(`GET /api/peers` 의 `seen_at` · `focus`, #1c — 통신할 때만 갱신) — 레일 점 · 사이드바 머리 · 대화 머리글. "방금 통신"(75초 안) ·
/// "n분 · n시간 · n일 전 통신" · "통신 기록 없음". 방금 통신했을 때만 점이 켜지고 집중이 앞에 붙는다.
export function presence(p, now = Math.floor(Date.now() / 1000)) {
  const at = p?.seen_at;
  if (at == null) return { key: "offline", text: "통신 기록 없음" };
  const ago = Math.max(0, now - at);
  const when =
    ago < 75
      ? "방금 통신"
      : ago < 3600
        ? `${Math.floor(ago / 60)}분 전 통신`
        : ago < 86400
          ? `${Math.floor(ago / 3600)}시간 전 통신`
          : `${Math.floor(ago / 86400)}일 전 통신`;
  if (ago >= 75) return { key: "offline", text: when };
  return p.focus ? { key: "focus", text: `집중 중 · ${when}` } : { key: "online", text: when };
}

/// 상대가 거절한 내 글의 사유(`refused`) — 실패 상태일 때만. 집중이면 "상대가 집중 중", 그 밖은 코드 그대로.
export function refusedText(m) {
  if (reqResend(m))
    return m.refused === "unreachable"
      ? "요청자에게 알리지 못함 · 오래 닿지 않음"
      : `요청자에게 알리지 못함 · 거절(${m.refused})`;
  if (!m.refused || m.state !== "error") return "";
  if (m.refused === "unreachable") return "보내지 못함 · 오래 닿지 않음";
  if (m.refused === "left") return "상대가 대화를 나갔습니다";
  return m.refused === "focus" ? "상대가 집중 중" : `상대가 거절(${m.refused})`;
}

/// 상대가 알려 준 내 실행 요청의 차례(`progress`) — 답을 기다리는(`sent`) 글만. 대기면 앞 사람 수, 실행이 시작되면 "처리 중".
export function progressText(m) {
  if (m.state !== "sent" || !m.progress) return "";
  if (m.progress.state === "queued") return `차례 기다리는 중 · 앞에 ${m.progress.ahead}건`;
  if (m.progress.state !== "running") return "";
  const step = stepText(m.progress.detail);
  return step ? `처리 중 · ${step}` : "처리 중";
}

/// 실행 중 단계 요약(#1c) — `{turns, tools: {이름: 횟수}, secs}` → "2턴 · Read 3 · Grep 1 · 41초"(없는 값은 뺀다).
export function stepText(d) {
  if (!d) return "";
  const tools = Object.entries(d.tools ?? {}).map(([name, n]) => `${name} ${n}`);
  return [d.turns ? `${d.turns}턴` : "", ...tools, d.secs ? `${d.secs}초` : ""].filter(Boolean).join(" · ");
}

/// 보관 중인 내 글(#1c — 상대에 아직 닿지 않음). 포기한 글은 거절 사유(`refusedText`)가 말한다.
export function outboxText(m) {
  const o = m.outbox;
  if (!o || o.state === "failed") return "";
  return o.tries > 0 ? `보내는 중 · 다시 보냄 ${o.tries}회` : "보내는 중";
}

/// 대화 글 서명 — 바뀐 것이 없으면 다시 그리지 않는다(HTML 본문 · 스크롤 상태가 5초마다 풀리지 않게). 상대 표시 이름 · 요청 상태 · 메모 · 거절 사유 ·
/// 상대가 알려 준 진행(대기 · 앞 사람 수 · 처리 중 — 글 상태는 `sent` 그대로)이 바뀌면 다시. 저장 여부는 넣지 않는다 — 저장 단추가 그 글자만 바꾼다.
export function messagesSigOf(peerName, ms) {
  return [
    peerName,
    ...ms.map(
      (m) =>
        `${m.id}:${m.state}:${m.text.length}:${m.req_state ?? ""}:${m.req_note ?? ""}:${m.refused ?? ""}:${progressText(m)}:${outboxText(m)}:${m.edited_at ?? ""}:${m.editable === true}`,
    ),
  ].join(",");
}

/// 이 PC 실행 문(`GET /api/runs`) → 실행 상세 '이 PC 실행' 머리 문구 · 줄(실행 중 먼저, 닿은 순서). 줄 = `{label, text, href}` — 누르면 그 대화.
export function runsSummary(r) {
  const row = (x, label) => ({
    label: `${label} · ${x.peer ?? "알 수 없는 상대"}/${x.agent ?? "에이전트"}`,
    text: x.text ?? "",
    href: x.conversation_id != null ? `#c/${x.conversation_id}` : "",
  });
  return {
    title: `실행 중 ${r.running.length} · 대기 ${r.waiting.length}`,
    rows: [...r.running.map((x) => row(x, "실행 중")), ...r.waiting.map((x, i) => row(x, `대기 ${i + 1}`))],
  };
}

/// 내 정보 포트 줄(`GET /api/me`) — 웹UI 포트 · 피어 포트(고정 | 임의 — 잡은 프로세스) · 열린 피어 연결 수.
export function portText(me) {
  const holder = me.peer_port_holder;
  const peer = me.peer_port_fixed
    ? `${me.port}(고정)`
    : holder
      ? `${me.port}(임의 — 47100 사용 중: ${holder.name}, PID ${holder.pid})`
      : `${me.port}(임의)`;
  return { ui: me.ui_port != null ? String(me.ui_port) : "알 수 없음", peer, conns: String(me.open_connections ?? 0) };
}

/// 입력 중인 멘션 — 커서(`caret`) 바로 앞이 `@…`(앞이 글 머리 · 공백, 그 뒤에 공백 없음)면 `{start: @ 자리, query: @ 뒤 글자}`, 아니면 `null`.
export function mentionQuery(text, caret) {
  const before = text.slice(0, caret);
  const at = before.lastIndexOf("@");
  if (at < 0 || (at > 0 && !/\s/.test(before[at - 1]))) return null;
  const query = before.slice(at + 1);
  return /\s/.test(query) ? null : { start: at, query };
}

/// 글의 첫 멘션 꼴 낱말 — 글 머리 · 공백 뒤 `@이름`(낱말 안의 `@` 는 아님), 없으면 `null`. 후보와 맞지 않는 멘션을 알릴 때 쓴다.
export function mentionToken(text) {
  const m = /(?:^|\s)(@[^\s@]+)/u.exec(text);
  return m ? m[1] : null;
}

/// 멘션처럼 보이는 글인가 — `mentionToken` 이 있다. 후보 목록 없이 보내지 않을 때 쓴다.
export function hasMention(text) {
  return mentionToken(text) != null;
}

/// 글의 멘션 대상(해석 2) — 첫 `@이름`(앞이 글 머리 · 공백, 뒤가 글 끝 · 공백 · 문장 부호)이 `names` 의 한 이름과 글자 그대로 같은 것. 긴 이름
/// 먼저(한 이름이 다른 이름의 앞부분이어도). 없으면 `null`.
export function mentionTarget(text, names) {
  const longFirst = [...names].sort((a, b) => b.length - a.length);
  for (let at = text.indexOf("@"); at >= 0; at = text.indexOf("@", at + 1)) {
    if (at > 0 && !/\s/.test(text[at - 1])) continue;
    const name = longFirst.find((n) => {
      const next = text[at + 1 + n.length];
      return text.startsWith(n, at + 1) && (next === undefined || /[\s\p{P}]/u.test(next));
    });
    if (name) return name;
  }
  return null;
}

/// 요청 상태 문구(#1d — an_core `ReqState`).
export const REQ_STATES = { wait: "대기", accepted: "수용", rejected: "반려", done: "완료" };

/// 사람 글의 종류 태그 — 요청이면 "요청 · {상태}"(받은 쪽이 바꾼다), 질의 · 답은 없음.
export function askTag(m) {
  if (m.author !== "human" || !m.request) return "";
  return `요청 · ${REQ_STATES[m.req_state] ?? REQ_STATES.wait}`;
}

/// 받은 요청에 지금 할 수 있는 상태 바꾸기 — 지금 상태 말고 네 상태 어디로든(되돌리기 포함 — 사용자 2026-10-19, 내 요청 · 질의 · 답은 없음). 닫힌 대화(`closed` —
/// [`convClosed`], #1w)는 읽기 전용이라 없음.
export function reqMoves(m, closed = false) {
  if (closed || m.author !== "human" || m.mine !== false || !m.request) return [];
  const from = m.req_state ?? "wait";
  return Object.keys(REQ_STATES)
    .filter((to) => to !== from)
    .map((to) => ({ to, label: REQ_STATES[to] }));
}

/// 받은 요청의 상태를 요청자에 알리지 못했다(오래 닿지 않아 포기 · 요청자가 거절) — 같은 상태로 "다시 알리기"(#1d R1 · R2). 닫힌 대화는 아님(#1w).
export function reqResend(m, closed = false) {
  return !closed && m.author === "human" && m.mine === false && m.request === true && !!m.refused;
}

/// '요청' 화면 행 — 상대 · 대화 제목 · 상태 · 메모 · 본문 앞부분 · 시각, 누르면 그 글로. `moves` = 받은 요청에 할 수 있는 상태 바꾸기,
/// `resend` = 알리지 못한 상태 다시 알리기.
export function requestRows(messages, peerNames) {
  return messages
    .filter((m) => m.conversation && m.request)
    .map((m) => {
      const conv = m.conversation;
      return {
        id: m.id,
        peer: conv.peer_id,
        who: shortName(peerNames.get(conv.peer_id)),
        title: convLabel(conv, peerNames).title,
        state: REQ_STATES[m.req_state] ?? REQ_STATES.wait,
        stateKey: m.req_state ?? "wait",
        done: m.req_state === "done" || m.req_state === "rejected",
        note: m.req_note ?? "",
        preview: preview(m.text),
        at: m.req_at ?? m.created_at,
        href: `#c/${conv.id}/m/${m.id}`,
        moves: reqMoves(m, !!convClosed(conv)),
        resend: reqResend(m, !!convClosed(conv)),
        archive: reqArchiveAction(m),
      };
    });
}

/// 요청 archive 단추(#1y — 이 PC 에만) — 받은 · 보낸 요청 모두: archive 됨 = "archive 해제", 완료 = "archive", 그 밖 `null`.
export function reqArchiveAction(m) {
  if (m.author !== "human" || !m.request) return null;
  if (m.archived) return { key: "unarchive", label: "archive 해제" };
  return m.req_state === "done" ? { key: "archive", label: "archive" } : null;
}

/// 주고받은 메시지 보기 기간(#1y) — 1 ~ 12주(서버 `run_opts::PEER_WEEKS_MAX`), 기본 4.
export const PEER_WEEKS = Array.from({ length: 12 }, (_, i) => i + 1);

/// 기간 고르기 항목 — `{value, label: "{n}주"}`.
export function peerWeeksOptions() {
  return PEER_WEEKS.map((n) => ({ value: String(n), label: `${n}주` }));
}

/// 요청 화면 절 데이터(#1q) — 쪽(`box`)마다 머리 건수 = 서버 집계(`GET /api/messages/req_counts` — 목록 상한과 무관, 적대적 평가 R1), 행 = 서버가 상태로 거른
/// 목록(`req=`) 그대로.
export function reqBoxes(boxes, lists, counts, peerNames) {
  return boxes.map((box, i) => ({ box, sum: counts[i], rows: requestRows(lists[i], peerNames) }));
}

/// 요청 개요 절 머리(#1q) — 이름 · 전체 건수 · 상태별 건수(대기 · 수용 · 완료 · 반려 순, 0건 상태는 뺀다).
export function reqHead(title, sum) {
  const states = ["wait", "accepted", "done", "rejected"].filter((k) => sum[k]).map((k) => ({ key: k, text: `${REQ_STATES[k]} ${sum[k]}` }));
  // archive 된 완료는 전체 수에 들지 않고 따로(#1y).
  if (sum.archived) states.push({ key: "archived", text: `archive ${sum.archived}` });
  return { title, count: String(sum.total), states };
}

/// 요청 목록 글자 단추 모양(#1q) — 바꿀 상태 → 색(`ok` 초록 · `no` 붉음) · 아이콘(`app.css` `.ico-{이름}`).
export const REQ_BTN = {
  wait: { tone: "wait", icon: "reset" },
  accepted: { tone: "ok", icon: "check" },
  done: { tone: "ok", icon: "check" },
  rejected: { tone: "no", icon: "x" },
};

/// 요청 보드 열 순서(#1v — D39 ①).
export const BOARD_COLS = ["wait", "accepted", "rejected", "done"];

/// 요청 보드 열(#1v) — 열 = 상태(`BOARD_COLS` 순), 건수 = 서버 집계(`req_counts` — 목록 상한과 무관), 카드 = 그 상태의 행.
export function boardColumns(rows, sum) {
  return BOARD_COLS.map((key) => ({ key, name: REQ_STATES[key], count: sum?.[key] ?? 0, rows: rows.filter((r) => r.stateKey === key) }));
}

/// 보드 카드를 그 열에 놓을 수 있나(#1v) — 받은 요청에 지금 할 수 있는 상태 바꾸기(`reqMoves` — 네 상태 사이 어디로든)만. 보낸 요청 · 닫힌 대화 · 같은 열은 거짓.
export function canDrop(row, to) {
  return row.moves.some((m) => m.to === to);
}

/// 요청 의견 말풍선(#1v) — 대기가 아닌 요청 바로 아래 하나: 지금 상태 · 메모 · 바뀐 시각, 방향 = 받은 쪽 사람 쪽(요청 말풍선의 반대). 아니면 `null`.
export function reqFeedback(m) {
  if (m.author !== "human" || !m.request || (m.req_state ?? "wait") === "wait") return null;
  return {
    key: m.req_state,
    state: REQ_STATES[m.req_state] ?? m.req_state,
    note: m.req_note ?? "",
    at: m.req_at ?? m.created_at,
    side: bubbleSide(m) === "mine" ? "theirs" : "mine",
  };
}

/// 사이드바 대화방 절 — ★ 는 줄의 별로 보이고 대화방은 제자리에 남는다(#1j — 모아 보기는 메뉴 "즐겨찾기"). `paired`(레일 순서 상대 id)를 주면
/// ('나', #1g) 상대별 절(`peer:{id}`)로 — 대화가 하나도 없으면 빈 '대화' 절 하나.
export function convSections(rows, paired = null, peerNames = new Map()) {
  if (paired == null) return [{ key: "convs", title: "대화", rows }];
  const groups = peerGroups(
    rows.map((r) => ({ ...r, peer: r.c.peer_id })),
    paired,
    peerNames,
  ).map((g) => ({ key: `peer:${g.peer}`, peer: g.peer, title: g.name, rows: g.rows }));
  return groups.length ? groups : [{ key: "convs", title: "대화", rows: [] }];
}

/// 대화 한 개의 관람자 상한(#1o — 서버 `share::MAX_OBSERVERS` 와 같다).
export const MAX_OBSERVERS = 5;

/// 대화 머리의 공유 표시(#1o) — 공유 대화(사본) = "읽기 전용" · 빠졌으면 "공유 해제됨", 내 대화 · agent 대화 = 관람자가 있으면 "관람자 n"(초대한 쪽은
/// `observers`, 원 상대는 알림 받은 `watchers`).
export function shareHead(conv) {
  if (conv.dir === "shared") return conv.share?.ended != null ? "공유 해제됨" : "읽기 전용";
  const n = conv.observers?.length || conv.watchers || 0;
  return n ? `관람자 ${n}` : "";
}

/// 관람자로 넣을 수 있는 상대(#1o) — 페어링된 상대 중 원 상대 · 이미 관람자인 상대를 뺀다. 공유 대화(사본) · 상한이면 빈 목록.
export function shareCandidates(conv, peers) {
  const now = conv.observers ?? [];
  if (conv.dir === "shared" || now.length >= MAX_OBSERVERS) return [];
  return peers.filter((p) => p.paired && p.id !== conv.peer_id && !now.includes(p.id));
}

/// 대화방 목록 묶음(#1o) — 대화의 `dir` 로: 내가 연 대화(자동 왕복 포함) · 상대가 연 대화(내 에이전트가 답). 순서 = 이 배열 순서.
export const CONV_GROUPS = [
  { dir: "out", title: "내 대화" },
  { dir: "in", title: "에이전트 대화" },
  { dir: "shared", title: "공유 대화" },
];

/// 한 절의 행을 묶음으로 나눈다(#1o) — 행 있는 묶음만, 묶음 안 순서는 들어온 그대로. 묶음이 둘 이상일 때만 머리를 보인다(하나면 지금처럼 한 목록).
export function splitByDir(rows) {
  return CONV_GROUPS.map((g) => ({ ...g, rows: rows.filter((r) => r.c.dir === g.dir) })).filter((g) => g.rows.length);
}

/// 사이드바 대화 목록에 보일 줄(#1v) — 나 · 상대 타일은 에이전트 대화(`in`)를 뺀다. 상대별 절 · 빈 안내 판정도 이 줄로 한다(적대적 평가 R1 — 빼기 전 줄로
/// 절을 만들면 에이전트 대화뿐인 상대 절 · 검색 결과가 빈 채로 남았다).
export function visibleConvRows(rows, group) {
  return group ? rows : rows.filter((r) => r.c.dir !== "in");
}

/// 사이드바 대화 목록 묶음(#1v) — 나 · 상대 타일(`group` 없음)은 에이전트 대화(`in`)를 빼고(사이드바 '에이전트 대화' 화면으로) 내 대화는 머리 없이,
/// 공유 대화만 머리. 그룹 타일은 지금대로(묶음이 둘 이상이면 묶음마다 머리). `head` = 머리를 보일지.
export function convListParts(rows, group) {
  const parts = splitByDir(visibleConvRows(rows, group));
  return parts.map((g) => ({ ...g, head: group ? parts.length > 1 : g.dir !== "out" }));
}

/// 에이전트 대화 화면(#1v) — 상대가 연 대화(`in`)를 요청한 상대별 절로(`peerGroups` — `paired` 순서, 그 뒤 풀린 상대). 상대 타일은 `paired` = [그 상대].
export function agentConvSections(convs, paired, peerNames) {
  const rows = convRows(convs.filter((c) => c.dir === "in"), peerNames, null).map((r) => ({ ...r, peer: r.c.peer_id }));
  return peerGroups(rows, paired, peerNames).map((g) => ({ key: `agentconvs:${g.peer}`, peer: g.peer, title: g.name, rows: g.rows }));
}

/// 대화 줄의 끌기 묶음 열쇠(#1o) — 끌어 옮기기 · Alt + ↑ / ↓ 는 같은 열쇠 안에서만(`dropKind`).
export function splitKey(section, dir) {
  return `${section}:${dir}`;
}

/// 상대별 절(#1g, '나') — 행(`peer` = 상대 id)을 레일 순서(`paired`)로 묶고, 그 뒤 페어링이 풀린 상대를 id 순으로. 행 없는 상대는 뺀다.
/// 절 안 순서는 들어온 순서(목록이 이미 최신순).
export function peerGroups(rows, paired, peerNames) {
  const rest = [...new Set(rows.map((r) => r.peer))].filter((id) => !paired.includes(id)).sort((a, b) => a - b);
  return [...paired, ...rest]
    .map((id) => ({ peer: id, name: shortName(peerNames.get(id)), rows: rows.filter((r) => r.peer === id) }))
    .filter((g) => g.rows.length);
}

/// 받은 대기 요청 수를 상대별로(#1g — '대화' 요약 머리).
export function waitingByPeer(messages) {
  const n = new Map();
  for (const m of messages) if (m.conversation) n.set(m.conversation.peer_id, (n.get(m.conversation.peer_id) ?? 0) + 1);
  return n;
}

/// '나' 의 '대화' = 상대별 요약(#1g) — 페어링된 상대(레일 순서)는 대화가 없어도, 풀린 상대는 대화가 있을 때만. 대화 수 · 안 읽음 합 · 대기 요청 ·
/// 최근 활동(`last_at` 최대, 없으면 `null`) · 그 상대 대화(손으로 놓은 순서 — #1i `sortByPos`).
export function peerSummary(convs, paired, peerNames, waiting = new Map()) {
  const rest = [...new Set(convs.map((c) => c.peer_id))].filter((id) => !paired.includes(id)).sort((a, b) => a - b);
  return [...paired, ...rest].map((id) => {
    const rows = sortByPos(convs.filter((c) => c.peer_id === id));
    const ats = rows.map((c) => c.last_at).filter((t) => t != null);
    return {
      peer: id,
      name: shortName(peerNames.get(id)),
      convs: rows.length,
      unread: rows.reduce((n, c) => n + (c.unread ?? 0), 0),
      waiting: waiting.get(id) ?? 0,
      last: ats.length ? Math.max(...ats) : null,
      rows,
    };
  });
}

/// 대화 행 표시 — 둘째 줄 `{에이전트} · {상태}`(받은 대화의 에이전트는 내 것), 실행 중 표시, 자동 왕복 칩 "왕복 {n}/{상한}", 안 읽음.
export function convMeta(c) {
  return {
    agent: c.dir === "in" ? `내 ${c.agent}` : c.agent,
    state: stateText(c.last_state),
    running: c.last_state === "running",
    relay: c.relay_agent && c.relay_cap != null ? `왕복 ${c.relay_count ?? 0}/${c.relay_cap}` : null,
    unread: c.unread > 0,
    // 닫힌 대화(#1w — 나감 · 상대가 나감 · 공유 해제됨) — 읽기 전용으로 남는다.
    tag: convClosed(c) || null,
  };
}

/// 닫힌 대화의 문구(#1w) — 내가 나감 "나감" · 상대가 나감 "상대가 나감" · 공유 사본을 빼거나 내가 나감 "공유 해제됨"(#1o), 아니면 빈 글자. 읽기 전용 · 대화 줄 태그 · 대화 머리.
export function convClosed(c) {
  if (!c) return "";
  if (c.dir === "shared") return c.share?.ended != null ? "공유 해제됨" : "";
  if (c.left_at != null) return "나감";
  return c.peer_left_at != null ? "상대가 나감" : "";
}

/// 나가기 확인 문구(#1x — 사용자 원문).
export const LEAVE_CONFIRM = "대화방이 목록에서 삭제됩니다.";

/// 대화 ⋯ 메뉴 끝 항목(#1x — D40 개정) — "기록 삭제" ▸(`sub` — 내 글만 삭제 · 모든 사용자 글 삭제) · "나가기"(확인 뒤 이 PC 목록에서 삭제). 닫힌 대화(나감 · 상대가 나감)는
/// 내 글만, 공유 사본(관람자 — 읽기 전용)은 나가기만. `icon` = `app.css` `.ico-{이름}`.
export function convMenuItems(conv) {
  const leave = { key: "leave", label: "나가기", icon: "exit", danger: true, confirm: LEAVE_CONFIRM };
  if (conv.dir === "shared") return [leave];
  const sub = [{ key: "clear_mine", label: "내 글만 삭제" }];
  if (!convClosed(conv)) sub.push({ key: "clear_all", label: "모든 사용자 글 삭제" });
  return [{ key: "clear", label: "기록 삭제", icon: "trash", sub }, leave];
}

/// 사이드바 머리 부제(#1x) — '나' = 이름(별칭이면 호스트 더), 상대 = 별칭이면 "{이름} · {호스트}" 아니면 호스트. 지문 · "페어링 됨" · 접속 문구는 없다(접속은 레일 점).
export function sideSub(kind, info, alias) {
  if (kind === "me") return [info.name, alias ? info.host : ""].filter(Boolean).join(" · ");
  return info.alias ? `${info.name} · ${info.host}` : info.host;
}

/// 세션 생성 화면 상대 칸 처음 값(#1x) — 레일에서 고른 상대가 페어링됐으면 그 id(문자열), 아니면 빈 값("상대 선택").
export function composeDefaults(railPeer, peers) {
  return peers.some((p) => p.id === railPeer && p.paired) ? String(railPeer) : "";
}

/// 서버 알림 → 브라우저 알림(#1x — 받은 요청 B안). 받은 요청 = 같은 tag(앞 알림을 바꾼다) · 소리 없음(효과음은 웹UI) · 누르면 요청 보드, 그 밖은 종류별 tag · 내 활동
/// (페어링은 호출자가 설정으로). `sound` = 효과음을 낼 알림인가.
export function requestNotice(n) {
  if (n.kind === "request") return { title: n.title, body: n.body, tag: "coworkagent-request", href: "#requests", sound: true };
  return { title: n.title, body: n.body, tag: "coworkagent-" + n.kind, href: "#activity", sound: false };
}

// ---- 레일 그룹 · 손으로 놓는 순서(#1i) ----

/// 타일 색 — id 로 다섯 색을 돈다(상대 · 그룹, 자리를 옮겨도 색이 따라간다).
export function colorOf(id) {
  return `peer-${(((id % 5) + 5) % 5) + 1}`;
}

/// 손으로 놓은 순서 — `pos` 작은 것 먼저, 같으면 id(서버 `peers::list` 와 같은 규칙). 새 배열.
export function sortByPos(items) {
  return [...items].sort((a, b) => (a.pos ?? 0) - (b.pos ?? 0) || a.id - b.id);
}

/// 전체 순서(`order` — id 목록)에서 `id` 를 빼 `target` 바로 앞(`after` 면 뒤)에 끼운다 — 걸러 보인 목록에서 옮겨도 그 앞뒤가 다른 화면에도
/// 같다(#1i A9). 같은 id 거나 `target` 이 없으면 그대로(새 배열).
export function moveInOrder(order, id, target, after = false) {
  if (id === target || !order.includes(id) || !order.includes(target)) return [...order];
  const rest = order.filter((x) => x !== id);
  const i = rest.indexOf(target) + (after ? 1 : 0);
  return [...rest.slice(0, i), id, ...rest.slice(i)];
}

/// 늦게 온 목록 응답에 덮어 씌울 순서(#1i 라운드 1) — 저장 상태 `saved = {last, busy, stamp}`, `stamp` = 그 읽기를 시작할 때의 표. 저장 중이거나
/// 읽는 사이 옮김 · 저장이 있었으면 마지막으로 놓은 순서, 아니면 `null`(받은 그대로).
export function orderToApply(saved, stamp) {
  if (!saved?.last) return null;
  return saved.busy || saved.stamp !== stamp ? saved.last : null;
}

/// 새 순서를 화면 값에 바로 — `ids` 의 자리(0 ..)를 `pos` 로, 빠진 항목은 제 `pos` 그대로(서버 `PUT /api/order` 와 같은 규칙). 새 배열.
export function withOrder(items, ids) {
  const at = new Map(ids.map((id, i) => [id, i]));
  return items.map((x) => (at.has(x.id) ? { ...x, pos: at.get(x.id) } : x));
}

/// 옮긴 뒤 화면 읽기 프로그램 알림 — "{이름} — n번째로 옮김"(1 부터).
export function movedText(name, index) {
  return `${name} · ${index + 1}번째로 옮김`;
}

/// 그룹 타일을 고른 채 열 수 없는 화면(#1i) — 내 활동 · 주고받은 글(#1j — '나' 에만) · 에이전트(대화 단위 아님) → '나' 로 바꿔 연다.
export function groupLeaves(view) {
  return view === "activity" || view === "agents" || view === "agentConvs" || view === "peer";
}

/// 상대 타일을 고른 채 열 수 없는 화면(#1z-2) — 에이전트 응답('나' 에만).
export function peerLeaves(view) {
  return view === "agentConvs";
}

/// 그룹의 안 읽음 합(레일 배지).
export function groupUnread(convs, group) {
  return inGroup(convs, group).reduce((n, c) => n + (c.unread ?? 0), 0);
}

/// 키보드 한 칸(Alt + ↑ / ↓) — 보이는 같은 묶음(`visible` — 화면 순서 id)에서 `dir`(-1 위 · +1 아래) 쪽 이웃의 앞 · 뒤. 끝이면 `null`.
export function stepTarget(visible, id, dir) {
  const i = visible.indexOf(id);
  const j = i + dir;
  if (i < 0 || j < 0 || j >= visible.length) return null;
  return { target: visible[j], after: dir > 0 };
}

/// 끌어 놓기 판정 — 끄는 것(`drag` = `{kind, section}`) → 놓을 곳(`target` = `{kind, section}`): `"move"`(같은 종류 · 대화는 같은 절) ·
/// `"join"`(대화 → 그룹 타일, 그룹에 넣기) · `null`(놓을 수 없음 — '나' · "+" · 설정 · 다른 묶음 · 다른 절).
export function dropKind(drag, target) {
  if (!drag || !target) return null;
  if (drag.kind === "conv" && target.kind === "group") return "join";
  if (drag.kind !== target.kind || !["peer", "group", "conv"].includes(drag.kind)) return null;
  if (drag.kind === "conv" && drag.section !== target.section) return null;
  return "move";
}

/// 그룹의 대화 — 넣은 대화(`convs`, #1w — 그룹 = 대화만). 순서는 들어온 그대로.
export function inGroup(convs, group) {
  const ids = new Set(group?.convs ?? []);
  return convs.filter((c) => ids.has(c.id));
}

/// 그룹 요약에 머리를 둘 페어링된 상대 — 레일 순서 중 그 그룹에 대화가 있는 상대만('나' 처럼 대화 없는 상대를 늘어놓지 않는다).
export function groupPeers(convs, paired) {
  const has = new Set(convs.map((c) => c.peer_id));
  return paired.filter((id) => has.has(id));
}

/// 목록 범위 쿼리(`peer=` · `group=`, 없으면 빈 문자열) — 상대 · 그룹 중 하나만.
export function scopeQuery(peer, group) {
  if (group != null) return `group=${group}`;
  if (peer != null) return `peer=${peer}`;
  return "";
}

/// 그룹 이름 미리 거르기(서버가 정본) — 앞뒤 공백 뺀 1 ~ 40자 · 같은 이름(`except` = 이름 바꾸는 그 그룹 id) 없음. 맞으면 `null`.
export function groupNameError(name, groups, except = null) {
  const n = (name ?? "").trim();
  if (!n || [...n].length > 40) return "그룹 이름은 1 ~ 40자";
  if (groups.some((g) => g.name === n && g.id !== except)) return "같은 이름의 그룹이 있습니다";
  return null;
}

/// 그룹 소속 바꾸기 — 고른 대화(`picked`) 와 지금 소속(`now`)의 차이 → `{add, remove}`(작은 id 부터).
export function membershipDiff(now, picked) {
  const a = new Set(now);
  const b = new Set(picked);
  return {
    add: [...b].filter((id) => !a.has(id)).sort((x, y) => x - y),
    remove: [...a].filter((id) => !b.has(id)).sort((x, y) => x - y),
  };
}

/// 레일 배지 — 상대별 안 읽음 합(0 인 상대는 없다).
export function unreadByPeer(convs) {
  const sums = new Map();
  for (const c of convs) if (c.unread > 0) sums.set(c.peer_id, (sums.get(c.peer_id) ?? 0) + c.unread);
  return sums;
}

/// 상대 타일 배지(#1z-2) — 그 타일에서 보이는 대화의 안 읽음만. 에이전트 대화(`dir = "in"`)는 '나' 의 '에이전트 응답' 이 센다(상대 타일에는 그 메뉴가 없다).
export function tileUnread(convs) {
  return unreadByPeer(convs.filter((c) => c.dir !== "in"));
}

/// 내 활동 거르기 탭(#1q) — `kind` = `GET /api/activity?kind=`, `icon` = `.ico-{이름}`(#1r).
export const ACT_KINDS = [
  { kind: "all", label: "전체", icon: "bell" },
  { kind: "request", label: "받은 요청", icon: "arrow-down" },
  { kind: "mine", label: "내 요청", icon: "arrow-up" },
  { kind: "answer", label: "받은 답", icon: "chat" },
  { kind: "failed", label: "실패", icon: "x" },
];

/// 내 활동 항목(`GET /api/activity`) → 목록 행(#1q). 라벨 = "받은 요청 · {상태}" · "받은 답"(실패한 답이면 "· 실패") · "내 요청 · {상태}"(+ 메모) ·
/// "실패"(내 쪽) · "상대 반영 실패 · {사유}". 글은 누르면 그 글로(`#c/{대화}/m/{글}`), 상대 반영 실패는 그 대화로. 대화를 모르는 항목 · 모르는 종류는 뺀다.
/// `key` = 다시 그려도 같은 항목을 가리키는 열쇠.
export function activityRows(items, peerNames) {
  return items
    .filter((a) => a.conversation && (a.message || a.kind === "sync_failed"))
    .map((a) => {
      const conv = a.conversation;
      const peer = peerNames.get(conv.peer_id);
      const m = a.message;
      const row = {
        key: `${a.kind}:${conv.id}:${m?.id ?? `${a.at}:${a.detail ?? ""}`}`,
        peer: conv.peer_id,
        at: a.at,
        unseen: a.unseen === true,
        title: convLabel(conv, peerNames).title,
        note: "",
      };
      if (!m) return { ...row, label: `상대 반영 실패 · ${a.detail ?? ""}`, who: shortName(peer), preview: "", href: `#c/${conv.id}` };
      const state = REQ_STATES[m.req_state] ?? REQ_STATES.wait;
      const label =
        a.kind === "failed"
          ? "실패"
          : a.kind === "my_request"
            ? `내 요청 · ${state}`
            : m.author === "human"
              ? `받은 요청 · ${state}`
              : m.state === "error"
                ? "받은 답 · 실패"
                : "받은 답";
      const note = a.kind === "my_request" ? (m.req_note ?? "") : "";
      return { ...row, label, note, author: m.author, who: authorLabel(conv, m, peer), preview: preview(m.text), href: `#c/${conv.id}/m/${m.id}` };
    });
}

/// 메시지 작업 막대(말풍선 아래 작은 단추) — 북마크 · 복사는 말풍선 우상단 아이콘(#1j), "다시 보내기" 는 없앴다(#1k — 입력창에서 다시 쓴다).
/// 받은 요청만 상태 바꾸기(`req:{상태}` — [`reqMoves`], 쓴 쪽에 간다). 복사는 말풍선 우상단 아이콘(#1f), 편집 · 삭제는 편집 모드(#1f) —
/// 막대에는 없다. 원문 보기는 머리글 하나.
export function messageActions(m, closed = false) {
  const acts = [];
  for (const r of reqMoves(m, closed)) acts.push({ key: `req:${r.to}`, label: r.label });
  if (reqResend(m, closed)) acts.push({ key: "req-again", label: "다시 알리기" });
  return acts;
}

/// 편집 표시(#1f) — 편집한 글이면 "(편집됨)".
export function editedTag(m) {
  return m.edited_at != null ? "(편집됨)" : "";
}

/// 입력창 서식 버튼 — 고른 글자(`start` ~ `end`)에 markdown 표기: 굵게 `**…**` · 코드 `` `…` ``(여러 줄이면 코드 블록) · 목록(걸친 줄마다 머리에
/// "- "). 돌려주는 `start` · `end` = 새 선택(빈 선택이면 커서 자리).
export function insertMarkdown(text, start, end, kind) {
  const sel = text.slice(start, end);
  const wrap = (open, close) => ({
    text: text.slice(0, start) + open + sel + close + text.slice(end),
    start: start + open.length,
    end: start + open.length + sel.length,
  });
  if (kind === "bold") return wrap("**", "**");
  if (kind === "code") return sel.includes("\n") ? wrap("```\n", "\n```") : wrap("`", "`");
  const head = start === 0 ? 0 : text.lastIndexOf("\n", start - 1) + 1;
  const lines = text.slice(head, end).split("\n");
  // 선택이 줄바꿈으로 끝나면 그 뒤 빈 줄에는 붙이지 않는다.
  const listed = lines.map((l, i) => (i > 0 && i === lines.length - 1 && l === "" ? l : "- " + l)).join("\n");
  const out = text.slice(0, head) + listed + text.slice(end);
  return start === end ? { text: out, start: end + 2, end: end + 2 } : { text: out, start: head, end: head + listed.length };
}

/// 발화자 종류 아이콘(#1z — 사람 ↔ 에이전트 구분) — 대화 목록 미리보기 · 메시지 줄 앞. `icon` = `app.css` `.ico-{이름}`.
export function speakerIcon(author) {
  return author === "agent" ? { kind: "agent", icon: "bot", label: "에이전트" } : { kind: "human", icon: "user", label: "사람" };
}

/// 주고받은 메시지 줄 아바타(#1z-2 — 시안 `Peer`) — 에이전트 · 나(내 사람 글) · 상대(상대 사람 글), 아이콘 · 메타의 "사람" 은 `speakerIcon` 과 같다.
export function avatarKind(author, mine) {
  return author === "agent" ? "agent" : mine ? "me" : "peer";
}

/// 레일 폭(#1z — 손잡이 끌기) — 끄는 동안 `min` ~ `max`, `split` 이상이면 작은 아이콘, 놓으면 `normal`(크게) · `small`(작게) 폭으로 맞춘다.
export const RAIL_W = { normal: 72, small: 168, split: 120, min: 64, max: 240 };

export function railSize(w) {
  return w >= RAIL_W.split ? "small" : "normal";
}

/// 요청 의견 글자 수(#1z) — 서버(`MAX_REQ_NOTE`)처럼 문자 수로 센다.
export const NOTE_MAX = 200;

export function noteCount(text) {
  return `${[...text].length} / ${NOTE_MAX}`;
}

/// 입력창 Enter = 보내기, Shift+Enter = 줄바꿈. 한글 조합 중(`isComposing` · keyCode 229)의 Enter 는 조합을 끝내는 키라 보내지 않는다.
/// 누르고 있는 Enter(키 반복)도 보내지 않는다 — 같은 글이 여러 번 가지 않게.
export function enterSends(ev) {
  return ev.key === "Enter" && !ev.shiftKey && !ev.repeat && !ev.isComposing && ev.keyCode !== 229;
}

/// 실행 상세 '진행' — 고른 요청(`timeline()` 항목)의 단계: 요청 → 대기열 → 실행 → 응답, 단계 상태 = done · now · wait · fail. 대기 시작 · 끝 시각은
/// 남지 않아 시각은 요청 · 응답 때만(HH:MM:SS). 보낸 쪽은 상대가 알려 준 진행(`progress`)으로 대기 · 실행을 가른다(없으면 답이 오기 전까지 대기열).
/// 실행 줄 = 답의 `{model} · {effort} · {n}턴 · {시간}`(없는 값은 뺀다). 상대가 보낸 요청이면 "요청 받음" · "응답 보냄".
export function runSteps({ request, reply }) {
  const mine = request.mine !== false;
  const failed = reply ? reply.state === "error" : request.state === "error";
  const done = reply != null && !failed;
  const running = !reply && (request.state === "running" || (request.state === "sent" && request.progress?.state === "running"));
  const meta = reply
    ? [reply.model, reply.effort, reply.turns != null ? `${reply.turns}턴` : "", formatDuration(reply.duration_ms)].filter(Boolean).join(" · ")
    : "";
  return [
    { key: "request", title: mine ? "요청 보냄" : "요청 받음", at: formatClock(request.created_at), state: "done" },
    { key: "queue", title: "대기열", at: "", state: reply || running || failed ? "done" : "now", meta: progressText(request) },
    {
      key: "run",
      title: failed ? "실패" : done ? "실행 완료" : running ? "실행 중" : "실행",
      at: "",
      state: failed ? "fail" : done ? "done" : running ? "now" : "wait",
      meta,
    },
    {
      key: "reply",
      title: mine ? "응답 받음" : "응답 보냄",
      at: reply ? formatClock(reply.created_at) : "",
      state: !reply ? "wait" : reply.state === "done" ? "done" : "fail",
      meta: reply && reply.state !== "done" ? stateText(reply.state) : "",
    },
  ];
}

/// 대화 에이전트의 마지막 답이 쓴 `{model} · {effort}`(없는 값은 빼고, 답이 없거나 그 답에 값이 없으면 "") — 대화 머리글 부제 · 실행 상세.
/// `mine` = 대화 에이전트가 이 PC 것인가(받은 대화) — 자동 왕복의 반대쪽 에이전트 답은 보지 않는다.
export function lastRun(messages, mine) {
  const m = [...messages].reverse().find((x) => x.author === "agent" && x.mine === mine);
  return m ? [m.model, m.effort].filter(Boolean).join(" · ") : "";
}

/// 통합 검색 결과(`GET /api/search`) → 묶음(메시지 · 대화 · 사람 · 에이전트 — 빈 묶음 없음). 행 = `{label, sub}` + 가는 곳: `href` · `tile`(그 타일
/// 선택 — 페어링된 사람) · `chat`(세션 생성에 상대 · 에이전트 채움 — 상대 에이전트).
export function searchGroups(r, peerNames) {
  const groups = [
    {
      title: "메시지",
      rows: (r.messages ?? [])
        .filter((m) => m.conversation)
        .map((m) => {
          const conv = m.conversation;
          const who = authorLabel(conv, m, peerNames.get(conv.peer_id));
          return { label: preview(m.text), sub: `${who} · ${convLabel(conv, peerNames).title}`, href: `#c/${conv.id}/m/${m.id}` };
        }),
    },
    {
      title: "대화",
      rows: (r.conversations ?? []).map((c) => {
        const l = convLabel(c, peerNames);
        return { label: l.title, sub: l.sub, href: `#c/${c.id}` };
      }),
    },
    {
      title: "사람",
      rows: (r.peers ?? []).map((p) =>
        p.paired ? { label: displayName(p), sub: p.host, tile: p.id } : { label: displayName(p), sub: `${p.host} · ${peerState(p)}`, href: "#settings/peers" },
      ),
    },
    {
      title: "에이전트",
      rows: (r.agents ?? []).map((a) =>
        a.owner === "me"
          ? { label: a.name, sub: "내 에이전트", href: "#settings/agents" }
          : { label: a.name, sub: `${shortName(peerNames.get(a.owner))} 의 에이전트`, chat: { peer: a.owner, agent: a.name } },
      ),
    },
  ];
  return groups.filter((g) => g.rows.length);
}

// ---- 뷰어 ----

/// HTML 문서로 보이는 본문(`<!doctype html` · `<html` 로 시작) — sandbox iframe 으로 그린다.
export function isHtmlDoc(text) {
  return /^\s*(<!doctype\s+html|<html[\s>])/i.test(text ?? "");
}

/// iframe `srcdoc` 앞에 붙이는 CSP — 스크립트 · 외부 리소스 차단, 인라인 스타일 · data: 그림만. `<base target="_top">` — 본문 링크를 눌러도
/// 프레임이 외부로 가지 않는다(top 이동은 sandbox 가 막는다).
export const HTML_CSP = "default-src 'none'; style-src 'unsafe-inline'; img-src data:";

export function htmlSrcdoc(text) {
  return `<meta http-equiv="Content-Security-Policy" content="${HTML_CSP}"><base target="_top">` + text;
}

/// 인라인 — `코드` · **굵게** · *기울임* · _기울임_. 그 밖(링크 · HTML 태그 포함)은 글자 그대로.
export function inlines(text) {
  const out = [];
  const re = /`([^`]+)`|\*\*([^*]+)\*\*|\*([^*\s][^*]*)\*|(?<![\p{L}\p{N}_])_([^_\s][^_]*)_(?![\p{L}\p{N}_])/gu;
  let at = 0;
  for (const m of text.matchAll(re)) {
    if (m.index > at) out.push({ t: "text", v: text.slice(at, m.index) });
    if (m[1] != null) out.push({ t: "code", v: m[1] });
    else if (m[2] != null) out.push({ t: "b", v: m[2] });
    else out.push({ t: "i", v: m[3] ?? m[4] });
    at = m.index + m[0].length;
  }
  if (at < text.length) out.push({ t: "text", v: text.slice(at) });
  return out;
}

const ROW = /^\s*\|.*\|\s*$/;
const RULE = /^\s*\|?\s*:?-{3,}/;
const ITEM = /^\s*([-*+]|\d+[.)])\s+(.*)$/;

function cells(line) {
  return line
    .trim()
    .replace(/^\|/, "")
    .replace(/\|$/, "")
    .split("|")
    .map((c) => inlines(c.trim()));
}

/// markdown → 블록 목록(제목 · 목록 · 코드 블록 · 인용 · 표 · 문단). 값은 글자뿐 — 그리는 쪽(app.js)이 textContent 로만 넣는다.
export function markdownBlocks(text) {
  const lines = (text ?? "").replace(/\r\n?/g, "\n").split("\n");
  const blocks = [];
  let para = [];
  const flush = () => {
    if (para.length) blocks.push({ type: "p", inlines: inlines(para.join(" ")) });
    para = [];
  };
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    let m;
    if (/^\s*```/.test(line)) {
      flush();
      const code = [];
      for (i++; i < lines.length && !/^\s*```/.test(lines[i]); i++) code.push(lines[i]);
      blocks.push({ type: "code", text: code.join("\n") });
    } else if ((m = /^(#{1,6})\s+(.*)$/.exec(line))) {
      flush();
      blocks.push({ type: "h", level: m[1].length, inlines: inlines(m[2].trim()) });
    } else if (ROW.test(line) && RULE.test(lines[i + 1] ?? "")) {
      flush();
      const head = cells(line);
      const rows = [];
      for (i += 2; i < lines.length && ROW.test(lines[i]); i++) rows.push(cells(lines[i]));
      i--;
      blocks.push({ type: "table", head, rows });
    } else if ((m = ITEM.exec(line))) {
      flush();
      const ordered = /\d/.test(m[1]);
      const items = [inlines(m[2])];
      for (let n; i + 1 < lines.length && (n = ITEM.exec(lines[i + 1])) && /\d/.test(n[1]) === ordered; i++) items.push(inlines(n[2]));
      blocks.push({ type: ordered ? "ol" : "ul", items });
    } else if ((m = /^\s*>\s?(.*)$/.exec(line))) {
      flush();
      const quote = [m[1]];
      for (; i + 1 < lines.length && /^\s*>/.test(lines[i + 1]); i++) quote.push(lines[i + 1].replace(/^\s*>\s?/, ""));
      blocks.push({ type: "quote", inlines: inlines(quote.join(" ")) });
    } else if (!line.trim()) {
      flush();
    } else {
      para.push(line.trim());
    }
  }
  flush();
  return blocks;
}

/// 모든 실행의 도구 — 탐색(읽기 전용) 전부(an_core `READ_ONLY_TOOLS` 와 같다, #1d — 프로필마다 고르지 않는다).
export const READ_ONLY_TOOLS = ["Read", "Grep", "Glob", "LS", "NotebookRead"];
/// 에이전트 화면의 도구 안내 한 줄.
export const TOOLS_NOTE = `탐색 도구(읽기 전용) 항상 사용: ${READ_ONLY_TOOLS.join(" · ")}`;
/// 고를 수 있는 모델(an_core `run_opts::MODELS` 와 같다).
export const MODELS = ["opus", "sonnet", "haiku"];
/// `claude --effort` 값(an_core `run_opts::EFFORTS` 와 같다).
export const EFFORTS = ["low", "medium", "high", "xhigh", "max"];

/// 탭 메모리 토큰(1a — 쿠키 없음). 주소의 `t`(64 hex)가 있으면 그것, 없으면 탭에 저장된 값. `clean` = 주소창에서 `t` 를 지울지.
export function entryToken(search, stored) {
  const hex = (t) => typeof t === "string" && /^[0-9a-f]{64}$/.test(t);
  const t = new URLSearchParams(search ?? "").get("t");
  if (hex(t)) return { token: t, clean: true };
  return { token: hex(stored) ? stored : null, clean: t !== null };
}

/// 토큰이 없거나 거절됐을 때 화면 위 안내.
export const DISCONNECTED = "연결이 끊겼습니다. 트레이 아이콘 메뉴 '열기' 로 다시 여세요.";

/// 컨텍스트 상한(토큰, #1m — 1M 고정) · 경고 문턱(60% 주황 · 80% 붉음 — 80% 는 자동 압축 문턱과 같다).
export const CONTEXT_LIMIT = 1_000_000;

/// 대화 컨텍스트 크기(#1m) — 마지막으로 끝난 압축 · 비우기 구분선 뒤 답들에서 세션(에이전트 · 쪽)별 마지막 창 사용량의 최대. 그 뒤 답이 없으면 0,
/// 구분선도 크기 있는 답도 없으면 `null`(모름 — 이전 버전 상대).
export function contextSize(ms) {
  const last = new Map();
  let reset = false;
  for (const m of ms) {
    if (m.author === "agent" && m.ctx_tokens != null) last.set(`${m.mine ? 1 : 0}:${m.agent ?? ""}`, m.ctx_tokens);
    if (m.author !== "context" || m.state !== "done") continue;
    reset = true;
    // 자동 왕복 부분 처리(상대 PC 쪽 실패 — `peer_*`) — 이 PC 쪽 세션만 비웠다, 상대 세션 값은 남긴다(라운드 2).
    if (m.refused?.startsWith("peer_")) {
      for (const k of [...last.keys()]) if (k.startsWith("1:")) last.delete(k);
    } else {
      last.clear();
    }
  }
  if (last.size) return Math.max(...last.values());
  return reset ? 0 : null;
}

/// 머리 표시 — `{text, level}`: "42k / 1M" · 단계 `ok` · `warn`(60%) · `high`(80%) · `unknown`.
export function contextLabel(n) {
  if (n == null) return { text: "컨텍스트 알 수 없음", level: "unknown" };
  const v = n < 1000 ? String(n) : n < CONTEXT_LIMIT ? `${Math.round(n / 1000)}k` : `${(n / CONTEXT_LIMIT).toFixed(1)}M`;
  const level = n >= CONTEXT_LIMIT * 0.8 ? "high" : n >= CONTEXT_LIMIT * 0.6 ? "warn" : "ok";
  return { text: `${v} / 1M`, level };
}

/// 실행 · 압축이 걸린 대화인가(#1m — 압축 · 비우기 메뉴를 끈다): 대기 · 실행 중 글, 상대 답을 기다리는 실행 요청, 진행 중 구분선.
export function contextBusy(ms) {
  return ms.some((m) => m.state === "queued" || m.state === "running" || (m.state === "sent" && (m.exec || m.author === "context")));
}

/// 압축 · 비우기 메뉴(#1m) — 실행 중이거나 답(세션)이 아직 없으면 꺼진다.
export function ctxMenu(busy, answered) {
  const off = busy || !answered;
  return [
    { key: "compact", label: "압축", icon: "compress", disabled: off },
    { key: "clear", label: "비우기", icon: "reset", disabled: off },
  ];
}

function contextWhy(code) {
  if (code === "unsupported") return "상대가 0.5.0 이상이어야 합니다";
  if (code === "unreachable") return "상대에 닿지 않음";
  if (code === "compact_failed") return "요약 실행 실패 · 세션은 그대로";
  return `실패(${code ?? "알 수 없음"})`;
}

/// 컨텍스트 구분선 문구(#1m) — 종류 · 상태 · 실패 사유.
export function contextNote(m) {
  const compact = m.ctx_op !== "clear";
  const name = m.ctx_op === "auto_compact" ? "자동 압축(80%)" : compact ? "컨텍스트 압축" : "컨텍스트 비움";
  if (m.state === "queued" || m.state === "running") return `${name} · ${compact ? "요약 중…" : "비우는 중…"}`;
  if (m.state === "sent") return `${name} · 상대 PC 처리 대기`;
  if (m.state === "error") return `${name} · ${contextWhy(m.refused)}`;
  const done = compact ? `${name} · 이후 새 세션이 요약을 이어 받습니다` : `${name} · 이후 새 세션(앞 대화 기억 없음)`;
  // 자동 왕복 — 이 PC 는 처리했고 상대 PC 쪽이 실패(라운드 1 — 부분 처리를 숨기지 않는다).
  if (m.refused?.startsWith("peer_")) return `${done} · 상대 PC 쪽은 실패(${contextWhy(m.refused.slice(5))}) · 다시 누르면 다시 요청`;
  return done;
}

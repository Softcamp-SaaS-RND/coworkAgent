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
  if (!p.my_ok) return p.their_ok ? "상대 승인함 — 내 승인 대기" : "내 승인 대기";
  return "상대 승인 대기";
}

/// 사이드바 · 머리글의 대화 이름. 제목이 없으면(메시지 없음) 방향 · 에이전트로. 방향은 화살표 대신 태그(`보냄` · `받음`).
export function convLabel(conv, peerNames) {
  const who = peerNames.get(conv.peer_id) ?? "알 수 없는 상대";
  const out = conv.dir === "out";
  const title = conv.title || (out ? `${conv.agent} 에게` : `내 ${conv.agent} 가 받은 요청`);
  const sub = out ? `${who} 의 ${conv.agent}` : `${who} 의 요청 · 내 ${conv.agent}`;
  return { title, sub, tag: out ? "보냄" : "받음" };
}

/// 이 PC 에서 부르는 상대 이름 — 표시 이름(별칭)이 있으면 그것.
export function displayName(p) {
  return p.alias || p.name;
}

/// 타일 글자 — 이름의 앞 두 글자(글자 묶음 단위 — 이모지 · 결합 문자를 반으로 자르지 않는다).
export function tileText(name) {
  const graphemes = new Intl.Segmenter("ko", { granularity: "grapheme" }).segment(name ?? "");
  return Array.from(graphemes, (g) => g.segment)
    .slice(0, 2)
    .join("");
}

/// 사이드바 대화 목록 — 레일에서 고른 타일의 대화만, `q` 가 있으면 제목으로만 거른다(대소문자 무시).
export function convRows(convs, peerNames, railPeer, q = "") {
  const needle = q.trim().toLowerCase();
  return filterByPeer(convs, railPeer)
    .map((c) => ({ c, label: convLabel(c, peerNames) }))
    .filter(({ label }) => !needle || label.title.toLowerCase().includes(needle));
}

/// 타일을 눌러도 열린 대화를 그대로 둘까 — 대화 화면(대화 · 보낸 메시지)이고 그 대화가 그 타일 것일 때만.
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
  if (m.author === "agent") {
    const agent = m.agent ?? conv.agent;
    if (conv.dir === "out") return m.mine === true ? `내 ${conv.relay_agent ?? "에이전트"}` : `${peer}/${agent}`;
    return m.mine === false ? `${peer}/${conv.relay_agent ?? "에이전트"}` : `내 ${agent}`;
  }
  if (m.via === "cli") return m.mine ? `내 ${CLI_LABEL}` : `${peer}/${CLI_LABEL}`;
  return m.mine ? "나" : peer;
}

/// "이름 · PC" 의 이름 — 마지막 구분 앞까지(표시 이름에 " · " 가 있어도 자르지 않는다). 모르는 상대 = "상대".
export function shortName(full) {
  const s = full ?? "상대";
  const cut = s.lastIndexOf(" · ");
  return cut < 0 ? s : s.slice(0, cut);
}

/// CLI 로 쓴 글(`via = "cli"`)의 발화자 — 사용자의 AI 에이전트.
const CLI_LABEL = "AI 에이전트(CLI)";

/// 글 종류 칸 — 에이전트 답 · 실행 요청(→ 에이전트) · 실행 없는 글, CLI 글은 사람 대신 AI 에이전트(CLI).
export function kindText(m) {
  if (m.author === "agent") return "에이전트";
  const who = m.via === "cli" ? CLI_LABEL : "사람";
  return m.exec ? `${who} → 에이전트` : who;
}

/// 대화 머리글의 자동 왕복 표시(D24)와 누를 수 있는 동작 — 보낸 쪽: 멈춤 · 재개, 받은 쪽: 멈춤만(재개는 연 쪽). 아니면 `null`.
export function relayText(conv, messages) {
  if (conv.dir === "out") {
    if (!conv.relay_agent) return null;
    const n = `자동 왕복 ${conv.relay_count ?? 0} / ${conv.relay_cap}`;
    return conv.relay_on ? { text: `${n} — 진행 중`, action: "stop" } : { text: `${n} — 멈춤`, action: "resume" };
  }
  if (conv.relay_blocked) return { text: "자동 왕복 거절 중 — 연 쪽이 재개하면 풀린다", action: null };
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
        { kind: "request", title: h.via === "cli" ? `요청 — ${CLI_LABEL}` : "요청", at: h.created_at, text: h.text, state: h.state },
        reply
          ? { kind: "run", title: "실행", meta: runMeta(reply), state: reply.state }
          : { kind: "run", title: "실행", meta: stateText(h.state), state: h.state },
        reply
          ? {
              kind: "reply",
              title: reply.state === "done" ? "응답" : `응답 — ${stateText(reply.state)}`,
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
/// `#activity` 내 활동 · `#saved` 저장된 항목 · `#agents` 에이전트 · `#settings`(`/{절}` = 그 절, 모르는 절은 처음) · `#me`(설정의 내 정보) ·
/// `#new` 새 대화. 그 밖은 안내 화면.
export function route(hash) {
  const h = (hash ?? "").replace(/^#/, "");
  let m = /^c\/(\d+)(?:\/m\/(\d+))?$/.exec(h);
  if (m) return m[2] ? { view: "conversation", id: Number(m[1]), msg: Number(m[2]) } : { view: "conversation", id: Number(m[1]) };
  m = /^s\/(\d+)$/.exec(h);
  if (m) return { view: "sent", id: Number(m[1]) };
  m = /^p\/(\d+)$/.exec(h);
  if (m) return { view: "peer", id: Number(m[1]) };
  if (["activity", "saved", "agents", "new"].includes(h)) return { view: h };
  m = /^settings(?:\/(\w+))?$/.exec(h);
  if (m) {
    const known = settingsTree().some((n) => n.id === m[1] || (n.children ?? []).some((c) => c.id === m[1]));
    return { view: "settings", focus: known ? m[1] : null };
  }
  if (h === "me") return { view: "settings", focus: "me" };
  return { view: "home" };
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

// ---- 설정 트리 ----

/// 설정 화면 왼쪽 트리 — 항목 `id` = 그 절의 요소 id(누르면 그 절로).
export function settingsTree() {
  return [
    { id: "me", label: "내 정보" },
    {
      id: "conn",
      label: "연결",
      children: [
        { id: "invite_make", label: "초대 생성" },
        { id: "invite_accept", label: "초대 수락" },
        { id: "peers", label: "연결 목록" },
      ],
    },
    { id: "agents", label: "내 에이전트" },
    { id: "run", label: "실행 — 모델 · effort" },
    { id: "screen", label: "화면 — 테마" },
    { id: "notify", label: "알림" },
  ];
}

// ---- 그룹 · 거르기 · 검색 ----

/// 그룹 이름이 없는 항목의 묶음 이름.
export const NO_GROUP = "그룹 없음";

/// 그룹 필터(`""` = 전체) · 검색(`fields` 값에 부분 일치, 대소문자 무시) 뒤 그룹별 묶음 — 그룹 이름순, 그룹 없음은 끝.
export function groupFilter(items, { group = "", q = "", fields = ["name"] } = {}) {
  const needle = q.trim().toLowerCase();
  const kept = items.filter(
    (it) =>
      (!group || (it.group ?? NO_GROUP) === group) &&
      (!needle || fields.some((f) => String(it[f] ?? "").toLowerCase().includes(needle))),
  );
  const groups = new Map();
  for (const it of kept) {
    const g = it.group ?? NO_GROUP;
    if (!groups.has(g)) groups.set(g, []);
    groups.get(g).push(it);
  }
  return [...groups.entries()]
    .sort(([a], [b]) => (a === NO_GROUP) - (b === NO_GROUP) || a.localeCompare(b, "ko"))
    .map(([name, list]) => ({ group: name, items: list }));
}

/// 그룹 필터 선택지 — 쓰인 그룹 이름(정렬), 그룹 없는 항목이 있으면 끝에 [`NO_GROUP`].
export function groupNames(items) {
  const names = [...new Set(items.map((it) => it.group).filter(Boolean))].sort((a, b) => a.localeCompare(b, "ko"));
  return items.some((it) => !it.group) ? [...names, NO_GROUP] : names;
}

// ---- 주고받은 글 · 보낸 글 ----

/// `GET /api/messages` 항목 → 목록 행. `kind` = `""`(전체) · `human`(사람 글) · `agent`(에이전트 답). 누르면 그 대화로.
export function inboxRows(messages, peerNames, { kind = "" } = {}) {
  return messages
    .filter((m) => m.conversation && (!kind || m.author === kind))
    .map((m) => {
      const conv = m.conversation;
      return {
        id: m.id,
        who: authorLabel(conv, m, peerNames.get(conv.peer_id)),
        kind: [kindText(m), askTag(m)].filter(Boolean).join(" · "),
        tag: conv.dir === "out" ? "보냄" : "받음",
        preview: preview(m.text),
        at: m.created_at,
        href: `#c/${conv.id}`,
      };
    });
}

/// 레일 상대 = 그 상대와 주고받은 글 — 상대 쪽(`theirs`) · 내 쪽(`mine`) 두 목록을 최신순(같은 시각이면 id 큰 것 먼저)으로 합쳐 `limit` 건.
export function peerMessages(theirs, mine, limit = 200) {
  return [...theirs, ...mine].sort((a, b) => b.created_at - a.created_at || b.id - a.id).slice(0, limit);
}

/// 목록에 보일 본문 앞부분 — 줄바꿈은 공백으로, 120자까지.
export function preview(text) {
  const one = (text ?? "").replace(/\s+/g, " ").trim();
  return one.length > 120 ? one.slice(0, 120) + "…" : one;
}

// ---- 작업 화면 — 상단 바 · 사이드바 · 내 활동 · 대화 · 실행 상세 · 통합 검색 ----

/// 사이드바 내비 — 고른 타일 범위(`tile` = 상대 id, `null` = '나' = 전부). 내 활동은 안 본 수(`count`), 주고받은 글은 상대 타일만.
export function sideNav(tile, unseen = 0) {
  const nav = [
    { label: "대화", href: "#" },
    { label: "내 활동", href: "#activity", count: unseen },
    { label: "저장된 항목", href: "#saved" },
    { label: "에이전트", href: "#agents" },
  ];
  if (tile != null) nav.push({ label: "주고받은 글", href: `#p/${tile}` });
  return nav;
}

/// 상단 바 "+ 새로 만들기" 메뉴 — 각 화면과 그 화면에서 초점을 줄 칸.
export function newMenu() {
  return [
    { label: "새 대화", href: "#new", focus: "chat_peer" },
    { label: "자동 왕복 대화", href: "#new", focus: "chat_relay_agent" },
    { label: "초대 생성", href: "#settings/invite_make", focus: "window_btn" },
    { label: "초대 수락", href: "#settings/invite_accept", focus: "invite_text" },
    { label: "에이전트 등록", href: "#settings/agents", focus: "agent_name" },
  ];
}

/// 상단 바 프로필 메뉴 — 표시 이름 · 아이콘('나' 타일 편집) · 집중(켜짐 표시) · 알림 일시 중지(켜짐 표시) · 환경설정.
export function profileMenu(paused, focus) {
  return [
    { key: "tile", label: "표시 이름 · 아이콘" },
    { key: "focus", label: "집중 — 요청 받지 않음", checked: focus === true },
    { key: "pause", label: "알림 일시 중지", checked: paused === true },
    { key: "settings", label: "환경설정", href: "#settings" },
  ];
}

/// 상대 접속 상태(`GET /api/peers` 의 `online` · `focus`) — 레일 점 · 사이드바 머리 · 대화 머리글. 오프라인이면 집중은 보이지 않는다.
export function presence(p) {
  if (!p?.online) return { key: "offline", text: "오프라인" };
  return p.focus ? { key: "focus", text: "집중 중" } : { key: "online", text: "온라인" };
}

/// 상대가 거절한 내 글의 사유(`refused`) — 실패 상태일 때만. 집중이면 "상대가 집중 중", 그 밖은 코드 그대로.
export function refusedText(m) {
  if (!m.refused || m.state !== "error") return "";
  return m.refused === "focus" ? "상대가 집중 중" : `상대가 거절(${m.refused})`;
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

/// 사람 글의 종류 태그 — 요청이면 "요청"(받은 쪽이 완료 표시하면 "요청 · 완료"), 질의 · 답은 없음.
export function askTag(m) {
  if (m.author !== "human" || !m.request) return "";
  return m.done ? "요청 · 완료" : "요청";
}

/// 사이드바 대화 섹션 — ★ 켠 대화는 '★ 즐겨찾기' 에만 보인다(대화 섹션에서 빠진다). 즐겨찾기가 없으면 그 섹션도 없다.
export function convSections(rows) {
  const starred = rows.filter((r) => r.c.starred);
  const rest = rows.filter((r) => !r.c.starred);
  return [...(starred.length ? [{ key: "starred", title: "★ 즐겨찾기", rows: starred }] : []), { key: "convs", title: "대화", rows: rest }];
}

/// 대화 행 표시 — 둘째 줄 `{에이전트} · {상태}`(받은 대화의 에이전트는 내 것), 실행 중 표시, 자동 왕복 칩 "왕복 {n}/{상한}", 안 읽음.
export function convMeta(c) {
  return {
    agent: c.dir === "out" ? c.agent : `내 ${c.agent}`,
    state: stateText(c.last_state),
    running: c.last_state === "running",
    relay: c.relay_agent && c.relay_cap != null ? `왕복 ${c.relay_count ?? 0}/${c.relay_cap}` : null,
    unread: c.unread > 0,
  };
}

/// 레일 배지 — 상대별 안 읽음 합(0 인 상대는 없다).
export function unreadByPeer(convs) {
  const sums = new Map();
  for (const c of convs) if (c.unread > 0) sums.set(c.peer_id, (sums.get(c.peer_id) ?? 0) + c.unread);
  return sums;
}

/// 내 활동 항목(`GET /api/activity`) → 목록 행. 라벨 = 글 종류 · 요청 태그(상대 쪽 글 — 실패한 답이면 "· 실패") · "실패"(내 쪽) · "자동 왕복 시작" ·
/// "자동 왕복 끝 — {사유}". 글은 누르면 그 글로(`#c/{대화}/m/{글}`), 자동 왕복 기록은 그 대화로. 대화를 모르는 항목은 뺀다.
/// `key` = 다시 그려도 같은 항목을 가리키는 열쇠.
export function activityRows(items, peerNames) {
  return items
    .filter((a) => a.conversation)
    .map((a) => {
      const conv = a.conversation;
      const peer = peerNames.get(conv.peer_id);
      const m = a.message;
      const row = {
        key: `${a.kind}:${conv.id}:${m?.id ?? `${a.at}:${a.detail ?? ""}`}`,
        at: a.at,
        unseen: a.unseen === true,
        tag: conv.dir === "out" ? "보냄" : "받음",
        title: convLabel(conv, peerNames).title,
      };
      if (m) {
        const kind = [kindText(m), askTag(m)].filter(Boolean).join(" · ");
        const label = a.kind === "failed" ? "실패" : m.state === "error" ? `${kind} · 실패` : kind;
        return { ...row, label, who: authorLabel(conv, m, peer), preview: preview(m.text), href: `#c/${conv.id}/m/${m.id}` };
      }
      const end = a.kind === "relay_end";
      return {
        ...row,
        label: end ? `자동 왕복 끝 — ${a.detail ?? ""}` : "자동 왕복 시작",
        who: shortName(peer),
        preview: end ? "" : preview(a.detail),
        href: `#c/${conv.id}`,
      };
    });
}

/// 메시지 작업 막대 — 모두 "저장"(저장됨이면 "저장 해제") · "복사", 내 실행 요청만 "다시 보내기"(같은 대화 · 같은 대상에 새 요청),
/// 받은 요청만 "완료 표시"(완료면 "완료 취소" — 쓴 쪽에 상태로 간다). 메시지별 편집 · 삭제 · 원문 버튼은 없다(원문 보기는 머리글 하나).
export function messageActions(m) {
  const acts = [
    { key: "save", label: m.saved ? "저장 해제" : "저장" },
    { key: "copy", label: "복사" },
  ];
  if (m.author === "human" && m.mine === true && m.exec !== false) acts.push({ key: "resend", label: "다시 보내기" });
  if (m.author === "human" && m.mine === false && m.request) acts.push({ key: "done", label: m.done ? "완료 취소" : "완료 표시" });
  return acts;
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

/// 입력창 Enter = 보내기, Shift+Enter = 줄바꿈. 한글 조합 중(`isComposing` · keyCode 229)의 Enter 는 조합을 끝내는 키라 보내지 않는다.
/// 누르고 있는 Enter(키 반복)도 보내지 않는다 — 같은 글이 여러 번 가지 않게.
export function enterSends(ev) {
  return ev.key === "Enter" && !ev.shiftKey && !ev.repeat && !ev.isComposing && ev.keyCode !== 229;
}

/// 실행 상세 '진행' — 고른 요청(`timeline()` 항목)의 단계: 요청 → 대기열 → 실행 → 응답, 단계 상태 = done · now · wait · fail. 대기 시작 · 끝 시각은
/// 남지 않아 시각은 요청 · 응답 때만(HH:MM:SS). 보낸 쪽은 상대의 대기 · 실행을 모르므로(상태 프레임 없음) 답이 오기 전까지 대기열에 둔다.
/// 실행 줄 = 답의 `{model} · {effort} · {n}턴 · {시간}`(없는 값은 뺀다). 상대가 보낸 요청이면 "요청 받음" · "응답 보냄".
export function runSteps({ request, reply }) {
  const mine = request.mine !== false;
  const failed = reply ? reply.state === "error" : request.state === "error";
  const done = reply != null && !failed;
  const running = !reply && request.state === "running";
  const meta = reply
    ? [reply.model, reply.effort, reply.turns != null ? `${reply.turns}턴` : "", formatDuration(reply.duration_ms)].filter(Boolean).join(" · ")
    : "";
  return [
    { key: "request", title: mine ? "요청 보냄" : "요청 받음", at: formatClock(request.created_at), state: "done" },
    { key: "queue", title: "대기열", at: "", state: reply || running || failed ? "done" : "now" },
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
/// 선택 — 페어링된 사람) · `chat`(새 대화에 상대 · 에이전트 채움 — 상대 에이전트).
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

/// 내 에이전트 도구 — D31 읽기 전용 목록(an_core `READ_ONLY_TOOLS` 와 같다).
export const READ_ONLY_TOOLS = ["Read", "Grep", "Glob", "LS", "NotebookRead"];
/// `claude --effort` 값(an_core `run_opts::EFFORTS` 와 같다).
export const EFFORTS = ["low", "medium", "high", "xhigh", "max"];

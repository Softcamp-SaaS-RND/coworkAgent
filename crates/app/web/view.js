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

/// 사이드바 · 머리글의 대화 이름. 제목이 없으면(메시지 없음) 방향 · 에이전트로.
export function convLabel(conv, peerNames) {
  const who = peerNames.get(conv.peer_id) ?? "알 수 없는 상대";
  const arrow = conv.dir === "out" ? "→" : "←";
  const title = conv.title || (conv.dir === "out" ? `${conv.agent} 에게` : `내 ${conv.agent} 가 받은 요청`);
  const sub = conv.dir === "out" ? `${who} 의 ${conv.agent}` : `${who} → 내 ${conv.agent}`;
  return { title: `${arrow} ${title}`, sub };
}

/// 레일에서 고른 상대의 대화만(`null` = 내 PC = 전부).
export function filterByPeer(convs, peerId) {
  return peerId == null ? convs : convs.filter((c) => c.peer_id === peerId);
}

/// 발화자 이름(D23) — 사람: 내 글 "나", 상대 글은 상대 이름. 에이전트: 대화 에이전트의 주인 쪽(보낸 대화 = 상대 PC, 받은 대화 = 내 PC).
/// 자동 왕복(D24)이면 반대쪽 에이전트 글도 있다 — 보낸 대화의 내 글 = 내 자동 왕복 에이전트, 받은 대화의 상대 글 = 상대 에이전트.
export function authorLabel(conv, m, peerName) {
  const peer = (peerName ?? "상대").split(" · ")[0];
  if (m.author === "agent") {
    if (conv.dir === "out") return m.mine === true ? `내 ${conv.relay_agent ?? "에이전트"}` : `${peer}/${conv.agent}`;
    return m.mine === false ? `${peer}/${conv.relay_agent ?? "에이전트"}` : `내 ${conv.agent}`;
  }
  return m.mine ? "나" : peer;
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

/// C 타임라인 — 사람 메시지(요청)마다 요청 → 실행 → 응답 단계. 연결 못 한 답은 `unlinkedReplies` 로 따로.
export function timeline(messages) {
  const { linked } = linkReplies(messages);
  return messages
    .filter((m) => m.author === "human" && m.exec !== false)
    .map((h) => {
      const reply = linked.get(h.id) ?? null;
      const steps = [
        { kind: "request", title: "요청", at: h.created_at, text: h.text, state: h.state },
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

/// 화면 위치(해시) — `#c/{id}` 대화 · `#t/{id}` 타임라인 · `#settings` · `#me`(설정의 내 정보) · `#new` 새 대화.
export function route(hash) {
  const h = (hash ?? "").replace(/^#/, "");
  let m = /^c\/(\d+)$/.exec(h);
  if (m) return { view: "conversation", id: Number(m[1]) };
  m = /^t\/(\d+)$/.exec(h);
  if (m) return { view: "timeline", id: Number(m[1]) };
  if (h === "settings") return { view: "settings", focus: null };
  if (h === "me") return { view: "settings", focus: "me" };
  if (h === "new") return { view: "new" };
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

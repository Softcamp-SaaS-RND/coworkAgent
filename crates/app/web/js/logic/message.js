// 글 — 말풍선 쪽 · 작성자 · 시간 줄 · 진행 · 멘션 · 메모 · 실행 단계.
import { CLI_LABEL } from "./author.js";
import { formatClock, formatDuration, stateText } from "./format.js";
import { reqMoves, reqResend } from "./request.js";

/// 대화 머리글의 자동 왕복 표시와 누를 수 있는 동작 — 보낸 쪽: 멈춤 · 재개, 받은 쪽: 멈춤만(재개는 연 쪽). 아니면 `null`.
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

export function runMeta(reply) {
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

/// 상대가 거절한 내 글의 사유(`refused`) — 요청자에게 다시 알릴 답(`reqResend`)이면 상태와 무관하게, 그 밖은 실패 상태일 때만. 집중이면 "상대가 집중 중", 그 밖은 코드 그대로.
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

/// 실행 중 단계 요약 — `{turns, tools: {이름: 횟수}, secs}` → "2턴 · Read 3 · Grep 1 · 41초"(없는 값은 뺀다).
export function stepText(d) {
  if (!d) return "";
  const tools = Object.entries(d.tools ?? {}).map(([name, n]) => `${name} ${n}`);
  return [d.turns ? `${d.turns}턴` : "", ...tools, d.secs ? `${d.secs}초` : ""].filter(Boolean).join(" · ");
}

/// 보관 중인 내 글(상대에 아직 닿지 않음). 포기한 글은 거절 사유(`refusedText`)가 말한다.
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

/// 글의 멘션 대상 — 첫 `@이름`(앞이 글 머리 · 공백, 뒤가 글 끝 · 공백 · 문장 부호)이 `names` 의 한 이름과 글자 그대로 같은 것. 긴 이름
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

/// 메시지 작업 막대(말풍선 아래 작은 단추) — 받은 요청만 상태 바꾸기(`req:{상태}` — [`reqMoves`], 쓴 쪽에 간다).
/// 북마크 · 복사는 말풍선 우상단 아이콘, 원문 보기는 머리글 하나 — 막대에는 없다.
export function messageActions(m, closed = false) {
  const acts = [];
  for (const r of reqMoves(m, closed)) acts.push({ key: `req:${r.to}`, label: r.label });
  if (reqResend(m, closed)) acts.push({ key: "req-again", label: "다시 알리기" });
  return acts;
}

/// 편집 표시 — 편집한 글이면 "(편집됨)".
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

/// 발화자 종류 아이콘(사람 ↔ 에이전트 구분) — 대화 목록 미리보기 · 메시지 줄 앞. `icon` = `app.css` `.ico-{이름}`.
export function speakerIcon(author) {
  return author === "agent" ? { kind: "agent", icon: "bot", label: "에이전트" } : { kind: "human", icon: "user", label: "사람" };
}

/// 주고받은 메시지 줄 아바타 — 에이전트(로봇) · 나(내 사람 글) · 상대(상대 사람 글) — 사람은 머리글자, 메타의 "사람" 은 `speakerIcon` 과 같다.
export function avatarKind(author, mine) {
  return author === "agent" ? "agent" : mine ? "me" : "peer";
}

/// 요청 의견 글자 수 — 서버(`MAX_REQ_NOTE`)처럼 문자 수로 센다.
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

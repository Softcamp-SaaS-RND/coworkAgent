// 주고받은 메시지.
import { api } from "./core/api.js";
import { $, act, button, el, ico, needsRender } from "./core/dom.js";
import { latest, state } from "./core/state.js";
import { iconButton, tileFace } from "./core/widgets.js";
import { formatTime } from "./logic/format.js";
import { PEER_PAGE, inboxRows, mergePage, pageMore, peerMsgsSig, peerSummaryRows } from "./logic/inbox.js";
import { avatarKind, speakerIcon } from "./logic/message.js";
import { initialOf, tileTint } from "./logic/peer.js";
import { BOOKMARK_SVG } from "./conv.js";
import { foldEl, pairedIds } from "./sidebar.js";

/// 주고받은 메시지('나' 에만) — 상대별 절(레일 순서) 처음 접힘, 머리 = 타일 · 이름 · 건수 · 마지막 메시지 한 줄 · 시각. 펼치면 그 상대 글을
/// 20건씩(서버 페이지), 끝 줄 "더 보기" 로 이어 받는다. 다시 그릴 때(5초 새로 고침)는 요약을 받고, 펼친 절은 보인 수만큼 다시 받는다.
export async function loadPeerMessages() {
  $("peer_title").textContent = "주고받은 메시지";
  const ticket = latest.next();
  let sums;
  try {
    // 기간은 다른 창에서 바뀌었을 수 있다 — 서버가 거른 기준(설정)을 함께 다시 읽어 받아 둔 꼬리에도 같은 기간을.
    let s;
    [sums, s] = await Promise.all([api("GET", "/api/messages/peers"), api("GET", "/api/settings")]);
    if (s.ui_peer_weeks !== undefined && s.ui_peer_weeks !== state.settings.ui_peer_weeks) {
      state.settings.ui_peer_weeks = s.ui_peer_weeks;
      $("ui_peer_weeks").value = String(s.ui_peer_weeks);
    }
  } catch (e) {
    $("status_msg").textContent = "주고받은 메시지 오류: " + e.message;
    return;
  }
  if (!latest.is(ticket)) return;
  state.peerHeads = peerSummaryRows(sums, pairedIds(), state.peerNames);
  for (const h of state.peerHeads) if (state.folded.has(`open:peer:${h.peer}`)) await loadPeerPage(h.peer);
  renderPeerMessages();
}

/// 한 상대의 글 — `more` 면 받은 마지막 글(`(created_at, id)` 커서) 앞의 20건을 이어 붙이고, 아니면 첫 페이지를 다시 받아 받은 꼬리와 합친다(`mergePage`).
/// 상대마다 한 줄로 차례를 지킨다(새로 고침과 "더 보기" 가 겹쳐 늦게 끝난 쪽이 다른 쪽 결과를 지우지 않게).
export function loadPeerPage(peer, more = false) {
  const run = async () => {
    const have = state.peerPages.get(peer) ?? [];
    const end = have.at(-1);
    const q = more && end ? `&limit=${PEER_PAGE}&before=${end.id}&before_at=${end.created_at}` : `&limit=${PEER_PAGE}`;
    try {
      const ms = await api("GET", `/api/messages?side=both&peer=${peer}${q}`);
      const now = state.peerPages.get(peer) ?? [];
      state.peerPages.set(peer, more ? [...now, ...ms.filter((m) => !now.some((x) => x.id === m.id))] : mergePage(ms, now, PEER_PAGE, peerSince()));
    } catch (e) {
      $("status_msg").textContent = "주고받은 메시지 오류: " + e.message;
    }
  };
  const next = (state.peerLoads.get(peer) ?? Promise.resolve()).then(run);
  state.peerLoads.set(peer, next);
  return next;
}

/// 주고받은 메시지 보기 기간의 시작(서버 `peer_since` 와 같은 셈, 유닉스 초).
export function peerSince() {
  return Math.floor(Date.now() / 1000) - (Number(state.settings.ui_peer_weeks) || 4) * 7 * 86400;
}

export function renderPeerMessages() {
  const heads = state.peerHeads;
  if (!heads.length) {
    $("peer_list").replaceChildren(el("li", { class: "hint", text: "주고받은 메시지 없음" }));
    return;
  }
  if (!needsRender($("peer_list"), JSON.stringify([peerMsgsSig(heads, state.peerPages), state.peerNames.size, state.peers.map((p) => [p.id, p.icon, p.color]), state.settings.me_color]), false)) return;
  $("peer_list").replaceChildren(
    ...heads.map((h) => {
      const p = state.peers.find((x) => x.id === h.peer);
      const head = [
        el("span", { class: "row-tile", "aria-hidden": "true" }, tileFace(p?.icon, h.name, p?.color)),
        el("strong", { text: h.name }),
        el("span", { class: "stat", text: h.count }),
        el("span", { class: "peer-last", text: h.last }),
        el("span", { class: "hint", text: formatTime(h.at) }),
      ];
      const page = state.peerPages.get(h.peer);
      const body = page ? [el("ol", { class: "rows" }, ...peerRowEls(inboxRows(page, state.peerNames), p?.color)), moreRow(h, page.length)] : [el("p", { class: "hint more-row", text: "불러오는 중 …" })];
      const d = foldEl(`peer:${h.peer}`, head, body, "peer-fold peer-msgs", null, true);
      d.addEventListener("toggle", async () => {
        if (!d.open || state.peerPages.has(h.peer)) return;
        await loadPeerPage(h.peer);
        renderPeerMessages();
      });
      return el("li", { class: "fold" }, d);
    }),
  );
}

/// 펼친 절 끝 줄 — "{보인 수} / {전체}" + 남았으면 "더 보기"(20건).
export function moreRow(h, shown) {
  const m = pageMore(shown, h.total);
  const more = async () => {
    await loadPeerPage(h.peer, true);
    renderPeerMessages();
  };
  return el("div", { class: "more-row" }, el("span", { text: m.text }), m.more ? button("더 보기", more, "small") : "");
}

/// 주고받은 메시지 행 — 아바타(상대 사람 · 나 = 머리글자 네모, 에이전트 = 로봇 — `avatarKind`) · 메타(이름 · "사람" / 에이전트 이름 + "에이전트" 칩 · 대화 제목 ·
/// 요청 태그) · 본문 · 오른쪽 시각, 에이전트 줄은 옅은 에이전트 색 바탕. 사람 아바타 = 타일 색(나 = `me_color` · 상대 = `peerColor`). 오른쪽 끝 북마크 아이콘(말풍선과 같은 그림 · 동작).
export function peerRowEls(rows, peerColor) {
  if (!rows.length) return [el("li", { class: "hint", text: "주고받은 메시지 없음" })];
  return rows.map((r) => {
    const kind = avatarKind(r.author, r.mine);
    const s = speakerIcon(r.author);
    const li = el(
      "li",
      { class: `has-mark pm-row pm-${kind}` },
      el(
        "a",
        { href: r.href, class: "pm-link" },
        el("span", { class: `pm-av pm-av-${kind} ${kind === "agent" ? "" : tileTint(kind === "me" ? state.settings.me_color : peerColor)}`.trim(), role: "img", "aria-label": s.label, title: s.label }, kind === "agent" ? ico(s.icon) : initialOf(r.who)),
        el(
          "span",
          { class: "pm-text" },
          el(
            "span",
            { class: "meta" },
            el("strong", { class: kind === "agent" ? "pm-agent-name" : null, text: r.who }),
            kind === "agent" ? el("span", { class: "tag agent-tag", text: "에이전트" }) : el("span", { text: "사람" }),
            r.room ? el("span", { class: "hint", text: r.room }) : "",
            r.kind ? el("span", { class: "tag ask", text: r.kind }) : "",
          ),
          el("span", { class: "pm-body", text: r.preview }),
        ),
        el("span", { class: "pm-time", text: formatTime(r.at) }),
      ),
    );
    const b = iconButton(BOOKMARK_SVG, r.saved ? "북마크 해제" : "북마크", () => markRow(r));
    b.classList.add("bookmark", "row-mark");
    if (r.saved) b.classList.add("on");
    b.setAttribute("aria-pressed", String(r.saved));
    li.append(b);
    return li;
  });
}

/// 행 북마크 넣기 · 빼기 — 말풍선 북마크와 같은 `PATCH saved`, 받은 글 사본 · 열린 대화 글도 같이 바꾼다.
export function markRow(r) {
  const on = !r.saved;
  act(on ? "북마크" : "북마크 해제", async () => {
    await api("PATCH", `/api/messages/${r.id}`, { saved: on });
    for (const ms of state.peerPages.values()) for (const m of ms) if (m.id === r.id) m.saved = on;
    const open = state.msgs.find((m) => m.id === r.id);
    if (open) open.saved = on;
    renderPeerMessages();
    return on ? "북마크했습니다" : "북마크 해제";
  });
}

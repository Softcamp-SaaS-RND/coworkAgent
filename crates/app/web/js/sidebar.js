// 사이드바 — 머리 · 내비 · 대화 목록 · 즐겨찾기 · 에이전트 응답 · 상대별 절(접기) · 폭 · 개수.
import { api } from "./core/api.js";
import { $, act, el, ico, needsRender, unchanged } from "./core/dom.js";
import { state } from "./core/state.js";
import { agentTile, kindChip } from "./core/widgets.js";
import { agentConvSections, agentTag, convListParts, convMeta, convRows, convSections, starDetail, visibleConvRows } from "./logic/conv.js";
import { displayName, formatTime } from "./logic/format.js";
import { sortByPos } from "./logic/order.js";
import { peerGroups, waitingByPeer } from "./logic/peer.js";
import { inGroup, scopeQuery, splitKey } from "./logic/rail.js";
import { SESSION_NEW, route, sideNav, sideSub, starredSections } from "./logic/shell.js";
import { SendLatest } from "./logic/sync.js";
import { newChat } from "./compose.js";
import { openConvMenu, openGroupPicker } from "./conv-menu.js";
import { renderConvHead } from "./conv.js";
import { convGrab, keepFocus, keepOrder, orderStamp } from "./drag.js";
import { refreshHome } from "./home.js";
import { railGroupOf, renderRail, scopedConvs } from "./rail.js";
import { loadRequests } from "./requests.js";
import { renderPeerList } from "./settings/conn.js";

export function renderSideHead() {
  const p = state.peers.find((x) => x.id === state.railPeer);
  const g = railGroupOf();
  if (g) {
    $("side_name").textContent = g.name;
    $("side_sub").textContent = `그룹 · 대화 ${inGroup(state.convs, g).length} · 이 PC 에만`;
  } else if (p) {
    // 접속 상태는 레일 점 — 부제에 "페어링 됨" · 접속 문구를 쓰지 않는다.
    $("side_name").textContent = displayName(p);
    $("side_sub").textContent = sideSub("peer", p);
  } else if (state.me) {
    const alias = state.settings.me_alias;
    $("side_name").textContent = alias || "나 · " + state.me.host;
    $("side_sub").textContent = sideSub("me", state.me, alias);
  }
}

/// 사이드바 내비 — 고른 타일 범위(`logic/shell.js` `sideNav`), 지금 화면은 강조. '개요' 는 대화 화면에서도. 절 사이 구분선.
export function renderSideNav() {
  const view = route(location.hash).view;
  const scoped = scopedConvs();
  const starred = scoped.filter((c) => c.starred === true).length;
  // 에이전트 응답('나' 에만) = 그 범위 상대가 연 대화의 안 읽음 합 — 대화 목록에서 빠져 배지가 여기로 온다.
  const agentUnread = scoped.filter((c) => c.dir === "in").reduce((n, c) => n + (c.unread ?? 0), 0);
  if (unchanged($("side_nav"), JSON.stringify([view, state.railPeer, state.railGroup, state.unseen, state.reqWaiting, starred, agentUnread]))) return;
  $("side_nav").replaceChildren(
    ...sideNav(state.railGroup != null ? "group" : state.railPeer, state.unseen, state.reqWaiting, starred, agentUnread).map((n) => {
      if (n.sep) return el("li", { class: "nav-sep", role: "separator" });
      if (n.head) return el("li", { class: "nav-head" }, el("span", { class: "nav-label" }, ico(n.icon), el("span", { text: n.label })));
      const target = route(n.href).view;
      const current = target === view || (target === "home" && (view === "conversation" || view === "sent"));
      return el(
        "li",
        { class: n.sub ? "nav-sub" : null },
        el(
          "a",
          {
            href: n.href,
            class: current ? "current" : null,
            "aria-current": current ? "page" : null,
            // 요청 화면에서 메뉴 '요청' 을 다시 누르면 개요로(해시가 같아 화면 바꿈이 없다).
            onclick: target === "requests" && current ? () => loadRequests(true) : null,
          },
          el("span", { class: "nav-label" }, ico(n.icon), el("span", { text: n.label })),
          n.count ? el("span", { class: "count", text: String(n.count) }) : "",
        ),
      );
    }),
  );
}

export async function loadConvs() {
  const stamp = orderStamp("conv");
  try {
    state.convs = keepOrder("conv", await api("GET", "/api/conversations"), stamp);
  } catch (e) {
    $("status_msg").textContent = "대화 목록 오류: " + e.message;
    return;
  }
  if (state.current) state.current = state.convs.find((c) => c.id === state.current.id) ?? state.current;
  renderConvList();
  renderRail();
  refreshHome();
  renderSideNav(); // 즐겨찾기 수 · 에이전트 응답 안 읽음
  renderPeerList(); // 설정 연결 · 그룹설정의 대화 줄
  if (route(location.hash).view === "starred") renderStarred();
  if (route(location.hash).view === "agentConvs") renderAgentConvs();
}

export const STAR_SVG =
  '<svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" aria-hidden="true"><path d="M12 3l2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1-4.4-4.3 6.1-.9z"></path></svg>';

/// 즐겨찾기 ☆ 단추(대화 머리) — 빠짐 = 회색 테두리 · 넣음 = 노랑. 누르면 넣고 빼고 목록을 다시 읽는다.
export function starButton(c, label) {
  const on = c.starred === true;
  const b = el("button", {
    type: "button",
    class: on ? "star-btn on" : "star-btn",
    "aria-pressed": String(on),
    title: on ? "즐겨찾기 빼기" : "즐겨찾기",
    "aria-label": `${on ? "즐겨찾기 빼기" : "즐겨찾기"} · ${label}`,
    onclick: () => toggleStar(c),
  });
  b.innerHTML = STAR_SVG;
  return b;
}

export async function toggleStar(c) {
  const on = c.starred !== true;
  await act(on ? "즐겨찾기" : "즐겨찾기 빼기", async () => {
    await api("PATCH", `/api/conversations/${c.id}`, { starred: on });
    c.starred = on;
    if (state.current?.id === c.id) state.current.starred = on;
    await loadConvs();
    if (state.current?.id === c.id) renderConvHead(state.current);
    return on ? "즐겨찾기에 넣었습니다" : "즐겨찾기에서 뺐습니다";
  });
}

/// `opts.star` = 줄 머리 ★(즐겨찾기 빼기), `opts.detail` = 제목 줄 아래 둘째 줄, `opts.agent` = 유형 아이콘 대신 답한 내 에이전트 타일 · 시각 뒤 `[이름]`.
export function convItem({ c, label }, section, sideList = false, opts = {}) {
  const meta = convMeta(c);
  const cls = [state.current?.id === c.id ? "current" : "", meta.unread ? "unread" : ""].filter(Boolean).join(" ");
  return el(
    "li",
    { class: cls || null, "data-conv": String(c.id), "data-drag": "conv", "data-id": String(c.id), "data-section": section, "data-kind": c.dir },
    // 손잡이 끌기 — 즐겨찾기 ☆ 는 대화 머리에서. 줄 링크는 HTML 끌기를 막는다(브라우저 끌기 확장).
    el("button", {
      type: "button",
      class: "drag-handle",
      title: "끌어서 옮기기 · 그룹 타일에 놓으면 그룹에",
      "aria-label": `${label.title} 옮기기`,
      onpointerdown: (ev) => convGrab(ev, ev.currentTarget.parentElement),
    }, ico("grip")),
    opts.star ? starButton(c, label.title) : "",
    el(
      "a",
      { href: `#c/${c.id}`, title: label.sub, draggable: "false" },
      // 한 줄: 유형 아이콘(무채 — 응답 = 로봇 · 문의 = 사람 · 공유 = 눈) · 제목 · 실행 중 점 · 안 읽음 수 · 시각.
      // 사이드바 대화 목록(`sideList`)의 내 대화 줄은 아이콘 없음 — 그 목록의 기본이 내가 연 대화라 다른 절(에이전트 · 공유)만 표시한다.
      opts.agent ? agentTile(state.agents.find((a) => a.name === c.agent)?.color, "small") : sideList && c.dir === "out" ? "" : kindChip(c, "room-kind"),
      el(
        "span",
        { class: "room-text" },
        el(
          "span",
          { class: "title" },
          el("span", { class: "room-title", text: label.title }),
          meta.running ? el("span", { class: "spin", role: "img", "aria-label": meta.state, title: meta.state }) : "",
          meta.unread ? el("span", { class: "count unread-n", "aria-label": `안 읽음 ${c.unread}`, text: String(c.unread) }) : "",
          opts.agent
            ? el("span", { class: "room-end" }, c.last_at != null ? el("span", { class: "room-time", text: formatTime(c.last_at) }) : "", el("span", { class: "room-agent", text: agentTag(c) }))
            : c.last_at != null
              ? el("span", { class: "room-time", text: formatTime(c.last_at) })
              : "",
        ),
        opts.detail ? el("span", { class: "sub room-detail", text: opts.detail }) : "",
      ),
    ),
    // 행 ⋯ — 마우스를 올리거나 초점이 오면 보인다.
    el("button", {
      type: "button",
      class: "icon-btn row-more",
      title: "대화 메뉴",
      "aria-label": `대화 메뉴 · ${label.title}`,
      "aria-haspopup": "menu",
      onclick: (ev) => openConvMenu(c.id, ev.currentTarget),
    }, ico("more")),
  );
}

export function renderConvList() {
  const q = $("conv_search").value.trim();
  const group = railGroupOf();
  const rows = visibleConvRows(convRows(sortByPos(scopedConvs()), state.peerNames, null, q), group);
  const sections = convSections(rows, state.railPeer == null ? pairedIds() : null, state.peerNames);
  const sig = [q, state.railPeer, group && [group.id, group.convs, group.peers], state.groups.length, state.current?.id, rows, sections.map((s) => s.title)];
  if (unchanged($("conv_sections"), JSON.stringify(sig))) return;
  const empty = el("li", { class: "hint", text: q ? "맞는 대화 없음" : "대화 없음" });
  const add = (peer) =>
    el("a", {
      href: "#new",
      class: "side-add",
      title: SESSION_NEW,
      "aria-label": SESSION_NEW,
      text: "+",
      onclick:
        peer == null
          ? null
          : (ev) => {
              ev.preventDefault();
              newChat(peer);
            },
    });
  // 그룹 — 목록 머리에 "대화 추가"(검색 + 체크로 여러 대화를 한 번에).
  const head = group
    ? [
        el(
          "div",
          { class: "side-title group-head" },
          el("span", { text: `그룹 대화 · ${inGroup(state.convs, group).length}` }),
          el("button", { type: "button", id: "group_pick_btn", class: "small", text: "대화 추가", onclick: (ev) => openGroupPicker(ev.currentTarget) }),
        ),
      ]
    : [];
  keepFocus($("conv_sections"), () =>
    $("conv_sections").replaceChildren(
      ...head,
      ...sections.map((s) => {
        // 묶음 — 나 · 상대 타일 = 내 대화(머리 없음) · 공유 대화(머리), 에이전트 대화는 '나' 의 사이드바 '에이전트 응답' 화면. 그룹 타일은 묶음마다 머리.
        const parts = convListParts(s.rows, group);
        const list = (g) => el("ul", { class: "side-list" }, ...g.rows.map((r) => convItem(r, splitKey(s.key, g.dir), true)));
        const lists = parts.length
          ? parts.flatMap((g) =>
              g.head ? [el("div", { class: "dir-head" }, el("span", { text: g.title }), el("b", { text: String(g.rows.length) })), list(g)] : [list(g)],
            )
          : [el("ul", { class: "side-list" }, ...(rows.length ? [] : [empty]))];
        // '나' 의 상대별 절 — 접을 수 있고 접힘은 이 PC 에 남는다.
        if (s.peer != null) return foldEl(`convs:${s.peer}`, [el("span", { text: s.title }), add(s.peer)], lists, "side-section", "side-title");
        return el("section", { class: "side-section" }, el("h2", { class: "side-title" }, el("span", { text: s.title }), s.key === "convs" ? add(null) : ""), ...lists);
      }),
    ),
  );
}

/// 즐겨찾기 화면 — 고른 타일 범위의 ★ 대화방, 대화방 줄 그대로('나' · 그룹은 상대별 절). ☆ 를 끄면 다음 그리기에서 빠진다.
export function renderStarred() {
  const sections = starredSections(sortByPos(scopedConvs()), state.peerNames, state.railPeer == null ? pairedIds() : null);
  const sig = JSON.stringify([state.current?.id, sections.map((s) => [s.key, s.title, s.rows.map((r) => [r.c, r.label])])]);
  if (!needsRender($("starred_list"), sig, false)) return;
  $("starred_list").replaceChildren(
    ...(sections.length
      ? sections.map((s) =>
          el(
            "section",
            { class: "side-section" },
            el("h2", { class: "side-title" }, el("span", { text: s.title })),
            el("ul", { class: "side-list" }, ...s.rows.map((r) => convItem(r, `starred:${s.key}`, false, { star: true, detail: starDetail(r.c, state.peerNames.get(r.c.peer_id)) }))),
          ),
        )
      : [el("p", { class: "empty hint", text: "즐겨찾기 없음" })]),
  );
}

/// 에이전트 응답 화면('나' 에만) — 고른 타일 범위의 상대가 연 대화(내 에이전트가 답)를 요청한 상대별 절로, 대화 줄 그대로(끌어 옮기기는 같은 절 안).
export function renderAgentConvs() {
  const paired = state.railPeer == null ? pairedIds() : [state.railPeer];
  const sections = agentConvSections(sortByPos(scopedConvs()), paired, state.peerNames);
  const sig = JSON.stringify([state.current?.id, state.agents.map((a) => [a.name, a.color]), sections.map((s) => [s.key, s.title, s.rows.map((r) => [r.c, r.label])])]);
  if (!needsRender($("agent_convs_list"), sig, false)) return;
  $("agent_convs_list").replaceChildren(
    ...(sections.length
      ? sections.map((s) =>
          el(
            "section",
            { class: "side-section" },
            // 사용자 이름만 — 대화 수는 없다, 안 읽음은 줄마다.
            el("h2", { class: "side-title" }, el("span", { text: s.title })),
            el("ul", { class: "side-list" }, ...s.rows.map((r) => convItem(r, s.key, false, { agent: true }))),
          ),
        )
      : [el("p", { class: "empty hint", text: "에이전트 응답 없음" })]),
  );
}

/// 페어링된 상대 id — 레일 순서.
export function pairedIds() {
  return state.peers.filter((p) => p.paired).map((p) => p.id);
}

/// 접을 수 있는 절 — `<details>`(기본 펼침), 열쇠가 접힘 목록에 있으면 접혀 시작한다. 접거나 펴면 설정 `ui_folded` 에 남긴다.
export function foldEl(key, head, body, cls = "peer-fold", headCls = null, shut = false) {
  // 처음 접힘(`shut`) — 펼친 것을 `open:{열쇠}` 로 기억한다(없으면 접힘).
  const mark = shut ? `open:${key}` : key;
  const d = el("details", { class: cls, open: shut ? state.folded.has(mark) : !state.folded.has(mark), "data-fold": key }, el("summary", { class: headCls }, ...head), ...body);
  d.addEventListener("toggle", () => setFolded(mark, shut ? d.open : !d.open));
  return d;
}

/// 접힘 저장 — 한 번에 하나, 보내는 동안 바뀌면 마지막 목록만 이어서(`SendLatest`). 실패는 상태 줄에, 화면 접힘은 그대로.
export const foldSaver = new SendLatest((keys) =>
  api("PUT", "/api/settings", { ui_folded: keys }).catch((e) => {
    $("status_msg").textContent = "접힘 저장 실패: " + e.message;
  }),
);

/// 접힘 바꾸기 — 바뀐 것이 있을 때만 목록을 통째로 저장.
export function setFolded(key, folded) {
  if (state.folded.has(key) === folded) return;
  if (folded) state.folded.add(key);
  else state.folded.delete(key);
  foldSaver.push([...state.folded]);
}

/// 목록 행 → 요소. '나' 면 상대별 절(`{화면}:{상대}`, 제목 = 상대 이름 · 건수), 상대 타일이면 그대로. `toEls` 는 빈 목록이면 안내 행을 준다.
export function grouped(screen, rows, toEls) {
  if (state.railPeer != null || !rows.length) return toEls(rows);
  return peerGroups(rows, pairedIds(), state.peerNames).map((g) =>
    el("li", { class: "fold" }, foldEl(`${screen}:${g.peer}`, [`${g.name} · ${g.rows.length}`], [el("ol", { class: "rows" }, ...toEls(g.rows))])),
  );
}

$("conv_search").addEventListener("input", renderConvList);

/// 안 본 내 활동 수 — 사이드바 내비(고른 타일 범위) · 종 점(전부).
export async function loadCounts() {
  const peer = state.railPeer;
  const group = state.railGroup;
  const scope = scopeQuery(peer, group);
  let r;
  try {
    r = await api("GET", "/api/activity?limit=0" + (peer != null ? `&peer=${peer}` : ""));
  } catch {
    return; // 다음 새로 고침에 다시
  }
  let waiting = state.reqWaiting;
  let by = null;
  try {
    const w = await api("GET", `/api/messages?side=theirs&ask=request&req=wait&limit=500${scope ? `&${scope}` : ""}`);
    waiting = w.length;
    by = waitingByPeer(w);
  } catch {
    // 다음 새로 고침에 다시
  }
  if (state.railPeer !== peer || state.railGroup !== group) return; // 그 사이 다른 타일을 골랐다
  state.reqWaiting = waiting;
  if (by) {
    state.waitingBy = by;
    refreshHome();
  }
  state.unseen = peer != null ? r.unseen : r.unseen_all;
  state.unseenAll = r.unseen_all;
  $("bell_dot").hidden = state.unseenAll === 0;
  renderSideNav();
}

// 대화 목록 폭(끌기 · ← → 키) — 이 PC 설정에 저장한다. 웹UI 포트가 실행마다 바뀌어(출처가 달라져) 브라우저 저장소로는
// 다음 실행에 이어지지 않는다.

export const SIDE_W = { min: 200, max: 560, step: 16 };
export let sideSave;

export function setSideWidth(w) {
  const v = Math.min(SIDE_W.max, Math.max(SIDE_W.min, Math.round(w)));
  $("sidebar").style.setProperty("--side-w", v + "px");
  $("side_resize").setAttribute("aria-valuenow", String(v));
  return v;
}

/// 폭을 바꾸고 잠시 뒤 저장 — 키를 누르고 있어도 마지막 값 한 번.
export function saveSideWidth(w) {
  const v = setSideWidth(w);
  clearTimeout(sideSave);
  sideSave = setTimeout(() => {
    api("PUT", "/api/settings", { ui_side_width: v }).catch((e) => {
      $("status_msg").textContent = "목록 폭 저장 실패: " + e.message;
    });
  }, 300);
}

$("side_resize").addEventListener("pointerdown", (ev) => {
  ev.preventDefault(); // 끄는 동안 글자 선택 안 함
  const start = ev.clientX;
  const from = $("sidebar").getBoundingClientRect().width;
  const move = (e) => setSideWidth(from + e.clientX - start);
  window.addEventListener("pointermove", move);
  window.addEventListener(
    "pointerup",
    () => {
      window.removeEventListener("pointermove", move);
      saveSideWidth($("sidebar").getBoundingClientRect().width);
    },
    { once: true },
  );
});
$("side_resize").addEventListener("keydown", (ev) => {
  const d = { ArrowLeft: -SIDE_W.step, ArrowRight: SIDE_W.step }[ev.key];
  if (!d) return;
  ev.preventDefault();
  saveSideWidth($("sidebar").getBoundingClientRect().width + d);
});

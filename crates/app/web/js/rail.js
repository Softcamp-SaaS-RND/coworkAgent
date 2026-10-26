// 레일 — 상대 · 그룹 타일, 고르기, 그룹 만들기 · 이름 바꾸기, 레일 폭.
import { api } from "./core/api.js";
import { $, button, el, unchanged } from "./core/dom.js";
import { closeFloat, focusAfterFloat, openFloat } from "./core/float.js";
import { state } from "./core/state.js";
import { railFace, tileName } from "./core/widgets.js";
import { keepsConversation } from "./logic/conv.js";
import { displayName } from "./logic/format.js";
import { presence, tileUnread } from "./logic/peer.js";
import { RAIL_W, groupNameError, groupUnread, inGroup, railSize, tileKeeps } from "./logic/rail.js";
import { route } from "./logic/shell.js";
import { renderAgentsView } from "./agents.js";
import { fillPeerSelects } from "./compose.js";
import { renderConvHead } from "./conv.js";
import { keepFocus, keepOrder, orderStamp, railDragging, tileGrab } from "./drag.js";
import { refreshHome } from "./home.js";
import { renderPeerList } from "./settings/conn.js";
import { applyTileSize, go } from "./shell.js";
import { loadCounts, renderConvList, renderSideHead, renderSideNav } from "./sidebar.js";
import { closeTileEdit, tileMenuEvent } from "./tiles.js";

/// 레일 타일 순서 — 상대 묶음(상대 타일 · +)과 그룹 묶음(그룹 타일 · 그룹 +)의 앞뒤. DOM 을 옮겨 Tab 순서도 화면과 같다(CSS `order` 는 어긋남).
export function applyRailOrder(order) {
  const peers = [$("rail_pcs"), $("rail_add")];
  const groups = [$("rail_groups"), $("rail_group_add")];
  const [first, second] = order === "groups_first" ? [groups, peers] : [peers, groups];
  $("rail_me").after(...first, $("rail_sep"), ...second);
}

export async function loadPeers(force = false) {
  const stamp = orderStamp("peer");
  try {
    state.peers = keepOrder("peer", await api("GET", "/api/peers"), stamp);
  } catch (e) {
    $("peer_msg").textContent = "그룹설정 오류: " + e.message;
    return;
  }
  state.peerNames = new Map(state.peers.map((p) => [p.id, `${displayName(p)} · ${p.host}`]));
  renderRail();
  renderSideHead();
  renderPeerList(force);
  fillPeerSelects();
  // 표시 이름이 바뀌었을 수 있다 — 목록 · 열린 대화 머리글도 새 이름으로.
  renderConvList();
  refreshHome();
  if (state.current && route(location.hash).view === "conversation") renderConvHead(state.current);
}

export function renderRail() {
  if (railDragging) return;
  const paired = state.peers.filter((p) => p.paired);
  if (state.railPeer != null && !paired.some((p) => p.id === state.railPeer)) {
    state.railPeer = null;
    closeTileEdit(); // 그 상대에 묶인 편집
  }
  const unread = tileUnread(state.convs); // 상대 타일에서 보이는 대화만(에이전트 응답은 '나' 에만)
  if (state.railGroup != null && !state.groups.some((g) => g.id === state.railGroup)) state.railGroup = null;
  const sig = [
    state.railPeer,
    state.railGroup,
    state.settings.me_alias,
    state.settings.me_icon,
    state.settings.me_color,
    paired.map((p) => [p.id, displayName(p), p.host, p.icon, p.color, unread.get(p.id) ?? 0, presence(p).text]),
    state.groups.map((g) => [g.id, g.name, g.icon, g.color, groupUnread(state.convs, g), inGroup(state.convs, g).length]),
  ];
  if (unchanged($("rail"), JSON.stringify(sig))) return;
  const me = $("rail_me");
  const meName = state.settings.me_alias || "나";
  me.setAttribute("aria-pressed", String(state.railPeer == null && state.railGroup == null));
  me.setAttribute("aria-label", meName);
  me.title = meName;
  me.replaceChildren(railFace(state.settings.me_icon, false, state.settings.me_color, true), tileName(meName));
  keepFocus($("rail"), () => drawRailTiles(paired, unread));
}

/// 상대 · 그룹 타일(사람 · 세 사람 모양 · 그림 + 아래 이름).
export function drawRailTiles(paired, unread) {
  $("rail_pcs").replaceChildren(
    ...paired.map((p) => {
      const n = unread.get(p.id) ?? 0;
      const pr = presence(p);
      return el(
        "button",
        {
          type: "button",
          class: "rail-btn",
          "aria-label": `${displayName(p)} · ${p.host}${n ? ` · 안 읽음 ${n}` : ""} · ${pr.text}`,
          "aria-pressed": String(state.railPeer === p.id),
          "data-peer": String(p.id),
          "data-drag": "peer",
          "data-id": String(p.id),
          title: displayName(p),
          onpointerdown: tileGrab,
          onclick: () => pickTile(p.id),
          oncontextmenu: (ev) => tileMenuEvent(ev, { kind: "peer", id: p.id }),
        },
        railFace(p.icon, false, p.color),
        tileName(displayName(p)),
        n ? el("span", { class: "rail-badge", "aria-hidden": "true", text: n > 99 ? "99+" : String(n) }) : "",
        el("span", { class: `rail-dot ${pr.key}`, "aria-hidden": "true" }),
      );
    }),
  );
  // 그룹 타일 — 상대 타일 아래 구분선 뒤, 세 사람 모양 또는 그림, 배지 = 그룹 대화의 안 읽음 합.
  $("rail_sep").hidden = false;
  $("rail_groups").replaceChildren(
    ...state.groups.map((g) => {
      const n = groupUnread(state.convs, g);
      return el(
        "button",
        {
          type: "button",
          class: "rail-btn group",
          "data-group": String(g.id),
          "data-drag": "group",
          "data-id": String(g.id),
          title: `그룹 ${g.name}`,
          onpointerdown: tileGrab,
          "aria-label": `그룹 ${g.name} · 대화 ${inGroup(state.convs, g).length}${n ? ` · 안 읽음 ${n}` : ""}`,
          "aria-pressed": String(state.railGroup === g.id),
          onclick: () => pickGroup(g.id),
          oncontextmenu: (ev) => tileMenuEvent(ev, { kind: "group", id: g.id }),
        },
        railFace(g.icon, true, g.color),
        tileName(g.name),
        n ? el("span", { class: "rail-badge", "aria-hidden": "true", text: n > 99 ? "99+" : String(n) }) : "",
      );
    }),
  );
}

/// 레일 그룹 읽기 — 레일 · 사이드바 · 요약을 다시 그린다.
export async function loadGroups() {
  const stamp = orderStamp("group");
  try {
    state.groups = keepOrder("group", await api("GET", "/api/groups"), stamp);
  } catch (e) {
    $("status_msg").textContent = "그룹 목록 오류: " + e.message;
    return;
  }
  renderRail();
  renderSideHead();
  renderSideNav();
  renderConvList();
  refreshHome();
  renderPeerList(); // 설정 그룹설정 화면
}

/// 고른 그룹(없으면 `null`).
export function railGroupOf() {
  return state.groups.find((g) => g.id === state.railGroup) ?? null;
}

/// 고른 레일 범위의 대화 — 상대 = 그 상대, 그룹 = 그 그룹 대화, '나' = 전부.
export function scopedConvs() {
  const g = railGroupOf();
  if (g) return inGroup(state.convs, g);
  return state.railPeer == null ? state.convs : state.convs.filter((c) => c.peer_id === state.railPeer);
}

/// 그룹 타일 누름 — 열린 대화가 그 그룹 것이면 그대로, 아니면 메인 = 그룹 요약(`#`).
export function pickGroup(id) {
  const changed = state.railGroup !== id;
  if (changed) closeTileEdit();
  state.railPeer = null;
  state.railGroup = id;
  renderRail();
  renderSideHead();
  const g = railGroupOf();
  const keep = ["conversation", "sent"].includes(route(location.hash).view) && state.current && inGroup([state.current], g).length > 0;
  if (!keep) state.current = null;
  renderConvList();
  if (changed) state.unseen = 0;
  renderSideNav();
  if (changed) loadCounts();
  if (!keep) go("#");
}

/// 상대 연결 확인 — 끝나면 접속 상태를 다시 읽는다(실패는 조용히 — 표시가 "n분 전 통신" 으로 남는다).
export function pingPeers(ids) {
  if (!ids.length) return;
  Promise.allSettled(ids.map((id) => api("POST", `/api/peers/${id}/ping`))).then(() => loadPeers());
}

/// 레일 선택 — 다른 PC 를 고르면 열린 대화를 놓는다(입력창이 이전 상대를 가리킨 채 남지 않게). 화면 이동은 부른 쪽이.
export function pickRail(id) {
  const changed = state.railPeer !== id || state.railGroup != null;
  if (changed) closeTileEdit();
  state.railPeer = id;
  state.railGroup = null;
  renderRail();
  renderSideHead();
  if (state.current && !keepsConversation(state.current, id)) state.current = null;
  renderConvList();
  if (changed) state.unseen = 0; // 새 범위의 수는 곧 다시 읽는다
  renderSideNav();
  // 그 상대 연결 확인(주기 확인 없음, 60초 안에 통신했으면 서버가 보내지 않는다).
  if (changed && id != null) pingPeers([id]);
  if (changed) loadCounts();
}

/// 타일 누름 — 사이드바 = 그 타일의 대화 목록('나' = 상대별 절). 열린 대화가 그 타일 것이면 그대로, 아니면 메인 = 그 타일 개요('나' = 상대별
/// 요약, 상대 = 그 상대 하나의 요약). 상대 타일을 누르면 그 PC 의 에이전트 목록은 다음에 볼 때 다시 받는다.
export function pickTile(id) {
  if (id != null) state.peerAgents.delete(id);
  pickRail(id);
  if (!tileKeeps(route(location.hash).view, state.current, id)) go("#");
}

$("rail_me").addEventListener("click", () => pickTile(null));

/// 상대 PC 의 에이전트 이름 — 원격 왕복이라 에이전트 화면 · 멘션이 없을 때만 묻는다.
export async function loadPeerAgents(id) {
  state.peerAgents.set(id, undefined); // 묻는 중 — 같은 상대를 겹쳐 묻지 않는다
  try {
    state.peerAgents.set(id, (await api("GET", `/api/peers/${id}/agents`)).names);
  } catch {
    state.peerAgents.set(id, null);
  }
  if (route(location.hash).view === "agents") renderAgentsView();
}

/// "그룹 +" — 이름 한 칸 → 만들고 그 그룹을 고른다. `after` 가 있으면(그룹설정 "+ 그룹 추가") 고르지 않고 그것을
/// 부른다. 거절 문구(빈 이름 · 40자 넘음 · 같은 이름)는 상자 안에.
export function openGroupNew(anchor, after = null) {
  // 길이는 `groupNameError`(글자 = 코드 포인트, 서버와 같음)로 — HTML `maxlength` 는 UTF-16 단위라 그림 문자 이름을 20자 안팎에서 막는다.
  const input = el("input", { id: "group_new_name", autocomplete: "off", placeholder: "예: 결제 모듈", "aria-label": "그룹 이름" });
  const msg = el("p", { class: "hint", role: "status" });
  const form = el(
    "form",
    { class: "float-form" },
    el("strong", { text: "새 그룹" }),
    input,
    el("p", { class: "hint", text: "상대에게 보이지 않는 이 PC 의 대화 묶음입니다." }),
    el("div", { class: "actions" }, button("취소", () => closeFloat(true)), el("button", { type: "submit", class: "primary", text: "만들기" })),
    msg,
  );
  form.addEventListener("submit", async (ev) => {
    ev.preventDefault();
    const bad = groupNameError(input.value, state.groups);
    if (bad) {
      msg.textContent = bad;
      return;
    }
    try {
      const g = await api("POST", "/api/groups", { name: input.value });
      closeFloat(false);
      await loadGroups();
      if (after) {
        await after(g);
        anchor.isConnected ? anchor.focus() : focusAfterFloat(`#rail_groups [data-id="${g.id}"]`);
        return;
      }
      pickGroup(g.id);
      focusAfterFloat(`#rail_groups [data-id="${g.id}"]`);
      $("status_msg").textContent = `그룹 ${g.name} 을 만들었습니다`;
    } catch (e) {
      msg.textContent = e.message;
    }
  });
  openFloat(anchor, "dialog", "새 그룹", [form]);
}

$("rail_group_add").addEventListener("click", (ev) => openGroupNew(ev.currentTarget));

/// 그룹 이름 바꾸기(그룹설정 상세) — 이름 한 칸, 거절 문구는 상자 안에. 바꾸면 레일 · 그룹설정이 함께.
export function openGroupRename(id, anchor) {
  const g = state.groups.find((x) => x.id === id);
  if (!g) return;
  const input = el("input", { value: g.name, autocomplete: "off", "aria-label": "그룹 이름" });
  const msg = el("p", { class: "hint", role: "status" });
  const form = el(
    "form",
    { class: "float-form" },
    el("strong", { text: `그룹 ${g.name} 이름 바꾸기` }),
    input,
    el("div", { class: "actions" }, button("취소", () => closeFloat(true)), el("button", { type: "submit", class: "primary", text: "저장" })),
    msg,
  );
  form.addEventListener("submit", async (ev) => {
    ev.preventDefault();
    const bad = groupNameError(input.value, state.groups, id);
    if (bad) {
      msg.textContent = bad;
      return;
    }
    try {
      await api("PATCH", `/api/groups/${id}`, { name: input.value });
      closeFloat(false);
      await loadGroups();
      $("peer_msg").textContent = `그룹 이름을 바꿨습니다`;
      focusAfterFloat(`.gs-card[data-id="${id}"] .gs-card-head .icon-btn`, "peer_q");
    } catch (e) {
      msg.textContent = e.message;
    }
  });
  openFloat(anchor, "dialog", "그룹 이름 바꾸기", [form]);
}

/// 소속 바꾸기 — 성공이면 참. 어느 쪽이든 그룹을 다시 읽는다, 실패는 상태 줄에(부르는 쪽은 참일 때만 성공 문구).
export async function changeGroup(id, add, remove) {
  let ok = true;
  try {
    await api("PUT", `/api/groups/${id}/conversations`, { add, remove });
  } catch (e) {
    ok = false;
    $("status_msg").textContent = "그룹 바꾸기 실패: " + e.message;
  }
  await loadGroups();
  return ok;
}

// 레일 폭(손잡이 끌기 · ← → 키) — 끄는 동안 폭이 따라가고 120px 기준으로 아이콘이 바뀐다. 놓으면 크게(72px) · 작게(168px) 중
// 가까운 쪽으로 맞추고 설정 "타일 크기" 에 저장한다(설정 화면 선택도 같이).

export function saveTileSize(size) {
  applyTileSize(size);
  $("ui_tile_size").value = size;
  api("PUT", "/api/settings", { ui_tile_size: size }).catch((e) => {
    $("status_msg").textContent = "타일 크기 저장 실패: " + e.message;
  });
}

// 손잡이에 포인터 캡처 — 끝은 pointerup(저장) · pointercancel · lostpointercapture(되돌림) 어느 쪽이든 한 번만 정리한다(취소 뒤 폭이 계속 바뀌지 않게).
$("rail_resize").addEventListener("pointerdown", (ev) => {
  if (ev.button !== 0) return;
  ev.preventDefault(); // 끄는 동안 글자 선택 안 함
  const h = ev.currentTarget;
  h.setPointerCapture(ev.pointerId);
  const start = ev.clientX;
  const from = $("rail").getBoundingClientRect().width;
  const before = $("app").classList.contains("tile-small") ? "small" : "normal";
  let w = from;
  let done = false;
  const move = (e) => {
    w = Math.min(RAIL_W.max, Math.max(RAIL_W.min, from + e.clientX - start));
    $("app").style.setProperty("--rail-w", w + "px");
    applyTileSize(railSize(w));
  };
  const end = (e) => {
    if (done) return;
    done = true;
    for (const [type, fn] of handlers) h.removeEventListener(type, fn);
    $("app").style.removeProperty("--rail-w");
    if (e.type === "pointerup") saveTileSize(railSize(w));
    else applyTileSize(before);
  };
  const handlers = [
    ["pointermove", move],
    ["pointerup", end],
    ["pointercancel", end],
    ["lostpointercapture", end],
  ];
  for (const [type, fn] of handlers) h.addEventListener(type, fn);
});
$("rail_resize").addEventListener("keydown", (ev) => {
  const size = { ArrowLeft: "normal", ArrowRight: "small" }[ev.key];
  if (!size) return;
  ev.preventDefault();
  saveTileSize(size);
});

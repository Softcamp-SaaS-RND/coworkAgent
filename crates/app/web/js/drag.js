// 끌어 옮기기 — 대화 줄 · 레일 타일 · 순서 저장. 손으로 놓는 순서(끌어 놓기 · Alt + ↑ / ↓)는 같은 묶음(상대 타일 · 그룹 타일 · 같은 절의 대화) 안에서만.
import { api } from "./core/api.js";
import { $ } from "./core/dom.js";
import { state } from "./core/state.js";
import { displayName } from "./logic/format.js";
import { moveInOrder, movedText, orderToApply, sortByPos, withOrder } from "./logic/order.js";
import { dropKind, pastSlop, railSlot, stepTarget } from "./logic/rail.js";
import { route } from "./logic/shell.js";
import { SendLatest } from "./logic/sync.js";
import { refreshHome } from "./home.js";
import { changeGroup, loadGroups, loadPeers, renderRail } from "./rail.js";
import { loadConvs, renderAgentConvs, renderConvList } from "./sidebar.js";

/// 다시 그리기 전 초점이 그 안의 항목(`data-drag`)에 있었으면 다시 그린 같은 항목(같은 자리의 링크 · ⋯ · ☆)에 되돌린다 — 타일을 누르면 레일이
/// 다시 그려지는데, 되돌리지 않으면 초점이 body 로 떨어져 Alt + ↑ / ↓ 가 듣지 않는다.
export function keepFocus(box, draw) {
  const a = document.activeElement;
  const d = box.contains(a) ? dragInfo(a) : null;
  const sub = d && a !== d.node ? ["a", ".row-more", ".star-btn"].find((s) => a.matches(s)) : null;
  draw();
  if (!d || d.node.isConnected) return;
  const n = box.querySelector(dragSel(d));
  (sub ? n?.querySelector(sub) : n)?.focus();
}

/// 레일 타일을 끄는 중 — 그동안 `renderRail` 은 건너뛴다(서명을 적지 않으니 끝나고 부르면 그린다).
export let railDragging = false;

/// 요소 → 끌기 정보(`data-drag` · `data-id` · `data-section`).
export function dragInfo(node) {
  const n = node?.closest?.("[data-drag]");
  return n ? { node: n, kind: n.dataset.drag, id: Number(n.dataset.id), section: n.dataset.section ?? null } : null;
}

/// 전체 순서(id) — 상대 = 그룹설정 순서(서버 자리 순), 그룹 = 레일 순서, 대화 = 자리 순.
export function orderOf(kind) {
  if (kind === "peer") return state.peers.map((p) => p.id);
  if (kind === "group") return state.groups.map((g) => g.id);
  return sortByPos(state.convs).map((c) => c.id);
}

/// 종류별 순서 저장 — 한 번에 하나, 보내는 동안 바뀌면 마지막 순서만 이어서(`SendLatest`). `last` = 마지막으로 놓은 순서(저장이 끝나기 전 ·
/// 그 사이 시작한 목록 읽기에는 이것을 덮어 씌운다 — 늦게 온 옛 응답이 방금 옮긴 자리를 되돌리지 않게). `stamp` = 옮김 · 저장 끝마다 하나씩.
export const orderSaves = {};

export function orderSave(kind) {
  if (orderSaves[kind]) return orderSaves[kind];
  const o = { last: null, stamp: 0 };
  o.sender = new SendLatest(async (ids) => {
    try {
      await api("PUT", "/api/order", { kind: { peer: "peers", group: "groups", conv: "conversations" }[kind], ids });
    } catch (e) {
      $("status_msg").textContent = "자리 저장 실패: " + e.message;
      if (o.last === ids) o.last = null; // 서버 값으로 되돌린다
    }
    o.stamp++;
    if (kind === "peer") await loadPeers();
    else if (kind === "group") await loadGroups();
    else await loadConvs();
  });
  orderSaves[kind] = o;
  return o;
}

/// 목록 읽기를 시작할 때의 표 — 응답 때 `keepOrder` 에 준다.
export function orderStamp(kind) {
  return orderSaves[kind]?.stamp ?? 0;
}

/// 받은 목록 — 순서 저장이 진행 중이거나 읽는 사이 옮김 · 저장이 있었으면 마지막으로 놓은 순서를 덮어 씌운다.
export function keepOrder(kind, items, stamp) {
  const o = orderSaves[kind];
  const last = orderToApply(o && { last: o.last, busy: o.sender.busy, stamp: o.stamp }, stamp);
  if (!last) return items;
  const fixed = withOrder(items, last);
  return kind === "conv" ? fixed : sortByPos(fixed);
}

/// 옮기기 — 화면을 먼저 바꾸고(새 순서를 자리로) 저장한다. 실패하면 서버 값으로 되돌린다.
export function moveItem(kind, id, target, after) {
  const order = moveInOrder(orderOf(kind), id, target, after);
  if (kind === "peer") state.peers = sortByPos(withOrder(state.peers, order));
  else if (kind === "group") state.groups = sortByPos(withOrder(state.groups, order));
  else state.convs = withOrder(state.convs, order);
  renderRail();
  renderConvList();
  if (route(location.hash).view === "agentConvs") renderAgentConvs();
  refreshHome();
  const o = orderSave(kind);
  o.last = order;
  o.stamp++;
  o.sender.push(order);
}

/// 끌기 표시 지우기.
export function clearDrop() {
  for (const n of document.querySelectorAll(".drop-before, .drop-after, .drop-into")) n.classList.remove("drop-before", "drop-after", "drop-into");
}

/// 대화 줄을 그룹 타일에 넣는다(복사 — 원래 자리 그대로) · 안내.
export function joinGroup(groupId, convId) {
  const g = state.groups.find((x) => x.id === groupId);
  const already = g?.convs.includes(convId);
  changeGroup(groupId, [convId], []).then((ok) => {
    if (ok) $("status_msg").textContent = already ? `이미 그룹 ${g.name} 에 있습니다` : `그룹 ${g?.name ?? ""} 에 넣었습니다`;
  });
}

/// 대화 줄 끌기(포인터) — 손잡이를 누른 채 움직이면 줄 사본이 따라오고, 같은 절 줄 위 = 앞 · 뒤 표시, 그룹 타일 위 = 넣기 표시. 떼면 `moveItem` · `joinGroup`.
/// HTML 끌어 놓기(`draggable`)는 브라우저 끌기 확장이 가로챈다(요청 보드 `boardGrab` 과 같다).
export function convGrab(ev, li) {
  if (ev.button !== 0) return;
  ev.preventDefault();
  const grip = ev.currentTarget;
  grip.setPointerCapture(ev.pointerId);
  const d = dragInfo(li);
  const box = li.getBoundingClientRect();
  const dx = ev.clientX - box.left;
  const dy = ev.clientY - box.top;
  const ghost = li.cloneNode(true);
  ghost.classList.add("board-ghost");
  ghost.style.width = `${box.width}px`;
  document.body.append(ghost);
  li.classList.add("dragging");
  let drop = null;
  const move = (e) => {
    ghost.style.transform = `translate(${e.clientX - dx}px, ${e.clientY - dy}px)`;
    const under = document.elementFromPoint(e.clientX, e.clientY);
    const t = dragInfo(under);
    const how = t && t.node !== li ? dropKind(d, t) : null;
    clearDrop();
    drop = null;
    if (!how) return;
    const r = t.node.getBoundingClientRect();
    const after = e.clientY > r.top + r.height / 2;
    t.node.classList.add(how === "join" ? "drop-into" : after ? "drop-after" : "drop-before");
    drop = { how, t, after };
  };
  let done = false;
  const end = (e) => {
    if (done) return; // pointerup 뒤 lostpointercapture 가 또 온다 — 한 번만
    done = true;
    grip.removeEventListener("pointermove", move);
    grip.removeEventListener("pointerup", end);
    grip.removeEventListener("pointercancel", end);
    grip.removeEventListener("lostpointercapture", end);
    ghost.remove();
    li.classList.remove("dragging");
    clearDrop();
    if (e.type !== "pointerup" || !drop) return;
    if (drop.how === "join") joinGroup(drop.t.id, d.id);
    else moveItem(d.kind, d.id, drop.t.id, drop.after);
  };
  grip.addEventListener("pointermove", move);
  grip.addEventListener("pointerup", end);
  grip.addEventListener("pointercancel", end);
  grip.addEventListener("lostpointercapture", end);
  move(ev);
}

/// 레일 타일 끌기(사용자 · 그룹 순서) — 포인터로(HTML 끌어 놓기는 브라우저 끌기 확장이 가로챈다, 대화 줄 · 요청 보드와 같다).
/// 누른 채 `DRAG_SLOP` 넘게 움직이면 끌기 — 사본이 따라오고 같은 묶음 타일 위 = 앞 · 뒤 표시, 떼면 `moveItem`. 끈 뒤 이어지는 click(타일 고르기)은 먹는다.
export function tileGrab(ev) {
  if (ev.button !== 0) return;
  const tile = ev.currentTarget;
  const d = dragInfo(tile);
  if (!d) return;
  tile.setPointerCapture(ev.pointerId);
  const sx = ev.clientX;
  const sy = ev.clientY;
  const box = tile.getBoundingClientRect();
  const across = getComputedStyle($("rail")).flexDirection === "row"; // 좁은 화면 = 가로 레일
  const same = [...$("rail").querySelectorAll(`[data-drag="${d.kind}"]`)];
  let ghost = null;
  let drop = null;
  let done = false;
  const move = (e) => {
    if (!ghost) {
      if (!pastSlop(e.clientX - sx, e.clientY - sy)) return;
      railDragging = true; // 끄는 동안 레일을 다시 그리지 않는다 — 캡처한 타일이 DOM 에서 빠지지 않게
      ghost = tile.cloneNode(true);
      ghost.classList.add("rail-ghost");
      ghost.style.width = `${box.width}px`;
      document.body.append(ghost);
      tile.classList.add("dragging");
    }
    ghost.style.transform = `translate(${e.clientX - (sx - box.left)}px, ${e.clientY - (sy - box.top)}px)`;
    clearDrop();
    drop = null;
    // 놓을 자리 = 레일 안이면 가장 가까운 같은 묶음 타일(타일 사이 틈 · 아래 이름 · 양 끝 밖에 놓아도 된다, `railSlot`).
    const rail = $("rail").getBoundingClientRect();
    if (across ? e.clientY < rail.top || e.clientY > rail.bottom : e.clientX < rail.left || e.clientX > rail.right) return;
    const mid = (n) => {
      const r = n.getBoundingClientRect();
      return across ? r.left + r.width / 2 : r.top + r.height / 2;
    };
    const slot = railSlot(same.map(mid), across ? e.clientX : e.clientY);
    const t = slot && dragInfo(same[slot.index]);
    if (!t || t.id === d.id) return;
    t.node.classList.add(slot.after ? "drop-after" : "drop-before");
    drop = { t, after: slot.after };
  };
  const end = (e) => {
    if (done) return;
    done = true;
    tile.removeEventListener("pointermove", move);
    tile.removeEventListener("pointerup", end);
    tile.removeEventListener("pointercancel", end);
    tile.removeEventListener("lostpointercapture", end);
    if (!ghost) return; // 끌지 않았다 — 그냥 누름(click 이 타일을 고른다)
    railDragging = false;
    ghost.remove();
    tile.classList.remove("dragging");
    clearDrop();
    const eat = (c) => {
      c.stopImmediatePropagation();
      c.preventDefault();
    };
    tile.addEventListener("click", eat, { capture: true, once: true });
    setTimeout(() => tile.removeEventListener("click", eat, { capture: true }), 0);
    if (e.type === "pointerup" && drop) moveItem(d.kind, d.id, drop.t.id, drop.after);
    else renderRail(); // 끄는 동안 미룬 다시 그리기
  };
  tile.addEventListener("pointermove", move);
  tile.addEventListener("pointerup", end);
  tile.addEventListener("pointercancel", end);
  tile.addEventListener("lostpointercapture", end);
}

// 레일 안 HTML 끌기(그림 · 링크 기본 끌기)는 막는다 — 타일 순서 · 대화 넣기 모두 포인터 끌기.
$("rail").addEventListener("dragstart", (ev) => ev.preventDefault());
for (const box of ["rail", "conv_sections", "agent_convs_list"]) {
  // Alt + ↑ / ↓ — 초점이 타일 · 대화 행(손잡이 · 링크 · ⋯)에 있으면 같은 묶음에서 한 칸.
  $(box).addEventListener("keydown", (ev) => {
    if (!ev.altKey || (ev.key !== "ArrowUp" && ev.key !== "ArrowDown")) return;
    const d = dragInfo(ev.target);
    if (!d) return;
    ev.preventDefault();
    stepMove(d, ev.key === "ArrowUp" ? -1 : 1);
  });
}

/// 그 항목을 찾는 선택자 — 대화 행은 같은 절(`data-section`) 안의 것.
export function dragSel(d) {
  return `[data-drag="${d.kind}"][data-id="${d.id}"]${d.section != null ? `[data-section="${CSS.escape(d.section)}"]` : ""}`;
}

/// 같은 묶음(보이는 순서)에서 한 칸 옮기기(`dir` -1 = 위) — Alt + ↑ / ↓ · 타일 메뉴 공용. 초점은 다시 그린 그 항목에(`keepFocus`), 바뀐 자리를 읽어 준다.
export function stepMove(d, dir) {
  const scope = d.kind === "conv" ? $("conv_sections") : $("rail");
  const same = `[data-drag="${d.kind}"]${d.section != null ? `[data-section="${CSS.escape(d.section)}"]` : ""}`;
  const ids = () => [...scope.querySelectorAll(same)].map((n) => Number(n.dataset.id));
  const step = stepTarget(ids(), d.id, dir);
  if (!step) return;
  moveItem(d.kind, d.id, step.target, step.after);
  const moved = scope.querySelector(dragSel(d));
  if (!scope.contains(document.activeElement)) moved?.focus(); // 타일 메뉴에서 옮겼으면 그 타일에
  const name =
    d.kind === "peer"
      ? displayName(state.peers.find((p) => p.id === d.id) ?? {})
      : d.kind === "group"
        ? (state.groups.find((g) => g.id === d.id)?.name ?? "")
        : (moved?.querySelector(".room-title")?.textContent ?? "대화");
  $("order_live").textContent = movedText(name, ids().indexOf(d.id));
}

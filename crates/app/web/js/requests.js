// 요청 — 목록 · 보드 · 보관.
import { api } from "./core/api.js";
import { $, act, el, ico, needsRender } from "./core/dom.js";
import { latest, state } from "./core/state.js";
import { textButton } from "./core/widgets.js";
import { formatTime } from "./logic/format.js";
import { scopeQuery } from "./logic/rail.js";
import { REQ_BTN, boardColumns, canDrop, reqBoxes, reqHead, requestRows } from "./logic/request.js";
import { noteForm, resendState } from "./conv.js";
import { foldEl, grouped, setFolded } from "./sidebar.js";

/// '요청' 화면 — 개요(`state.reqBox` = `null`: 받은 요청 · 보낸 요청 두 절, 머리에 상태별 건수) 또는 탭으로 고른 쪽 목록만. 고른 타일 범위,
/// 최신부터. 머리 건수는 서버 집계(범위 전체), 상태 거르기는 서버에서(`req=` — 오래된 대기 요청도 나온다). 메뉴로 열면 개요 · 거르기 처음으로.
/// ponytail: 행은 쪽(· 상태)마다 최근 500건(서버 최대)까지 — 넘으면 커서 페이지.
export async function loadRequests(open = false) {
  if (open) {
    $("req_state").value = "";
    $("req_archive").checked = false; // archive 항목은 들어올 때마다 숨김
    state.reqBox = reqBoard() ? "theirs" : null; // 보드는 탭 한 쪽만 — 개요는 목록
  }
  const box = state.reqBox;
  const board = reqBoard() && box != null;
  for (const b of ["theirs", "mine"]) $(`req_tab_${b}`).setAttribute("aria-current", b === box ? "page" : "false");
  $("req_view_list").setAttribute("aria-pressed", String(!board));
  $("req_view_board").setAttribute("aria-pressed", String(board));
  // 보드의 열이 곧 상태 — 상태 거르기는 목록에서만.
  $("req_state").hidden = $("req_state_label").hidden = board;
  const boxes = box ? [box] : ["theirs", "mine"];
  const scope = scopeQuery(state.railPeer, state.railGroup);
  const st = board ? "" : $("req_state").value;
  const side = (b) => `side=${b}${scope ? `&${scope}` : ""}`;
  const archived = $("req_archive").checked;
  const ticket = latest.next();
  let lists, counts, olds;
  try {
    [lists, counts, olds] = await Promise.all([
      Promise.all(boxes.map((b) => api("GET", `/api/messages?${side(b)}&ask=request&limit=500${st ? `&req=${st}` : ""}`))),
      Promise.all(boxes.map((b) => api("GET", `/api/messages/req_counts?${side(b)}`))),
      archived ? Promise.all(boxes.map((b) => api("GET", `/api/messages?${side(b)}&ask=request&archived=true&limit=500`))) : null,
    ]);
  } catch (e) {
    $("status_msg").textContent = "요청 목록 오류: " + e.message;
    return;
  }
  if (!latest.is(ticket)) return;
  const data = reqBoxes(boxes, lists, counts, state.peerNames);
  renderReqArchive(boxes, olds);
  $("req_list").hidden = board;
  $("req_board").hidden = !board;
  if (board) {
    if (needsRender($("req_board"), JSON.stringify([box, state.railPeer, state.railGroup, data]), open))
      $("req_board").replaceChildren(...boardEls(boardColumns(data[0].rows, data[0].sum)));
    return;
  }
  if (!needsRender($("req_list"), JSON.stringify([box, st, state.railPeer, state.railGroup, data]), open)) return;
  $("req_list").replaceChildren(...(box ? grouped(`requests:${box}`, data[0].rows, (rs) => requestEls(rs, box)) : data.map(reqBoxEl)));
}

/// archive 된 요청 — "archive 항목 보기" 일 때만, 목록 · 보드 아래 따로 된 절(쪽마다, 처음 펼침).
export function renderReqArchive(boxes, olds) {
  const list = $("req_archive_list");
  list.hidden = !olds;
  if (!olds) return list.replaceChildren();
  list.replaceChildren(
    ...boxes.map((box, i) => {
      const rows = requestRows(olds[i], state.peerNames);
      const head = [el("strong", { text: `archive — ${box === "theirs" ? "받은 요청" : "보낸 요청"}` }), el("span", { class: "stat", text: String(rows.length) })];
      const body = [el("ol", { class: "rows" }, ...(rows.length ? rows.map(requestItem) : [el("li", { class: "hint", text: "없음" })]))];
      return el("li", { class: "fold" }, foldEl(`reqarchive:${box}`, head, body, "peer-fold req-box"));
    }),
  );
}

/// 요청 archive · 해제(이 PC 에만).
export function archiveReq(id, on) {
  act("archive", async () => {
    await api("POST", `/api/messages/${id}/archive`, { archived: on });
    await loadRequests();
    return on ? "archive 로 옮김" : "archive 해제";
  });
}

/// 요청 보기 — 보드면 참. 이 PC 에 남긴다.
/// ponytail: 접힘 목록(`ui_folded`)의 열쇠 하나를 빌려 쓴다 — 보기 설정(그룹 · 정렬 · 거르기)이 늘면 전용 설정 키.
export const REQ_BOARD = "reqboard:me";
export function reqBoard() {
  return state.folded.has(REQ_BOARD);
}

/// 요청 보드 — 열 = 상태(머리 = 이름 · 서버 집계 건수), 카드 = 요청 행 그대로(글자 단추 · 의견 칸 포함). 받은 요청 카드는 손잡이로 끌어
/// 다른 열에 놓으면 카드가 그 열로 옮겨진 채 의견 칸이 열리고, 확인해야 요청자에게 간다(취소면 제자리로).
/// 끌기는 포인터 이벤트 — HTML 끌어 놓기(`draggable`)는 브라우저 끌기 확장(링크 · 글 끌어 검색 등)이 가로채 반응한다.
export function boardEls(cols) {
  return cols.map((c) => {
    const cards = c.rows.map((r) => {
      const li = requestItem(r);
      li.classList.add("board-card", `req-${r.stateKey}`);
      if (r.moves.length) {
        const grip = el("button", { type: "button", class: "drag-handle", title: "끌어서 상태 옮기기", "aria-label": "끌어서 상태 옮기기" }, ico("grip"));
        grip.addEventListener("pointerdown", (ev) => boardGrab(ev, li, r));
        li.prepend(grip);
      }
      return li;
    });
    return el(
      "section",
      { class: `board-col req-${c.key}`, "data-col": c.key },
      el("h3", { class: "board-head" }, el("span", { class: `tag ask req-${c.key}`, text: c.name }), el("b", { class: "count", text: String(c.count) })),
      el("ol", { class: "rows" }, ...(cards.length ? cards : [el("li", { class: "hint", text: "없음" })])),
    );
  });
}

/// 보드 카드 끌기(포인터) — 손잡이를 누른 채 움직이면 카드 사본이 따라오고, 놓을 수 있는 열은 강조 · 없는 열은 흐리게. 놓을 수 있는 열 위에서 떼면 `boardDrop`.
export function boardGrab(ev, li, r) {
  if (ev.button !== 0) return;
  ev.preventDefault();
  const grip = ev.currentTarget;
  grip.setPointerCapture(ev.pointerId);
  const box = li.getBoundingClientRect();
  const dx = ev.clientX - box.left;
  const dy = ev.clientY - box.top;
  const ghost = li.cloneNode(true);
  ghost.classList.add("board-ghost");
  ghost.style.width = `${box.width}px`;
  document.body.append(ghost);
  li.classList.add("dragging");
  const cols = [...$("req_board").querySelectorAll(".board-col")];
  for (const c of cols) c.classList.add(canDrop(r, c.dataset.col) ? "drop-can" : "drop-no");
  let over = null;
  const move = (e) => {
    ghost.style.transform = `translate(${e.clientX - dx}px, ${e.clientY - dy}px)`;
    const col = document.elementFromPoint(e.clientX, e.clientY)?.closest(".board-col");
    over = col && canDrop(r, col.dataset.col) ? col : null;
    for (const c of cols) c.classList.toggle("drop-ok", c === over);
  };
  const end = (e) => {
    grip.removeEventListener("pointermove", move);
    grip.removeEventListener("pointerup", end);
    grip.removeEventListener("pointercancel", end);
    ghost.remove();
    li.classList.remove("dragging");
    for (const c of cols) c.classList.remove("drop-ok", "drop-can", "drop-no");
    if (e.type === "pointerup" && over) boardDrop(li, r, over);
  };
  grip.addEventListener("pointermove", move);
  grip.addEventListener("pointerup", end);
  grip.addEventListener("pointercancel", end);
  move(ev);
}

/// 카드를 놓은 열로 옮기고 그 상태의 의견 칸을 연다 — 확인하면 바뀌고(새로 고침), 취소면 원래 열 · 자리로 돌아간다.
export function boardDrop(li, r, col) {
  const mv = r.moves.find((m) => m.to === col.dataset.col);
  const bar = li.querySelector(".req-acts");
  if (!mv || !bar) return;
  const home = li.parentNode;
  const next = li.nextSibling;
  const list = col.querySelector(".rows");
  list.querySelector(":scope > li.hint")?.remove();
  list.prepend(li);
  if (!home.querySelector(":scope > li[data-req]")) home.append(el("li", { class: "hint", text: "없음" }));
  li.classList.add("pending");
  noteForm(bar, r.id, mv.to, mv.label, () => {
    li.classList.remove("pending");
    home.querySelector(":scope > li.hint")?.remove();
    home.insertBefore(li, next);
    if (!list.querySelector(":scope > li[data-req]")) list.append(el("li", { class: "hint", text: "없음" }));
  });
}

// 보기 바꾸기 — 보드를 고를 때 개요면 받은 요청 탭으로.
for (const [id, on] of [
  ["req_view_list", false],
  ["req_view_board", true],
])
  $(id).addEventListener("click", () => {
    setFolded(REQ_BOARD, on);
    if (on && state.reqBox == null) state.reqBox = "theirs";
    loadRequests();
  });

/// 요청 개요 한 절 — 머리 = 이름 · 건수 · 상태별 건수, 안 = 그 쪽 목록('나' 면 상대별 절). 받은 요청은 처음 펼침, 보낸 요청은 처음 접힘.
export function reqBoxEl({ box, sum, rows }) {
  const h = reqHead(box === "theirs" ? "받은 요청" : "보낸 요청", sum);
  const head = [el("strong", { text: h.title }), el("span", { class: "stat", text: h.count }), ...h.states.map((x) => el("span", { class: `stat req-${x.key}`, text: x.text }))];
  const body = [el("ol", { class: "rows" }, ...grouped(`requests:${box}`, rows, (rs) => requestEls(rs, box)))];
  return el("li", { class: "fold" }, foldEl(`requests:${box}`, head, body, "peer-fold req-box", null, box === "mine"));
}

/// 요청 행 — 위 링크(이름 · 상태 · 시각 · 대화 / 본문), 바로 아래 상태 바꾸기 글자 단추(의견 칸도 이 줄에서), 그 아래 의견 · 알리지 못함.
export function requestEls(rows, box) {
  if (!rows.length) return [el("li", { class: "hint", text: box === "theirs" ? "받은 요청 없음" : "보낸 요청 없음" })];
  return rows.map(requestItem);
}

/// 요청 한 줄(목록 행 · 보드 카드 같은 모양).
export function requestItem(r) {
  const li = el(
    "li",
    { "data-req": String(r.id) },
    el(
      "a",
      { href: r.href, draggable: "false" },
      el(
        "span",
        { class: "meta" },
        el("strong", { text: r.who }),
        el("span", { class: r.done ? "tag ask done" : "tag ask", text: r.state }),
        el("span", { text: formatTime(r.at) }),
        el("span", { class: "hint", text: r.title }),
      ),
      el("span", { text: r.preview }),
    ),
  );
  if (r.moves.length || r.resend || r.archive) {
    const bar = el("div", { class: "req-acts" });
    for (const mv of r.moves) bar.append(textButton(mv.label, REQ_BTN[mv.to], () => noteForm(bar, r.id, mv.to, mv.label)));
    if (r.resend) bar.append(textButton("다시 알리기", null, () => resendState(r.id, r.stateKey, r.note)));
    if (r.archive) bar.append(textButton(r.archive.label, null, () => archiveReq(r.id, r.archive.key === "archive")));
    li.append(bar);
  }
  // 의견(받은 쪽이 상태와 함께 남긴 글, 여러 줄) — 회색 힌트가 아니라 상태 색 띠의 상자로 읽히게.
  if (r.note) li.append(el("div", { class: `req-feedback req-${r.stateKey}` }, el("span", { class: "req-feedback-label", text: "의견" }), el("span", { text: r.note })));
  if (r.resend) li.append(el("div", { class: "req-notes" }, el("span", { class: "hint", text: "요청자에게 알리지 못함" })));
  return li;
}

// 탭 — 누르면 그 쪽 목록만, 눌린 탭을 다시 누르면 개요.
for (const b of ["theirs", "mine"])
  $(`req_tab_${b}`).addEventListener("click", (ev) => {
    ev.preventDefault();
    state.reqBox = state.reqBox === b && !reqBoard() ? null : b; // 보드에는 개요가 없다
    loadRequests();
  });
$("req_state").addEventListener("input", () => loadRequests());
$("req_archive").addEventListener("change", () => loadRequests());

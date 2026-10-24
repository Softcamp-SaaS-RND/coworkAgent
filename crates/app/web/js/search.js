// 통합 검색 — 250 ms 뒤 조회, 묶음(메시지 · 대화 · 사람 · 에이전트)별 결과 상자, 누르면 그곳으로.
import { api } from "./core/api.js";
import { $, el, ico } from "./core/dom.js";
import { state } from "./core/state.js";
import { agentTile, rowTile } from "./core/widgets.js";
import { convKindIcon } from "./logic/conv.js";
import { colorOf } from "./logic/peer.js";
import { searchGroups } from "./logic/search.js";
import { Latest } from "./logic/sync.js";
import { newChat } from "./compose.js";
import { pickTile } from "./rail.js";
import { go } from "./shell.js";

export const searchAsk = new Latest();
export let searchTimer;

export function closeSearch() {
  clearTimeout(searchTimer);
  searchAsk.next(); // 오고 있는 결과가 닫힌 상자를 다시 열지 않게
  $("search_box").hidden = true;
}

/// 결과 상자 — 머리(나간 대화 포함 스위치 · 안내)는 그대로 두고 결과만 바꾼다(스위치 초점이 남게).
export function showSearch(children) {
  $("search_results").replaceChildren(...children);
  $("search_box").hidden = false;
}

/// 검색 줄 앞 칩 — 메시지 = 말풍선 · 대화 = 유형 아이콘(둘 다 대화 유형 색, 공유 사본은 점선 빈 칩), 사람 = 그 타일, 에이전트 = 로봇(내 것 = 고른 아이콘 색 · 상대 것 = 그 상대 타일 색).
export function searchMark(m) {
  if (m.type === "msg" || m.type === "conv") {
    const n = m.type === "msg" ? "chat" : convKindIcon({ dir: m.kind });
    return el("span", { class: "s-mark kind-chip", "data-kind": m.kind, "aria-hidden": "true" }, n ? ico(n) : "");
  }
  if (m.type === "peer") {
    const p = state.peers.find((x) => x.id === m.peer);
    return rowTile(`small ${colorOf(m.peer)}`, p?.icon);
  }
  if (m.owner === "me") return agentTile(state.agents.find((a) => a.name === m.name)?.color, "small");
  return el("span", { class: `pa small ${colorOf(m.owner)}`, "aria-hidden": "true" }, ico("bot"));
}

export async function runSearch() {
  const q = $("search_q").value.trim();
  const ticket = searchAsk.next();
  if (!q) {
    closeSearch();
    return;
  }
  let r;
  try {
    r = await api("GET", `/api/search?q=${encodeURIComponent(q)}&closed=${$("search_closed").checked}`);
  } catch (e) {
    if (searchAsk.is(ticket)) showSearch([el("p", { class: "hint", text: "검색 실패: " + e.message })]);
    return;
  }
  if (!searchAsk.is(ticket)) return; // 그 사이 더 쳤다
  const groups = searchGroups(r, state.peerNames, state.convs);
  if (!groups.length) {
    showSearch([el("p", { class: "hint", text: "맞는 것 없음" })]);
    return;
  }
  showSearch(
    groups.map((g) =>
      el(
        "section",
        {},
        el("h3", { text: g.title }),
        el(
          "ul",
          {},
          ...g.rows.map((row) =>
            el(
              "li",
              {},
              el(
                "button",
                { type: "button", class: "s-row", onclick: () => searchGo(row) },
                searchMark(row.mark),
                el("span", { class: "s-text" }, el("span", { text: row.label }), el("span", { class: "sub", text: row.sub })),
                row.tag ? el("span", { class: "chip", text: row.tag }) : "",
              ),
            ),
          ),
        ),
      ),
    ),
  );
}

// 나간 대화 포함 — 이 탭이 열린 동안만 기억(설정 아님), 바꾸면 같은 검색어로 다시.
$("search_closed").addEventListener("change", () => runSearch());

/// 검색 결과로 — 메시지 = 그 글 위치, 대화 = 그 대화, 사람 = 그 타일, 상대 에이전트 = 세션 생성 채움, 내 에이전트 = 설정.
export function searchGo(row) {
  closeSearch();
  if (row.tile != null) pickTile(row.tile);
  else if (row.chat) newChat(row.chat.peer, row.chat.agent);
  else go(row.href);
}

$("search_q").addEventListener("input", () => {
  clearTimeout(searchTimer);
  searchTimer = setTimeout(runSearch, 250);
});
$("search_q").addEventListener("focus", () => {
  if ($("search_q").value.trim() && $("search_box").hidden) runSearch();
});
$("search_q").addEventListener("keydown", (ev) => {
  if (ev.key === "ArrowDown") {
    ev.preventDefault();
    $("search_box").querySelector("button")?.focus();
  }
});

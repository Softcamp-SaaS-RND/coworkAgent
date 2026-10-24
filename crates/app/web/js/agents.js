// 에이전트 화면 — 내 에이전트 · 상대 PC 에이전트 목록.
import { $, el } from "./core/dom.js";
import { state } from "./core/state.js";
import { agentTile } from "./core/widgets.js";
import { displayName } from "./logic/format.js";
import { SESSION_NEW } from "./logic/shell.js";
import { newChat } from "./compose.js";
import { PENCIL_SVG } from "./conv.js";
import { loadPeerAgents } from "./rail.js";
import { renderAgentList } from "./settings/agents.js";
import { goSettings, mdRowId, pick } from "./settings/index.js";
import { foldEl } from "./sidebar.js";

/// 에이전트 화면 — 고른 타일 범위('나' = 내 에이전트 + 페어링된 상대 전부의 에이전트). 상대 목록은 원격 왕복이라 이 화면을 열 때 없는 것만 받는다.
export function loadAgentsView() {
  const p = state.peers.find((x) => x.id === state.railPeer);
  const peers = p ? [p] : state.peers.filter((x) => x.paired);
  for (const x of peers) if (!state.peerAgents.has(x.id)) loadPeerAgents(x.id);
  renderAgentsView();
}

export function renderAgentsView() {
  const p = state.peers.find((x) => x.id === state.railPeer);
  const peers = p ? [p] : state.peers.filter((x) => x.paired);
  $("agents_title").textContent = p ? `${displayName(p)}의 에이전트` : "등록된 에이전트";
  // 앞 표시 = 상대 에이전트 점 · 내 에이전트는 고른 색 로봇 타일.
  const row = (name, sub, props, lead = el("span", { class: "agent-dot", "aria-hidden": "true" })) =>
    el(
      "li",
      {},
      el(
        "a",
        props,
        el("span", { class: "meta" }, lead, el("strong", { class: "agent-name", text: name })),
        sub ? el("span", { class: "hint", text: sub }) : "",
      ),
    );
  const peerRows = (x) => {
    const names = state.peerAgents.get(x.id);
    if (names === undefined) return [el("li", { class: "hint", text: `${displayName(x)} · 불러오는 중 …` })];
    if (names === null) return [el("li", { class: "hint", text: `${displayName(x)} · 목록을 받지 못했습니다` })];
    // 줄 오른쪽 "세션 생성" 단추 — 줄을 눌러도 같다.
    return names.map((n) => {
      const li = row(n, null, {
        href: "#new",
        onclick: (ev) => {
          ev.preventDefault();
          newChat(x.id, n);
        },
      });
      li.classList.add("has-edit");
      li.append(el("button", { type: "button", class: "small agent-new", "aria-label": `${SESSION_NEW} · ${n}`, text: SESSION_NEW, onclick: () => newChat(x.id, n) }));
      return li;
    });
  };
  const none = [el("li", { class: "hint", text: "에이전트 없음" })];
  if (p) {
    const rows = peerRows(p);
    $("agents_list").replaceChildren(...(rows.length ? rows : none));
    return;
  }
  // '나' = 절: 내 에이전트(맨 앞) · 상대마다 '{이름}의 에이전트' — 에이전트가 없는 상대는 숨긴다(불러오는 중 · 실패는 보인다).
  // 내 에이전트 — 줄 오른쪽 연필 = 설정 '에이전트' 에서 그 에이전트를 골라 상세로(`editAgent`, 줄을 눌러도 같다).
  const mine = state.agents.map((a) => {
    const li = row(
      a.name,
      `내 에이전트 · ${a.folder}`,
      {
        href: "#settings/agents",
        onclick: (ev) => {
          ev.preventDefault();
          editAgent(a.id);
        },
      },
      agentTile(a.color, "small"),
    );
    const pen = el("button", { type: "button", class: "icon-btn agent-edit", title: "편집", "aria-label": `${a.name} 편집`, onclick: () => editAgent(a.id) });
    pen.innerHTML = PENCIL_SVG;
    li.classList.add("has-edit");
    li.append(pen);
    return li;
  });
  const sections = [
    ...(mine.length ? [["agents:me", `내 에이전트 · ${mine.length}`, mine]] : []),
    ...peers.map((x) => {
      const names = state.peerAgents.get(x.id);
      return [`agents:${x.id}`, `${displayName(x)}의 에이전트${Array.isArray(names) ? ` · ${names.length}` : ""}`, peerRows(x)];
    }),
  ].filter(([, , rows]) => rows.length);
  $("agents_list").replaceChildren(
    ...(sections.length ? sections.map(([key, title, rows]) => el("li", { class: "fold" }, foldEl(key, [title], [el("ol", { class: "rows" }, ...rows)]))) : none),
  );
}

/// 내 에이전트 편집 — 설정 '에이전트' 로 가서 그 에이전트를 고르고(오른쪽 상세) 목록의 그 줄에 초점.
export function editAgent(id) {
  pick.agents = id;
  renderAgentList(true);
  goSettings("agents", mdRowId("agents", id));
}

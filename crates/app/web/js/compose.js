// 세션 생성 폼.
import { api } from "./core/api.js";
import { $, act, el, unchanged } from "./core/dom.js";
import { state } from "./core/state.js";
import { composeDefaults } from "./logic/conv.js";
import { displayName } from "./logic/format.js";
import { Latest } from "./logic/sync.js";
import { loadGroups } from "./rail.js";
import { go } from "./shell.js";
import { loadConvs } from "./sidebar.js";

/// 세션 생성 화면에 상대 · 에이전트를 채워서 연다(사이드바 · 검색 결과의 상대 에이전트).
export async function newChat(peer, agent) {
  if (!state.peers.some((p) => p.id === peer && p.paired)) {
    $("status_msg").textContent = "페어링되지 않은 상대 · 그룹설정에서 승인하세요";
    go("#settings/peers");
    return;
  }
  state.focusAfter = "chat_title";
  go("#new");
  $("chat_peer").value = String(peer);
  await loadChatAgents(agent);
}

export function fillPeerSelects() {
  // 고르는 중인 칸을 5초마다 다시 만들지 않게 — 그리는 값(상대 · 페어링 · 이름)이 같으면 둔다.
  if (unchanged($("chat_peer"), JSON.stringify(state.peers.map((p) => [p.id, p.paired, displayName(p), p.host])))) return;
  const paired = state.peers.filter((p) => p.paired);
  const sel = $("chat_peer");
  const keep = sel.value;
  sel.replaceChildren(
    el("option", { value: "", text: paired.length ? "상대 선택" : "페어링된 상대 없음" }),
    ...paired.map((p) => el("option", { value: p.id, text: `${displayName(p)} · ${p.host}` })),
  );
  if (paired.some((p) => String(p.id) === keep)) sel.value = keep;
}

export const agentsAsk = new Latest();
/// 세션 생성의 상대 에이전트 — 고른 상대 PC 에 묻는다. `want` 가 그 목록에 있으면 그것을 고른다.
export async function loadChatAgents(want) {
  const id = $("chat_peer").value;
  const sel = $("chat_agent");
  sel.replaceChildren();
  const ticket = agentsAsk.next();
  if (!id) return;
  $("chat_msg").textContent = "상대 에이전트 불러오는 중 …";
  try {
    const r = await api("GET", `/api/peers/${id}/agents`);
    if (!agentsAsk.is(ticket)) return; // 그 사이 다른 상대를 골랐다
    sel.replaceChildren(...r.names.map((n) => el("option", { value: n, text: n })));
    if (want && r.names.includes(want)) sel.value = want;
    $("chat_msg").textContent = r.names.length ? "" : "상대 PC 에 등록된 에이전트가 없습니다";
  } catch (e) {
    $("chat_msg").textContent = "상대 에이전트 목록 실패: " + e.message;
  }
}
$("chat_peer").addEventListener("change", () => loadChatAgents());

/// 세션 생성 '최대 응답 횟수' — '내 응답 에이전트' 를 고를 때만 보이는 하위 줄. 숨긴 동안은 `disabled` 라 폼 검사에서 빠진다
/// (숨긴 칸의 범위 밖 값이 보내기를 소리 없이 막지 않게), 보일 때는 `required` · 1 ~ 50 정수를 브라우저가 검사한다.
export function syncRelayCap() {
  const off = !$("chat_relay_agent").value;
  $("chat_relay_cap").hidden = $("chat_relay_cap_label").hidden = $("chat_relay_cap").disabled = off;
}
$("chat_relay_agent").addEventListener("change", syncRelayCap);

/// 세션 생성 화면을 열 때 — 상대 칸이 비었으면 레일에서 고른 상대로 채우고 그 에이전트 목록을 읽는다.
export function openCompose() {
  if ($("chat_peer").value) return;
  const want = composeDefaults(state.railPeer, state.peers);
  if (!want) return;
  $("chat_peer").value = want;
  loadChatAgents();
}

// 세션 생성 — 글 없이 대화방만 만들고 그 대화방으로(첫 글은 대화방 입력창에서, 질의 · 요청도 거기서).
$("chat_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  const body = {
    peer_id: Number($("chat_peer").value),
    agent: $("chat_agent").value,
    title: $("chat_title").value,
  };
  if (!body.peer_id || !body.agent) {
    $("chat_msg").textContent = "상대와 상대 에이전트를 고르세요";
    return;
  }
  if ($("chat_relay_agent").value) {
    body.relay_agent = $("chat_relay_agent").value;
    body.relay_cap = $("chat_relay_cap").valueAsNumber;
  }
  // 그룹을 고른 채 만든 대화는 그 그룹에.
  if (state.railGroup != null) body.group = state.railGroup;
  act(
    "생성",
    async () => {
      const r = await api("POST", "/api/conversations", body);
      $("chat_title").value = "";
      if (body.group != null) await loadGroups();
      await loadConvs();
      state.focusAfter = "reply_text";
      location.hash = `#c/${r.conversation.id}`;
      return "대화를 만들었습니다 — 메시지를 입력하세요";
    },
    "chat_msg",
  );
});

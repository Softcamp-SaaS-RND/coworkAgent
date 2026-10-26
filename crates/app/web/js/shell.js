// 셸 — 상태 · 내 정보 · 테마 · 화면 전환(라우터) · 상단 바(+ 새로 만들기 · 프로필 메뉴 · 종).
import { api, appOffline } from "./core/api.js";
import { $, act, el, ico } from "./core/dom.js";
import { state } from "./core/state.js";
import { railFace } from "./core/widgets.js";
import { keepsConversation } from "./logic/conv.js";
import { presenceOf } from "./logic/peer.js";
import { RAIL_W, groupLeaves, inGroup, peerLeaves } from "./logic/rail.js";
import { portText } from "./logic/settings.js";
import { newMenu, profileMenu, route } from "./logic/shell.js";
import { loadActivity, loadSaved } from "./activity.js";
import { loadAgentsView } from "./agents.js";
import { openCompose } from "./compose.js";
import { findConv, openConversation, showConversation } from "./conv.js";
import { showDetail } from "./detail.js";
import { renderHome } from "./home.js";
import { loadPeerMessages } from "./peers.js";
import { pickRail, pickTile, railGroupOf } from "./rail.js";
import { loadRequests } from "./requests.js";
import { openSettings } from "./settings/index.js";
import { fillPrefs } from "./settings/prefs.js";
import { renderAgentConvs, renderConvList, renderSideHead, renderSideNav, renderStarred } from "./sidebar.js";
import { openTileEdit } from "./tiles.js";

export async function showStatus() {
  const KEYS = ["version", "data_dir", "db"];
  try {
    const s = await api("GET", "/api/status");
    for (const k of KEYS) $(k).textContent = s[k];
    $("top_version").textContent = `v${s.version}`; // 상단 바 wordmark 뒤
  } catch (e) {
    for (const k of KEYS) $(k).textContent = "오류: " + e.message;
  }
}

export async function showMe() {
  try {
    state.me = await api("GET", "/api/me");
    const me = state.me;
    $("me_name").textContent = me.name;
    $("me_host").textContent = me.host;
    $("me_fingerprint").textContent = me.fingerprint;
    $("me_addrs").textContent = me.addrs.length ? me.addrs.join(", ") : "포트 " + me.port;
    const ports = portText(me);
    $("me_ui_port").textContent = ports.ui;
    $("me_peer_port").textContent = ports.peer;
    $("me_conns").textContent = ports.conns;
    renderSideHead();
  } catch (e) {
    for (const id of ["me_name", "me_host", "me_fingerprint", "me_addrs", "me_ui_port", "me_peer_port", "me_conns", "side_name"]) $(id).textContent = "오류: " + e.message;
  }
}

export function applyTheme(theme) {
  document.documentElement.dataset.theme = theme === "dark" ? "dark" : "light";
}

/// 레일 타일 크기 — 작게 = 작은 아이콘 + 오른쪽 이름 한 줄(레일이 넓어진다, `app.css` `#app.tile-small`).
export function applyTileSize(size) {
  $("app").classList.toggle("tile-small", size === "small");
  $("rail_resize").setAttribute("aria-valuenow", String(size === "small" ? RAIL_W.small : RAIL_W.normal));
}

/// 해시로 이동 — 이미 그 해시면 hashchange 가 없으므로 바로 다시 그린다(빈 해시 = `#`).
export function go(hash) {
  if ((location.hash || "#") === hash) render();
  else location.hash = hash;
}

/// 상단 바 프로필 단추 — 24px 네모 얼굴(레일 '나' 와 같은 얼굴 — `railFace`) + 표시 이름 한 번. 얼굴 점 = 내 상태(`presenceOf` — 받기 가능 초록 · 집중 주황 ·
/// 앱과 연결 끊김 붉음). 이름은 CSS 로 약 12자에서 말줄임, 전체는 `title`.
export function renderAvatar() {
  const name = state.settings.me_alias || "나";
  const st = presenceOf(state.settings.focus, appOffline());
  const face = el("span", { class: `avatar-face st-${st.key}`, "aria-hidden": "true" }, railFace(state.settings.me_icon, false, state.settings.me_color, true), el("span", { class: "avatar-dot" }));
  $("avatar_btn").title = `${name} · ${st.text}`;
  $("avatar_btn").setAttribute("aria-label", `프로필 · ${st.text}`);
  $("avatar_btn").replaceChildren(face, el("span", { class: "avatar-name", text: name }));
}

/// 화면 → 메인의 절 id.
export const VIEW_EL = {
  home: "home",
  activity: "activity",
  requests: "requests",
  starred: "starred",
  saved: "saved",
  agents: "agents_view",
  agentConvs: "agent_convs",
  peer: "peer",
  conversation: "conversation",
  sent: "conversation",
  new: "new",
  settings: "settings",
};

export function show(view) {
  const id = VIEW_EL[view];
  for (const v of new Set(Object.values(VIEW_EL))) $(v).hidden = v !== id;
  // 설정은 대화 목록 없이 레일 옆 전체 폭, 실행 상세는 대화 화면에서만.
  $("sidebar").hidden = $("side_resize").hidden = view === "settings";
  showDetail();
  renderSideNav();
}

/// 그린 뒤 초점 — 메뉴로 온 화면의 첫 칸(접힌 `<details>` 안이면 편다).
export function applyFocus() {
  const id = state.focusAfter;
  state.focusAfter = null;
  const f = id ? $(id) : null;
  if (!f) return;
  f.closest("details")?.setAttribute("open", "");
  f.focus();
}

export async function render() {
  const r = route(location.hash);
  closeMenus();
  $("rail_me").setAttribute("aria-pressed", String(state.railPeer == null && state.railGroup == null));
  // `#s/{id}`(보낸 글 탭 링크 호환)는 그 대화로.
  if (r.view === "sent") {
    location.replace(`#c/${r.id}`);
    return;
  }
  if (r.view === "conversation") {
    const conv = await findConv(r.id);
    if (!conv) {
      $("status_msg").textContent = `대화 ${r.id} 를 찾지 못했습니다`;
      show("home");
      return;
    }
    state.current = conv;
    // 뒤로 가기 등으로 레일에서 고른 PC · 그룹과 다른 대화를 열면 레일을 '나'(전부)로 — 열린 대화로 보낼 수 있게.
    if (!keepsConversation(conv, state.railPeer) || (railGroupOf() && !inGroup([conv], railGroupOf()).length)) pickRail(null);
    renderConvList();
    showConversation(conv);
    state.focusMsg = r.msg ?? null;
    await openConversation(conv);
    applyFocus();
    return;
  }
  // `#p/{id}`(상대 주고받은 글 링크 호환)는 그 상대를 골라 그 상대 개요로(상대 타일의 첫 화면).
  if (r.view === "peer" && r.id != null) {
    pickRail(r.id);
    location.replace("#");
    return;
  }
  state.current = null;
  // 내 활동 · 주고받은 글(`#p`)은 '나' 에만.
  if ((r.view === "activity" || r.view === "peer") && state.railPeer != null) pickRail(null);
  // 그룹에는 내 활동 · 주고받은 글 · 에이전트 메뉴가 없다 — '나' 로 바꿔 연다.
  if (groupLeaves(r.view) && state.railGroup != null) pickRail(null);
  if (peerLeaves(r.view) && state.railPeer != null) pickRail(null); // 에이전트 응답은 '나' 에만
  renderConvList();
  show(r.view);
  if (r.view === "home") {
    state.previewAll = true; // 개요에 들어옴 — 미리보기 글을 전부 새로(다른 화면에서 고친 글)
    renderHome();
  }
  if (r.view === "activity") await loadActivity(true);
  if (r.view === "requests") await loadRequests(true);
  if (r.view === "starred") renderStarred();
  if (r.view === "saved") await loadSaved();
  if (r.view === "agents") loadAgentsView();
  if (r.view === "agentConvs") renderAgentConvs();
  if (r.view === "new") openCompose();
  if (r.view === "peer") await loadPeerMessages(r.id);
  // 초대 수락 칸 — 트리 · 새로 만들기 "초대 수락" · 레일 + 로 오면 펼치고, 다른 설정 항목 · 다른 화면으로 가면 접는다.
  if (r.view === "settings") openSettings(r);
  applyFocus();
}

export const MENUS = [
  ["new_btn", "new_menu"],
  ["avatar_btn", "profile_menu"],
];

export function closeMenus() {
  for (const [b, m] of MENUS) {
    $(m).hidden = true;
    $(b).setAttribute("aria-expanded", "false");
  }
}

export function toggleMenu(b, m) {
  const open = $(m).hidden;
  closeMenus();
  if (!open) return;
  $(m).hidden = false;
  $(b).setAttribute("aria-expanded", "true");
  $(m).querySelector("button")?.focus();
}

$("new_menu").replaceChildren(
  ...newMenu().map((i) =>
    el(
      "button",
      {
        type: "button",
        role: "menuitem",
        onclick: () => {
          closeMenus();
          state.focusAfter = i.focus;
          go(i.href);
          if (i.press) $(i.press).click();
        },
      },
      ico(i.icon),
      el("span", { text: i.label }),
    ),
  ),
);
$("new_btn").addEventListener("click", () => toggleMenu("new_btn", "new_menu"));

/// 집중 켜고 끄기 — 켜 있는 동안 상대 PC 의 새 요청을 받지 않고 알림을 멈춘다(알림 일시 중지 값은 그대로).
export function setFocus(on) {
  act("집중", async () => {
    fillPrefs(await api("PUT", "/api/settings", { focus: on }));
    return on ? "집중으로 바꿨습니다" : "받기 가능으로 바꿨습니다";
  });
}

/// 프로필 메뉴 — 머리 "상태" · 받기 가능 / 집중(하나만) · 앱과 연결 끊김(자동 표시 — 단추 아님) | 그 밖. 켜짐은 설정 값으로(열 때마다).
export function renderProfileMenu() {
  $("profile_menu").replaceChildren(
    el("div", { class: "menu-head", text: "상태" }),
    ...profileMenu(state.settings.notify_paused, state.settings.focus, appOffline()).map((i) => {
      if (i.sep) return el("hr", { class: "menu-sep" });
      const parts = [i.state ? el("span", { class: `st-dot st-${i.state}`, "aria-hidden": "true" }) : i.icon ? ico(i.icon) : "", el("span", { text: i.label }), i.hint ? el("span", { class: "menu-hint", text: i.hint }) : ""];
      if (i.info) return el("div", { class: i.current ? "menu-info on" : "menu-info", role: "none" }, ...parts);
      return el(
        "button",
        {
          type: "button",
          role: i.radio ? "menuitemradio" : i.checked === undefined ? "menuitem" : "menuitemcheckbox",
          "aria-checked": i.checked === undefined ? null : String(i.checked),
          onclick: () => profileAct(i),
        },
        ...parts,
      );
    }),
  );
}

export function profileAct(i) {
  closeMenus();
  if (i.key === "settings") {
    go(i.href);
  } else if (i.key === "me") {
    $("me_dialog").showModal();
  } else if (i.key === "tile") {
    pickTile(null); // '나' 타일 — 사이드바 머리의 편집 폼
    openTileEdit();
    // 설정 화면에서 왔으면 사이드바가 아직 숨어 있다 — 안내 화면을 그린 뒤 초점.
    if ($("sidebar").hidden) state.focusAfter = "tile_alias";
  } else if (i.key === "avail" || i.key === "focus") {
    if (!i.checked) setFocus(i.key === "focus"); // 이미 고른 상태면 그대로
  } else {
    // 알림 일시 중지(이 PC 에만) — 켜 있는 동안 브라우저 알림 · 트레이 표시를 내지 않는다, 풀면 알림 설정대로.
    const on = !i.checked;
    act("알림 일시 중지", async () => {
      fillPrefs(await api("PUT", "/api/settings", { notify_paused: on }));
      return on ? "알림을 일시 중지했습니다" : "알림을 다시 켰습니다";
    });
  }
}

$("avatar_btn").addEventListener("click", () => {
  renderProfileMenu();
  toggleMenu("avatar_btn", "profile_menu");
});
// 종 = 내 활동 전부.
$("bell").addEventListener("click", (ev) => {
  ev.preventDefault();
  pickRail(null);
  go("#activity");
});

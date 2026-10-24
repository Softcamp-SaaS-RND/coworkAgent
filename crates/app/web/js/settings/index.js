// 설정 — 메뉴 나무 · 목록 + 상세 도우미 · 초대 수락 칸.
import { $, button, el, ico, lead } from "../core/dom.js";
import { state } from "../core/state.js";
import { settingsTree } from "../logic/settings.js";
import { renderAgentList, showAgentNew } from "./agents.js";
import { renderPeerList } from "./conn.js";
import { loadDebug } from "./debug.js";
import { go } from "../shell.js";

/// 설정 화면의 한 화면으로(`#settings/{화면 · 별칭}`) — 그린 뒤 `focus` 칸에 초점.
export function goSettings(id, focus = null) {
  state.focusAfter = focus;
  go(`#settings/${id}`);
}

// 레일 '사용자' 단추 — 새로 만들기 "초대 수락" 과 같게 초대 수락 칸을 열어 바로 붙여 넣게(칸은 라우터가 연다).
$("rail_add").addEventListener("click", () => goSettings("invite_accept", "invite_text"));
$("rail_settings").addEventListener("click", () => go("#settings"));

/// 설정 목록에서 고른 항목 — `conn` 사용자 id · `"invite"`(초대 생성) · `"accept"`(초대 수락), `groups` 그룹 id, `agents` 에이전트 id · `"new"`(등록).
/// null = 안 고름(상세 = 안내). 다시 그려도(5초 새로 고침) 그대로.
export const pick = { conn: null, groups: null, agents: null };

/// 고름을 바꾸고 그 화면을 다시 그린다.
export function setPick(page, id) {
  pick[page] = id;
  if (page === "agents") renderAgentList(true);
  else renderPeerList(true);
}

export function mdRowId(page, id) {
  return `md_${page}_${id}`;
}

/// 상세 닫기(✕ · 닫기) = 고름 풀기, 초점은 그 상세를 연 곳(목록 줄 · 머리 단추)으로 — 숨은 칸에 초점이 남지 않게.
export function closePick(page) {
  const was = pick[page];
  setPick(page, null);
  const opener = { invite: "window_btn", accept: "accept_btn", new: "agent_new_btn" }[was];
  $(opener ?? mdRowId(page, was))?.focus();
}

/// 목록 줄 — 누르면 그 항목을 고른다(상세가 오른쪽에). `face` = 타일, `end` = 줄 끝(상태 · 표시).
export function mdRow(page, id, face, name, sub, ...end) {
  return el(
    "li",
    {},
    el(
      "button",
      { type: "button", id: mdRowId(page, id), class: "md-row", "aria-pressed": String(pick[page] === id), onclick: () => setPick(page, id) },
      face,
      el("span", { class: "md-row-text" }, el("strong", { text: name }), sub ? el("span", { class: "hint", text: sub }) : ""),
      ...end,
    ),
  );
}

/// 상세 머리 — 타일 · 이름 · 부제 · ✕(상세 닫기).
export function mdHead(page, face, name, sub = "") {
  return el(
    "div",
    { class: "md-head" },
    face,
    el("span", { class: "md-title" }, el("strong", { text: name }), sub),
    el("button", { type: "button", class: "icon-btn md-x", title: "닫기", "aria-label": "상세 닫기", onclick: () => closePick(page) }, ico("x")),
  );
}

/// 상세 아래 단추 줄 — 작업들 · 끝에 닫기.
export function mdFoot(page, ...actions) {
  return el("div", { class: "md-foot" }, ...actions, el("span", { class: "grow" }), button("닫기", () => closePick(page), undefined, "x"));
}

// 정적 상세(초대 생성 · 초대 수락 · 새 에이전트)의 ✕ · 닫기.
for (const b of document.querySelectorAll("#settings [data-close]")) b.addEventListener("click", () => closePick(b.dataset.close));

/// 연결 "초대 수락" — 상세 칸에 붙여 넣기 폼을 연다 · 닫는다.
export function showAccept(on) {
  if (on) setPick("conn", "accept");
  else if (pick.conn === "accept") setPick("conn", null);
}
$("accept_btn").addEventListener("click", () => {
  const on = pick.conn !== "accept";
  showAccept(on);
  if (on) $("invite_text").focus({ preventScroll: true });
});

/// 설정 왼쪽 메뉴(평평한 8개). 고른 화면은 `openSettings` 가 `aria-current="page"` 로 표시한다.
export function renderTree() {
  const item = (n) =>
    el(
      "li",
      {},
      el(
        "a",
        {
          href: `#settings/${n.id}`,
          "data-page": n.id,
          onclick: (ev) => {
            ev.preventDefault();
            goSettings(n.id);
          },
        },
        ico(n.icon),
        el("span", { text: n.label }),
      ),
    );
  $("settings_tree").replaceChildren(el("ul", {}, ...settingsTree().map(item)));
  // 화면 제목 = 메뉴와 같은 아이콘(한 곳, 강조색은 CSS). 다시 그려도 한 번만.
  for (const n of settingsTree()) {
    const h = $(`${n.id}_h`);
    if (!h.querySelector(".ico")) h.prepend(lead(n.icon));
  }
}

/// 설정 화면 — 고른 메뉴 화면만 보이고 그 메뉴에 `aria-current`. 진입 이름(`sub`): 초대 수락 = 상세에 붙여 넣기 폼(다른 진입이면 닫음),
/// 새 에이전트 = 상세에 등록 폼. 초대 생성은 새로 만들기가 단추를 누른다(해시로 다시 올 때는 누르지 않는다).
export function openSettings(r) {
  for (const n of settingsTree()) $(n.id).hidden = n.id !== r.page;
  for (const a of $("settings_tree").querySelectorAll("a")) a.setAttribute("aria-current", a.dataset.page === r.page ? "page" : "false");
  showAccept(r.sub === "invite_accept");
  if (r.sub === "agent_new") showAgentNew(true);
  if (r.page === "debug") loadDebug();
}

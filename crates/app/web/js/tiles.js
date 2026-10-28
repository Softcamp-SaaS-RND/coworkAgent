// 타일 편집(표시 이름 · 아이콘 — 이 PC 에만) · 타일 메뉴.
import { api } from "./core/api.js";
import { $, act, button, el, ico } from "./core/dom.js";
import { closeFloat, focusAfterFloat, openFloat } from "./core/float.js";
import { state } from "./core/state.js";
import { railFace } from "./core/widgets.js";
import { AGENT_COLORS } from "./logic/agent.js";
import { groupNameError, stepTarget } from "./logic/rail.js";
import { stepMove } from "./drag.js";
import { refresh } from "./events.js";
import { loadGroups, loadPeers, pickGroup, pickTile } from "./rail.js";
import { saveSettings } from "./settings/prefs.js";

export const ICON_PX = 80; // 타일 40px × 2(고해상도 화면)
/// 열린 타일 편집 — `kind` = `me` · `peer` · `group`, `id` = 상대 · 그룹 id, `icon` = `undefined`(그대로) · `""`(지움) · PNG data URL,
/// `color` = `undefined`(그대로) · 고른 타일 색(`AGENT_COLORS`, '나' 는 `""` = 흑연도).
export let tileEdit = null;

/// 지금 고른 타일(레일 범위).
export function currentTile() {
  if (state.railGroup != null) return { kind: "group", id: state.railGroup };
  if (state.railPeer != null) return { kind: "peer", id: state.railPeer };
  return { kind: "me", id: null };
}

/// 타일의 지금 값 — 이름(편집 칸) · 빈 이름일 때 보일 이름 · 그림 · 타일 색('나' 는 `""` = 흑연).
export function tileNow(t) {
  if (t.kind === "group") {
    const g = state.groups.find((x) => x.id === t.id);
    return g && { value: g.name, fallback: g.name, icon: g.icon, color: g.color };
  }
  if (t.kind === "peer") {
    const p = state.peers.find((x) => x.id === t.id);
    return p && { value: p.alias ?? "", fallback: p.name, icon: p.icon, color: p.color };
  }
  return { value: state.settings.me_alias ?? "", fallback: "나", icon: state.settings.me_icon, color: state.settings.me_color ?? "" };
}

export function openTileEdit() {
  const t = currentTile();
  const now = tileNow(t);
  if (!now) return;
  tileEdit = { ...t, icon: undefined, color: undefined };
  const group = t.kind === "group";
  $("tile_alias_label").textContent = group ? "그룹 이름" : "표시 이름";
  // 그룹 이름 길이는 `groupNameError`(코드 포인트 40자)로 — HTML 제한은 UTF-16 단위라 두지 않는다.
  if (group) $("tile_alias").removeAttribute("maxlength");
  else $("tile_alias").maxLength = 32;
  $("tile_alias").value = now.value;
  $("tile_alias").placeholder = now.fallback;
  $("tile_icon_file").value = "";
  $("tile_msg").textContent = "";
  renderTilePreview();
  renderTileColors();
  $("tile_form").hidden = false;
  $("tile_alias").focus();
}

export function closeTileEdit() {
  tileEdit = null;
  $("tile_form").hidden = true;
}

/// 미리보기 — 고친 그림 · 색으로 그린 레일 타일(그림이 없으면 사람 · 세 사람 모양), 이름 칸 입력 · 색 고르기마다 다시 그린다.
export function renderTilePreview() {
  const now = tileNow(tileEdit);
  if (!now) return;
  const icon = tileEdit.icon !== undefined ? tileEdit.icon : now.icon;
  const color = tileEdit.color !== undefined ? tileEdit.color : now.color;
  const box = $("tile_preview");
  box.className = tileEdit.kind === "me" ? "rail-btn tile-preview me" : "rail-btn tile-preview";
  box.replaceChildren(railFace(icon, tileEdit.kind === "group", color, tileEdit.kind === "me"));
}

/// 아이콘 색 단추 — 에이전트 상세와 같은 여섯 색(`AGENT_COLORS`), '나' 는 맨 앞 흑연(기본 — `""`) 하나 더. 고르면 미리보기만, 저장 때 보낸다.
export function renderTileColors() {
  const now = tileNow(tileEdit);
  if (!now) return;
  const picked = tileEdit.color !== undefined ? tileEdit.color : now.color;
  const box = $("tile_colors");
  box.replaceChildren(
    ...(tileEdit.kind === "me" ? ["", ...AGENT_COLORS] : AGENT_COLORS).map((c) =>
      el("button", {
        type: "button",
        class: `swatch ${c || "rail-2"}`,
        title: `아이콘 색 ${c || "흑연"}`,
        "aria-label": `아이콘 색 ${c || "흑연"}`,
        "aria-pressed": String(c === picked),
        onclick: (ev) => {
          if (!tileEdit) return;
          tileEdit.color = c;
          for (const b of box.children) b.setAttribute("aria-pressed", String(b === ev.currentTarget));
          renderTilePreview();
        },
      }),
    ),
  );
}

/// 고른 그림 → 가운데 정사각형을 잘라 `ICON_PX` 크기 PNG data URL(서버는 PNG data URL 만 받는다). 그림은 data URL 로 읽는다 — CSP
/// `img-src 'self' data:` 가 `blob:` 을 막는다(실측 — 객체 URL 이면 "cannot be decoded").
export async function iconFromFile(file) {
  const url = await new Promise((resolve, reject) => {
    const r = new FileReader();
    r.onload = () => resolve(r.result);
    r.onerror = () => reject(r.error ?? new Error("그림을 읽지 못했습니다"));
    r.readAsDataURL(file);
  });
  const img = new Image();
  img.src = url;
  await img.decode();
  const s = Math.min(img.naturalWidth, img.naturalHeight);
  if (!s) throw new Error("크기를 알 수 없는 그림");
  const canvas = el("canvas", { width: ICON_PX, height: ICON_PX });
  canvas.getContext("2d").drawImage(img, (img.naturalWidth - s) / 2, (img.naturalHeight - s) / 2, s, s, 0, 0, ICON_PX, ICON_PX);
  return canvas.toDataURL("image/png");
}

$("tile_edit_btn").addEventListener("click", () => ($("tile_form").hidden ? openTileEdit() : closeTileEdit()));
$("tile_more_btn").addEventListener("click", (ev) => openTileMenu(currentTile(), ev.currentTarget));
$("rail_me").addEventListener("contextmenu", (ev) => tileMenuEvent(ev, { kind: "me", id: null }));
// 레일 어디서든(타일 사이 빈칸 · 바탕 포함) 브라우저 메뉴는 띄우지 않는다 — 타일 위면 타일 메뉴(`tileMenuEvent`)만.
$("rail").addEventListener("contextmenu", (ev) => ev.preventDefault());

/// 레일 타일 우클릭 · 메뉴 키(Shift + F10 포함 — 브라우저가 같은 `contextmenu` 를 낸다) → 타일 메뉴.
export function tileMenuEvent(ev, t) {
  ev.preventDefault();
  openTileMenu(t, ev.currentTarget);
}

/// 그 타일을 고른다 — 타일 편집 폼은 고른 타일의 사이드바 머리에 있다.
export function selectTile(t) {
  if (t.kind === "group") {
    if (state.railGroup !== t.id) pickGroup(t.id);
  } else if (state.railPeer !== t.id || state.railGroup != null) pickTile(t.id);
}

/// 타일 메뉴 — "타일 편집"(이름 · 그림) · "이미지 지우기"(그림이 있을 때) · 위로 · 아래로 옮기기 · 그룹이면 "그룹 삭제"(확인 한 번 —
/// 대화는 지우지 않는다).
export function openTileMenu(t, anchor) {
  const now = tileNow(t);
  if (!now) return;
  const item = (icon, text, onclick, cls) => el("button", { type: "button", role: "menuitem", onclick, class: cls }, ico(icon), el("span", { text }));
  const items = [
    item("pencil", "타일 편집", () => {
      closeFloat(false);
      selectTile(t);
      openTileEdit();
    }),
  ];
  if (now.icon)
    items.push(
      item("image-off", "이미지 지우기", async () => {
        closeFloat(true);
        await act("이미지 지우기", async () => {
          if (t.kind === "me") await saveSettings({ me_icon: "" });
          else if (t.kind === "group") {
            await api("PATCH", `/api/groups/${t.id}`, { icon: "" });
            await loadGroups();
          } else {
            await api("PATCH", `/api/peers/${t.id}`, { icon: "" });
            await loadPeers(true);
          }
          return "이미지를 지웠습니다";
        });
      }),
    );
  // 위로 · 아래로 옮기기 — Alt + ↑ / ↓ 와 같다. 맨 위 · 맨 아래면 그 항목은 꺼진다.
  if (t.kind !== "me") {
    items.push(el("hr", { class: "menu-sep" }));
    const order = [...$("rail").querySelectorAll(`[data-drag="${t.kind}"]`)].map((n) => Number(n.dataset.id));
    for (const [text, dir] of [
      ["위로 옮기기", -1],
      ["아래로 옮기기", 1],
    ]) {
      const b = item(dir < 0 ? "arrow-up" : "arrow-down", text, () => {
        closeFloat(false);
        stepMove({ kind: t.kind, id: t.id, section: null }, dir);
      });
      b.append(el("span", { class: "kbd", "aria-hidden": "true", text: dir < 0 ? "Alt + ↑" : "Alt + ↓" }));
      b.disabled = !stepTarget(order, t.id, dir);
      items.push(b);
    }
  }
  if (t.kind === "group")
    items.push(
      item(
        "trash",
        "그룹 삭제",
        () => {
          $("float").replaceChildren(groupDeleteForm(t.id));
          $("float").setAttribute("role", "dialog");
          $("float").querySelector("button")?.focus();
        },
        "danger-text",
      ),
    );
  openFloat(anchor, "menu", "타일 메뉴", items);
}

/// 그룹 삭제 확인(한 번) — 타일 메뉴 · 그룹설정 상세 공용. 그룹 · 소속만 지운다(대화 · 상대는 그대로).
export function groupDeleteForm(id) {
  const g = state.groups.find((x) => x.id === id);
  return el(
    "div",
    { class: "float-form" },
    el("strong", { text: `그룹 ${g?.name ?? ""} 삭제` }),
    el("p", { class: "hint", text: "그룹만 지웁니다. 대화는 남습니다." }),
    el(
      "div",
      { class: "actions" },
      button("취소", () => closeFloat(true)),
      button(
        "삭제",
        async () => {
          closeFloat(false);
          await act("그룹 삭제", async () => {
            await api("DELETE", `/api/groups/${id}`);
            const was = state.railGroup === id;
            await loadGroups();
            if (was) pickTile(null);
            focusAfterFloat("#rail_group_add");
            return `그룹 ${g?.name ?? ""} 을 지웠습니다`;
          });
        },
        "danger",
      ),
    ),
  );
}

$("tile_cancel").addEventListener("click", closeTileEdit);
$("tile_alias").addEventListener("input", () => tileEdit && renderTilePreview());
$("tile_icon_clear").addEventListener("click", () => {
  if (!tileEdit) return;
  tileEdit.icon = "";
  $("tile_icon_file").value = "";
  renderTilePreview();
});
$("tile_icon_file").addEventListener("change", async () => {
  const file = $("tile_icon_file").files[0];
  const edit = tileEdit;
  if (!file || !edit) return;
  try {
    const icon = await iconFromFile(file);
    if (tileEdit !== edit) return; // 그 사이 닫았거나 다른 타일로 옮겼다
    edit.icon = icon;
    renderTilePreview();
    $("tile_msg").textContent = "저장하면 아이콘이 바뀝니다";
  } catch (e) {
    $("tile_msg").textContent = "아이콘을 만들지 못했습니다: " + e.message;
  }
});
$("tile_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  const edit = tileEdit;
  if (!edit) return;
  const { kind, id, icon } = edit;
  const alias = $("tile_alias").value;
  // 색은 바꿨을 때만 보낸다.
  const color = edit.color !== undefined && edit.color !== tileNow(edit)?.color ? edit.color : undefined;
  const tile = (body) => ({ ...body, ...(icon === undefined ? {} : { icon }), ...(color === undefined ? {} : { color }) });
  if (kind === "group") {
    const bad = groupNameError(alias, state.groups, id);
    if (bad) {
      $("tile_msg").textContent = bad;
      return;
    }
  }
  act(
    "타일 저장",
    async () => {
      if (kind === "me") {
        const body = { me_alias: alias, ...(icon === undefined ? {} : { me_icon: icon }), ...(color === undefined ? {} : { me_color: color }) };
        await saveSettings(body);
      } else if (kind === "group") {
        await api("PATCH", `/api/groups/${id}`, tile({ name: alias }));
        await loadGroups();
      } else {
        await api("PATCH", `/api/peers/${id}`, tile({ alias }));
        await loadPeers(true);
        await refresh(); // 열린 대화 · 목록의 발화자 이름도 새 표시 이름으로
      }
      if (tileEdit === edit) closeTileEdit(); // 그 사이 다른 타일 편집을 열었으면 그대로
    },
    "tile_msg",
  );
});

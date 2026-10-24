// 대화 메뉴 — 나가기 · 기록 삭제 · 그룹 넣기.
import { api } from "./core/api.js";
import { $, act, button, el, ico } from "./core/dom.js";
import { closeFloat, floatFrom, focusAfterFloat, openFloat } from "./core/float.js";
import { state } from "./core/state.js";
import { LEAVE_CONFIRM, convLabel, convMenuItems } from "./logic/conv.js";
import { sortByPos } from "./logic/order.js";
import { membershipDiff } from "./logic/rail.js";
import { refresh } from "./events.js";
import { changeGroup, loadGroups, railGroupOf } from "./rail.js";
import { go } from "./shell.js";

/// 대화 ⋯ 메뉴 — "그룹" ▸ 그룹마다 체크(누를 때마다 넣고 뺌, 메뉴는 열린 채), 그룹을 고른 채면 "그룹에서 빼기".
export function openConvMenu(convId, anchor) {
  const items = () => {
    const out = [el("p", { class: "menu-head" }, ico("users"), el("span", { text: "그룹" }))];
    if (!state.groups.length) out.push(el("p", { class: "hint", text: "그룹 없음" }));
    for (const g of state.groups) {
      const on = g.convs.includes(convId);
      out.push(
        el("button", {
          type: "button",
          role: "menuitemcheckbox",
          "aria-checked": String(on),
          onclick: async (ev) => {
            const b = ev.currentTarget;
            await changeGroup(g.id, on ? [] : [convId], on ? [convId] : []);
            if (!$("float").hidden && floatFrom === anchor) {
              $("float").replaceChildren(...items());
              ($("float").querySelector(`[data-g="${g.id}"]`) ?? b).focus?.();
            }
          },
          "data-g": String(g.id),
        }, ico("users"), el("span", { text: g.name })),
      );
    }
    const g = railGroupOf();
    if (g?.convs.includes(convId))
      out.push(
        el("button", {
          type: "button",
          role: "menuitem",
          class: "danger-text",
          onclick: async () => {
            closeFloat(false);
            if (await changeGroup(g.id, [], [convId])) $("status_msg").textContent = `그룹 ${g.name} 에서 뺐습니다`;
            focusAfterFloat("#conv_sections li[data-conv] a", "group_pick_btn");
          },
        }, ico("minus"), el("span", { text: "그룹에서 빼기" })),
      );
    // 기록 삭제 ▸ · 나가기 — 확인 한 번. 하위 메뉴는 그 자리에서 펼친다(→ 펼침 · ← 접음).
    const conv = state.convs.find((c) => c.id === convId) ?? (state.current?.id === convId ? state.current : null);
    if (conv) {
      out.push(el("hr", { class: "menu-sep" }));
      const confirm = (key) => {
        $("float").replaceChildren(convEndForm(conv, key));
        $("float").setAttribute("role", "dialog");
        $("float").querySelector("button")?.focus();
      };
      for (const it of convMenuItems(conv)) {
        if (!it.sub) {
          out.push(
            el("button", { type: "button", role: "menuitem", class: it.danger ? "danger-text" : null, onclick: () => confirm(it.key) }, ico(it.icon), el("span", { text: it.label })),
          );
          continue;
        }
        const subs = it.sub.map((x) => el("button", { type: "button", role: "menuitem", class: "menu-sub danger-text", hidden: true, onclick: () => confirm(x.key) }, el("span", { text: x.label })));
        const toggle = (open) => {
          head.setAttribute("aria-expanded", String(open));
          for (const b of subs) b.hidden = !open;
          // 펼쳐 길어진 상자가 화면 아래로 넘치면 위로 올린다(`openFloat` 와 같은 여백).
          const box = $("float").getBoundingClientRect();
          if (box.bottom > window.innerHeight - 8) $("float").style.top = Math.max(8, window.innerHeight - box.height - 8) + "px";
          if (open) subs[0].focus();
        };
        const head = el(
          "button",
          {
            type: "button",
            role: "menuitem",
            "aria-haspopup": "menu",
            "aria-expanded": "false",
            onclick: () => toggle(head.getAttribute("aria-expanded") !== "true"),
            onkeydown: (ev) => {
              if (ev.key === "ArrowRight") toggle(true);
            },
          },
          ico(it.icon),
          el("span", { text: it.label }),
          el("span", { class: "kbd", "aria-hidden": "true", text: "▸" }),
        );
        for (const b of subs)
          b.addEventListener("keydown", (ev) => {
            if (ev.key !== "ArrowLeft") return;
            toggle(false);
            head.focus();
          });
        out.push(head, ...subs);
      }
    }
    return out;
  };
  openFloat(anchor, "menu", "대화 메뉴", items());
}

/// 확인 문구 — 나가기 · 기록 삭제 둘(내 글만 · 모든 사용자). `[머리, 안내, 단추]`.
export const END_TEXT = {
  leave: ["대화 나가기", LEAVE_CONFIRM, "나가기"],
  clear_mine: ["기록 삭제 · 내 글만", "이 PC 의 대화 기록을 지웁니다. 대화는 남습니다.", "삭제"],
  clear_all: ["기록 삭제 · 모든 사용자", "나와 상대 PC 의 대화 기록을 모두 지웁니다. 대화는 남습니다.", "삭제"],
};

/// 나가기 · 기록 삭제 확인. 나가기 = 나가며 이 PC 목록 · 글 삭제(`DELETE` — 지금 삭제), 기록 삭제 = 글만(`POST …/clear`). 지운 대화가 열려 있었으면 '개요'로.
export function convEndForm(conv, key) {
  const [head, hint, label] = END_TEXT[key];
  const title = convLabel(conv, state.peerNames).title;
  return el(
    "div",
    { class: "float-form" },
    el("strong", { text: `${head} · ${title}` }),
    el("p", { class: "hint", text: hint }),
    el(
      "div",
      { class: "actions" },
      button("취소", () => closeFloat(true)),
      button(
        label,
        () => {
          closeFloat(false);
          act(
            label,
            async () => {
              if (key === "leave") {
                await api("DELETE", `/api/conversations/${conv.id}`);
                if (state.current?.id === conv.id) {
                  state.current = null;
                  go("#");
                }
                await loadGroups();
                return "대화에서 나갔습니다";
              }
              await api("POST", `/api/conversations/${conv.id}/clear`, { all: key === "clear_all" });
              return key === "clear_all" ? "모든 사용자의 대화 기록을 지웠습니다" : "이 PC 의 대화 기록을 지웠습니다";
            },
            "status_msg",
            refresh,
          );
        },
        "danger",
      ),
    ),
  );
}

$("conv_more_btn").addEventListener("click", (ev) => {
  if (state.current) openConvMenu(state.current.id, ev.currentTarget);
});

/// 그룹 "대화 추가" — 제목 · 상대 검색 + 모든 대화 체크 목록(지금 소속은 체크된 채) → "적용" 으로 넣고 빼기 한 번에.
export function openGroupPicker(anchor) {
  const g = railGroupOf();
  if (!g) return;
  const picked = new Set(g.convs);
  const q = el("input", { type: "search", placeholder: "제목 · 상대 검색", "aria-label": "대화 검색", autocomplete: "off" });
  const list = el("ul", { class: "pick-list" });
  const draw = () => {
    const needle = q.value.trim().toLowerCase();
    const rows = sortByPos(state.convs)
      .map((c) => ({ c, label: convLabel(c, state.peerNames) }))
      .filter(({ label }) => !needle || `${label.title} ${label.sub}`.toLowerCase().includes(needle));
    list.replaceChildren(
      ...(rows.length
        ? rows.map(({ c, label }) => {
            const box = el("input", { type: "checkbox", checked: picked.has(c.id) });
            box.addEventListener("change", () => (box.checked ? picked.add(c.id) : picked.delete(c.id)));
            return el("li", {}, el("label", {}, box, el("span", { text: label.title }), el("span", { class: "hint", text: label.sub })));
          })
        : [el("li", { class: "hint", text: "맞는 대화 없음" })]),
    );
  };
  q.addEventListener("input", draw);
  draw();
  const apply = async () => {
    const d = membershipDiff(g.convs, [...picked]);
    closeFloat(true);
    if (!d.add.length && !d.remove.length) return;
    if (await changeGroup(g.id, d.add, d.remove)) $("status_msg").textContent = `그룹 ${g.name} · 넣음 ${d.add.length} · 뺌 ${d.remove.length}`;
    focusAfterFloat("#group_pick_btn");
  };
  openFloat(anchor, "dialog", `그룹 ${g.name} 에 대화 추가`, [
    el("div", { class: "float-form picker" }, el("strong", { text: `대화 추가 · ${g.name}` }), q, list, el("div", { class: "actions" }, button("취소", () => closeFloat(true)), button("적용", apply, "primary"))),
  ]);
}

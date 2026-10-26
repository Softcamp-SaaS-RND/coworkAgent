// 설정 · 그룹설정.
import { $, button, el, lead, needsRender } from "../core/dom.js";
import { openFloat } from "../core/float.js";
import { state } from "../core/state.js";
import { rowTile } from "../core/widgets.js";
import { convLabel } from "../logic/conv.js";
import { displayName } from "../logic/format.js";
import { sortByPos } from "../logic/order.js";
import { groupSettingsList } from "../logic/rail.js";
import { changeGroup, openGroupNew, openGroupRename } from "../rail.js";
import { convAct, convLine, convSig } from "./conn.js";
import { mdFoot, mdHead, mdRow, pick, setPick } from "./index.js";
import { groupDeleteForm } from "../tiles.js";

/// 그룹설정 — 왼쪽 그룹 목록(검색 = 그룹 이름 · 그 그룹 대화 제목 · 사용자), 오른쪽 상세(이 그룹의 대화 · 넣을 수 있는 대화).
export function renderGroupPage(force) {
  const box = $("groups");
  const groupSig = state.groups.map((g) => [g.id, g.name, g.icon, g.color, g.convs]);
  const peerSig = state.peers.map((p) => [p.id, displayName(p), p.host, p.icon, p.color]);
  const sig = JSON.stringify([groupSig, peerSig, convSig(), $("group_q").value, pick.groups]);
  if (!needsRender(box, sig, force)) return;
  if (pick.groups != null && !state.groups.some((g) => g.id === pick.groups)) pick.groups = null; // 지운 그룹
  const q = $("group_q").value;
  const convs = sortByPos(state.convs);
  const list = groupSettingsList(state.groups, state.peers, convs, q, state.peerNames);
  $("group_list").replaceChildren(
    ...list.groups.map((g) => mdRow("groups", g.id, rowTile("small", g.icon, g.color, true), g.name, `대화 ${g.count}`)),
    ...(list.groups.length ? [] : [el("li", { class: "hint md-none", text: q ? "맞는 그룹 없음" : "그룹 없음 — 그룹 추가" })]),
  );
  const g = state.groups.find((x) => x.id === pick.groups);
  $("group_detail").replaceChildren(...(g ? groupDetail(g, convs) : [el("p", { class: "md-empty", text: "그룹을 고르세요" })]));
}

/// 그룹 상세 — 이 그룹의 대화(빼기) · 넣을 수 있는 대화(사용자별 · 넣기) · 이름 바꾸기 · 삭제 · 닫기. 넣기는 복사(다른 그룹 · 원래 자리 그대로).
export function groupDetail(g, convs) {
  const inside = convs.filter((c) => g.convs.includes(c.id));
  const byUser = state.peers
    .map((p) => [p, convs.filter((c) => c.peer_id === p.id && !g.convs.includes(c.id))])
    .filter(([, cs]) => cs.length);
  const say = (t) => ($("group_msg").textContent = t);
  const title = (c) => convLabel(c, state.peerNames).title;
  return [
    mdHead("groups", rowTile("", g.icon, g.color, true), g.name, el("span", { class: "hint", text: `대화 ${inside.length}` })),
    el(
      "div",
      { class: "md-body md-cols" },
      el(
        "section",
        { class: "md-col", "aria-label": "이 그룹의 대화" },
        el("strong", { class: "md-sec" }, lead("chat"), "이 그룹의 대화"),
        inside.length
          ? el(
              "ul",
              { class: "md-convs" },
              ...inside.map((c) =>
                convLine(
                  c,
                  convAct(
                    "빼기",
                    title(c),
                    async () => {
                      if (await changeGroup(g.id, [], [c.id])) say(`그룹 ${g.name} 에서 뺐습니다`);
                    },
                    "minus",
                  ),
                ),
              ),
            )
          : el("p", { class: "hint", text: "대화 없음 — 넣을 수 있는 대화에서 넣기" }),
      ),
      el(
        "section",
        { class: "md-col", "aria-label": "넣을 수 있는 대화" },
        el("strong", { class: "md-sec" }, lead("list"), "넣을 수 있는 대화"),
        ...(byUser.length
          ? byUser.map(([p, cs]) =>
              el(
                "div",
                { class: "md-user-convs" },
                el("span", { class: "md-user", text: displayName(p) }),
                el(
                  "ul",
                  { class: "md-convs" },
                  ...cs.map((c) =>
                    convLine(
                      c,
                      convAct(
                        "넣기",
                        title(c),
                        async () => {
                          if (await changeGroup(g.id, [c.id], [])) say(`그룹 ${g.name} 에 넣었습니다`);
                        },
                        "plus",
                      ),
                    ),
                  ),
                ),
              ),
            )
          : [el("p", { class: "hint", text: "넣을 대화 없음" })]),
      ),
    ),
    mdFoot(
      "groups",
      button("이름 바꾸기", (ev) => openGroupRename(g.id, ev.currentTarget), undefined, "pencil"),
      button("삭제", (ev) => openFloat(ev.currentTarget, "dialog", "그룹 삭제", [groupDeleteForm(g.id)]), "danger", "trash"),
    ),
  ];
}

$("group_q").addEventListener("input", () => renderGroupPage(true));
$("peer_group_add").addEventListener("click", (ev) =>
  openGroupNew(ev.currentTarget, (g) => {
    setPick("groups", g.id); // 만든 그룹을 고른다
    $("group_msg").textContent = `그룹 ${g.name} 을 만들었습니다`;
  }),
);

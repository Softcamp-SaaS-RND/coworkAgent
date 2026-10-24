// 내 활동 · 북마크 목록.
import { api } from "./core/api.js";
import { $, el, ico, needsRender } from "./core/dom.js";
import { latest, state } from "./core/state.js";
import { speakerEl } from "./core/widgets.js";
import { ACT_KINDS, activityRows } from "./logic/activity.js";
import { formatTime } from "./logic/format.js";
import { inboxRows } from "./logic/inbox.js";
import { scopeQuery } from "./logic/rail.js";
import { grouped, loadCounts } from "./sidebar.js";

export function rowEls(rows, empty) {
  if (!rows.length) return [el("li", { class: "hint", text: empty })];
  return rows.map((r) =>
    el(
      "li",
      {},
      el(
        "a",
        { href: r.href },
        el(
          "span",
          { class: "meta" },
          r.author ? speakerEl(r.author) : "",
          el("strong", { text: r.who }),
          el("span", { text: formatTime(r.at) }),
          r.room ? el("span", { class: "chip", text: r.room }) : "",
          r.kind ? el("span", { class: "tag ask", text: r.kind }) : "",
        ),
        el("span", { text: r.preview }),
      ),
    ),
  );
}

/// 내 활동 행 — 종류 라벨 · 발화자 · 시각 · 대화 제목 · 본문 앞부분, 안 본 항목은 강조.
export function activityEls(rows) {
  if (!rows.length) return [el("li", { class: "hint", text: "내 활동 없음" })];
  return rows.map((r) =>
    el(
      "li",
      { class: r.unseen ? "unseen" : null },
      el(
        "a",
        { href: r.href },
        el(
          "span",
          { class: "meta" },
          el("span", { class: `tag act-${r.tone}`, text: r.label }),
          r.author ? speakerEl(r.author) : "",
          el("strong", { text: r.who }),
          el("span", { text: formatTime(r.at) }),
          el("span", { class: "hint", text: r.title }),
        ),
        r.preview ? el("span", { text: r.preview }) : "",
        r.note ? el("span", { class: "hint keep-lines", text: `의견: ${r.note}` }) : "",
      ),
    ),
  );
}

/// 내 활동 — '나' 에만(상대별 절). 열 때(`open`) 거르기를 처음으로 되돌리고(본 것으로 표시하는 범위 = 보이는 범위) 전부를 본 것으로(탭이 보일 때만).
/// 그때 안 본 항목은 다시 그려도 강조를 남긴다. 여는 동안의 새로 고침은 건너뛴다(늦게 온 것으로 버려져 표시를 놓치지 않게).
export async function loadActivity(open = false) {
  if (!open && state.actOpening) return;
  if (open) {
    state.actOpening = true;
    state.actKind = "all";
    $("act_q").value = "";
  }
  renderActTabs();
  try {
    const params = new URLSearchParams({ kind: state.actKind });
    if ($("act_q").value.trim()) params.set("q", $("act_q").value.trim());
    const ticket = latest.next();
    let r;
    try {
      r = await api("GET", "/api/activity?" + params);
    } catch (e) {
      $("status_msg").textContent = "내 활동 오류: " + e.message;
      return;
    }
    if (!latest.is(ticket)) return;
    const rows = activityRows(r.items, state.peerNames);
    if (open) state.actFresh = new Set(rows.filter((x) => x.unseen).map((x) => x.key));
    for (const x of rows) x.unseen ||= state.actFresh.has(x.key);
    if (needsRender($("act_list"), JSON.stringify([rows, state.peerNames.size]), open)) $("act_list").replaceChildren(...grouped("activity", rows, activityEls));
    if (open && document.visibilityState === "visible") {
      // 받은 범위(`upto`)까지만 — 받은 뒤 온 것은 안 본 것으로 남게. `peer` 가 없으니 전부.
      await api("POST", "/api/activity/seen", r.upto).catch(() => {});
      await loadCounts();
    }
  } finally {
    if (open) state.actOpening = false;
  }
}

export let actTimer;
$("act_q").addEventListener("input", () => {
  clearTimeout(actTimer);
  actTimer = setTimeout(() => loadActivity(), 250);
});

/// 내 활동 거르기 탭 — 전체 · 받은 요청 · 내 요청 · 받은 답 · 실패, 아이콘 + 이름.
export function renderActTabs() {
  $("act_tabs").replaceChildren(
    ...ACT_KINDS.map((k) =>
      el("a", {
        href: "#activity",
        "data-kind": k.kind,
        "aria-current": k.kind === state.actKind ? "page" : "false",
        onclick: (ev) => {
          ev.preventDefault();
          state.actKind = k.kind;
          loadActivity();
        },
      }, ico(k.icon), el("span", { text: k.label })),
    ),
  );
}

/// 북마크 항목 — 고른 타일 범위, 북마크한 순 최신부터, 누르면 그 글로.
export async function loadSaved() {
  const ticket = latest.next();
  let ms;
  try {
    const scope = scopeQuery(state.railPeer, state.railGroup);
    ms = await api("GET", "/api/saved" + (scope ? `?${scope}` : ""));
  } catch (e) {
    $("status_msg").textContent = "북마크 오류: " + e.message;
    return;
  }
  if (!latest.is(ticket)) return;
  const rows = inboxRows(ms, state.peerNames).map((r) => ({ ...r, href: `${r.href}/m/${r.id}` }));
  if (needsRender($("saved_list"), JSON.stringify([state.railPeer, state.railGroup, rows]), false))
    $("saved_list").replaceChildren(...grouped("saved", rows, (rs) => rowEls(rs, "북마크 없음")));
}

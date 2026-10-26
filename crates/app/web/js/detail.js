// 세부정보 — 이 PC 실행 · 진행 · 대화 정보 · 관람자 · 요청 · 북마크.
import { api } from "./core/api.js";
import { $, button, el, lead, unchanged } from "./core/dom.js";
import { state } from "./core/state.js";
import { authorLabel } from "./logic/author.js";
import { INFO_CMD_HINT, convInfoRows } from "./logic/conv.js";
import { formatTime, preview } from "./logic/format.js";
import { runSteps, timeline } from "./logic/message.js";
import { askTag, reqFold } from "./logic/request.js";
import { runsSummary } from "./logic/sync.js";
import { renderShare } from "./conv.js";
import { go } from "./shell.js";

/// 실행 상세 열고 닫기(`state.detailOpen`) — 대화 화면일 때만 보인다.
export function showDetail() {
  $("detail").hidden = !(state.detailOpen && !$("conversation").hidden);
  $("detail_btn").setAttribute("aria-pressed", String(state.detailOpen));
  renderDetail();
  if (!$("detail").hidden) loadRuns();
}

/// 이 PC 실행 문(`GET /api/runs`) — 실행 상세가 열려 있을 때만 읽는다(새로 고침 주기).
export async function loadRuns() {
  try {
    state.runs = await api("GET", "/api/runs");
  } catch (e) {
    state.runs = null;
    $("status_msg").textContent = "실행 목록 오류: " + e.message;
  }
  renderRuns();
}

export function renderRuns() {
  const r = state.runs ? runsSummary(state.runs) : null;
  $("detail_runs_n").textContent = r ? r.title : "";
  if (unchanged($("detail_runs"), JSON.stringify(r))) return;
  $("detail_runs").replaceChildren(
    ...(!r
      ? [el("li", { class: "hint", text: "불러오는 중 …" })]
      : r.rows.length
        ? r.rows.map((x) =>
            el(
              "li",
              {},
              el(
                "a",
                {
                  href: x.href || "#",
                  onclick: (ev) => {
                    ev.preventDefault();
                    if (x.href) go(x.href);
                  },
                },
                el("span", { class: "sub", text: x.label }),
                el("span", { class: "text", text: preview(x.text) }),
              ),
            ),
          )
        : [el("li", { class: "hint", text: "실행 · 대기 없음" })]),
  );
}

export function stepEl(s) {
  return el(
    "li",
    { class: s.state },
    el("div", { class: "t", text: s.title }),
    s.at ? el("div", { class: "at", text: s.at }) : "",
    s.meta ? el("div", { class: "meta", text: s.meta }) : "",
  );
}

export function renderDetail() {
  const conv = state.current;
  if ($("detail").hidden || !conv) return;
  // 이 대화의 글을 받기 전에는 이전 대화의 내용을 남기지 않는다.
  const ready = state.msgsOf === conv.id;
  const ms = ready ? state.msgs : [];
  // 진행 — 고른 글의 요청(그 답을 골라도 그 요청), 없으면 마지막 요청. 고른 요청은 대화에서 표시.
  const items = timeline(ms);
  const item = items.find((i) => i.request.id === state.pickMsg || i.reply?.id === state.pickMsg) ?? items.at(-1);
  const steps = !ready ? "불러오는 중 …" : item ? runSteps(item) : "요청 없음";
  if (!unchanged($("detail_steps"), JSON.stringify([conv.id, steps])))
    $("detail_steps").replaceChildren(...(typeof steps === "string" ? [el("li", { class: "hint", text: steps })] : steps.map(stepEl)));
  const picked = state.pickMsg != null && item ? item.request.id : null;
  for (const a of $("messages").querySelectorAll(".msg.picked")) if (a.dataset.msg !== String(picked)) a.classList.remove("picked");
  if (picked != null) $("messages").querySelector(`[data-msg="${picked}"]`)?.classList.add("picked");
  renderInfo(conv, ms);
  renderShare(conv);
  // 요청 — 이 대화의 요청 글(최근 글부터, 상태 · 메모). 최근 몇 건만, "더보기" 로 펼침 — 다른 대화로 옮기면 다시 접는다.
  const peerName = state.peerNames.get(conv.peer_id);
  const reqs = ms.filter((m) => m.author === "human" && m.request).reverse();
  if (reqView.conv !== conv.id) [reqView.conv, reqView.open] = [conv.id, false];
  const fold = reqFold(reqs, reqView.open);
  $("detail_req_n").textContent = reqs.length ? String(reqs.length) : "";
  if (!unchanged($("detail_req"), JSON.stringify([conv.id, ready, peerName, reqView.open, reqs.map((m) => [m.id, m.req_state, m.req_note, m.text.length])])))
    $("detail_req").replaceChildren(
      ...(reqs.length
        ? fold.shown.map((m) => detailItem(conv, m, [askTag(m), m.req_note, authorLabel(conv, m, peerName), formatTime(m.created_at)].filter(Boolean).join(" · ")))
        : [el("li", { class: "hint", text: !ready ? "불러오는 중 …" : "요청 없음" })]),
      ...(fold.toggle ? [el("li", { class: "req-more" }, button(fold.toggle, toggleReqs, "small"))] : []),
    );
  // 북마크 항목 — 이 대화에서 북마크한 글(최근 글부터).
  const saved = ms.filter((m) => m.saved).reverse();
  $("detail_saved_n").textContent = saved.length ? String(saved.length) : "";
  if (unchanged($("detail_saved"), JSON.stringify([conv.id, ready, peerName, saved.map((m) => [m.id, m.text.length])]))) return;
  $("detail_saved").replaceChildren(
    ...(saved.length
      ? saved.map((m) => detailItem(conv, m, `${authorLabel(conv, m, peerName)} · ${formatTime(m.created_at)}`))
      : [el("li", { class: "hint", text: ready ? "북마크 없음" : "불러오는 중 …" })]),
  );
}

/// 세부정보 '요청' 펼침 — 어느 대화에서 펼쳤나(대화를 옮기면 접힌다).
export const reqView = { conv: null, open: false };

/// 세부정보 '요청' 더보기 · 접기.
export function toggleReqs() {
  reqView.open = !reqView.open;
  renderDetail();
}

/// 세부정보 '대화 정보' — 대화 · 글 수 · 마지막 글 상태가 바뀔 때만 `GET …/info` 를 다시 읽는다(실행이 끝나면 명령 · 시각이 바뀐다).
/// 응답이 오기 전 = "불러오는 중 …", 그 사이 다른 대화로 옮겼으면 버린다.
export const infoView = { key: "", gen: 0, conv: null, data: null };
export function renderInfo(conv, ms) {
  const last = ms.at(-1);
  const key = JSON.stringify([conv.id, ms.length, last?.id, last?.state]);
  if (infoView.key !== key) {
    infoView.key = key;
    if (infoView.conv !== conv.id) [infoView.conv, infoView.data] = [conv.id, null];
    // 응답은 마지막 읽기 것만 — 키가 같아도(A → B → A) 앞선 읽기의 늦은 응답은 버린다.
    const gen = ++infoView.gen;
    api("GET", `/api/conversations/${conv.id}/info`)
      .then((d) => {
        if (infoView.gen !== gen) return;
        infoView.data = d;
        drawInfo(conv);
      })
      .catch((e) => {
        if (infoView.gen !== gen) return;
        infoView.key = ""; // 실패한 읽기는 다음 그리기(5초 새로 고침 · 다시 열기)에서 다시
        $("info_rows").replaceChildren(el("dd", { class: "info-hint", text: "대화 정보 오류: " + e.message }));
        $("info_rows").dataset.sig = "";
      });
  }
  drawInfo(conv);
}

export function drawInfo(conv) {
  const d = infoView.conv === conv.id ? infoView.data : null;
  const box = $("info_rows");
  if (unchanged(box, JSON.stringify([conv.id, conv.dir, conv.relay_blocked, d]))) return;
  if (!d) {
    box.replaceChildren(el("dd", { class: "info-hint", text: "불러오는 중 …" }));
    return;
  }
  const rows = convInfoRows({ ...d, relay_blocked: conv.relay_blocked }, conv.dir);
  box.replaceChildren(
    ...rows.flatMap((r) => [
      el("dt", {}, lead(r.icon), r.label),
      r.copy
        ? el(
            "dd",
            { class: "info-cmd" },
            el("code", { class: "mono", title: r.text, text: r.text }),
            el("button", { type: "button", class: "small", onclick: (ev) => copyLine(r.copy, ev.currentTarget) }, lead("copy"), "복사"),
          )
        : el("dd", { class: r.mono ? "mono" : null, text: r.text }),
    ]),
    ...(rows.some((r) => r.copy) ? [el("dd", { class: "info-hint", text: INFO_CMD_HINT })] : []),
  );
}

/// 실행 명령 복사 — 전체 명령을 클립보드로, 결과는 상태 줄(`#status_msg`)에.
export async function copyLine(text, btn) {
  try {
    await navigator.clipboard.writeText(text);
    $("status_msg").textContent = "실행 명령을 복사했습니다";
    btn.classList.add("copied");
    setTimeout(() => btn.classList.remove("copied"), 1500);
  } catch (e) {
    $("status_msg").textContent = "복사 실패: " + e.message;
  }
}

/// 실행 상세 목록의 글 한 줄 — 위 줄 `sub`(발화자 · 시각 …) · 본문 앞부분, 누르면 그 글로(`#c/{대화}/m/{글}`).
export function detailItem(conv, m, sub) {
  const href = `#c/${conv.id}/m/${m.id}`;
  return el(
    "li",
    {},
    el(
      "a",
      {
        href,
        onclick: (ev) => {
          ev.preventDefault();
          go(href);
        },
      },
      el("span", { class: "sub", text: sub }),
      el("span", { class: "text", text: preview(m.text) }),
    ),
  );
}

// 머리글 "북마크 n" — 실행 상세를 열고 그 절로.
$("saved_btn").addEventListener("click", () => {
  state.detailOpen = true;
  showDetail();
  $("detail_saved_sec").scrollIntoView({ block: "start" });
});

$("detail_btn").addEventListener("click", () => {
  state.detailOpen = !state.detailOpen;
  showDetail();
});
$("detail_close").addEventListener("click", () => {
  state.detailOpen = false;
  showDetail();
});

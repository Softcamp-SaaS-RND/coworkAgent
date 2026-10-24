// 설정 · 에이전트 — 목록 · 상세 · 등록.
import { api } from "../core/api.js";
import { $, act, button, el, field, lead, needsRender } from "../core/dom.js";
import { state } from "../core/state.js";
import { agentTile } from "../core/widgets.js";
import { AGENT_COLORS, CODEX_READ_NOTE, PARENT_HINT, PARENT_NOTE, SHELL_HINT, SHELL_NOTE, SHELL_ON, SHELL_WARN, SKIP_GIT_HINT, TOOLS_NOTE, agentColor, agentEngineBlocks, agentPatch, engineLabel, shellConfirmNeeded } from "../logic/agent.js";
import { draftOf, pruneDrafts } from "../logic/settings.js";
import { route } from "../logic/shell.js";
import { renderAgentsView } from "../agents.js";
import { syncRelayCap } from "../compose.js";
import { closePick, mdFoot, mdHead, mdRow, mdRowId, pick, setPick } from "./index.js";

export async function loadAgents() {
  try {
    state.agents = await api("GET", "/api/agents");
  } catch (e) {
    $("agent_msg").textContent = "에이전트 목록 오류: " + e.message;
    return;
  }
  // 세션 생성의 "내 응답 에이전트" 후보.
  const relaySel = $("chat_relay_agent");
  const keep = relaySel.value;
  relaySel.replaceChildren(
    el("option", { value: "", text: "안 함" }),
    ...state.agents.map((a) => el("option", { value: a.name, text: a.name })),
  );
  if (state.agents.some((a) => a.name === keep)) relaySel.value = keep;
  syncRelayCap();
  renderAgentList(true);
  if (route(location.hash).view === "agents") renderAgentsView();
}

/// 에이전트 상세의 저장 전 값(다른 줄을 골랐다 돌아와도 남게, 저장 성공이면 지운다). 에이전트 id → `{owner, folder, prompt, parent_read, shell, color, skip_git}`(`owner` = 그 에이전트 uid — 같은 id 를 다시 받은 새 에이전트에 옛 값이 붙지 않게).
export const agentDrafts = new Map();

/// 에이전트 — 왼쪽 목록(로봇 타일 · 이름 · 실행 도구 · 폴더 · 상위 탐색 · 셸 표시 — 검색 칸 없음), 오른쪽 상세(한 격자) · 새 에이전트 폼(2단계).
export function renderAgentList(force = false) {
  const box = $("agents");
  const sig = JSON.stringify([state.agents, pick.agents]);
  if (!needsRender(box, sig, force)) return;
  if (typeof pick.agents === "number" && !state.agents.some((a) => a.id === pick.agents)) pick.agents = null; // 지운 에이전트
  pruneDrafts(agentDrafts, state.agents.map((a) => a.id));
  const shown = state.agents;
  $("agent_groups").replaceChildren(
    ...shown.map((a) =>
      mdRow(
        "agents",
        a.id,
        agentTile(a.color, "small"),
        a.name,
        `${engineLabel(a.engine)} · ${a.folder}`,
        ...(a.parent_read ? [el("span", { class: "chip", text: "상위 탐색" })] : []),
        ...(a.shell ? [el("span", { class: "chip", text: "셸" })] : []),
      ),
    ),
    ...(shown.length ? [] : [el("li", { class: "hint md-none", text: "등록된 에이전트 없음" })]),
  );
  const mode = pick.agents;
  $("agent_empty").hidden = mode != null;
  $("agent_new").hidden = mode !== "new";
  $("agent_new_btn").setAttribute("aria-expanded", String(mode === "new"));
  const a = typeof mode === "number" ? state.agents.find((x) => x.id === mode) : null;
  $("agent_pane").hidden = !a;
  $("agent_pane").replaceChildren(...(a ? agentDetail(a) : []));
}

/// 에이전트 상세(한 격자 · 오른쪽 칸) — 이름 · 실행 도구(읽기 전용) · 아이콘 색 · 폴더 · 좌우 옵션 블록(Claude Code — 상위 탐색 · 셸 · 도구,
/// Codex — Git 확인 건너뜀 · 읽기 범위, 안 쓰는 도구 블록은 비활성) · 추가 지시, 저장 · 삭제 · 닫기.
export function agentDetail(a) {
  const d = draftOf(agentDrafts, a.id, a.uid) ?? {};
  const folder = el("input", { value: d.folder ?? a.folder, class: "mono" });
  const parent = el("input", { type: "checkbox", role: "switch", title: PARENT_NOTE });
  parent.checked = d.parent_read ?? a.parent_read;
  const shell = el("input", { type: "checkbox", role: "switch", title: SHELL_NOTE });
  shell.checked = d.shell ?? a.shell;
  const skip = el("input", { type: "checkbox", role: "switch" });
  skip.checked = d.skip_git ?? a.skip_git ?? true;
  const prompt = el("textarea", { rows: "5" });
  prompt.value = d.prompt ?? a.prompt;
  const promptBox = el("div", { class: "field" }, prompt);
  // 아이콘 색 — 고르면 단추 표시만, 저장 때 보낸다.
  let color = agentColor(d.color ?? a.color);
  const keep = () =>
    agentDrafts.set(a.id, { owner: a.uid, folder: folder.value, prompt: prompt.value, parent_read: parent.checked, shell: shell.checked, color, skip_git: skip.checked });
  folder.addEventListener("input", keep);
  prompt.addEventListener("input", keep);
  parent.addEventListener("change", keep);
  shell.addEventListener("change", keep);
  skip.addEventListener("change", keep);
  const blocks = agentEngineBlocks(a.engine ?? "claude");
  const block = (legend, b, ...kids) =>
    el("fieldset", { class: "engine-block", disabled: b.disabled }, el("legend", {}, lead("gear"), legend), el("span", { class: "hint engine-off", text: b.note }), ...kids);
  const switchLine = (box, label, hint, icon) => [el("label", { class: "check-line" }, box, el("span", {}, lead(icon), label)), el("span", { class: "hint", text: hint })];
  const swatches = el(
    "span",
    { class: "swatches", role: "group", "aria-label": "아이콘 색" },
    ...AGENT_COLORS.map((c) =>
      el("button", {
        type: "button",
        class: `swatch ${c}`,
        title: `아이콘 색 ${c}`,
        "aria-label": `아이콘 색 ${c}`,
        "aria-pressed": String(c === color),
        onclick: (ev) => {
          color = c;
          for (const b of swatches.children) b.setAttribute("aria-pressed", String(b === ev.currentTarget));
          keep();
        },
      }),
    ),
  );
  return [
    mdHead("agents", agentTile(a.color), a.name, el("span", { class: "hint mono", text: a.folder })),
    el(
      "div",
      { class: "form-grid md-body" },
      el("span", { class: "label" }, lead("title"), "이름"),
      el("span", { class: "check-line" }, el("strong", { text: a.name }), el("span", { class: "hint", text: "등록 뒤 바꿀 수 없음" })),
      el("span", { class: "label" }, lead("bot"), "실행 도구"),
      el(
        "span",
        { class: "check-line" },
        el("span", { class: "chip engine-chip", "data-engine": a.engine ?? "claude", text: engineLabel(a.engine) }),
        el("span", { class: "hint", text: "등록 뒤 바꿀 수 없음 — 바꾸려면 새로 등록" }),
      ),
      el("span", { class: "label" }, lead("palette"), "아이콘 색"),
      swatches,
      ...field("폴더(절대 경로)", folder, folder, "folder"),
      ...(a.parent_read
        ? [el("span", { class: "label hint", text: "읽는 위 폴더" }), el("span", { class: "hint mono", text: a.parent_dir ?? "열지 않음(드라이브 루트 · 사용자 홈 · 데이터 폴더를 품음)" })]
        : []),
      ...(a.shell ? [el("span", { class: "label hint", text: "셸" }), el("span", { class: "hint shell-on", text: SHELL_ON })] : []),
      el(
        "div",
        { class: "engine-blocks" },
        block(
          "Claude Code 옵션",
          blocks.claude,
          ...switchLine(parent, "상위 한단계 탐색 허용", PARENT_HINT, "folder-up"),
          ...switchLine(shell, "셸 명령 실행 허용", SHELL_HINT, "terminal"),
          el("span", { class: "hint", text: TOOLS_NOTE }),
        ),
        block(
          "Codex 옵션",
          blocks.codex,
          ...switchLine(skip, "Git 저장소 아닌 폴더 허용", SKIP_GIT_HINT, "git"),
          el("span", { class: "hint", text: CODEX_READ_NOTE }),
        ),
      ),
      ...field("추가 지시", prompt, promptBox, "message"),
    ),
    mdFoot(
      "agents",
      button(
        "저장",
        () => {
          // 셸을 켤 때 한 번 위험 확인 — 취소면 보내지 않는다.
          if (shellConfirmNeeded(a.shell, shell.checked) && !window.confirm(SHELL_WARN)) return;
          const body = agentPatch(a, { folder: folder.value, prompt: prompt.value, parent_read: parent.checked, shell: shell.checked, color, skip_git: skip.checked });
          act(
            "에이전트 저장",
            async () => {
              const sent = agentDrafts.get(a.id); // 응답을 기다리는 사이 또 고치면 새 객체라 남는다
              await api("PATCH", `/api/agents/${a.id}`, body);
              if (agentDrafts.get(a.id) === sent) agentDrafts.delete(a.id);
            },
            "agent_msg",
            loadAgents,
          );
        },
        "primary",
        "save",
      ),
      button(
        "삭제",
        () =>
          act(
            "삭제",
            async () => {
              await api("DELETE", `/api/agents/${a.id}`);
              agentDrafts.delete(a.id);
            },
            "agent_msg",
            loadAgents,
          ),
        "danger",
        "trash",
      ),
    ),
  ];
}

/// "에이전트 등록"(2단계) — 오른쪽 상세 칸에 새 에이전트 폼. 새로 열면 1단계(실행 도구 고르기), 등록 · 취소면 닫는다.
export function showAgentNew(on) {
  if (on && pick.agents !== "new") showNewEngine(null);
  if (on) setPick("agents", "new");
  else if (pick.agents === "new") setPick("agents", null);
}
$("agent_new_btn").addEventListener("click", () => {
  const on = pick.agents !== "new";
  showAgentNew(on);
  if (on) $("agent_engine_claude").focus();
});

/// 등록 1단계에서 고른 실행 도구 — null = 1단계(도구 타일), 고르면 2단계(폼 — 도구 칩 · 안 쓰는 도구 옵션 블록 비활성).
export let newEngine = null;
export function showNewEngine(engine) {
  newEngine = engine;
  $("agent_engine_step").hidden = !!engine;
  $("agent_step1_foot").hidden = !!engine;
  $("agent_form").hidden = !engine;
  $("agent_step2_foot").hidden = !engine;
  $("agent_new_step").textContent = engine ? "2 / 2 — 이름 · 폴더 · 옵션" : "1 / 2 — 실행 도구 고르기";
  const chip = $("agent_engine_chip");
  chip.textContent = engineLabel(engine);
  chip.dataset.engine = engine ?? "";
  const blocks = agentEngineBlocks(engine);
  for (const [k, id] of [["claude", "agent_claude_opts"], ["codex", "agent_codex_opts"]]) {
    $(id).disabled = blocks[k].disabled;
    $(id).querySelector(".engine-off").textContent = blocks[k].note;
  }
  // Codex 는 Claude 전용 옵션을 쓰지 않는다 — 켠 채 도구를 바꿔도 보내지 않게.
  if (engine === "codex") $("agent_parent").checked = $("agent_shell").checked = false;
}
for (const b of document.querySelectorAll(".engine-tile"))
  b.addEventListener("click", () => {
    showNewEngine(b.dataset.engine);
    $("agent_name").focus();
  });
$("agent_engine_back").addEventListener("click", () => {
  showNewEngine(null);
  $("agent_engine_claude").focus();
});

$("agent_new_cancel").addEventListener("click", () => closePick("agents"));
$("agent_engine_cancel").addEventListener("click", () => closePick("agents"));

$("agent_tools").textContent = TOOLS_NOTE;

$("agent_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  if (!newEngine) return; // 1단계에서 도구를 골라야 폼이 보인다(도구 없는 등록 없음)
  const codex = newEngine === "codex";
  const body = {
    name: $("agent_name").value,
    folder: $("agent_folder").value,
    prompt: $("agent_prompt").value,
    parent_read: !codex && $("agent_parent").checked,
    shell: !codex && $("agent_shell").checked,
    engine: newEngine,
    ...(codex ? { skip_git: $("agent_skip_git").checked } : {}),
  };
  // 셸을 켠 채 등록하면 한 번 위험 확인 — 취소면 보내지 않는다.
  if (shellConfirmNeeded(false, body.shell) && !window.confirm(SHELL_WARN)) return;
  act(
    "등록",
    async () => {
      const made = await api("POST", "/api/agents", body);
      $("agent_form").reset();
      showNewEngine(null);
      pick.agents = made?.id ?? null; // 등록한 에이전트를 고른다(목록은 loadAgents 가 다시 그린다)
      state.focusAfter = null;
    },
    "agent_msg",
    async () => {
      await loadAgents();
      $(pick.agents != null ? mdRowId("agents", pick.agents) : "agent_new_btn")?.focus(); // 숨긴 폼에 초점이 남지 않게
    },
  );
});

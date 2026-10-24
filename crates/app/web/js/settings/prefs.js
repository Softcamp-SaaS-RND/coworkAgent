// 설정값 — 실행 · 화면 · 시작 · 알림 칸.
import { api } from "../core/api.js";
import { $, act, el } from "../core/dom.js";
import { state } from "../core/state.js";
import { peerWeeksOptions } from "../logic/peer.js";
import { EFFORTS, MODELS } from "../logic/settings.js";
import { renderRuns } from "../detail.js";
import { applyHomeZoom } from "../home.js";
import { loadPeerMessages } from "../peers.js";
import { applyRailOrder, renderRail } from "../rail.js";
import { renderPeerList } from "./conn.js";
import { applyTheme, applyTileSize, renderAvatar } from "../shell.js";
import { renderSideHead, setSideWidth } from "../sidebar.js";

/// 설정 입력 — 입력 id = 설정 이름.
export const PREFS = [
  "notify_pair",
  "notify_request",
  "notify_reply",
  "notify_relay",
  "notify_preview",
  "relay_cap_default",
  "claude_model",
  "claude_effort",
  "ui_theme",
  "ui_rail_order",
  "ui_tile_size",
  "ui_peer_weeks",
  "notify_sound",
  "autostart",
  "auto_compact",
];

export function fillPrefs(s) {
  state.settings = { ...state.settings, ...s };
  for (const key of PREFS) {
    if (s[key] === undefined) continue;
    const input = $(key);
    if (input.type === "checkbox") input.checked = s[key] === true;
    else input.value = String(s[key]);
  }
  if (s.ui_theme !== undefined) applyTheme(s.ui_theme);
  if (s.ui_rail_order !== undefined) applyRailOrder(s.ui_rail_order);
  if (s.ui_tile_size !== undefined) applyTileSize(s.ui_tile_size);
  if (s.notify_paused !== undefined || s.focus !== undefined) $("bell").classList.toggle("paused", s.notify_paused === true || s.focus === true);
  if (s.me_alias !== undefined || s.me_icon !== undefined) {
    renderRail();
    renderSideHead();
  }
  if (s.me_alias !== undefined || s.me_icon !== undefined || s.focus !== undefined) renderAvatar();
  // 세션 생성의 상한 칸 처음 값.
  if (s.relay_cap_default !== undefined) $("chat_relay_cap").value = String(s.relay_cap_default);
}

$("claude_effort").replaceChildren(...EFFORTS.map((e) => el("option", { value: e, text: e })));
$("ui_peer_weeks").replaceChildren(...peerWeeksOptions().map((o) => el("option", { value: o.value, text: o.label })));
$("claude_model").replaceChildren(...MODELS.map((m) => el("option", { value: m, text: m })));

for (const key of PREFS) {
  const input = $(key);
  input.addEventListener("change", () => {
    const value = input.type === "checkbox" ? input.checked : input.type === "number" || "num" in input.dataset ? Number(input.value) : input.value.trim();
    input.disabled = true;
    act("설정", async () => {
      try {
        fillPrefs(await api("PUT", "/api/settings", { [key]: value }));
        if (key === "claude_model" || key === "claude_effort") renderPeerList(); // 상대별 칸의 "기본값(…)" 문구
        if (key === "ui_peer_weeks") {
          state.peerPages.clear(); // 받은 꼬리에 기간 밖 글이 남지 않게
          loadPeerMessages();
        }
        return "설정 저장";
      } catch (e) {
        // 저장이 실패하면 서버 값으로 되돌린다 — 화면과 실제 설정이 어긋나지 않게.
        fillPrefs(await api("GET", "/api/settings").catch(() => ({})));
        throw e;
      } finally {
        input.disabled = false;
      }
    });
  });
}

// 동시 실행 상한 — 설정 저장소가 아니라 실행 문(`PUT /api/runs`)에. 실패하면 서버 값으로 되돌린다.
$("max_runs").addEventListener("change", () => {
  const input = $("max_runs");
  input.disabled = true;
  act("동시 실행 상한", async () => {
    try {
      const r = await api("PUT", "/api/runs", { cap: Number(input.value) });
      input.value = String(r.cap);
      state.runs = r;
      renderRuns();
      return `동시 실행 상한 ${r.cap}`;
    } catch (e) {
      input.value = String((await api("GET", "/api/runs").catch(() => ({ cap: "" }))).cap);
      throw e;
    } finally {
      input.disabled = false;
    }
  });
});

export async function showSettings() {
  const box = $("notify_inbound");
  try {
    const s = await api("GET", "/api/settings");
    box.checked = s.notify_inbound === true;
    fillPrefs(s);
    // 폭은 시작할 때만 — 다른 설정 저장 응답(fillPrefs)이 막 끈 폭을 이전 값으로 되돌리지 않게.
    setSideWidth(s.ui_side_width ?? 272);
    // 접힘도 시작할 때만 — 그 뒤는 화면이 바꾼 값이 정본.
    state.folded = new Set(s.ui_folded ?? []);
    state.homeView = s.ui_home_view ?? "preview"; // 개요 보기도 시작할 때만
    applyHomeZoom(s.ui_home_zoom); // 미리보기 크기도 시작할 때만
    $("max_runs").value = String((await api("GET", "/api/runs")).cap);
  } catch (e) {
    $("status_msg").textContent = "설정 오류: " + e.message;
  }
  box.addEventListener("change", () => {
    const wanted = box.checked;
    // 브라우저 알림 권한은 사용자 동작(이 클릭) 안에서만 물을 수 있다. 거부 · 미지원이어도 설정은 저장하고 트레이 표시만 쓴다고 알린다.
    const permission =
      wanted && "Notification" in window
        ? Notification.permission === "default"
          ? Notification.requestPermission()
          : Promise.resolve(Notification.permission)
        : Promise.resolve(wanted ? "unsupported" : "off");
    box.disabled = true;
    act("알림 설정", async () => {
      try {
        const r = await api("PUT", "/api/settings", { notify_inbound: wanted });
        box.checked = r.notify_inbound === true;
        if (!box.checked) return "알림을 껐습니다";
        const p = await permission;
        return p === "granted"
          ? "알림을 켰습니다"
          : "알림을 켰습니다. 브라우저 알림을 쓸 수 없어 트레이에만 표시합니다.";
      } catch (e) {
        // 저장이 실패하면 서버 값으로 되돌린다 — 화면과 실제 알림 상태가 어긋나지 않게.
        box.checked = await api("GET", "/api/settings")
          .then((s) => s.notify_inbound === true)
          .catch(() => !wanted);
        throw e;
      } finally {
        box.disabled = false;
      }
    });
  });
}

// 설정 · 디버깅.
import { api } from "../core/api.js";
import { $, act, el } from "../core/dom.js";
import { state } from "../core/state.js";
import { fileSize, formatClock } from "../logic/format.js";
import { diagText, logLines } from "../logic/settings.js";

/// 디버깅 정보(`GET /api/debug`) — 진단 정보 복사가 쓴다.
export let debugInfo = null;

/// 디버깅 화면 — 파일 카드(버전 · 데이터 폴더 · DB · 로그: 경로 · 크기 · 복사)와 로그 끝 200줄. 화면을 열 때 · 새로 고침 때 읽는다.
export async function loadDebug() {
  try {
    const [info, log] = await Promise.all([api("GET", "/api/debug"), api("GET", "/api/debug/log")]);
    debugInfo = info;
    const file = (k) => info.files.find((f) => f.key === k);
    const card = (label, path, size, copyLabel) =>
      el(
        "div",
        { class: "set-tile debug-file" },
        el("span", { class: "set-tile-label" }, el("strong", { text: label }), el("span", { class: "hint mono", text: path })),
        size !== null ? el("span", { class: "hint", text: size }) : "",
        el("button", { type: "button", class: "small", text: "복사", "aria-label": copyLabel, onclick: () => debugCopy(path, `${label} 복사`) }),
      );
    $("debug_files").replaceChildren(
      card("버전", `${info.version} (${info.os} ${info.arch})`, null, "버전 복사"),
      card("데이터 폴더", info.data_dir, null, "데이터 폴더 경로 복사"),
      card("DB", file("db").path, fileSize(file("db").size), "DB 경로 복사"),
      card("로그", file("log").path, fileSize(file("log").size), "로그 경로 복사"),
    );
    debugLog = log.lines;
    $("debug_log_path").textContent = log.path;
    renderDebugLog();
    $("debug_msg").textContent = "";
  } catch (e) {
    $("debug_msg").textContent = "디버깅 정보 오류: " + e.message;
  }
}

/// 마지막으로 읽은 로그 줄.
export let debugLog = [];

/// 로그 상자 — 고른 수준만, 줄 앞 유닉스 초는 이 PC 시각(월-일 시:분:초)으로. 끝(최신)이 보이게 아래로.
export function renderDebugLog() {
  const day = (at) => {
    const d = new Date(at * 1000);
    return `${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")} ${formatClock(at)}`;
  };
  const rows = logLines(debugLog, $("debug_level").value);
  $("debug_log").textContent = rows.length ? rows.map((x) => (x.at != null ? `${day(x.at)} ${x.level} ${x.rest}` : x.rest)).join("\n") : "로그 없음";
  $("debug_log").scrollTop = $("debug_log").scrollHeight;
}

/// 디버깅 복사 — 글을 클립보드로, 결과는 디버깅 상태 줄.
export async function debugCopy(text, label) {
  await act(label, async () => {
    await navigator.clipboard.writeText(text);
    return `${label} 했습니다`;
  }, "debug_msg");
}

$("debug_level").addEventListener("change", renderDebugLog);
$("debug_refresh").addEventListener("click", loadDebug);
// 진단 정보 복사 — 버전 · OS · 경로 · 크기 · 포트 · 열린 연결 수(신원 키 · 토큰 · 지문 없음 — `diagText`).
$("debug_copy").addEventListener("click", async () => {
  if (!debugInfo) await loadDebug();
  if (debugInfo) await debugCopy(diagText(debugInfo, state.me), "진단 정보 복사");
});

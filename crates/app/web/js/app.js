// 진입 — 토큰 확인 뒤 도메인 모듈을 불러 이벤트를 걸고 첫 화면을 그린다.
// 화면 조립 — 5영역: 상단 바(통합 검색 · + 새로 만들기 · 알림 · 프로필) · 레일 · 사이드바(내비 · 대화방 ·
// 에이전트) · 메인(대화 · 내 활동 · 북마크 항목 · 에이전트 · 설정 트리) · 실행 상세. 값은 textContent 로만 넣는다(HTML 주입
// 없음) — 원격에서 온 이름 · 본문도 같다. HTML 문서 본문만 sandbox iframe(srcdoc · CSP)으로. 판단 로직은 logic/(node 테스트).
import "./core/token.js"; // 맨 앞 — 토큰이 없으면 여기서 멈춰 다른 모듈의 최상위 코드가 돌지 않는다.
import { $ } from "./core/dom.js";
import { closeFloat, floatFrom } from "./core/float.js";
import { state } from "./core/state.js";
import { listen, refresh } from "./events.js";
import { loadGroups, loadPeers, pingPeers } from "./rail.js";
import { closeSearch } from "./search.js";
import { loadAgents } from "./settings/agents.js";
import { renderTree } from "./settings/index.js";
import { showSettings } from "./settings/prefs.js";
import { MENUS, closeMenus, render, showMe, showStatus } from "./shell.js";
import { loadConvs, loadCounts } from "./sidebar.js";
import "./tiles.js";
import "./home.js";
import "./conv.js";
import "./conv-menu.js";
import "./composer.js";
import "./detail.js";
import "./activity.js";
import "./requests.js";
import "./agents.js";
import "./peers.js";
import "./compose.js";
import "./drag.js";
import "./settings/conn.js";
import "./settings/groups.js";
import "./settings/debug.js";

// 메뉴 · 검색 결과 밖을 누르면 닫는다.
document.addEventListener("click", (ev) => {
  for (const [b, m] of MENUS) {
    if ($(m).hidden || $(b).contains(ev.target) || $(m).contains(ev.target)) continue;
    $(m).hidden = true;
    $(b).setAttribute("aria-expanded", "false");
  }
  if (!ev.target.closest?.(".search")) closeSearch();
  // 상자 안에서 눌러 바뀐 항목(그 사이 떨어져 나간 요소)은 바깥 누름이 아니다.
  if (!$("float").hidden && ev.target.isConnected && !$("float").contains(ev.target) && !floatFrom?.contains(ev.target)) {
    // 초점을 받지 못하는 바탕을 누르면 초점이 body 로 떨어진다 — 그때만 연 요소로 돌린다. 초점을 받는 곳을 눌렀으면 그대로.
    const lost = document.activeElement === document.body || $("float").contains(document.activeElement);
    closeFloat(lost && !ev.target.closest("a, button, input, select, textarea, summary, [tabindex]"));
  }
});
// Ctrl+K = 통합 검색 칸, Esc = 메뉴 · 검색 결과 닫기.
document.addEventListener("keydown", (ev) => {
  if ((ev.ctrlKey || ev.metaKey) && ev.key.toLowerCase() === "k") {
    ev.preventDefault();
    $("search_q").focus();
    $("search_q").select();
  } else if (ev.key === "Escape") {
    const from = MENUS.find(([, m]) => $(m).contains(document.activeElement));
    const inSearch = $("search_box").contains(document.activeElement);
    if (!$("float").hidden) {
      closeFloat(true);
      return;
    }
    closeMenus();
    closeSearch();
    if (from) $(from[0]).focus();
    else if (inSearch) $("search_q").focus();
  }
});

renderTree();
window.addEventListener("hashchange", render);
await showSettings(); // 테마 · 기본값을 먼저 — 설정 연결의 "기본값(…)" 문구가 쓴다
await Promise.all([showStatus(), showMe(), loadPeers(), loadAgents(), loadConvs(), loadGroups()]);
// 웹UI 를 열 때 페어링된 상대 연결 확인 한 번.
pingPeers(state.peers.filter((p) => p.paired).map((p) => p.id));
await render();
await loadCounts();
listen();
// ponytail: 이 PC 에서 생긴 변경 · 시간이 지나 꺼지는 접속 표시는 신호가 없어 5초마다 전부 다시 읽는다 — 그 신호가 생기면 줄인다(D39 ②).
setInterval(() => refresh(), 5000);

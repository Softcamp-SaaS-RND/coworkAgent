// 새로 고침 · 변경 신호 스트림 · 알림 소리.
import { api } from "./core/api.js";
import { $ } from "./core/dom.js";
import { state } from "./core/state.js";
import { token } from "./core/token.js";
import { requestNotice } from "./logic/conv.js";
import { previewStaleOf } from "./logic/home.js";
import { route } from "./logic/shell.js";
import { OneAtATime, loadersFor, mergeChanges } from "./logic/sync.js";
import { loadActivity, loadSaved } from "./activity.js";
import { openConversation } from "./conv.js";
import { loadRuns } from "./detail.js";
import { refreshHome } from "./home.js";
import { loadPeerMessages } from "./peers.js";
import { loadGroups, loadPeers, pickRail } from "./rail.js";
import { REQ_BOARD, loadRequests } from "./requests.js";
import { go, showMe } from "./shell.js";
import { loadConvs, loadCounts, setFolded } from "./sidebar.js";

/// `only` = 다시 읽을 로더 이름 집합(`loadersFor`), 없으면 전부. 한 번에 하나만 돈다 — 도는 중에 온 요청은 합쳐 끝난 뒤 한 번 더
/// (먼저 시작한 오래된 조회가 나중 조회를 덮지 않게). 기다리는 쪽은 자기 요청이 반영된 뒤 풀린다.
export const refresher = new OneAtATime((only) => refreshOnce(only));
export function refresh(only) {
  return refresher.call(only ?? null);
}

export async function refreshOnce(only) {
  const want = (k) => !only || only.has(k);
  if (want("peers")) await loadPeers();
  if (want("groups")) await loadGroups();
  if (want("convs")) await loadConvs();
  const r = route(location.hash);
  if (state.current && r.view === "conversation" && want("current")) await openConversation(state.current);
  if (r.view === "activity" && want("activity")) await loadActivity();
  if (r.view === "requests" && want("requests")) await loadRequests();
  if (r.view === "saved" && want("saved")) await loadSaved();
  if (r.view === "peer" && want("peer")) await loadPeerMessages(r.id);
  if (!$("detail").hidden && want("runs")) await loadRuns();
  if (r.view === "settings" && want("settings")) await showMe(); // 열린 피어 연결 수
  if (want("counts")) await loadCounts();
}

/// 변경 신호를 잠깐 모아 한 번만 다시 읽는다(연달아 온 글 · 진행).
export let changeQ;
export function onChange(res) {
  const first = changeQ === undefined;
  changeQ = mergeChanges(changeQ, res);
  if (!first) return;
  setTimeout(() => {
    const q = changeQ;
    changeQ = undefined;
    refresh(loadersFor(q, state.current?.id)).catch(refreshFailed);
  }, 50);
}

/// 변경 신호 다시 읽기의 실패 — 기다리는 쪽이 없어 상태 줄에 보인다. 다음 신호 · 5초가 다시 읽는다.
export function refreshFailed(e) {
  $("status_msg").textContent = "새로 고침 실패: " + e.message;
}

/// 페어링 종류 알림 — 누르면 설정의 연결, 그 밖은 요청 보드(받은 요청) · 내 활동.
export const PAIRING_KINDS = new Set(["pair_request", "auto_paired", "approved"]);

/// 받은 요청 효과음 — 짧은 두 음(WebAudio, 파일 없음). 브라우저는 사용자 입력 전에는 소리를 막는다(autoplay) — 첫 입력에서 만든다.
export let audio = null;
for (const type of ["pointerdown", "keydown"])
  addEventListener(type, () => (audio ??= new AudioContext()), { once: true, capture: true });

export function chime() {
  if (!audio) return;
  audio.resume?.();
  const t = audio.currentTime;
  [880, 1320].forEach((hz, i) => {
    const at = t + i * 0.12;
    const o = audio.createOscillator();
    const g = audio.createGain();
    o.frequency.value = hz;
    g.gain.setValueAtTime(0.15, at);
    g.gain.exponentialRampToValueAtTime(0.001, at + 0.12);
    o.connect(g).connect(audio.destination);
    o.start(at);
    o.stop(at + 0.12);
  });
}

/// 서버 → 웹UI 알림 스트림. 붙는 순간 트레이 "새 요청" 이 지워진다. 원격 이름은 텍스트로만(브라우저 알림 본문도 텍스트).
export function listen() {
  const es = new EventSource(`/api/events?t=${encodeURIComponent(token)}`);
  // 붙은 뒤 "봤다" 를 알린다(트레이 새 요청 지움) — 상태 변경은 POST 로만.
  es.onopen = () => {
    api("POST", "/api/seen").catch(() => {});
    // 끊긴 동안의 변경 신호는 다시 오지 않는다 — 붙을 때마다 미리보기 글을 전부 새로.
    state.previewAll = true;
    refreshHome();
  };
  es.onmessage = (ev) => {
    let n;
    try {
      n = JSON.parse(ev.data);
    } catch {
      return;
    }
    $("status_msg").textContent = `${n.title} — ${n.body.split("\n")[0]}`;
    // 다시 읽기는 변경 신호(`change`)가 맡는다 — 알림은 설정에 따라 오지 않을 수 있다.
    // 받은 요청 = 같은 tag 한 장(앞 알림을 바꾼다) · 누르면 요청 보드 · 효과음은 웹UI(OS 기본음 대신 — 알림은 무음).
    const nn = requestNotice(n);
    if ($("notify_inbound").checked && "Notification" in window && Notification.permission === "granted") {
      const shown = new Notification(nn.title, { body: nn.body, tag: nn.tag, silent: nn.sound });
      shown.onclick = () => {
        window.focus();
        if (PAIRING_KINDS.has(n.kind)) {
          go("#settings/peers");
          return;
        }
        pickRail(null); // 내 활동 · 요청 전부
        if (nn.href === "#requests") {
          setFolded(REQ_BOARD, true);
          state.reqBox ??= "theirs";
        }
        go(nn.href);
      };
    }
    if (nn.sound && state.settings.notify_sound !== false && !state.settings.focus) chime();
  };
  // 변경 신호 — 알림 설정 · 집중과 무관하게 온다. 읽지 못하면 전체.
  es.addEventListener("change", (ev) => {
    let c;
    try {
      c = JSON.parse(ev.data);
    } catch {
      c = {};
    }
    // 개요 미리보기 — 신호가 가리킨 대화(전체면 전부)는 다음 그리기에서 다시 읽는다.
    const stale = previewStaleOf(c.res);
    if (stale == null) state.previewAll = true;
    else for (const id of stale) state.previewStale.add(id);
    onChange(c.res);
  });
}

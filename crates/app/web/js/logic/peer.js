// 상대 — 접속 · 요약 · 안 읽음 · 색 · 주고받은 메시지.
import { AGENT_COLORS } from "./agent.js";
import { shortName } from "./format.js";
import { sortByPos } from "./order.js";

/// 타일 색 class — 여섯 색(`AGENT_COLORS` — 에이전트 아이콘 색과 같음)이면 `tinted {색}`, 아니면 빈 글자(흑연 · 무채).
/// 레일 · 줄 타일 · 말풍선 · 주고받은 메시지 · 프로필 얼굴이 같은 규칙(`app.css` `.tinted`).
export function tileTint(color) {
  return AGENT_COLORS.includes(color) ? `tinted ${color}` : "";
}

/// '나' 얼굴 색 — 고른 색, 없으면 흑연(`me-tile`).
export function meTint(color) {
  return tileTint(color) || "me-tile";
}

/// 내 말풍선 면 열쇠 — '나' 타일 색(여섯 중 하나), 없으면 흑연(`rail-2`). `app.css` `.msg.own-{열쇠}`.
export function ownTint(color) {
  return AGENT_COLORS.includes(color) ? color : "rail-2";
}

/// 피어 승인 상태 → 문구.
export function peerState(p) {
  if (p.paired) return "페어링됨";
  if (!p.my_ok) return "내 승인 대기";
  return "상대 승인 대기";
}

/// 내 상태(프로필 아이콘 테두리 · 점) — 앱과 연결 끊김(붉음)이 먼저, 그다음 집중(주황), 그 밖은 받기 가능(초록).
export function presenceOf(focus, offline) {
  if (offline === true) return { key: "offline", text: "앱과 연결 끊김" };
  return focus === true ? { key: "focus", text: "집중" } : { key: "online", text: "받기 가능" };
}

/// 상대 접속 상태(`GET /api/peers` 의 `seen_at` · `focus` — 통신할 때만 갱신) — 레일 점 · 사이드바 머리 · 대화 머리글. "방금 통신"(75초 안) ·
/// "n분 · n시간 · n일 전 통신" · "통신 기록 없음". 방금 통신했을 때만 점이 켜지고 집중이 앞에 붙는다.
export function presence(p, now = Math.floor(Date.now() / 1000)) {
  const at = p?.seen_at;
  if (at == null) return { key: "offline", text: "통신 기록 없음" };
  const ago = Math.max(0, now - at);
  const when =
    ago < 75
      ? "방금 통신"
      : ago < 3600
        ? `${Math.floor(ago / 60)}분 전 통신`
        : ago < 86400
          ? `${Math.floor(ago / 3600)}시간 전 통신`
          : `${Math.floor(ago / 86400)}일 전 통신`;
  if (ago >= 75) return { key: "offline", text: when };
  return p.focus ? { key: "focus", text: `집중 중 · ${when}` } : { key: "online", text: when };
}

/// 주고받은 메시지 보기 기간 — 1 ~ 12주(서버 `run_opts::PEER_WEEKS_MAX`), 기본 4.
export const PEER_WEEKS = Array.from({ length: 12 }, (_, i) => i + 1);

/// 기간 고르기 항목 — `{value, label: "{n}주"}`.
export function peerWeeksOptions() {
  return PEER_WEEKS.map((n) => ({ value: String(n), label: `${n}주` }));
}

/// 상대별 절('나') — 행(`peer` = 상대 id)을 레일 순서(`paired`)로 묶고, 그 뒤 페어링이 풀린 상대를 id 순으로. 행 없는 상대는 뺀다.
/// 절 안 순서는 들어온 순서(목록이 이미 최신순).
export function peerGroups(rows, paired, peerNames) {
  const rest = [...new Set(rows.map((r) => r.peer))].filter((id) => !paired.includes(id)).sort((a, b) => a - b);
  return [...paired, ...rest]
    .map((id) => ({ peer: id, name: shortName(peerNames.get(id)), rows: rows.filter((r) => r.peer === id) }))
    .filter((g) => g.rows.length);
}

/// 받은 대기 요청 수를 상대별로('대화' 요약 머리).
export function waitingByPeer(messages) {
  const n = new Map();
  for (const m of messages) if (m.conversation) n.set(m.conversation.peer_id, (n.get(m.conversation.peer_id) ?? 0) + 1);
  return n;
}

/// '나' 의 '대화' = 상대별 요약 — 페어링된 상대(레일 순서)는 대화가 없어도, 풀린 상대는 대화가 있을 때만. 대화 수 · 안 읽음 합 · 대기 요청 ·
/// 최근 활동(`last_at` 최대, 없으면 `null`) · 그 상대 대화(손으로 놓은 순서 — `sortByPos`).
export function peerSummary(convs, paired, peerNames, waiting = new Map()) {
  const rest = [...new Set(convs.map((c) => c.peer_id))].filter((id) => !paired.includes(id)).sort((a, b) => a - b);
  return [...paired, ...rest].map((id) => {
    const rows = sortByPos(convs.filter((c) => c.peer_id === id));
    const ats = rows.map((c) => c.last_at).filter((t) => t != null);
    return {
      peer: id,
      name: shortName(peerNames.get(id)),
      convs: rows.length,
      unread: rows.reduce((n, c) => n + (c.unread ?? 0), 0),
      waiting: waiting.get(id) ?? 0,
      last: ats.length ? Math.max(...ats) : null,
      rows,
    };
  });
}

/// 레일 배지 — 상대별 안 읽음 합(0 인 상대는 없다).
export function unreadByPeer(convs) {
  const sums = new Map();
  for (const c of convs) if (c.unread > 0) sums.set(c.peer_id, (sums.get(c.peer_id) ?? 0) + c.unread);
  return sums;
}

/// 상대 타일 배지 — 그 타일에서 보이는 대화의 안 읽음만. 에이전트 대화(`dir = "in"`)는 '나' 의 '에이전트 응답' 이 센다(상대 타일에는 그 메뉴가 없다).
export function tileUnread(convs) {
  return unreadByPeer(convs.filter((c) => c.dir !== "in"));
}

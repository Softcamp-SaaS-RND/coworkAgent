// 요청 — 상태 · 이동 · 보드 · 보관.
import { bubbleSide } from "./author.js";
import { convClosed, convLabel } from "./conv.js";
import { preview, shortName } from "./format.js";

/// 요청 상태 문구(an_core `ReqState`).
export const REQ_STATES = { wait: "대기", accepted: "수용", rejected: "반려", done: "완료" };

/// 사람 글의 종류 태그 — 요청이면 "요청 · {상태}"(받은 쪽이 바꾼다), 질의 · 답은 없음.
export function askTag(m) {
  if (m.author !== "human" || !m.request) return "";
  return `요청 · ${REQ_STATES[m.req_state] ?? REQ_STATES.wait}`;
}

/// 받은 요청에 지금 할 수 있는 상태 바꾸기 — 지금 상태 말고 네 상태 어디로든(되돌리기 포함, 내 요청 · 질의 · 답은 없음). 닫힌 대화(`closed` —
/// [`convClosed`])는 읽기 전용이라 없음.
export function reqMoves(m, closed = false) {
  if (closed || m.author !== "human" || m.mine !== false || !m.request) return [];
  const from = m.req_state ?? "wait";
  return Object.keys(REQ_STATES)
    .filter((to) => to !== from)
    .map((to) => ({ to, label: REQ_STATES[to] }));
}

/// 받은 요청의 상태를 요청자에 알리지 못했다(오래 닿지 않아 포기 · 요청자가 거절) — 같은 상태로 "다시 알리기". 닫힌 대화는 아님.
export function reqResend(m, closed = false) {
  return !closed && m.author === "human" && m.mine === false && m.request === true && !!m.refused;
}

/// '요청' 화면 행 — 상대 · 대화 제목 · 상태 · 메모 · 본문 앞부분 · 시각, 누르면 그 글로. `moves` = 받은 요청에 할 수 있는 상태 바꾸기,
/// `resend` = 알리지 못한 상태 다시 알리기.
export function requestRows(messages, peerNames) {
  return messages
    .filter((m) => m.conversation && m.request)
    .map((m) => {
      const conv = m.conversation;
      return {
        id: m.id,
        peer: conv.peer_id,
        who: shortName(peerNames.get(conv.peer_id)),
        title: convLabel(conv, peerNames).title,
        state: REQ_STATES[m.req_state] ?? REQ_STATES.wait,
        stateKey: m.req_state ?? "wait",
        done: m.req_state === "done" || m.req_state === "rejected",
        note: m.req_note ?? "",
        preview: preview(m.text),
        at: m.req_at ?? m.created_at,
        href: `#c/${conv.id}/m/${m.id}`,
        moves: reqMoves(m, !!convClosed(conv)),
        resend: reqResend(m, !!convClosed(conv)),
        archive: reqArchiveAction(m),
      };
    });
}

/// 요청 archive 단추(이 PC 에만) — 받은 · 보낸 요청 모두: archive 됨 = "archive 해제", 완료 = "archive", 그 밖 `null`.
export function reqArchiveAction(m) {
  if (m.author !== "human" || !m.request) return null;
  if (m.archived) return { key: "unarchive", label: "archive 해제" };
  return m.req_state === "done" ? { key: "archive", label: "archive" } : null;
}

/// 요청 화면 절 데이터 — 쪽(`box`)마다 머리 건수 = 서버 집계(`GET /api/messages/req_counts` — 목록 상한과 무관), 행 = 서버가 상태로 거른
/// 목록(`req=`) 그대로.
export function reqBoxes(boxes, lists, counts, peerNames) {
  return boxes.map((box, i) => ({ box, sum: counts[i], rows: requestRows(lists[i], peerNames) }));
}

/// 요청 개요 절 머리 — 이름 · 전체 건수 · 상태별 건수(대기 · 수용 · 완료 · 반려 순, 0건 상태는 뺀다).
export function reqHead(title, sum) {
  const states = ["wait", "accepted", "done", "rejected"].filter((k) => sum[k]).map((k) => ({ key: k, text: `${REQ_STATES[k]} ${sum[k]}` }));
  // archive 된 완료는 전체 수에 들지 않고 따로.
  if (sum.archived) states.push({ key: "archived", text: `archive ${sum.archived}` });
  return { title, count: String(sum.total), states };
}

/// 요청 목록 글자 단추 모양 — 바꿀 상태 → 색(`ok` 초록 · `no` 붉음) · 아이콘(`app.css` `.ico-{이름}`).
export const REQ_BTN = {
  wait: { tone: "wait", icon: "reset" },
  accepted: { tone: "ok", icon: "check" },
  done: { tone: "ok", icon: "check" },
  rejected: { tone: "no", icon: "x" },
};

/// 요청 보드 열 순서.
export const BOARD_COLS = ["wait", "accepted", "rejected", "done"];

/// 요청 보드 열 — 열 = 상태(`BOARD_COLS` 순), 건수 = 서버 집계(`req_counts` — 목록 상한과 무관), 카드 = 그 상태의 행.
export function boardColumns(rows, sum) {
  return BOARD_COLS.map((key) => ({ key, name: REQ_STATES[key], count: sum?.[key] ?? 0, rows: rows.filter((r) => r.stateKey === key) }));
}

/// 보드 카드를 그 열에 놓을 수 있나 — 받은 요청에 지금 할 수 있는 상태 바꾸기(`reqMoves` — 네 상태 사이 어디로든)만. 보낸 요청 · 닫힌 대화 · 같은 열은 거짓.
export function canDrop(row, to) {
  return row.moves.some((m) => m.to === to);
}

/// 요청 의견 말풍선 — 대기가 아닌 요청 바로 아래 하나: 지금 상태 · 메모 · 바뀐 시각, 방향 = 받은 쪽 사람 쪽(요청 말풍선의 반대). 아니면 `null`.
export function reqFeedback(m) {
  if (m.author !== "human" || !m.request || (m.req_state ?? "wait") === "wait") return null;
  return {
    key: m.req_state,
    state: REQ_STATES[m.req_state] ?? m.req_state,
    note: m.req_note ?? "",
    at: m.req_at ?? m.created_at,
    side: bubbleSide(m) === "mine" ? "theirs" : "mine",
  };
}

// 주고받은 메시지 — 상대 절 · 줄 · 페이지.
import { authorLabel } from "./author.js";
import { preview, shortName } from "./format.js";
import { askTag } from "./request.js";

/// `GET /api/messages` 항목 → 목록 행. `kind` = `""`(전체) · `human`(사람 글) · `agent`(에이전트 답). 누르면 그 대화로.
export function inboxRows(messages, peerNames, { kind = "" } = {}) {
  return messages
    .filter((m) => m.conversation && (!kind || m.author === kind))
    .map((m) => {
      const conv = m.conversation;
      // 방향 라벨(보냄 · 받음 등)은 없다 — 이름 · 요청 칩 · 대화방 제목.
      return {
        id: m.id,
        who: authorLabel(conv, m, peerNames.get(conv.peer_id)),
        // 어느 에이전트와의 대화인지(주고받은 글 칩): 보낸 대화 = 상대 에이전트, 받은 대화 = 내 에이전트.
        agent: conv.dir === "in" ? `내 ${conv.agent}` : conv.agent,
        kind: askTag(m),
        saved: m.saved === true,
        room: conv.title || "",
        preview: preview(m.text),
        at: m.created_at,
        href: `#c/${conv.id}`,
        peer: conv.peer_id,
        author: m.author,
        mine: m.mine === true,
      };
    });
}

/// 주고받은 메시지 한 번에 받는 건수.
export const PEER_PAGE = 20;

/// 주고받은 메시지 상대 절 머리(`GET /api/messages/peers`) — 레일 순서(`paired`), 그 밖(페어링 풀린 상대)은 id 순으로 뒤. 이름 · 건수 · 마지막 메시지
/// 한 줄 · 시각.
export function peerSummaryRows(items, paired, peerNames) {
  const rank = (id) => (paired.includes(id) ? paired.indexOf(id) : paired.length + id);
  return [...items]
    .sort((a, b) => rank(a.peer) - rank(b.peer))
    .map((x) => ({ peer: x.peer, name: shortName(peerNames.get(x.peer)), count: `${x.count}건`, total: x.count, last: preview(x.last?.text), at: x.last?.created_at ?? 0 }));
}

/// 주고받은 메시지 다시 그리기 서명 — 행에 보이는 값 전부(본문 · 요청 상태 · 메모 · 편집 · 상태 · 북마크 · 대화 제목 · 에이전트).
export function peerMsgsSig(heads, pages) {
  const msg = (m) => [
    m.id,
    m.text,
    m.saved,
    m.req_state,
    m.req_note,
    m.edited_at,
    m.state,
    m.created_at,
    // 발화자 표시(`authorLabel`)가 기대는 값 — 공유 사본은 같은 글의 이것만 바뀔 수 있다.
    m.author,
    m.mine,
    m.via,
    m.agent,
    m.owner,
    m.conversation?.title,
    m.conversation?.agent,
    m.conversation?.dir,
  ];
  return JSON.stringify([heads, heads.map((h) => (pages.get(h.peer) ?? []).map(msg))]);
}

/// 펼친 절 새로 고침 — 새로 받은 첫 페이지(`fresh`, 최신순) + 이미 받은 글 중 그 페이지 마지막보다 앞(`(created_at, id)`)의 꼬리.
/// 첫 페이지가 다 차지 않으면(그 상대 글 전부) 꼬리는 없다 — "더 보기" 로 받은 쪽을 5초 새로 고침이 지우지 않는다.
export function mergePage(fresh, have, page = PEER_PAGE, since = -Infinity) {
  // 받아 둔 꼬리도 보기 기간(`since` 이후에 쓴 글)만 — 시간이 지나 기간 밖이 된 글이 남지 않게.
  have = have.filter((m) => m.created_at >= since);
  if (fresh.length < page) return fresh;
  const end = fresh.at(-1);
  const older = (m) => m.created_at < end.created_at || (m.created_at === end.created_at && m.id < end.id);
  const ids = new Set(fresh.map((m) => m.id));
  return [...fresh, ...have.filter((m) => older(m) && !ids.has(m.id))];
}

/// 펼친 상대 절 끝 줄 — "{보인 수} / {전체}", 남았을 때만 더 보기.
export function pageMore(shown, total) {
  return { text: `${shown} / ${total}`, more: shown < total };
}

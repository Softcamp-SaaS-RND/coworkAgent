// 글 작성자 — 이름 · 말풍선 쪽.
import { shortName } from "./format.js";

/// 말풍선 쪽 — 이 PC 쪽 글(`mine` — 보낸 대화의 내 글 · 받은 대화의 내 에이전트 답)은 오른쪽, 그 밖은 왼쪽.
/// 공유 대화는 공유한 사람 쪽 글(`owner`)이 오른쪽 — 대화가 오가는 순서를 알기 쉽게.
export function bubbleSide(m) {
  return m.mine === true || m.owner === true ? "mine" : "theirs";
}

/// 내가 직접 쓴 글 — 이 PC 사람 글(CLI 글 · 공유 대화 빼고). 말풍선에 '나' 얼굴 · 면(`app.css` `.msg.own`).
export function ownHuman(conv, m) {
  return conv.dir !== "shared" && m.author !== "agent" && m.mine === true && m.via !== "cli";
}

/// 발화자 이름 — 사람: 내 글 "나", 상대 글은 상대 이름. 에이전트: 대화 에이전트의 주인 쪽(보낸 대화 = 상대 PC, 받은 대화 = 내 PC) —
/// 멘션한 요청의 답이면 실행한 에이전트(`m.agent`). 자동 왕복이면 반대쪽 에이전트 글도 있다 — 보낸 대화의 내 글 = 내 자동 왕복 에이전트,
/// 받은 대화의 상대 글 = 상대 에이전트.
export function authorLabel(conv, m, peerName) {
  const peer = shortName(peerName);
  if (conv.dir === "shared") return sharedAuthor(conv, m, peer);
  if (m.author === "agent") {
    const agent = m.agent ?? conv.agent;
    if (conv.dir === "out") return m.mine === true ? `내 ${conv.relay_agent ?? "에이전트"}` : `${peer}/${agent}`;
    return m.mine === false ? `${peer}/${conv.relay_agent ?? "에이전트"}` : `내 ${agent}`;
  }
  if (m.via === "cli") return m.mine ? `내 ${CLI_LABEL}` : `${peer}/${CLI_LABEL}`;
  return m.mine ? "나" : peer;
}

/// 공유 대화의 발화자 — `owner` = 초대한 사람(`peer`) 쪽 글, 아니면 원 상대(`share.other`). 에이전트는 그 쪽 사람의 것 — 대화 에이전트는
/// 초대한 사람이 연 대화(`owner_dir = "out"`)면 원 상대 것, 받은 대화면 초대한 사람 것.
export function sharedAuthor(conv, m, peer) {
  const other = conv.share?.other ?? "상대";
  const who = m.owner === true ? peer : other;
  if (m.author === "agent") {
    const ownsAgent = (conv.share?.owner_dir === "out") !== (m.owner === true);
    return `${who}/${m.agent ?? (ownsAgent ? conv.agent : "에이전트")}`;
  }
  if (m.via === "cli") return `${who}/${CLI_LABEL}`;
  return who;
}

/// CLI 로 쓴 글(`via = "cli"`)의 발화자 — 사용자의 AI 에이전트.
export const CLI_LABEL = "AI 에이전트(CLI)";

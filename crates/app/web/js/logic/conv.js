// 대화 — 이름 · 유형 · 목록 줄 · 절 · 메뉴 · 공유 · 대화 정보.
import { engineLabel } from "./agent.js";
import { authorLabel } from "./author.js";
import { dayLabel, formatTime, preview, shortName, stateText } from "./format.js";
import { peerGroups } from "./peer.js";

/// 사이드바 · 머리글의 대화(대화방) 이름. 제목이 없으면(메시지 없음) 에이전트로. 보냄 · 받음 표시는 없다(대화방 하나에 주고받은 글이 이어진다).
export function convLabel(conv, peerNames) {
  const who = peerNames.get(conv.peer_id) ?? "알 수 없는 상대";
  // 공유 대화 — 초대한 사람(`peer_id`)과 원 상대(`share.other`)의 대화를 보기만 한다.
  if (conv.dir === "shared") return { title: conv.title || `${conv.agent} 대화`, sub: `공유 · ${shortName(who)} ↔ ${conv.share?.other ?? "상대"}` };
  const out = conv.dir === "out";
  const title = conv.title || (out ? `${conv.agent} 에게` : `내 ${conv.agent} 가 받은 요청`);
  const sub = out ? `${who} 의 ${conv.agent}` : `${who} 의 요청 · 내 ${conv.agent}`;
  return { title, sub };
}

/// 대화방 줄 미리보기 — 마지막 글 `{말한 이}: {본문 한 줄}`(서버 `last`), 글이 없으면 빈 글자.
export function roomPreview(conv, peerName) {
  const l = conv.last;
  if (!l) return "";
  return `${authorLabel(conv, l, peerName)}: ${preview(l.text)}`;
}

/// 사이드바 대화 목록 — 레일에서 고른 타일의 대화만, `q` 가 있으면 제목으로만 거른다(대소문자 무시).
export function convRows(convs, peerNames, railPeer, q = "") {
  const needle = q.trim().toLowerCase();
  return filterByPeer(convs, railPeer)
    .map((c) => ({ c, label: convLabel(c, peerNames) }))
    .filter(({ label }) => !needle || label.title.toLowerCase().includes(needle));
}

/// 레일에서 고른 상대의 대화만(`null` = 내 PC = 전부).
export function filterByPeer(convs, peerId) {
  return peerId == null ? convs : convs.filter((c) => c.peer_id === peerId);
}

/// 열린 대화가 레일에서 고른 상대와 맞나 — 아니면 그 대화를 닫는다(다른 PC 를 고른 화면에서 이전 상대에게 보내지 않게).
export function keepsConversation(conv, railPeer) {
  return conv != null && (railPeer == null || conv.peer_id === railPeer);
}

/// 사이드바 대화방 절 — ★ 는 줄의 별로 보이고 대화방은 제자리에 남는다(모아 보기는 메뉴 "즐겨찾기"). `paired`(레일 순서 상대 id)를 주면
/// ('나') 상대별 절(`peer:{id}`)로 — 대화가 하나도 없으면 빈 '대화' 절 하나.
export function convSections(rows, paired = null, peerNames = new Map()) {
  if (paired == null) return [{ key: "convs", title: "대화", rows }];
  const groups = peerGroups(
    rows.map((r) => ({ ...r, peer: r.c.peer_id })),
    paired,
    peerNames,
  ).map((g) => ({ key: `peer:${g.peer}`, peer: g.peer, title: g.name, rows: g.rows }));
  return groups.length ? groups : [{ key: "convs", title: "대화", rows: [] }];
}

/// 대화 한 개의 관람자 상한(서버 `share::MAX_OBSERVERS` 와 같다).
export const MAX_OBSERVERS = 5;

/// 대화 머리의 공유 표시 — 공유 대화(사본) = "읽기 전용" · 빠졌으면 "공유 해제됨", 내 대화 · agent 대화 = 관람자가 있으면 "관람자 n"(초대한 쪽은
/// `observers`, 원 상대는 알림 받은 `watchers`).
export function shareHead(conv) {
  if (conv.dir === "shared") return conv.share?.ended != null ? "공유 해제됨" : "읽기 전용";
  const n = conv.observers?.length || conv.watchers || 0;
  return n ? `관람자 ${n}` : "";
}

/// 관람자로 넣을 수 있는 상대 — 페어링된 상대 중 원 상대 · 이미 관람자인 상대를 뺀다. 공유 대화(사본) · 상한이면 빈 목록.
export function shareCandidates(conv, peers) {
  const now = conv.observers ?? [];
  if (conv.dir === "shared" || now.length >= MAX_OBSERVERS) return [];
  return peers.filter((p) => p.paired && p.id !== conv.peer_id && !now.includes(p.id));
}

/// 대화방 목록 묶음 — 대화의 `dir` 로: 내가 연 대화(자동 왕복 포함) · 상대가 연 대화(내 에이전트가 답). 순서 = 이 배열 순서.
export const CONV_GROUPS = [
  { dir: "out", title: "내 대화" },
  { dir: "in", title: "에이전트 대화" },
  { dir: "shared", title: "공유 대화" },
];

/// 한 절의 행을 묶음으로 나눈다 — 행 있는 묶음만, 묶음 안 순서는 들어온 그대로.
export function splitByDir(rows) {
  return CONV_GROUPS.map((g) => ({ ...g, rows: rows.filter((r) => r.c.dir === g.dir) })).filter((g) => g.rows.length);
}

/// 사이드바 대화 목록에 보일 줄 — 나 · 상대 타일은 에이전트 대화(`in`)를 뺀다. 상대별 절 · 빈 안내 판정도 이 줄로 한다(빼기 전 줄로
/// 절을 만들면 에이전트 대화뿐인 상대 절 · 검색 결과가 빈 채로 남는다).
export function visibleConvRows(rows, group) {
  return group ? rows : rows.filter((r) => r.c.dir !== "in");
}

/// 사이드바 대화 목록 묶음 — 나 · 상대 타일(`group` 없음)은 에이전트 대화(`in`)를 빼고(사이드바 '에이전트 응답' 화면으로) 내 대화는 머리 없이,
/// 공유 대화만 머리. 그룹 타일은 묶음이 둘 이상이면 묶음마다 머리. `head` = 머리를 보일지.
export function convListParts(rows, group) {
  const parts = splitByDir(visibleConvRows(rows, group));
  return parts.map((g) => ({ ...g, head: group ? parts.length > 1 : g.dir !== "out" }));
}

/// 에이전트 응답 화면 — 상대가 연 대화(`in`)를 요청한 상대별 절로(`peerGroups` — `paired` 순서, 그 뒤 풀린 상대). 상대 타일은 `paired` = [그 상대].
export function agentConvSections(convs, paired, peerNames) {
  const rows = convRows(convs.filter((c) => c.dir === "in"), peerNames, null).map((r) => ({ ...r, peer: r.c.peer_id }));
  return peerGroups(rows, paired, peerNames).map((g) => ({ key: `agentconvs:${g.peer}`, peer: g.peer, title: g.name, rows: g.rows }));
}

/// 대화 행 표시 — 둘째 줄 `{에이전트} · {상태}`(받은 대화의 에이전트는 내 것), 실행 중 표시, 자동 왕복 칩 "왕복 {n}/{상한}", 안 읽음.
export function convMeta(c) {
  return {
    agent: c.dir === "in" ? `내 ${c.agent}` : c.agent,
    state: stateText(c.last_state),
    running: c.last_state === "running",
    relay: c.relay_agent && c.relay_cap != null ? `왕복 ${c.relay_count ?? 0}/${c.relay_cap}` : null,
    unread: c.unread > 0,
    // 닫힌 대화(나감 · 상대가 나감 · 공유 해제됨) — 읽기 전용으로 남는다.
    tag: convClosed(c) || null,
  };
}

/// 닫힌 대화의 문구 — 내가 나감 "나감" · 상대가 나감 "상대가 나감" · 공유 사본을 빼거나 내가 나감 "공유 해제됨", 아니면 빈 글자. 읽기 전용 · 대화 줄 태그 · 대화 머리.
export function convClosed(c) {
  if (!c) return "";
  if (c.dir === "shared") return c.share?.ended != null ? "공유 해제됨" : "";
  if (c.left_at != null) return "나감";
  return c.peer_left_at != null ? "상대가 나감" : "";
}

/// 나가기 확인 문구.
export const LEAVE_CONFIRM = "대화방이 목록에서 삭제됩니다.";

/// 대화 ⋯ 메뉴 끝 항목 — "기록 삭제" ▸(`sub` — 내 글만 삭제 · 모든 사용자 글 삭제) · "나가기"(확인 뒤 이 PC 목록에서 삭제). 닫힌 대화(나감 · 상대가 나감)는
/// 내 글만, 공유 사본(관람자 — 읽기 전용)은 나가기만. `icon` = `app.css` `.ico-{이름}`.
export function convMenuItems(conv) {
  const leave = { key: "leave", label: "나가기", icon: "exit", danger: true, confirm: LEAVE_CONFIRM };
  if (conv.dir === "shared") return [leave];
  const sub = [{ key: "clear_mine", label: "내 글만 삭제" }];
  if (!convClosed(conv)) sub.push({ key: "clear_all", label: "모든 사용자 글 삭제" });
  return [{ key: "clear", label: "기록 삭제", icon: "trash", sub }, leave];
}

/// 세션 생성 화면 상대 칸 처음 값 — 레일에서 고른 상대가 페어링됐으면 그 id(문자열), 아니면 빈 값("상대 선택").
export function composeDefaults(railPeer, peers) {
  return peers.some((p) => p.id === railPeer && p.paired) ? String(railPeer) : "";
}

/// 서버 알림 → 브라우저 알림. 받은 요청 = 같은 tag(앞 알림을 바꾼다) · 소리 없음(효과음은 웹UI) · 누르면 요청 보드, 그 밖은 종류별 tag · 내 활동
/// (페어링은 호출자가 설정으로). `sound` = 효과음을 낼 알림인가.
export function requestNotice(n) {
  if (n.kind === "request") return { title: n.title, body: n.body, tag: "coworkagent-request", href: "#requests", sound: true };
  return { title: n.title, body: n.body, tag: "coworkagent-" + n.kind, href: "#activity", sound: false };
}

/// 받은 쪽이 멈춘 자동 왕복 — 세부정보 '대화 정보' 줄.
export const RELAY_STOPPED = "멈춤 — 이 PC 가 멈춤(재개는 연 쪽)";

/// 실행 명령 복사 안내 — 명령은 띄운 그대로라 이어 쓰기 세션 인자가 들고, 프롬프트는 표준 입력으로 받는다.
export const INFO_CMD_HINT = "이어 쓰기 세션 인자가 든 명령 그대로 — 터미널에서 돌리면 이 대화 세션에 이어지고, 프롬프트는 표준 입력으로 받습니다. Windows 는 PowerShell 7.3 이상(pwsh)에 붙여 넣으세요.";

/// 세부정보 '대화 정보' 줄(`GET /api/conversations/{id}/info`) → `{icon, label, text, mono, copy}`. 받은 대화(`in`) = 기간 · 답하는 도구 · 읽는 폴더 ·
/// 컨텍스트 · 실행 명령(`copy` = 전체 명령) — 이 PC 가 자동 왕복을 멈췄으면(`relay_blocked` — 대화에서 덧붙임) 컨텍스트 뒤에 '자동 왕복' 줄,
/// 보낸 대화 · 공유 사본 = 기간 · 컨텍스트(실행은 상대 PC 가 정한다).
export function convInfoRows(info, dir, now = Date.now() / 1000) {
  const when = (t) => `${dayLabel(t, now)} ${formatTime(t)}`;
  const mine = dir === "in";
  const ctx = info.compacts ? `압축 ${info.compacts}회 · ${when(info.last_compact_at)}` : "압축 없음";
  const rows = [{ icon: "clock", label: "기간", text: `${when(info.created_at)} 시작 · 마지막 ${when(info.last_at)}` }];
  if (mine)
    rows.push(
      { icon: "bot", label: "답하는 도구", text: info.agent ? engineLabel(info.agent.engine) : "지운 에이전트" },
      { icon: "folder", label: "읽는 폴더", text: info.agent?.folder ?? "—", mono: true },
    );
  rows.push({ icon: "compress", label: "컨텍스트", text: info.clears ? `${ctx} · 비우기 ${info.clears}회` : ctx });
  if (mine && info.relay_blocked) rows.push({ icon: "swap", label: "자동 왕복", text: RELAY_STOPPED });
  if (mine)
    rows.push(
      info.cmd
        ? { icon: "terminal", label: "실행 명령", text: info.cmd.text, mono: true, copy: info.cmd.text }
        : { icon: "terminal", label: "실행 명령", text: "아직 실행 없음" },
    );
  return rows;
}
/// 대화 유형 아이콘 — 내 에이전트가 답(`in`) = 로봇, 내가 문의(`out`) = 사람(사람 ↔ 에이전트 구분, 발화자 표시 `speakerIcon` 과 같은 그림), 공유 대화 = 눈.
export function convKindIcon(conv) {
  return conv.dir === "in" ? "bot" : conv.dir === "out" ? "user" : conv.dir === "shared" ? "eye" : null;
}
/// 대화 유형 이름(유형 아이콘의 읽는 이름 — 공유는 사이드바 절 머리와 같은 말).
export const KIND_NAMES = { in: "내 에이전트 응답", out: "내가 문의", shared: "공유 대화" };

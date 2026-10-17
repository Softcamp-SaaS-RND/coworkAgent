// node --test crates/app/web/ — 화면 로직(view.js) 시험. DOM 없음.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  entryToken,
  DISCONNECTED,
  stateText,
  peerState,
  convLabel,
  filterByPeer,
  authorLabel,
  roomPreview,
  bubbleSide,
  dayKey,
  dayLabel,
  dayBreaks,
  relayText,
  keepsConversation,
  formatDuration,
  formatTime,
  timeline,
  unlinkedReplies,
  route,
  Latest,
  settingsTree,
  searchItems,
  peerSections,
  NO_PEER_GROUP,
  peerMove,
  peerGroupItems,
  josa,
  peerGroupText,
  inboxRows,
  isHtmlDoc,
  htmlSrcdoc,
  markdownBlocks,
  peerMessages,
  presenceOf,
  starredSections,
  displayName,
  convRows,
  tileKeeps,
  sideNav,
  newMenu,
  profileMenu,
  convSections,
  splitByDir,
  splitKey,
  sharedAuthor,
  shareHead,
  shareCandidates,
  MAX_OBSERVERS,
  convMeta,
  unreadByPeer,
  activityRows,
  messageActions,
  editedTag,
  keepPicked,
  insertMarkdown,
  enterSends,
  runSteps,
  searchGroups,
  lastRun,
  presence,
  refusedText,
  progressText,
  outboxText,
  stepText,
  runsSummary,
  portText,
  messagesSigOf,
  askTag,
  reqMoves,
  requestRows,
  peerGroups,
  peerSummary,
  sortByPos,
  inGroup,
  groupPeers,
  scopeQuery,
  groupNameError,
  membershipDiff,
  colorOf,
  moveInOrder,
  stepTarget,
  dropKind,
  withOrder,
  movedText,
  groupLeaves,
  groupUnread,
  orderToApply,
  waitingByPeer,
  SendLatest,
  holdsRender,
  REQ_STATES,
  MODELS,
  TOOLS_NOTE,
  READ_ONLY_TOOLS,
  mentionQuery,
  mentionTarget,
  hasMention,
  mentionToken,
  contextSize,
  contextLabel,
  contextBusy,
  ctxMenu,
  contextNote,
  CONTEXT_LIMIT,
} from "./view.js";

test("상태 문구", () => {
  assert.equal(stateText("running"), "실행 중");
  assert.equal(stateText("undelivered"), "답 미전달");
  assert.equal(stateText("weird"), "weird");
  assert.equal(stateText(undefined), "");
  assert.equal(peerState({ paired: true }), "페어링됨");
  assert.equal(peerState({ paired: false, my_ok: false, their_ok: true }), "내 승인 대기");
  assert.equal(peerState({ paired: false, my_ok: true, their_ok: false }), "상대 승인 대기");
});

test("대화 이름 — 제목 · 상대, 보냄 · 받음 표시 없음(#1j)", () => {
  const names = new Map([[2, "이도윤 · DYOON-MAC"]]);
  assert.deepEqual(convLabel({ peer_id: 2, dir: "out", agent: "review", title: "가드 봐 줘" }, names), {
    title: "가드 봐 줘",
    sub: "이도윤 · DYOON-MAC 의 review",
  });
  assert.deepEqual(convLabel({ peer_id: 9, dir: "in", agent: "dev", title: "" }, names), {
    title: "내 dev 가 받은 요청",
    sub: "알 수 없는 상대 의 요청 · 내 dev",
  });
});

test("레일 거르기", () => {
  const convs = [{ id: 1, peer_id: 2 }, { id: 2, peer_id: 3 }, { id: 3, peer_id: 2 }];
  assert.deepEqual(filterByPeer(convs, null).map((c) => c.id), [1, 2, 3]);
  assert.deepEqual(filterByPeer(convs, 2).map((c) => c.id), [1, 3]);
  assert.deepEqual(filterByPeer(convs, 7), []);
  // 사이드바 목록 — 타일로 거르고, 검색은 제목으로만.
  const names = new Map([[2, "user2 · PC-b"]]);
  const titled = [
    { id: 1, peer_id: 2, dir: "out", agent: "guide", title: "배포 절차" },
    { id: 2, peer_id: 3, dir: "out", agent: "guide", title: "로그인 오류" },
  ];
  assert.deepEqual(convRows(titled, names, null, " 배포 ").map((r) => r.c.id), [1], "제목으로");
  assert.deepEqual(convRows(titled, names, null, "user2"), [], "상대 이름으로는 거르지 않는다(타일이 한다)");
  assert.deepEqual(convRows(titled, names, 3).map((r) => r.c.id), [2]);
  assert.equal(convRows(titled, names, null, "LOGIN").length, 0);
});

test("레일 전환 — 다른 PC 를 고르면 열린 대화를 닫는다", () => {
  const conv = { id: 5, peer_id: 2 };
  assert.equal(keepsConversation(conv, null), true, "내 PC = 전부");
  assert.equal(keepsConversation(conv, 2), true);
  assert.equal(keepsConversation(conv, 3), false);
  assert.equal(keepsConversation(null, 2), false);
  // 타일을 눌렀을 때 — 그 타일의 대화 화면이면 그대로, 아니면 안내로.
  assert.equal(tileKeeps("conversation", conv, 2), true);
  assert.equal(tileKeeps("sent", conv, null), true, "'나' = 전부");
  assert.equal(tileKeeps("conversation", conv, 3), false);
  assert.equal(tileKeeps("activity", conv, 2), false, "대화 화면이 아니면 안내");
  assert.equal(tileKeeps("conversation", null, null), false);
});

test("시간 표기", () => {
  assert.equal(formatDuration(null), "");
  assert.equal(formatDuration(850), "850ms");
  assert.equal(formatDuration(18400), "18.4초");
  assert.equal(formatDuration(125000), "2분 5초");
  const t = 1791691336;
  const d = new Date(t * 1000);
  assert.equal(formatTime(t), `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`);
  assert.equal(formatTime(null), "");
});

test("타임라인 — reply_to 로 묶고, 실행 정보 · 대기 · 미전달", () => {
  const ms = [
    { id: 1, author: "human", text: "하나", state: "done", created_at: 100 },
    { id: 2, author: "human", text: "둘", state: "sent", created_at: 101 },
    { id: 3, author: "agent", text: "답 하나", state: "done", created_at: 120, reply_to: 1, turns: 3, duration_ms: 18400 },
  ];
  const tl = timeline(ms);
  assert.equal(tl.length, 2);
  assert.equal(tl[0].reply.id, 3);
  assert.deepEqual(tl[0].steps.map((s) => s.title), ["요청", "실행", "응답"]);
  assert.equal(tl[0].steps[1].meta, "3턴 · 18.4초");
  assert.equal(tl[1].reply, null, "답 없는 요청");
  assert.equal(tl[1].steps[2].title, "응답 대기");
  assert.equal(tl[1].steps[1].meta, "보냄");
  // 실행 정보 없는 답 · 미전달 답.
  const tl2 = timeline([
    { id: 1, author: "human", text: "x", state: "error", created_at: 1 },
    { id: 2, author: "agent", text: "실패", state: "undelivered", created_at: 2, reply_to: 1 },
  ]);
  assert.equal(tl2[0].steps[1].meta, "실행 정보 없음");
  assert.equal(tl2[0].steps[2].title, "응답 · 답 미전달");
});

test("타임라인 — 스키마 5 이전 이력(reply_to 없음)은 기다리는 요청이 하나일 때만 연결", () => {
  const tl = timeline([
    { id: 1, author: "human", text: "a" },
    { id: 2, author: "agent", text: "A" },
    { id: 3, author: "human", text: "b" },
    { id: 4, author: "agent", text: "B" },
  ]);
  assert.deepEqual(tl.map((x) => x.reply?.text), ["A", "B"]);
  // 겹친 요청(H1,H2,A1,A2) — 어느 답인지 모르므로 연결하지 않고 따로 보인다.
  const overlap = [
    { id: 1, author: "human", text: "h1" },
    { id: 2, author: "human", text: "h2" },
    { id: 3, author: "agent", text: "a1" },
    { id: 4, author: "agent", text: "a2" },
  ];
  assert.deepEqual(timeline(overlap).map((x) => x.reply), [null, null]);
  assert.deepEqual(unlinkedReplies(overlap).map((m) => m.text), ["a1", "a2"]);
  // reply_to 가 있는 답은 겹쳐도 정확히.
  const explicit = timeline([
    { id: 1, author: "human", text: "h1" },
    { id: 2, author: "human", text: "h2" },
    { id: 3, author: "agent", text: "for h2", reply_to: 2 },
    { id: 4, author: "agent", text: "for h1", reply_to: 1 },
  ]);
  assert.deepEqual(explicit.map((x) => x.reply.text), ["for h1", "for h2"]);
});

test("화면 위치", () => {
  assert.deepEqual(route("#c/12"), { view: "conversation", id: 12 });
  assert.deepEqual(route("#c/12/m/40"), { view: "conversation", id: 12, msg: 40 }, "그 글 위치");
  assert.deepEqual(route("#s/3"), { view: "sent", id: 3 });
  assert.deepEqual(route("#p/2"), { view: "peer", id: 2 });
  assert.deepEqual(route("#activity"), { view: "activity" });
  assert.deepEqual(route("#saved"), { view: "saved" });
  assert.deepEqual(route("#starred"), { view: "starred" }, "즐겨찾기(#1j)");
  assert.deepEqual(route("#agents"), { view: "agents" });
  assert.deepEqual(route("#me"), { view: "home" }, "내 정보는 프로필 메뉴 팝업(#1l) — 경로 없음");
  assert.deepEqual(route("#settings"), { view: "settings", focus: null });
  assert.deepEqual(route("#settings/invite_accept"), { view: "settings", focus: "invite_accept" });
  assert.deepEqual(route("#settings/nope"), { view: "settings", focus: null }, "모르는 절 = 설정 처음");
  assert.deepEqual(route("#new"), { view: "new" });
  assert.deepEqual(route(""), { view: "home" });
  assert.deepEqual(route("#c/abc"), { view: "home" });
  assert.deepEqual(route("#inbox"), { view: "home" }, "없어진 화면 — 내 활동이 대신한다");
  assert.deepEqual(route("#t/3"), { view: "home" }, "없어진 화면 — 실행 상세가 대신한다");
});

test("늦게 온 응답은 버린다", () => {
  const latest = new Latest();
  const a = latest.next(); // 대화 A 조회 시작
  const b = latest.next(); // 그 사이 대화 B 로 전환
  assert.equal(latest.is(a), false, "A 응답은 늦었다");
  assert.equal(latest.is(b), true);
});

test("발화자 이름 — 양쪽 사람 · 에이전트 주인", () => {
  const out = { dir: "out", agent: "review" };
  const inn = { dir: "in", agent: "dev" };
  assert.equal(authorLabel(out, { author: "human", mine: true }, "이도윤 · DYOON"), "나");
  assert.equal(authorLabel(out, { author: "human", mine: false }, "이도윤 · DYOON"), "이도윤");
  assert.equal(authorLabel(out, { author: "agent" }, "이도윤 · DYOON"), "이도윤/review");
  assert.equal(authorLabel(inn, { author: "agent" }, "이도윤 · DYOON"), "내 dev");
  assert.equal(authorLabel(inn, { author: "human", mine: false }, undefined), "상대");
  assert.equal(authorLabel(out, { author: "agent" }, "세일즈 · 김팀장 · PC-b"), "세일즈 · 김팀장/review", "표시 이름의 · 는 자르지 않는다");
});

test("자동 왕복 — 누구의 에이전트인지 · 머리글 문구", () => {
  const out = { dir: "out", agent: "review", relay_agent: "dev", relay_cap: 10, relay_count: 3, relay_on: true };
  const inn = { dir: "in", agent: "review", relay_blocked: false };
  assert.equal(authorLabel(out, { author: "agent", mine: true }, "이도윤 · DYOON"), "내 dev");
  assert.equal(authorLabel(out, { author: "agent", mine: false }, "이도윤 · DYOON"), "이도윤/review");
  assert.equal(authorLabel(inn, { author: "agent", mine: false }, "김 · KIM"), "김/에이전트");
  assert.equal(authorLabel({ ...inn, relay_agent: "dev" }, { author: "agent", mine: false }, "김 · KIM"), "김/dev");
  assert.equal(authorLabel(inn, { author: "agent", mine: true }, "김 · KIM"), "내 review");
  assert.deepEqual(relayText(out, []), { text: "자동 왕복 3 / 10 · 진행 중", action: "stop" });
  assert.deepEqual(relayText({ ...out, relay_on: false }, []), { text: "자동 왕복 3 / 10 · 멈춤", action: "resume" });
  assert.equal(relayText({ dir: "out", agent: "review", relay_agent: null }, []), null);
  assert.equal(relayText(inn, [{ author: "agent", mine: true }]), null, "보통 받은 대화");
  assert.deepEqual(relayText(inn, [{ author: "agent", mine: false }]), { text: "자동 왕복 받는 중", action: "stop" });
  assert.equal(relayText({ ...inn, relay_blocked: true }, []).action, null, "받은 쪽은 재개 없음");
});

test("타임라인 — 실행 없는 사람 글은 요청이 아니다", () => {
  const ms = [
    { id: 1, author: "human", exec: true, text: "q" },
    { id: 2, author: "human", exec: false, text: "메모" },
    { id: 3, author: "agent", text: "a" },
  ];
  const tl = timeline(ms);
  assert.deepEqual(tl.map((x) => x.request.text), ["q"]);
  assert.equal(tl[0].reply.text, "a", "메모가 끼어도 이전 이력 답은 기다리는 요청 하나에");
});

test("CLI 출처 — 보는 쪽 기준 라벨 · 글 종류 · 타임라인", () => {
  const out = { dir: "out", agent: "review" };
  const inn = { dir: "in", agent: "review" };
  assert.equal(authorLabel(out, { author: "human", mine: true, via: "cli" }, "이도윤 · DYOON"), "내 AI 에이전트(CLI)");
  assert.equal(authorLabel(inn, { author: "human", mine: false, via: "cli" }, "김 · KIM"), "김/AI 에이전트(CLI)");
  assert.equal(authorLabel(out, { author: "human", mine: true, via: null }, "이도윤 · DYOON"), "나", "웹UI 글은 그대로");
  const items = timeline([
    { id: 1, author: "human", exec: true, text: "q", via: "cli" },
    { id: 2, author: "agent", reply_to: 1, text: "a", state: "done" },
    { id: 3, author: "human", exec: true, text: "w" },
  ]);
  assert.equal(items[0].steps[0].title, "요청 · AI 에이전트(CLI)");
  assert.equal(items[1].steps[0].title, "요청");
});

test("settings_tree — 설정 항목 전부(한 열 순서 — 내 정보 없음, #1l) · 연결 아래 초대 생성 · 초대 수락 · 연결 목록", () => {
  const tree = settingsTree();
  const flat = tree.flatMap((n) => [n, ...(n.children ?? [])]);
  assert.deepEqual(
    flat.map((n) => n.label),
    ["연결", "초대 생성", "초대 수락", "연결 목록", "내 에이전트", "실행", "화면", "시작", "알림"],
  );
  assert.equal(new Set(flat.map((n) => n.id)).size, flat.length, "id 는 절 요소 id — 겹치지 않는다");
  const html = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  for (const n of flat) assert.ok(html.includes(`<section id="${n.id}"`), `index.html 에 절 ${n.id}`);
  assert.ok(!html.includes(`<section id="me"`), "설정에 내 정보 카드 없음(#1l)");
  const order = [...html.matchAll(/<section id="(\w+)" class="card/g)].map((m) => m[1]);
  assert.deepEqual(order, ["conn", "peers", "agents", "run", "screen", "startup", "notify"], "카드 한 열 순서 = 트리 순서");
});

test("menu_icons — 사이드바 · 설정 트리 · 새로 만들기 항목마다 아이콘, 모두 app.css 에 정의(#1l)", () => {
  const css = readFileSync(new URL("./app.css", import.meta.url), "utf8");
  const items = [
    ...[null, 3, "group"].flatMap((t) => sideNav(t).filter((n) => !n.sep)),
    ...settingsTree().flatMap((n) => [n, ...(n.children ?? [])]),
    ...newMenu(),
  ];
  for (const i of items) {
    assert.ok(i.icon, `${i.label} 아이콘`);
    assert.ok(css.includes(`.ico-${i.icon} {`), `app.css .ico-${i.icon}`);
  }
  assert.deepEqual(
    sideNav(null).filter((n) => !n.sep).map((n) => n.icon),
    ["home", "bell", "inbox", "swap", "star", "bookmark", "bot"],
  );
});

test("markdown_blocks_safe — 블록 · 인라인, HTML · 링크는 글자 그대로", () => {
  const blocks = markdownBlocks(
    [
      "# 배포 절차",
      "결제 모듈은 **스테이징 검증 뒤** 운영에 `main` 으로.",
      "",
      "1. 브랜치",
      "2. 점검표",
      "- 롤백",
      "",
      "| 환경 | 승인 |",
      "|---|---|",
      "| 운영 | *리드* |",
      "> 주의 <script>alert(1)</script>",
      "```",
      "<img src=x onerror=alert(1)>",
      "```",
      "[링크](javascript:alert(1)) <b>굵게</b>",
    ].join("\n"),
  );
  assert.deepEqual(
    blocks.map((b) => b.type),
    ["h", "p", "ol", "ul", "table", "quote", "code", "p"],
  );
  assert.deepEqual(blocks[1].inlines, [
    { t: "text", v: "결제 모듈은 " },
    { t: "b", v: "스테이징 검증 뒤" },
    { t: "text", v: " 운영에 " },
    { t: "code", v: "main" },
    { t: "text", v: " 으로." },
  ]);
  assert.equal(blocks[2].items.length, 2);
  assert.deepEqual(blocks[4].head.map((c) => c[0].v), ["환경", "승인"]);
  assert.deepEqual(blocks[4].rows[0][1], [{ t: "i", v: "리드" }]);
  assert.equal(blocks[5].inlines[0].v, "주의 <script>alert(1)</script>", "태그는 글자");
  assert.equal(blocks[6].text, "<img src=x onerror=alert(1)>");
  assert.deepEqual(blocks[7].inlines, [{ t: "text", v: "[링크](javascript:alert(1)) <b>굵게</b>" }], "링크 · 태그는 글자");
  assert.deepEqual(markdownBlocks("설정_키_값 과 _기울임_")[0].inlines, [{ t: "text", v: "설정_키_값 과 " }, { t: "i", v: "기울임" }], "낱말 안 _ 는 글자");
  // 블록 · 인라인 종류는 정해진 것뿐 — 그리는 쪽이 고를 태그가 정해져 있다.
  const kinds = new Set();
  const walk = (x) => (Array.isArray(x) ? x.forEach(walk) : x && typeof x === "object" && (x.t && kinds.add(x.t), Object.values(x).forEach(walk)));
  walk(blocks);
  assert.deepEqual([...kinds].sort(), ["b", "code", "i", "text"]);
});

test("html_doc_detect — HTML 문서만 sandbox, CSP 를 앞에", () => {
  assert.equal(isHtmlDoc("<!DOCTYPE html><html><body>x</body></html>"), true);
  assert.equal(isHtmlDoc("  \n<html lang=ko>"), true);
  assert.equal(isHtmlDoc("<p>조각</p>"), false);
  assert.equal(isHtmlDoc("# 제목\n<html>"), false);
  assert.equal(isHtmlDoc("<htmlx>"), false);
  const doc = htmlSrcdoc("<!doctype html><p>x</p>");
  assert.ok(doc.startsWith('<meta http-equiv="Content-Security-Policy" content="default-src \'none\'; style-src \'unsafe-inline\'; img-src data:">'));
  assert.ok(doc.includes('<base target="_top">'), "링크로 프레임이 밖으로 가지 않게");
  assert.ok(doc.endsWith("<!doctype html><p>x</p>"));
});

test("search_items — 고른 칸에 부분 일치 · 대소문자 · 앞뒤 공백 무시, 빈 검색이면 전부(순서 그대로)", () => {
  const peers = [
    { id: 1, name: "user2", host: "DEV-PC" },
    { id: 2, name: "kim", host: "QA-01", alias: "김 대리" },
    { id: 3, name: "lee", host: "DEV-LAP" },
  ];
  const fields = ["alias", "name", "host"];
  assert.deepEqual(searchItems(peers, " dev ", fields).map((p) => p.id), [1, 3]);
  assert.deepEqual(searchItems(peers, "김", fields).map((p) => p.id), [2], "표시 이름도");
  assert.deepEqual(searchItems(peers, "", fields), peers);
  assert.deepEqual(searchItems(peers, undefined, fields), peers);
  assert.deepEqual(searchItems(peers, "없음", fields), []);
});

test("peer_sections — 레일 그룹 순서의 절(한 상대는 여러 절에, 빈 그룹 절도) + 끝에 '그룹에 없는 연결', 상대는 준 순서", () => {
  const peers = [{ id: 3 }, { id: 1 }, { id: 2 }];
  const groups = [
    { id: 7, name: "결제", peers: [1, 3] },
    { id: 5, name: "빈 그룹", peers: [] },
    { id: 9, name: "QA", peers: [1, 99] },
  ];
  assert.deepEqual(
    peerSections(peers, groups).map((s) => [s.key, s.group, s.title, s.peers.map((p) => p.id)]),
    [
      ["g:7", 7, "결제", [3, 1]],
      ["g:5", 5, "빈 그룹", []],
      ["g:9", 9, "QA", [1]],
      ["none", null, NO_PEER_GROUP, [2]],
    ],
  );
  assert.equal(NO_PEER_GROUP, "그룹에 없는 연결");
  assert.deepEqual(peerSections(peers, []).map((s) => [s.key, s.peers.length]), [["none", 3]], "그룹이 없으면 한 절");
  assert.deepEqual(peerSections([], groups).at(-1).peers, []);
  assert.deepEqual(peerSections([{ id: 1 }], [{ id: 4, name: "옛 응답" }])[0].peers, [], "peers 없는 그룹(옛 서버)도");
});

test("peer_move — 놓은 그룹에 넣고 옛 그룹에서 뺀다, 그룹 밖에서 오면 넣기만 · 그룹 밖에 놓으면 빼기만", () => {
  const groups = [
    { id: 1, peers: [5] },
    { id: 2, peers: [] },
    { id: 3, peers: [5] },
  ];
  assert.deepEqual(peerMove(5, 1, 2, groups), [
    { group: 2, add: [5], remove: [] },
    { group: 1, add: [], remove: [5] },
  ]);
  assert.deepEqual(peerMove(5, null, 2, groups), [{ group: 2, add: [5], remove: [] }]);
  assert.deepEqual(peerMove(5, 1, null, groups), [{ group: 1, add: [], remove: [5] }]);
  assert.deepEqual(peerMove(5, 1, 3, groups), [{ group: 1, add: [], remove: [5] }], "이미 있는 그룹이면 옛 절에서 빼기만");
  assert.deepEqual(peerMove(5, 2, 2, groups), [], "같은 절");
  assert.deepEqual(peerMove(5, null, null, groups), []);
});

test("peer_move_round_trip — 놓기 결과를 그룹에 적용하면 그 상대는 놓은 절에만 보인다(옛 절 · 그룹 밖 절에서 빠짐)", () => {
  const peers = [{ id: 5 }, { id: 6 }];
  let groups = [
    { id: 1, name: "가", peers: [5] },
    { id: 2, name: "나", peers: [] },
  ];
  const apply = (ops) => {
    for (const op of ops) {
      groups = groups.map((g) => (g.id === op.group ? { ...g, peers: [...g.peers.filter((x) => !op.remove.includes(x)), ...op.add] } : g));
    }
  };
  const where = (id) => peerSections(peers, groups).filter((s) => s.peers.some((p) => p.id === id)).map((s) => s.key);
  apply(peerMove(5, 1, 2, groups));
  assert.deepEqual(where(5), ["g:2"]);
  apply(peerMove(6, null, 1, groups));
  assert.deepEqual(where(6), ["g:1"], "그룹 밖에서 넣기");
  apply(peerMove(5, 2, null, groups));
  assert.deepEqual(where(5), ["none"], "그룹 밖으로 빼기");
});

test("group_summary_peer_member — 그룹 개요의 상대 머리는 그 상대를 넣은 그룹이면 대화를 하나씩 넣지 않아도 나온다", () => {
  const convs = [
    { id: 1, peer_id: 2, last_at: 10 },
    { id: 2, peer_id: 3, last_at: 20 },
  ];
  const g = { id: 1, convs: [], peers: [3] };
  const scoped = inGroup(convs, g);
  assert.deepEqual(groupPeers(scoped, [2, 3]), [3]);
  const names = new Map([[3, "최유나 · DEV"]]);
  assert.deepEqual(peerSummary(scoped, groupPeers(scoped, [2, 3]), names).map((x) => [x.peer, x.convs]), [[3, 1]]);
});

test("room_preview_cli — 마지막 글이 AI 에이전트(CLI) 가 쓴 글이면 말한 이도 CLI(라운드 2 — 서버 last.via)", () => {
  const conv = { dir: "out", agent: "guide", relay_agent: null };
  assert.equal(roomPreview({ ...conv, last: { author: "human", mine: true, via: "cli", text: "빌드 로그 봐 줘" } }, "이준호 · QA"), "내 AI 에이전트(CLI): 빌드 로그 봐 줘");
  assert.equal(roomPreview({ ...conv, dir: "in", last: { author: "human", mine: false, via: "cli", text: "x" } }, "이준호 · QA"), "이준호/AI 에이전트(CLI): x");
  assert.equal(roomPreview({ ...conv, last: { author: "human", mine: true, via: null, text: "y" } }, "이준호 · QA"), "나: y", "CLI 가 아니면 그대로");
});

test("in_group_peers — 그룹의 대화 = 하나씩 넣은 대화 + 넣은 상대의 대화 전부(뒤에 생긴 것도, #1j)", () => {
  const convs = [
    { id: 1, peer_id: 2 },
    { id: 2, peer_id: 3 },
    { id: 3, peer_id: 2 },
    { id: 4, peer_id: 4 },
  ];
  assert.deepEqual(inGroup(convs, { convs: [2], peers: [2] }).map((c) => c.id), [1, 2, 3]);
  assert.deepEqual(inGroup(convs, { convs: [4] }).map((c) => c.id), [4], "peers 없는 그룹(옛 응답)");
  assert.deepEqual(inGroup(convs, { convs: [], peers: [9] }), []);
  assert.deepEqual(groupUnread([{ id: 1, peer_id: 2, unread: 2 }, { id: 5, peer_id: 7, unread: 4 }], { convs: [], peers: [2] }), 2);
});

test("peer_group_items — 상대의 그룹 고르기: 레일 그룹 순서 · 넣은 그룹 체크 · 그 그룹에 보이는 대화 수", () => {
  const convs = [
    { id: 1, peer_id: 2 },
    { id: 2, peer_id: 3 },
  ];
  const groups = [
    { id: 8, name: "결제", convs: [2], peers: [2] },
    { id: 4, name: "QA", convs: [], peers: [] },
  ];
  assert.deepEqual(peerGroupItems(groups, convs, 2), [
    { id: 8, name: "결제", checked: true, count: 2 },
    { id: 4, name: "QA", checked: false, count: 0 },
  ]);
  assert.deepEqual(peerGroupItems(groups, convs, 3).map((g) => g.checked), [false, false], "대화만 넣은 상대는 체크 아님");
  assert.deepEqual(peerGroupItems([], convs, 2), []);
});

test("josa — 받침에 맞춘 조사, 한글이 아니면 둘 다", () => {
  assert.equal(josa("이준호", "을", "를"), "를");
  assert.equal(josa("박지훈", "을", "를"), "을");
  assert.equal(josa("최유나", "과", "와"), "와");
  assert.equal(josa("결제 모듈", "과", "와"), "과");
  assert.equal(josa("build", "을", "를"), "을(를)");
  assert.equal(josa("PC-07", "과", "와"), "과(와)");
  assert.equal(josa("", "을", "를"), "을(를)");
});

test("peer_group_text — 상대를 그룹에 넣고 뺀 안내", () => {
  assert.equal(peerGroupText("이준호", "결제 모듈", true), '이준호를 그룹 "결제 모듈" 에 넣었습니다');
  assert.equal(peerGroupText("박지훈", "QA", false), '박지훈을 그룹 "QA" 에서 뺐습니다');
  assert.ok(peerGroupText("build", "QA", true).startsWith("build을(를) "));
});

test("inbox_rows_agent — 주고받은 메시지 칩(#1k): 보낸 대화 = 상대 에이전트, 받은 대화 = 내 에이전트 · 행마다 상대 id", () => {
  const names = new Map([[2, "이준호 · QA"]]);
  const rows = inboxRows(
    [
      { id: 1, author: "agent", mine: false, text: "답", created_at: 2, conversation: { id: 4, peer_id: 2, agent: "guide", dir: "out" } },
      { id: 2, author: "human", mine: false, text: "요청", created_at: 1, conversation: { id: 5, peer_id: 2, agent: "review", dir: "in" } },
    ],
    names,
  );
  assert.deepEqual(rows.map((r) => [r.agent, r.peer]), [["guide", 2], ["내 review", 2]]);
});

test("message_actions_no_resend — 다시 보내기 없음(#1k), 받은 요청 상태 · 다시 알리기는 남는다", () => {
  const keys = (m) => messageActions(m).map((a) => a.key);
  for (const m of [{ author: "human", mine: true, exec: true }, { author: "human", mine: true, request: true, req_state: "wait" }, { author: "human", mine: true, exec: false }]) {
    assert.ok(!keys(m).includes("resend"), JSON.stringify(m));
  }
  assert.deepEqual(keys({ author: "human", mine: false, request: true, req_state: "wait" }), ["req:accepted", "req:rejected"]);
});

test("inbox_rows — 메시지 행 · 종류 거르기 · 누르면 그 대화", () => {
  const names = new Map([[2, "user2 · DEV-PC"]]);
  const ms = [
    { id: 9, author: "human", exec: true, mine: false, text: "로그인\n오류 정리", created_at: 100, conversation: { id: 5, peer_id: 2, agent: "guide", dir: "in" } },
    { id: 8, author: "agent", mine: false, text: "x".repeat(130), created_at: 90, conversation: { id: 4, peer_id: 2, agent: "review", dir: "out" } },
    { id: 7, author: "human", mine: false, text: "고아", created_at: 80, conversation: null },
  ];
  const rows = inboxRows(ms, names);
  assert.deepEqual(
    rows.map((r) => [r.id, r.who, r.kind, r.room, r.href]),
    [
      [9, "user2", "", "", "#c/5"],
      [8, "user2/review", "", "", "#c/4"],
    ],
    "방향 라벨(사람 → 에이전트 · 보냄 · 받음) 없음(#1j)",
  );
  assert.ok(rows.every((r) => !("tag" in r)));
  const req = inboxRows([{ ...ms[0], request: true, req_state: "wait", conversation: { ...ms[0].conversation, title: "로그인 오류" } }], names)[0];
  assert.deepEqual([req.kind, req.room], ["요청 · 대기", "로그인 오류"]);
  assert.equal(rows[0].preview, "로그인 오류 정리");
  assert.equal(rows[1].preview.length, 121, "120자 + …");
  assert.deepEqual(inboxRows(ms, names, { kind: "agent" }).map((r) => r.id), [8]);
  assert.deepEqual(inboxRows(ms, names, { kind: "human" }).map((r) => r.id), [9]);
});

test("peer_messages — 레일 상대 = 주고받은 메시지, 최신순 합침 · 상한", () => {
  const theirs = [
    { id: 9, created_at: 300 },
    { id: 4, created_at: 100 },
  ];
  const mine = [
    { id: 8, created_at: 300 },
    { id: 6, created_at: 200 },
  ];
  assert.deepEqual(peerMessages(theirs, mine).map((m) => m.id), [9, 8, 6, 4], "같은 시각은 id 큰 것 먼저");
  assert.deepEqual(peerMessages(theirs, mine, 3).map((m) => m.id), [9, 8, 6]);
  assert.deepEqual(peerMessages([], []), []);
});

test("direction_tag — 방향 표시 없음(#1j — 보냄 · 받음 태그도 없앰)", () => {
  const names = new Map([[2, "user2 · DEV-PC"]]);
  const out = convLabel({ peer_id: 2, dir: "out", agent: "guide", title: "→ 배포 절차" }, names);
  const inn = convLabel({ peer_id: 2, dir: "in", agent: "guide", title: "" }, names);
  assert.deepEqual([out.tag, inn.tag], [undefined, undefined]);
  assert.equal(out.title, "→ 배포 절차", "제목 글자는 건드리지 않는다");
  for (const l of [out, inn]) assert.ok(!/[→←]/.test(l.sub), l.sub);
  assert.ok(!/[→←]/.test(inn.title));
});

test("display_name — 이 PC 에서 부르는 이름 = 표시 이름(없거나 빈 글자면 원래 이름)", () => {
  assert.equal(displayName({ name: "user-b", alias: "세일즈" }), "세일즈");
  assert.equal(displayName({ name: "user-b", alias: null }), "user-b");
  assert.equal(displayName({ name: "user-b", alias: "" }), "user-b", "빈 표시 이름 = 원래 이름");
});

test("side_nav — '나' 메뉴 7개 세 절(개요 | 내 활동 · 요청 · 주고받은 메시지 | 즐겨찾기 · 북마크 · 에이전트), 상대 타일 4개 두 절(요청 | 즐겨찾기 · 북마크 · 에이전트 — #1j)", () => {
  const show = (xs) => xs.map((n) => (n.sep ? "|" : [n.label, n.href, n.count ?? 0]));
  assert.deepEqual(show(sideNav(null, 3, 2, 4)), [
    ["개요", "#", 0],
    "|",
    ["내 활동", "#activity", 3],
    ["요청", "#requests", 2],
    ["주고받은 메시지", "#p", 0],
    "|",
    ["즐겨찾기", "#starred", 4],
    ["북마크", "#saved", 0],
    ["에이전트", "#agents", 0],
  ]);
  const peer = sideNav(2, 5, 1);
  assert.deepEqual(show(peer), [["요청", "#requests", 1], "|", ["즐겨찾기", "#starred", 0], ["북마크", "#saved", 0], ["에이전트", "#agents", 0]]);
  assert.deepEqual(sideNav(2)[0].count, 0, "대기 수 기본 0");
  assert.deepEqual(peer.filter((n) => !n.sep).map((n) => route(n.href).view), ["requests", "starred", "saved", "agents"]);
  assert.ok(!sideNav(2)[0].sep && !sideNav(2).at(-1).sep, "구분선은 절 사이에만 — 머리 · 끝에 없다");
  assert.deepEqual(route("#p/2"), { view: "peer", id: 2 }, "옛 주소 — 화면은 그 상대 개요로 넘긴다");
  assert.deepEqual(route("#p"), { view: "peer", id: null }, "'나' 주고받은 메시지 = 전부");
});

test("side_nav_exchanged_me_only — '주고받은 메시지' 은 '나' 에만, 상대에는 '개요' 메뉴도 없다(첫 화면이 그 상대 개요 — #1j)", () => {
  const labels = (tile) => sideNav(tile).filter((n) => !n.sep).map((n) => n.label);
  assert.ok(labels(null).includes("주고받은 메시지"));
  for (const tile of [7, "group"]) assert.ok(!labels(tile).includes("주고받은 메시지"), String(tile));
  assert.ok(!labels(7).includes("개요"));
  assert.deepEqual(labels("group").slice(0, 1), ["개요"]);
});

test("side_nav_starred — 즐겨찾기는 모든 타일의 하단 첫 칸, 수 = 그 범위 ★ 대화", () => {
  for (const tile of [null, 3, "group"]) {
    const xs = sideNav(tile, 0, 0, 2);
    const i = xs.findIndex((n) => n.label === "즐겨찾기");
    assert.ok(xs[i - 1].sep, `${tile} — 하단 절의 처음`);
    assert.deepEqual([xs[i].href, xs[i].count, xs[i + 1].label], ["#starred", 2, "북마크"], String(tile));
  }
  assert.equal(sideNav(null).find((n) => n.label === "즐겨찾기").count, 0, "기본 0");
});

test("peer_groups — 상대별 절: 레일(페어링) 순서, 그 뒤 풀린 상대 id 순, 행 없는 상대 숨김, 절 안 순서 그대로", () => {
  const names = new Map([
    [1, "가람 · A-PC"],
    [2, "나래 · B-PC"],
    [3, "다솜 · C-PC"],
    [9, "옛친구 · Z-PC"],
    [7, "옛동료 · Y-PC"],
  ]);
  const rows = [
    { id: 10, peer: 2 },
    { id: 11, peer: 9 },
    { id: 12, peer: 1 },
    { id: 13, peer: 2 },
    { id: 14, peer: 7 },
  ];
  const g = peerGroups(rows, [3, 2, 1], names);
  assert.deepEqual(
    g.map((x) => [x.peer, x.name, x.rows.map((r) => r.id)]),
    [
      [2, "나래", [10, 13]],
      [1, "가람", [12]],
      [7, "옛동료", [14]],
      [9, "옛친구", [11]],
    ],
    "3 은 행이 없어 숨김",
  );
  assert.deepEqual(peerGroups([], [1, 2], names), []);
});

test("row_peer — 주고받은 메시지 · 요청 · 내 활동 행에 상대 id(상대별 절 열쇠)", () => {
  const conv = { id: 5, peer_id: 4, dir: "out", agent: "guide", title: "" };
  const names = new Map([[4, "라온 · D-PC"]]);
  const m = { id: 1, author: "human", mine: true, text: "x", created_at: 1, conversation: conv, request: true };
  assert.equal(inboxRows([m], names)[0].peer, 4);
  assert.equal(requestRows([m], names)[0].peer, 4);
  assert.equal(activityRows([{ kind: "relay_start", at: 1, conversation: conv }], names)[0].peer, 4);
  assert.equal(activityRows([{ kind: "message", at: 1, conversation: conv, message: m }], names)[0].peer, 4);
});

test("waiting_by_peer — 받은 대기 요청을 상대별로 센다", () => {
  const ms = [
    { id: 1, conversation: { peer_id: 2 } },
    { id: 2, conversation: { peer_id: 2 } },
    { id: 3, conversation: { peer_id: 5 } },
    { id: 4 },
  ];
  assert.deepEqual([...waitingByPeer(ms)], [
    [2, 2],
    [5, 1],
  ]);
});

test("peer_summary — '나' 대화 요약: 페어링된 상대는 대화 없어도, 풀린 상대는 대화 있을 때만 · 수 · 안 읽음 · 대기 · 손으로 놓은 순서(#1i)", () => {
  const names = new Map([
    [1, "가람 · A-PC"],
    [2, "나래 · B-PC"],
    [8, "옛친구 · Z-PC"],
  ]);
  const convs = [
    { id: 11, peer_id: 1, unread: 2, last_at: 100, pos: 2 },
    { id: 12, peer_id: 1, unread: 0, last_at: 300, pos: 5 },
    { id: 13, peer_id: 1, unread: 1, last_at: null, pos: -1 },
    { id: 14, peer_id: 8, unread: 0, last_at: 50, pos: 0 },
  ];
  const s = peerSummary(convs, [2, 1], names, new Map([[1, 3]]));
  assert.deepEqual(
    s.map((x) => [x.peer, x.name, x.convs, x.unread, x.waiting, x.last, x.rows.map((c) => c.id)]),
    [
      [2, "나래", 0, 0, 0, null, []],
      [1, "가람", 3, 3, 3, 300, [13, 11, 12]],
      [8, "옛친구", 1, 0, 0, 50, [14]],
    ],
  );
  assert.deepEqual(peerSummary([], [], names), []);
});

test("conv_sections_by_peer — '나' 사이드바: 상대별 절(★ 대화도 제자리 — #1j), 대화가 없으면 빈 '대화' 절", () => {
  const names = new Map([
    [1, "가람 · A-PC"],
    [2, "나래 · B-PC"],
  ]);
  const rows = [
    { c: { id: 1, peer_id: 1, starred: false } },
    { c: { id: 2, peer_id: 2, starred: true } },
    { c: { id: 3, peer_id: 2, starred: false } },
    { c: { id: 4, peer_id: 1, starred: true } },
  ];
  assert.deepEqual(
    convSections(rows, [2, 1], names).map((x) => [x.key, x.title, x.peer ?? null, x.rows.map((r) => r.c.id)]),
    [
      ["peer:2", "나래", 2, [2, 3]],
      ["peer:1", "가람", 1, [1, 4]],
    ],
  );
  assert.deepEqual(convSections([], [1], names).map((x) => [x.key, x.rows.length]), [["convs", 0]]);
  assert.deepEqual(convSections(rows).map((x) => [x.key, x.title, x.rows.length]), [["convs", "대화", 4]], "상대 타일 = 대화 절 하나");
});

test("icons_defined — app.js ico(이름) · index.html ico-이름 으로 쓴 아이콘이 모두 app.css 에 있다(#1l — 빠진 아이콘은 빈 칸으로 보임)", () => {
  const css = readFileSync(new URL("./app.css", import.meta.url), "utf8");
  const app = readFileSync(new URL("./app.js", import.meta.url), "utf8");
  const html = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  const used = new Set([...app.matchAll(/ico\("([a-z-]+)"\)/g), ...html.matchAll(/class="ico ico-([a-z-]+)"/g)].map((m) => m[1]));
  for (const name of ["arrow-up", "arrow-down"]) used.add(name); // 삼항으로 고르는 이름
  // 타일 ⋯ 메뉴 `item(icon, text, onclick, cls)` — 첫 글자 인자는 아이콘 이름이어야 한다(#1l 라운드 1 — "그룹 삭제" 가 아이콘 자리에 들어가 메뉴가 예외로 멈췄다).
  const items = [...app.matchAll(/\bitem\(\s*"([^"]+)"/g)].map((m) => m[1]);
  assert.ok(items.length >= 3, `item 호출 ${items.length}`);
  for (const name of items) assert.match(name, /^[a-z-]+$/, `item 첫 인자 ${name} — 아이콘 이름 자리`);
  for (const name of items) used.add(name);
  assert.ok(used.size >= 15, `쓰인 아이콘 ${used.size}`);
  for (const name of used) assert.ok(css.includes(`.ico-${name} {`), `app.css .ico-${name}`);
});

test("new_menu — 네 항목이 각 화면으로, 초점 칸까지(자동 왕복은 새 대화 화면 안 — #1l)", () => {
  const items = newMenu();
  assert.deepEqual(items.map((i) => i.label), ["새 대화", "초대 생성", "초대 수락", "에이전트 등록"]);
  assert.deepEqual(items.map((i) => route(i.href)), [
    { view: "new" },
    { view: "settings", focus: "invite_make" },
    { view: "settings", focus: "invite_accept" },
    { view: "settings", focus: "agents" },
  ]);
  assert.deepEqual(items.map((i) => i.focus), ["chat_peer", "window_btn", "invite_text", "agent_name"]);
  const html = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  for (const i of items) assert.ok(html.includes(`id="${i.focus}"`), `index.html 에 ${i.focus}`);
});

test("profile_menu — 상태(받기 가능 · 집중 고르기 · 앱과 연결 끊김 표시) | 사용자 정보(#1l) · 프로필 편집 · 알림 일시 중지(켜짐 표시) · 설정(#1j)", () => {
  assert.deepEqual(
    profileMenu(false, false).map((i) => (i.sep ? "|" : [i.key, i.label, i.checked])),
    [
      ["avail", "받기 가능", true],
      ["focus", "집중", false],
      ["offline", "앱과 연결 끊김", undefined],
      "|",
      ["me", "사용자 정보", undefined],
      ["tile", "프로필 편집", undefined],
      ["pause", "알림 일시 중지", false],
      ["settings", "설정", undefined],
    ],
  );
  const pause = (m) => m.find((i) => i.key === "pause").checked;
  assert.equal(pause(profileMenu(true, false)), true);
  assert.equal(pause(profileMenu(undefined, undefined)), false, "설정을 못 읽었으면 꺼짐");
  assert.deepEqual(route(profileMenu(false, false).at(-1).href), { view: "settings", focus: null });
});

test("profile_menu_radio — 받기 가능 · 집중은 하나만 켜짐(menuitemradio), 끊김은 고를 수 없는 표시 · 색 열쇠는 presenceOf 와 같다", () => {
  const on = (focus) =>
    profileMenu(false, focus)
      .filter((i) => i.radio)
      .map((i) => [i.key, i.checked]);
  assert.deepEqual(on(true), [["avail", false], ["focus", true]]);
  assert.deepEqual(on(false), [["avail", true], ["focus", false]]);
  assert.deepEqual(on(undefined), [["avail", true], ["focus", false]], "설정을 못 읽었으면 받기 가능");
  const off = (x) => profileMenu(false, false, x).find((i) => i.key === "offline");
  assert.deepEqual([off(true).info, off(true).current, off(false).current, off(true).radio], [true, true, false, undefined]);
  for (const i of profileMenu(false, false).filter((x) => x.state)) {
    assert.equal(i.state, presenceOf(i.key === "focus", i.key === "offline").key, i.key);
  }
});

test("presence_of — 내 상태 색: 앱과 연결 끊김(붉음)이 먼저, 집중(주황), 그 밖 받기 가능(초록)", () => {
  assert.deepEqual(presenceOf(false, false), { key: "online", text: "받기 가능" });
  assert.deepEqual(presenceOf(true, false), { key: "focus", text: "집중" });
  assert.deepEqual(presenceOf(true, true), { key: "offline", text: "앱과 연결 끊김" }, "끊김이 집중보다 먼저");
  assert.deepEqual(presenceOf(undefined, undefined).key, "online", "설정을 못 읽었으면 받기 가능");
});

test("starred_sections — ★ 대화만 · '나' 는 상대별 절(레일 순서) · 빈 절 없음 · 없으면 빈 배열", () => {
  const names = new Map([
    [1, "가람 · A"],
    [2, "나래 · B"],
  ]);
  const convs = [
    { id: 1, peer_id: 1, agent: "a", dir: "out", title: "하나", starred: true },
    { id: 2, peer_id: 2, agent: "a", dir: "out", title: "둘", starred: false },
    { id: 3, peer_id: 2, agent: "a", dir: "in", title: "셋", starred: true },
    { id: 4, peer_id: 1, agent: "a", dir: "out", title: "넷" },
  ];
  assert.deepEqual(
    starredSections(convs, names, [2, 1]).map((s) => [s.key, s.rows.map((r) => r.c.id)]),
    [
      ["peer:2", [3]],
      ["peer:1", [1]],
    ],
  );
  assert.deepEqual(starredSections(convs, names).map((s) => [s.title, s.rows.map((r) => r.c.id)]), [["대화", [1, 3]]], "상대 타일 = 절 하나");
  assert.deepEqual(starredSections(convs.slice(1, 2), names, [1, 2]), []);
  assert.deepEqual(starredSections([], names), []);
});

test("presence — 상대 접속 상태(레일 점 · 사이드바 머리 · 대화 머리글) · 거절 사유", () => {
  const now = 1_000_000;
  assert.deepEqual(presence({ seen_at: now - 10, focus: false }, now), { key: "online", text: "방금 통신" });
  assert.deepEqual(presence({ seen_at: now - 10, focus: true }, now), { key: "focus", text: "집중 중 · 방금 통신" });
  assert.deepEqual(presence({ seen_at: now - 600, focus: true }, now), { key: "offline", text: "10분 전 통신" }, "오래되면 집중 표시 없음");
  assert.deepEqual(presence(undefined, now), { key: "offline", text: "통신 기록 없음" }, "모르는 상대");
  assert.equal(refusedText({ state: "error", refused: "focus" }), "상대가 집중 중");
  assert.equal(refusedText({ state: "error", refused: "busy" }), "상대가 거절(busy)");
  assert.equal(refusedText({ state: "error", refused: null }), "");
  assert.equal(refusedText({ state: "done", refused: "focus" }), "", "다시 보내 성공한 글에는 사유를 보이지 않는다");
});

test("conv_sections — ★ 대화도 제자리(#1j), 행 표시 · 레일 배지", () => {
  const names = new Map([[2, "이도윤 · DYOON"]]);
  const convs = [
    { id: 1, peer_id: 2, dir: "out", agent: "guide", title: "배포", starred: true, last_state: "done", unread: 0 },
    { id: 2, peer_id: 2, dir: "in", agent: "review", title: "로그인", starred: false, last_state: "running", unread: 2 },
    { id: 3, peer_id: 3, dir: "out", agent: "guide", title: "릴리스", starred: false, relay_agent: "dev", relay_cap: 10, relay_count: 3, unread: 1 },
  ];
  assert.deepEqual(
    convSections(convRows(convs, names, null)).map((s) => [s.title, s.rows.map((r) => r.c.id)]),
    [["대화", [1, 2, 3]]],
  );
  assert.deepEqual(convSections(convRows(convs, names, 3)).map((s) => s.title), ["대화"]);
  assert.deepEqual(convMeta(convs[0]), { agent: "guide", state: "완료", running: false, relay: null, unread: false, tag: null });
  assert.deepEqual(convMeta(convs[1]), { agent: "내 review", state: "실행 중", running: true, relay: null, unread: true, tag: null });
  assert.deepEqual(convMeta(convs[2]), { agent: "guide", state: "", running: false, relay: "왕복 3/10", unread: true, tag: null });
  assert.deepEqual([...unreadByPeer(convs)], [
    [2, 2],
    [3, 1],
  ]);
});

test("activity_rows — 상대 글 · 실패 · 자동 왕복 시작 · 끝, 안 본 항목, 누르면 그 글", () => {
  const names = new Map([[2, "이도윤 · DYOON"]]);
  const conv = { id: 5, peer_id: 2, agent: "guide", dir: "out", title: "배포 절차" };
  const items = [
    { kind: "message", at: 300, unseen: true, conversation: conv, message: { id: 9, author: "agent", mine: false, text: "답\n둘째 줄" } },
    { kind: "failed", at: 200, unseen: false, conversation: conv, message: { id: 8, author: "human", mine: true, exec: true, text: "요청", state: "error" } },
    { kind: "relay_end", at: 150, unseen: false, conversation: conv, detail: "상한 도달" },
    { kind: "relay_start", at: 100, unseen: false, conversation: conv, detail: "첫 답 앞부분" },
    { kind: "message", at: 95, unseen: false, conversation: conv, message: { id: 7, author: "agent", mine: false, text: "실행 실패", state: "error" } },
    { kind: "message", at: 90, unseen: true, conversation: null, message: { id: 1, author: "human", text: "고아" } },
  ];
  const rows = activityRows(items, names);
  assert.deepEqual(
    rows.map((r) => [r.label, r.who, r.preview, r.href, r.unseen]),
    [
      ["", "이도윤/guide", "답 둘째 줄", "#c/5/m/9", true],
      ["실패", "나", "요청", "#c/5/m/8", false],
      ["자동 왕복 끝 · 상한 도달", "이도윤", "", "#c/5", false],
      ["자동 왕복 시작", "이도윤", "첫 답 앞부분", "#c/5", false],
      ["실패", "이도윤/guide", "실행 실패", "#c/5/m/7", false],
    ],
    "방향 라벨 없음(#1j) — 상대 에이전트의 실패 답은 실패",
  );
  assert.ok(rows.every((r) => r.title === "배포 절차" && !("tag" in r)));
  assert.equal(new Set(rows.map((r) => r.key)).size, rows.length, "항목 열쇠는 겹치지 않는다");
});

test("message_actions — 북마크 · 복사는 말풍선 아이콘(#1j, 막대에 없음), 다시 보내기 없음(#1k)", () => {
  const labels = (m) => messageActions(m).map((a) => a.label);
  assert.deepEqual(labels({ author: "human", mine: true, exec: true, saved: false }), []);
  assert.deepEqual(labels({ author: "human", mine: true, exec: true, saved: true }), []);
  assert.deepEqual(labels({ author: "human", mine: false, exec: true }), [], "상대 글 — 단추 없음");
  assert.deepEqual(labels({ author: "agent", mine: true }), [], "에이전트 답");
  assert.deepEqual(labels({ author: "human", mine: true, exec: false }), [], "실행 없는 글");
  for (const m of [{ author: "human", mine: true, exec: true }, { author: "agent", mine: false }]) {
    assert.ok(!messageActions(m).some((a) => /복사|편집|삭제|원문/.test(a.label)), "막대에 복사 · 편집 · 삭제 · 원문 없음");
  }
});

test("edited_tag — 편집한 글만 '(편집됨)'", () => {
  assert.equal(editedTag({ edited_at: 1760400000 }), "(편집됨)");
  assert.equal(editedTag({ edited_at: null }), "");
  assert.equal(editedTag({}), "");
});

test("keep_picked — 새로 읽은 글 중 아직 있고 편집할 수 있는 것만 남긴다", () => {
  const ms = [
    { id: 1, editable: true },
    { id: 2, editable: false },
    { id: 4, editable: true },
  ];
  assert.deepEqual([...keepPicked(new Set([1, 2, 3, 4]), ms)], [1, 4], "편집 불가(2) · 지워진 글(3)은 빠진다");
  assert.deepEqual([...keepPicked(new Set(), ms)], []);
  const sig = (m) => messagesSigOf("p", [{ id: 1, state: "done", text: "a", ...m }]);
  assert.notEqual(sig({}), sig({ edited_at: 5 }), "편집하면 다시 그린다");
  assert.notEqual(sig({ editable: false }), sig({ editable: true }), "편집 가능이 바뀌면 다시 그린다");
});

test("activity_rows — 상대 반영 실패는 그 대화로", () => {
  const conv = { id: 5, peer_id: 2, agent: "guide", dir: "out", title: "배포 절차" };
  const rows = activityRows([{ kind: "sync_failed", at: 1, unseen: true, conversation: conv, detail: "편집 — 고친 질의" }], new Map([[2, "이도윤 · DYOON"]]));
  assert.deepEqual(
    rows.map((r) => [r.label, r.who, r.preview, r.href, r.unseen]),
    [["상대 반영 실패 · 편집 — 고친 질의", "이도윤", "", "#c/5", true]],
  );
});

test("insert_markdown — 굵게 · 목록 · 코드 표기를 넣고 고른 글자를 유지", () => {
  assert.deepEqual(insertMarkdown("배포 절차", 3, 5, "bold"), { text: "배포 **절차**", start: 5, end: 7 });
  assert.deepEqual(insertMarkdown("", 0, 0, "bold"), { text: "****", start: 2, end: 2 }, "빈 선택 = 표기 사이에 커서");
  assert.deepEqual(insertMarkdown("a\nb\nc", 0, 3, "list"), { text: "- a\n- b\nc", start: 0, end: 7 });
  assert.deepEqual(insertMarkdown("a\nb\n", 0, 4, "list"), { text: "- a\n- b\n", start: 0, end: 8 }, "끝 줄바꿈 뒤 빈 줄은 그대로");
  assert.deepEqual(insertMarkdown("\nabc", 0, 0, "list"), { text: "- \nabc", start: 2, end: 2 }, "첫 줄이 빈 줄");
  assert.deepEqual(insertMarkdown("첫 줄\n둘", 5, 5, "list"), { text: "첫 줄\n- 둘", start: 7, end: 7 }, "커서 줄 머리에");
  assert.deepEqual(insertMarkdown("run cargo", 4, 9, "code"), { text: "run `cargo`", start: 5, end: 10 });
  assert.deepEqual(insertMarkdown("x\ny", 0, 3, "code"), { text: "```\nx\ny\n```", start: 4, end: 7 }, "여러 줄 = 코드 블록");
});

test("enter_sends — Enter 보내기 · Shift+Enter 줄바꿈 · 한글 조합 중 Enter 는 보내지 않음", () => {
  assert.equal(enterSends({ key: "Enter" }), true);
  assert.equal(enterSends({ key: "Enter", shiftKey: true }), false);
  assert.equal(enterSends({ key: "Enter", isComposing: true }), false, "조합 중");
  assert.equal(enterSends({ key: "Enter", keyCode: 229 }), false, "조합 중(keyCode 229)");
  assert.equal(enterSends({ key: "Enter", repeat: true }), false, "누르고 있는 Enter(키 반복)");
  assert.equal(enterSends({ key: "a" }), false);
});

test("run_steps — 보냄 → 대기열 → 실행 → 응답, 시각 HH:MM:SS · 실행 줄 · 받은 요청은 받음 · 보냄", () => {
  const at = 1791691336;
  const clock = (t) => {
    const d = new Date(t * 1000);
    return [d.getHours(), d.getMinutes(), d.getSeconds()].map((n) => String(n).padStart(2, "0")).join(":");
  };
  const steps = (ms) => runSteps(timeline(ms)[0]);
  const ask = { id: 1, author: "human", mine: true, exec: true, text: "q", created_at: at };
  const done = steps([
    { ...ask, state: "done" },
    { id: 2, author: "agent", mine: false, reply_to: 1, text: "a", state: "done", created_at: at + 41, model: "sonnet", effort: "high", turns: 3, duration_ms: 41000 },
  ]);
  assert.deepEqual(
    done.map((s) => [s.title, s.state, s.at]),
    [
      ["요청 보냄", "done", clock(at)],
      ["대기열", "done", ""],
      ["실행 완료", "done", ""],
      ["응답 받음", "done", clock(at + 41)],
    ],
  );
  assert.equal(done[2].meta, "sonnet · high · 3턴 · 41.0초");
  assert.deepEqual(steps([{ ...ask, state: "sent" }]).map((s) => s.state), ["done", "now", "wait", "wait"], "보낸 쪽은 답 전까지 대기열");
  assert.deepEqual(
    steps([{ ...ask, mine: false, state: "running" }]).map((s) => [s.title, s.state]),
    [
      ["요청 받음", "done"],
      ["대기열", "done"],
      ["실행 중", "now"],
      ["응답 보냄", "wait"],
    ],
  );
  const failed = steps([
    { ...ask, state: "error" },
    { id: 2, author: "agent", reply_to: 1, text: "x", state: "error", created_at: at + 5 },
  ]);
  assert.deepEqual(failed.map((s) => s.state), ["done", "done", "fail", "fail"]);
  assert.deepEqual([failed[2].title, failed[2].meta], ["실패", ""], "실행 정보 없으면 빈 줄");
  // 머리글 부제 — 대화 에이전트(보낸 대화 = 상대 쪽, 받은 대화 = 내 쪽) 마지막 답의 model · effort. 자동 왕복의 반대쪽 에이전트 답은 보지 않고,
  // 마지막 답에 값이 없으면 거슬러 오르지 않는다.
  const ans = (mine, model, effort) => ({ author: "agent", mine, model, effort });
  assert.equal(lastRun([ans(false, "opus", "low"), ans(true, "sonnet", "high"), { author: "human", mine: true }], false), "opus · low");
  assert.equal(lastRun([ans(false, "opus", "low"), ans(true, "sonnet", "high")], true), "sonnet · high", "받은 대화 = 내 에이전트 답");
  assert.equal(lastRun([ans(false, "opus", "low"), ans(false, null, null)], false), "", "마지막 답에 없으면 빈 글자");
  assert.equal(lastRun([{ author: "human", mine: false, model: "x" }], false), "", "사람 글은 보지 않는다");
});

test("search_groups — 메시지 · 대화 · 사람 · 에이전트 묶음, 빈 묶음 없음, 누르면 그곳으로", () => {
  const names = new Map([[2, "이도윤 · DYOON"]]);
  const conv = { id: 5, peer_id: 2, agent: "guide", dir: "out", title: "배포 절차" };
  const groups = searchGroups(
    {
      messages: [{ id: 9, author: "agent", mine: false, text: "롤백은\n이전 태그로", conversation: conv }],
      conversations: [{ ...conv }],
      peers: [
        { id: 2, name: "dyoon", alias: "이도윤", host: "DYOON", paired: true },
        { id: 4, name: "kim", alias: null, host: "KIM-PC", paired: false, my_ok: true, their_ok: false },
      ],
      agents: [
        { name: "guide", owner: 2 },
        { name: "review", owner: "me" },
      ],
    },
    names,
  );
  assert.deepEqual(groups.map((g) => g.title), ["메시지", "대화", "사람", "에이전트"]);
  assert.deepEqual(groups[0].rows.map((r) => [r.label, r.sub, r.href]), [["롤백은 이전 태그로", "이도윤/guide · 배포 절차", "#c/5/m/9"]]);
  assert.deepEqual(groups[1].rows.map((r) => [r.label, r.href]), [["배포 절차", "#c/5"]]);
  assert.deepEqual(
    groups[2].rows.map((r) => [r.label, r.tile, r.href]),
    [
      ["이도윤", 2, undefined],
      ["kim", undefined, "#settings/peers"],
    ],
    "페어링된 사람 = 타일, 아니면 연결 목록",
  );
  assert.deepEqual(
    groups[3].rows.map((r) => [r.label, r.href, r.chat]),
    [
      ["guide", undefined, { peer: 2, agent: "guide" }],
      ["review", "#settings/agents", undefined],
    ],
    "상대 에이전트 = 새 대화 채움, 내 것 = 설정",
  );
  assert.deepEqual(searchGroups({ messages: [], conversations: [], peers: [], agents: [] }, names), []);
});

test("ask_marks — 요청 태그(상태) · 상태 바꾸기 작업(받은 요청만) · 내 활동 · 보낸 메시지 라벨", () => {
  const req = (mine, req_state) => ({ author: "human", mine, exec: false, request: true, req_state });
  assert.equal(askTag(req(true, "wait")), "요청 · 대기");
  assert.equal(askTag(req(false, "done")), "요청 · 완료");
  assert.equal(askTag(req(false, undefined)), "요청 · 대기", "상태가 없으면 대기");
  assert.equal(askTag({ author: "human", request: false }), "", "질의는 표시 없음");
  assert.equal(askTag({ author: "agent", request: true }), "", "답에는 없다");
  const acts = (m) => messageActions(m).map((x) => [x.key, x.label]);
  assert.deepEqual(acts(req(false, "wait")).slice(-2), [["req:accepted", "수용"], ["req:rejected", "반려"]]);
  assert.deepEqual(acts(req(false, "accepted")).at(-1), ["req:done", "완료"]);
  assert.ok(!acts(req(false, "done")).some(([k]) => k.startsWith("req:")), "끝난 요청");
  assert.ok(!acts(req(true, "wait")).some(([k]) => k.startsWith("req:")), "내 요청은 못 바꾼다");
  assert.deepEqual(acts(req(true, "wait")), [], "내 요청 — 다시 보내기 없음(#1k)");
  assert.ok(!acts({ author: "human", mine: false, exec: true, request: false }).some(([k]) => k.startsWith("req:")), "질의");
  const conv = { id: 1, peer_id: 2, agent: "guide", dir: "in", title: "t" };
  const rows = activityRows([{ kind: "message", at: 1, unseen: false, conversation: conv, message: { id: 5, ...req(false, "done"), text: "x", state: "done" } }], new Map());
  assert.equal(rows[0].label, "요청 · 완료");
  const sent = inboxRows([{ id: 6, ...req(true, "accepted"), text: "y", created_at: 1, conversation: { ...conv, dir: "out" } }], new Map());
  assert.equal(sent[0].kind, "요청 · 수용");
});

test("req_moves — 대기 → 수용 · 반려, 수용 → 완료, 그 밖 없음", () => {
  const got = (req_state, mine = false) => reqMoves({ author: "human", mine, request: true, req_state }).map((x) => x.to);
  assert.deepEqual(got("wait"), ["accepted", "rejected"]);
  assert.deepEqual(got("accepted"), ["done"]);
  assert.deepEqual(got("rejected"), []);
  assert.deepEqual(got("done"), []);
  assert.deepEqual(got("wait", true), [], "내 요청");
  assert.deepEqual(reqMoves({ author: "human", mine: false, request: false }), [], "질의");
  assert.deepEqual(Object.keys(REQ_STATES), ["wait", "accepted", "rejected", "done"], "an_core ReqState 와 같은 넷");
  // 요청자에 알리지 못한 상태(포기) — 표시 · "다시 알리기"(R1).
  const lost = { author: "human", mine: false, request: true, req_state: "done", state: "done", refused: "unreachable" };
  assert.ok(messageActions(lost).some((a) => a.key === "req-again" && a.label === "다시 알리기"));
  assert.match(refusedText(lost), /요청자에게 알리지 못함/);
  assert.ok(!messageActions({ ...lost, mine: true }).some((a) => a.key === "req-again"), "내 요청은 아님");
  assert.match(refusedText({ ...lost, refused: "unknown" }), /거절\(unknown/, "요청자가 거절(R2)");
  assert.ok(messageActions({ ...lost, refused: "unknown" }).some((a) => a.key === "req-again"));
});

test("request_rows — 요청 화면 행: 상대 · 대화 · 상태 · 메모 · 바꾼 시각 · 그 글로", () => {
  const names = new Map([[2, "user-b · PC-B"]]);
  const conv = { id: 9, peer_id: 2, agent: "review", dir: "in", title: "로그" };
  const rows = requestRows(
    [
      { id: 3, author: "human", mine: false, request: true, req_state: "accepted", req_note: "이번 주", req_at: 50, created_at: 10, text: "형식 바꿔 주세요", conversation: conv },
      { id: 4, author: "human", mine: false, request: false, created_at: 11, text: "질의", conversation: conv },
      { id: 5, author: "human", mine: false, request: true, req_state: "rejected", created_at: 12, text: "x" },
    ],
    names,
  );
  assert.equal(rows.length, 1, "요청 · 대화가 있는 것만");
  const [r] = rows;
  assert.deepEqual(
    [r.who, r.title, r.state, r.done, r.note, r.at, r.href, r.moves.map((m) => m.to)],
    ["user-b", "로그", "수용", false, "이번 주", 50, "#c/9/m/3", ["done"]],
  );
});

test("requests_route — #requests 화면 · 사이드바 '요청' 은 그 화면", () => {
  assert.deepEqual(route("#requests"), { view: "requests" });
  assert.equal(sideNav(null).find((n) => n.label === "요청").href, "#requests");
  assert.equal(sideNav(4).find((n) => n.label === "요청").href, "#requests");
});

test("models_tools — 모델 드롭다운 셋 · 도구 안내는 읽기 전용 다섯", () => {
  assert.deepEqual(MODELS, ["opus", "sonnet", "haiku"], "an_core run_opts::MODELS 와 같다");
  assert.deepEqual(READ_ONLY_TOOLS, ["Read", "Grep", "Glob", "LS", "NotebookRead"]);
  assert.ok(READ_ONLY_TOOLS.every((t) => TOOLS_NOTE.includes(t)), TOOLS_NOTE);
});

test("messages_sig_req — 요청 상태 · 메모가 바뀌면 다시 그린다", () => {
  const m = { id: 1, state: "done", text: "x", request: true, req_state: "wait" };
  const a = messagesSigOf("민수", [m]);
  assert.notEqual(messagesSigOf("민수", [{ ...m, req_state: "accepted" }]), a);
  assert.notEqual(messagesSigOf("민수", [{ ...m, req_note: "메모" }]), a);
  assert.equal(messagesSigOf("민수", [{ ...m }]), a);
});

test("mention — 입력 중인 @ 범위 · 글의 첫 @이름(긴 이름 먼저) · 답 발화자 = 실행한 에이전트", () => {
  assert.deepEqual(mentionQuery("안녕 @gu", 7), { start: 3, query: "gu" });
  assert.deepEqual(mentionQuery("@", 1), { start: 0, query: "" });
  assert.deepEqual(mentionQuery("@gu 뒤", 3), { start: 0, query: "gu" }, "커서 앞까지");
  assert.equal(mentionQuery("메일a@gu", 7), null, "앞이 글자면 아님");
  assert.equal(mentionQuery("@guide 봐", 9), null, "공백 뒤");
  const names = ["guide", "guide-x", "review"];
  assert.equal(mentionTarget("@guide-x 봐 줘", names), "guide-x", "긴 이름 먼저");
  assert.equal(mentionTarget("@guide, 봐 줘", names), "guide", "뒤가 문장 부호");
  assert.equal(mentionTarget("이것 @guide", names), "guide", "뒤가 글 끝");
  assert.equal(mentionTarget("@guidex 봐", names), null, "뒤가 글자면 아님");
  assert.equal(mentionTarget("a@guide 봐", names), null, "앞이 글자면 아님");
  assert.equal(mentionTarget("@nobody 그리고 @review", names), "review", "첫 일치 이름");
  assert.equal(mentionTarget("@review @guide", names), "review", "하나만 — 첫 것");
  assert.equal(mentionTarget("멘션 없음", names), null);
  assert.equal(hasMention("@review 봐"), true, "글 머리");
  assert.equal(hasMention("이것 @x"), true, "공백 뒤");
  assert.equal(hasMention("메일 a@b.com"), false, "낱말 안의 @ 는 아님");
  assert.equal(hasMention("@ 혼자 · 끝에 @"), false, "이름 없음");
  assert.equal(mentionToken("이것 @guidee 봐"), "@guidee", "알릴 낱말");
  assert.equal(mentionToken("메일 a@b.com"), null);
  const out = { dir: "out", agent: "review" };
  assert.equal(authorLabel(out, { author: "agent", agent: "guide" }, "이도윤 · DYOON"), "이도윤/guide");
  assert.equal(authorLabel(out, { author: "agent", agent: null }, "이도윤 · DYOON"), "이도윤/review");
  assert.equal(authorLabel({ dir: "in", agent: "dev" }, { author: "agent", agent: "guide" }, "김 · KIM"), "내 guide");
});

test("entry_token", () => {
  const a = "a".repeat(64);
  const b = "b".repeat(64);
  // 주소의 토큰이 먼저 — 주소창에서 지운다.
  assert.deepEqual(entryToken(`?t=${a}`, b), { token: a, clean: true });
  // 주소에 없으면 탭에 저장된 값.
  assert.deepEqual(entryToken("", b), { token: b, clean: false });
  // 꼴이 틀린 주소 토큰은 쓰지 않지만 지운다.
  assert.deepEqual(entryToken("?t=xyz", null), { token: null, clean: true });
  assert.deepEqual(entryToken("?t=xyz", b), { token: b, clean: true });
  // 저장된 값도 꼴을 본다.
  assert.deepEqual(entryToken("", "zz"), { token: null, clean: false });
  assert.deepEqual(entryToken(undefined, null), { token: null, clean: false });
  assert.match(DISCONNECTED, /트레이 아이콘 메뉴 '열기'/);
});

test("progressText — 보낸 글의 상대 차례 · 실행 상세 진행에도", () => {
  const q = { state: "sent", progress: { state: "queued", ahead: 2 } };
  assert.equal(progressText(q), "차례 기다리는 중 · 앞에 2건");
  assert.equal(progressText({ state: "sent", progress: { state: "running", ahead: 0 } }), "처리 중");
  assert.equal(progressText({ state: "sent", progress: null }), "");
  assert.equal(progressText({ state: "done", progress: { state: "running", ahead: 0 } }), "", "답이 오면 지운다");
  const req = { mine: true, created_at: 0, ...q };
  assert.equal(runSteps({ request: req }).find((s) => s.key === "queue").meta, progressText(q));
  const run = runSteps({ request: { ...req, progress: { state: "running", ahead: 0 } } });
  assert.deepEqual(run.map((s) => s.state), ["done", "done", "now", "wait"]);
});

test("runsSummary — 이 PC 실행 머리 · 실행 중 먼저 · 대기 순번", () => {
  const r = runsSummary({
    cap: 1,
    running: [{ conversation_id: 3, peer: "민수", agent: "review", text: "로그 봐줘" }],
    waiting: [{ conversation_id: 5, peer: "지은", agent: "review", text: "a" }, { message_id: 9 }],
  });
  assert.equal(r.title, "실행 중 1 · 대기 2");
  assert.deepEqual(r.rows, [
    { label: "실행 중 · 민수/review", text: "로그 봐줘", href: "#c/3" },
    { label: "대기 1 · 지은/review", text: "a", href: "#c/5" },
    { label: "대기 2 · 알 수 없는 상대/에이전트", text: "", href: "" },
  ]);
  assert.equal(runsSummary({ cap: 2, running: [], waiting: [] }).title, "실행 중 0 · 대기 0");
});

test("portText — 웹UI 포트 · 피어 포트(고정 | 임의 — 잡은 프로세스) · 열린 연결", () => {
  assert.deepEqual(portText({ ui_port: 51234, port: 47100, peer_port_fixed: true, peer_port_holder: null, open_connections: 2 }), {
    ui: "51234",
    peer: "47100(고정)",
    conns: "2",
  });
  const busy = { ui_port: 51234, port: 50001, peer_port_fixed: false, peer_port_holder: { name: "python.exe", pid: 812 }, open_connections: 0 };
  assert.equal(portText(busy).peer, "50001(임의 — 47100 사용 중: python.exe, PID 812)");
  assert.equal(portText({ port: 50001, peer_port_fixed: false }).peer, "50001(임의)");
  assert.equal(portText({ port: 50001 }).ui, "알 수 없음");
});

test("messagesSigOf — 진행만 바뀌어도(sent 그대로) 다시 그린다", () => {
  const m = { id: 1, state: "sent", text: "q", done: false, refused: null };
  const q1 = messagesSigOf("민수", [{ ...m, progress: { state: "queued", ahead: 1 } }]);
  const q0 = messagesSigOf("민수", [{ ...m, progress: { state: "queued", ahead: 0 } }]);
  const run = messagesSigOf("민수", [{ ...m, progress: { state: "running", ahead: 0 } }]);
  assert.equal(new Set([messagesSigOf("민수", [m]), q1, q0, run]).size, 4);
  assert.equal(messagesSigOf("민수", [{ ...m, saved: true }]), messagesSigOf("민수", [m]), "저장은 서명 밖");
});

test("outboxText · refusedText — 보관 중 · 보내지 못함(#1c)", () => {
  assert.equal(outboxText({ outbox: { state: "inflight", tries: 0 } }), "보내는 중");
  assert.equal(outboxText({ outbox: { state: "pending", tries: 3 } }), "보내는 중 · 다시 보냄 3회");
  assert.equal(outboxText({ outbox: { state: "failed", tries: 9 } }), "", "포기는 거절 사유가 말한다");
  assert.equal(outboxText({ outbox: null }), "");
  assert.match(refusedText({ state: "error", refused: "unreachable" }), /^보내지 못함/);
  assert.ok(!refusedText({ state: "error", refused: "unreachable" }).includes("다시 보내기"), "없앤 단추를 가리키지 않는다(#1k)");
  const sent = { id: 1, state: "sent", text: "q", done: false, refused: null };
  assert.notEqual(messagesSigOf("x", [{ ...sent, outbox: { state: "pending", tries: 1 } }]), messagesSigOf("x", [sent]), "닿으면 다시 그린다");
});

test("stepText · progressText — 처리 중 단계 요약(#1c)", () => {
  const d = { turns: 2, tools: { Read: 3, Grep: 1 }, secs: 41 };
  assert.equal(stepText(d), "2턴 · Read 3 · Grep 1 · 41초");
  assert.equal(stepText({ turns: 0, tools: {}, secs: 0 }), "");
  assert.equal(progressText({ state: "sent", progress: { state: "running", ahead: 0, detail: d } }), "처리 중 · 2턴 · Read 3 · Grep 1 · 41초");
  assert.equal(progressText({ state: "sent", progress: { state: "running", ahead: 0, detail: null } }), "처리 중");
});

test("presence — 마지막 통신 n분 · n시간 · n일 전(#1c — 주기 확인 없음)", () => {
  const now = 2_000_000;
  assert.equal(presence({ seen_at: now - 74 }, now).text, "방금 통신");
  assert.equal(presence({ seen_at: now - 75 }, now).text, "1분 전 통신");
  assert.equal(presence({ seen_at: now - 3 * 3600 }, now).text, "3시간 전 통신");
  assert.equal(presence({ seen_at: now - 2 * 86400 }, now).text, "2일 전 통신");
  assert.equal(presence({ seen_at: now + 30 }, now).text, "방금 통신", "시계가 앞서도 음수 없음");
});

test("hold_render — 배경 새로 고침은 입력칸에서만 미룬다(절 머리 summary · 단추에 초점이 있어도 다시 그린다, 적대적 평가 라운드 1)", () => {
  for (const tag of ["INPUT", "TEXTAREA", "SELECT"]) assert.equal(holdsRender(tag), true, tag);
  for (const tag of ["SUMMARY", "BUTTON", "A", "DETAILS", "BODY", undefined]) assert.equal(holdsRender(tag), false, String(tag));
});

test("send_latest — 접힘 저장은 한 번에 하나, 보내는 동안 온 값은 마지막 것만 이어서(응답 순서가 바뀌어 옛 값이 남지 않게, 적대적 평가 라운드 1)", async () => {
  const sent = [];
  const done = [];
  const q = new SendLatest(
    (v) =>
      new Promise((ok, fail) => {
        sent.push(v);
        done.push(v === "B" ? fail : ok);
      }),
  );
  q.push("A");
  q.push("B");
  q.push("C");
  assert.deepEqual(sent, ["A"], "보내는 중이면 기다린다");
  done[0]();
  await new Promise((r) => setTimeout(r, 0));
  assert.deepEqual(sent, ["A", "C"], "가운데 값(B)은 건너뛰고 마지막 값만");
  q.push("D");
  done[1]();
  await new Promise((r) => setTimeout(r, 0));
  assert.deepEqual(sent, ["A", "C", "D"]);
  const f = new SendLatest((v) => {
    sent.push(v);
    return Promise.reject(new Error("끊김"));
  });
  f.push("E");
  f.push("F");
  await new Promise((r) => setTimeout(r, 0));
  await new Promise((r) => setTimeout(r, 0));
  assert.deepEqual(sent.slice(-2), ["E", "F"], "실패해도 다음 값을 보낸다");
});

// ---- #1i 레일 그룹 · 손으로 놓는 순서 ----

test("side_nav_sections — 어느 타일이든 구분선은 절 사이에만(머리 · 끝 · 연달아 없음), '대화' · '저장된 항목' 이름 없음", () => {
  for (const tile of [null, 3, "group"]) {
    const xs = sideNav(tile, 1, 1);
    assert.ok(!xs[0].sep && !xs.at(-1).sep, String(tile));
    assert.ok(xs.every((n, i) => !(n.sep && xs[i + 1]?.sep)), String(tile));
    assert.ok(!xs.some((n) => n.label === "대화" || n.label === "저장된 항목"), String(tile));
    assert.equal(xs.at(-1).label === "에이전트" || xs.at(-1).label === "북마크", true, "하단 = 사용자가 모은 · 등록한 것");
  }
});

test("side_nav_counts — 안 본 내 활동 수는 '나' 에만, 대기 요청 수는 모든 타일의 요청에", () => {
  const count = (tile, label) => sideNav(tile, 4, 9).find((n) => n.label === label)?.count;
  assert.equal(count(null, "내 활동"), 4);
  for (const tile of [null, 3, "group"]) assert.equal(count(tile, "요청"), 9, String(tile));
  assert.equal(count(3, "내 활동"), undefined, "상대 타일엔 내 활동 없음");
  assert.equal(count("group", "에이전트"), undefined, "그룹엔 에이전트 없음");
});

test("message_actions_no_bookmark — 북마크는 막대에 없다(#1j 아이콘), 받은 요청은 상태 단추가 남는다", () => {
  for (const m of [{ author: "agent", saved: false }, { author: "human", mine: true, exec: true, saved: true }]) {
    assert.ok(!messageActions(m).some((a) => a.key === "save"));
  }
  assert.deepEqual(
    messageActions({ author: "human", mine: false, request: true, req_state: "wait" }).map((a) => a.key),
    ["req:accepted", "req:rejected"],
  );
});

test("bubble_side — 이 PC 쪽 글(mine)은 오른쪽, 그 밖은 왼쪽", () => {
  assert.equal(bubbleSide({ author: "human", mine: true }), "mine");
  assert.equal(bubbleSide({ author: "agent", mine: true }), "mine", "받은 대화의 내 에이전트 답");
  assert.equal(bubbleSide({ author: "agent", mine: false }), "theirs");
  assert.equal(bubbleSide({ author: "human" }), "theirs", "모르면 왼쪽");
});

test("day_label · day_breaks — 오늘 · 어제 · M.D · 다른 해, 날짜가 바뀔 때만 구분선", () => {
  const at = (y, mo, d, h = 12) => new Date(y, mo - 1, d, h).getTime() / 1000;
  const now = at(2026, 10, 15, 20);
  assert.equal(dayLabel(at(2026, 10, 15, 1), now), "오늘");
  assert.equal(dayLabel(at(2026, 10, 14, 23), now), "어제");
  assert.equal(dayLabel(at(2026, 9, 3), now), "9.3");
  assert.equal(dayLabel(at(2025, 12, 31), now), "2025.12.31");
  assert.equal(dayKey(at(2026, 10, 15, 0)), dayKey(at(2026, 10, 15, 23)));
  const ms = [at(2026, 10, 14, 9), at(2026, 10, 14, 10), at(2026, 10, 15, 8), at(2026, 10, 15, 9)].map((t, i) => ({ id: i, created_at: t }));
  assert.deepEqual(dayBreaks(ms, now), ["어제", null, "오늘", null]);
  assert.deepEqual(dayBreaks([], now), []);
});

test("room_preview — 대화 줄 미리보기 = 마지막 글 '{말한 이}: {한 줄}', 글 없으면 빈 글자", () => {
  const conv = { dir: "out", agent: "guide", peer_id: 2 };
  assert.equal(roomPreview({ ...conv, last: { author: "agent", mine: false, text: "승인자는\n팀 리드", agent: null } }, "이준호 · QA"), "이준호/guide: 승인자는 팀 리드");
  assert.equal(roomPreview({ ...conv, last: { author: "human", mine: true, text: "고마워" } }, "이준호 · QA"), "나: 고마워");
  assert.equal(roomPreview({ ...conv, last: null }, "이준호 · QA"), "");
  assert.equal(roomPreview({ ...conv, last: { author: "human", mine: false, text: "x".repeat(200) } }, "이준호 · QA").length, "이준호: ".length + 121);
});

test("side_nav_group — 그룹 타일 메뉴 4개 세 절(개요 | 요청 | 즐겨찾기 · 북마크), 내 활동 · 주고받은 메시지 · 에이전트 없음(#1j)", () => {
  assert.deepEqual(
    sideNav("group", 7, 2, 1).map((n) => (n.sep ? "|" : [n.label, n.href, n.count ?? 0])),
    [["개요", "#", 0], "|", ["요청", "#requests", 2], "|", ["즐겨찾기", "#starred", 1], ["북마크", "#saved", 0]],
  );
});

test("sort_by_pos — 자리 작은 것 먼저, 같으면 id, 자리 없으면 0 · 원본은 그대로", () => {
  const xs = [
    { id: 5, pos: 1 },
    { id: 3, pos: 1 },
    { id: 9, pos: -2 },
    { id: 1 },
  ];
  assert.deepEqual(sortByPos(xs).map((x) => x.id), [9, 1, 3, 5]);
  assert.deepEqual(xs.map((x) => x.id), [5, 3, 9, 1], "새 배열");
});

test("in_group · group_peers — 그룹 대화만 · 그 그룹에 대화가 있는 상대만(레일 순서)", () => {
  const convs = [
    { id: 1, peer_id: 7 },
    { id: 2, peer_id: 8 },
    { id: 3, peer_id: 9 },
    { id: 4, peer_id: 7 },
  ];
  const g = { id: 1, convs: [4, 2] };
  assert.deepEqual(inGroup(convs, g).map((c) => c.id), [2, 4]);
  assert.deepEqual(inGroup(convs, null), []);
  assert.deepEqual(groupPeers(inGroup(convs, g), [9, 8, 7]), [8, 7], "9 는 그룹에 대화가 없다");
});

test("scope_query — 그룹이 있으면 그룹, 아니면 상대, 둘 다 없으면 빈 값", () => {
  assert.equal(scopeQuery(null, null), "");
  assert.equal(scopeQuery(3, null), "peer=3");
  assert.equal(scopeQuery(null, 5), "group=5");
});

test("group_name_error — 1 ~ 40자(앞뒤 공백 뺌) · 같은 이름(이름 바꾸는 그룹 자신은 뺌)", () => {
  const gs = [
    { id: 1, name: "결제" },
    { id: 2, name: "배포" },
  ];
  assert.equal(groupNameError("  ", gs), "그룹 이름은 1 ~ 40자");
  assert.equal(groupNameError("가".repeat(41), gs), "그룹 이름은 1 ~ 40자");
  assert.equal(groupNameError(" " + "가".repeat(40) + " ", gs), null);
  assert.equal(groupNameError(" 결제 ", gs), "같은 이름의 그룹이 있습니다");
  assert.equal(groupNameError("결제", gs, 1), null, "자기 이름");
  assert.equal(groupNameError("배포", gs, 1), "같은 이름의 그룹이 있습니다");
});

test("membership_diff — 고른 것과 지금 소속의 차이(넣기 · 빼기)", () => {
  assert.deepEqual(membershipDiff([3, 1, 2], [2, 5, 4, 3]), { add: [4, 5], remove: [1] });
  assert.deepEqual(membershipDiff([], []), { add: [], remove: [] });
});

test("color_of — id 로 다섯 색(자리와 무관)", () => {
  assert.deepEqual([1, 2, 5, 6, 10, 11].map(colorOf), ["peer-2", "peer-3", "peer-1", "peer-2", "peer-1", "peer-2"]);
});

test("move_in_order — 대상 앞 · 뒤에 끼우기, 걸러 보인 이웃 기준이어도 전체 순서에서 · 그대로인 경우", () => {
  const order = [1, 2, 3, 4, 5];
  assert.deepEqual(moveInOrder(order, 5, 2), [1, 5, 2, 3, 4], "앞");
  assert.deepEqual(moveInOrder(order, 1, 4, true), [2, 3, 4, 1, 5], "뒤");
  assert.deepEqual(moveInOrder(order, 2, 4, true), [1, 3, 4, 2, 5], "보이지 않는 3 을 건너 4 뒤로");
  assert.deepEqual(moveInOrder(order, 3, 3), order, "자기 자신");
  assert.deepEqual(moveInOrder(order, 9, 1), order, "없는 id");
  assert.deepEqual(moveInOrder(order, 1, 9), order, "없는 대상");
});

test("step_target — 보이는 묶음에서 한 칸(위 = 앞 이웃 앞, 아래 = 뒤 이웃 뒤), 끝이면 null", () => {
  const vis = [7, 3, 9];
  assert.deepEqual(stepTarget(vis, 3, -1), { target: 7, after: false });
  assert.deepEqual(stepTarget(vis, 3, 1), { target: 9, after: true });
  assert.equal(stepTarget(vis, 7, -1), null);
  assert.equal(stepTarget(vis, 9, 1), null);
  assert.equal(stepTarget(vis, 5, 1), null, "없는 항목");
  // 한 칸 옮긴 결과 — 전체 순서에 숨은 항목이 있어도 보이는 앞뒤가 한 칸 바뀐다.
  const t = stepTarget(vis, 9, -1);
  assert.deepEqual(moveInOrder([7, 1, 3, 2, 9], 9, t.target, t.after), [7, 1, 9, 3, 2]);
});

test("drop_kind — 같은 종류만 옮기기, 대화는 같은 절만, 대화 → 그룹 타일은 넣기, 그 밖은 놓을 수 없음", () => {
  const conv = (section) => ({ kind: "conv", section });
  assert.equal(dropKind({ kind: "peer" }, { kind: "peer" }), "move");
  assert.equal(dropKind({ kind: "group" }, { kind: "group" }), "move");
  assert.equal(dropKind(conv("peer:2"), conv("peer:2")), "move");
  assert.equal(dropKind(conv("peer:2"), conv("peer:3")), null, "다른 상대 절");
  assert.equal(dropKind(conv("starred"), conv("peer:2")), null, "★ ↔ 상대 절");
  assert.equal(dropKind(conv("peer:2"), { kind: "group" }), "join");
  assert.equal(dropKind({ kind: "peer" }, { kind: "group" }), null, "상대 ↔ 그룹");
  assert.equal(dropKind({ kind: "group" }, { kind: "peer" }), null);
  assert.equal(dropKind({ kind: "peer" }, { kind: "me" }), null, "'나' 고정");
  assert.equal(dropKind(null, { kind: "peer" }), null);
});

test("with_order — 보낸 순서를 자리로, 빠진 항목은 그대로(서버와 같은 규칙) · 원본 그대로", () => {
  const xs = [
    { id: 1, pos: -1 },
    { id: 2, pos: -2 },
    { id: 3, pos: 7 },
  ];
  const ys = withOrder(xs, [3, 1]);
  assert.deepEqual(ys.map((x) => [x.id, x.pos]), [
    [1, 1],
    [2, -2],
    [3, 0],
  ]);
  assert.deepEqual(sortByPos(ys).map((x) => x.id), [2, 3, 1]);
  assert.equal(xs[0].pos, -1, "원본 그대로");
});

test("moved_text — 화면 읽기 알림(1 부터)", () => {
  assert.equal(movedText("결제 모듈", 0), "결제 모듈 · 1번째로 옮김");
  assert.equal(movedText("이준호", 2), "이준호 · 3번째로 옮김");
});

test("group_leaves — 그룹을 고른 채 내 활동 · 에이전트 · 주고받은 메시지(#1j)은 '나' 로, 그 밖(즐겨찾기 포함)은 그대로", () => {
  assert.deepEqual(
    ["home", "activity", "requests", "starred", "saved", "agents", "peer", "conversation", "new"].filter(groupLeaves),
    ["activity", "agents", "peer"],
  );
});

test("group_unread — 그룹 대화의 안 읽음 합(그룹 밖 대화는 세지 않음)", () => {
  const convs = [
    { id: 1, unread: 2 },
    { id: 2, unread: 5 },
    { id: 3 },
  ];
  assert.equal(groupUnread(convs, { convs: [1, 3] }), 2);
  assert.equal(groupUnread(convs, { convs: [] }), 0);
});

test("group_summary — 그룹 요약: 그룹에 대화가 있는 페어링 상대만(레일 순서) + 지운 상대의 대화는 뒤(A11)", () => {
  const names = new Map([
    [1, "가람 · A-PC"],
    [2, "나래 · B-PC"],
    [8, "옛친구 · Z-PC"],
  ]);
  const convs = [
    { id: 11, peer_id: 1, pos: 1 },
    { id: 12, peer_id: 2, pos: 0 },
    { id: 13, peer_id: 8, pos: 2 },
    { id: 14, peer_id: 1, pos: 3 },
  ];
  const mine = inGroup(convs, { convs: [11, 13] });
  const s = peerSummary(mine, groupPeers(mine, [2, 1]), names);
  assert.deepEqual(
    s.map((x) => [x.peer, x.rows.map((c) => c.id)]),
    [
      [1, [11]],
      [8, [13]],
    ],
    "나래(2)는 그룹에 대화가 없어 빠지고, 지운 상대(8)는 뒤",
  );
});

test("group_sections — 그룹 사이드바: ★(그룹 안) + 상대별 절(레일 순서, 지운 상대 뒤), 순서 = 자리", () => {
  const names = new Map([
    [1, "가람 · A-PC"],
    [2, "나래 · B-PC"],
    [8, "옛친구 · Z-PC"],
  ]);
  const convs = sortByPos([
    { id: 21, peer_id: 2, pos: 5, starred: true },
    { id: 22, peer_id: 1, pos: 2 },
    { id: 23, peer_id: 8, pos: 0 },
    { id: 24, peer_id: 1, pos: 1 },
  ]);
  const rows = convs.map((c) => ({ c, label: {} }));
  const secs = convSections(rows, [2, 1], names);
  assert.deepEqual(
    secs.map((x) => [x.key, x.rows.map((r) => r.c.id)]),
    [
      ["peer:2", [21]],
      ["peer:1", [24, 22]],
      ["peer:8", [23]],
    ],
  );
});

test("order_one_for_all — 그룹 화면에서 옮긴 앞뒤가 '나' · 상대 타일에서도 같다(대화마다 순서 하나, A9)", () => {
  const convs = [
    { id: 1, peer_id: 2, pos: 0 },
    { id: 2, peer_id: 3, pos: 1 },
    { id: 3, peer_id: 2, pos: 2 },
    { id: 4, peer_id: 2, pos: 3 },
  ];
  const group = { convs: [1, 4] };
  // 그룹 화면(상대 2 절)에서 4 를 1 앞으로.
  const order = moveInOrder(sortByPos(convs).map((c) => c.id), 4, 1, false);
  const after = sortByPos(withOrder(convs, order));
  assert.deepEqual(inGroup(after, group).map((c) => c.id), [4, 1], "그룹");
  assert.deepEqual(after.filter((c) => c.peer_id === 2).map((c) => c.id), [4, 1, 3], "상대 타일 — 3 은 그대로 1 뒤");
  assert.deepEqual(after.map((c) => c.id), [4, 1, 2, 3], "'나'");
});

test("drop_kind_join — ★ 절의 대화도 그룹 타일에 넣기, '나' · \"+\" 위는 놓기 없음", () => {
  assert.equal(dropKind({ kind: "conv", section: "starred" }, { kind: "group" }), "join");
  assert.equal(dropKind({ kind: "conv", section: "starred" }, { kind: "peer" }), null);
  assert.equal(dropKind({ kind: "group" }, { kind: "conv", section: "peer:2" }), null);
  assert.equal(groupNameError("😀".repeat(40), []), null, "그림 문자도 한 글자(서버 chars 와 같게)");
  assert.equal(groupNameError("😀".repeat(41), []), "그룹 이름은 1 ~ 40자");
});

test("order_to_apply — 저장 중 · 읽는 사이 옮김이면 마지막으로 놓은 순서를 덮는다(늦게 온 옛 응답, 라운드 1), 아니면 받은 그대로", () => {
  const last = [3, 1, 2];
  assert.equal(orderToApply(undefined, 0), null, "옮긴 적 없음");
  assert.equal(orderToApply({ last: null, busy: false, stamp: 4 }, 0), null, "저장 실패로 되돌림");
  assert.equal(orderToApply({ last, busy: true, stamp: 2 }, 2), last, "저장 중");
  assert.equal(orderToApply({ last, busy: false, stamp: 5 }, 3), last, "읽는 사이 옮김 · 저장 끝");
  assert.equal(orderToApply({ last, busy: false, stamp: 5 }, 5), null, "저장 뒤 시작한 읽기 = 서버 값");
  // 역순 응답: 옛 순서([1,2,3])를 담은 늦은 응답이 와도 화면은 마지막으로 놓은 순서.
  const old = [
    { id: 1, pos: 0 },
    { id: 2, pos: 1 },
    { id: 3, pos: 2 },
  ];
  const keep = orderToApply({ last, busy: true, stamp: 1 }, 0);
  assert.deepEqual(sortByPos(withOrder(old, keep)).map((x) => x.id), [3, 1, 2]);
});

test("context_size — 마지막 끝난 구분선 뒤 세션별 마지막 값의 최대 · 구분선 뒤 답 없음 0 · 모름 null(#1m)", () => {
  const a = (ctx, extra = {}) => ({ author: "agent", mine: false, agent: null, ctx_tokens: ctx, state: "done", ...extra });
  assert.equal(contextSize([]), null);
  assert.equal(contextSize([a(null)]), null, "0.4.0 상대 답");
  assert.equal(contextSize([a(100), a(300), a(200)]), 200, "같은 세션은 마지막 값");
  assert.equal(contextSize([a(500), a(900, { agent: "doc" }), a(200)]), 900, "멘션 세션이 더 크면 그 값");
  assert.equal(contextSize([a(500), a(700, { mine: true })]), 700, "자동 왕복 — 내 세션");
  const done = { author: "context", state: "done", ctx_op: "compact" };
  assert.equal(contextSize([a(900), done]), 0, "압축 뒤 답 없음");
  assert.equal(contextSize([a(900), done, a(40)]), 40);
  assert.equal(contextSize([a(900), { ...done, state: "sent" }]), 900, "진행 중 구분선은 아직 아님");
  // 자동 왕복 부분 처리 — 이 PC 쪽만 비웠다, 상대 세션 값은 남는다(라운드 2).
  const partial = { ...done, refused: "peer_compact_failed" };
  assert.equal(contextSize([a(850_000), a(400_000, { mine: true }), partial]), 850_000);
  assert.equal(contextSize([a(400_000, { mine: true }), partial]), 0);
});

test("context_label — 1M 기준 표기 · 60% 주황 · 80% 붉음(#1m)", () => {
  assert.equal(CONTEXT_LIMIT, 1_000_000);
  assert.deepEqual(contextLabel(null), { text: "컨텍스트 알 수 없음", level: "unknown" });
  assert.deepEqual(contextLabel(950), { text: "950 / 1M", level: "ok" });
  assert.deepEqual(contextLabel(42_300), { text: "42k / 1M", level: "ok" });
  assert.equal(contextLabel(599_999).level, "ok");
  assert.equal(contextLabel(600_000).level, "warn");
  assert.equal(contextLabel(800_000).level, "high");
  assert.deepEqual(contextLabel(1_200_000), { text: "1.2M / 1M", level: "high" });
});

test("ctx_menu — 실행 중 · 답 없음이면 꺼짐, 아이콘은 app.css 에(#1m)", () => {
  const css = readFileSync(new URL("./app.css", import.meta.url), "utf8");
  assert.deepEqual(ctxMenu(false, true).map((i) => [i.key, i.disabled]), [["compact", false], ["clear", false]]);
  assert.ok(ctxMenu(true, true).every((i) => i.disabled));
  assert.ok(ctxMenu(false, false).every((i) => i.disabled));
  for (const i of ctxMenu(false, true)) assert.ok(css.includes(`.ico-${i.icon} {`), i.icon);
  assert.ok(css.includes(".ico-gauge {"));
  assert.equal(contextBusy([{ author: "human", state: "done", exec: true }]), false);
  assert.equal(contextBusy([{ author: "human", state: "sent", exec: true }]), true, "상대 답 기다림");
  assert.equal(contextBusy([{ author: "human", state: "sent", exec: false }]), false, "메모");
  assert.equal(contextBusy([{ author: "context", state: "sent", exec: false }]), true);
  assert.equal(contextBusy([{ author: "agent", state: "queued" }]), true);
});

test("context_note — 종류 · 상태 · 실패 사유(#1m)", () => {
  const m = (ctx_op, state, refused = null) => ({ author: "context", ctx_op, state, refused });
  assert.equal(contextNote(m("compact", "done")), "컨텍스트 압축 · 이후 새 세션이 요약을 이어 받습니다");
  assert.equal(contextNote(m("auto_compact", "running")), "자동 압축(80%) · 요약 중…");
  assert.equal(contextNote(m("clear", "done")), "컨텍스트 비움 · 이후 새 세션(앞 대화 기억 없음)");
  assert.equal(contextNote(m("clear", "sent")), "컨텍스트 비움 · 상대 PC 처리 대기");
  assert.equal(contextNote(m("compact", "error", "unsupported")), "컨텍스트 압축 · 상대가 0.5.0 이상이어야 합니다");
  assert.equal(contextNote(m("compact", "error", "unreachable")), "컨텍스트 압축 · 상대에 닿지 않음");
  assert.equal(contextNote(m("compact", "error", "compact_failed")), "컨텍스트 압축 · 요약 실행 실패 · 세션은 그대로");
  assert.equal(
    contextNote(m("compact", "done", "peer_unsupported")),
    "컨텍스트 압축 · 이후 새 세션이 요약을 이어 받습니다 · 상대 PC 쪽은 실패(상대가 0.5.0 이상이어야 합니다) · 다시 누르면 다시 요청",
    "자동 왕복 부분 처리(라운드 1)",
  );
});

// ---- 대화 목록 나눔(#1o) ----

const splitConvs = [
  { id: 1, peer_id: 1, dir: "out", title: "빌드 질문", last_at: 50 },
  { id: 2, peer_id: 1, dir: "in", title: "로그 위치", last_at: 40 },
  { id: 3, peer_id: 2, dir: "out", title: "자동 왕복", relay_agent: "review", relay_cap: 5, last_at: 30 },
  { id: 4, peer_id: 2, dir: "in", title: "배포 순서", last_at: 20 },
  { id: 5, peer_id: 1, dir: "out", title: "빌드 옵션", last_at: 10 },
];
const splitNames = new Map([
  [1, "user-b"],
  [2, "user-c"],
]);

test("conv_split_rows — 내 대화 · 에이전트 대화, 순서 · 제목 · 내가 연 자동 왕복 = 내 대화 · 한쪽만 · 검색 뒤 개수", () => {
  const rows = convRows(splitConvs, splitNames, null);
  const parts = splitByDir(rows);
  assert.deepEqual(
    parts.map((g) => [g.dir, g.title, g.rows.map((r) => r.c.id)]),
    [
      ["out", "내 대화", [1, 3, 5]],
      ["in", "에이전트 대화", [2, 4]],
    ],
  );
  assert.equal(splitByDir(rows.filter((r) => r.c.dir === "out")).length, 1, "한쪽만 = 묶음 하나(머리 없음)");
  assert.deepEqual(splitByDir([]), []);
  const found = splitByDir(convRows(splitConvs, splitNames, null, "빌드"));
  assert.deepEqual(found.map((g) => [g.title, g.rows.length]), [["내 대화", 2]], "검색으로 에이전트 대화가 0 이면 그 묶음이 빠진다");
});

test("conv_split_scopes — 상대 타일 · 그룹 · '나'(상대별 절 안) 같은 규칙 · 열쇠 {절}:{dir}", () => {
  const rows = convRows(splitConvs, splitNames, null);
  const me = convSections(rows, [1, 2], splitNames).map((s) => [s.key, splitByDir(s.rows).map((g) => [splitKey(s.key, g.dir), g.rows.map((r) => r.c.id)])]);
  assert.deepEqual(me, [
    [
      "peer:1",
      [
        ["peer:1:out", [1, 5]],
        ["peer:1:in", [2]],
      ],
    ],
    [
      "peer:2",
      [
        ["peer:2:out", [3]],
        ["peer:2:in", [4]],
      ],
    ],
  ]);
  const tile = convSections(convRows(splitConvs, splitNames, 1));
  assert.deepEqual(
    tile.map((s) => splitByDir(s.rows).map((g) => [splitKey(s.key, g.dir), g.rows.length])),
    [
      [
        ["convs:out", 2],
        ["convs:in", 1],
      ],
    ],
    "상대 타일 · 그룹 = 대화 절 하나 안에서",
  );
});

test("conv_split_drop — 다른 묶음으로는 놓지 못함 · 같은 묶음은 옮기기 · 그룹 타일 넣기는 두 묶음 모두", () => {
  const conv = (section) => ({ kind: "conv", section });
  assert.equal(dropKind(conv("peer:1:out"), conv("peer:1:in")), null);
  assert.equal(dropKind(conv("peer:1:out"), conv("peer:1:out")), "move");
  assert.equal(dropKind(conv("convs:in"), conv("convs:in")), "move");
  for (const s of ["peer:1:out", "peer:1:in"]) assert.equal(dropKind(conv(s), { kind: "group", section: null }), "join");
});

test("conv_split_starred — 즐겨찾기 화면은 나누지 않는다(절 열쇠 · 제목 그대로)", () => {
  const convs = splitConvs.map((c) => ({ ...c, starred: c.id !== 5 }));
  assert.deepEqual(
    starredSections(convs, splitNames, [1, 2]).map((s) => [s.key, s.rows.map((r) => r.c.id)]),
    [
      ["peer:1", [1, 2]],
      ["peer:2", [3, 4]],
    ],
  );
  assert.deepEqual(starredSections(convs, splitNames).map((s) => [s.key, s.title]), [["convs", "대화"]]);
});

// ---- 공유 대화(#1o 관람자) ----

const shared = (over = {}) => ({
  id: 9,
  peer_id: 1,
  dir: "shared",
  agent: "review",
  title: "빌드 질문",
  share: { other: "user-c", owner_dir: "out", ended: null },
  ...over,
});

test("share_groups — 공유 대화 = 셋째 묶음(내 대화 · 에이전트 대화 뒤) · 해제되면 '공유 해제됨' 태그", () => {
  const rows = convRows([...splitConvs, shared()], splitNames, null);
  assert.deepEqual(
    splitByDir(rows).map((g) => [g.title, g.rows.map((r) => r.c.id)]),
    [
      ["내 대화", [1, 3, 5]],
      ["에이전트 대화", [2, 4]],
      ["공유 대화", [9]],
    ],
  );
  assert.equal(convMeta(shared()).tag, null);
  assert.equal(convMeta(shared({ share: { other: "user-c", owner_dir: "out", ended: 100 } })).tag, "공유 해제됨");
  assert.equal(convMeta(splitConvs[0]).tag, null);
  assert.equal(convMeta(shared()).agent, "review", "공유 대화의 에이전트에 '내' 를 붙이지 않는다");
});

test("share_labels — 제목 · 줄 설명 · 발화자(초대한 사람 · 원 상대 · 각자의 에이전트)", () => {
  const c = shared();
  assert.deepEqual(convLabel(c, splitNames), { title: "빌드 질문", sub: "공유 · user-b ↔ user-c" });
  assert.equal(authorLabel(c, { author: "human", owner: true }, "user-b"), "user-b");
  assert.equal(authorLabel(c, { author: "human", owner: false }, "user-b"), "user-c");
  // 초대한 사람이 연 대화 — 대화 에이전트는 원 상대 것.
  assert.equal(authorLabel(c, { author: "agent", owner: false }, "user-b"), "user-c/review");
  // 받은 대화 — 대화 에이전트는 초대한 사람 것.
  const inbound = shared({ share: { other: "user-c", owner_dir: "in", ended: null } });
  assert.equal(authorLabel(inbound, { author: "agent", owner: true }, "user-b"), "user-b/review");
  assert.equal(authorLabel(c, { author: "human", owner: true, via: "cli" }, "user-b"), "user-b/AI 에이전트(CLI)");
  assert.equal(sharedAuthor(c, { author: "agent", owner: true, agent: "relay-x" }, "user-b"), "user-b/relay-x");
});

test("share_read_only — 자동 왕복 표시 · 동작 없음(사본은 mine 이 없다) · 공유한 사람 글은 오른쪽", () => {
  assert.equal(relayText(shared({ relay_agent: "review", relay_cap: 5, relay_on: true }), [{ author: "agent", mine: false }]), null);
  assert.equal(bubbleSide({ mine: false, owner: true }), "mine", "공유한 사람 쪽 글은 오른쪽");
  assert.equal(bubbleSide({ mine: false, owner: false }), "theirs", "원 상대 쪽 글은 왼쪽");
});

test("share_head_candidates — 머리 표시 · 관람자 후보(원 상대 · 이미 관람자 · 페어링 안 됨 제외, 5명 상한, 사본은 없음)", () => {
  assert.equal(shareHead(shared()), "읽기 전용");
  assert.equal(shareHead(shared({ share: { other: "user-c", owner_dir: "in", ended: 7 } })), "공유 해제됨");
  assert.equal(shareHead({ dir: "out", observers: [2, 3] }), "관람자 2");
  assert.equal(shareHead({ dir: "in", watchers: 1 }), "관람자 1");
  assert.equal(shareHead({ dir: "out" }), "");
  const peers = [1, 2, 3, 4, 5, 6, 7, 8].map((id) => ({ id, paired: id !== 8 }));
  const conv = { dir: "out", peer_id: 1, observers: [2] };
  assert.deepEqual(shareCandidates(conv, peers).map((p) => p.id), [3, 4, 5, 6, 7]);
  assert.equal(MAX_OBSERVERS, 5);
  assert.deepEqual(shareCandidates({ ...conv, observers: [2, 3, 4, 5, 6] }, peers), [], "5명이면 더 넣지 못한다");
  assert.deepEqual(shareCandidates(shared(), peers), [], "공유 대화는 다시 공유하지 않는다");
});

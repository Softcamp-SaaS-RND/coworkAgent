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
  kindText,
  relayText,
  keepsConversation,
  formatDuration,
  formatTime,
  timeline,
  unlinkedReplies,
  route,
  Latest,
  settingsTree,
  groupFilter,
  groupNames,
  NO_GROUP,
  inboxRows,
  isHtmlDoc,
  htmlSrcdoc,
  markdownBlocks,
  peerMessages,
  tileText,
  displayName,
  convRows,
  tileKeeps,
  sideNav,
  newMenu,
  profileMenu,
  convSections,
  convMeta,
  unreadByPeer,
  activityRows,
  messageActions,
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
  mentionQuery,
  mentionTarget,
  hasMention,
  mentionToken,
} from "./view.js";

test("상태 문구", () => {
  assert.equal(stateText("running"), "실행 중");
  assert.equal(stateText("undelivered"), "답 미전달");
  assert.equal(stateText("weird"), "weird");
  assert.equal(stateText(undefined), "");
  assert.equal(peerState({ paired: true }), "페어링됨");
  assert.equal(peerState({ paired: false, my_ok: false, their_ok: true }), "상대 승인함 — 내 승인 대기");
  assert.equal(peerState({ paired: false, my_ok: true, their_ok: false }), "상대 승인 대기");
});

test("대화 이름 — 제목 · 방향 · 상대", () => {
  const names = new Map([[2, "이도윤 · DYOON-MAC"]]);
  assert.deepEqual(convLabel({ peer_id: 2, dir: "out", agent: "review", title: "가드 봐 줘" }, names), {
    title: "가드 봐 줘",
    sub: "이도윤 · DYOON-MAC 의 review",
    tag: "보냄",
  });
  assert.deepEqual(convLabel({ peer_id: 9, dir: "in", agent: "dev", title: "" }, names), {
    title: "내 dev 가 받은 요청",
    sub: "알 수 없는 상대 의 요청 · 내 dev",
    tag: "받음",
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
  assert.equal(tl2[0].steps[2].title, "응답 — 답 미전달");
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
  assert.deepEqual(route("#agents"), { view: "agents" });
  assert.deepEqual(route("#me"), { view: "settings", focus: "me" });
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
  assert.deepEqual(relayText(out, []), { text: "자동 왕복 3 / 10 — 진행 중", action: "stop" });
  assert.deepEqual(relayText({ ...out, relay_on: false }, []), { text: "자동 왕복 3 / 10 — 멈춤", action: "resume" });
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
  assert.equal(kindText({ author: "human", exec: true, via: "cli" }), "AI 에이전트(CLI) → 에이전트");
  assert.equal(kindText({ author: "human", exec: false, via: "cli" }), "AI 에이전트(CLI)");
  assert.equal(kindText({ author: "human", exec: true }), "사람 → 에이전트");
  assert.equal(kindText({ author: "human", exec: false }), "사람");
  assert.equal(kindText({ author: "agent", via: null }), "에이전트");
  const items = timeline([
    { id: 1, author: "human", exec: true, text: "q", via: "cli" },
    { id: 2, author: "agent", reply_to: 1, text: "a", state: "done" },
    { id: 3, author: "human", exec: true, text: "w" },
  ]);
  assert.equal(items[0].steps[0].title, "요청 — AI 에이전트(CLI)");
  assert.equal(items[1].steps[0].title, "요청");
});

test("settings_tree — 설정 항목 전부 · 연결 아래 초대 생성 · 초대 수락 · 연결 목록", () => {
  const tree = settingsTree();
  const flat = tree.flatMap((n) => [n, ...(n.children ?? [])]);
  assert.deepEqual(
    flat.map((n) => n.label),
    ["내 정보", "연결", "초대 생성", "초대 수락", "연결 목록", "내 에이전트", "실행 — 모델 · effort", "화면 — 테마", "알림"],
  );
  assert.equal(new Set(flat.map((n) => n.id)).size, flat.length, "id 는 절 요소 id — 겹치지 않는다");
  const html = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  for (const n of flat) assert.ok(html.includes(`<section id="${n.id}"`), `index.html 에 절 ${n.id}`);
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

test("group_filter_search — 그룹 필터 · 검색 · 그룹 없음은 끝", () => {
  const peers = [
    { id: 1, name: "user2", host: "DEV-PC", group: "개발팀" },
    { id: 2, name: "kim", host: "QA-01", group: null },
    { id: 3, name: "lee", host: "DEV-LAP", group: "QA" },
    { id: 4, name: "park", host: "dev-mac", group: "개발팀" },
  ];
  const fields = ["name", "host"];
  assert.deepEqual(
    groupFilter(peers, { fields }).map((g) => [g.group, g.items.map((p) => p.id)]),
    [["개발팀", [1, 4]], ["QA", [3]], [NO_GROUP, [2]]],
    "ko 정렬 — 한글이 먼저",
  );
  assert.deepEqual(groupFilter(peers, { group: "개발팀", fields }).map((g) => g.group), ["개발팀"]);
  assert.deepEqual(groupFilter(peers, { group: NO_GROUP, fields })[0].items.map((p) => p.id), [2]);
  assert.deepEqual(
    groupFilter(peers, { q: " dev ", fields }).flatMap((g) => g.items.map((p) => p.id)),
    [1, 4, 3],
    "대소문자 무시 · 앞뒤 공백 무시",
  );
  assert.deepEqual(groupFilter(peers, { q: "없음", fields }), []);
  assert.deepEqual(groupNames(peers), ["개발팀", "QA", NO_GROUP]);
  assert.deepEqual(groupNames(peers.filter((p) => p.group)), ["개발팀", "QA"]);
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
    rows.map((r) => [r.id, r.who, r.kind, r.tag, r.href]),
    [
      [9, "user2", "사람 → 에이전트", "받음", "#c/5"],
      [8, "user2/review", "에이전트", "보냄", "#c/4"],
    ],
  );
  assert.equal(rows[0].preview, "로그인 오류 정리");
  assert.equal(rows[1].preview.length, 121, "120자 + …");
  assert.deepEqual(inboxRows(ms, names, { kind: "agent" }).map((r) => r.id), [8]);
  assert.deepEqual(inboxRows(ms, names, { kind: "human" }).map((r) => r.id), [9]);
});

test("peer_messages — 레일 상대 = 주고받은 글, 최신순 합침 · 상한", () => {
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

test("direction_tag — 방향은 화살표 대신 보냄 · 받음 태그", () => {
  const names = new Map([[2, "user2 · DEV-PC"]]);
  const out = convLabel({ peer_id: 2, dir: "out", agent: "guide", title: "→ 배포 절차" }, names);
  const inn = convLabel({ peer_id: 2, dir: "in", agent: "guide", title: "" }, names);
  assert.deepEqual([out.tag, inn.tag], ["보냄", "받음"]);
  assert.equal(out.title, "→ 배포 절차", "제목 글자는 건드리지 않는다");
  for (const l of [out, inn]) assert.ok(!/[→←]/.test(l.sub), l.sub);
  assert.ok(!/[→←]/.test(inn.title));
});

test("tile_text — 타일 글자 = 표시 이름(없으면 원래 이름)의 앞 두 글자", () => {
  assert.equal(tileText("user-b"), "us");
  assert.equal(tileText("세일즈팀"), "세일");
  assert.equal(tileText("나"), "나");
  assert.equal(tileText("👨‍👩‍👧팀장"), "👨‍👩‍👧팀", "이모지 묶음을 반으로 자르지 않는다");
  assert.equal(tileText(""), "");
  assert.equal(tileText(undefined), "");
  assert.equal(displayName({ name: "user-b", alias: "세일즈" }), "세일즈");
  assert.equal(displayName({ name: "user-b", alias: null }), "user-b");
  assert.equal(displayName({ name: "user-b", alias: "" }), "user-b", "빈 표시 이름 = 원래 이름");
});

test("side_nav — 고른 타일 범위의 내비, 내 활동은 안 본 수, 주고받은 글은 상대 타일만", () => {
  assert.deepEqual(
    sideNav(null, 3).map((n) => [n.label, n.href, n.count ?? 0]),
    [
      ["대화", "#", 0],
      ["내 활동", "#activity", 3],
      ["저장된 항목", "#saved", 0],
      ["에이전트", "#agents", 0],
    ],
  );
  const peer = sideNav(2);
  assert.deepEqual(peer.map((n) => n.label), ["대화", "내 활동", "저장된 항목", "에이전트", "주고받은 글"]);
  assert.equal(peer[1].count, 0, "안 본 수 기본 0");
  assert.deepEqual(peer.map((n) => route(n.href).view), ["home", "activity", "saved", "agents", "peer"]);
  assert.deepEqual(route(peer.at(-1).href), { view: "peer", id: 2 });
});

test("new_menu — 다섯 항목이 각 화면으로, 초점 칸까지", () => {
  const items = newMenu();
  assert.deepEqual(items.map((i) => i.label), ["새 대화", "자동 왕복 대화", "초대 생성", "초대 수락", "에이전트 등록"]);
  assert.deepEqual(items.map((i) => route(i.href)), [
    { view: "new" },
    { view: "new" },
    { view: "settings", focus: "invite_make" },
    { view: "settings", focus: "invite_accept" },
    { view: "settings", focus: "agents" },
  ]);
  assert.deepEqual(items.map((i) => i.focus), ["chat_peer", "chat_relay_agent", "window_btn", "invite_text", "agent_name"]);
  const html = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  for (const i of items) assert.ok(html.includes(`id="${i.focus}"`), `index.html 에 ${i.focus}`);
});

test("profile_menu — 표시 이름 · 아이콘 · 집중(켜짐 표시) · 알림 일시 중지(켜짐 표시) · 환경설정", () => {
  assert.deepEqual(
    profileMenu(false, false).map((i) => [i.key, i.label, i.checked]),
    [
      ["tile", "표시 이름 · 아이콘", undefined],
      ["focus", "집중 — 요청 받지 않음", false],
      ["pause", "알림 일시 중지", false],
      ["settings", "환경설정", undefined],
    ],
  );
  assert.equal(profileMenu(true, false)[2].checked, true);
  assert.equal(profileMenu(false, true)[1].checked, true);
  assert.equal(profileMenu(undefined, undefined)[2].checked, false, "설정을 못 읽었으면 꺼짐");
  assert.equal(profileMenu(undefined, undefined)[1].checked, false);
  assert.deepEqual(route(profileMenu(false, false)[3].href), { view: "settings", focus: null });
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

test("conv_sections — ★ 대화는 즐겨찾기에만, 행 표시 · 레일 배지", () => {
  const names = new Map([[2, "이도윤 · DYOON"]]);
  const convs = [
    { id: 1, peer_id: 2, dir: "out", agent: "guide", title: "배포", starred: true, last_state: "done", unread: 0 },
    { id: 2, peer_id: 2, dir: "in", agent: "review", title: "로그인", starred: false, last_state: "running", unread: 2 },
    { id: 3, peer_id: 3, dir: "out", agent: "guide", title: "릴리스", starred: false, relay_agent: "dev", relay_cap: 10, relay_count: 3, unread: 1 },
  ];
  assert.deepEqual(
    convSections(convRows(convs, names, null)).map((s) => [s.title, s.rows.map((r) => r.c.id)]),
    [
      ["★ 즐겨찾기", [1]],
      ["대화", [2, 3]],
    ],
  );
  assert.deepEqual(convSections(convRows(convs, names, 3)).map((s) => s.title), ["대화"], "즐겨찾기가 없으면 그 섹션도 없다");
  assert.deepEqual(convMeta(convs[0]), { agent: "guide", state: "완료", running: false, relay: null, unread: false });
  assert.deepEqual(convMeta(convs[1]), { agent: "내 review", state: "실행 중", running: true, relay: null, unread: true });
  assert.deepEqual(convMeta(convs[2]), { agent: "guide", state: "", running: false, relay: "왕복 3/10", unread: true });
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
      ["에이전트", "이도윤/guide", "답 둘째 줄", "#c/5/m/9", true],
      ["실패", "나", "요청", "#c/5/m/8", false],
      ["자동 왕복 끝 — 상한 도달", "이도윤", "", "#c/5", false],
      ["자동 왕복 시작", "이도윤", "첫 답 앞부분", "#c/5", false],
      ["에이전트 · 실패", "이도윤/guide", "실행 실패", "#c/5/m/7", false],
    ],
    "상대 에이전트의 실패 답은 실패 상태로",
  );
  assert.ok(rows.every((r) => r.title === "배포 절차" && r.tag === "보냄"));
  assert.equal(new Set(rows.map((r) => r.key)).size, rows.length, "항목 열쇠는 겹치지 않는다");
});

test("message_actions — 저장(저장 해제) · 복사, 내 실행 요청만 다시 보내기", () => {
  const labels = (m) => messageActions(m).map((a) => a.label);
  assert.deepEqual(labels({ author: "human", mine: true, exec: true, saved: false }), ["저장", "복사", "다시 보내기"]);
  assert.deepEqual(labels({ author: "human", mine: true, exec: true, saved: true }), ["저장 해제", "복사", "다시 보내기"]);
  assert.deepEqual(labels({ author: "human", mine: false, exec: true }), ["저장", "복사"], "상대 글");
  assert.deepEqual(labels({ author: "agent", mine: true }), ["저장", "복사"], "에이전트 답");
  assert.deepEqual(labels({ author: "human", mine: true, exec: false }), ["저장", "복사"], "실행 없는 글");
  for (const m of [{ author: "human", mine: true, exec: true }, { author: "agent", mine: false }]) {
    assert.ok(!messageActions(m).some((a) => /편집|삭제|원문/.test(a.label)), "메시지별 편집 · 삭제 · 원문 없음");
  }
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

test("ask_marks — 요청 태그 · 완료 작업(받은 요청만) · 내 활동 · 보낸 메시지 라벨", () => {
  const req = (mine, done) => ({ author: "human", mine, exec: true, request: true, done });
  assert.equal(askTag(req(true, false)), "요청");
  assert.equal(askTag(req(false, true)), "요청 · 완료");
  assert.equal(askTag({ author: "human", request: false }), "", "질의는 표시 없음");
  assert.equal(askTag({ author: "agent", request: true }), "", "답에는 없다");
  const acts = (m) => messageActions(m).map((x) => [x.key, x.label]);
  assert.deepEqual(acts(req(false, false)).at(-1), ["done", "완료 표시"]);
  assert.deepEqual(acts(req(false, true)).at(-1), ["done", "완료 취소"]);
  assert.ok(!acts(req(true, false)).some(([k]) => k === "done"), "내 요청은 못 바꾼다");
  assert.ok(!acts({ author: "human", mine: false, exec: true, request: false }).some(([k]) => k === "done"), "질의");
  const conv = { id: 1, peer_id: 2, agent: "guide", dir: "in", title: "t" };
  const rows = activityRows([{ kind: "message", at: 1, unseen: false, conversation: conv, message: { id: 5, ...req(false, true), text: "x", state: "done" } }], new Map());
  assert.equal(rows[0].label, "사람 → 에이전트 · 요청 · 완료");
  const sent = inboxRows([{ id: 6, ...req(true, false), text: "y", created_at: 1, conversation: { ...conv, dir: "out" } }], new Map());
  assert.equal(sent[0].kind, "사람 → 에이전트 · 요청");
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
  assert.equal(progressText(q), "상대 PC 에 처리 중인 작업이 많아 응답을 기다립니다(앞에 2건)");
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
  assert.equal(r.title, "이 PC 실행 — 실행 중 1 · 대기 2");
  assert.deepEqual(r.rows, [
    { label: "실행 중 · 민수/review", text: "로그 봐줘", href: "#c/3" },
    { label: "대기 1 · 지은/review", text: "a", href: "#c/5" },
    { label: "대기 2 · 알 수 없는 상대/에이전트", text: "", href: "" },
  ]);
  assert.equal(runsSummary({ cap: 2, running: [], waiting: [] }).title, "이 PC 실행 — 실행 중 0 · 대기 0");
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
  assert.equal(outboxText({ outbox: { state: "pending", tries: 3 } }), "보내는 중 — 상대에 닿지 않아 다시 보냄(3회)");
  assert.equal(outboxText({ outbox: { state: "failed", tries: 9 } }), "", "포기는 거절 사유가 말한다");
  assert.equal(outboxText({ outbox: null }), "");
  assert.match(refusedText({ state: "error", refused: "unreachable" }), /^보내지 못함/);
  const sent = { id: 1, state: "sent", text: "q", done: false, refused: null };
  assert.notEqual(messagesSigOf("x", [{ ...sent, outbox: { state: "pending", tries: 1 } }]), messagesSigOf("x", [sent]), "닿으면 다시 그린다");
});

test("stepText · progressText — 처리 중 단계 요약(#1c)", () => {
  const d = { turns: 2, tools: { Read: 3, Grep: 1 }, secs: 41 };
  assert.equal(stepText(d), "2턴 · Read 3 · Grep 1 · 41초");
  assert.equal(stepText({ turns: 0, tools: {}, secs: 0 }), "");
  assert.equal(progressText({ state: "sent", progress: { state: "running", ahead: 0, detail: d } }), "처리 중 — 2턴 · Read 3 · Grep 1 · 41초");
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

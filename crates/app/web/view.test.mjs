// node --test crates/app/web/ — 화면 로직(js/logic/) 시험. DOM 없음.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import {
  stateText,
  dayKey,
  dayLabel,
  dayBreaks,
  formatDuration,
  formatTime,
  DATE_FORMATS,
  setDateFormat,
  timeKey,
  josa,
  fileSize,
  displayName,
} from "./js/logic/format.js";
import {
  convLabel,
  filterByPeer,
  roomPreview,
  starDetail,
  agentTag,
  keepsConversation,
  LEAVE_CONFIRM,
  composeDefaults,
  requestNotice,
  convMenuItems,
  convClosed,
  convRows,
  convSections,
  splitByDir,
  shareHead,
  shareCandidates,
  MAX_OBSERVERS,
  convMeta,
  convListParts,
  visibleConvRows,
  agentConvSections,
  convInfoRows,
  INFO_CMD_HINT,
  RELAY_STOPPED,
  convKindIcon,
  asksKind,
} from "./js/logic/conv.js";
import {
  relayText,
  timeline,
  unlinkedReplies,
  messageActions,
  editedTag,
  insertMarkdown,
  enterSends,
  runSteps,
  lastRun,
  refusedText,
  progressText,
  outboxText,
  stepText,
  messagesSigOf,
  mentionQuery,
  mentionTarget,
  hasMention,
  mentionToken,
  speakerIcon,
  noteCount,
  avatarKind,
} from "./js/logic/message.js";
import {
  contextSize,
  contextLabel,
  contextBusy,
  ctxMenu,
  contextNote,
  CONTEXT_LIMIT,
} from "./js/logic/context.js";
import {
  isHtmlDoc,
  htmlSrcdoc,
  markdownBlocks,
} from "./js/logic/markdown.js";
import {
  peerState,
  presenceOf,
  unreadByPeer,
  presence,
  peerWeeksOptions,
  PEER_WEEKS,
  peerGroups,
  peerSummary,
  waitingByPeer,
  tileUnread,
  meTint,
  ownTint,
  tileTint,
} from "./js/logic/peer.js";
import {
  inboxRows,
  peerSummaryRows,
  pageMore,
  peerMsgsSig,
  mergePage,
  PEER_PAGE,
} from "./js/logic/inbox.js";
import {
  authorLabel,
  bubbleSide,
  sharedAuthor,
  ownHuman,
} from "./js/logic/author.js";
import {
  sortByPos,
  moveInOrder,
  withOrder,
  movedText,
  orderToApply,
} from "./js/logic/order.js";
import {
  tileKeeps,
  splitKey,
  inGroup,
  groupPeers,
  scopeQuery,
  groupNameError,
  membershipDiff,
  stepTarget,
  dropKind,
  groupLeaves,
  groupUnread,
  RAIL_W,
  railSize,
  groupSettingsList,
  railSlot,
  peerLeaves,
  DRAG_SLOP,
  pastSlop,
} from "./js/logic/rail.js";
import {
  previewSections,
  previewZoom,
  HOME_ZOOMS,
  previewBubbles,
  previewKey,
  previewStaleOf,
  PREVIEW_MSGS,
} from "./js/logic/home.js";
import {
  reqResend,
  boardColumns,
  canDrop,
  reqFeedback,
  askTag,
  reqMoves,
  requestRows,
  reqBoxes,
  reqHead,
  reqArchiveAction,
  REQ_BTN,
  REQ_STATES,
  REQ_FOLD,
  reqFold,
} from "./js/logic/request.js";
import {
  activityRows,
  ACT_KINDS,
} from "./js/logic/activity.js";
import {
  settingsTree,
  SETTINGS_ALIAS,
  logLines,
  diagText,
  draftOf,
  pruneDrafts,
  portText,
  MODELS,
} from "./js/logic/settings.js";
import {
  TOOLS_NOTE,
  ENGINES,
  WEB_TOOLS,
  EDIT_TOOLS,
  engineLabel,
  agentEngineBlocks,
  READ_ONLY_TOOLS,
  SHELL_NOTE,
  SHELL_WARN,
  shellConfirmNeeded,
  agentPatch,
  AGENT_COLORS,
  agentColor,
  agentSavedText,
  agentSaveFailText,
  agentDeleteAsk,
} from "./js/logic/agent.js";
import {
  entryToken,
  DISCONNECTED,
  route,
  SESSION_NEW,
  sideSub,
  starredSections,
  sideNav,
  newMenu,
  profileMenu,
  holdsRender,
} from "./js/logic/shell.js";
import {
  Latest,
  runsSummary,
  SendLatest,
  CONV_LOADERS,
  loadersFor,
  mergeChanges,
  unionLoaders,
  OneAtATime,
} from "./js/logic/sync.js";
import {
  searchItems,
  searchGroups,
} from "./js/logic/search.js";

// 화면 원본 — `js/` 모듈을 이어 붙인 글(`logic` = DOM 없는 `js/logic/`, 아니면 그 밖 UI 모듈 전부).
const source = (logic) =>
  readdirSync(new URL("./js/", import.meta.url), { recursive: true })
    .map((f) => f.replaceAll("\\", "/"))
    .filter((f) => f.endsWith(".js") && f.startsWith("logic/") === logic)
    .sort()
    .map((f) => readFileSync(new URL(`./js/${f}`, import.meta.url), "utf8"))
    .join("\n");

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
  assert.equal(formatTime(null), "");
});

test("agent_dialogs — 에이전트 상세 저장 완료 · 실패 창 · 삭제 확인 글", () => {
  assert.equal(agentSavedText("가이드"), "에이전트 가이드를 저장했습니다.");
  assert.equal(agentSavedText("guide"), "에이전트 guide을(를) 저장했습니다.");
  assert.equal(agentSaveFailText("가이드", "폴더가 없습니다"), "에이전트 가이드 저장 실패: 폴더가 없습니다");
  const ask = agentDeleteAsk("검토봇");
  assert.ok(ask.startsWith("에이전트 검토봇을 삭제할까요?"), ask);
  assert.ok(ask.includes("세션"), "이어 쓰던 세션이 끊김을 알린다");
});

test("format_time — 날짜 표시방식 md(기본) · relative · ymd · time, 다른 해 · 초", () => {
  const at = (y, mo, d, h, mi, se = 0) => new Date(y, mo - 1, d, h, mi, se).getTime() / 1000;
  const now = at(2026, 10, 28, 20, 0);
  const t = at(2026, 10, 28, 14, 3, 21);
  const y = at(2026, 10, 27, 9, 5);
  const old = at(2026, 10, 25, 18, 0);
  const past = at(2025, 12, 31, 9, 15);
  const f = (ts, fmt, seconds = false) => formatTime(ts, { fmt, now, seconds });
  assert.deepEqual([f(t, "md"), f(y, "md"), f(past, "md")], ["10.28 14:03", "10.27 09:05", "2025.12.31 09:15"]);
  assert.deepEqual([f(t, "relative"), f(y, "relative"), f(old, "relative"), f(past, "relative")], ["오늘 14:03", "어제 09:05", "10.25 18:00", "2025.12.31 09:15"]);
  assert.deepEqual([f(t, "ymd"), f(past, "ymd")], ["2026-10-28 14:03", "2025-12-31 09:15"]);
  assert.equal(f(t, "time"), "14:03");
  assert.deepEqual([f(t, "md", true), f(t, "relative", true), f(t, "ymd", true), f(t, "time", true)], ["10.28 14:03:21", "오늘 14:03:21", "2026-10-28 14:03:21", "14:03:21"]);
  assert.deepEqual(DATE_FORMATS, ["md", "relative", "ymd", "time"]);
  assert.equal(formatTime(t, { now }), "10.28 14:03", "기본 = md");
  assert.equal(setDateFormat("ymd"), true);
  assert.equal(setDateFormat("ymd"), false, "같은 값 = 안 바뀜");
  assert.equal(formatTime(t, { now }), "2026-10-28 14:03", "고른 방식을 따른다");
  setDateFormat("huge");
  assert.equal(formatTime(t, { now }), "10.28 14:03", "모르는 값 = md");
});

test("time_key — 그린 시각의 다시 그리기 열쇠: 날짜 표시방식 · 지역 날짜(자정 · 해 넘김에 바뀜)", () => {
  const at = (y, mo, d, h, mi) => new Date(y, mo - 1, d, h, mi).getTime() / 1000;
  assert.equal(timeKey(at(2026, 10, 28, 0, 1)), timeKey(at(2026, 10, 28, 23, 59)), "같은 날 = 같은 열쇠");
  assert.notEqual(timeKey(at(2026, 10, 28, 23, 59)), timeKey(at(2026, 10, 29, 0, 1)), "자정 넘김");
  assert.notEqual(timeKey(at(2026, 12, 31, 23, 59)), timeKey(at(2027, 1, 1, 0, 1)), "해 넘김");
  const now = at(2026, 10, 28, 12, 0);
  const md = timeKey(now);
  setDateFormat("relative");
  assert.notEqual(timeKey(now), md, "방식이 바뀌면 다른 열쇠");
  setDateFormat("md");
  assert.ok(messagesSigOf("p", []).endsWith(timeKey()), "메시지 서명에 붙는다");
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
  assert.deepEqual(route("#settings"), { view: "settings", page: "conn", sub: null });
  assert.deepEqual(route("#settings/invite_accept"), { view: "settings", page: "conn", sub: "invite_accept" });
  assert.deepEqual(route("#settings/nope"), { view: "settings", page: "conn", sub: null }, "모르는 = 연결(#1z-8)");
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
  assert.equal(relayText({ ...inn, relay_blocked: true }, [{ author: "agent", mine: false }]), null, "받은 쪽이 멈춘 뒤 — 표시 없음(재개는 연 쪽)");
});

test("세부정보 요청 — 최근 3건, 더보기로 펼치고 접기로 접는다", () => {
  const reqs = [5, 4, 3, 2, 1].map((id) => ({ id }));
  assert.equal(REQ_FOLD, 3);
  assert.deepEqual(reqFold(reqs, false), { shown: reqs.slice(0, 3), toggle: "더보기" });
  assert.deepEqual(reqFold(reqs, true), { shown: reqs, toggle: "접기" });
  assert.deepEqual(reqFold(reqs.slice(0, 3), false), { shown: reqs.slice(0, 3), toggle: null }, "3건 이하면 단추 없음");
  assert.deepEqual(reqFold([], true), { shown: [], toggle: null });
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

test("settings_tree — 설정 메뉴 8개 평평하게(#1z-8 사용자 seed 글자 그대로 — 내 정보 없음, #1l) · 화면마다 section 하나 · 처음 숨김", () => {
  const tree = settingsTree();
  assert.deepEqual(
    tree.map((n) => [n.id, n.label, n.icon]),
    [
      ["conn", "연결", "link"],
      ["groups", "그룹설정", "users"],
      ["agents", "에이전트", "bot"],
      ["run", "실행", "play"],
      ["screen", "화면", "monitor"],
      ["startup", "시작", "power"],
      ["notify", "알림", "bell"],
      ["debug", "디버깅", "bug"],
    ],
  );
  assert.ok(tree.every((n) => !n.children && !n.press), "하위 · 누름 없음");
  const html = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  const order = [...html.matchAll(/<section id="(\w+)" class="set-page" aria-labelledby="\w+" hidden>/g)].map((m) => m[1]);
  assert.deepEqual(order, tree.map((n) => n.id), "화면 순서 = 메뉴 순서, 모두 처음 숨김");
  assert.ok(!html.includes(`<section id="me"`), "설정에 내 정보 카드 없음(#1l)");
  assert.ok(!html.includes('class="card'), "옛 카드 한 열 없음(#1z-8 — 고른 화면만)");
  for (const [alias, page] of Object.entries(SETTINGS_ALIAS)) assert.ok(tree.some((n) => n.id === page), `${alias} → ${page}`);
});

test("settings_route — 설정 해시(#1z-8): 화면 · 별칭 · 빈 · 모르는 = 연결(첫 화면)", () => {
  assert.deepEqual(route("#settings"), { view: "settings", page: "conn", sub: null }, "빈 = 연결");
  assert.deepEqual(route("#settings/nope"), { view: "settings", page: "conn", sub: null }, "모르는 = 연결");
  for (const id of ["conn", "groups", "agents", "run", "screen", "startup", "notify", "debug"])
    assert.deepEqual(route(`#settings/${id}`), { view: "settings", page: id, sub: null }, id);
  assert.deepEqual(
    Object.keys(SETTINGS_ALIAS).map((a) => [a, route(`#settings/${a}`)]),
    [
      ["invite_make", { view: "settings", page: "conn", sub: "invite_make" }],
      ["invite_accept", { view: "settings", page: "conn", sub: "invite_accept" }],
      ["peers", { view: "settings", page: "conn", sub: "peers" }],
      ["agent_new", { view: "settings", page: "agents", sub: "agent_new" }],
    ],
  );
});

test("menu_icons — 사이드바 · 설정 트리 · 새로 만들기 항목마다 아이콘, 모두 app.css 에 정의(#1l)", () => {
  const css = readFileSync(new URL("./app.css", import.meta.url), "utf8");
  const items = [
    ...[null, 3, "group"].flatMap((t) => sideNav(t).filter((n) => !n.sep)),
    ...settingsTree(),
    ...newMenu(),
  ];
  for (const i of items) {
    assert.ok(i.icon, `${i.label} 아이콘`);
    assert.ok(css.includes(`.ico-${i.icon} {`), `app.css .ico-${i.icon}`);
  }
  assert.deepEqual(
    sideNav(null).filter((n) => !n.sep).map((n) => n.icon),
    ["home", "bell", "inbox", "star", "bookmark", "bot", "bot", "chat"],
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

test("group_summary_conv_member — 그룹 개요의 상대 머리는 그 그룹에 넣은 대화의 상대만(#1w — 그룹 = 대화만)", () => {
  const convs = [
    { id: 1, peer_id: 2, last_at: 10 },
    { id: 2, peer_id: 3, last_at: 20 },
  ];
  const g = { id: 1, convs: [2] };
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

test("in_group_convs_only — 그룹의 대화 = 넣은 대화만(#1w), 옛 응답의 상대 소속(`peers`)은 보지 않는다", () => {
  const convs = [
    { id: 1, peer_id: 2 },
    { id: 2, peer_id: 3 },
    { id: 3, peer_id: 2 },
    { id: 4, peer_id: 4 },
  ];
  assert.deepEqual(inGroup(convs, { convs: [2], peers: [2] }).map((c) => c.id), [2]);
  assert.deepEqual(inGroup(convs, { convs: [4] }).map((c) => c.id), [4]);
  assert.deepEqual(inGroup(convs, { convs: [], peers: [9] }), []);
  assert.deepEqual(groupUnread([{ id: 1, peer_id: 2, unread: 2 }, { id: 5, peer_id: 7, unread: 4 }], { convs: [1], peers: [7] }), 2);
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
  assert.deepEqual(keys({ author: "human", mine: false, request: true, req_state: "wait" }), ["req:accepted", "req:rejected", "req:done"]);
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

test("side_nav — '나' 메뉴 세 절(개요 | 내 활동 · 요청 | 즐겨찾기 · 북마크 · 에이전트 › 등록된 에이전트 · 에이전트 응답), 상대 타일 세 절(개요 | 요청 | …)(#1j · #1v · #1z-2 · 사용자 지시)", () => {
  const show = (xs) => xs.map((n) => (n.sep ? "|" : [n.label, n.href ?? null, n.count ?? 0]));
  assert.deepEqual(show(sideNav(null, 3, 2, 4)), [
    ["개요", "#", 0],
    "|",
    ["내 활동", "#activity", 3],
    ["요청", "#requests", 2],
    "|",
    ["즐겨찾기", "#starred", 4],
    ["북마크", "#saved", 0],
    ["에이전트", null, 0],
    ["등록된 에이전트", "#agents", 0],
    ["에이전트 응답", "#agents/convs", 0],
  ]);
  const peer = sideNav(2, 5, 1);
  assert.deepEqual(show(peer), [
    ["개요", "#", 0],
    "|",
    ["요청", "#requests", 1],
    "|",
    ["즐겨찾기", "#starred", 0],
    ["북마크", "#saved", 0],
    ["에이전트", null, 0],
    ["등록된 에이전트", "#agents", 0],
  ]);
  assert.deepEqual(sideNav(2)[2].count, 0, "대기 수 기본 0");
  assert.deepEqual(peer.filter((n) => !n.sep && n.href).map((n) => route(n.href).view), ["home", "requests", "starred", "saved", "agents"]);
  assert.ok(!sideNav(2)[0].sep && !sideNav(2).at(-1).sep, "구분선은 절 사이에만 — 머리 · 끝에 없다");
  assert.deepEqual(route("#p/2"), { view: "peer", id: 2 }, "옛 주소 — 화면은 그 상대 개요로 넘긴다");
  assert.deepEqual(route("#p"), { view: "peer", id: null }, "'나' 주고받은 메시지 = 전부");
});

test("side_nav_overview_all — '주고받은 메시지' 메뉴 없음(내 활동과 겹침), 모든 타일의 첫 칸 = '개요'(상대는 요청 위 — 사용자 지시)", () => {
  const labels = (tile) => sideNav(tile).filter((n) => !n.sep).map((n) => n.label);
  for (const tile of [null, 7, "group"]) {
    assert.ok(!labels(tile).includes("주고받은 메시지"), String(tile));
    assert.deepEqual(labels(tile).slice(0, 1), ["개요"], String(tile));
  }
  assert.ok(labels(7).indexOf("개요") < labels(7).indexOf("요청"));
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
  assert.equal(activityRows([{ kind: "sync_failed", at: 1, conversation: conv, detail: "삭제" }], names)[0].peer, 4);
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

test("preview_sections — 개요 미리보기(#1z-5 2차 시안 A): 상대마다 절 · 카드 최근 활동 순 · 한 줄(열 수) + 숨은 수, 상대 타일은 전부", () => {
  const names = new Map([
    [1, "가람 · A-PC"],
    [2, "나래 · B-PC"],
  ]);
  const convs = [
    { id: 11, peer_id: 1, last_at: 100, pos: 2 },
    { id: 12, peer_id: 1, last_at: 300, pos: 5 },
    { id: 13, peer_id: 1, last_at: null, pos: -1 },
    { id: 14, peer_id: 1, last_at: 200, pos: 7 },
  ];
  const groups = peerSummary(convs, [2, 1], names, new Map([[1, 3]]));
  const ids = (secs) => secs.map((x) => [x.peer, x.name, x.waiting, x.cards.map((c) => c.id), x.more]);
  assert.deepEqual(
    ids(previewSections(groups, false, 4)),
    [
      [2, "나래", 0, [], 0],
      [1, "가람", 3, [12, 14, 11, 13], 0],
    ],
    "넘치지 않으면 전부 — 최근 활동 순, 시각 없으면 뒤",
  );
  assert.deepEqual(ids(previewSections(groups, false, 3))[1], [1, "가람", 3, [12, 14], 2], "넘치면 열 수 - 1 개 + 숨은 수(끝 칸 \"+n개\")");
  assert.deepEqual(ids(previewSections(groups, false, 1))[1], [1, "가람", 3, [12], 3], "한 열이어도 카드 하나는");
  assert.deepEqual(ids(previewSections(groups, true, 2))[1], [1, "가람", 3, [12, 14, 11, 13], 0], "상대 타일 = 전부");
  assert.deepEqual(
    groups[1].rows.map((c) => c.id),
    [13, 11, 12, 14],
    "peerSummary 의 손으로 놓은 순서는 그대로(미리보기만 최근 순)",
  );
});

test("preview_zoom — 개요 미리보기 크기(#1z-6): 100 ~ 200 25 단계(축소 없음) → 본문 배수만(#1z-8 — 말풍선은 모든 크기 2줄, 줄 수 없음), 목록 밖은 100%", () => {
  assert.deepEqual(HOME_ZOOMS, [100, 125, 150, 175, 200]);
  assert.deepEqual(
    HOME_ZOOMS.map((z) => previewZoom(z)),
    [{ scale: 1 }, { scale: 1.25 }, { scale: 1.5 }, { scale: 1.75 }, { scale: 2 }],
  );
  assert.deepEqual(previewZoom("150"), { scale: 1.5 }, "select 값(글자)도");
  for (const z of [75, 110, 250, undefined, null, "x"]) assert.deepEqual(previewZoom(z), { scale: 1 }, String(z));
});

test("preview_bubbles — 미리보기 말풍선(#1z-4): 마지막 n 개 · 요약 글 뺌 · 쪽 · 이름 · 글자만 · 상태 색", () => {
  const conv = { id: 1, dir: "out", agent: "review", peer_id: 2 };
  const ms = [
    { id: 1, author: "human", mine: true, text: "처음", state: "sent" },
    { id: 2, author: "context", text: "요약", state: "done" },
    { id: 3, author: "human", mine: true, text: "줄\n바꿈  <b>굵게</b>", state: "sent" },
    { id: 4, author: "agent", mine: false, text: "답", state: "done" },
    { id: 5, author: "agent", mine: false, text: "", state: "running" },
    { id: 6, author: "agent", mine: false, text: "", state: "error" },
  ];
  const b = previewBubbles(conv, ms, "나래 · B-PC", 4);
  assert.deepEqual(
    b.map((x) => [x.id, x.side, x.who, x.text, x.tone]),
    [
      [3, "mine", "나", "줄 바꿈 <b>굵게</b>", "human"],
      [4, "theirs", "나래/review", "답", "agent"],
      [5, "theirs", "나래/review", "실행 중", "running"],
      [6, "theirs", "나래/review", "오류", "error"],
    ],
  );
  assert.equal(previewBubbles(conv, ms, "나래").length, 5, "기본 PREVIEW_MSGS(12) — 글 여섯 중 요약 글 뺀 다섯");
  assert.equal(PREVIEW_MSGS, 12);
  // R1 — 마지막이 아닌 글의 편집 · 상태 변경도 말풍선(= 다시 그리기 서명)이 바뀐다.
  const edited = ms.map((m) => (m.id === 4 ? { ...m, text: "고친 답" } : m.id === 5 ? { ...m, state: "done", text: "끝" } : m));
  assert.notDeepEqual(previewBubbles(conv, edited, "나래", 4), b);
  assert.deepEqual(
    previewBubbles(conv, edited, "나래", 4).map((x) => [x.id, x.text, x.tone]),
    [
      [3, "줄 바꿈 <b>굵게</b>", "human"],
      [4, "고친 답", "agent"],
      [5, "끝", "agent"],
      [6, "오류", "error"],
    ],
  );
});

test("preview_reload — 다시 읽을 때(요약 · 변경 신호) — #1z-4 사용자 Q3", () => {
  const c = { id: 7, last_at: 5, last_state: "done", last: { text: "a  b" } };
  assert.equal(previewKey(c), previewKey({ ...c, unread: 3 }), "안 읽음만 바뀌면 다시 읽지 않는다");
  assert.notEqual(previewKey(c), previewKey({ ...c, last_state: "running" }));
  assert.notEqual(previewKey(c), previewKey({ ...c, last: { text: "a c" } }));
  assert.deepEqual(previewStaleOf([{ kind: "conv", id: 3 }, { kind: "conv", id: 4 }]), [3, 4]);
  for (const all of [[], undefined, null, [{ kind: "conv" }], [{ kind: "peer" }], [{ kind: "conv", id: 1 }, { kind: "peer" }]]) {
    assert.equal(previewStaleOf(all), null, JSON.stringify(all));
  }
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
  const app = source(false);
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

test("new_menu — 네 항목이 각 화면으로, 초점 칸까지(자동 왕복은 세션 생성 화면 안 — #1l)", () => {
  const items = newMenu();
  assert.deepEqual(items.map((i) => i.label), ["세션 생성", "초대 생성", "초대 수락", "에이전트 등록"]);
  assert.deepEqual(items.map((i) => route(i.href)), [
    { view: "new" },
    { view: "settings", page: "conn", sub: "invite_make" },
    { view: "settings", page: "conn", sub: "invite_accept" },
    { view: "settings", page: "agents", sub: "agent_new" },
  ]);
  assert.deepEqual(items.map((i) => i.focus), ["chat_peer", "window_btn", "invite_text", "agent_engine_claude"]); // #1z-9 등록 1단계 = 도구 타일
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
  assert.deepEqual(route(profileMenu(false, false).at(-1).href), { view: "settings", page: "conn", sub: null }, "설정 첫 화면 = 연결(#1z-8)");
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

test("activity_rows — 받은 요청 · 받은 답 · 내 요청 · 실패(#1q), 안 본 항목, 누르면 그 글", () => {
  const names = new Map([[2, "이도윤 · DYOON"]]);
  const conv = { id: 5, peer_id: 2, agent: "guide", dir: "out", title: "배포 절차" };
  const items = [
    { kind: "message", at: 300, unseen: true, conversation: conv, message: { id: 9, author: "agent", mine: false, text: "답\n둘째 줄" } },
    { kind: "my_request", at: 250, unseen: true, conversation: conv, message: { id: 6, author: "human", mine: true, request: true, req_state: "rejected", req_note: "범위 밖", text: "QA 서버 올려 주세요" } },
    { kind: "message", at: 240, unseen: false, conversation: conv, message: { id: 4, author: "human", mine: false, request: true, req_state: "accepted", text: "로그 형식 바꿔 주세요" } },
    { kind: "failed", at: 200, unseen: false, conversation: conv, message: { id: 8, author: "human", mine: true, exec: true, text: "요청", state: "error" } },
    { kind: "message", at: 95, unseen: false, conversation: conv, message: { id: 7, author: "agent", mine: false, text: "실행 실패", state: "error" } },
    { kind: "message", at: 90, unseen: true, conversation: null, message: { id: 1, author: "human", text: "고아" } },
  ];
  const rows = activityRows(items, names);
  assert.deepEqual(
    rows.map((r) => [r.label, r.who, r.preview, r.note, r.href, r.unseen]),
    [
      ["받은 답", "이도윤/guide", "답 둘째 줄", "", "#c/5/m/9", true],
      ["내 요청 · 반려", "나", "QA 서버 올려 주세요", "범위 밖", "#c/5/m/6", true],
      ["받은 요청 · 수용", "이도윤", "로그 형식 바꿔 주세요", "", "#c/5/m/4", false],
      ["실패", "나", "요청", "", "#c/5/m/8", false],
      ["받은 답 · 실패", "이도윤/guide", "실행 실패", "", "#c/5/m/7", false],
    ],
  );
  assert.ok(rows.every((r) => r.title === "배포 절차" && !("tag" in r)));
  // 종류 색(#1z-3) — 탭 이름과 같은 키.
  assert.deepEqual(rows.map((r) => r.tone), ["answer", "mine", "request", "failed", "failed"]);
  assert.equal(activityRows([{ kind: "sync_failed", at: 1, conversation: conv, detail: "x" }], names)[0].tone, "failed");
  assert.ok(rows.every((r) => ACT_KINDS.some((k) => k.kind === r.tone)), "색 키 = 거르기 탭 kind");
  assert.equal(new Set(rows.map((r) => r.key)).size, rows.length, "항목 열쇠는 겹치지 않는다");
});

test("activity_rows_tabs — 내 활동 거르기 탭(#1q): 전체 · 받은 요청 · 내 요청 · 받은 답 · 실패, 자동 왕복 없음", () => {
  assert.deepEqual(
    ACT_KINDS.map((k) => [k.kind, k.label]),
    [
      ["all", "전체"],
      ["request", "받은 요청"],
      ["mine", "내 요청"],
      ["answer", "받은 답"],
      ["failed", "실패"],
    ],
  );
  const conv = { id: 5, peer_id: 2, agent: "guide", dir: "out", title: "t" };
  assert.deepEqual(activityRows([{ kind: "relay_end", at: 1, unseen: true, conversation: conv, detail: "상한" }], new Map()), [], "옛 자동 왕복 항목은 그리지 않는다");
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

test("messages_sig_edit — 편집 · 편집 가능이 바뀌면 다시 그린다", () => {
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

test("run_steps — 보냄 → 대기열 → 실행 → 응답, 시각 = 날짜 표시방식 + 초 · 실행 줄 · 받은 요청은 받음 · 보냄", () => {
  const at = 1791691336;
  const clock = (t) => formatTime(t, { seconds: true });
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
  assert.deepEqual(
    groups.map((g) => g.rows.map((r) => r.mark)),
    [
      [{ type: "msg", kind: "out" }],
      [{ type: "conv", kind: "out" }],
      [
        { type: "peer", peer: 2 },
        { type: "peer", peer: 4 },
      ],
      [
        { type: "agent", owner: 2, name: "guide" },
        { type: "agent", owner: "me", name: "review" },
      ],
    ],
    "줄 앞 칩(#1z-8 S1) — 메시지 · 대화 = 대화 유형, 사람 = 그 상대, 에이전트 = 주인",
  );
  assert.deepEqual(groups.flatMap((g) => g.rows.map((r) => r.tag)), ["", "", "", "", "", ""], "열린 대화 = 태그 없음");
  assert.deepEqual(searchGroups({ messages: [], conversations: [], peers: [], agents: [] }, names), []);
});

test("search_groups_closed — 닫힌 대화 태그(#1z-8): 나감 · 상대가 나감 · 끝난 공유 사본(응답에 공유 정보가 없으면 convs 의 같은 대화로)", () => {
  const names = new Map([[2, "이도윤 · DYOON"]]);
  const left = { id: 5, peer_id: 2, agent: "guide", dir: "out", title: "나간 대화", left_at: 10, peer_left_at: null };
  const gone = { id: 6, peer_id: 2, agent: "guide", dir: "in", title: "상대 나감", left_at: null, peer_left_at: 11 };
  const shared = { id: 7, peer_id: 2, agent: "guide", dir: "shared", title: "공유", left_at: null, peer_left_at: null };
  const r = {
    messages: [{ id: 1, author: "human", mine: true, text: "글", conversation: shared }],
    conversations: [left, gone, shared],
    peers: [],
    agents: [],
  };
  const convs = [{ ...shared, share: { ended: true } }];
  const g = searchGroups(r, names, convs);
  assert.deepEqual(g[0].rows.map((x) => [x.tag, x.mark]), [["공유 해제됨", { type: "msg", kind: "shared" }]]);
  assert.deepEqual(g[1].rows.map((x) => x.tag), ["나감", "상대가 나감", "공유 해제됨"]);
  assert.deepEqual(searchGroups(r, names)[1].rows.map((x) => x.tag), ["나감", "상대가 나감", ""], "convs 없으면 응답 그대로");
});

test("ask_marks — 요청 태그(상태) · 상태 바꾸기 작업(받은 요청만) · 내 활동 · 보낸 메시지 라벨", () => {
  const req = (mine, req_state) => ({ author: "human", mine, exec: false, request: true, req_state });
  assert.equal(askTag(req(true, "wait")), "요청 · 대기");
  assert.equal(askTag(req(false, "done")), "요청 · 완료");
  assert.equal(askTag(req(false, undefined)), "요청 · 대기", "상태가 없으면 대기");
  assert.equal(askTag({ author: "human", request: false }), "", "질의는 표시 없음");
  assert.equal(askTag({ author: "agent", request: true }), "", "답에는 없다");
  const acts = (m) => messageActions(m).map((x) => [x.key, x.label]);
  assert.deepEqual(acts(req(false, "wait")).slice(-3), [["req:accepted", "수용"], ["req:rejected", "반려"], ["req:done", "완료"]]);
  assert.deepEqual(acts(req(false, "accepted")).at(-1), ["req:done", "완료"]);
  assert.deepEqual(acts(req(false, "done")).filter(([k]) => k.startsWith("req:")).map(([k]) => k), ["req:wait", "req:accepted", "req:rejected"], "끝난 요청도 되돌린다");
  assert.ok(!acts(req(true, "wait")).some(([k]) => k.startsWith("req:")), "내 요청은 못 바꾼다");
  assert.deepEqual(acts(req(true, "wait")), [], "내 요청 — 다시 보내기 없음(#1k)");
  assert.ok(!acts({ author: "human", mine: false, exec: true, request: false }).some(([k]) => k.startsWith("req:")), "질의");
  const conv = { id: 1, peer_id: 2, agent: "guide", dir: "in", title: "t" };
  const rows = activityRows([{ kind: "message", at: 1, unseen: false, conversation: conv, message: { id: 5, ...req(false, "done"), text: "x", state: "done" } }], new Map());
  assert.equal(rows[0].label, "받은 요청 · 완료", "#1q 내 활동 라벨");
  const sent = inboxRows([{ id: 6, ...req(true, "accepted"), text: "y", created_at: 1, conversation: { ...conv, dir: "out" } }], new Map());
  assert.equal(sent[0].kind, "요청 · 수용");
});

test("req_moves — 받은 요청은 지금 상태 말고 네 상태 어디로든(되돌리기 포함), 이름 = 상태 이름", () => {
  const got = (req_state, mine = false) => reqMoves({ author: "human", mine, request: true, req_state }).map((x) => x.to);
  assert.deepEqual(got("wait"), ["accepted", "rejected", "done"]);
  assert.deepEqual(got("accepted"), ["wait", "rejected", "done"]);
  assert.deepEqual(got("rejected"), ["wait", "accepted", "done"]);
  assert.deepEqual(got("done"), ["wait", "accepted", "rejected"]);
  assert.deepEqual(got(undefined), ["accepted", "rejected", "done"], "상태 없음 = 대기");
  assert.deepEqual(reqMoves({ author: "human", mine: false, request: true, req_state: "done" })[0], { to: "wait", label: "대기" });
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
    ["user-b", "로그", "수용", false, "이번 주", 50, "#c/9/m/3", ["wait", "rejected", "done"]],
  );
});

test("requests_route — #requests 화면 · 사이드바 '요청' 은 그 화면", () => {
  assert.deepEqual(route("#requests"), { view: "requests" });
  assert.equal(sideNav(null).find((n) => n.label === "요청").href, "#requests");
  assert.equal(sideNav(4).find((n) => n.label === "요청").href, "#requests");
});

test("models_tools — 모델 드롭다운 셋 · 도구 안내는 읽기 전용 셋", () => {
  assert.deepEqual(MODELS, ["opus", "sonnet", "haiku"], "an_core run_opts::MODELS 와 같다");
  assert.deepEqual(READ_ONLY_TOOLS, ["Read", "Grep", "Glob"]);
  assert.ok(READ_ONLY_TOOLS.every((t) => TOOLS_NOTE.includes(t)), TOOLS_NOTE);
});

test("shell_confirm — 셸 위험 확인은 끔 → 켬일 때만 · 안내 글은 등록 칸 title 과 같다(#05-3)", () => {
  assert.equal(shellConfirmNeeded(false, true), true, "켤 때");
  assert.equal(shellConfirmNeeded(true, true), false, "켠 채 다른 칸만");
  assert.equal(shellConfirmNeeded(true, false), false, "끌 때");
  assert.equal(shellConfirmNeeded(false, false), false);
  assert.equal(shellConfirmNeeded(undefined, true), true, "옛 응답(칸 없음) = 끔");
  const html = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  assert.ok(html.includes(`<input type="checkbox" id="agent_shell" role="switch" title="${SHELL_NOTE}">`), "등록 칸 title");
  for (const risk of ["workflow", "허용하지 않습니다", "사람 승인 없이", "폴더 · 상위 읽기 제한이 적용되지 않습니다", "신원 키", "~/.claude", "~/.ssh", "막는 장치가 아닙니다", "자동 왕복", "환경 변수", "프록시"]) {
    assert.ok(SHELL_WARN.includes(risk), risk);
  }
});

test("agent_patch — 셸 · 상위 읽기는 바꿨을 때만 보낸다(오래 열린 상세가 다른 창에서 끈 셸을 되살리지 않게, #05-3 R1)", () => {
  const loaded = { shell: true, parent_read: false };
  // 다른 창에서 셸을 껐어도 이 창은 켜진 채 — 추가 지시만 고쳐 저장하면 shell 을 싣지 않는다(서버 값 그대로 = 끔).
  assert.deepEqual(agentPatch(loaded, { folder: "D:\\w", prompt: "x", parent_read: false, shell: true }), { folder: "D:\\w", prompt: "x" });
  assert.deepEqual(agentPatch(loaded, { folder: "f", prompt: "", parent_read: true, shell: false }), { folder: "f", prompt: "", parent_read: true, shell: false });
  assert.deepEqual(agentPatch({}, { folder: "f", prompt: "", parent_read: false, shell: true }), { folder: "f", prompt: "", shell: true }, "옛 응답(칸 없음) = 끔");
});

test("agent_color — 아이콘 색은 바꿨을 때만 보내고, 모르는 값 · 옛 응답은 기본(#1z-7)", () => {
  assert.deepEqual(AGENT_COLORS, ["agent-tile", "peer-1", "peer-2", "peer-3", "peer-4", "peer-5"]);
  assert.equal(agentColor(undefined), "agent-tile");
  assert.equal(agentColor("#ff0000"), "agent-tile");
  assert.equal(agentColor("peer-3"), "peer-3");
  const a = { shell: false, parent_read: false, color: "peer-2" };
  assert.deepEqual(agentPatch(a, { folder: "f", prompt: "", color: "peer-2" }), { folder: "f", prompt: "" });
  assert.deepEqual(agentPatch(a, { folder: "f", prompt: "", color: "peer-5" }), { folder: "f", prompt: "", color: "peer-5" });
  assert.deepEqual(agentPatch({}, { folder: "f", prompt: "", color: "agent-tile" }), { folder: "f", prompt: "" }, "옛 응답 = 기본 색");
});

test("conv_kind_icon — 내 에이전트 응답 = 로봇, 내가 문의 = 사람, 공유 대화 = 눈, 모르는 유형 = 없음", () => {
  assert.equal(convKindIcon({ dir: "in" }), "bot");
  assert.equal(convKindIcon({ dir: "out" }), "user");
  assert.equal(convKindIcon({ dir: "shared" }), "eye");
  assert.equal(convKindIcon({ dir: "x" }), null);
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
    assert.equal(["에이전트 응답", "등록된 에이전트", "북마크"].includes(xs.at(-1).label), true, "하단 = 사용자가 모은 · 등록한 것");
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
    ["req:accepted", "req:rejected", "req:done"],
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

test("star_detail · agent_tag — 즐겨찾기 줄 둘째 줄 '상대 · 에이전트 · 상태', 에이전트 응답 줄 '[이름]'", () => {
  const out = { dir: "out", agent: "review", peer_id: 2, last_state: "done" };
  assert.equal(starDetail(out, "김도윤 · QA-PC"), "김도윤 · review · 완료");
  assert.equal(starDetail({ ...out, dir: "in", agent: "coder", last_state: "running" }, "김도윤 · QA-PC"), "김도윤 · 내 coder · 실행 중");
  assert.equal(starDetail({ ...out, last_state: null }, "김도윤"), "김도윤 · review", "상태 없으면 생략");
  assert.equal(starDetail({ ...out, last_state: null }, undefined), "상대 · review", "모르는 상대");
  const shared = { dir: "shared", agent: "review", peer_id: 2, last_state: "done", share: { other: "박서연" } };
  assert.equal(starDetail(shared, "김도윤 · QA-PC"), "공유 · 김도윤 ↔ 박서연 · 완료");
  assert.equal(agentTag({ agent: "AI Discovery" }), "[AI Discovery]");
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

test("tile_tint — 여섯 색(에이전트 아이콘 색과 같음)이면 tinted + 색 class, 아니면 빈 글자(흑연 · 무채)", () => {
  for (const c of ["agent-tile", "peer-1", "peer-2", "peer-3", "peer-4", "peer-5"]) assert.equal(tileTint(c), `tinted ${c}`);
  for (const c of [undefined, null, "", "rail-2", "red", "peer-6", " peer-1", "tinted"]) assert.equal(tileTint(c), "", JSON.stringify(c));
});

test("me_tint — '나' 얼굴 = 고른 색, 없으면 흑연(me-tile)", () => {
  assert.equal(meTint("peer-2"), "tinted peer-2");
  for (const c of [undefined, null, "", "red"]) assert.equal(meTint(c), "me-tile", JSON.stringify(c));
});

test("own_tint — 내 말풍선 면 = '나' 타일 색, 없거나 틀리면 흑연(rail-2)", () => {
  for (const c of ["agent-tile", "peer-1", "peer-2", "peer-3", "peer-4", "peer-5"]) assert.equal(ownTint(c), c);
  for (const c of [undefined, null, "", "red", "peer-6", "tinted peer-1"]) assert.equal(ownTint(c), "rail-2", JSON.stringify(c));
});

test("own_human — 내가 직접 쓴 글: 사람 · 내 쪽 · CLI 아님 · 공유 대화 아님", () => {
  const out = { dir: "out" };
  const inn = { dir: "in" };
  assert.equal(ownHuman(out, { author: "human", mine: true }), true, "보낸 대화 내 글");
  assert.equal(ownHuman(inn, { author: "human", mine: true }), true, "받은 대화 내 글");
  assert.equal(ownHuman(inn, { author: "human", mine: false }), false, "상대 글");
  assert.equal(ownHuman(inn, { author: "agent", mine: true }), false, "내 에이전트 답");
  assert.equal(ownHuman(out, { author: "human", mine: true, via: "cli" }), false, "CLI 글");
  assert.equal(ownHuman({ dir: "shared" }, { author: "human", owner: true, mine: true }), false, "공유 대화");
  assert.equal(ownHuman(out, { author: "human" }), false, "mine 없음");
});

test("asks_kind — 질의 · 요청 단추는 받은 대화에 없다", () => {
  assert.equal(asksKind({ dir: "in" }), false);
  assert.equal(asksKind({ dir: "out" }), true);
  assert.equal(asksKind({ dir: "out", relay_agent: "a" }), true);
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

test("req_overview_summary — 머리 건수(#1q R1): 서버 집계 그대로 — 목록이 상한으로 잘려도 건수는 범위 전체", () => {
  const conv = { id: 9, peer_id: 2, agent: "review", dir: "in", title: "로그" };
  const m = (id, req_state) => ({ id, author: "human", mine: false, request: true, req_state, created_at: id, text: "t", conversation: conv });
  const counts = { wait: 1, accepted: 0, rejected: 0, done: 501, total: 502 };
  const [d] = reqBoxes(["theirs"], [[m(600, "done")]], [counts], new Map());
  assert.equal(d.rows.length, 1, "행은 받은 목록");
  assert.deepEqual(d.sum, counts, "건수는 서버 집계");
  assert.deepEqual(reqHead("받은 요청", d.sum).states.map((x) => x.text), ["대기 1", "완료 501"]);
});

test("req_overview_head — 절 머리(#1q): 이름 · 전체 건수 · 대기 · 수용 · 완료 · 반려 순, 0건 상태는 뺀다", () => {
  const h = reqHead("받은 요청", { total: 5, wait: 2, accepted: 1, done: 1, rejected: 1 });
  assert.deepEqual(h, {
    title: "받은 요청",
    count: "5",
    states: [
      { key: "wait", text: "대기 2" },
      { key: "accepted", text: "수용 1" },
      { key: "done", text: "완료 1" },
      { key: "rejected", text: "반려 1" },
    ],
  });
  assert.deepEqual(reqHead("보낸 요청", { total: 3, wait: 0, accepted: 0, done: 3, rejected: 0 }).states, [{ key: "done", text: "완료 3" }]);
  assert.deepEqual(reqHead("보낸 요청", { total: 0, wait: 0, accepted: 0, done: 0, rejected: 0 }), { title: "보낸 요청", count: "0", states: [] });
});

test("req_overview_buttons — 글자 단추(#1q): 수용 · 완료 = ✓ 초록(ok), 반려 = ✕ 붉음(no), 대기 = 되돌리기(wait), 상태 바꾸기마다 모양이 있다", () => {
  assert.deepEqual(REQ_BTN.wait, { tone: "wait", icon: "reset" });
  assert.deepEqual(REQ_BTN.accepted, { tone: "ok", icon: "check" });
  assert.deepEqual(REQ_BTN.done, { tone: "ok", icon: "check" });
  assert.deepEqual(REQ_BTN.rejected, { tone: "no", icon: "x" });
  for (const req_state of ["wait", "accepted", "rejected", "done"])
    for (const mv of reqMoves({ author: "human", mine: false, request: true, req_state })) assert.ok(REQ_BTN[mv.to], mv.to);
});

test("peer_msgs_heads — 주고받은 메시지 상대 절 머리(#1q): 레일 순서 · 페어링 풀린 상대는 뒤 · 이름 · 건수 · 마지막 메시지 한 줄 · 시각", () => {
  const names = new Map([[2, "이준호 · QA"], [3, "최유나 · DEV"], [9, "옛 상대 · OLD"]]);
  const last = (id, text, created_at) => ({ id, text, created_at, author: "human", mine: false });
  const heads = peerSummaryRows(
    [
      { peer: 9, count: 3, last: last(90, "옛 글", 10) },
      { peer: 2, count: 48, last: last(20, "QA 서버 올려\n주세요", 300) },
      { peer: 3, count: 31, last: last(30, "그 PR 머지됐어요", 200) },
    ],
    [3, 2],
    names,
  );
  assert.deepEqual(
    heads.map((h) => [h.peer, h.name, h.count, h.last, h.at]),
    [
      [3, "최유나", "31건", "그 PR 머지됐어요", 200],
      [2, "이준호", "48건", "QA 서버 올려 주세요", 300],
      [9, "옛 상대", "3건", "옛 글", 10],
    ],
  );
  assert.deepEqual(peerSummaryRows([], [2], names), []);
});

test("peer_msgs_more — 20건씩(#1q): 보인 수 / 전체 · 더 보기는 남았을 때만", () => {
  assert.equal(PEER_PAGE, 20);
  assert.deepEqual(pageMore(20, 48), { text: "20 / 48", more: true });
  assert.deepEqual(pageMore(48, 48), { text: "48 / 48", more: false });
  assert.deepEqual(pageMore(3, 3), { text: "3 / 3", more: false });
});

test("peer_msgs_saved — 행 북마크 상태(#1q): 메시지 saved 를 행에 그대로", () => {
  const conv = { id: 4, peer_id: 2, agent: "guide", dir: "out", title: "t" };
  const rows = inboxRows(
    [
      { id: 1, author: "human", mine: true, text: "a", created_at: 1, saved: true, conversation: conv },
      { id: 2, author: "agent", mine: false, text: "b", created_at: 2, saved: false, conversation: conv },
    ],
    new Map(),
  );
  assert.deepEqual(rows.map((r) => [r.id, r.saved]), [[1, true], [2, false]]);
});

test("req_overview_filter — 절 데이터(#1q): 쪽 순서 그대로 · 행 = 서버가 상태로 거른 목록 · 요청 아닌 글은 빠짐", () => {
  const conv = { id: 9, peer_id: 2, agent: "review", dir: "in", title: "로그" };
  const m = (id, req_state, mine, request = true) => ({ id, author: "human", mine, request, req_state, created_at: id, text: "t", conversation: conv });
  const zero = { wait: 0, accepted: 0, rejected: 0, done: 0, total: 0 };
  const got = reqBoxes(["theirs", "mine"], [[m(2, "done", false), m(5, null, false, false)], []], [zero, zero], new Map());
  assert.deepEqual(
    got.map((d) => [d.box, d.rows.map((r) => r.id)]),
    [
      ["theirs", [2]],
      ["mine", []],
    ],
  );
});

test("peer_msgs_sig — 펼친 절 다시 그리기(#1q R1): 요청 상태 · 메모 · 같은 길이 편집 · 북마크가 바뀌면 서명이 바뀐다", () => {
  const heads = [{ peer: 2, name: "a", count: "1건", total: 1, last: "x", at: 1 }];
  const conv = { id: 4, peer_id: 2, agent: "guide", dir: "out", title: "t" };
  const base = { id: 7, author: "human", mine: false, text: "abc", saved: false, request: true, req_state: "wait", req_note: null, edited_at: null, state: "done", conversation: conv };
  const sig = (m) => peerMsgsSig(heads, new Map([[2, [{ ...base, ...m }]]]));
  const same = sig({});
  for (const change of [
    { req_state: "accepted" },
    { req_note: "곧" },
    { text: "abd", edited_at: 5 },
    { text: "abd" },
    { saved: true },
    { conversation: { ...conv, title: "u" } },
    { owner: true },
    { author: "agent" },
    { mine: true },
    { via: "cli" },
    { agent: "guide" },
  ])
    assert.notEqual(sig(change), same, JSON.stringify(change));
  assert.equal(sig({}), same);
});

test("peer_msgs_merge — 새로 고침(#1q R2): 첫 페이지 + 그보다 앞의 받은 꼬리, 겹침 없음 · 첫 페이지가 다 안 차면 꼬리 없음", () => {
  const m = (id, created_at) => ({ id, created_at });
  const have = [m(30, 300), m(29, 290), m(28, 280), m(27, 270)]; // 20건 페이지 대신 2건 페이지로
  const fresh = [m(31, 310), m(30, 300)];
  assert.deepEqual(mergePage(fresh, have, 2).map((x) => x.id), [31, 30, 29, 28, 27], "새 글 위 · 받은 꼬리 보존");
  assert.deepEqual(mergePage([m(31, 310)], have, 2).map((x) => x.id), [31], "다 안 찬 첫 페이지 = 전부");
  assert.deepEqual(mergePage([m(30, 300), m(28, 280)], have, 2).map((x) => x.id), [30, 28, 27], "지워진 29 는 첫 페이지 범위 안이라 빠진다");
  assert.deepEqual(mergePage([m(5, 300), m(4, 300)], [m(5, 300), m(4, 300), m(9, 290)], 2).map((x) => x.id), [5, 4, 9], "같은 시각은 id 로");
});

test("ui_polish_menu_icons — 프로필 메뉴(상태 셋 빼고) · 내 활동 탭 · 요청 탭 항목마다 아이콘, 모두 app.css 에 정의(#1r)", () => {
  const css = readFileSync(new URL("./app.css", import.meta.url), "utf8");
  const html = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  const items = [...profileMenu(false, false).filter((i) => !i.sep && !i.state), ...ACT_KINDS];
  assert.deepEqual(items.map((i) => i.icon), ["user", "pencil", "bell", "gear", "bell", "arrow-down", "arrow-up", "chat", "x"]);
  for (const i of items) assert.ok(css.includes(`.ico-${i.icon} {`), `app.css .ico-${i.icon}`);
  for (const [id, icon] of [["req_tab_theirs", "arrow-down"], ["req_tab_mine", "arrow-up"]]) {
    const tab = html.slice(html.indexOf(`id="${id}"`), html.indexOf("</a>", html.indexOf(`id="${id}"`)));
    assert.ok(tab.includes(`ico-${icon}`), `${id} 아이콘`);
  }
});

test("ui_polish_invite_press — 새로 만들기의 '초대 생성' 만 초대 생성 단추를 누른다(#1r — 설정 메뉴는 하위 없음, #1z-8)", () => {
  assert.deepEqual(settingsTree().filter((n) => n.press), []);
  assert.deepEqual(newMenu().filter((i) => i.press).map((i) => [i.label, i.press]), [["초대 생성", "window_btn"]]);
});

test("ui_polish_new_chat — 세션 생성 폼: '자동 왕복' 없음 · 내 응답 에이전트 · 최대 응답 횟수(처음 숨김) · 라벨마다 아이콘(#1r · #1x)", () => {
  const html = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  const form = html.slice(html.indexOf('<form id="chat_form"'), html.indexOf("</form>", html.indexOf('<form id="chat_form"')));
  assert.ok(!form.includes("자동 왕복"), "세션 생성에 자동 왕복 낱말 없음");
  assert.ok(form.includes(">내 응답 에이전트</label>") && form.includes(">최대 응답 횟수</label>"));
  assert.match(form, /<label for="chat_relay_cap" id="chat_relay_cap_label" class="sub-label" hidden>/);
  assert.match(form, /<input id="chat_relay_cap" type="number" min="1" max="50" step="1" value="10" required disabled hidden>/, "숨긴 동안 disabled — 폼 검사에서 빠진다");
  const labels = [...form.matchAll(/<label for="\w+"[^>]*>(.*?)<\/label>/g)].map((m) => m[1]);
  assert.equal(labels.length, 5, "상대 · 상대 에이전트 · 내 응답 에이전트 · 최대 응답 횟수 · 제목(#1x — 메시지 · 메시지 종류 없음)");
  for (const l of labels) assert.match(l, /^<span class="ico ico-[a-z-]+" aria-hidden="true"><\/span>\S/, l);
});

test("loadersFor — 대화 변경은 연결 · 목록 · 본문 · 내 활동 · 건수 등 그 대화를 쓰는 로더를 모두 고른다(설정은 아님)", () => {
  const got = loadersFor([{ kind: "conv", id: 7 }], 7);
  for (const l of ["peers", "groups", "convs", "current", "activity", "requests", "saved", "peer", "runs", "counts"]) assert.ok(got.has(l), l);
  assert.ok(!got.has("settings"));
  assert.deepEqual([...got].sort(), [...CONV_LOADERS].sort());
});

test("loadersFor — 다른 대화만 바뀌면 열린 대화 본문은 다시 읽지 않는다", () => {
  assert.ok(!loadersFor([{ kind: "conv", id: 8 }], 7).has("current"));
  assert.ok(loadersFor([{ kind: "conv", id: 8 }], 7).has("convs"), "목록은 다시 읽는다");
  assert.ok(!loadersFor([{ kind: "conv", id: 8 }], undefined).has("current"), "열린 대화 없음");
});

test("loadersFor — id 를 모르는 대화 변경은 열린 대화도 다시 읽는다", () => {
  assert.ok(loadersFor([{ kind: "conv" }], 7).has("current"));
  assert.ok(loadersFor([{ kind: "conv", id: 8 }, { kind: "conv", id: 7 }], 7).has("current"));
});

test("loadersFor — 빈 목록 · 배열 아님 · 모르는 종류 · 연결(peer)은 전체(null)", () => {
  assert.equal(loadersFor([], 7), null);
  assert.equal(loadersFor(undefined, 7), null);
  assert.equal(loadersFor([{ kind: "board" }], 7), null);
  assert.equal(loadersFor([{ kind: "conv", id: 7 }, { kind: "peer" }], 7), null);
});

test("mergeChanges — 짧은 간격의 신호를 이어 붙인다", () => {
  const a = [{ kind: "conv", id: 1 }];
  const b = [{ kind: "conv", id: 2 }];
  assert.deepEqual(mergeChanges(undefined, a), a);
  assert.deepEqual(mergeChanges(a, b), [...a, ...b]);
});

test("mergeChanges — 하나라도 전체면 전체(빈 목록)", () => {
  const a = [{ kind: "conv", id: 1 }];
  assert.deepEqual(mergeChanges(a, []), []);
  assert.deepEqual(mergeChanges([], a), []);
  assert.deepEqual(mergeChanges(a, undefined), []);
  assert.equal(loadersFor(mergeChanges(a, []), 1), null);
});

test("unionLoaders — 없음이면 그대로, 하나라도 전부(null)면 전부, 아니면 합집합", () => {
  assert.equal(unionLoaders(undefined, null), null);
  assert.deepEqual([...unionLoaders(undefined, new Set(["convs"]))], ["convs"]);
  assert.equal(unionLoaders(new Set(["convs"]), null), null);
  assert.equal(unionLoaders(null, new Set(["convs"])), null);
  assert.deepEqual([...unionLoaders(new Set(["convs"]), new Set(["counts", "convs"]))].sort(), ["convs", "counts"]);
});

test("OneAtATime — 도는 중 온 요청은 겹쳐 돌지 않고 합쳐 끝난 뒤 한 번 더(오래된 조회가 나중 조회를 덮지 않게)", async () => {
  const runs = [];
  let active = 0;
  let most = 0;
  const gates = [];
  const one = new OneAtATime(async (only) => {
    active += 1;
    most = Math.max(most, active);
    runs.push(only === null ? "all" : [...only].sort().join(","));
    await new Promise((r) => gates.push(r));
    active -= 1;
  });
  const first = one.call(new Set(["convs"]));
  const second = one.call(new Set(["counts"])); // 신호
  const third = one.call(new Set(["peers"])); // 또 신호
  assert.equal(runs.length, 1, "도는 중에는 시작하지 않는다");
  gates.shift()();
  await new Promise((r) => setTimeout(r, 0));
  assert.deepEqual(runs, ["convs", "counts,peers"], "쌓인 둘을 합쳐 한 번");
  const fourth = one.call(null); // 5초 전체 새로 고침이 겹침
  gates.shift()();
  await new Promise((r) => setTimeout(r, 0));
  assert.deepEqual(runs, ["convs", "counts,peers", "all"]);
  gates.shift()();
  await Promise.all([first, second, third, fourth]);
  assert.equal(most, 1, "동시에 둘이 돈 적 없음");
  assert.equal(one.busy, null);
});

test("OneAtATime — 앞 묶음이 실패해도 쌓인 요청은 돌고, 약속은 자기 묶음 결과로 풀리며, 그 뒤 호출도 돈다", async () => {
  const runs = [];
  const gates = [];
  const one = new OneAtATime(async (only) => {
    runs.push(only === null ? "all" : [...only].sort().join(","));
    await new Promise((ok, no) => gates.push({ ok, no }));
  });
  const first = one.call(new Set(["convs"]));
  const second = one.call(new Set(["counts"]));
  gates.shift().no(new Error("api 실패"));
  await assert.rejects(first, /api 실패/);
  await new Promise((r) => setTimeout(r, 0));
  assert.deepEqual(runs, ["convs", "counts"], "쌓인 요청이 실행됐다");
  gates.shift().ok();
  await second;
  assert.equal(one.busy, null);
  const third = one.call(null);
  gates.shift().ok();
  await third;
  assert.deepEqual(runs, ["convs", "counts", "all"]);
  assert.equal(one.busy, null);
});

// ---- 요청 보드 · 사이드바 에이전트 대화 · 피드백 말풍선 · 입력창(#1v) ----

const boardMsgs = [
  { id: 1, author: "human", request: true, mine: false, req_state: "wait", text: "로그 형식", created_at: 10, conversation: { id: 5, peer_id: 1, agent: "dev", dir: "in" } },
  { id: 2, author: "human", request: true, mine: false, req_state: "accepted", text: "체크리스트", created_at: 20, conversation: { id: 5, peer_id: 1, agent: "dev", dir: "in" } },
  { id: 3, author: "human", request: true, mine: false, req_state: "done", text: "보고서", created_at: 30, conversation: { id: 6, peer_id: 2, agent: "dev", dir: "in" } },
  { id: 4, author: "human", request: true, mine: false, req_state: "rejected", text: "권한", created_at: 40, conversation: { id: 6, peer_id: 2, agent: "dev", dir: "in" } },
];

test("req_board_columns — 열 = 대기 · 수용 · 반려 · 완료 순, 건수 = 서버 집계, 카드 = 그 상태", () => {
  const rows = requestRows(boardMsgs, splitNames);
  const cols = boardColumns(rows, { total: 9, wait: 5, accepted: 1, done: 2, rejected: 1 });
  assert.deepEqual(
    cols.map((c) => [c.key, c.name, c.count, c.rows.map((r) => r.id)]),
    [
      ["wait", "대기", 5, [1]],
      ["accepted", "수용", 1, [2]],
      ["rejected", "반려", 1, [4]],
      ["done", "완료", 2, [3]],
    ],
    "건수는 목록 길이가 아니라 집계",
  );
  assert.deepEqual(boardColumns([], undefined).map((c) => c.count), [0, 0, 0, 0], "집계 없으면 0");
});

test("req_board_drop — 받은 요청은 다른 세 열 어디로든(끝난 요청도), 같은 열 · 보낸 요청은 놓지 못함", () => {
  const [wait, acc, done, rej] = requestRows(boardMsgs, splitNames);
  const ok = (r) => ["wait", "accepted", "rejected", "done"].filter((to) => canDrop(r, to));
  assert.deepEqual(ok(wait), ["accepted", "rejected", "done"]);
  assert.deepEqual(ok(acc), ["wait", "rejected", "done"]);
  assert.deepEqual(ok(done), ["wait", "accepted", "rejected"]);
  assert.deepEqual(ok(rej), ["wait", "accepted", "done"]);
  const [mine] = requestRows([{ ...boardMsgs[0], mine: true }], splitNames);
  assert.deepEqual(ok(mine), [], "보낸 요청 보드는 읽기 전용");
});

test("conv_list_mine — 나 · 상대 타일 = 에이전트 대화 없음 · 내 대화 머리 없음 · 공유 대화 머리, 그룹 타일은 지금대로", () => {
  const rows = convRows([...splitConvs, shared()], splitNames, null);
  const show = (parts) => parts.map((g) => [g.dir, g.head, g.rows.map((r) => r.c.id)]);
  assert.deepEqual(show(convListParts(rows, null)), [
    ["out", false, [1, 3, 5]],
    ["shared", true, [9]],
  ]);
  assert.deepEqual(show(convListParts(rows.filter((r) => r.c.dir !== "shared"), null)), [["out", false, [1, 3, 5]]], "공유 없으면 머리 없는 한 목록");
  assert.deepEqual(show(convListParts(rows, { id: 1 })), [
    ["out", true, [1, 3, 5]],
    ["in", true, [2, 4]],
    ["shared", true, [9]],
  ]);
  assert.deepEqual(show(convListParts(rows.filter((r) => r.c.dir === "in"), { id: 1 })), [["in", false, [2, 4]]], "그룹 — 묶음 하나면 머리 없음");
  assert.deepEqual(convListParts(rows.filter((r) => r.c.dir === "in"), null), [], "에이전트 대화뿐이면 빈 목록");
});

test("conv_list_mine_sections — 에이전트 대화뿐인 상대 절 · 검색 결과는 빈 안내로(적대적 평가 R1): 절 · 빈 판정은 보일 줄로", () => {
  const inOnly = splitConvs.filter((c) => c.dir === "in");
  const visible = visibleConvRows(convRows(inOnly, splitNames, null), null);
  assert.deepEqual(visible, [], "나 · 상대 타일 — 에이전트 대화뿐이면 보일 줄 없음");
  assert.deepEqual(convSections(visible, [1, 2], splitNames).map((s) => [s.key, s.rows.length]), [["convs", 0]], "'나' — 빈 상대 절 대신 빈 '대화' 절 하나(빈 안내)");
  const me = convSections(visibleConvRows(convRows(splitConvs, splitNames, null), null), [1, 2], splitNames);
  assert.deepEqual(me.map((s) => [s.key, s.rows.map((r) => r.c.id)]), [["peer:1", [1, 5]], ["peer:2", [3]]], "상대별 절은 보일 줄만");
  const found = visibleConvRows(convRows(splitConvs, splitNames, null, "로그"), null);
  assert.deepEqual(found, [], "검색어가 에이전트 대화에만 맞으면 결과 없음 → '맞는 대화 없음'");
  assert.equal(visibleConvRows(convRows(inOnly, splitNames, null), { id: 1 }).length, 2, "그룹 타일은 그대로");
});

test("side_nav_agents — 에이전트 머리(링크 없음) + 하위(등록된 에이전트 · '나' 에만 에이전트 응답 = 안 읽음 수), 그룹 타일엔 없음(#1z-2)", () => {
  const tail = (tile) => {
    const xs = sideNav(tile, 0, 0, 0, 4);
    return xs.slice(xs.findIndex((n) => n.head)).map((n) => [n.label, n.href ?? null, !!n.sub, n.count ?? 0]);
  };
  assert.deepEqual(tail(null), [
    ["에이전트", null, false, 0],
    ["등록된 에이전트", "#agents", true, 0],
    ["에이전트 응답", "#agents/convs", true, 4],
  ]);
  assert.deepEqual(tail(3), [
    ["에이전트", null, false, 0],
    ["등록된 에이전트", "#agents", true, 0],
  ], "상대 타일 — 에이전트 응답 없음");
  assert.ok(!sideNav("group", 0, 0, 0, 4).some((n) => n.head || n.sub), "그룹 타일엔 없음");
  assert.ok(![null, 3, "group"].some((tile) => sideNav(tile, 0, 0, 0, 4).some((n) => n.label === "에이전트 대화")), "옛 이름 없음");
  assert.deepEqual(route("#agents/convs"), { view: "agentConvs" });
  assert.deepEqual(route("#agents"), { view: "agents" });
  assert.ok(groupLeaves("agentConvs"), "그룹 타일에서 열면 '나' 로");
  assert.ok(peerLeaves("agentConvs"), "상대 타일에서 열면 '나' 로");
  assert.ok(!peerLeaves("agents") && !peerLeaves("home") && !peerLeaves("requests"), "등록된 에이전트 · 개요 · 요청은 상대 타일에 그대로");
});

test("agent_convs_sections — 상대가 연 대화(in)만, 요청한 상대별 절(레일 순서), 상대 타일 = 그 상대만", () => {
  const all = [...splitConvs, shared()];
  const show = (xs) => xs.map((s) => [s.key, s.title, s.rows.map((r) => r.c.id)]);
  assert.deepEqual(show(agentConvSections(all, [2, 1], splitNames)), [
    ["agentconvs:2", "user-c", [4]],
    ["agentconvs:1", "user-b", [2]],
  ]);
  assert.deepEqual(show(agentConvSections(all.filter((c) => c.peer_id === 1), [1], splitNames)), [["agentconvs:1", "user-b", [2]]]);
  assert.deepEqual(agentConvSections(splitConvs.filter((c) => c.dir === "out"), [1, 2], splitNames), [], "없으면 빈 목록");
});

test("req_feedback — 대기 · 질의 · 답은 없음, 수용 · 반려 · 완료면 상태 · 메모 · 시각, 방향 = 요청 말풍선 반대", () => {
  const req = (over) => ({ id: 1, author: "human", request: true, mine: true, created_at: 10, ...over });
  assert.equal(reqFeedback(req({ req_state: "wait" })), null);
  assert.equal(reqFeedback(req({})), null, "상태 없음 = 대기");
  assert.equal(reqFeedback({ author: "human", request: false, req_state: "done" }), null, "질의");
  assert.equal(reqFeedback({ author: "agent", request: true, req_state: "done" }), null, "답");
  assert.deepEqual(
    reqFeedback(req({ req_state: "accepted", req_note: "오늘 중", req_at: 50 })),
    { key: "accepted", state: "수용", note: "오늘 중", at: 50, side: "theirs" },
    "내 요청 → 받은 쪽(왼쪽)",
  );
  assert.deepEqual(reqFeedback(req({ mine: false, req_state: "rejected" })), { key: "rejected", state: "반려", note: "", at: 10, side: "mine" }, "받은 요청 → 내 쪽(오른쪽)");
});

test("composer_a — 입력창: 질의 / 요청 라디오 2개 · 안내 문구 없음 · placeholder '메시지 입력' · 창 폭에 맞춰 늘어남(최대 폭 없음, 좌우 = 말풍선 영역 max(48px, 8%) — #1z), 편집 모드 없음(#1v)", () => {
  const html = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  const css = readFileSync(new URL("./app.css", import.meta.url), "utf8");
  const form = html.split('<form id="reply_form"')[1].split("</form>")[0];
  assert.equal(form.match(/name="ask"/g)?.length, 2);
  assert.ok(!html.includes("mention_hint") && !html.includes("에이전트 부르기"), "안내 문구 없음");
  assert.ok(form.includes('id="reply_text" rows="1" placeholder="메시지 입력"'));
  const composer = css.split(".composer {")[1].split("}")[0];
  assert.doesNotMatch(composer, /max-width/, "입력창은 창 폭에 맞춰 늘어난다");
  // #1z — 말풍선 ↔ 입력창 24px 는 스크롤 밖(입력창 위 여백), 말풍선 영역 아래 패딩 없음, 좌우 같은 여백.
  assert.match(composer, /margin: var\(--sp-5\) var\(--chat-gutter\) var\(--sp-4\);/);
  assert.match(css, /#messages \{ gap: var\(--sp-4\); padding: var\(--sp-5\) var\(--chat-gutter\) 0; \}/);
  assert.ok(css.includes("--chat-gutter: max(48px, 8%);"), "좌우 여백 토큰");
  assert.match(css, /\.composer textarea \{[^}]*min-height: 44px; max-height: 200px;/);
  for (const id of ['id="edit_btn"', "edit_bar", "scope_dialog"]) assert.ok(!html.includes(id), id);
  const app = source(false);
  assert.ok(!app.includes('$("reply_text").placeholder'), "대화를 열 때 placeholder 를 덮어쓰지 않는다(적대적 평가 R2)");
});

test("conv_menu_items_1x — 대화 ⋯ 끝: 기록 삭제 ▸(내 글만 · 모든 사용자) · 나가기(확인 = 목록에서 삭제), 닫힌 대화는 내 글만, 관람자는 나가기만", () => {
  const leave = { key: "leave", label: "나가기", icon: "exit", danger: true, confirm: "대화방이 목록에서 삭제됩니다." };
  assert.equal(LEAVE_CONFIRM, "대화방이 목록에서 삭제됩니다.");
  assert.deepEqual(convMenuItems({ dir: "out" }), [
    { key: "clear", label: "기록 삭제", icon: "trash", sub: [{ key: "clear_mine", label: "내 글만 삭제" }, { key: "clear_all", label: "모든 사용자 글 삭제" }] },
    leave,
  ]);
  const subs = (c) => convMenuItems(c)[0].sub?.map((x) => x.key);
  assert.deepEqual(subs({ dir: "in", left_at: null, peer_left_at: null }), ["clear_mine", "clear_all"], "에이전트 대화도 같다");
  assert.deepEqual(subs({ dir: "in", peer_left_at: 5 }), ["clear_mine"], "상대가 나간 대화 — 내 글만");
  assert.deepEqual(subs({ dir: "out", left_at: 5 }), ["clear_mine"], "1.0.x 에서 내가 나간 대화 — 내 글만");
  assert.deepEqual(convMenuItems({ dir: "out", left_at: 5 }).map((i) => i.key), ["clear", "leave"]);
  assert.deepEqual(convMenuItems({ dir: "shared", share: { ended: null } }), [leave], "관람자 — 나가기만");
  assert.deepEqual(convMenuItems({ dir: "shared", share: { ended: 9 } }), [leave], "해제된 사본도 나가기로 목록에서 삭제");
  const all = [{ dir: "out" }, { dir: "in", peer_left_at: 1 }, { dir: "shared", share: {} }].flatMap((c) => convMenuItems(c).flatMap((i) => [i.label, ...(i.sub ?? []).map((x) => x.label)]));
  for (const gone of ["삭제", "이력유지", "이력삭제", "대화 유지", "대화 제거", "대화방 종료"]) assert.ok(!all.includes(gone), gone);
  const css = readFileSync(new URL("./app.css", import.meta.url), "utf8");
  for (const i of convMenuItems({ dir: "out" })) assert.ok(css.includes(`.ico-${i.icon} {`), `app.css .ico-${i.icon}`);
});

test("side_sub — 사이드바 머리 부제: 나 = 이름(별칭이면 호스트), 상대 = 호스트(별칭이면 이름 · 호스트), 지문 · 페어링 됨 · 접속 문구 없음(#1x)", () => {
  const me = { name: "tskim", host: "DEV-01", fingerprint: "06A7 9531 1111" };
  assert.equal(sideSub("me", me, ""), "tskim");
  assert.equal(sideSub("me", me, "나의 PC"), "tskim · DEV-01");
  const peer = { name: "kim", host: "DEV-02", paired: true, seen_at: Math.floor(Date.now() / 1000) };
  assert.equal(sideSub("peer", peer), "DEV-02");
  assert.equal(sideSub("peer", { ...peer, alias: "팀원 A" }), "kim · DEV-02");
  for (const t of [sideSub("me", me, "x"), sideSub("peer", peer), sideSub("peer", { ...peer, alias: "a" })])
    for (const bad of ["지문", "06A7", "페어링 됨", "방금 통신", "통신"]) assert.ok(!t.includes(bad), `${t} ∌ ${bad}`);
});

test("compose_defaults — 세션 생성 상대 칸 처음 값 = 레일에서 고른 페어링된 상대(#1x), 나 · 그룹 · 미페어링은 빈 값", () => {
  const peers = [{ id: 3, paired: true }, { id: 4, paired: false }];
  assert.equal(composeDefaults(3, peers), "3");
  assert.equal(composeDefaults(4, peers), "", "페어링 안 됨");
  assert.equal(composeDefaults(null, peers), "", "나");
  assert.equal(composeDefaults(9, peers), "", "없는 상대");
});

test("request_notice — 받은 요청은 한 tag(앞 알림을 바꿈) · 요청 보드 · 효과음, 그 밖은 종류별 tag · 내 활동 · 소리 없음(#1x B안)", () => {
  assert.deepEqual(requestNotice({ kind: "request", title: "받은 요청 3", body: "팀원 A · 문구\n외 2건" }), {
    title: "받은 요청 3",
    body: "팀원 A · 문구\n외 2건",
    tag: "coworkagent-request",
    href: "#requests",
    sound: true,
  });
  const m = requestNotice({ kind: "message", title: "메시지 — review", body: "x" });
  assert.deepEqual([m.tag, m.href, m.sound], ["coworkagent-message", "#activity", false]);
});

test("session_label — '새 대화' 라벨이 화면 원문에 없고 '세션 생성' 이다(#1x — 사용자 2026-10-20)", () => {
  assert.equal(SESSION_NEW, "세션 생성");
  assert.equal(newMenu()[0].label, SESSION_NEW);
  for (const [f, src] of [["./index.html", readFileSync(new URL("./index.html", import.meta.url), "utf8")], ["js", source(false)], ["js/logic", source(true)]]) {
    const hits = [...src.matchAll(/["'`>][^"'`<\n]*새 대화/g)].map((m) => m[0]);
    assert.deepEqual(hits, [], f);
  }
  const html = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  assert.ok(html.includes("<h1>세션 생성</h1>"));
  const form = html.slice(html.indexOf('<form id="chat_form"'), html.indexOf("</form>", html.indexOf('<form id="chat_form"')));
  assert.ok(!form.includes("메시지 종류") && !form.includes("chat_text"), "세션 생성에 메시지 · 메시지 종류 없음(대화방 입력창의 질의 | 요청은 그대로)");
  assert.ok(!html.includes("실행하지 않고 상대의 요청 목록에 올립니다"), "요청 안내 없음");
  assert.match(html, /<form id="chat_form"[\s\S]*?<button type="submit" class="primary">생성<\/button>/);
  assert.match(html, /<html lang="ko" data-theme="light">/, "기본 테마 라이트");
});

test("conv_closed — 닫힌 대화 문구(나감 · 상대가 나감 · 공유 해제됨) = 대화 줄 태그, 열린 대화는 빈 글자", () => {
  assert.equal(convClosed({ dir: "out", left_at: 3 }), "나감");
  assert.equal(convClosed({ dir: "out", left_at: 3, peer_left_at: 4 }), "나감", "내가 나갔으면 나감");
  assert.equal(convClosed({ dir: "in", peer_left_at: 4 }), "상대가 나감");
  assert.equal(convClosed({ dir: "shared", share: { ended: 1 } }), "공유 해제됨");
  assert.equal(convClosed({ dir: "shared", share: { ended: null } }), "");
  assert.equal(convClosed({ dir: "out", left_at: null, peer_left_at: null }), "");
  assert.equal(convClosed(null), "");
  assert.equal(convMeta({ dir: "in", agent: "a", peer_left_at: 4 }).tag, "상대가 나감");
  assert.equal(convMeta({ dir: "out", agent: "a" }).tag, null);
});

test("refused_left — 상대가 나간 대화로 보낸 글의 거절 사유", () => {
  assert.equal(refusedText({ state: "error", refused: "left" }), "상대가 대화를 나갔습니다");
  assert.equal(refusedText({ state: "sent", refused: "left" }), "", "실패 상태일 때만");
});

test("req_moves_closed — 닫힌 대화의 받은 요청은 읽기 전용(상태 바꾸기 · 다시 알리기 · 보드 끌기 없음)", () => {
  const m = { author: "human", mine: false, request: true, req_state: "wait", state: "done" };
  assert.deepEqual(reqMoves(m, true), []);
  assert.equal(reqMoves(m).length, 3, "열린 대화는 그대로");
  assert.equal(reqResend({ ...m, refused: "unreachable" }, true), false);
  assert.deepEqual(messageActions({ ...m, refused: "unreachable" }, true), []);
  const conv = { id: 4, peer_id: 2, agent: "review", dir: "in", title: "t", peer_left_at: 7 };
  const [row] = requestRows([{ ...m, id: 1, text: "요청", created_at: 1, conversation: conv }], new Map());
  assert.deepEqual([row.moves, row.resend, canDrop(row, "accepted")], [[], false, false]);
  const [open] = requestRows([{ ...m, id: 1, text: "요청", created_at: 1, conversation: { ...conv, peer_left_at: null } }], new Map());
  assert.equal(canDrop(open, "accepted"), true);
});

test("screen_settings_1x — 설정 '타일 크기'(크게 · 작게 — #1z 표시 이름) · 알림 '효과음' 칸, 작은 타일 CSS(넓은 화면 · 레일 폭 토큰)", () => {
  const html = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  const sel = html.slice(html.indexOf('<select id="ui_tile_size">'), html.indexOf("</select>", html.indexOf('<select id="ui_tile_size">')));
  assert.deepEqual([...sel.matchAll(/<option value="(\w+)">([^<]*)</g)].map((m) => [m[1], m[2]]), [["normal", "크게"], ["small", "작게 — 작은 아이콘 + 이름 한 줄"]]);
  assert.ok(html.includes('<label for="ui_tile_size"><span class="ico ico-resize lead" aria-hidden="true"></span>타일 크기</label>'), "#1z-10 라벨 앞 아이콘");
  assert.match(html, /<label for="notify_sound"><span class="ico ico-volume lead" aria-hidden="true"><\/span>효과음<\/label>.*?<input type="checkbox" id="notify_sound" role="switch">/);
  const css = readFileSync(new URL("./app.css", import.meta.url), "utf8");
  assert.ok(css.includes("--rail-w-small:"), "레일 폭 토큰");
  assert.match(css, /@media \(min-width: 721px\) \{\s*#app\.tile-small \{ --rail-w: var\(--rail-w-small\); \}/);
  assert.match(css, /grid-template-columns: var\(--rail-w\) auto auto minmax\(0, 1fr\) auto;/, "레일 열 = --rail-w(#1z 끌기)");
});

test("peer_weeks_options — 주고받은 메시지 기간 1 ~ 12주(#1y, 서버 PEER_WEEKS_MAX 와 같다)", () => {
  assert.deepEqual(PEER_WEEKS, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
  const o = peerWeeksOptions();
  assert.equal(o.length, 12);
  assert.deepEqual([o[0], o[3], o[11]], [{ value: "1", label: "1주" }, { value: "4", label: "4주" }, { value: "12", label: "12주" }]);
});

test("req_head_archive — archive 된 완료는 전체 수 밖 · 머리 끝에 따로(#1y)", () => {
  const h = reqHead("받은 요청", { total: 2, wait: 1, accepted: 0, done: 1, rejected: 0, archived: 3 });
  assert.equal(h.count, "2");
  assert.deepEqual(h.states.map((x) => x.text), ["대기 1", "완료 1", "archive 3"]);
  assert.deepEqual(reqHead("받은 요청", { total: 1, wait: 1, archived: 0 }).states.map((x) => x.key), ["wait"], "0 이면 없음");
});

test("req_archive_action — 완료 = archive · archive 됨 = 해제 · 그 밖(대기 · 질의 · 답) 없음, 받은 · 보낸 요청 모두(#1y)", () => {
  const req = (o) => ({ author: "human", request: true, mine: false, ...o });
  assert.deepEqual(reqArchiveAction(req({ req_state: "done" })), { key: "archive", label: "archive" });
  assert.deepEqual(reqArchiveAction(req({ req_state: "done", mine: true })), { key: "archive", label: "archive" });
  assert.deepEqual(reqArchiveAction(req({ req_state: "done", archived: true })), { key: "unarchive", label: "archive 해제" });
  for (const m of [req({ req_state: "wait" }), req({ req_state: "rejected" }), { author: "human", request: false }, { author: "agent", request: false }])
    assert.equal(reqArchiveAction(m), null, JSON.stringify(m));
  const conv = { id: 9, peer_id: 2, agent: "review", dir: "in" };
  const [r] = requestRows([{ id: 3, author: "human", mine: false, request: true, req_state: "done", created_at: 1, text: "x", conversation: conv }], new Map());
  assert.equal(r.archive.key, "archive", "요청 행에 붙는다");
});

test("merge_page_since — 받아 둔 꼬리도 보기 기간 안만(#1y 적대적 평가 R1 — 시간이 지나 기간 밖이 된 글이 남지 않게)", () => {
  const m = (id, at) => ({ id, created_at: at });
  const fresh = [m(30, 300), m(29, 290)];
  const have = [m(30, 300), m(29, 290), m(28, 280), m(27, 100)];
  assert.deepEqual(mergePage(fresh, have, 2, 200).map((x) => x.id), [30, 29, 28], "기간 밖 27 은 빠진다");
  assert.deepEqual(mergePage(fresh, have, 2).map((x) => x.id), [30, 29, 28, 27], "기간 없으면 그대로");
});

test("speaker_icon — 발화자 종류(#1z): 에이전트 = 로봇, 그 밖 = 사람", () => {
  assert.deepEqual(speakerIcon("agent"), { kind: "agent", icon: "bot", label: "에이전트" });
  for (const a of ["human", undefined, ""]) assert.deepEqual(speakerIcon(a), { kind: "human", icon: "user", label: "사람" }, String(a));
});

test("inbox_rows_author — 메시지 줄에 발화자(#1z): 주고받은 메시지 · 내 활동", () => {
  const names = new Map([[2, "이준호 · QA"]]);
  const conv = { id: 4, peer_id: 2, agent: "guide", dir: "out" };
  const rows = inboxRows(
    [
      { id: 1, author: "agent", mine: false, text: "답", created_at: 2, conversation: conv },
      { id: 2, author: "human", mine: true, text: "질의", created_at: 1, conversation: conv },
    ],
    names,
  );
  assert.deepEqual(rows.map((r) => r.author), ["agent", "human"]);
  assert.deepEqual(rows.map((r) => r.mine), [false, true], "주고받은 메시지 아바타(#1z-2) — 내 사람 글 구분");
  const act = activityRows([{ kind: "message", at: 1, conversation: conv, message: { id: 1, author: "agent", text: "답" } }], names);
  assert.equal(act[0].author, "agent");
});

test("rail_size — 레일 끌기 경계(#1z): 120px 미만 크게 · 이상 작게, 놓는 폭 72 · 168", () => {
  assert.deepEqual([64, 119].map(railSize), ["normal", "normal"]);
  assert.deepEqual([120, 121, 240].map(railSize), ["small", "small", "small"]);
  assert.deepEqual([RAIL_W.normal, RAIL_W.small, RAIL_W.split, RAIL_W.min, RAIL_W.max], [72, 168, 120, 64, 240]);
});

test("note_count — 의견 글자 수(#1z): 문자 수 / 200, 줄바꿈도 한 글자", () => {
  assert.equal(noteCount(""), "0 / 200");
  assert.equal(noteCount("가\n나"), "3 / 200");
  assert.equal(noteCount("가".repeat(200)), "200 / 200");
});

test("group_settings_list — 그룹설정 목록(#1z 시안 A): 대화 수 · 검색 = 그룹 이름 · 사용자(이름 · PC · 지문) · 대화 제목", () => {
  const peers = [
    { id: 1, name: "jiwon", alias: "이지원", host: "DEV-01", fingerprint: "AB12" },
    { id: 2, name: "seojun", alias: "", host: "QA-PC", fingerprint: "CD34" },
  ];
  const convs = [
    { id: 10, peer_id: 1, agent: "review", dir: "out", title: "배포 노트" },
    { id: 11, peer_id: 2, agent: "qa", dir: "out", title: "로그인 오류" },
    { id: 12, peer_id: 2, agent: "qa", dir: "in", title: "" },
  ];
  const groups = [
    { id: 7, name: "배포 준비", convs: [10], color: "peer-4" },
    { id: 8, name: "QA", convs: [11, 12] },
    { id: 9, name: "빈 그룹", convs: [] },
  ];
  const names = new Map([[1, "이지원 · DEV-01"], [2, "seojun · QA-PC"]]);
  const all = groupSettingsList(groups, peers, convs, "", names);
  assert.deepEqual(all.groups.map((g) => [g.id, g.count]), [[7, 1], [8, 2], [9, 0]]);
  assert.deepEqual(all.users, [{ id: 1, count: 1 }, { id: 2, count: 2 }]);
  const ids = (q) => {
    const l = groupSettingsList(groups, peers, convs, q, names);
    return [l.groups.map((g) => g.id), l.users.map((u) => u.id)];
  };
  assert.deepEqual(ids("빈"), [[9], []], "그룹 이름");
  assert.deepEqual(ids("qa-pc"), [[8], [2]], "PC(대소문자 무시) — 그 사용자 대화가 든 그룹도");
  assert.deepEqual(ids("ab12"), [[7], [1]], "지문");
  assert.deepEqual(ids("로그인"), [[8], [2]], "대화 제목");
  assert.deepEqual(ids("없는말"), [[], []]);
  // 상세에 보일 대화(적대적 리뷰 R1): 빈 검색 · 그룹 이름이 맞으면 전부, 대화 · 사용자로 맞으면 맞는 대화만.
  const shown = (q) => groupSettingsList(groups, peers, convs, q, names).groups.map((g) => [g.id, g.convs]);
  assert.deepEqual(shown(""), [[7, [10]], [8, [11, 12]], [9, []]]);
  assert.deepEqual(shown("배포 준비"), [[7, [10]]], "그룹 이름만 맞아도 그 그룹 대화 전부");
  assert.deepEqual(shown("qa"), [[8, [11, 12]]], "그룹 이름 'QA' — 전부(사용자 'qa-pc' 와 겹쳐도)");
  assert.deepEqual(shown("로그인"), [[8, [11]]], "대화 제목 — 맞는 대화만");
  assert.deepEqual(shown("ab12"), [[7, [10]]], "지문 — 그 사용자 대화만");
  // 타일 색은 목록 줄 머리글자 네모에 쓴다(적대적 리뷰 #1z-15 R1-1) — 거른 뒤에도 그대로.
  for (const q of ["", "배포"]) assert.equal(groupSettingsList(groups, peers, convs, q, names).groups[0].color, "peer-4", q);
});

test("past_slop — 레일 타일 끌기 시작(#1z): 5px 미만 = 누름, 5px 이상 = 끌기", () => {
  assert.equal(DRAG_SLOP, 5);
  assert.equal(pastSlop(0, 0), false);
  assert.equal(pastSlop(4, 0), false);
  assert.equal(pastSlop(3, 3), false, "대각 4.24px");
  assert.equal(pastSlop(5, 0), true);
  assert.equal(pastSlop(0, -5), true);
  assert.equal(pastSlop(3, 4), true, "대각 정확히 5px");
  assert.equal(pastSlop(6, 0), true);
});

// ---- #1z-2 화면 다듬기 ----

test("rail_slot — 레일 타일 놓을 자리(#1z-2): 가장 가까운 타일 · 가운데 넘으면 뒤, 틈 · 양 끝 밖도", () => {
  const c = [100, 172, 244]; // 타일 가운데(보통 크기 72px 간격)
  assert.equal(railSlot([], 50), null, "타일 없음");
  assert.deepEqual(railSlot(c, 100), { index: 0, after: false }, "가운데 정확히 = 앞");
  assert.deepEqual(railSlot(c, 101), { index: 0, after: true }, "가운데 1px 지나면 뒤");
  assert.deepEqual(railSlot(c, 99), { index: 0, after: false });
  assert.deepEqual(railSlot(c, 132), { index: 0, after: true }, "타일 아래 이름 · 틈(첫 타일 쪽)");
  assert.deepEqual(railSlot(c, 140), { index: 1, after: false }, "틈(둘째 타일 쪽)");
  assert.deepEqual(railSlot(c, 136), { index: 0, after: true }, "정확히 한가운데 틈 = 앞 타일(먼저 찾은 쪽)");
  assert.deepEqual(railSlot(c, 10), { index: 0, after: false }, "첫 타일 위 밖 = 맨 앞");
  assert.deepEqual(railSlot(c, 900), { index: 2, after: true }, "마지막 타일 아래 밖 = 맨 뒤");
});

test("tile_unread — 상대 타일 배지(#1z-2): 에이전트 대화(in) 안 읽음은 빼고 센다", () => {
  const convs = [
    { peer_id: 1, dir: "out", unread: 2 },
    { peer_id: 1, dir: "in", unread: 5 },
    { peer_id: 2, dir: "in", unread: 3 },
    { peer_id: 2, dir: "shared", unread: 1 },
  ];
  assert.deepEqual([...tileUnread(convs)], [[1, 2], [2, 1]]);
  assert.deepEqual([...unreadByPeer(convs)], [[1, 7], [2, 4]], "전체 합은 그대로");
});

test("avatar_kind — 주고받은 메시지 아바타(#1z-2): 에이전트 · 나 · 상대", () => {
  assert.equal(avatarKind("agent", true), "agent");
  assert.equal(avatarKind("agent", false), "agent");
  assert.equal(avatarKind("human", true), "me");
  assert.equal(avatarKind("human", false), "peer");
});

test("debug_text — 디버깅(#1z-8): 파일 크기 글자 · 로그 줄 나누기(수준 거르기) · 진단 글(신원 키 · 토큰 · 지문 없음)", () => {
  assert.deepEqual([null, 0, 1023, 1024, 1536, 12.4 * 1024 * 1024].map(fileSize), ["없음", "0 B", "1023 B", "1.0 KB", "1.5 KB", "12.4 MB"]);
  const lines = ["1792714855 INFO coworkAgent: 시작 — 127.0.0.1:55612", "1792714856 WARN 상대 응답 없음", "그 밖 줄"];
  assert.deepEqual(logLines(lines), [
    { at: 1792714855, level: "INFO", rest: "coworkAgent: 시작 — 127.0.0.1:55612" },
    { at: 1792714856, level: "WARN", rest: "상대 응답 없음" },
    { at: null, level: null, rest: "그 밖 줄" },
  ]);
  assert.deepEqual(logLines(lines, "WARN").map((x) => x.rest), ["상대 응답 없음"]);
  const info = {
    version: "1.3.4",
    os: "windows",
    arch: "x86_64",
    data_dir: "C:\\Users\\u\\AppData\\Local\\coworkAgent",
    files: [
      { key: "db", path: "C:\\d\\coworkAgent.db", size: 2048 },
      { key: "log", path: "C:\\d\\logs\\coworkAgent.log", size: null },
    ],
  };
  const me = { port: 47100, peer_port_fixed: true, ui_port: 55612, open_connections: 3, fingerprint: "06A7 9531", name: "u", host: "PC" };
  const text = diagText(info, me);
  assert.deepEqual(text.split("\n"), [
    "coworkAgent 1.3.4 (windows x86_64)",
    "데이터 폴더: C:\\Users\\u\\AppData\\Local\\coworkAgent",
    "DB: C:\\d\\coworkAgent.db (2.0 KB)",
    "로그: C:\\d\\logs\\coworkAgent.log (없음)",
    "웹UI 포트: 55612",
    "피어 포트: 47100(고정)",
    "열린 피어 연결: 3",
  ]);
  assert.ok(!text.includes("06A7") && !/token|identity|지문/i.test(text), "지문 · 토큰 · 신원 키 없음");
  assert.equal(diagText(info, null).split("\n").length, 4, "내 정보가 없으면 포트 줄 없음");
});

test("draft_owner — 저장 전 값(#1z-8 R1): 주인이 다른 같은 id(지운 뒤 다시 씀)에는 되살리지 않고 버림, 목록에 없는 id 는 버림", () => {
  const drafts = new Map([[3, { owner: "coder", folder: "C:/old", parent_read: true }]]);
  assert.equal(draftOf(drafts, 3, "coder").folder, "C:/old");
  assert.equal(draftOf(drafts, 3, "writer"), undefined);
  assert.equal(drafts.has(3), false);
  drafts.set(1, { owner: "a" }).set(2, { owner: "b" });
  pruneDrafts(drafts, [2]);
  assert.deepEqual([...drafts.keys()], [2]);
});

test("agent_engine — 실행 도구 라벨 · 좌우 옵션 블록 · Git 건너뜀 저장 · 도구 안내(#1z-9)", () => {
  assert.deepEqual(ENGINES, ["claude", "codex"]);
  assert.equal(engineLabel("codex"), "Codex");
  assert.equal(engineLabel("claude"), "Claude Code");
  assert.equal(engineLabel(undefined), "Claude Code", "옛 응답 = Claude Code");
  assert.deepEqual(agentEngineBlocks("codex"), {
    claude: { disabled: true, note: "Codex 에이전트는 쓰지 않음" },
    codex: { disabled: false, note: "" },
  });
  assert.deepEqual(agentEngineBlocks("claude"), {
    claude: { disabled: false, note: "" },
    codex: { disabled: true, note: "Claude Code 에이전트는 쓰지 않음" },
  });
  assert.ok(agentEngineBlocks(null).claude.disabled && agentEngineBlocks(null).codex.disabled, "고르기 전 = 둘 다 비활성");
  const cx = { folder: "f", prompt: "", parent_read: false, shell: false, engine: "codex", skip_git: true };
  assert.deepEqual(agentPatch(cx, { folder: "f", prompt: "", skip_git: true }), { folder: "f", prompt: "" }, "그대로면 안 보냄");
  assert.deepEqual(agentPatch(cx, { folder: "f", prompt: "", skip_git: false }), { folder: "f", prompt: "", skip_git: false });
  assert.deepEqual(agentPatch({ folder: "f", prompt: "" }, { folder: "f", prompt: "", skip_git: true }), { folder: "f", prompt: "" }, "옛 응답 = 켬");
  for (const t of [...WEB_TOOLS, ...EDIT_TOOLS]) assert.ok(TOOLS_NOTE.includes(t), t);
});

test("conv_info — 대화 정보 줄: 받은 대화 = 기간 · 도구 · 폴더 · 컨텍스트 · 실행 명령, 보낸 · 공유 = 기간 · 컨텍스트(#1z-10)", () => {
  const now = new Date(2026, 9, 23, 18, 0).getTime() / 1000;
  const at = (d, h, m) => new Date(2026, 9, d, h, m).getTime() / 1000;
  const info = {
    created_at: at(21, 10, 50),
    last_at: at(23, 16, 5),
    compacts: 1,
    last_compact_at: at(22, 9, 0),
    clears: 0,
    agent: { engine: "codex", folder: "/work/shop" },
    cmd: { text: "& 'codex.exe' exec resume t -", at: at(23, 16, 4) },
  };
  const mine = convInfoRows(info, "in", now);
  assert.deepEqual(
    mine.map((r) => [r.icon, r.label]),
    [["clock", "기간"], ["bot", "답하는 도구"], ["folder", "읽는 폴더"], ["compress", "컨텍스트"], ["terminal", "실행 명령"]],
  );
  assert.equal(mine[0].text, "10.21 10:50 시작 · 마지막 10.23 16:05");
  assert.equal(mine[1].text, "Codex");
  assert.deepEqual([mine[2].text, mine[2].mono], ["/work/shop", true]);
  assert.equal(mine[3].text, "압축 1회 · 10.22 09:00");
  assert.equal(mine[4].copy, info.cmd.text, "복사 = 전체 명령");
  for (const dir of ["out", "shared"])
    assert.deepEqual(
      convInfoRows({ ...info, agent: null, cmd: null }, dir, now).map((r) => r.label),
      ["기간", "컨텍스트"],
      `${dir} = 기간 · 컨텍스트만`,
    );
  const fresh = convInfoRows({ ...info, compacts: 0, last_compact_at: null, clears: 2, agent: null, cmd: null }, "in", now);
  assert.deepEqual(
    fresh.map((r) => r.text).slice(1),
    ["지운 에이전트", "—", "압축 없음 · 비우기 2회", "아직 실행 없음"],
  );
  assert.equal(fresh.at(-1).copy, undefined, "명령이 없으면 복사 없음");
  // 이 PC 가 자동 왕복을 멈춘 받은 대화 — 컨텍스트 뒤 '자동 왕복' 줄(머리글 · 세부정보 절에는 표시 없음).
  const stopped = convInfoRows({ ...info, relay_blocked: true }, "in", now);
  assert.deepEqual(stopped.map((r) => r.label), ["기간", "답하는 도구", "읽는 폴더", "컨텍스트", "자동 왕복", "실행 명령"]);
  assert.equal(stopped[4].text, RELAY_STOPPED);
  assert.equal(stopped[4].icon, "swap");
  assert.ok(!convInfoRows({ ...info, relay_blocked: true }, "out", now).some((r) => r.label === "자동 왕복"), "보낸 대화에는 없음");
  assert.match(INFO_CMD_HINT, /세션/);
});

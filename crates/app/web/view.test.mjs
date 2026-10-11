// node --test crates/app/web/ — 화면 로직(view.js) 시험. DOM 없음.
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  stateText,
  peerState,
  convLabel,
  filterByPeer,
  authorLabel,
  relayText,
  keepsConversation,
  formatDuration,
  formatTime,
  timeline,
  unlinkedReplies,
  route,
  Latest,
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
    title: "→ 가드 봐 줘",
    sub: "이도윤 · DYOON-MAC 의 review",
  });
  assert.deepEqual(convLabel({ peer_id: 9, dir: "in", agent: "dev", title: "" }, names), {
    title: "← 내 dev 가 받은 요청",
    sub: "알 수 없는 상대 → 내 dev",
  });
});

test("레일 거르기", () => {
  const convs = [{ id: 1, peer_id: 2 }, { id: 2, peer_id: 3 }, { id: 3, peer_id: 2 }];
  assert.deepEqual(filterByPeer(convs, null).map((c) => c.id), [1, 2, 3]);
  assert.deepEqual(filterByPeer(convs, 2).map((c) => c.id), [1, 3]);
  assert.deepEqual(filterByPeer(convs, 7), []);
});

test("레일 전환 — 다른 PC 를 고르면 열린 대화를 닫는다", () => {
  const conv = { id: 5, peer_id: 2 };
  assert.equal(keepsConversation(conv, null), true, "내 PC = 전부");
  assert.equal(keepsConversation(conv, 2), true);
  assert.equal(keepsConversation(conv, 3), false);
  assert.equal(keepsConversation(null, 2), false);
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
  assert.deepEqual(route("#t/3"), { view: "timeline", id: 3 });
  assert.deepEqual(route("#me"), { view: "settings", focus: "me" });
  assert.deepEqual(route("#settings"), { view: "settings", focus: null });
  assert.deepEqual(route("#new"), { view: "new" });
  assert.deepEqual(route(""), { view: "home" });
  assert.deepEqual(route("#c/abc"), { view: "home" });
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

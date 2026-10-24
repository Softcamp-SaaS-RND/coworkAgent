// 동기화 — 마지막 값만 보내기 · 늦은 응답 버리기 · 한 번에 하나 · 변경 신호(SSE `event: change`) → 다시 읽을 것.

/// 마지막 값만 보내기 — 한 번에 하나만 보내고, 보내는 동안 들어온 값은 마지막 것만 이어서 보낸다(동시에 보낸 옛 값이 늦게
/// 닿아 새 값을 덮지 않게). `send` 의 실패는 `send` 가 알린다 — 여기서는 다음 값을 막지 않는다.
export class SendLatest {
  constructor(send) {
    this.send = send;
    this.busy = false;
    this.waiting = false;
    this.next = undefined;
  }
  push(value) {
    if (this.busy) {
      this.waiting = true;
      this.next = value;
      return;
    }
    this.run(value);
  }
  async run(value) {
    this.busy = true;
    try {
      await this.send(value);
    } catch {
      // `send` 가 알린다
    }
    this.busy = false;
    if (this.waiting) {
      this.waiting = false;
      this.run(this.next);
    }
  }
}

/// 늦게 온 응답 판별 — 요청마다 `next()` 로 번호를 받고, 응답 때 `is(번호)` 가 거짓이면(그 뒤 다른 요청이 있었다) 버린다.
export class Latest {
  constructor() {
    this.n = 0;
  }
  next() {
    this.n += 1;
    return this.n;
  }
  is(ticket) {
    return ticket === this.n;
  }
}

/// 대화 변경 한 건이 닿는 로더 전부(자원 단위 — 화면 단위가 아니다). 상대 요청은 접속 표시(마지막 통신 시각)도 바꿔 연결도 넣는다.
export const CONV_LOADERS = ["peers", "groups", "convs", "current", "activity", "requests", "saved", "peer", "runs", "counts"];

/// 바뀐 자원 → 다시 읽을 로더 집합, `null` = 전체(빈 목록 · 모르는 종류 · 연결 변경). 열린 대화 본문은 그 id · id 모름일 때만.
export function loadersFor(res, currentId) {
  if (!Array.isArray(res) || !res.length || res.some((r) => r?.kind !== "conv")) return null;
  const out = new Set(CONV_LOADERS);
  if (currentId == null || !res.some((r) => r.id == null || r.id === currentId)) out.delete("current");
  return out;
}

/// 짧은 간격에 온 신호 합치기 — 처음이면 그대로, 하나라도 전체(빈 · 배열 아님)면 전체(`[]`).
export function mergeChanges(q, res) {
  if (q === undefined) return res;
  if (!Array.isArray(q) || !q.length || !Array.isArray(res) || !res.length) return [];
  return [...q, ...res];
}

/// 도는 중 쌓인 다시 읽기 요청 합치기 — `undefined` = 아직 없음, `null` = 전부(하나라도 전부면 전부), `Set` = 합집합.
export function unionLoaders(pending, only) {
  if (pending === undefined) return only;
  if (pending === null || only === null) return null;
  return new Set([...pending, ...only]);
}

/// 다시 읽기를 한 번에 하나만 돌린다 — 도는 중 온 요청은 `unionLoaders` 로 합쳐 끝난 뒤 한 묶음으로 한 번 더. `call` 이 돌려주는 약속은
/// 자기 요청이 든 묶음의 결과다 — 앞 묶음이 실패해도 쌓인 묶음은 돈다.
export class OneAtATime {
  constructor(run) {
    this.run = run;
    this.busy = null;
    this.queued = null; // { only, promise, resolve, reject }
  }
  call(only) {
    if (!this.busy) return this.start(only);
    if (this.queued) {
      this.queued.only = unionLoaders(this.queued.only, only);
    } else {
      const q = { only };
      q.promise = new Promise((resolve, reject) => Object.assign(q, { resolve, reject }));
      this.queued = q;
    }
    return this.queued.promise;
  }
  start(only) {
    const p = (async () => {
      try {
        await this.run(only);
      } finally {
        this.busy = null;
        const q = this.queued;
        this.queued = null;
        if (q) this.start(q.only).then(q.resolve, q.reject);
      }
    })();
    this.busy = p;
    return p;
  }
}

/// 이 PC 실행 문(`GET /api/runs`) → 실행 상세 '이 PC 실행' 머리 문구 · 줄(실행 중 먼저, 닿은 순서). 줄 = `{label, text, href}` — 누르면 그 대화.
export function runsSummary(r) {
  const row = (x, label) => ({
    label: `${label} · ${x.peer ?? "알 수 없는 상대"}/${x.agent ?? "에이전트"}`,
    text: x.text ?? "",
    href: x.conversation_id != null ? `#c/${x.conversation_id}` : "",
  });
  return {
    title: `실행 중 ${r.running.length} · 대기 ${r.waiting.length}`,
    rows: [...r.running.map((x) => row(x, "실행 중")), ...r.waiting.map((x, i) => row(x, `대기 ${i + 1}`))],
  };
}

// usage: node shots.mjs <port> <theme> <out dir> — #1o 시안 캡처(mock7 가상 데이터): 상대 타일 → 대화 1 열고, 목록 구분 + 말풍선 음영 변형(A · B · C)을 주입해 찍는다.
const [, , port, theme, out] = process.argv;
const { chromium } = await import("playwright-core");
const browser = await chromium.launch({ channel: "msedge", headless: true });
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
const errors = [];
page.on("pageerror", (e) => errors.push("pageerror: " + e.message));
const wait = (ms = 900) => page.waitForTimeout(ms);

const CSS = {
  // A — 섹션 머리 그룹 · 음영 약하게(내 글 = 강조색 14% 층 + 오른쪽 띠, 상대 글 층 12% → 7%)
  A: `
.msg.side-mine .bubble{isolation:isolate;background:var(--surface);color:var(--text);border:1px solid var(--border);border-right:3px solid var(--accent)}
.msg.side-mine .bubble::before{content:"";position:absolute;inset:0;z-index:-1;border-radius:inherit;background:var(--accent);opacity:.14;pointer-events:none}
.msg.side-mine .bubble a{color:var(--accent)}
.msg.side-theirs .bubble::before{opacity:.07}
.grp-head{display:flex;justify-content:space-between;align-items:center;margin:10px 8px 4px;font-size:11px;font-weight:700;letter-spacing:.04em;color:var(--muted,#8A93A6);text-transform:none}
.grp-head b{font-weight:600;color:var(--muted,#8A93A6)}
`,
  // B — 구분선 · 음영 더 약하게(내 글 = 중립 바탕 + 얇은 강조 띠, 상대 글 층 4%)
  B: `
.msg.side-mine .bubble{background:var(--surface-2);color:var(--text);border:1px solid var(--border);border-right:2px solid var(--accent)}
.msg.side-mine .bubble a{color:var(--accent)}
.msg.side-theirs .bubble{border-left-width:2px}
.msg.side-theirs .bubble::before{opacity:.04}
.grp-rule{display:flex;align-items:center;gap:8px;margin:8px 8px;font-size:11px;color:var(--muted,#8A93A6)}
.grp-rule::before,.grp-rule::after{content:"";flex:1;border-top:1px solid var(--border)}
`,
  // C — 탭(세그먼트) · 테두리 위주(바탕 없음 — 내 글 강조색 테두리, 상대 글 타일 색 테두리)
  C: `
.msg.side-mine .bubble{background:transparent;color:var(--text);border:1px solid var(--accent)}
.msg.side-mine .bubble a{color:var(--accent)}
.msg.side-theirs .bubble{background:transparent;border:1px solid var(--tone,var(--wait))}
.msg.side-theirs .bubble::before{display:none}
.grp-seg{display:flex;gap:2px;margin:4px 8px 8px;padding:2px;border:1px solid var(--border);border-radius:8px}
.grp-seg span{flex:1;text-align:center;padding:5px 0;border-radius:6px;font-size:12px;color:var(--muted,#8A93A6)}
.grp-seg span.on{background:var(--surface-2);color:var(--text);font-weight:600}
`,
};

async function split(v) {
  await page.evaluate(async (v) => {
    const convs = await (await fetch("/api/conversations")).json();
    const list = Array.isArray(convs) ? convs : convs.items ?? convs.conversations ?? [];
    const dir = new Map(list.map((c) => [String(c.id), c.dir]));
    const ul = document.querySelector("#conv_sections ul.side-list");
    const lis = [...ul.querySelectorAll(":scope > li[data-conv]")];
    const mine = lis.filter((li) => dir.get(li.dataset.conv) === "out");
    const agent = lis.filter((li) => dir.get(li.dataset.conv) === "in");
    const li = (cls, html) => Object.assign(document.createElement("li"), { className: cls, innerHTML: html });
    if (v === "A") ul.replaceChildren(li("grp-head", `<span>내 대화</span><b>${mine.length}</b>`), ...mine, li("grp-head", `<span>agent 대화</span><b>${agent.length}</b>`), ...agent);
    if (v === "B") ul.replaceChildren(...mine, li("grp-rule", "agent 대화"), ...agent);
    if (v === "C") {
      ul.replaceChildren(...mine);
      const seg = Object.assign(document.createElement("div"), { className: "grp-seg", innerHTML: `<span class="on">내 대화 ${mine.length}</span><span>agent 대화 ${agent.length}</span>` });
      ul.before(seg);
    }
  }, v);
}

for (const v of ["0", "A", "B", "C"]) {
  await page.goto(`http://127.0.0.1:${port}/?t=${"a".repeat(64)}`, { waitUntil: "commit" });
  await wait(2200);
  await page.locator("#rail_pcs .rail-btn").first().click();
  await wait(1000);
  await page.evaluate(() => (location.hash = "#c/1"));
  await wait(1500);
  if (v !== "0") {
    await page.addStyleTag({ content: CSS[v] });
    await split(v);
    await wait(300);
  }
  await page.evaluate(() => document.querySelector(".msg")?.scrollIntoView({ block: "start" }));
  await wait(300);
  await page.mouse.move(1430, 890);
  await page.screenshot({ path: `${out}/${v}_${theme}.jpg`, type: "jpeg", quality: 82 });
}
console.log(errors.length ? errors.join("\n") : "no errors");
await browser.close();

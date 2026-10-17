// 가상 데이터 mock(mock.py) 화면 캡처 — Edge headless 1440×900 @2x. 눈 확인 세트 또는 문서 캡처(README · 사용자 가이드 5장 — 연결 · 에이전트 등록은 설정 카드만).
// usage: node shots.mjs <port> <out dir> <dark|light> [check|doc]
//   playwright-core 위치는 PW 환경 변수(파일 URL)로 — 없으면 "playwright-core" 를 그대로 import 한다.
//   예) python mock.py crates/app/web 8899 dark  →  PW=file:///…/playwright-core/index.mjs node shots.mjs 8899 out dark doc
const [, , port, out, theme = "dark", mode = "check"] = process.argv;
const { chromium } = await import(process.env.PW ?? "playwright-core");
const browser = await chromium.launch({ channel: "msedge", headless: true });
const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 });
const errors = [];
page.on("pageerror", (e) => errors.push("pageerror: " + e.message));
page.on("console", (m) => m.type() === "error" && !m.text().includes("status of 404") && errors.push("console: " + m.text()));
const wait = (ms = 1200) => page.waitForTimeout(ms);
const shot = (name) => page.screenshot({ path: `${out}/${name}${mode === "check" ? `_${theme}` : ""}.png` });
const open = async () => {
  await page.goto(`http://127.0.0.1:${port}/?t=${"a".repeat(64)}`, { waitUntil: "commit" });
  await wait(2200);
};

await open();
await page.evaluate(async (t) => {
  await fetch("/api/settings", { method: "PUT", headers: { "content-type": "application/json" }, body: JSON.stringify({ ui_theme: t }) });
}, theme);
await open();

// 개요('나')
await page.click("#rail_me");
await wait(1200);
await shot("home");

// 대화 — 결제 모듈 배포 절차, 입력창 @ 멘션 후보
await page.locator("#rail_pcs .rail-btn").first().click();
await wait(1000);
await page.locator("#conv_sections").getByText("결제 모듈 배포 절차").first().click();
await wait(1800);
await page.click("#reply_text");
await page.keyboard.type("@");
await wait(800);
await shot("workspace");
await page.keyboard.press("Backspace");

// 그룹 — 결제 모듈(이준호 대화 2 + 최유나 대화 1), 대화 추가 상자
await page.evaluate(async () => {
  const j = { "content-type": "application/json" };
  const g = await (await fetch("/api/groups", { method: "POST", headers: j, body: JSON.stringify({ name: "결제 모듈" }) })).json();
  if (g.id) await fetch(`/api/groups/${g.id}/conversations`, { method: "PUT", headers: j, body: JSON.stringify({ add: [1, 2, 6], remove: [] }) });
});
await open();
await page.locator("#rail_groups .rail-btn").first().click();
await wait(1400);
await page.click("#group_pick_btn");
await wait(800);
await shot("group");
await page.keyboard.press("Escape");

// 에이전트 · 설정 — 문서 캡처는 README 표(연결 · 에이전트 등록)에 맞춰 설정 카드만 자른다(초대 생성 뒤 페어링 + 연결 목록 · 등록 칸을 연 내 에이전트).
if (mode === "doc") {
  await page.evaluate(() => (location.hash = "#settings"));
  await wait(1500);
  await page.click("#window_btn");
  await wait(1200);
  const a = await page.locator("#conn").boundingBox();
  const b = await page.locator("#peers").boundingBox();
  await page.screenshot({ path: `${out}/pairing.png`, clip: { x: a.x, y: a.y, width: a.width, height: b.y + b.height - a.y } });
  await page.click("#agent_new_btn");
  await wait(800);
  await page.locator("section#agents").screenshot({ path: `${out}/agents.png` });
} else {
  await page.evaluate(() => (location.hash = "#agents"));
  await wait(1500);
  await shot("agents");
  await page.evaluate(() => (location.hash = "#settings"));
  await wait(1500);
  await shot("settings");
}

console.log("ERRORS", JSON.stringify(errors));
await browser.close();

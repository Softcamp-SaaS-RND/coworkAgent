// 사용자 가이드 캡처 10장(라이트) — ego-browser 로 mock(port 8899)을 1440×900 에서 찍어 docs/images/ 에 쓴다.
// 실행: python -u mock.py crates/app/web 8899 light  →  ego-browser nodejs < shots-ego.mjs(Git Bash · cmd). 경로는 개발 PC 절대 경로.
// 브라우저 확장이 페이지에 넣는 요소(#ds365-convert-panel-host · #crx-mouse-redesign-content-root)는 캡처 전에 숨긴다.
const task = await taskSpace("coworkAgent guide shots");
const page = task.page("p1");
const OUT = "D:/dev/git/coworkAgent/docs/images/";
await page.cdp("Emulation.setDeviceMetricsOverride", { width: 1440, height: 900, deviceScaleFactor: 2, mobile: false });
await page.goto("http://127.0.0.1:8899/?t=" + "a".repeat(64));
await page.waitForTimeout(2000);
await page.evaluate(() => {
  const s = document.createElement("style");
  s.textContent = "#ds365-convert-panel-host, #crx-mouse-redesign-content-root { display: none !important; }";
  document.documentElement.appendChild(s);
});
const go = async (hash) => { await page.evaluate((h) => { location.hash = h; }, hash); await page.waitForTimeout(1300); };
const shot = (n) => page.screenshot({ path: OUT + n + ".png" });

await go("#"); await shot("home");
await page.click("#rail_groups .rail-btn"); await page.waitForTimeout(1300); await shot("group");
await page.click("#rail_me"); await page.waitForTimeout(800);
await go("#settings/conn"); await page.click("#window_btn"); await page.waitForTimeout(1300); await shot("pairing");
await go("#settings/agents"); await page.click("text=review"); await page.waitForTimeout(1000); await shot("agents");
await go("#new");
await page.selectOption("#chat_peer", { index: 1 }); await page.waitForTimeout(800);
const agents = await page.evaluate(() => [...document.querySelectorAll("#chat_agent option")].map((o) => o.textContent));
await page.selectOption("#chat_agent", { index: 0 }); await page.waitForTimeout(400);
await shot("new-conv");
await page.selectOption("#chat_relay_agent", { label: "review" }); await page.waitForTimeout(500); await shot("relay");
await go("#c/1"); await page.click("#reply_text"); await page.keyboard.type("@"); await page.waitForTimeout(800); await shot("workspace");
await page.keyboard.press("Backspace");
await go("#requests"); await shot("requests");
await go("#settings/notify"); await shot("notify");
await go("#settings/run"); await shot("settings");
const left = await page.evaluate(() => !!document.getElementById("disconnected"));
console.log({ agents, disconnected: left });
